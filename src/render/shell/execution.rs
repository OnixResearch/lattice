use anyhow::Context;

pub(super) fn execute_step(
    request: &crate::render::model::Request,
    batch: &mut dyn super::Batch,
    action: super::Action,
    fault: super::Fault,
    journal: &mut super::Journal,
) -> anyhow::Result<()> {
    assert!(!journal.record.superseded, "a superseded owner must never reach producer execution");
    if journal.record.accepted.is_some() {
        assert_eq!(action, super::Action::Cancel, "accepted computations may only enter explicit cancellation");
    }
    let computation = super::dispatch::PreparedRun::new(request)?;
    super::dispatch::apply(&computation, batch, action, fault, journal)?;
    // Read-only reconciliation is allowed even after deadline; no new Submit,
    // placement, retry or budget reservation is produced here.
    let snapshot = batch.snapshot(computation.run)?;
    crate::render::model::validate_snapshot(request, &computation.prepared, &snapshot)?;
    super::check_current(request)?;
    validate_progress(journal, &snapshot)?;
    journal.record.snapshot = Some(snapshot.clone());
    journal.save()?;
    anyhow::ensure!(fault != super::Fault::AfterObservation, "fixture interruption after retained observation");
    if let Some(outcome) = observed_outcome(journal, &snapshot) {
        if matches!(outcome, crate::render::model::Outcome::Pending | crate::render::model::Outcome::Unknown) {
            super::artifacts::fetch_objects(request, batch, journal, &snapshot)?;
        }
        return Ok(());
    }
    super::artifacts::fetch_objects(request, batch, journal, &snapshot)?;
    super::publication::publish(request, journal, &snapshot, fault)
}

pub(super) fn step(
    request: &crate::render::model::Request,
    batch: &mut dyn super::Batch,
    action: super::Action,
    fault: super::Fault,
) -> anyhow::Result<super::Receipt> {
    request.prepare()?;
    let mut journal = super::Journal::open(request)?;
    if action == super::Action::Supersede {
        return supersede(&mut journal);
    }
    // Stale owners cannot cancel or accept late output; the tombstone is durable.
    if journal.record.superseded {
        return journal.receipt(false);
    }
    if super::check_current(request).is_err() {
        return supersede(&mut journal);
    }
    if journal.record.accepted.is_some() && action != super::Action::Cancel {
        return reused_receipt(request, &journal);
    }
    execute_observed(request, batch, action, fault, &mut journal)
}

fn validate_progress(journal: &super::Journal, snapshot: &molten_batch_core::ProgressSnapshot) -> anyhow::Result<()> {
    if let Some(previous) = &journal.record.snapshot {
        anyhow::ensure!(snapshot.state_revision >= previous.state_revision, "stale progress snapshot");
        anyhow::ensure!(
            snapshot.state_revision != previous.state_revision || *snapshot == *previous,
            "conflicting observation at the same committed revision"
        );
        for (task, value) in &previous.tasks {
            if let molten_batch_core::ProgressTask::Accepted { .. } = value {
                anyhow::ensure!(snapshot.tasks.get(task) == Some(value), "accepted output changed or disappeared");
            }
        }
    }
    Ok(())
}

fn observed_outcome(
    journal: &mut super::Journal,
    snapshot: &molten_batch_core::ProgressSnapshot,
) -> Option<crate::render::model::Outcome> {
    let has_cancellation = journal.record.cancellation_requested || snapshot.cancel_requested;
    let outcome = if has_cancellation {
        journal.record.cancellation_requested = true;
        Some(if snapshot.phase == molten_batch_core::RunPhase::Cancelled {
            crate::render::model::Outcome::Stopped
        } else {
            crate::render::model::Outcome::CancellationRequested
        })
    } else {
        producer_outcome(snapshot)
    };
    if has_cancellation {
        assert!(
            matches!(
                outcome,
                Some(crate::render::model::Outcome::Stopped | crate::render::model::Outcome::CancellationRequested)
            ),
            "cancellation prevents computation classification"
        );
    }
    if outcome.is_none() {
        assert_eq!(
            snapshot.phase,
            molten_batch_core::RunPhase::Succeeded,
            "only producer success may enter publication"
        );
    }
    if let Some(outcome) = outcome {
        journal.record.outcome = outcome;
    }
    outcome
}

fn supersede(journal: &mut super::Journal) -> anyhow::Result<super::Receipt> {
    journal.record.superseded = true;
    journal.record.outcome = crate::render::model::Outcome::Superseded;
    journal.save()?;
    journal.receipt(false)
}

fn reused_receipt(request: &crate::render::model::Request, journal: &super::Journal) -> anyhow::Result<super::Receipt> {
    let accepted = journal.record.accepted.as_ref().context("missing accepted manifest")?;
    let objects = super::artifacts::retained_objects(&journal.root, &accepted.snapshot)?;
    let mut recomputed = crate::render::model::accept(request, &accepted.snapshot, &objects)?;
    recomputed.host_execution = journal.owner()?;
    anyhow::ensure!(recomputed == *accepted, "committed artifacts no longer verify");
    journal.receipt(true)
}

fn execute_observed(
    request: &crate::render::model::Request,
    batch: &mut dyn super::Batch,
    action: super::Action,
    fault: super::Fault,
    journal: &mut super::Journal,
) -> anyhow::Result<super::Receipt> {
    let has_prior_dispatch_intent = journal.record.dispatch_possible;
    let has_prior_cancellation_intent = journal.record.cancellation_requested;
    let start = crate::operation::shell::SystemClock.observe_monotonic_instant();
    let result = execute_step(request, batch, action, fault, journal);
    if has_prior_dispatch_intent {
        assert!(journal.record.dispatch_possible, "possible Submit must never become eligible for redispatch");
    }
    if has_prior_cancellation_intent {
        assert!(journal.record.cancellation_requested, "cancellation intent must remain sticky across errors");
    }
    journal.record.measured_adapter_wall_millis = journal
        .record
        .measured_adapter_wall_millis
        .checked_add(u64::try_from(start.elapsed().as_millis())?)
        .context("adapter timing overflow")?;
    if result.is_err()
        && !matches!(
            journal.record.outcome,
            crate::render::model::Outcome::Superseded | crate::render::model::Outcome::Stopped
        )
    {
        journal.record.outcome = if journal.record.cancellation_requested {
            crate::render::model::Outcome::CancellationRequested
        } else {
            crate::render::model::Outcome::Unknown
        };
    }
    journal.save()?;
    // Errors after possible effects become an explicit durable uncertain receipt.
    // Missing content/transport is never interpreted as failed computation.
    journal.receipt(false)
}

fn producer_outcome(snapshot: &molten_batch_core::ProgressSnapshot) -> Option<crate::render::model::Outcome> {
    if snapshot.phase == molten_batch_core::RunPhase::Failed {
        return Some(crate::render::model::Outcome::Failed);
    }
    if snapshot.phase == molten_batch_core::RunPhase::Succeeded {
        return None;
    }
    let has_uncertain_task = snapshot
        .tasks
        .values()
        .any(|task| matches!(task, molten_batch_core::ProgressTask::Uncertain { .. }));
    Some(if has_uncertain_task {
        crate::render::model::Outcome::Unknown
    } else {
        crate::render::model::Outcome::Pending
    })
}

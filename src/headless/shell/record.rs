//! The persistence sequence shared by fresh and checkpoint-continued snapshot runs.

/// Run facts the snapshot persistence sequence records.
pub(super) struct SnapshotRunFacts {
    pub(super) run_id: crate::engine::executor::RunId,
    pub(super) owner: crate::store::RunOwner,
    pub(super) started_epoch_ms: i64,
}

/// The persistence sequence of one snapshot run: the suspension, or the run record, details and
/// replay events.
pub(super) fn persist_snapshot_outcome(
    store: &crate::store::Store,
    workflow: &crate::engine::workflow::Workflow,
    executor: &mut crate::engine::executor::Executor,
    facts: SnapshotRunFacts,
    outcome: crate::engine::executor::ApprovalRunOutcome,
) -> Result<super::SnapshotRunOutcome, crate::headless::HeadlessRunError> {
    let SnapshotRunFacts {
        run_id,
        owner,
        started_epoch_ms,
    } = facts;
    let observed_completion_order = executor.observed_completion_order().to_vec();
    crate::headless::persist_completed_outputs_to_cache(store, &outcome.results);
    if let Some(suspension) = outcome.suspension {
        let (summary, _request) =
            crate::workflow::gate::shell::persist_suspension(crate::workflow::gate::shell::SuspensionCommitInput {
                store,
                workflow,
                run_id,
                owner,
                started_epoch_millis: started_epoch_ms,
                observed_epoch_millis: crate::headless::run_finished_epoch_ms(),
                results: &outcome.results,
                suspension,
                policy_decisions: executor.take_policy_decisions(),
            })
            .map_err(|error| crate::headless::HeadlessRunError::Store(error.to_string()))?;
        return Ok(super::SnapshotRunOutcome {
            summary,
            cancelled: executor.cancellation_observed(),
        });
    }
    let results = outcome.results;
    let summary = crate::headless::summarize_run(run_id, workflow, &results);
    let finished_epoch_ms = crate::headless::run_finished_epoch_ms();
    let record = super::run_record_from_summary_with_owner(&summary, started_epoch_ms, Some(finished_epoch_ms), owner);
    let mut details = crate::run::history::detail_record_from_results_with_completion_order(
        run_id,
        workflow,
        &results,
        observed_completion_order,
    );
    // r[impl lattice.typed_policy.record]
    crate::workflow::decision::ledger::attach(&mut details, executor.take_policy_decisions());
    debug_assert_eq!(details.run_id, run_id);
    debug_assert_eq!(details.nodes.len(), results.len());
    store.save_run(&record)?;
    // r[impl lattice.workflow_execution.headless_run_details]
    store.save_run_details(&details)?;
    let replay_events = crate::replay::replay_events_from_results(
        run_id,
        workflow,
        &results,
        started_epoch_ms,
        Some(finished_epoch_ms),
    );
    store.save_run_replay_events(run_id, &replay_events)?;

    Ok(super::SnapshotRunOutcome {
        summary,
        cancelled: executor.cancellation_observed(),
    })
}

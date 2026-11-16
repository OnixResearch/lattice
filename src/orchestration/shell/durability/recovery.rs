// r[impl lattice.agent_orchestration.events]
pub fn replay(
    store: &crate::store::Store,
    instance_id: &str,
) -> Result<crate::orchestration::core::ReplayReport, super::super::Error> {
    let current = super::super::load_instance(store, instance_id)?;
    let initial = crate::orchestration::core::new_instance(
        current.definition.clone(),
        current.sequence,
        current.trigger_idempotency_key.clone(),
    )?;
    let events = store.load_orchestration_events(&current.id)?;
    let report = crate::orchestration::core::replay_events(&initial, &events, current.event_head.as_ref());
    if !report.valid || report.instance != current {
        return Err(super::super::Error::ReplayInvalid(Box::new(report)));
    }
    Ok(report)
}

// r[impl lattice.agent_orchestration.durability]
// r[impl lattice.agent_orchestration.nominal_identities.recovery]
pub fn recover_pending(
    store: &crate::store::Store,
    instance_id: &str,
) -> Result<super::super::RecoveryOutcome, super::super::Error> {
    let instance = super::super::load_instance(store, instance_id)?;
    let Some(intent) = instance.pending_dispatch.as_ref() else {
        return Ok(super::super::RecoveryOutcome::NothingPending);
    };
    let Some(record) = store.load_run(intent.run_id)? else {
        return Ok(super::super::RecoveryOutcome::PendingUnobserved { run_id: intent.run_id });
    };
    super::persistence::validate_run_owner(&instance, intent, &record)?;
    if record.finished_epoch_ms.is_none() {
        return Ok(super::super::RecoveryOutcome::PendingUnobserved { run_id: intent.run_id });
    }
    outcome(reconcile_run_record(store, &instance, intent, &record), intent.run_id)
}

fn outcome(
    result: Result<crate::orchestration::core::Instance, super::super::Error>,
    run_id: crate::engine::RunId,
) -> Result<super::super::RecoveryOutcome, super::super::Error> {
    match result {
        Ok(reconciled) => Ok(super::super::RecoveryOutcome::Reconciled(Box::new(reconciled))),
        Err(super::super::Error::CancellationObservationUnknown { .. }) => {
            Ok(super::super::RecoveryOutcome::PendingUnobserved { run_id })
        }
        Err(error) => Err(error),
    }
}

pub(super) fn reconcile_run_record(
    store: &crate::store::Store,
    instance: &crate::orchestration::core::Instance,
    intent: &crate::orchestration::core::DispatchIntent,
    record: &crate::store::RunRecord,
) -> Result<crate::orchestration::core::Instance, super::super::Error> {
    let request = finish_request(store, instance, intent, record)?;
    let action = super::settlement::finish_action(store, instance, request)?;
    super::persistence::apply_and_commit(store, instance, action)
}

fn finish_request(
    store: &crate::store::Store,
    instance: &crate::orchestration::core::Instance,
    intent: &crate::orchestration::core::DispatchIntent,
    record: &crate::store::RunRecord,
) -> Result<super::settlement::FinishRequest, super::super::Error> {
    let is_succeeded = record.failures == 0;
    let is_retryable_observed_failure =
        !is_succeeded && record.finished_epoch_ms.is_some() && instance.definition.retry.is_some();
    let guarded_summary = crate::guarded::status::optional(store, intent.run_id, &instance.definition.id)?;
    let summary = guarded_summary.unwrap_or_else(|| crate::guarded::status::recovered_fallback(record));
    Ok(super::settlement::FinishRequest {
        dispatch_id: intent.id.clone(),
        run_id: intent.run_id,
        status: crate::orchestration::usage::RunOutcomeObservation {
            succeeded: is_succeeded,
            cancelled: None,
        },
        is_retryable_observed_failure,
        safe_summary: summary,
    })
}

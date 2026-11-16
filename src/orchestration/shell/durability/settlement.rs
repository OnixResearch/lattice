mod receipt;

pub(in crate::orchestration::shell) struct FinishRequest {
    pub dispatch_id: crate::orchestration::DispatchId,
    pub run_id: crate::engine::RunId,
    pub status: crate::orchestration::usage::RunOutcomeObservation,
    pub is_retryable_observed_failure: bool,
    pub safe_summary: String,
}

fn reserved_charge(
    instance: &crate::orchestration::core::Instance,
    dispatch_id: &crate::orchestration::DispatchId,
) -> Result<crate::orchestration::core::BudgetCharge, super::super::Error> {
    let reservation_id = crate::orchestration::ReservationId::from_dispatch(dispatch_id);
    instance
        .budgets
        .reservations
        .get(&reservation_id)
        .map(|reservation| reservation.charge)
        .ok_or_else(|| super::super::Error::InvalidProposal("pending dispatch has no budget reservation".to_string()))
}

pub(in crate::orchestration::shell) fn accounting_charge(
    reserved: crate::orchestration::core::BudgetCharge,
    interval: Option<&crate::orchestration::usage::HostInterval>,
    reports: &[crate::orchestration::usage::ProviderReport],
    policy: &crate::orchestration::core::BudgetPolicy,
) -> crate::orchestration::core::BudgetCharge {
    let mut charged = reserved;
    if policy.enforced.duration_millis
        && let Some(duration) = interval.and_then(crate::orchestration::usage::HostInterval::duration_millis)
    {
        charged.duration_millis = duration;
    }
    // Partial/estimated evidence can increase consumption but cannot refund the
    // unobserved remainder. Saturation fails conservatively at the ledger bound.
    if policy.enforced.tokens {
        let known = reports.iter().filter_map(|report| report.tokens).fold(0u64, u64::saturating_add);
        charged.tokens = charged.tokens.max(known);
    }
    // The adapter exposes no currency/pricing identity: retain reported cost but
    // do not interpret its arbitrary microunits as the admitted monetary unit.
    charged
}

// r[impl lattice.agent_orchestration.usage]
// r[impl lattice.agent_orchestration.usage.scenario.recovery]
// r[impl lattice.agent_orchestration.usage.scenario.bindings]
pub(in crate::orchestration::shell) fn finish_action(
    store: &crate::store::Store,
    instance: &crate::orchestration::core::Instance,
    request: FinishRequest,
) -> Result<crate::orchestration::core::Action, super::super::Error> {
    let reserved = reserved_charge(instance, &request.dispatch_id)?;
    let mut observation = match store.load_command_worker_usage_for_run(request.run_id)? {
        Some(observation) => observation,
        None => receipt::host_observation(store, instance, &request, reserved)?,
    };
    let accounting = validate_accounting_receipt(store, instance, &request, &mut observation, reserved)?;
    accounting
        .refine_outcome(request.status)
        .map_err(|error| super::super::Error::InvalidProposal(error.to_string()))?;
    // Usage is frozen; only absent owned cancellation evidence may become known.
    // The store merges this atomically against any concurrent recovery writer.
    store.save_command_worker_usage(&observation)?;
    let accounting = settled_accounting(store, request.run_id)?;
    observed_action(instance, request, accounting)
}

fn settled_accounting(
    store: &crate::store::Store,
    run_id: crate::engine::RunId,
) -> Result<crate::orchestration::usage::RunAccounting, super::super::Error> {
    store
        .load_command_worker_usage_for_run(run_id)?
        .and_then(|observation| observation.accounting)
        .ok_or(super::super::Error::RunIdentityMismatch { run_id })
}

fn observed_action(
    instance: &crate::orchestration::core::Instance,
    request: FinishRequest,
    accounting: crate::orchestration::usage::RunAccounting,
) -> Result<crate::orchestration::core::Action, super::super::Error> {
    let status = accounting
        .outcome
        .ok_or(super::super::Error::CancellationObservationUnknown { run_id: request.run_id })?;
    let is_retryable_failure = request.is_retryable_observed_failure && status.cancelled == Some(false);
    Ok(crate::orchestration::core::Action::RunFinishedObserved {
        dispatch_id: request.dispatch_id,
        run_id: request.run_id,
        succeeded: status.succeeded,
        cancelled: status
            .cancelled
            .ok_or(super::super::Error::CancellationObservationUnknown { run_id: request.run_id })?,
        retryable_observed_failure: is_retryable_failure && !status.succeeded && !instance.cancellation_requested,
        observed_usage: accounting.charged,
        usage_basis: crate::orchestration::core::BudgetUsageBasis::ConservativeObservation,
        safe_summary: request.safe_summary,
    })
}

fn validate_accounting_receipt<'a>(
    store: &crate::store::Store,
    instance: &crate::orchestration::core::Instance,
    request: &FinishRequest,
    observation: &'a mut crate::orchestration::usage::CommandWorkerObservation,
    reserved: crate::orchestration::core::BudgetCharge,
) -> Result<&'a mut crate::orchestration::usage::RunAccounting, super::super::Error> {
    validate_receipt_identity(instance, request, observation)?;
    let is_host_attempt = observation.scope == crate::orchestration::usage::Scope::HostRun
        && observation.attempt_id == request.dispatch_id.as_str();
    let accounting = require_host_accounting(observation)?;
    if accounting.reserved != reserved {
        return Err(super::super::Error::InvalidProposal("usage reservation binding mismatch".to_string()));
    }
    if !is_host_attempt || !has_bound_charge(accounting, &instance.definition.budgets, reserved) {
        return Err(super::super::Error::InvalidProposal("host accounting charge binding mismatch".to_string()));
    }
    if let Some(interval) = &accounting.host_interval {
        validate_host_interval(store, instance, request.run_id, interval)?;
    }
    Ok(accounting)
}

fn has_bound_charge(
    accounting: &crate::orchestration::usage::RunAccounting,
    policy: &crate::orchestration::core::BudgetPolicy,
    reserved: crate::orchestration::core::BudgetCharge,
) -> bool {
    accounting.charged
        == accounting_charge(reserved, accounting.host_interval.as_ref(), &accounting.provider_reports, policy)
}

fn require_host_accounting(
    observation: &mut crate::orchestration::usage::CommandWorkerObservation,
) -> Result<&mut crate::orchestration::usage::RunAccounting, super::super::Error> {
    // Legacy worker assertions cannot replace durable host-owned settlement
    // evidence, and are not silently overwritten or coerced into measurements.
    observation.accounting.as_mut().ok_or_else(|| super::super::Error::InvalidProposal(
        "unqualified usage observation conflicts with host accounting; reconcile the exact receipt before settlement".to_string(),
    ))
}

fn validate_receipt_identity(
    instance: &crate::orchestration::core::Instance,
    request: &FinishRequest,
    observation: &crate::orchestration::usage::CommandWorkerObservation,
) -> Result<(), super::super::Error> {
    let is_dispatch_match =
        observation.run_id == request.run_id.0 && observation.dispatch_id == request.dispatch_id.as_str();
    let is_revision_match = observation.instance_id == instance.id.as_str()
        && observation.workflow_revision == instance.workflow_revision.content_hash.as_str();
    if !is_dispatch_match || !is_revision_match {
        return Err(super::super::Error::RunIdentityMismatch { run_id: request.run_id });
    }
    Ok(())
}

fn validate_host_interval(
    store: &crate::store::Store,
    instance: &crate::orchestration::core::Instance,
    run_id: crate::engine::RunId,
    interval: &crate::orchestration::usage::HostInterval,
) -> Result<(), super::super::Error> {
    let record = store.load_run(run_id)?.ok_or(super::super::Error::RunIdentityMismatch { run_id })?;
    let intent = instance
        .pending_dispatch
        .as_ref()
        .ok_or_else(|| super::super::Error::NoPendingDispatch(instance.id.to_string()))?;
    super::persistence::validate_run_owner(instance, intent, &record)?;
    if record.started_epoch_ms != interval.started_epoch_ms
        || record.finished_epoch_ms != Some(interval.finished_epoch_ms)
    {
        return Err(super::super::Error::RunIdentityMismatch { run_id });
    }
    Ok(())
}

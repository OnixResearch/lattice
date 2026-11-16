#[derive(Debug, Clone, Copy)]
pub(in crate::orchestration::core) struct RunFinishedInput<'a> {
    pub(in crate::orchestration::core) dispatch_id: &'a crate::orchestration::identity::DispatchId,
    pub(in crate::orchestration::core) run_id: crate::engine::RunId,
    pub(in crate::orchestration::core) is_succeeded: bool,
    /// None preserves historical event replay; Some is the current protocol.
    pub(in crate::orchestration::core) observed_cancelled: Option<bool>,
    pub(in crate::orchestration::core) is_retryable_observed_failure: bool,
    pub(in crate::orchestration::core) observed_usage: super::super::BudgetCharge,
    pub(in crate::orchestration::core) usage_basis: super::super::BudgetUsageBasis,
    pub(in crate::orchestration::core) safe_summary: &'a str,
}

// r[impl lattice.agent_orchestration.observed_failure_retry]
// r[impl lattice.agent_orchestration.usage.scenario.cancellation]
pub(in crate::orchestration::core) fn finish_run(
    instance: &mut super::super::Instance,
    input: RunFinishedInput<'_>,
) -> Result<super::super::EventKind, super::super::Diagnostic> {
    validate_run_completion(instance, &input)?;
    if input.is_succeeded && input.is_retryable_observed_failure {
        return Err(super::super::model::failures::diagnostic(
            super::super::DiagnosticCode::RetryEvidenceMissing,
            "successful run cannot be marked as observed retryable failure",
        ));
    }
    let reservation_id = crate::orchestration::identity::ReservationId::from_dispatch(input.dispatch_id);
    let settlement = super::super::settle_budget(&mut instance.budgets, &reservation_id, input.observed_usage)?;
    instance.last_usage_basis = input.usage_basis;
    instance.active_run_id = None;
    instance.pending_dispatch = None;
    instance.retry.retryable_failure = None;
    instance.terminal_summary =
        instance.definition.redaction.retain_safe_summaries.then(|| input.safe_summary.to_string());
    let kind = if input.is_succeeded {
        super::super::EventKind::RoundCompleted
    } else {
        super::super::EventKind::RoundFailed
    };
    if input.observed_cancelled == Some(true) {
        instance.status = super::super::Status::Cancelled;
        return Ok(kind);
    }
    if settlement.breached {
        instance.status = super::super::Status::Exhausted;
        return Ok(kind);
    }
    if input.observed_cancelled.unwrap_or(instance.cancellation_requested) {
        instance.status = super::super::Status::Cancelled;
        return Ok(kind);
    }
    if !input.is_succeeded {
        instance.status = super::super::Status::Failed;
        if input.is_retryable_observed_failure
            && instance.definition.retry.is_some()
            && (input.observed_cancelled.is_none() || !instance.cancellation_requested)
        {
            instance.retry.retryable_failure = Some(super::super::RetryableFailure {
                run_id: input.run_id,
                dispatch_id: input.dispatch_id.clone(),
            });
        }
        debug_assert!(instance.active_run_id.is_none());
        debug_assert!(instance.pending_dispatch.is_none());
        return Ok(kind);
    }
    let next_status = match instance.definition.completion {
        super::super::CompletionPolicy::RunSucceeded => super::super::Status::Completed,
        super::super::CompletionPolicy::AwaitProposal => super::super::Status::WaitingForApproval,
    };
    if instance.pause_requested && !super::super::is_terminal_status(next_status) {
        instance.paused_status = Some(next_status);
        instance.status = super::super::Status::Paused;
    } else {
        instance.status = next_status;
    }
    debug_assert!(instance.active_run_id.is_none());
    debug_assert!(instance.pending_dispatch.is_none());
    Ok(kind)
}

pub(super) fn validate_run_completion(
    instance: &super::super::Instance,
    input: &RunFinishedInput<'_>,
) -> Result<(), super::super::Diagnostic> {
    validate_observed_cancellation(instance, input)?;
    let dispatch = super::super::lifecycle::routing::matching_dispatch(instance, input.dispatch_id)?;
    if dispatch.run_id != input.run_id || instance.active_run_id != Some(input.run_id) {
        return Err(super::super::model::failures::diagnostic(
            super::super::DiagnosticCode::DispatchMismatch,
            "run completion does not match the active dispatch identity",
        ));
    }
    Ok(())
}

pub(super) fn validate_observed_cancellation(
    instance: &super::super::Instance,
    input: &RunFinishedInput<'_>,
) -> Result<(), super::super::Diagnostic> {
    if let Some(cancelled) = input.observed_cancelled {
        if instance.status != super::super::Status::Running {
            return Err(super::super::model::failures::invalid_transition(
                "observed completion requires an active owned run",
            ));
        }
        if cancelled && (input.is_succeeded || input.is_retryable_observed_failure) {
            return Err(super::super::model::failures::invalid_transition(
                "observed cancellation contradicts success or retryable failure",
            ));
        }
    }
    Ok(())
}

pub(in crate::orchestration::core) fn observe_cancellation(
    instance: &mut super::super::Instance,
    run_id: crate::engine::RunId,
    is_cancelled: bool,
) -> Result<super::super::EventKind, super::super::Diagnostic> {
    if !instance.cancellation_requested || instance.active_run_id != Some(run_id) {
        return Err(super::super::model::failures::invalid_transition(
            "cancellation observation does not match an active cancellation request",
        ));
    }
    if is_cancelled {
        instance.status = super::super::Status::Cancelled;
        instance.active_run_id = None;
        instance.pending_dispatch = None;
    }
    Ok(super::super::EventKind::CancellationObserved)
}

// r[impl lattice.agent_orchestration.unobserved_abandon]
pub(in crate::orchestration::core) fn abandon_pending_dispatch(
    instance: &mut super::super::Instance,
    actor: &str,
    dispatch_id: &crate::orchestration::identity::DispatchId,
    run_id: crate::engine::RunId,
) -> Result<super::super::EventKind, super::super::Diagnostic> {
    super::super::lifecycle::review::authorize_actor(instance, actor)?;
    if !instance.cancellation_requested {
        return Err(super::super::model::failures::diagnostic(
            super::super::DiagnosticCode::CancellationRequired,
            "pending dispatch abandonment requires a prior cancellation request",
        ));
    }
    let dispatch = super::super::lifecycle::routing::matching_dispatch(instance, dispatch_id)?;
    if dispatch.run_id != run_id || instance.active_run_id != Some(run_id) {
        return Err(super::super::model::failures::diagnostic(
            super::super::DiagnosticCode::DispatchMismatch,
            "abandonment does not match the active dispatch and run identities",
        ));
    }
    let dispatch_status = dispatch.status;
    let observed_usage = match dispatch_status {
        super::super::DispatchStatus::Pending => super::super::BudgetCharge {
            rounds: 0,
            agent_invocations: 0,
            tokens: 0,
            monetary_microunits: 0,
            duration_millis: 0,
        },
        super::super::DispatchStatus::Acknowledged => {
            instance
                .budgets
                .reservations
                .get(&crate::orchestration::identity::ReservationId::from_dispatch(dispatch_id))
                .ok_or_else(|| {
                    super::super::model::failures::diagnostic(
                        super::super::DiagnosticCode::MissingReservation,
                        "acknowledged abandonment is missing its conservative budget reservation",
                    )
                })?
                .charge
        }
    };
    let reservation_id = crate::orchestration::identity::ReservationId::from_dispatch(dispatch_id);
    let settlement = super::super::settle_budget(&mut instance.budgets, &reservation_id, observed_usage)?;
    instance.last_usage_basis = super::super::BudgetUsageBasis::ReservedFallback;
    let safe_summary = match dispatch_status {
        super::super::DispatchStatus::Pending => super::super::ABANDONED_PENDING_SUMMARY,
        super::super::DispatchStatus::Acknowledged => super::super::ABANDONED_ACKNOWLEDGED_SUMMARY,
    };
    instance.active_run_id = None;
    instance.pending_dispatch = None;
    instance.pause_requested = false;
    instance.paused_status = None;
    instance.cancellation_requested = false;
    instance.retry.retryable_failure = None;
    instance.status = super::super::Status::Failed;
    instance.terminal_summary = instance.definition.redaction.retain_safe_summaries.then(|| safe_summary.to_string());
    debug_assert!(!settlement.breached);
    debug_assert!(instance.active_run_id.is_none());
    debug_assert!(instance.pending_dispatch.is_none());
    Ok(super::super::EventKind::DispatchAbandoned)
}

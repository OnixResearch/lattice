pub(in crate::orchestration::core) fn start_round(
    instance: &mut super::super::Instance,
    intent: &super::super::DispatchIntent,
    planned_charge: super::super::BudgetChargePlan,
) -> Result<super::super::EventKind, super::super::Diagnostic> {
    if instance.status != super::super::Status::Ready || instance.pause_requested {
        return Err(super::super::model::failures::invalid_transition(
            "a round can start only from an unpaused ready instance",
        ));
    }
    begin_dispatch(instance, intent, planned_charge)?;
    Ok(super::super::EventKind::RoundStarted)
}

// r[impl lattice.agent_orchestration.retry_round]
pub(in crate::orchestration::core) fn start_retry_round(
    instance: &mut super::super::Instance,
    actor: &str,
    intent: &super::super::DispatchIntent,
    planned_charge: super::super::BudgetChargePlan,
) -> Result<super::super::EventKind, super::super::Diagnostic> {
    let failed = preflight_retry(instance, actor, planned_charge)?;
    if intent.run_id == failed.run_id || intent.id == failed.dispatch_id {
        return Err(super::super::model::failures::diagnostic(
            super::super::DiagnosticCode::DispatchMismatch,
            "retry must use run and dispatch identities distinct from the observed failure",
        ));
    }
    if dispatch_identity(instance, intent.run_id)? != intent.id {
        return Err(super::super::model::failures::diagnostic(
            super::super::DiagnosticCode::InvalidIdentity,
            "retry dispatch BLAKE3 identity does not match its canonical inputs",
        ));
    }
    begin_dispatch(instance, intent, planned_charge)?;
    instance.retry.admitted = instance.retry.admitted.saturating_add(super::super::RETRY_INCREMENT);
    instance.retry.retryable_failure = None;
    instance.terminal_summary = None;
    debug_assert!(instance.retry.admitted > 0);
    debug_assert!(instance.retry.retryable_failure.is_none());
    Ok(super::super::EventKind::RetryRoundStarted)
}

// r[impl lattice.agent_orchestration.retry_round]
pub fn preflight_retry(
    instance: &super::super::Instance,
    actor: &str,
    planned_charge: super::super::BudgetChargePlan,
) -> Result<super::super::RetryableFailure, super::super::Diagnostic> {
    let failed = retryable_failure(instance, actor)?;
    let mut budget = instance.budgets.clone();
    let preflight_id =
        crate::orchestration::identity::ReservationId::try_from_text(super::super::RETRY_PREFLIGHT_RESERVATION_ID)
            .map_err(super::super::model::failures::identity_error)?;
    super::super::reserve_budget(&mut budget, preflight_id, planned_charge)?;
    Ok(failed)
}

pub(super) fn retryable_failure(
    instance: &super::super::Instance,
    actor: &str,
) -> Result<super::super::RetryableFailure, super::super::Diagnostic> {
    super::review::authorize_actor(instance, actor)?;
    if instance.status != super::super::Status::Failed
        || instance.active_run_id.is_some()
        || instance.pending_dispatch.is_some()
    {
        return Err(super::super::model::failures::invalid_transition(
            "retry requires a terminal failed instance without an active or pending dispatch",
        ));
    }
    let policy = instance.definition.retry.ok_or_else(|| {
        super::super::model::failures::diagnostic(
            super::super::DiagnosticCode::RetryDisabled,
            "orchestration definition does not enable observed-failure retries",
        )
    })?;
    if instance.retry.admitted >= policy.max_observed_failure_retries {
        return Err(super::super::model::failures::diagnostic(
            super::super::DiagnosticCode::RetryLimitExceeded,
            "observed-failure retry limit is exhausted",
        ));
    }
    let failed = instance.retry.retryable_failure.clone().ok_or_else(|| {
        super::super::model::failures::diagnostic(
            super::super::DiagnosticCode::RetryEvidenceMissing,
            "failed orchestration has no exact persisted terminal-failure retry evidence",
        )
    })?;
    debug_assert_eq!(instance.status, super::super::Status::Failed);
    debug_assert!(instance.retry.admitted < policy.max_observed_failure_retries);
    Ok(failed)
}

pub(super) fn begin_dispatch(
    instance: &mut super::super::Instance,
    intent: &super::super::DispatchIntent,
    planned_charge: super::super::BudgetChargePlan,
) -> Result<(), super::super::Diagnostic> {
    if intent.workflow_revision != instance.workflow_revision {
        return Err(super::super::model::failures::diagnostic(
            super::super::DiagnosticCode::WorkflowRevisionMismatch,
            "dispatch intent workflow revision does not match the active orchestration revision",
        ));
    }
    if intent.status != super::super::DispatchStatus::Pending {
        return Err(super::super::model::failures::diagnostic(
            super::super::DiagnosticCode::InvalidLifecycleTransition,
            "new dispatch intent must begin in pending state",
        ));
    }
    super::super::reserve_budget(
        &mut instance.budgets,
        crate::orchestration::identity::ReservationId::from_dispatch(&intent.id),
        planned_charge,
    )?;
    instance.round = instance.round.saturating_add(super::super::ROUND_INCREMENT);
    instance.active_run_id = Some(intent.run_id);
    instance.pending_dispatch = Some(intent.clone());
    instance.status = super::super::Status::Running;
    instance.cancellation_requested = false;
    debug_assert_eq!(instance.active_run_id, Some(intent.run_id));
    debug_assert_eq!(instance.pending_dispatch.as_ref(), Some(intent));
    Ok(())
}

pub(in crate::orchestration::core) fn acknowledge_dispatch(
    instance: &mut super::super::Instance,
    dispatch_id: &crate::orchestration::identity::DispatchId,
) -> Result<super::super::EventKind, super::super::Diagnostic> {
    let dispatch = matching_dispatch_mut(instance, dispatch_id)?;
    dispatch.status = super::super::DispatchStatus::Acknowledged;
    Ok(super::super::EventKind::DispatchAcknowledged)
}

pub(in crate::orchestration::core) fn matching_dispatch<'a>(
    instance: &'a super::super::Instance,
    dispatch_id: &crate::orchestration::identity::DispatchId,
) -> Result<&'a super::super::DispatchIntent, super::super::Diagnostic> {
    instance.pending_dispatch.as_ref().filter(|intent| &intent.id == dispatch_id).ok_or_else(|| {
        super::super::model::failures::diagnostic(
            super::super::DiagnosticCode::DispatchMismatch,
            "dispatch identity does not match the pending orchestration dispatch",
        )
    })
}

pub(super) fn matching_dispatch_mut<'a>(
    instance: &'a mut super::super::Instance,
    dispatch_id: &crate::orchestration::identity::DispatchId,
) -> Result<&'a mut super::super::DispatchIntent, super::super::Diagnostic> {
    instance.pending_dispatch.as_mut().filter(|intent| &intent.id == dispatch_id).ok_or_else(|| {
        super::super::model::failures::diagnostic(
            super::super::DiagnosticCode::DispatchMismatch,
            "dispatch identity does not match the pending orchestration dispatch",
        )
    })
}

pub fn dispatch_identity(
    instance: &super::super::Instance,
    run_id: crate::engine::RunId,
) -> Result<crate::orchestration::identity::DispatchId, super::super::Diagnostic> {
    Ok(crate::orchestration::identity::DispatchId::from_generated(
        super::super::model::canonical::hash_serializable(
            super::super::INSTANCE_HASH_LABEL,
            &(
                instance.id.as_str(),
                instance.epoch,
                instance.round.saturating_add(super::super::ROUND_INCREMENT),
                instance.workflow_revision.content_hash.as_str(),
                run_id,
            ),
        )?,
    ))
}

pub(in crate::orchestration::core) fn pause(
    instance: &mut super::super::Instance,
    actor: &str,
) -> Result<super::super::EventKind, super::super::Diagnostic> {
    super::review::authorize_actor(instance, actor)?;
    if instance.pause_requested || instance.status == super::super::Status::Paused {
        return Err(super::super::model::failures::invalid_transition("orchestration instance is already paused"));
    }
    instance.pause_requested = true;
    if instance.status != super::super::Status::Running {
        instance.paused_status = Some(instance.status);
        instance.status = super::super::Status::Paused;
    }
    Ok(super::super::EventKind::Paused)
}

pub(in crate::orchestration::core) fn resume(
    instance: &mut super::super::Instance,
    actor: &str,
) -> Result<super::super::EventKind, super::super::Diagnostic> {
    super::review::authorize_actor(instance, actor)?;
    if !instance.pause_requested {
        return Err(super::super::model::failures::invalid_transition("orchestration instance is not paused"));
    }
    instance.pause_requested = false;
    if instance.status == super::super::Status::Paused {
        instance.status = instance.paused_status.take().ok_or_else(|| {
            super::super::model::failures::diagnostic(
                super::super::DiagnosticCode::InvalidLifecycleTransition,
                "paused instance is missing its prior lifecycle state",
            )
        })?;
    }
    Ok(super::super::EventKind::Resumed)
}

pub(in crate::orchestration::core) fn cancel(
    instance: &mut super::super::Instance,
    actor: &str,
) -> Result<super::super::EventKind, super::super::Diagnostic> {
    super::review::authorize_actor(instance, actor)?;
    instance.cancellation_requested = true;
    if instance.status != super::super::Status::Running {
        instance.status = super::super::Status::Cancelled;
    }
    Ok(super::super::EventKind::CancellationRequested)
}

pub(in crate::orchestration::core) fn exhaust_budget(
    instance: &mut super::super::Instance,
    safe_reason: &str,
) -> Result<super::super::EventKind, super::super::Diagnostic> {
    if instance.status == super::super::Status::Failed {
        validate_retry_exhaustion(instance)?;
    }
    if instance.status == super::super::Status::Running {
        return Err(super::super::model::failures::invalid_transition(
            "budget exhaustion cannot erase an active runner observation",
        ));
    }
    instance.status = super::super::Status::Exhausted;
    instance.terminal_summary = instance.definition.redaction.retain_safe_summaries.then(|| safe_reason.to_string());
    Ok(super::super::EventKind::BudgetExhausted)
}

fn validate_retry_exhaustion(instance: &super::super::Instance) -> Result<(), super::super::Diagnostic> {
    validate_settled_retry(instance)?;
    let actor = instance.definition.authority.approvers.iter().next().ok_or_else(|| {
        super::super::model::failures::invalid_transition("failed-round exhaustion requires an authorized operator")
    })?;
    let is_exhausted =
        super::super::preflight_retry(instance, actor, retry_exhaustion_plan(instance)).is_err_and(is_retry_exhaustion);
    if !is_exhausted {
        return Err(super::super::model::failures::invalid_transition(
            "failed round still has retry capacity or lacks exhaustion evidence",
        ));
    }
    Ok(())
}

fn is_retry_exhaustion(error: super::super::Diagnostic) -> bool {
    matches!(
        error.code,
        super::super::DiagnosticCode::RetryLimitExceeded
            | super::super::DiagnosticCode::BudgetExceeded
            | super::super::DiagnosticCode::MissingBudgetBound
    )
}

fn validate_settled_retry(instance: &super::super::Instance) -> Result<(), super::super::Diagnostic> {
    if instance.active_run_id.is_some()
        || instance.pending_dispatch.is_some()
        || instance.retry.retryable_failure.is_none()
    {
        return Err(super::super::model::failures::invalid_transition(
            "failed-round exhaustion requires exact settled retry evidence",
        ));
    }
    Ok(())
}

fn retry_exhaustion_plan(instance: &super::super::Instance) -> super::super::BudgetChargePlan {
    let configured = instance.definition.budgets.per_round_upper_bound;
    super::super::BudgetChargePlan {
        rounds: Some(configured.rounds),
        agent_invocations: Some(configured.agent_invocations),
        tokens: Some(configured.tokens),
        monetary_microunits: Some(configured.monetary_microunits),
        duration_millis: Some(configured.duration_millis),
    }
}

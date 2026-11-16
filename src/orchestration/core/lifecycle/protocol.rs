#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum Action {
    StartRound {
        intent: super::super::DispatchIntent,
        planned_charge: super::super::BudgetChargePlan,
    },
    RetryRound {
        actor: String,
        intent: super::super::DispatchIntent,
        planned_charge: super::super::BudgetChargePlan,
    },
    DispatchAcknowledged {
        dispatch_id: crate::orchestration::identity::DispatchId,
    },
    /// Current completion protocol: settle the exact reservation before closing
    /// ownership, using observed termination rather than a cancellation request.
    RunFinishedObserved {
        dispatch_id: crate::orchestration::identity::DispatchId,
        run_id: crate::engine::RunId,
        succeeded: bool,
        cancelled: bool,
        retryable_observed_failure: bool,
        observed_usage: super::super::BudgetCharge,
        usage_basis: super::super::BudgetUsageBasis,
        safe_summary: String,
    },
    /// Historical event decoding/replay only; production emits RunFinishedObserved.
    RunFinished {
        dispatch_id: crate::orchestration::identity::DispatchId,
        run_id: crate::engine::RunId,
        succeeded: bool,
        #[serde(default = "default_retryable_failure", skip_serializing_if = "is_false")]
        retryable_observed_failure: bool,
        observed_usage: super::super::BudgetCharge,
        safe_summary: String,
    },
    /// Historical event decoding/replay only; production emits RunFinishedObserved.
    RunFinishedMeasured {
        dispatch_id: crate::orchestration::identity::DispatchId,
        run_id: crate::engine::RunId,
        succeeded: bool,
        #[serde(default = "default_retryable_failure", skip_serializing_if = "is_false")]
        retryable_observed_failure: bool,
        measured_usage: super::super::BudgetCharge,
        #[serde(
            default = "default_measured_basis",
            skip_serializing_if = "super::super::BudgetUsageBasis::is_measured"
        )]
        usage_basis: super::super::BudgetUsageBasis,
        safe_summary: String,
    },
    SubmitProposal {
        proposal: super::super::RevisionProposal,
    },
    DecideProposal {
        approval: super::super::ApprovalDecision,
        successor_validated: bool,
    },
    RejectProposal {
        actor: String,
        proposal_id: crate::orchestration::identity::ProposalId,
    },
    PauseRequested {
        actor: String,
    },
    ResumeRequested {
        actor: String,
    },
    CancelRequested {
        actor: String,
    },
    /// Historical event decoding/replay only; cancellation is settled atomically
    /// by RunFinishedObserved in the current protocol.
    CancellationObserved {
        run_id: crate::engine::RunId,
        cancelled: bool,
    },
    AbandonPendingDispatch {
        actor: String,
        dispatch_id: crate::orchestration::identity::DispatchId,
        run_id: crate::engine::RunId,
    },
    BudgetExhausted {
        safe_reason: String,
    },
}

pub(super) fn is_false(value: &bool) -> bool {
    !*value
}

const fn default_measured_basis() -> super::super::BudgetUsageBasis {
    super::super::BudgetUsageBasis::Measured
}

const fn default_retryable_failure() -> bool {
    false
}

pub(in crate::orchestration::core) fn apply_action(
    instance: &mut super::super::Instance,
    action: &Action,
) -> Result<super::super::EventKind, super::super::Diagnostic> {
    match action {
        Action::StartRound { intent, planned_charge } => super::routing::start_round(instance, intent, *planned_charge),
        Action::RetryRound {
            actor,
            intent,
            planned_charge,
        } => super::routing::start_retry_round(instance, actor, intent, *planned_charge),
        Action::DispatchAcknowledged { dispatch_id } => super::routing::acknowledge_dispatch(instance, dispatch_id),
        Action::RunFinishedObserved {
            dispatch_id,
            run_id,
            succeeded,
            retryable_observed_failure,
            observed_usage,
            safe_summary,
            ..
        }
        | Action::RunFinished {
            dispatch_id,
            run_id,
            succeeded,
            retryable_observed_failure,
            observed_usage,
            safe_summary,
        }
        | Action::RunFinishedMeasured {
            dispatch_id,
            run_id,
            succeeded,
            retryable_observed_failure,
            measured_usage: observed_usage,
            safe_summary,
            ..
        } => {
            super::super::ledger::settlement::finish_run(instance, super::super::ledger::settlement::RunFinishedInput {
                dispatch_id,
                run_id: *run_id,
                is_succeeded: *succeeded,
                observed_cancelled: observed_cancellation(action),
                is_retryable_observed_failure: *retryable_observed_failure,
                observed_usage: *observed_usage,
                usage_basis: completion_usage_basis(action),
                safe_summary,
            })
        }
        Action::SubmitProposal { proposal } => super::review::submit_proposal(instance, proposal),
        Action::DecideProposal {
            approval,
            successor_validated: is_successor_validated,
        } => super::review::decide_proposal(instance, approval, *is_successor_validated),
        Action::RejectProposal { actor, proposal_id } => super::review::reject_proposal(instance, actor, proposal_id),
        Action::PauseRequested { actor } => super::control::pause(instance, actor),
        Action::ResumeRequested { actor } => super::control::resume(instance, actor),
        Action::CancelRequested { actor } => super::control::cancel(instance, actor),
        Action::CancellationObserved {
            run_id,
            cancelled: is_cancelled,
        } => super::super::ledger::settlement::observe_cancellation(instance, *run_id, *is_cancelled),
        Action::AbandonPendingDispatch {
            actor,
            dispatch_id,
            run_id,
        } => super::super::ledger::settlement::abandon_pending_dispatch(instance, actor, dispatch_id, *run_id),
        Action::BudgetExhausted { safe_reason } => super::control::exhaust_budget(instance, safe_reason),
    }
}

fn observed_cancellation(action: &Action) -> Option<bool> {
    match action {
        Action::RunFinishedObserved {
            cancelled: is_cancelled,
            ..
        } => Some(*is_cancelled),
        _ => None,
    }
}

fn completion_usage_basis(action: &Action) -> super::super::BudgetUsageBasis {
    match action {
        Action::RunFinishedObserved { usage_basis, .. } | Action::RunFinishedMeasured { usage_basis, .. } => {
            *usage_basis
        }
        _ => super::super::BudgetUsageBasis::ReservedFallback,
    }
}

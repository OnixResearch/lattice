//! Orchestration correlation for safe inspection.

const MAX_DISPATCH_DIAGNOSTICS: usize = 3;

#[derive(Clone, Copy)]
struct DispatchTarget<'a> {
    run_id: crate::engine::RunId,
    dispatch_id: &'a str,
}

#[derive(Clone, Copy)]
struct FinishOutcome {
    succeeded: bool,
    observed_usage: crate::orchestration::BudgetCharge,
}

pub(super) struct DispatchFacts {
    exact_start_count: usize,
    acknowledgement_count: usize,
    finish_count: usize,
    abandonment_count: usize,
    has_mismatch: bool,
    planned: Option<crate::orchestration::BudgetChargePlan>,
    observed: Option<crate::orchestration::BudgetCharge>,
    run_succeeded: Option<bool>,
}

impl DispatchFacts {
    fn new() -> Self {
        Self {
            exact_start_count: 0,
            acknowledgement_count: 0,
            finish_count: 0,
            abandonment_count: 0,
            has_mismatch: false,
            planned: None,
            observed: None,
            run_succeeded: None,
        }
    }

    pub(super) fn has_exact_binding(&self) -> bool {
        self.exact_start_count == 1
            && self.acknowledgement_count <= 1
            && self.finish_count.saturating_add(self.abandonment_count) <= 1
            && !self.has_mismatch
    }
}

pub(super) fn orchestration_event_chain(
    instance: &crate::orchestration::OrchestrationInstance,
    events: &[crate::orchestration::OrchestrationEvent],
) -> super::model::EvidenceState {
    let Ok(initial) = crate::orchestration::new_instance(
        instance.definition.clone(),
        instance.sequence,
        instance.trigger_idempotency_key.clone(),
    ) else {
        return super::model::EvidenceState::Invalid;
    };
    let replay = crate::orchestration::replay_events(&initial, events, instance.event_head.as_deref());
    if replay.valid && replay.instance == *instance {
        super::model::EvidenceState::Valid
    } else {
        super::model::EvidenceState::Invalid
    }
}

pub(super) fn collect_dispatch_facts(
    run_id: crate::engine::RunId,
    dispatch_id: &str,
    events: &[crate::orchestration::OrchestrationEvent],
) -> DispatchFacts {
    let target = DispatchTarget { run_id, dispatch_id };
    let mut facts = DispatchFacts::new();
    for event in events {
        match &event.fact.action {
            crate::orchestration::OrchestrationAction::StartRound { intent, planned_charge } => {
                collect_start_fact(&mut facts, target, intent, *planned_charge)
            }
            crate::orchestration::OrchestrationAction::DispatchAcknowledged {
                dispatch_id: acknowledged,
            } if acknowledged == target.dispatch_id => {
                facts.acknowledgement_count = facts.acknowledgement_count.saturating_add(1);
            }
            crate::orchestration::OrchestrationAction::RunFinished {
                dispatch_id: finished_dispatch,
                run_id: finished_run,
                succeeded,
                observed_usage,
                ..
            } => collect_finish_fact(&mut facts, target, finished_run, finished_dispatch, FinishOutcome {
                succeeded: *succeeded,
                observed_usage: *observed_usage,
            }),
            crate::orchestration::OrchestrationAction::AbandonPendingDispatch {
                dispatch_id: abandoned_dispatch,
                run_id: abandoned_run,
                ..
            } => collect_abandonment_fact(&mut facts, target, abandoned_run, abandoned_dispatch),
            _ => {}
        }
    }
    debug_assert!(facts.exact_start_count <= events.len());
    debug_assert!(facts.finish_count.saturating_add(facts.abandonment_count) <= events.len());
    facts
}

fn collect_start_fact(
    facts: &mut DispatchFacts,
    target: DispatchTarget<'_>,
    intent: &crate::orchestration::DispatchIntent,
    planned_charge: crate::orchestration::BudgetChargePlan,
) {
    let has_dispatch = intent.id == target.dispatch_id;
    let has_run = intent.run_id == target.run_id;
    if has_dispatch && has_run {
        facts.exact_start_count = facts.exact_start_count.saturating_add(1);
        facts.planned = Some(planned_charge);
    } else if has_dispatch || has_run {
        facts.has_mismatch = true;
    }
}

fn collect_finish_fact(
    facts: &mut DispatchFacts,
    target: DispatchTarget<'_>,
    finished_run: &crate::engine::RunId,
    finished_dispatch: &str,
    outcome: FinishOutcome,
) {
    let has_dispatch = finished_dispatch == target.dispatch_id;
    let has_run = *finished_run == target.run_id;
    if has_dispatch && has_run {
        facts.finish_count = facts.finish_count.saturating_add(1);
        facts.observed = Some(outcome.observed_usage);
        facts.run_succeeded = Some(outcome.succeeded);
    } else if has_dispatch || has_run {
        facts.has_mismatch = true;
    }
}

fn collect_abandonment_fact(
    facts: &mut DispatchFacts,
    target: DispatchTarget<'_>,
    abandoned_run: &crate::engine::RunId,
    abandoned_dispatch: &str,
) {
    let has_dispatch = abandoned_dispatch == target.dispatch_id;
    let has_run = *abandoned_run == target.run_id;
    if has_dispatch && has_run {
        facts.abandonment_count = facts.abandonment_count.saturating_add(1);
    } else if has_dispatch || has_run {
        facts.has_mismatch = true;
    }
}

pub(super) fn dispatch_diagnostics(facts: &DispatchFacts) -> Vec<super::model::DiagnosticCode> {
    let mut diagnostics = Vec::with_capacity(MAX_DISPATCH_DIAGNOSTICS);
    if facts.exact_start_count == 0 {
        diagnostics.push(super::model::DiagnosticCode::DispatchBindingMissing);
    } else if facts.exact_start_count > 1 {
        diagnostics.push(super::model::DiagnosticCode::DispatchBindingDuplicate);
    }
    if facts.has_mismatch || facts.acknowledgement_count > 1 {
        diagnostics.push(super::model::DiagnosticCode::DispatchFactMismatch);
    }
    if facts.finish_count.saturating_add(facts.abandonment_count) > 1 {
        diagnostics.push(super::model::DiagnosticCode::DispatchTerminalConflict);
    }
    diagnostics
}

pub(super) fn summarize_orchestration(
    instance: &crate::orchestration::OrchestrationInstance,
    event_chain: super::model::EvidenceState,
    facts: &DispatchFacts,
) -> super::model::SafeOrchestrationEvidence {
    let has_valid_facts = facts.has_exact_binding();
    let dispatch = dispatch_state(facts, event_chain, has_valid_facts);
    super::model::SafeOrchestrationEvidence {
        event_chain,
        status: instance.status,
        dispatch,
        recovery: recovery_state(dispatch),
        cancellation_requested: instance.cancellation_requested,
        run_succeeded: facts.run_succeeded,
        budget: summarize_budget(instance, facts, dispatch),
    }
}

fn dispatch_state(
    facts: &DispatchFacts,
    event_chain: super::model::EvidenceState,
    has_valid_facts: bool,
) -> super::model::DispatchEvidenceState {
    if event_chain != super::model::EvidenceState::Valid || !has_valid_facts {
        return super::model::DispatchEvidenceState::Invalid;
    }
    if facts.abandonment_count == 1 {
        if facts.acknowledgement_count == 0 {
            super::model::DispatchEvidenceState::AbandonedPending
        } else {
            super::model::DispatchEvidenceState::AbandonedAcknowledged
        }
    } else if facts.finish_count == 1 {
        super::model::DispatchEvidenceState::Finished
    } else if facts.acknowledgement_count == 1 {
        super::model::DispatchEvidenceState::Acknowledged
    } else {
        super::model::DispatchEvidenceState::Reserved
    }
}

fn recovery_state(dispatch: super::model::DispatchEvidenceState) -> super::model::RecoveryEvidenceState {
    match dispatch {
        super::model::DispatchEvidenceState::NotApplicable => super::model::RecoveryEvidenceState::NotApplicable,
        super::model::DispatchEvidenceState::Finished
        | super::model::DispatchEvidenceState::AbandonedPending
        | super::model::DispatchEvidenceState::AbandonedAcknowledged => super::model::RecoveryEvidenceState::Closed,
        super::model::DispatchEvidenceState::Reserved | super::model::DispatchEvidenceState::Acknowledged => {
            super::model::RecoveryEvidenceState::RunObservedPendingReconciliation
        }
        super::model::DispatchEvidenceState::Invalid => super::model::RecoveryEvidenceState::Invalid,
    }
}

fn summarize_budget(
    instance: &crate::orchestration::OrchestrationInstance,
    facts: &DispatchFacts,
    dispatch: super::model::DispatchEvidenceState,
) -> super::model::SafeBudgetEvidence {
    let settlement = match dispatch {
        super::model::DispatchEvidenceState::NotApplicable => super::model::BudgetSettlementState::NotApplicable,
        super::model::DispatchEvidenceState::Reserved | super::model::DispatchEvidenceState::Acknowledged => {
            super::model::BudgetSettlementState::Reserved
        }
        super::model::DispatchEvidenceState::Finished => super::model::BudgetSettlementState::Settled,
        super::model::DispatchEvidenceState::AbandonedPending => {
            super::model::BudgetSettlementState::ConservativePendingAbandonment
        }
        super::model::DispatchEvidenceState::AbandonedAcknowledged => {
            super::model::BudgetSettlementState::ConservativeAcknowledgedAbandonment
        }
        super::model::DispatchEvidenceState::Invalid => super::model::BudgetSettlementState::Invalid,
    };
    let result = super::model::SafeBudgetEvidence {
        settlement,
        planned_upper_bound: facts.planned,
        observed: facts.observed,
        orchestration_ledger_breached: instance.budgets.breached,
    };
    debug_assert_eq!(
        result.settlement == super::model::BudgetSettlementState::Invalid,
        dispatch == super::model::DispatchEvidenceState::Invalid
    );
    debug_assert_eq!(result.observed.is_some(), facts.observed.is_some());
    result
}

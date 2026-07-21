#![allow(assertion_density, path_segment_repetition)]

use super::*;

#[test]
fn fallback_policy_deny_returns_deny() {
    assert_eq!(
        evaluate_with(
            &standard_ledger(),
            &remote_chain(),
            crate::FallbackPolicy::Deny,
            false,
        ),
        crate::FallbackDecision::Deny,
    );
}

#[test]
fn exhausted_repair_budget_fails_route() {
    assert_eq!(
        evaluate_with(
            &standard_ledger(),
            &local_chain(),
            crate::FallbackPolicy::BestValidEvidence,
            true,
        ),
        budget_exhausted_decision(crate::BudgetKind::Repair),
    );
}

#[test]
fn exhausted_candidate_budget_fails_route() {
    assert_eq!(
        evaluate_with(
            &candidate_exhausted_ledger(),
            &remote_chain(),
            crate::FallbackPolicy::BestValidEvidence,
            false,
        ),
        budget_exhausted_decision(crate::BudgetKind::Candidate),
    );
}

#[test]
fn exhausted_parallelism_budget_fails_route() {
    assert_eq!(
        evaluate_with(
            &parallelism_exhausted_ledger(),
            &remote_chain(),
            crate::FallbackPolicy::BestValidEvidence,
            false,
        ),
        budget_exhausted_decision(crate::BudgetKind::Parallelism),
    );
}

#[test]
fn cyclic_fallback_chain_fails_route() {
    assert_eq!(
        evaluate_with(
            &standard_ledger(),
            &cyclic_chain(),
            crate::FallbackPolicy::BestValidEvidence,
            false,
        ),
        crate::FallbackDecision::FailRoute(crate::FallbackRouteReason::CyclicFallback),
    );
}

#[test]
fn safe_single_requires_eligible_fallback() {
    assert_eq!(
        evaluate_with(
            &standard_ledger(),
            &remote_chain(),
            crate::FallbackPolicy::SafeSingle,
            false,
        ),
        crate::FallbackDecision::FailRoute(crate::FallbackRouteReason::IneligibleFallback),
    );
}

#[test]
fn materialize_execution_plan_rejects_cyclic_fallback() {
    // r[verify agent_harness_runtime.model_router.execution_plan.invariants.cyclic_fallback]
    let cycle_catalog = alloc::vec![
        entry(EntryFixture {
            label: MODEL_PRIMARY,
            backend: BACKEND_LOCAL,
            role: "primary",
            local: true,
        }),
        entry(EntryFixture {
            label: MODEL_ESCALATION,
            backend: BACKEND_LOCAL,
            role: "escalation",
            local: true,
        }),
        entry(EntryFixture {
            label: MODEL_PRIMARY,
            backend: BACKEND_LOCAL,
            role: "fallback",
            local: true,
        }),
    ];
    let plan = confidence_plan(crate::FallbackEligibility::Eligible);

    assert!(!cycle_catalog.is_empty());
    assert_eq!(plan.family(), crate::RouteFamily::Confidence);
    assert_eq!(plan.selected_models(), &[MODEL_PRIMARY, MODEL_ESCALATION]);
    assert_eq!(plan.privacy(), crate::PrivacyBand::RemoteAllowed);
    assert_eq!(
        crate::materialize_execution_plan(&plan, &cycle_catalog, &budget()),
        Err(crate::ExecutionPlanError::CyclicFallback)
    );
}

#[test]
fn materialize_execution_plan_rejects_ineligible_fallback() {
    // r[verify agent_harness_runtime.model_router.execution_plan.invariants.fallback_eligibility]
    let plan = confidence_plan(crate::FallbackEligibility::Ineligible);

    assert_eq!(plan.family(), crate::RouteFamily::Confidence);
    assert_eq!(plan.selected_models(), &[MODEL_PRIMARY, MODEL_ESCALATION]);
    assert_eq!(plan.privacy(), crate::PrivacyBand::RemoteAllowed);
    assert_eq!(
        crate::materialize_execution_plan(&plan, &catalog(), &budget()),
        Err(crate::ExecutionPlanError::FallbackModelNotEligible)
    );
}

fn confidence_plan(fallback_eligibility: crate::FallbackEligibility) -> crate::SelectedPlan {
    selected_plan(SelectedPlanFixture {
        recipe_id: RECIPE_CONFIDENCE,
        family: crate::RouteFamily::Confidence,
        selected_models: &[MODEL_PRIMARY, MODEL_ESCALATION],
        output_contract: crate::OutputContract::Json,
        bounds: plan_bounds(BoundsFixture {
            max_candidates: MAX_CANDIDATES_THREE,
            max_parallelism: MAX_PARALLELISM_THREE,
            min_success_quorum: QUORUM_ONE,
            timeout_class: crate::limit::DeadlineClass::Standard,
            max_synthesis_attempts: SYNTHESIS_ONE,
            max_repair_attempts: REPAIR_ONE,
            fallback_policy: crate::limit::FallbackPolicy::BestValidEvidence,
        }),
        privacy: crate::PrivacyBand::RemoteAllowed,
        fallback_eligibility,
    })
}

fn evaluate_with(
    ledger: &crate::BudgetLedger,
    chain: &crate::FallbackChain,
    policy: crate::FallbackPolicy,
    can_retry_same_model: bool,
) -> crate::FallbackDecision {
    crate::evaluate_fallback(
        &crate::StepFailure::new(MODEL_PRIMARY, can_retry_same_model),
        ledger,
        chain,
        policy,
    )
}

fn budget_exhausted_decision(kind: crate::BudgetKind) -> crate::FallbackDecision {
    crate::FallbackDecision::FailRoute(crate::FallbackRouteReason::BudgetExhausted(
        crate::BudgetExhausted::new(kind, ZERO_COUNT),
    ))
}

fn standard_ledger() -> crate::BudgetLedger {
    crate::BudgetLedger::from(&standard_bounds())
}

fn candidate_exhausted_ledger() -> crate::BudgetLedger {
    crate::BudgetLedger::from(&candidate_exhausted_bounds())
}

fn parallelism_exhausted_ledger() -> crate::BudgetLedger {
    crate::BudgetLedger::from(&parallelism_exhausted_bounds())
}

fn standard_bounds() -> crate::Bounds {
    crate::limit::test_bounds(BoundsFixture {
        max_candidates: MAX_CANDIDATES_THREE,
        max_parallelism: MAX_PARALLELISM_THREE,
        min_success_quorum: QUORUM_ONE,
        timeout_class: crate::limit::DeadlineClass::Standard,
        max_synthesis_attempts: SYNTHESIS_ONE,
        max_repair_attempts: REPAIR_NONE,
        fallback_policy: crate::limit::FallbackPolicy::BestValidEvidence,
    })
}

fn candidate_exhausted_bounds() -> crate::Bounds {
    crate::limit::test_bounds(BoundsFixture {
        max_candidates: ZERO_COUNT,
        max_parallelism: MAX_PARALLELISM_THREE,
        min_success_quorum: QUORUM_ONE,
        timeout_class: crate::limit::DeadlineClass::Standard,
        max_synthesis_attempts: SYNTHESIS_ONE,
        max_repair_attempts: REPAIR_NONE,
        fallback_policy: crate::limit::FallbackPolicy::BestValidEvidence,
    })
}

fn parallelism_exhausted_bounds() -> crate::Bounds {
    crate::limit::test_bounds(BoundsFixture {
        max_candidates: MAX_CANDIDATES_THREE,
        max_parallelism: ZERO_COUNT,
        min_success_quorum: QUORUM_ONE,
        timeout_class: crate::limit::DeadlineClass::Standard,
        max_synthesis_attempts: SYNTHESIS_ONE,
        max_repair_attempts: REPAIR_NONE,
        fallback_policy: crate::limit::FallbackPolicy::BestValidEvidence,
    })
}

fn local_chain() -> crate::FallbackChain {
    crate::evidence::test_fallback_chain(alloc::vec![local_step()])
}

fn remote_chain() -> crate::FallbackChain {
    crate::evidence::test_fallback_chain(alloc::vec![remote_step()])
}

fn cyclic_chain() -> crate::FallbackChain {
    crate::evidence::test_fallback_chain(alloc::vec![local_step(), local_step()])
}

fn local_step() -> crate::FallbackStep {
    crate::evidence::test_fallback_step(crate::evidence::FallbackStepFixture {
        role: crate::Role::Fallback,
        model_label: MODEL_PRIMARY,
        backend_label: BACKEND_LOCAL,
        is_local: true,
    })
}

fn remote_step() -> crate::FallbackStep {
    crate::evidence::test_fallback_step(crate::evidence::FallbackStepFixture {
        role: crate::Role::Fallback,
        model_label: MODEL_ESCALATION,
        backend_label: BACKEND_REMOTE,
        is_local: false,
    })
}

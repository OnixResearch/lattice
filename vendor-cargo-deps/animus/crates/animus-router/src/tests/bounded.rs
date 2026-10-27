// Each case delegates the eight-bucket assertions to `assert_signal_labels`
// so the scenario setup stays readable while assertion coverage remains high.
#![allow(unknown_lints, assertion_density)]

use super::*;
use crate::SignalSource;

#[test]
fn simple_remote_request_projects_low_bands() {
    let metadata = metadata(crate::RequestMetadataDraft {
        prompt_char_estimate: SIMPLE_PROMPT_CHARS,
        tool_call_count: TOOL_COUNT_NONE,
        contains_code: false,
        requires_structured_output: false,
        context_window_fill_per_mille: LOW_CONTEXT_PER_MILLE,
        privacy_policy: "remote_allowed",
        fallback_hint: "eligible",
    });

    assert_signal_labels(
        &metadata,
        [
            "simple",
            "low",
            "low",
            "low",
            "low",
            "low",
            "remote_allowed",
            "eligible",
        ],
    );
}

#[test]
fn local_code_request_projects_coding_bands() {
    let metadata = metadata(crate::RequestMetadataDraft {
        prompt_char_estimate: SIMPLE_PROMPT_CHARS,
        tool_call_count: TOOL_COUNT_NONE,
        contains_code: true,
        requires_structured_output: false,
        context_window_fill_per_mille: LOW_CONTEXT_PER_MILLE,
        privacy_policy: "local_only",
        fallback_hint: "ineligible",
    });

    assert_signal_labels(
        &metadata,
        [
            "coding",
            "low",
            "low",
            "low",
            "low",
            "low",
            "local_only",
            "ineligible",
        ],
    );
}

#[test]
fn structured_review_request_projects_pressure_bands() {
    let metadata = metadata(crate::RequestMetadataDraft {
        prompt_char_estimate: REVIEW_PROMPT_CHARS,
        tool_call_count: TOOL_COUNT_NONE,
        contains_code: false,
        requires_structured_output: true,
        context_window_fill_per_mille: HIGH_CONTEXT_PER_MILLE,
        privacy_policy: "remote_allowed",
        fallback_hint: "eligible",
    });

    assert_signal_labels(
        &metadata,
        [
            "review",
            "medium",
            "strict",
            "high",
            "medium",
            "medium",
            "remote_allowed",
            "eligible",
        ],
    );
}

#[test]
fn workflow_request_projects_strict_bands() {
    let metadata = metadata(crate::RequestMetadataDraft {
        prompt_char_estimate: REASONING_PROMPT_CHARS,
        tool_call_count: TOOL_COUNT_TWO,
        contains_code: false,
        requires_structured_output: false,
        context_window_fill_per_mille: STRICT_CONTEXT_PER_MILLE,
        privacy_policy: "remote_allowed",
        fallback_hint: "eligible",
    });

    assert_signal_labels(
        &metadata,
        [
            "workflow",
            "strict",
            "high",
            "strict",
            "high",
            "high",
            "remote_allowed",
            "eligible",
        ],
    );
}

#[test]
fn rule_signal_source_feeds_signal_projection() {
    let source = crate::RuleSignalSource;
    let metadata = metadata(crate::RequestMetadataDraft {
        prompt_char_estimate: STRICT_PROMPT_CHARS,
        tool_call_count: TOOL_COUNT_ONE,
        contains_code: false,
        requires_structured_output: false,
        context_window_fill_per_mille: STRICT_CONTEXT_PER_MILLE,
        privacy_policy: "remote_allowed",
        fallback_hint: "eligible",
    });
    let draft = source
        .project(&metadata)
        .expect("rule source should project");
    let projected = crate::Signals::project(draft).expect("projected labels should parse");

    assert_eq!(projected.task_shape(), crate::TaskShape::Reasoning);
    assert_eq!(projected.risk(), crate::SignalBand::Strict);
    assert_eq!(projected.privacy(), crate::PrivacyBand::RemoteAllowed);
}

fn assert_signal_labels(
    metadata: &crate::RequestMetadata,
    expected: [&str; crate::projection::SIGNAL_BUCKET_COUNT],
) {
    let draft = crate::project_signals(metadata);
    let projected = crate::Signals::project(draft).expect("signal labels should parse");
    let actual = projected.bucket_labels();

    assert_eq!(actual.len(), expected.len());
    for (actual_label, expected_label) in actual.iter().zip(expected) {
        assert_eq!(actual_label.as_str(), expected_label);
    }
}

fn metadata(draft: crate::RequestMetadataDraft<'_>) -> crate::RequestMetadata {
    crate::RequestMetadata::new(draft).expect("metadata should validate")
}

#[test]
fn materialize_execution_plan_rejects_zero_quorum() {
    // r[verify agent_harness_runtime.model_router.execution_plan.invariants.zero_quorum]
    let plan = selected_plan(SelectedPlanFixture {
        recipe_id: RECIPE_SINGLE,
        family: crate::RouteFamily::Single,
        selected_models: &[MODEL_PRIMARY],
        output_contract: crate::OutputContract::Json,
        bounds: plan_bounds(BoundsFixture {
            max_candidates: MAX_CANDIDATES_ONE,
            max_parallelism: MAX_PARALLELISM_ONE,
            min_success_quorum: ZERO_COUNT,
            timeout_class: crate::limit::DeadlineClass::Standard,
            max_synthesis_attempts: SYNTHESIS_ONE,
            max_repair_attempts: REPAIR_ONE,
            fallback_policy: crate::limit::FallbackPolicy::BestValidEvidence,
        }),
        privacy: crate::PrivacyBand::RemoteAllowed,
        fallback_eligibility: crate::FallbackEligibility::Eligible,
    });

    assert_eq!(
        crate::materialize_execution_plan(&plan, &catalog(), &budget()),
        Err(crate::ExecutionPlanError::ZeroQuorum)
    );
}

#[test]
fn materialize_execution_plan_rejects_impossible_quorum() {
    // r[verify agent_harness_runtime.model_router.execution_plan.invariants.impossible_quorum]
    let plan = selected_plan(SelectedPlanFixture {
        recipe_id: RECIPE_SINGLE,
        family: crate::RouteFamily::Single,
        selected_models: &[MODEL_PRIMARY],
        output_contract: crate::OutputContract::Json,
        bounds: plan_bounds(BoundsFixture {
            max_candidates: MAX_CANDIDATES_ONE,
            max_parallelism: MAX_PARALLELISM_ONE,
            min_success_quorum: QUORUM_TWO,
            timeout_class: crate::limit::DeadlineClass::Standard,
            max_synthesis_attempts: SYNTHESIS_ONE,
            max_repair_attempts: REPAIR_ONE,
            fallback_policy: crate::limit::FallbackPolicy::BestValidEvidence,
        }),
        privacy: crate::PrivacyBand::RemoteAllowed,
        fallback_eligibility: crate::FallbackEligibility::Eligible,
    });

    assert_eq!(
        crate::materialize_execution_plan(&plan, &catalog(), &budget()),
        Err(crate::ExecutionPlanError::ImpossibleQuorum)
    );
}

#[test]
fn materialize_execution_plan_rejects_unbounded_repair() {
    // r[verify agent_harness_runtime.model_router.execution_plan.invariants.unbounded_repair]
    let plan = selected_plan(SelectedPlanFixture {
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
            max_repair_attempts: ZERO_COUNT,
            fallback_policy: crate::limit::FallbackPolicy::BestValidEvidence,
        }),
        privacy: crate::PrivacyBand::RemoteAllowed,
        fallback_eligibility: crate::FallbackEligibility::Eligible,
    });

    assert_eq!(
        crate::materialize_execution_plan(&plan, &catalog(), &budget()),
        Err(crate::ExecutionPlanError::UnboundedRepair)
    );
}

#[test]
fn materialize_execution_plan_rejects_missing_role_mapping() {
    // r[verify agent_harness_runtime.model_router.execution_plan.invariants.missing_role_mapping]
    let plan = selected_plan(SelectedPlanFixture {
        recipe_id: RECIPE_RATINGS,
        family: crate::RouteFamily::Ratings,
        selected_models: &[MODEL_PRIMARY],
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
        fallback_eligibility: crate::FallbackEligibility::Eligible,
    });

    assert_eq!(
        crate::materialize_execution_plan(&plan, &catalog(), &budget()),
        Err(crate::ExecutionPlanError::MissingRoleMapping)
    );
}

#[test]
fn budget_ledger_rejects_exhausted_consumption() {
    let ledger = crate::BudgetLedger::from(&crate::limit::test_bounds(BoundsFixture {
        max_candidates: ZERO_COUNT,
        max_parallelism: ZERO_COUNT,
        min_success_quorum: QUORUM_ONE,
        timeout_class: crate::limit::DeadlineClass::Standard,
        max_synthesis_attempts: SYNTHESIS_NONE,
        max_repair_attempts: REPAIR_NONE,
        fallback_policy: crate::limit::FallbackPolicy::Deny,
    }));
    let cases = [
        (ledger.consume_candidate(), crate::BudgetKind::Candidate),
        (ledger.consume_parallelism(), crate::BudgetKind::Parallelism),
        (
            ledger.consume_synthesis_attempt(),
            crate::BudgetKind::Synthesis,
        ),
        (ledger.consume_repair_attempt(), crate::BudgetKind::Repair),
    ];

    for (result, kind) in cases {
        assert_eq!(result, Err(crate::BudgetExhausted::new(kind, ZERO_COUNT)));
    }
}

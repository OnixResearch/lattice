#[path = "traceability_promotion/core.rs"]
mod core;

use core::CoverageReceipt;
use std::collections::BTreeSet;

const BASELINE: &str = include_str!("fixtures/traceability-agent-runtime-completion-current.json");
const CURRENT: &str = include_str!("fixtures/traceability-agent-runtime-promotion-current.json");
const ANIMUS: &str = include_str!("test_animus_agent.rs");
const SHELL: &str = include_str!("../src/animus_agent/shell/tests.rs");
const OPENAI: &str = include_str!("openai/execution.rs");
const POLICY: &str = include_str!("../cairn-policy/traceability-inventory.ncl");
const BASELINE_COUNT: usize = 304;
const CURRENT_COUNT: usize = 290;
const PROMOTED_COUNT: usize = 14;
const IDS: &[&str] = &[
    "lattice.agent_runtime.animus_compatibility.scenario.missing",
    "lattice.agent_runtime.animus_compatibility.scenario.routes",
    "lattice.agent_runtime.animus_effects.scenario.credential",
    "lattice.agent_runtime.animus_effects.scenario.provider-failure",
    "lattice.agent_runtime.animus_effects.scenario.tool-denied",
    "lattice.agent_runtime.animus_fabric.scenario.complete",
    "lattice.agent_runtime.animus_fabric.scenario.exhaustion",
    "lattice.agent_runtime.animus_fabric.scenario.identity",
    "lattice.agent_runtime.animus_leaf.scenario.authority",
    "lattice.agent_runtime.animus_leaf.scenario.execute",
    "lattice.agent_runtime.animus_output.scenario.non-authoritative",
    "lattice.agent_runtime.animus_output.scenario.typed",
    "lattice.agent_runtime.animus_output.scenario.unsupported",
    "lattice.agent_runtime.inline_reasoning_boundary.scenario.plain",
];
const ANIMUS_BINDINGS: &[(&str, &str)] = &[
    (IDS[0], "missing_or_failed_animus_provider_does_not_fallback"),
    (IDS[1], "existing_direct_openai_route_does_not_invoke_animus"),
    (IDS[2], "credential_is_used_only_at_lattice_transport_and_is_absent_from_evidence"),
    (IDS[3], "missing_or_failed_animus_provider_does_not_fallback"),
    (IDS[5], "production_animus_route_runs_one_model_call_for_text_and_json"),
    (IDS[7], "pure_admission_rejects_identity_turn_input_and_output_failures"),
    (IDS[8], "authority_bearing_response_remains_ordinary_agent_output"),
    (IDS[9], "typed_examples_load_validate_and_execute_through_the_runner"),
    (IDS[11], "production_animus_route_runs_one_model_call_for_text_and_json"),
    (IDS[12], "unsupported_animus_inputs_fail_before_provider_dispatch"),
];
const SHELL_BINDINGS: &[(&str, &str)] = &[
    (IDS[4], "absent_tool_service_fails_before_a_product_tool_effect"),
    (IDS[6], "bounded_fabric_prevents_a_model_dispatch_after_turn_exhaustion"),
    (IDS[10], "bounded_fabric_prevents_a_model_dispatch_after_turn_exhaustion"),
    (IDS[10], "stopped_and_max_turn_reports_map_to_node_failures"),
    (IDS[10], "absent_tool_service_fails_before_a_product_tool_effect"),
];
const OPENAI_BINDINGS: &[(&str, &str)] = &[(IDS[13], "production_adapter_decodes_text_and_json_outputs")];
const UNRELATED: &str = "lattice.agent_orchestration.authority.scenario.ambient-denied";
const ADDED: &str = "lattice.example.unaccepted.scenario.added";
const DANGLING: &str = "lattice.example.dangling.scenario.marker";

fn receipt(source: &str) -> CoverageReceipt {
    serde_json::from_str(source).expect("coverage decodes")
}
fn codes(issues: &[core::PromotionIssue]) -> BTreeSet<&str> {
    issues.iter().map(|issue| issue.code).collect()
}

// r[verify lattice.repository_validation.agent_runtime_promotion]
// r[verify lattice.repository_validation.agent_runtime_promotion.scenario.delta]
#[test]
fn exact_agent_runtime_promotion_preserves_unrelated_gaps() {
    let baseline = receipt(BASELINE);
    let current = receipt(CURRENT);
    let issues = core::validate_promotion(&baseline, &current, IDS);
    assert!(issues.is_empty(), "issues: {issues:?}");
    assert_eq!(baseline.missing.len(), BASELINE_COUNT);
    assert_eq!(current.missing.len(), CURRENT_COUNT);
    assert_eq!(IDS.len(), PROMOTED_COUNT);
    assert!(baseline.dangling.is_empty());
    assert!(current.dangling.is_empty());
    assert!(!baseline.valid);
    assert!(!current.valid);
}

// r[verify lattice.repository_validation.agent_runtime_promotion.scenario.bindings]
#[test]
fn agent_runtime_markers_are_locally_bound_and_leave_policy() {
    let animus = core::validate_marker_bindings(ANIMUS, ANIMUS_BINDINGS);
    let shell = core::validate_marker_bindings(SHELL, SHELL_BINDINGS);
    let openai = core::validate_marker_bindings(OPENAI, OPENAI_BINDINGS);
    let policy = core::validate_policy_excludes_promoted_ids(POLICY, IDS);
    assert!(animus.is_empty(), "animus: {animus:?}");
    assert!(shell.is_empty(), "shell: {shell:?}");
    assert!(openai.is_empty(), "openai: {openai:?}");
    assert!(policy.is_empty(), "policy: {policy:?}");
}

// r[verify lattice.repository_validation.agent_runtime_promotion.scenario.fail_closed]
#[test]
fn agent_runtime_promotion_drift_fails_closed() {
    let mut baseline = receipt(BASELINE);
    baseline.missing.push(baseline.missing[0].clone());
    let mut current = receipt(CURRENT);
    current.missing.retain(|id| id != UNRELATED);
    current.missing.push(IDS[0].to_string());
    current.missing.push(IDS[0].to_string());
    current.missing.push(ADDED.to_string());
    current.dangling.push(DANGLING.to_string());
    current.valid = true;
    let mut allowlist = IDS.to_vec();
    allowlist.push(IDS[0]);
    let promotion_issues = core::validate_promotion(&baseline, &current, &allowlist);
    let promotion_codes = codes(&promotion_issues);
    let binding_issues = core::validate_marker_bindings(ANIMUS, &[(IDS[0], "missing_test")]);
    let binding_codes = codes(&binding_issues);
    let stale = format!("requirement_id = \"{}\"", IDS[13]);
    let policy_issues = core::validate_policy_excludes_promoted_ids(&stale, IDS);
    let policy_codes = codes(&policy_issues);
    for code in [
        "baseline.duplicate_missing",
        "baseline.unsorted",
        "current.duplicate_missing",
        "current.unsorted",
        "current.dangling",
        "current.expected_invalid",
        "allowlist.duplicate",
        "transition.expected_removal_missing",
        "transition.unexpected_removed",
        "transition.unexpected_added",
    ] {
        assert!(promotion_codes.contains(code));
    }
    assert!(binding_codes.contains("binding.missing"));
    assert!(policy_codes.contains("policy.promoted_present"));
}

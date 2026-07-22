#[path = "traceability_promotion/core.rs"]
mod core;

use core::CoverageReceipt;
use std::collections::BTreeSet;

const BASELINE: &str = include_str!("fixtures/traceability-workflow-validation-audit-current.json");
const CURRENT: &str = include_str!("fixtures/traceability-workflow-validation-promotion-current.json");
const COMPLETION: &str = include_str!("test_workflow_validation_completion.rs");
const WORKFLOW: &str = include_str!("test_workflow_validation.rs");
const N8N: &str = include_str!("test_n8n_json.rs");
const REPLAY: &str = include_str!("test_replay.rs");
const CONTROL: &str = include_str!("test_control_flow.rs");
const POLICY: &str = include_str!("../cairn-policy/traceability-inventory.ncl");
const BASELINE_COUNT: usize = 290;
const CURRENT_COUNT: usize = 261;
const PROMOTED_COUNT: usize = 29;
const IDS: &[&str] = &[
    concat!("n8n", "clone.workflow_validation.credentials.providers.config"),
    concat!("n8n", "clone.workflow_validation.credentials.providers.field"),
    concat!("n8n", "clone.workflow_validation.credentials.providers.key"),
    concat!("n8n", "clone.workflow_validation.diagnostics.core.empty_command"),
    concat!("n8n", "clone.workflow_validation.diagnostics.core.missing_config"),
    concat!("n8n", "clone.workflow_validation.diagnostics.core.valid"),
    concat!("n8n", "clone.workflow_validation.diagnostics.negative.invalid_edge"),
    concat!("n8n", "clone.workflow_validation.generic_runner_admission.effects"),
    concat!("n8n", "clone.workflow_validation.generic_runner_admission.graph"),
    concat!("n8n", "clone.workflow_validation.generic_runner_admission.terminal_state"),
    concat!("n8n", "clone.workflow_validation.n8n_json.export_limits.node"),
    concat!("n8n", "clone.workflow_validation.n8n_json.unsupported.connection"),
    concat!("n8n", "clone.workflow_validation.n8n_json.unsupported.expression"),
    concat!("n8n", "clone.workflow_validation.n8n_json.unsupported.node_kind"),
    concat!("n8n", "clone.workflow_validation.n8n_parity_diagnostics.malformed_connection"),
    concat!("n8n", "clone.workflow_validation.n8n_parity_diagnostics.plaintext_credential"),
    concat!("n8n", "clone.workflow_validation.n8n_parity_diagnostics.unsupported_expression"),
    concat!("n8n", "clone.workflow_validation.n8n_parity_diagnostics.unsupported_node"),
    concat!("n8n", "clone.workflow_validation.node_kind_provider"),
    concat!("n8n", "clone.workflow_validation.replay_records.artifact"),
    concat!("n8n", "clone.workflow_validation.replay_records.order"),
    concat!("n8n", "clone.workflow_validation.replay_records.terminal"),
    concat!("n8n", "clone.workflow_validation.run_gate.block"),
    concat!("n8n", "clone.workflow_validation.run_gate.warning"),
    concat!("n8n", "clone.workflow_validation.subworkflows.mapping"),
    concat!("n8n", "clone.workflow_validation.subworkflows.missing"),
    concat!("n8n", "clone.workflow_validation.subworkflows.recursive"),
    concat!("n8n", "clone.workflow_validation.tui.panel.empty"),
    concat!("n8n", "clone.workflow_validation.tui.panel.open"),
];
const COMPLETION_BINDINGS: &[(&str, &str)] = &[
    (IDS[0], "credential_provider_errors_are_fatal_safe_and_pre_effect"),
    (IDS[1], "credential_provider_errors_are_fatal_safe_and_pre_effect"),
    (IDS[2], "credential_provider_errors_are_fatal_safe_and_pre_effect"),
    (IDS[7], "selected_runtime_rejects_unsupported_effect_policy_and_accepts_its_own"),
    (IDS[8], "generic_graph_admission_rejects_all_bounded_invalid_classes_without_mutation"),
    (IDS[9], "terminal_runner_transition_is_rejected_without_workflow_mutation"),
    (IDS[18], "every_builtin_node_kind_has_provider_validation_and_runtime_admission"),
    (IDS[24], "unknown_subworkflow_mapping_is_fatal_and_preserves_configuration"),
    (IDS[27], "tui_renders_ready_state_and_actionable_diagnostic_location"),
    (IDS[28], "tui_renders_ready_state_and_actionable_diagnostic_location"),
];
const WORKFLOW_BINDINGS: &[(&str, &str)] = &[
    (IDS[3], "empty_shell_command_is_fatal"),
    (IDS[4], "missing_node_config_is_fatal"),
    (IDS[5], "valid_workflow_has_no_fatal_diagnostics"),
    (IDS[6], "invalid_edge_reference_is_fatal_and_does_not_panic"),
    (IDS[22], "headless_run_gate_rejects_invalid_workflow_without_history"),
    (IDS[23], "warning_only_workflow_does_not_block_run_gate"),
];
const N8N_BINDINGS: &[(&str, &str)] = &[
    (IDS[10], "unsupported_export_node_reports_diagnostic"),
    (IDS[11], "malformed_connections_are_rejected"),
    (IDS[12], "unsupported_expressions_are_rejected"),
    (IDS[13], "unsupported_node_fixture_reports_fatal_diagnostic"),
    (IDS[14], "parity_unsupported_fixtures_fail_with_stable_safe_diagnostics"),
    (IDS[15], "parity_unsupported_fixtures_fail_with_stable_safe_diagnostics"),
    (IDS[16], "parity_unsupported_fixtures_fail_with_stable_safe_diagnostics"),
    (IDS[17], "parity_unsupported_fixtures_fail_with_stable_safe_diagnostics"),
];
const REPLAY_BINDINGS: &[(&str, &str)] = &[
    (IDS[19], "missing_artifact_payload_renders_placeholder_without_validation_failure"),
    (IDS[20], "duplicate_and_out_of_order_events_are_rejected"),
    (IDS[21], "terminal_state_mutation_is_rejected"),
];
const CONTROL_BINDINGS: &[(&str, &str)] = &[
    (IDS[25], "subworkflow_reference_check_accepts_present_child_and_rejects_missing_or_recursive_child"),
    (IDS[26], "subworkflow_reference_check_accepts_present_child_and_rejects_missing_or_recursive_child"),
];
const UNRELATED: &str = "lattice.agent_orchestration.authority.scenario.ambient-denied";
const ADDED: &str = "lattice.example.unaccepted.scenario.added";
const DANGLING: &str = "lattice.example.dangling.scenario.marker";

fn receipt(source: &str) -> CoverageReceipt {
    serde_json::from_str(source).expect("coverage decodes")
}
fn codes(issues: &[core::PromotionIssue]) -> BTreeSet<&str> {
    issues.iter().map(|issue| issue.code).collect()
}

// r[verify lattice.repository_validation.workflow_validation_promotion]
// r[verify lattice.repository_validation.workflow_validation_promotion.scenario.delta]
#[test]
fn exact_workflow_validation_promotion_preserves_unrelated_gaps() {
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

// r[verify lattice.repository_validation.workflow_validation_promotion.scenario.bindings]
#[test]
fn workflow_validation_markers_are_locally_bound_and_leave_policy() {
    for (name, issues) in [
        ("completion", core::validate_marker_bindings(COMPLETION, COMPLETION_BINDINGS)),
        ("workflow", core::validate_marker_bindings(WORKFLOW, WORKFLOW_BINDINGS)),
        ("n8n", core::validate_marker_bindings(N8N, N8N_BINDINGS)),
        ("replay", core::validate_marker_bindings(REPLAY, REPLAY_BINDINGS)),
        ("control", core::validate_marker_bindings(CONTROL, CONTROL_BINDINGS)),
    ] {
        assert!(issues.is_empty(), "{name}: {issues:?}");
    }
    let policy = core::validate_policy_excludes_promoted_ids(POLICY, IDS);
    assert!(policy.is_empty(), "policy: {policy:?}");
}

// r[verify lattice.repository_validation.workflow_validation_promotion.scenario.fail_closed]
#[test]
fn workflow_validation_promotion_drift_fails_closed() {
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
    let binding_issues = core::validate_marker_bindings(COMPLETION, &[(IDS[0], "missing_test")]);
    let binding_codes = codes(&binding_issues);
    let stale = format!("requirement_id = \"{}\"", IDS[28]);
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

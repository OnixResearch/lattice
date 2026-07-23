#[path = "traceability_promotion/core.rs"]
mod core;

use core::CoverageReceipt;
use std::collections::BTreeSet;

const BASELINE: &str = include_str!("fixtures/traceability-workflow-format-completion-current.json");
const CURRENT: &str = include_str!("fixtures/traceability-workflow-format-promotion-current.json");
const WF: &str = include_str!("test_workflow_format.rs");
const CONTROL: &str = include_str!("test_control_flow.rs");
const CREDENTIALS: &str = include_str!("test_credentials.rs");
const N8N: &str = include_str!("test_n8n_json.rs");
const REPLAY: &str = include_str!("test_replay.rs");
const TRIGGERS: &str = include_str!("test_triggers.rs");
const NODE_KIND: &str = include_str!("test_node_kind_provider.rs");
const WEBHOOKS: &str = include_str!("webhooks.rs");
const POLICY: &str = include_str!("../cairn-policy/traceability-inventory.ncl");
const BASELINE_COUNT: usize = 222;
const CURRENT_COUNT: usize = 181;
const PROMOTED_COUNT: usize = 41;
#[path = "traceability_workflow_format_ids.rs"]
mod workflow_format_ids;
use workflow_format_ids::IDS;
const WF_BINDINGS: &[(&str, &str)] = &[
    (IDS[31], "nickel_export_import_preserves_pipeline_shape_and_config"),
    (IDS[14], "nickel_export_import_preserves_runner_metadata"),
    (IDS[36], "portable_import_defaults_empty_runner_metadata_for_older_workflows"),
    (IDS[27], "cli_export_then_import_saves_workflow"),
    (IDS[29], "cli_export_then_import_saves_workflow"),
    (IDS[26], "cli_export_missing_workflow_writes_no_file"),
    (IDS[5], "nickel_export_import_preserves_image_input_and_generic_agent_node"),
    (IDS[6], "nickel_export_import_preserves_image_input_and_generic_agent_node"),
    (IDS[7], "nickel_export_import_preserves_image_input_and_generic_agent_node"),
    (IDS[16], "markdown_agent_prompt_file_loads_and_round_trips_source"),
    (IDS[0], "markdown_agent_prompt_source_cardinality_fails_closed"),
    (IDS[2], "markdown_agent_prompt_source_cardinality_fails_closed"),
    (IDS[15], "markdown_agent_prompt_symlink_escape_fails_closed"),
    (IDS[0], "cli_import_rejects_unsupported_declared_image_format_without_mutating_store"),
    (IDS[1], "cli_import_rejects_unsupported_declared_image_format_without_mutating_store"),
    (IDS[0], "portable_import_rejects_invalid_agent_target"),
    (IDS[4], "portable_import_rejects_invalid_agent_target"),
    (IDS[0], "portable_import_rejects_incompatible_image_agent_port"),
    (IDS[3], "portable_import_rejects_incompatible_image_agent_port"),
    (IDS[28], "nickel_import_rejects_missing_required_field"),
    (IDS[30], "nickel_import_rejects_missing_required_field"),
];
const CONTROL_BINDINGS: &[(&str, &str)] = &[
    (IDS[8], "portable_format_round_trips_control_flow_configs"),
    (IDS[9], "portable_format_round_trips_control_flow_configs"),
    (IDS[10], "portable_import_rejects_malformed_control_flow_without_partial_workflow"),
];
const CREDENTIAL_BINDINGS: &[(&str, &str)] = &[
    (IDS[11], "resolved_secret_is_not_saved_or_exported_with_workflow"),
    (IDS[12], "nickel_import_round_trip_preserves_opaque_provider_reference_without_resolution"),
    (IDS[13], "nickel_import_rejects_all_non_null_plaintext_credential_shapes_with_safe_error"),
];
const N8N_BINDINGS: &[(&str, &str)] = &[
    (IDS[24], "parity_supported_fixtures_import_validate_and_match_golden_reports"),
    (IDS[25], "parity_supported_fixtures_import_validate_and_match_golden_reports"),
    (IDS[24], "parity_unsupported_fixtures_fail_with_stable_safe_diagnostics"),
    (IDS[25], "parity_unsupported_fixtures_fail_with_stable_safe_diagnostics"),
    (IDS[20], "parity_unsupported_fixtures_fail_with_stable_safe_diagnostics"),
    (IDS[19], "supported_shell_fixture_imports_to_typed_workflow"),
    (IDS[22], "supported_shell_fixture_imports_to_typed_workflow"),
    (IDS[19], "supported_http_fixture_imports_credential_placeholder"),
    (IDS[22], "supported_http_fixture_imports_credential_placeholder"),
    (IDS[17], "supported_shell_workflow_exports_to_n8n_json"),
    (IDS[21], "malformed_json_is_rejected"),
    (IDS[18], "unsupported_export_node_reports_diagnostic"),
    (IDS[23], "import_n8n_cli_persists_workflow_and_report"),
];
const REPLAY_BINDINGS: &[(&str, &str)] = &[
    (IDS[33], "replay_bundle_round_trips_safe_events"),
    (IDS[34], "replay_bundle_round_trips_safe_events"),
    (IDS[35], "replay_bundle_rejects_plaintext_secret_markers"),
];
const TRIGGER_BINDINGS: &[(&str, &str)] = &[
    (IDS[37], "workflow_format_round_trips_triggers_through_portable_and_nickel"),
    (IDS[38], "workflow_format_round_trips_triggers_through_portable_and_nickel"),
];
const NODE_KIND_BINDINGS: &[(&str, &str)] =
    &[(IDS[32], "provider_registry_resolves_supported_and_rejects_unknown_kinds")];
const WEBHOOK_BINDINGS: &[(&str, &str)] = &[
    (IDS[39], "webhook_cookbook_import_persists_trigger_and_dry_runs"),
    (IDS[40], "webhook_cookbook_malformed_drift_is_rejected_deterministically"),
];
const UNRELATED: &str = concat!("n8n", "clone.tui_ux.add_node.cancel");
const ADDED: &str = "lattice.example.unaccepted.scenario.added";
const DANGLING: &str = "lattice.example.dangling.scenario.marker";

fn receipt(source: &str) -> CoverageReceipt {
    serde_json::from_str(source).expect("coverage decodes")
}
fn codes(issues: &[core::PromotionIssue]) -> BTreeSet<&str> {
    issues.iter().map(|issue| issue.code).collect()
}

// r[verify lattice.repository_validation.workflow_format_promotion]
// r[verify lattice.repository_validation.workflow_format_promotion.scenario.delta]
#[test]
fn exact_workflow_format_promotion_preserves_unrelated_gaps() {
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

// r[verify lattice.repository_validation.workflow_format_promotion.scenario.bindings]
#[test]
fn workflow_format_markers_are_locally_bound_and_leave_policy() {
    for (name, issues) in [
        ("workflow", core::validate_marker_bindings(WF, WF_BINDINGS)),
        ("control", core::validate_marker_bindings(CONTROL, CONTROL_BINDINGS)),
        ("credentials", core::validate_marker_bindings(CREDENTIALS, CREDENTIAL_BINDINGS)),
        ("n8n", core::validate_marker_bindings(N8N, N8N_BINDINGS)),
        ("replay", core::validate_marker_bindings(REPLAY, REPLAY_BINDINGS)),
        ("triggers", core::validate_marker_bindings(TRIGGERS, TRIGGER_BINDINGS)),
        ("node-kind", core::validate_marker_bindings(NODE_KIND, NODE_KIND_BINDINGS)),
        ("webhooks", core::validate_marker_bindings(WEBHOOKS, WEBHOOK_BINDINGS)),
    ] {
        assert!(issues.is_empty(), "{name}: {issues:?}");
    }
    let policy = core::validate_policy_excludes_promoted_ids(POLICY, IDS);
    assert!(policy.is_empty(), "policy: {policy:?}");
}

// r[verify lattice.repository_validation.workflow_format_promotion.scenario.fail_closed]
#[test]
fn workflow_format_promotion_drift_fails_closed() {
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
    let binding_issues = core::validate_marker_bindings(WF, &[(IDS[0], "missing_test")]);
    let binding_codes = codes(&binding_issues);
    let stale = format!("requirement_id = \"{}\"", IDS[PROMOTED_COUNT - 1]);
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

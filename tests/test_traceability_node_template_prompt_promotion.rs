#[path = "traceability_promotion/core.rs"]
mod core;

use core::CoverageReceipt;
use core::validate_marker_bindings;
use core::validate_policy_excludes_promoted_ids;
use core::validate_promotion;
use std::collections::BTreeSet;

const BASELINE_COVERAGE: &str = include_str!("fixtures/traceability-node-template-prompt-completion-current.json");
const CURRENT_COVERAGE: &str = include_str!("fixtures/traceability-node-template-prompt-promotion-current.json");
const APP_SOURCE: &str = include_str!("../src/tui/app.rs");
const RENDER_TEST_SOURCE: &str = include_str!("test_template_parameter_prompt_rendering.rs");
const INVENTORY_POLICY: &str = include_str!("../cairn-policy/traceability-inventory.ncl");
const BASELINE_MISSING_COUNT: usize = 327;
const CURRENT_MISSING_COUNT: usize = 324;
const PROMOTED_COUNT: usize = 3;
const UNRELATED_MISSING_ID: &str = "lattice.agent_orchestration.authority.scenario.ambient-denied";
const UNEXPECTED_ADDED_ID: &str = "lattice.example.unaccepted.scenario.added";
const DANGLING_ID: &str = "lattice.example.dangling.scenario.marker";
const PROMPT_ID: &str = concat!("n8n", "clone.node_templates.parameters.prompt");
const CANCEL_ID: &str = concat!("n8n", "clone.node_templates.parameters.prompt.cancel");
const REQUIRED_ID: &str = concat!("n8n", "clone.node_templates.parameters.prompt.required");
const PROMOTED_IDS: &[&str] = &[PROMPT_ID, CANCEL_ID, REQUIRED_ID];
const PROMPT_TEST: &str = "add_node_flow_prompts_and_renders_template_parameters";
const CANCEL_TEST: &str = "cancel_template_parameter_entry_preserves_workflow";
const REQUIRED_TEST: &str = "required_template_parameter_prompt_is_visible_before_node_creation";
const APP_BINDINGS: &[(&str, &str)] = &[(PROMPT_ID, PROMPT_TEST), (CANCEL_ID, CANCEL_TEST)];
const RENDER_BINDINGS: &[(&str, &str)] = &[(REQUIRED_ID, REQUIRED_TEST)];

fn decode_receipt(source: &str) -> CoverageReceipt {
    serde_json::from_str(source).expect("coverage fixture must decode")
}

fn issue_codes(issues: &[core::PromotionIssue]) -> BTreeSet<&str> {
    issues.iter().map(|issue| issue.code).collect()
}

// r[verify lattice.repository_validation.node_template_prompt_promotion]
// r[verify lattice.repository_validation.node_template_prompt_promotion.scenario.exact_delta]
#[test]
fn exact_node_template_prompt_promotion_preserves_every_unrelated_gap() {
    let baseline = decode_receipt(BASELINE_COVERAGE);
    let current = decode_receipt(CURRENT_COVERAGE);
    let issues = validate_promotion(&baseline, &current, PROMOTED_IDS);

    assert!(issues.is_empty(), "issues: {issues:?}");
    assert_eq!(baseline.missing.len(), BASELINE_MISSING_COUNT);
    assert_eq!(current.missing.len(), CURRENT_MISSING_COUNT);
    assert_eq!(PROMOTED_IDS.len(), PROMOTED_COUNT);
    assert!(baseline.dangling.is_empty());
    assert!(current.dangling.is_empty());
    assert!(!baseline.valid);
    assert!(!current.valid);
}

// r[verify lattice.repository_validation.node_template_prompt_promotion.scenario.local_binding]
#[test]
fn node_template_prompt_markers_are_attached_to_exact_tests_and_leave_debt_policy() {
    let app_binding_issues = validate_marker_bindings(APP_SOURCE, APP_BINDINGS);
    let render_binding_issues = validate_marker_bindings(RENDER_TEST_SOURCE, RENDER_BINDINGS);
    let policy_issues = validate_policy_excludes_promoted_ids(INVENTORY_POLICY, PROMOTED_IDS);

    assert!(app_binding_issues.is_empty(), "app binding issues: {app_binding_issues:?}");
    assert!(render_binding_issues.is_empty(), "render binding issues: {render_binding_issues:?}");
    assert!(policy_issues.is_empty(), "policy issues: {policy_issues:?}");
}

// r[verify lattice.repository_validation.node_template_prompt_promotion.scenario.fail_closed]
#[test]
fn duplicate_unbound_stale_dangling_and_successful_prompt_transitions_fail_closed() {
    let mut baseline = decode_receipt(BASELINE_COVERAGE);
    baseline.missing.push(baseline.missing[0].clone());
    let mut current = decode_receipt(CURRENT_COVERAGE);
    current.missing.retain(|identifier| identifier != UNRELATED_MISSING_ID);
    current.missing.push(PROMPT_ID.to_string());
    current.missing.push(PROMPT_ID.to_string());
    current.missing.push(UNEXPECTED_ADDED_ID.to_string());
    current.dangling.push(DANGLING_ID.to_string());
    current.valid = true;
    let allowlist = [PROMPT_ID, PROMPT_ID, CANCEL_ID, REQUIRED_ID];

    let promotion_issues = validate_promotion(&baseline, &current, &allowlist);
    let promotion_codes = issue_codes(&promotion_issues);
    let binding_issues = validate_marker_bindings(APP_SOURCE, &[(PROMPT_ID, "missing_test")]);
    let binding_codes = issue_codes(&binding_issues);
    let stale_policy = format!("requirement_id = \"{REQUIRED_ID}\"");
    let policy_issues = validate_policy_excludes_promoted_ids(&stale_policy, PROMOTED_IDS);
    let policy_codes = issue_codes(&policy_issues);

    assert!(promotion_codes.contains("baseline.duplicate_missing"));
    assert!(promotion_codes.contains("baseline.unsorted"));
    assert!(promotion_codes.contains("current.duplicate_missing"));
    assert!(promotion_codes.contains("current.unsorted"));
    assert!(promotion_codes.contains("current.dangling"));
    assert!(promotion_codes.contains("current.expected_invalid"));
    assert!(promotion_codes.contains("allowlist.duplicate"));
    assert!(promotion_codes.contains("transition.expected_removal_missing"));
    assert!(promotion_codes.contains("transition.unexpected_removed"));
    assert!(promotion_codes.contains("transition.unexpected_added"));
    assert!(binding_codes.contains("binding.missing"));
    assert!(policy_codes.contains("policy.promoted_present"));
}

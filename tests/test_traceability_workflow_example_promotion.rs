#[path = "traceability_promotion/core.rs"]
mod core;

use core::CoverageReceipt;
use core::validate_marker_bindings;
use core::validate_policy_excludes_promoted_ids;
use core::validate_promotion;
use std::collections::BTreeSet;

const BASELINE: &str = include_str!("fixtures/traceability-workflow-example-completion-current.json");
const CURRENT: &str = include_str!("fixtures/traceability-workflow-example-promotion-current.json");
const TEST_SOURCE: &str = include_str!("test_image_workflow_example.rs");
const POLICY: &str = include_str!("../cairn-policy/traceability-inventory.ncl");
const BASELINE_COUNT: usize = 317;
const CURRENT_COUNT: usize = 315;
const INVALID_ID: &str = concat!("n8n", "clone.workflow_examples.image.invalid");
const SELFIE_ID: &str = concat!("n8n", "clone.workflow_examples.image.selfie_to_fine_art");
const IDS: &[&str] = &[INVALID_ID, SELFIE_ID];
const BINDINGS: &[(&str, &str)] = &[
    (SELFIE_ID, "selfie_to_fine_art_example_imports_and_runs_with_fake_clankers_adapter"),
    (INVALID_ID, "invalid_image_example_reports_format_diagnostics_without_partial_completion"),
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

// r[verify lattice.repository_validation.workflow_example_promotion]
// r[verify lattice.repository_validation.workflow_example_promotion.scenario.delta]
#[test]
fn exact_workflow_example_promotion_preserves_unrelated_gaps() {
    let baseline = receipt(BASELINE);
    let current = receipt(CURRENT);
    let issues = validate_promotion(&baseline, &current, IDS);
    assert!(issues.is_empty(), "issues: {issues:?}");
    assert_eq!(baseline.missing.len(), BASELINE_COUNT);
    assert_eq!(current.missing.len(), CURRENT_COUNT);
    assert!(baseline.dangling.is_empty());
    assert!(current.dangling.is_empty());
    assert!(!baseline.valid);
    assert!(!current.valid);
}

// r[verify lattice.repository_validation.workflow_example_promotion.scenario.bindings]
#[test]
fn workflow_example_markers_are_bound_and_leave_policy() {
    let binding_issues = validate_marker_bindings(TEST_SOURCE, BINDINGS);
    let policy_issues = validate_policy_excludes_promoted_ids(POLICY, IDS);
    assert!(binding_issues.is_empty(), "binding issues: {binding_issues:?}");
    assert!(policy_issues.is_empty(), "policy issues: {policy_issues:?}");
}

#[test]
fn workflow_example_promotion_drift_fails_closed() {
    let mut baseline = receipt(BASELINE);
    baseline.missing.push(baseline.missing[0].clone());
    let mut current = receipt(CURRENT);
    current.missing.retain(|id| id != UNRELATED);
    current.missing.push(INVALID_ID.to_string());
    current.missing.push(INVALID_ID.to_string());
    current.missing.push(ADDED.to_string());
    current.dangling.push(DANGLING.to_string());
    current.valid = true;
    let allowlist = [INVALID_ID, INVALID_ID, SELFIE_ID];
    let promotion_issues = validate_promotion(&baseline, &current, &allowlist);
    let promotion_codes = codes(&promotion_issues);
    let binding_issues = validate_marker_bindings(TEST_SOURCE, &[(INVALID_ID, "missing_test")]);
    let binding_codes = codes(&binding_issues);
    let stale_policy = format!("requirement_id = \"{INVALID_ID}\"");
    let policy_issues = validate_policy_excludes_promoted_ids(&stale_policy, IDS);
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

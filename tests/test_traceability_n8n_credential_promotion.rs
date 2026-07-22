#[path = "traceability_promotion/core.rs"]
mod core;

use core::CoverageReceipt;
use core::validate_marker_bindings;
use core::validate_policy_excludes_promoted_ids;
use core::validate_promotion;
use std::collections::BTreeSet;

const BASELINE_COVERAGE: &str = include_str!("fixtures/traceability-n8n-credential-completion-current.json");
const CURRENT_COVERAGE: &str = include_str!("fixtures/traceability-n8n-credential-promotion-current.json");
const BOUNDARY_TEST_SOURCE: &str = include_str!("test_n8n_credential_boundaries.rs");
const INVENTORY_POLICY: &str = include_str!("../cairn-policy/traceability-inventory.ncl");
const BASELINE_MISSING_COUNT: usize = 340;
const CURRENT_MISSING_COUNT: usize = 336;
const PROMOTED_COUNT: usize = 4;
const UNRELATED_MISSING_ID: &str = "lattice.agent_orchestration.authority.scenario.ambient-denied";
const UNEXPECTED_ADDED_ID: &str = "lattice.example.unaccepted.scenario.added";
const DANGLING_ID: &str = "lattice.example.dangling.scenario.marker";

const EXPORT_ID: &str = concat!("n8n", "clone.credentials.n8n_json.placeholders.export");
const PLAINTEXT_ID: &str = concat!("n8n", "clone.credentials.n8n_json.placeholders.plaintext");
const REFERENCE_ID: &str = concat!("n8n", "clone.credentials.n8n_json.placeholders.reference");
const PARITY_ID: &str = concat!("n8n", "clone.credentials.n8n_parity_secret_safety");
const PROMOTED_IDS: &[&str] = &[EXPORT_ID, PLAINTEXT_ID, REFERENCE_ID, PARITY_ID];

const IMPORT_TEST: &str = "imported_placeholder_resolves_before_execution_and_missing_value_blocks_side_effect";
const EXPORT_TEST: &str = "resolved_reference_stays_opaque_across_save_raw_store_and_cli_export";
const PLAINTEXT_TEST: &str = "plaintext_cli_import_reports_safe_guidance_without_creating_store";
const PARITY_TEST: &str = "parity_sentinel_is_bounded_to_one_negative_input_and_all_outputs_are_safe";
const REQUIRED_BINDINGS: &[(&str, &str)] = &[
    (REFERENCE_ID, IMPORT_TEST),
    (EXPORT_ID, EXPORT_TEST),
    (PLAINTEXT_ID, PLAINTEXT_TEST),
    (PARITY_ID, PLAINTEXT_TEST),
    (PARITY_ID, PARITY_TEST),
];

fn decode_receipt(source: &str) -> CoverageReceipt {
    serde_json::from_str(source).expect("coverage fixture must decode")
}

fn issue_codes(issues: &[core::PromotionIssue]) -> BTreeSet<&str> {
    issues.iter().map(|issue| issue.code).collect()
}

// r[verify lattice.repository_validation.n8n_credential_promotion]
// r[verify lattice.repository_validation.n8n_credential_promotion.scenario.exact_delta]
#[test]
fn exact_n8n_credential_promotion_preserves_every_unrelated_gap() {
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

// r[verify lattice.repository_validation.n8n_credential_promotion.scenario.local_binding]
#[test]
fn n8n_credential_markers_are_attached_to_exact_tests_and_leave_debt_policy() {
    let binding_issues = validate_marker_bindings(BOUNDARY_TEST_SOURCE, REQUIRED_BINDINGS);
    let policy_issues = validate_policy_excludes_promoted_ids(INVENTORY_POLICY, PROMOTED_IDS);

    assert!(binding_issues.is_empty(), "binding issues: {binding_issues:?}");
    assert!(policy_issues.is_empty(), "policy issues: {policy_issues:?}");
}

// r[verify lattice.repository_validation.n8n_credential_promotion.scenario.fail_closed]
#[test]
fn duplicate_unbound_stale_dangling_and_successful_n8n_transitions_fail_closed() {
    let mut baseline = decode_receipt(BASELINE_COVERAGE);
    baseline.missing.push(baseline.missing[0].clone());
    let mut current = decode_receipt(CURRENT_COVERAGE);
    current.missing.retain(|identifier| identifier != UNRELATED_MISSING_ID);
    current.missing.push(EXPORT_ID.to_string());
    current.missing.push(EXPORT_ID.to_string());
    current.missing.push(UNEXPECTED_ADDED_ID.to_string());
    current.dangling.push(DANGLING_ID.to_string());
    current.valid = true;
    let allowlist = [EXPORT_ID, EXPORT_ID, PLAINTEXT_ID, REFERENCE_ID, PARITY_ID];

    let promotion_issues = validate_promotion(&baseline, &current, &allowlist);
    let promotion_codes = issue_codes(&promotion_issues);
    let binding_issues = validate_marker_bindings(BOUNDARY_TEST_SOURCE, &[(EXPORT_ID, "missing_test")]);
    let binding_codes = issue_codes(&binding_issues);
    let stale_policy = format!("requirement_id = \"{PARITY_ID}\"");
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

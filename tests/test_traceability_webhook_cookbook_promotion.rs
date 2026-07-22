#[path = "traceability_promotion/core.rs"]
mod core;

use core::CoverageReceipt;
use core::marker_is_attached_to_test;
use core::validate_promotion;
use std::collections::BTreeSet;

const BASELINE_COVERAGE: &str = include_str!("fixtures/traceability-remaining-credentials-current.json");
const CURRENT_COVERAGE: &str = include_str!("fixtures/traceability-webhook-cookbook-promotion-current.json");
const WEBHOOK_TEST_SOURCE: &str = include_str!("webhooks.rs");
const INVENTORY_POLICY: &str = include_str!("../cairn-policy/traceability-inventory.ncl");
const BASELINE_MISSING_COUNT: usize = 341;
const CURRENT_MISSING_COUNT: usize = 340;
const PROMOTED_COUNT: usize = 1;
const UNRELATED_MISSING_ID: &str = "lattice.agent_orchestration.authority.scenario.ambient-denied";
const UNEXPECTED_ADDED_ID: &str = "lattice.example.unaccepted.scenario.added";
const DANGLING_ID: &str = "lattice.example.dangling.scenario.marker";
const COOKBOOK_SAFETY_ID: &str = concat!("n8n", "clone.credentials.webhook_cookbook_secret_safety");
const PROMOTED_IDS: &[&str] = &[COOKBOOK_SAFETY_ID];
const REQUIRED_TESTS: &[&str] = &[
    "webhook_cookbook_valid_request_starts_one_run_and_records_safe_history",
    "webhook_cookbook_rejected_requests_do_not_start_runs_or_expose_secrets",
];

fn decode_receipt(source: &str) -> CoverageReceipt {
    serde_json::from_str(source).expect("coverage fixture must decode")
}

fn issue_codes(issues: &[core::PromotionIssue]) -> BTreeSet<&str> {
    issues.iter().map(|issue| issue.code).collect()
}

// r[verify lattice.repository_validation.cookbook_credential_promotion]
// r[verify lattice.repository_validation.cookbook_credential_promotion.scenario.exact_delta]
#[test]
fn exact_cookbook_promotion_preserves_every_unrelated_gap() {
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

// r[verify lattice.repository_validation.cookbook_credential_promotion.scenario.local_binding]
#[test]
fn cookbook_marker_is_attached_to_both_exact_tests_and_leaves_debt_policy() {
    for test_name in REQUIRED_TESTS {
        assert!(
            marker_is_attached_to_test(WEBHOOK_TEST_SOURCE, test_name, COOKBOOK_SAFETY_ID),
            "cookbook marker was not attached to {test_name}",
        );
    }
    assert!(!INVENTORY_POLICY.contains(COOKBOOK_SAFETY_ID));
}

// r[verify lattice.repository_validation.cookbook_credential_promotion.scenario.fail_closed]
#[test]
fn duplicate_unexpected_dangling_and_successful_cookbook_transitions_fail_closed() {
    let mut baseline = decode_receipt(BASELINE_COVERAGE);
    baseline.missing.push(baseline.missing[0].clone());
    let mut current = decode_receipt(CURRENT_COVERAGE);
    current.missing.retain(|identifier| identifier != UNRELATED_MISSING_ID);
    current.missing.push(COOKBOOK_SAFETY_ID.to_string());
    current.missing.push(COOKBOOK_SAFETY_ID.to_string());
    current.missing.push(UNEXPECTED_ADDED_ID.to_string());
    current.dangling.push(DANGLING_ID.to_string());
    current.valid = true;
    let allowlist = [COOKBOOK_SAFETY_ID, COOKBOOK_SAFETY_ID];

    let issues = validate_promotion(&baseline, &current, &allowlist);
    let codes = issue_codes(&issues);

    assert!(codes.contains("baseline.duplicate_missing"));
    assert!(codes.contains("baseline.unsorted"));
    assert!(codes.contains("current.duplicate_missing"));
    assert!(codes.contains("current.unsorted"));
    assert!(codes.contains("current.dangling"));
    assert!(codes.contains("current.expected_invalid"));
    assert!(codes.contains("allowlist.duplicate"));
    assert!(codes.contains("transition.expected_removal_missing"));
    assert!(codes.contains("transition.unexpected_removed"));
    assert!(codes.contains("transition.unexpected_added"));
}

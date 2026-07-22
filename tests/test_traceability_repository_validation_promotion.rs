#[path = "traceability_promotion/core.rs"]
mod core;

use core::CoverageReceipt;
use core::validate_marker_bindings;
use core::validate_policy_excludes_promoted_ids;
use core::validate_promotion;
use std::collections::BTreeSet;

const BASELINE: &str = include_str!("fixtures/traceability-repository-validation-completion-current.json");
const CURRENT: &str = include_str!("fixtures/traceability-repository-validation-promotion-current.json");
const REPOSITORY_TESTS: &str = include_str!("test_repository_validation.rs");
const POLICY: &str = include_str!("../cairn-policy/traceability-inventory.ncl");
const BASELINE_COUNT: usize = 315;
const CURRENT_COUNT: usize = 304;
const PROMOTED_COUNT: usize = 11;

const CLIPPY_ID: &str = concat!("n8n", "clone.repository_validation.clippy_scope");
const CLIPPY_VENDOR_ID: &str = concat!("n8n", "clone.repository_validation.clippy_scope.scenario.vendor-lints");
const FLAKE_ID: &str = concat!("n8n", "clone.repository_validation.flake_check");
const FLAKE_MISSING_KEY_ID: &str =
    concat!("n8n", "clone.repository_validation.flake_check.scenario.missing-signing-key");
const FLAKE_NORMAL_SIGNING_ID: &str = concat!("n8n", "clone.repository_validation.flake_check.scenario.normal-signing");
const FORMAT_ID: &str = concat!("n8n", "clone.repository_validation.format_scope");
const FORMAT_VENDOR_ID: &str = concat!("n8n", "clone.repository_validation.format_scope.scenario.vendor-format");
const TEST_SOURCE_ID: &str = concat!("n8n", "clone.repository_validation.test_source");
const TEST_JSON_ID: &str = concat!("n8n", "clone.repository_validation.test_source.scenario.json-fixture");
const VENDOR_ID: &str = concat!("n8n", "clone.repository_validation.vendor_snapshot");
const VENDOR_NESTED_ID: &str =
    concat!("n8n", "clone.repository_validation.vendor_snapshot.scenario.nested-output-name");
const PROMOTED: &[&str] = &[
    CLIPPY_ID,
    CLIPPY_VENDOR_ID,
    FLAKE_ID,
    FLAKE_MISSING_KEY_ID,
    FLAKE_NORMAL_SIGNING_ID,
    FORMAT_ID,
    FORMAT_VENDOR_ID,
    TEST_SOURCE_ID,
    TEST_JSON_ID,
    VENDOR_ID,
    VENDOR_NESTED_ID,
];
const BINDINGS: &[(&str, &str)] = &[
    (CLIPPY_ID, "clippy_denies_owned_warnings_without_admitting_vendor_lints"),
    (CLIPPY_VENDOR_ID, "clippy_denies_owned_warnings_without_admitting_vendor_lints"),
    (FLAKE_ID, "check_scoped_signing_override_leaves_flake_signing_external"),
    (FLAKE_MISSING_KEY_ID, "check_scoped_signing_override_leaves_flake_signing_external"),
    (FLAKE_NORMAL_SIGNING_ID, "check_scoped_signing_override_leaves_flake_signing_external"),
    (FORMAT_ID, "format_check_is_scoped_away_from_nonconforming_vendor_fixture"),
    (FORMAT_VENDOR_ID, "format_check_is_scoped_away_from_nonconforming_vendor_fixture"),
    (TEST_SOURCE_ID, "filtered_nix_source_compiles_nested_vendor_and_json_fixtures"),
    (TEST_JSON_ID, "filtered_nix_source_compiles_nested_vendor_and_json_fixtures"),
    (VENDOR_ID, "filtered_nix_source_compiles_nested_vendor_and_json_fixtures"),
    (VENDOR_NESTED_ID, "filtered_nix_source_compiles_nested_vendor_and_json_fixtures"),
];
const UNRELATED: &str = "lattice.agent_orchestration.authority.scenario.ambient-denied";
const ADDED: &str = "lattice.example.unaccepted.scenario.added";
const DANGLING: &str = "lattice.example.dangling.scenario.marker";

fn receipt(source: &str) -> CoverageReceipt {
    serde_json::from_str(source).expect("coverage fixture must decode")
}

fn codes(issues: &[core::PromotionIssue]) -> BTreeSet<&str> {
    issues.iter().map(|issue| issue.code).collect()
}

// r[verify lattice.repository_validation.repository_checks_promotion]
// r[verify lattice.repository_validation.repository_checks_promotion.scenario.delta]
#[test]
fn exact_repository_validation_promotion_preserves_every_unrelated_gap() {
    let baseline = receipt(BASELINE);
    let current = receipt(CURRENT);
    let issues = validate_promotion(&baseline, &current, PROMOTED);
    assert!(issues.is_empty(), "issues: {issues:?}");
    assert_eq!(baseline.missing.len(), BASELINE_COUNT);
    assert_eq!(current.missing.len(), CURRENT_COUNT);
    assert_eq!(PROMOTED.len(), PROMOTED_COUNT);
    assert!(baseline.dangling.is_empty());
    assert!(current.dangling.is_empty());
    assert!(!baseline.valid);
    assert!(!current.valid);
}

// r[verify lattice.repository_validation.repository_checks_promotion.scenario.bindings]
#[test]
fn repository_validation_markers_are_bound_to_exact_tests_and_leave_policy() {
    let binding_issues = validate_marker_bindings(REPOSITORY_TESTS, BINDINGS);
    let policy_issues = validate_policy_excludes_promoted_ids(POLICY, PROMOTED);
    assert!(binding_issues.is_empty(), "binding issues: {binding_issues:?}");
    assert!(policy_issues.is_empty(), "policy issues: {policy_issues:?}");
}

// r[verify lattice.repository_validation.repository_checks_promotion.scenario.fail_closed]
#[test]
fn repository_validation_promotion_drift_fails_closed() {
    let mut baseline = receipt(BASELINE);
    baseline.missing.push(baseline.missing[0].clone());
    let mut current = receipt(CURRENT);
    current.missing.retain(|identifier| identifier != UNRELATED);
    current.missing.push(CLIPPY_ID.to_string());
    current.missing.push(CLIPPY_ID.to_string());
    current.missing.push(ADDED.to_string());
    current.dangling.push(DANGLING.to_string());
    current.valid = true;
    let allowlist = [
        CLIPPY_ID,
        CLIPPY_ID,
        CLIPPY_VENDOR_ID,
        FLAKE_ID,
        FLAKE_MISSING_KEY_ID,
        FLAKE_NORMAL_SIGNING_ID,
        FORMAT_ID,
        FORMAT_VENDOR_ID,
        TEST_SOURCE_ID,
        TEST_JSON_ID,
        VENDOR_ID,
        VENDOR_NESTED_ID,
    ];
    let promotion_issues = validate_promotion(&baseline, &current, &allowlist);
    let promotion_codes = codes(&promotion_issues);
    let binding_issues = validate_marker_bindings(REPOSITORY_TESTS, &[(CLIPPY_ID, "missing_test")]);
    let binding_codes = codes(&binding_issues);
    let stale_policy = format!("requirement_id = \"{VENDOR_NESTED_ID}\"");
    let policy_issues = validate_policy_excludes_promoted_ids(&stale_policy, PROMOTED);
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

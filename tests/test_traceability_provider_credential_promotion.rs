#[path = "traceability_promotion/core.rs"]
mod core;

use core::CoverageReceipt;
use core::validate_marker_bindings;
use core::validate_policy_excludes_promoted_ids;
use core::validate_promotion;
use std::collections::BTreeSet;

const BASELINE_COVERAGE: &str = include_str!("fixtures/traceability-provider-credential-completion-current.json");
const CURRENT_COVERAGE: &str = include_str!("fixtures/traceability-provider-credential-promotion-current.json");
const BOUNDARY_TEST_SOURCE: &str = include_str!("test_provider_credential_boundaries.rs");
const INVENTORY_POLICY: &str = include_str!("../cairn-policy/traceability-inventory.ncl");
const BASELINE_MISSING_COUNT: usize = 336;
const CURRENT_MISSING_COUNT: usize = 329;
const PROMOTED_COUNT: usize = 7;
const UNRELATED_MISSING_ID: &str = "lattice.agent_orchestration.authority.scenario.ambient-denied";
const UNEXPECTED_ADDED_ID: &str = "lattice.example.unaccepted.scenario.added";
const DANGLING_ID: &str = "lattice.example.dangling.scenario.marker";

const CACHE_ID: &str = concat!("n8n", "clone.credentials.provider_cache_identity.no_plaintext");
const ERROR_ID: &str = concat!("n8n", "clone.credentials.provider_diagnostics.redaction.error");
const PROBE_ID: &str = concat!("n8n", "clone.credentials.provider_diagnostics.redaction.test");
const COMMAND_ID: &str = concat!("n8n", "clone.credentials.providers.command.resolve");
const TIMEOUT_ID: &str = concat!("n8n", "clone.credentials.providers.command.timeout");
const VAULT_ID: &str = concat!("n8n", "clone.credentials.providers.local_vault.resolve");
const TAMPER_ID: &str = concat!("n8n", "clone.credentials.providers.local_vault.tamper");
const PROMOTED_IDS: &[&str] = &[
    CACHE_ID, ERROR_ID, PROBE_ID, COMMAND_ID, TIMEOUT_ID, VAULT_ID, TAMPER_ID,
];

const CACHE_TEST: &str = "cache_identity_includes_provider_key_and_field_but_not_resolved_values";
const COMMAND_TEST: &str = "command_provider_reaches_execution_and_timeout_blocks_side_effects";
const VAULT_TEST: &str = "local_vault_keeps_workflow_opaque_and_tamper_diagnostics_safe";
const PROBE_TEST: &str = "provider_probe_rendering_uses_fixed_safe_messages_and_opaque_identity";
const REQUIRED_BINDINGS: &[(&str, &str)] = &[
    (CACHE_ID, CACHE_TEST),
    (ERROR_ID, PROBE_TEST),
    (PROBE_ID, PROBE_TEST),
    (COMMAND_ID, COMMAND_TEST),
    (TIMEOUT_ID, COMMAND_TEST),
    (VAULT_ID, VAULT_TEST),
    (TAMPER_ID, VAULT_TEST),
];

fn decode_receipt(source: &str) -> CoverageReceipt {
    serde_json::from_str(source).expect("coverage fixture must decode")
}

fn issue_codes(issues: &[core::PromotionIssue]) -> BTreeSet<&str> {
    issues.iter().map(|issue| issue.code).collect()
}

// r[verify lattice.repository_validation.provider_credential_promotion]
// r[verify lattice.repository_validation.provider_credential_promotion.scenario.exact_delta]
#[test]
fn exact_provider_promotion_preserves_every_unrelated_gap() {
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

// r[verify lattice.repository_validation.provider_credential_promotion.scenario.local_binding]
#[test]
fn provider_markers_are_attached_to_exact_tests_and_leave_debt_policy() {
    let binding_issues = validate_marker_bindings(BOUNDARY_TEST_SOURCE, REQUIRED_BINDINGS);
    let policy_issues = validate_policy_excludes_promoted_ids(INVENTORY_POLICY, PROMOTED_IDS);

    assert!(binding_issues.is_empty(), "binding issues: {binding_issues:?}");
    assert!(policy_issues.is_empty(), "policy issues: {policy_issues:?}");
}

// r[verify lattice.repository_validation.provider_credential_promotion.scenario.fail_closed]
#[test]
fn duplicate_unbound_stale_dangling_and_successful_provider_transitions_fail_closed() {
    let mut baseline = decode_receipt(BASELINE_COVERAGE);
    baseline.missing.push(baseline.missing[0].clone());
    let mut current = decode_receipt(CURRENT_COVERAGE);
    current.missing.retain(|identifier| identifier != UNRELATED_MISSING_ID);
    current.missing.push(CACHE_ID.to_string());
    current.missing.push(CACHE_ID.to_string());
    current.missing.push(UNEXPECTED_ADDED_ID.to_string());
    current.dangling.push(DANGLING_ID.to_string());
    current.valid = true;
    let allowlist = [
        CACHE_ID, CACHE_ID, ERROR_ID, PROBE_ID, COMMAND_ID, TIMEOUT_ID, VAULT_ID, TAMPER_ID,
    ];

    let promotion_issues = validate_promotion(&baseline, &current, &allowlist);
    let promotion_codes = issue_codes(&promotion_issues);
    let binding_issues = validate_marker_bindings(BOUNDARY_TEST_SOURCE, &[(CACHE_ID, "missing_test")]);
    let binding_codes = issue_codes(&binding_issues);
    let stale_policy = format!("requirement_id = \"{TAMPER_ID}\"");
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

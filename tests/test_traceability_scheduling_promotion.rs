#[path = "traceability_promotion/core.rs"]
mod core;

use core::CoverageReceipt;
use core::validate_marker_bindings;
use core::validate_policy_excludes_promoted_ids;
use core::validate_promotion;
use std::collections::BTreeSet;

const BASELINE: &str = include_str!("fixtures/traceability-scheduling-completion-current.json");
const CURRENT: &str = include_str!("fixtures/traceability-scheduling-promotion-current.json");
const TRIGGER_TESTS: &str = include_str!("test_triggers.rs");
const WEBHOOK_TESTS: &str = include_str!("webhooks.rs");
const POLICY: &str = include_str!("../cairn-policy/traceability-inventory.ncl");
const BASELINE_COUNT: usize = 324;
const CURRENT_COUNT: usize = 317;
const PROMOTED_COUNT: usize = 7;
const DISABLED_ID: &str = concat!("n8n", "clone.scheduling.triggers.admission.disabled");
const HISTORY_ID: &str = concat!("n8n", "clone.scheduling.triggers.history.persist");
const COMPAT_ID: &str = concat!("n8n", "clone.scheduling.triggers.scheduler_compat.due");
const INVALID_ID: &str = concat!("n8n", "clone.scheduling.triggers.webhook.invalid");
const VALID_ID: &str = concat!("n8n", "clone.scheduling.triggers.webhook.valid");
const COOKBOOK_ID: &str = concat!("n8n", "clone.scheduling.webhook_cookbook_operator_flow");
const LISTENER_ID: &str = concat!("n8n", "clone.scheduling.webhook_listener.process.start");
const PROMOTED: &[&str] = &[
    DISABLED_ID,
    HISTORY_ID,
    COMPAT_ID,
    INVALID_ID,
    VALID_ID,
    COOKBOOK_ID,
    LISTENER_ID,
];
const TRIGGER_BINDINGS: &[(&str, &str)] = &[
    (VALID_ID, "payload_binding_supports_json_query_header_and_raw_body_failures"),
    (INVALID_ID, "webhook_admission_checks_method_content_type_and_redacted_auth"),
    (VALID_ID, "webhook_admission_checks_method_content_type_and_redacted_auth"),
    (COMPAT_ID, "schedule_trigger_preserves_schedule_semantics"),
    (HISTORY_ID, "admitted_trigger_plan_starts_headless_run_and_records_history"),
    (VALID_ID, "admitted_trigger_plan_starts_headless_run_and_records_history"),
];
const WEBHOOK_BINDINGS: &[(&str, &str)] = &[
    (DISABLED_ID, "missing_path_disabled_bad_method_bad_content_and_malformed_body_reject_without_runs"),
    (INVALID_ID, "missing_path_disabled_bad_method_bad_content_and_malformed_body_reject_without_runs"),
    (LISTENER_ID, "operator_listener_loads_persisted_trigger_and_accepts_loopback_request"),
    (COOKBOOK_ID, "webhook_cookbook_import_persists_trigger_and_dry_runs"),
    (COOKBOOK_ID, "webhook_cookbook_valid_request_starts_one_run_and_records_safe_history"),
    (COOKBOOK_ID, "webhook_cookbook_rejected_requests_do_not_start_runs_or_expose_secrets"),
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

// r[verify lattice.repository_validation.scheduling_promotion]
// r[verify lattice.repository_validation.scheduling_promotion.scenario.delta]
#[test]
fn exact_scheduling_promotion_preserves_every_unrelated_gap() {
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

// r[verify lattice.repository_validation.scheduling_promotion.scenario.bindings]
#[test]
fn scheduling_markers_are_bound_to_exact_tests_and_leave_policy() {
    let trigger_issues = validate_marker_bindings(TRIGGER_TESTS, TRIGGER_BINDINGS);
    let webhook_issues = validate_marker_bindings(WEBHOOK_TESTS, WEBHOOK_BINDINGS);
    let policy_issues = validate_policy_excludes_promoted_ids(POLICY, PROMOTED);
    assert!(trigger_issues.is_empty(), "trigger binding issues: {trigger_issues:?}");
    assert!(webhook_issues.is_empty(), "webhook binding issues: {webhook_issues:?}");
    assert!(policy_issues.is_empty(), "policy issues: {policy_issues:?}");
}

// r[verify lattice.repository_validation.scheduling_promotion.scenario.fail_closed]
#[test]
fn scheduling_promotion_drift_fails_closed() {
    let mut baseline = receipt(BASELINE);
    baseline.missing.push(baseline.missing[0].clone());
    let mut current = receipt(CURRENT);
    current.missing.retain(|id| id != UNRELATED);
    current.missing.push(DISABLED_ID.to_string());
    current.missing.push(DISABLED_ID.to_string());
    current.missing.push(ADDED.to_string());
    current.dangling.push(DANGLING.to_string());
    current.valid = true;
    let allowlist = [
        DISABLED_ID,
        DISABLED_ID,
        HISTORY_ID,
        COMPAT_ID,
        INVALID_ID,
        VALID_ID,
        COOKBOOK_ID,
        LISTENER_ID,
    ];
    let promotion_issues = validate_promotion(&baseline, &current, &allowlist);
    let promotion_codes = codes(&promotion_issues);
    let binding_issues = validate_marker_bindings(TRIGGER_TESTS, &[(HISTORY_ID, "missing_test")]);
    let binding_codes = codes(&binding_issues);
    let stale_policy = format!("requirement_id = \"{LISTENER_ID}\"");
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

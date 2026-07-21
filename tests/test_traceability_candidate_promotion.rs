#[path = "traceability_promotion/core.rs"]
mod core;

use core::CoverageReceipt;
use core::marker_is_attached_to_test;
use core::validate_promotion;
use std::collections::BTreeSet;

const BASELINE_COVERAGE: &str = include_str!("fixtures/traceability-candidate-promotion-baseline.json");
const CURRENT_COVERAGE: &str = include_str!("fixtures/traceability-candidate-promotion-current.json");
const INSPECTION_TESTS: &str = include_str!("test_run_inspection.rs");
const RECOVERY_TESTS: &str = include_str!("test_run_store_recovery.rs");
const INVENTORY_POLICY: &str = include_str!("../cairn-policy/traceability-inventory.ncl");
const BASELINE_MISSING_COUNT: usize = 367;
const CURRENT_MISSING_COUNT: usize = 361;
const PROMOTED_COUNT: usize = 6;
const UNRELATED_MISSING_ID: &str = "lattice.agent_orchestration.authority.scenario.ambient-denied";
const UNEXPECTED_ADDED_ID: &str = "lattice.example.unaccepted.scenario.added";
const DANGLING_ID: &str = "lattice.example.dangling.scenario.marker";
const PROMOTED_IDS: &[&str] = &[
    "lattice.workflow_execution.run_store_recovery.scenario.non-claim",
    "lattice.workflow_execution.safe_run_inspection.scenario.missing",
    "lattice.workflow_execution.safe_run_inspection.scenario.not_recorded",
    "lattice.workflow_execution.safe_run_inspection.scenario.owner_binding",
    "lattice.workflow_execution.safe_run_inspection.scenario.replay",
    "lattice.workflow_execution.safe_run_inspection.scenario.safe_output",
];

struct Attachment {
    source: &'static str,
    test_name: &'static str,
    requirement_id: &'static str,
}

const REQUIRED_ATTACHMENTS: &[Attachment] = &[
    Attachment {
        source: INSPECTION_TESTS,
        test_name: "safe_projection_omits_arbitrary_persisted_content",
        requirement_id: "lattice.workflow_execution.safe_run_inspection.scenario.safe_output",
    },
    Attachment {
        source: INSPECTION_TESTS,
        test_name: "safe_projection_omits_arbitrary_persisted_content",
        requirement_id: "lattice.workflow_execution.safe_run_inspection.scenario.replay",
    },
    Attachment {
        source: INSPECTION_TESTS,
        test_name: "missing_optional_history_is_explicit_without_invalidating_legacy_run",
        requirement_id: "lattice.workflow_execution.safe_run_inspection.scenario.not_recorded",
    },
    Attachment {
        source: INSPECTION_TESTS,
        test_name: "present_foreign_detail_and_replay_identity_fail_closed",
        requirement_id: "lattice.workflow_execution.safe_run_inspection.scenario.replay",
    },
    Attachment {
        source: INSPECTION_TESTS,
        test_name: "exact_orchestration_owner_reports_finished_dispatch_and_settled_budget",
        requirement_id: "lattice.workflow_execution.safe_run_inspection.scenario.owner_binding",
    },
    Attachment {
        source: INSPECTION_TESTS,
        test_name: "foreign_dispatch_owner_and_duplicate_admission_are_invalid",
        requirement_id: "lattice.workflow_execution.safe_run_inspection.scenario.owner_binding",
    },
    Attachment {
        source: INSPECTION_TESTS,
        test_name: "cli_renders_safe_text_and_json_for_persisted_run",
        requirement_id: "lattice.workflow_execution.safe_run_inspection.scenario.safe_output",
    },
    Attachment {
        source: INSPECTION_TESTS,
        test_name: "cli_reports_exact_orchestration_binding_without_private_content",
        requirement_id: "lattice.workflow_execution.safe_run_inspection.scenario.safe_output",
    },
    Attachment {
        source: INSPECTION_TESTS,
        test_name: "cli_reports_exact_orchestration_binding_without_private_content",
        requirement_id: "lattice.workflow_execution.safe_run_inspection.scenario.owner_binding",
    },
    Attachment {
        source: INSPECTION_TESTS,
        test_name: "cli_invalid_replay_exits_unsuccessfully_without_summaries",
        requirement_id: "lattice.workflow_execution.safe_run_inspection.scenario.replay",
    },
    Attachment {
        source: INSPECTION_TESTS,
        test_name: "cli_missing_run_is_not_synthesized_from_orchestration_history",
        requirement_id: "lattice.workflow_execution.safe_run_inspection.scenario.missing",
    },
    Attachment {
        source: RECOVERY_TESTS,
        test_name: "panic_and_abort_require_explicit_repair_before_safe_inspection",
        requirement_id: "lattice.workflow_execution.run_store_recovery.scenario.non-claim",
    },
];

fn decode_receipt(source: &str) -> CoverageReceipt {
    serde_json::from_str(source).expect("coverage fixture must decode")
}

fn issue_codes(issues: &[core::PromotionIssue]) -> BTreeSet<&str> {
    issues.iter().map(|issue| issue.code).collect()
}

// r[verify lattice.repository_validation.reviewed_tracey_promotion.scenario.exact_delta]
#[test]
fn exact_six_id_promotion_preserves_every_unrelated_gap() {
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

// r[verify lattice.repository_validation.reviewed_tracey_promotion.scenario.local_binding]
#[test]
fn promoted_markers_are_attached_to_exact_tests_and_leave_the_debt_policy() {
    for attachment in REQUIRED_ATTACHMENTS {
        assert!(
            marker_is_attached_to_test(attachment.source, attachment.test_name, attachment.requirement_id),
            "marker {} was not attached to {}",
            attachment.requirement_id,
            attachment.test_name
        );
    }
    for promoted in PROMOTED_IDS {
        assert!(!INVENTORY_POLICY.contains(promoted), "promoted ID remained in debt policy: {promoted}");
    }
}

// r[verify lattice.repository_validation.reviewed_tracey_promotion.scenario.fail_closed]
#[test]
fn duplicate_unexpected_dangling_and_successful_transitions_fail_closed() {
    let mut baseline = decode_receipt(BASELINE_COVERAGE);
    baseline.missing.push(baseline.missing[0].clone());
    let mut current = decode_receipt(CURRENT_COVERAGE);
    current.missing.retain(|identifier| identifier != UNRELATED_MISSING_ID);
    current.missing.push(PROMOTED_IDS[0].to_string());
    current.missing.push(PROMOTED_IDS[0].to_string());
    current.missing.push(UNEXPECTED_ADDED_ID.to_string());
    current.dangling.push(DANGLING_ID.to_string());
    current.valid = true;
    let mut allowlist = PROMOTED_IDS.to_vec();
    allowlist.push(PROMOTED_IDS[0]);

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

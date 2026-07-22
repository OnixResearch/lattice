#[path = "traceability_promotion/core.rs"]
mod core;

use core::CoverageReceipt;
use core::marker_is_attached_to_test;
use core::validate_promotion;
use std::collections::BTreeSet;

const BASELINE_COVERAGE: &str = include_str!("fixtures/traceability-operator-diagnostics-current.json");
const CURRENT_COVERAGE: &str = include_str!("fixtures/traceability-operator-diagnostics-promotion-current.json");
const DOCTOR_CORE_SOURCE: &str = include_str!("../src/doctor/core.rs");
const INVENTORY_POLICY: &str = include_str!("../cairn-policy/traceability-inventory.ncl");
const BASELINE_MISSING_COUNT: usize = 361;
const CURRENT_MISSING_COUNT: usize = 353;
const PROMOTED_COUNT: usize = 8;
const UNRELATED_MISSING_ID: &str = "lattice.agent_orchestration.authority.scenario.ambient-denied";
const UNEXPECTED_ADDED_ID: &str = "lattice.example.unaccepted.scenario.added";
const DANGLING_ID: &str = "lattice.example.dangling.scenario.marker";
const DOCTOR_CORE_ID: &str = concat!("n8n", "clone.operator_diagnostics.doctor_core");
const DOCTOR_CORE_FATAL_ID: &str = concat!("n8n", "clone.operator_diagnostics.doctor_core.fatal");
const DOCTOR_CORE_HEALTHY_ID: &str = concat!("n8n", "clone.operator_diagnostics.doctor_core.healthy");
const DOCTOR_OUTPUT_ID: &str = concat!("n8n", "clone.operator_diagnostics.doctor_output");
const DOCTOR_OUTPUT_JSON_ID: &str = concat!("n8n", "clone.operator_diagnostics.doctor_output.json");
const DOCTOR_OUTPUT_TEXT_ID: &str = concat!("n8n", "clone.operator_diagnostics.doctor_output.text");
const DOCTOR_REDACTION_ID: &str = concat!("n8n", "clone.operator_diagnostics.doctor_redaction");
const DOCTOR_REDACTION_JSON_ID: &str = concat!("n8n", "clone.operator_diagnostics.doctor_redaction.json");
const PROMOTED_IDS: &[&str] = &[
    DOCTOR_CORE_ID,
    DOCTOR_CORE_FATAL_ID,
    DOCTOR_CORE_HEALTHY_ID,
    DOCTOR_OUTPUT_ID,
    DOCTOR_OUTPUT_JSON_ID,
    DOCTOR_OUTPUT_TEXT_ID,
    DOCTOR_REDACTION_ID,
    DOCTOR_REDACTION_JSON_ID,
];

struct Attachment {
    test_name: &'static str,
    requirement_id: &'static str,
}

const REQUIRED_ATTACHMENTS: &[Attachment] = &[
    Attachment {
        test_name: "healthy_facts_produce_a_deterministic_healthy_report",
        requirement_id: DOCTOR_CORE_ID,
    },
    Attachment {
        test_name: "healthy_facts_produce_a_deterministic_healthy_report",
        requirement_id: DOCTOR_CORE_HEALTHY_ID,
    },
    Attachment {
        test_name: "every_enumerated_failing_fact_produces_a_stable_safe_finding",
        requirement_id: DOCTOR_CORE_FATAL_ID,
    },
    Attachment {
        test_name: "mixed_findings_render_as_exact_deterministic_actionable_text",
        requirement_id: DOCTOR_OUTPUT_ID,
    },
    Attachment {
        test_name: "mixed_findings_render_as_exact_deterministic_actionable_text",
        requirement_id: DOCTOR_OUTPUT_TEXT_ID,
    },
    Attachment {
        test_name: "mixed_findings_render_as_exact_deterministic_automation_safe_json",
        requirement_id: DOCTOR_OUTPUT_JSON_ID,
    },
    Attachment {
        test_name: "sensitive_fact_values_are_absent_from_text_and_json",
        requirement_id: DOCTOR_REDACTION_ID,
    },
    Attachment {
        test_name: "sensitive_fact_values_are_absent_from_text_and_json",
        requirement_id: DOCTOR_REDACTION_JSON_ID,
    },
];

fn decode_receipt(source: &str) -> CoverageReceipt {
    serde_json::from_str(source).expect("coverage fixture must decode")
}

fn issue_codes(issues: &[core::PromotionIssue]) -> BTreeSet<&str> {
    issues.iter().map(|issue| issue.code).collect()
}

#[test]
fn exact_eight_id_promotion_preserves_every_unrelated_gap() {
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

#[test]
fn promoted_markers_are_attached_to_exact_tests_and_leave_the_debt_policy() {
    for attachment in REQUIRED_ATTACHMENTS {
        assert!(
            marker_is_attached_to_test(DOCTOR_CORE_SOURCE, attachment.test_name, attachment.requirement_id),
            "marker {} was not attached to {}",
            attachment.requirement_id,
            attachment.test_name
        );
    }
    for promoted in PROMOTED_IDS {
        assert!(!INVENTORY_POLICY.contains(promoted), "promoted ID remained in debt policy: {promoted}");
    }
}

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

#[path = "traceability_review/core.rs"]
mod core;

use core::CoverageSnapshot;
use core::ExpectedDecision;
use core::ReviewRow;
use core::validate_no_selected_markers;
use core::validate_tranche;
use core::validate_unchanged_coverage;
use std::collections::BTreeSet;

const BASELINE_COVERAGE: &str = include_str!("fixtures/traceability-node-template-prompt-promotion-current.json");
const CURRENT_COVERAGE: &str = include_str!("fixtures/traceability-scheduling-audit-current.json");
const REVIEWED_INVENTORY: &str = include_str!("fixtures/traceability-scheduling-audit-reviewed.json");
const TRIGGER_TESTS: &str = include_str!("test_triggers.rs");
const WEBHOOK_TESTS: &str = include_str!("webhooks.rs");
const HEADLESS_SOURCE: &str = include_str!("../src/headless.rs");
const TRIGGER_SOURCE: &str = include_str!("../src/triggers.rs");
const MAIN_SOURCE: &str = include_str!("../src/main.rs");
const EXAMPLES_README: &str = include_str!("../examples/README.md");
const COOKBOOK_WORKFLOW: &str = include_str!("../examples/workflows/webhook-cookbook.ncl");
const TRIGGER_TESTS_PATH: &str = "tests/test_triggers.rs";
const WEBHOOK_TESTS_PATH: &str = "tests/webhooks.rs";
const HEADLESS_SOURCE_PATH: &str = "src/headless.rs";
const TRIGGER_SOURCE_PATH: &str = "src/triggers.rs";
const MAIN_SOURCE_PATH: &str = "src/main.rs";
const EXAMPLES_README_PATH: &str = "examples/README.md";
const COOKBOOK_WORKFLOW_PATH: &str = "examples/workflows/webhook-cookbook.ncl";
const SUBSYSTEM: &str = "scheduling";
const EXACT: &str = "exact_maintained_evidence";
const PARTIAL: &str = "partial_evidence_only";
const NO_EXACT: &str = "no_exact_evidence_identified";
const SELECTED_COUNT: usize = 7;
const DISABLED_ID: &str = concat!("n8n", "clone.scheduling.triggers.admission.disabled");
const HISTORY_ID: &str = concat!("n8n", "clone.scheduling.triggers.history.persist");
const COMPAT_ID: &str = concat!("n8n", "clone.scheduling.triggers.scheduler_compat.due");
const INVALID_ID: &str = concat!("n8n", "clone.scheduling.triggers.webhook.invalid");
const VALID_ID: &str = concat!("n8n", "clone.scheduling.triggers.webhook.valid");
const COOKBOOK_ID: &str = concat!("n8n", "clone.scheduling.webhook_cookbook_operator_flow");
const LISTENER_ID: &str = concat!("n8n", "clone.scheduling.webhook_listener.process.start");
const SELECTED_IDS: &[&str] = &[
    DISABLED_ID,
    HISTORY_ID,
    COMPAT_ID,
    INVALID_ID,
    VALID_ID,
    COOKBOOK_ID,
    LISTENER_ID,
];
const UNRELATED_MISSING_ID: &str = "lattice.agent_orchestration.authority.scenario.ambient-denied";
const UNEXPECTED_ADDED_ID: &str = "lattice.example.unaccepted.scenario.added";
const DANGLING_ID: &str = "lattice.example.dangling.scenario.marker";
const UNEXPECTED_REVIEW_ID: &str = concat!("n8n", "clone.scheduling.unexpected.audit-row");

#[derive(Debug, Clone, serde::Deserialize)]
struct InventoryReport {
    rows: Vec<InventoryRow>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct InventoryRow {
    requirement_id: String,
    subsystem: String,
    classification: String,
    evidence: Vec<EvidenceReference>,
    rationale_code: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct EvidenceReference {
    path: String,
    blake3: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct CoverageReceipt {
    missing: Vec<String>,
    dangling: Vec<String>,
    valid: bool,
}

fn expected_decisions() -> Vec<ExpectedDecision> {
    vec![
        ExpectedDecision {
            requirement_id: DISABLED_ID,
            classification: EXACT,
            rationale_code: "assertions.disabled-rejected-without-run",
            evidence_paths: &[WEBHOOK_TESTS_PATH],
        },
        ExpectedDecision {
            requirement_id: HISTORY_ID,
            classification: PARTIAL,
            rationale_code: "gap.terminal-status-and-complete-fields-unasserted",
            evidence_paths: &[HEADLESS_SOURCE_PATH, TRIGGER_SOURCE_PATH, TRIGGER_TESTS_PATH],
        },
        ExpectedDecision {
            requirement_id: COMPAT_ID,
            classification: PARTIAL,
            rationale_code: "gap.full-old-new-plan-comparison-absent",
            evidence_paths: &[TRIGGER_TESTS_PATH],
        },
        ExpectedDecision {
            requirement_id: INVALID_ID,
            classification: EXACT,
            rationale_code: "assertions.invalid-webhooks-reject-without-run",
            evidence_paths: &[TRIGGER_TESTS_PATH, WEBHOOK_TESTS_PATH],
        },
        ExpectedDecision {
            requirement_id: VALID_ID,
            classification: EXACT,
            rationale_code: "assertions.valid-webhook-binds-and-runs",
            evidence_paths: &[TRIGGER_TESTS_PATH, WEBHOOK_TESTS_PATH],
        },
        ExpectedDecision {
            requirement_id: COOKBOOK_ID,
            classification: EXACT,
            rationale_code: "assertions.cookbook-import-dry-run-admit-reject-history",
            evidence_paths: &[EXAMPLES_README_PATH, COOKBOOK_WORKFLOW_PATH, WEBHOOK_TESTS_PATH],
        },
        ExpectedDecision {
            requirement_id: LISTENER_ID,
            classification: PARTIAL,
            rationale_code: "gap.live-process-socket-acceptance-unasserted",
            evidence_paths: &[MAIN_SOURCE_PATH, WEBHOOK_TESTS_PATH],
        },
    ]
}

fn review_rows(report: &InventoryReport) -> Vec<ReviewRow> {
    report
        .rows
        .iter()
        .map(|row| ReviewRow {
            requirement_id: row.requirement_id.clone(),
            subsystem: row.subsystem.clone(),
            classification: row.classification.clone(),
            rationale_code: row.rationale_code.clone(),
            evidence_paths: row.evidence.iter().map(|evidence| evidence.path.clone()).collect(),
        })
        .collect()
}

fn coverage(source: &str) -> CoverageSnapshot {
    let receipt: CoverageReceipt = serde_json::from_str(source).expect("coverage fixture must decode");
    CoverageSnapshot {
        missing: receipt.missing,
        dangling: receipt.dangling,
        valid: receipt.valid,
    }
}

fn issue_codes(issues: &[core::ReviewIssue]) -> BTreeSet<&str> {
    issues.iter().map(|issue| issue.code).collect()
}

fn source_by_path(path: &str) -> &'static str {
    match path {
        TRIGGER_TESTS_PATH => TRIGGER_TESTS,
        WEBHOOK_TESTS_PATH => WEBHOOK_TESTS,
        HEADLESS_SOURCE_PATH => HEADLESS_SOURCE,
        TRIGGER_SOURCE_PATH => TRIGGER_SOURCE,
        MAIN_SOURCE_PATH => MAIN_SOURCE,
        EXAMPLES_README_PATH => EXAMPLES_README,
        COOKBOOK_WORKFLOW_PATH => COOKBOOK_WORKFLOW,
        _ => panic!("unexpected evidence path: {path}"),
    }
}

// r[verify lattice.repository_validation.scheduling_audit]
// r[verify lattice.repository_validation.scheduling_audit.scenario.complete]
// r[verify lattice.repository_validation.scheduling_audit.scenario.evidence]
#[test]
fn scheduling_audit_is_complete_byte_bound_and_unpromoted() {
    let report: InventoryReport = serde_json::from_str(REVIEWED_INVENTORY).expect("inventory must decode");
    let review_issues = validate_tranche(&review_rows(&report), SUBSYSTEM, &expected_decisions());
    let coverage_issues = validate_unchanged_coverage(&coverage(BASELINE_COVERAGE), &coverage(CURRENT_COVERAGE));
    let sources = [
        TRIGGER_TESTS,
        WEBHOOK_TESTS,
        HEADLESS_SOURCE,
        TRIGGER_SOURCE,
        MAIN_SOURCE,
        EXAMPLES_README,
        COOKBOOK_WORKFLOW,
    ];
    let marker_issues = validate_no_selected_markers(&sources, SELECTED_IDS);
    let selected_rows = report.rows.iter().filter(|row| SELECTED_IDS.contains(&row.requirement_id.as_str()));

    assert!(review_issues.is_empty(), "review issues: {review_issues:?}");
    assert!(coverage_issues.is_empty(), "coverage issues: {coverage_issues:?}");
    assert!(marker_issues.is_empty(), "marker issues: {marker_issues:?}");
    assert_eq!(selected_rows.count(), SELECTED_COUNT);
    for row in report.rows.iter().filter(|row| SELECTED_IDS.contains(&row.requirement_id.as_str())) {
        for evidence in &row.evidence {
            assert_eq!(blake3::hash(source_by_path(&evidence.path).as_bytes()).to_hex().to_string(), evidence.blake3);
        }
    }
}

// r[verify lattice.repository_validation.scheduling_audit.scenario.fail_closed]
#[test]
fn scheduling_audit_classification_marker_evidence_and_coverage_drift_fail_closed() {
    let report: InventoryReport = serde_json::from_str(REVIEWED_INVENTORY).expect("inventory must decode");
    let mut rows = review_rows(&report);
    rows.retain(|row| row.requirement_id != DISABLED_ID);
    let history = rows.iter_mut().find(|row| row.requirement_id == HISTORY_ID).expect("history row");
    history.classification = EXACT.to_string();
    history.rationale_code = "invalid.rationale".to_string();
    history.evidence_paths = vec!["../unsafe".to_string()];
    rows.push(ReviewRow {
        requirement_id: UNEXPECTED_REVIEW_ID.to_string(),
        subsystem: SUBSYSTEM.to_string(),
        classification: NO_EXACT.to_string(),
        rationale_code: "invalid.unexpected".to_string(),
        evidence_paths: Vec::new(),
    });
    let review_issues = validate_tranche(&rows, SUBSYSTEM, &expected_decisions());
    let review_codes = issue_codes(&review_issues);

    let promoted = format!("// r[verify {DISABLED_ID}]");
    let marker_issues = validate_no_selected_markers(&[promoted.as_str()], &[DISABLED_ID, DISABLED_ID]);
    let marker_codes = issue_codes(&marker_issues);

    let baseline = coverage(BASELINE_COVERAGE);
    let mut current = coverage(CURRENT_COVERAGE);
    current.missing.retain(|identifier| identifier != UNRELATED_MISSING_ID);
    current.missing.push(UNEXPECTED_ADDED_ID.to_string());
    current.missing.push(UNEXPECTED_ADDED_ID.to_string());
    current.dangling.push(DANGLING_ID.to_string());
    current.valid = true;
    let coverage_issues = validate_unchanged_coverage(&baseline, &current);
    let coverage_codes = issue_codes(&coverage_issues);

    for code in [
        "review.missing",
        "review.unexpected",
        "review.classification",
        "review.rationale",
        "review.evidence_paths",
    ] {
        assert!(review_codes.contains(code));
    }
    assert!(marker_codes.contains("marker.promoted"));
    assert!(marker_codes.contains("marker.duplicate_selected"));
    for code in [
        "coverage.removed",
        "coverage.added",
        "coverage.current_duplicate",
        "coverage.current_unsorted",
        "coverage.current_dangling",
        "coverage.current_valid",
    ] {
        assert!(coverage_codes.contains(code));
    }
}

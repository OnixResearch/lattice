#[path = "traceability_review/core.rs"]
mod core;

use core::CoverageSnapshot;
use core::ExpectedDecision;
use core::ReviewRow;
use core::validate_no_selected_markers;
use core::validate_tranche;
use core::validate_unchanged_coverage;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

const BASELINE_COVERAGE: &str = include_str!("fixtures/traceability-provider-credential-promotion-current.json");
const CURRENT_COVERAGE: &str = include_str!("fixtures/traceability-webhook-auth-completion-current.json");
const REVIEWED_INVENTORY: &str = include_str!("fixtures/traceability-webhook-auth-completion-reviewed.json");
const HEADLESS_SOURCE: &str = include_str!("fixtures/traceability-reviewed-headless-source.txt");
const STORE_SOURCE: &str = include_str!("../src/store/mod.rs");
const TRIGGER_SOURCE: &str = include_str!("../src/triggers.rs");
const WEBHOOK_CONTROLS_SOURCE: &str = include_str!("../src/webhooks/controls.rs");
const WEBHOOK_FLOW_SOURCE: &str = include_str!("../src/webhooks/flow.rs");
const BOUNDARY_TESTS: &str = include_str!("test_webhook_credential_boundaries.rs");

const HEADLESS_SOURCE_PATH: &str = "src/headless.rs";
const STORE_SOURCE_PATH: &str = "src/store/mod.rs";
const TRIGGER_SOURCE_PATH: &str = "src/triggers.rs";
const WEBHOOK_CONTROLS_SOURCE_PATH: &str = "src/webhooks/controls.rs";
const WEBHOOK_FLOW_SOURCE_PATH: &str = "src/webhooks/flow.rs";
const BOUNDARY_TESTS_PATH: &str = "tests/test_webhook_credential_boundaries.rs";
const SUBSYSTEM: &str = "credentials";
const EXACT_CLASSIFICATION: &str = "exact_maintained_evidence";
const UNASSESSED_CLASSIFICATION: &str = "unassessed";
const TOTAL_COUNT: usize = 329;
const REVIEWED_COUNT: usize = 2;
const EXACT_COUNT: usize = 2;
const UNASSESSED_COUNT: usize = 327;
const UNRELATED_MISSING_ID: &str = "lattice.agent_orchestration.authority.scenario.ambient-denied";
const UNEXPECTED_ADDED_ID: &str = "lattice.example.unaccepted.scenario.added";
const DANGLING_ID: &str = "lattice.example.dangling.scenario.marker";
const UNEXPECTED_REVIEW_ID: &str = concat!("n8n", "clone.credentials.unexpected.webhook-completion");
const REDACTION_ID: &str = concat!("n8n", "clone.credentials.webhook_auth.redaction");
const VALID_ID: &str = concat!("n8n", "clone.credentials.webhook_auth.valid");
const SELECTED_IDS: &[&str] = &[REDACTION_ID, VALID_ID];
const REDACTION_MARKER: &str = concat!("// r[verify n8n", "clone.credentials.webhook_auth.redaction]\n");
const VALID_MARKER: &str = concat!("// r[verify n8n", "clone.credentials.webhook_auth.valid]\n");
const MARKER_COUNT: usize = 1;
const REDACTION_EVIDENCE: &[&str] = &[
    STORE_SOURCE_PATH,
    TRIGGER_SOURCE_PATH,
    WEBHOOK_CONTROLS_SOURCE_PATH,
    WEBHOOK_FLOW_SOURCE_PATH,
    BOUNDARY_TESTS_PATH,
];
const VALID_EVIDENCE: &[&str] = &[
    HEADLESS_SOURCE_PATH,
    STORE_SOURCE_PATH,
    TRIGGER_SOURCE_PATH,
    WEBHOOK_FLOW_SOURCE_PATH,
    BOUNDARY_TESTS_PATH,
];

#[derive(Debug, Clone, serde::Deserialize)]
struct InventoryReport {
    summary: InventorySummary,
    rows: Vec<InventoryRow>,
    issues: Vec<serde_json::Value>,
    valid: bool,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct InventorySummary {
    total: usize,
    reviewed: usize,
    unassessed: usize,
    by_classification: BTreeMap<String, usize>,
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
            requirement_id: REDACTION_ID,
            classification: EXACT_CLASSIFICATION,
            rationale_code: "assertions.fingerprinted-context-render-and-raw-store",
            evidence_paths: REDACTION_EVIDENCE,
        },
        ExpectedDecision {
            requirement_id: VALID_ID,
            classification: EXACT_CLASSIFICATION,
            rationale_code: "assertions.valid-admission-history-and-raw-store",
            evidence_paths: VALID_EVIDENCE,
        },
    ]
}

fn selected_review_rows(report: &InventoryReport) -> Vec<ReviewRow> {
    report
        .rows
        .iter()
        .filter(|row| SELECTED_IDS.contains(&row.requirement_id.as_str()))
        .map(|row| ReviewRow {
            requirement_id: row.requirement_id.clone(),
            subsystem: row.subsystem.clone(),
            classification: row.classification.clone(),
            rationale_code: row.rationale_code.clone(),
            evidence_paths: row.evidence.iter().map(|evidence| evidence.path.clone()).collect(),
        })
        .collect()
}

fn coverage_snapshot(source: &str) -> CoverageSnapshot {
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

fn historical_boundary_tests() -> String {
    assert_eq!(BOUNDARY_TESTS.matches(REDACTION_MARKER).count(), MARKER_COUNT);
    assert_eq!(BOUNDARY_TESTS.matches(VALID_MARKER).count(), MARKER_COUNT);
    BOUNDARY_TESTS.replace(REDACTION_MARKER, "").replace(VALID_MARKER, "")
}

fn actual_evidence_hashes() -> BTreeMap<&'static str, String> {
    let historical_boundary_tests = historical_boundary_tests();
    BTreeMap::from([
        (HEADLESS_SOURCE_PATH, blake3::hash(HEADLESS_SOURCE.as_bytes()).to_hex().to_string()),
        (STORE_SOURCE_PATH, blake3::hash(STORE_SOURCE.as_bytes()).to_hex().to_string()),
        (TRIGGER_SOURCE_PATH, blake3::hash(TRIGGER_SOURCE.as_bytes()).to_hex().to_string()),
        (WEBHOOK_CONTROLS_SOURCE_PATH, blake3::hash(WEBHOOK_CONTROLS_SOURCE.as_bytes()).to_hex().to_string()),
        (WEBHOOK_FLOW_SOURCE_PATH, blake3::hash(WEBHOOK_FLOW_SOURCE.as_bytes()).to_hex().to_string()),
        (BOUNDARY_TESTS_PATH, blake3::hash(historical_boundary_tests.as_bytes()).to_hex().to_string()),
    ])
}

// r[verify lattice.repository_validation.webhook_auth_completion]
// r[verify lattice.repository_validation.webhook_auth_completion.scenario.context]
// r[verify lattice.repository_validation.webhook_auth_completion.scenario.boundaries]
// r[verify lattice.repository_validation.webhook_auth_completion.scenario.exact]
#[test]
fn two_webhook_auth_rows_retain_their_historical_exact_review() {
    let report: InventoryReport = serde_json::from_str(REVIEWED_INVENTORY).expect("inventory must decode");
    let rows = selected_review_rows(&report);
    let review_issues = validate_tranche(&rows, SUBSYSTEM, &expected_decisions());
    let baseline = coverage_snapshot(BASELINE_COVERAGE);
    let current = coverage_snapshot(CURRENT_COVERAGE);
    let coverage_issues = validate_unchanged_coverage(&baseline, &current);
    let marker_issues = validate_no_selected_markers(
        &[
            HEADLESS_SOURCE,
            STORE_SOURCE,
            TRIGGER_SOURCE,
            WEBHOOK_CONTROLS_SOURCE,
            WEBHOOK_FLOW_SOURCE,
            historical_boundary_tests().as_str(),
        ],
        SELECTED_IDS,
    );
    let hashes = actual_evidence_hashes();
    let current_missing = current.missing.iter().map(String::as_str).collect::<BTreeSet<_>>();

    assert!(review_issues.is_empty(), "review issues: {review_issues:?}");
    assert!(coverage_issues.is_empty(), "coverage issues: {coverage_issues:?}");
    assert!(marker_issues.is_empty(), "marker issues: {marker_issues:?}");
    assert!(report.valid);
    assert!(report.issues.is_empty());
    assert_eq!(report.summary.total, TOTAL_COUNT);
    assert_eq!(report.summary.reviewed, REVIEWED_COUNT);
    assert_eq!(report.summary.unassessed, UNASSESSED_COUNT);
    assert_eq!(report.summary.by_classification.get(EXACT_CLASSIFICATION), Some(&EXACT_COUNT));
    assert_eq!(report.summary.by_classification.get(UNASSESSED_CLASSIFICATION), Some(&UNASSESSED_COUNT));

    for identifier in SELECTED_IDS {
        assert!(current_missing.contains(identifier));
    }
    for row in report.rows.iter().filter(|row| SELECTED_IDS.contains(&row.requirement_id.as_str())) {
        for evidence in &row.evidence {
            assert_eq!(hashes.get(evidence.path.as_str()), Some(&evidence.blake3));
        }
    }
}

// r[verify lattice.repository_validation.webhook_auth_completion.scenario.fail_closed]
#[test]
fn webhook_auth_classification_marker_and_coverage_drift_fail_closed() {
    let report: InventoryReport = serde_json::from_str(REVIEWED_INVENTORY).expect("inventory must decode");
    let mut rows = selected_review_rows(&report);
    rows.retain(|row| row.requirement_id != REDACTION_ID);
    rows.iter_mut().find(|row| row.requirement_id == VALID_ID).expect("valid row").classification =
        "partial_evidence_only".to_string();
    rows.push(ReviewRow {
        requirement_id: UNEXPECTED_REVIEW_ID.to_string(),
        subsystem: SUBSYSTEM.to_string(),
        classification: EXACT_CLASSIFICATION.to_string(),
        rationale_code: "invalid.unexpected".to_string(),
        evidence_paths: Vec::new(),
    });
    let review_issues = validate_tranche(&rows, SUBSYSTEM, &expected_decisions());
    let review_codes = issue_codes(&review_issues);

    let promoted_source = format!("// r[verify {REDACTION_ID}]");
    let marker_issues = validate_no_selected_markers(&[promoted_source.as_str()], &[REDACTION_ID, REDACTION_ID]);
    let marker_codes = issue_codes(&marker_issues);

    let baseline = coverage_snapshot(BASELINE_COVERAGE);
    let mut current = coverage_snapshot(CURRENT_COVERAGE);
    current.missing.retain(|identifier| identifier != UNRELATED_MISSING_ID);
    current.missing.push(UNEXPECTED_ADDED_ID.to_string());
    current.missing.push(UNEXPECTED_ADDED_ID.to_string());
    current.dangling.push(DANGLING_ID.to_string());
    current.valid = true;
    let coverage_issues = validate_unchanged_coverage(&baseline, &current);
    let coverage_codes = issue_codes(&coverage_issues);

    assert!(review_codes.contains("review.missing"));
    assert!(review_codes.contains("review.unexpected"));
    assert!(review_codes.contains("review.classification"));
    assert!(marker_codes.contains("marker.promoted"));
    assert!(marker_codes.contains("marker.duplicate_selected"));
    assert!(coverage_codes.contains("coverage.removed"));
    assert!(coverage_codes.contains("coverage.added"));
    assert!(coverage_codes.contains("coverage.current_duplicate"));
    assert!(coverage_codes.contains("coverage.current_unsorted"));
    assert!(coverage_codes.contains("coverage.current_dangling"));
    assert!(coverage_codes.contains("coverage.current_valid"));
}

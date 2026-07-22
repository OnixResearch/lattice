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

const BASELINE_COVERAGE: &str = include_str!("fixtures/traceability-webhook-cookbook-promotion-current.json");
const CURRENT_COVERAGE: &str = include_str!("fixtures/traceability-n8n-credential-completion-current.json");
const REVIEWED_INVENTORY: &str = include_str!("fixtures/traceability-n8n-credential-completion-reviewed.json");

const MAIN_SOURCE: &str = include_str!("../src/main.rs");
const N8N_SOURCE: &str = include_str!("../src/n8n_json.rs");
const EXECUTOR_SOURCE: &str = include_str!("../src/engine/executor.rs");
const BOUNDARY_TESTS: &str = include_str!("test_n8n_credential_boundaries.rs");
const N8N_TESTS: &str = include_str!("test_n8n_json.rs");
const COMMAND_FIXTURE: &str = include_str!("fixtures/n8n/parity/supported/command-pipeline.json");
const COMMAND_REPORT: &str = include_str!("fixtures/n8n/parity/supported/command-pipeline.report.json");
const DEFAULT_HTTP_FIXTURE: &str = include_str!("fixtures/n8n/parity/supported/default-http-method.json");
const HTTP_FIXTURE: &str = include_str!("fixtures/n8n/parity/supported/http-credential-chain.json");
const HTTP_REPORT: &str = include_str!("fixtures/n8n/parity/supported/http-credential-chain.report.json");
const EXPRESSION_FIXTURE: &str = include_str!("fixtures/n8n/parity/unsupported/expression-url.json");
const MALFORMED_FIXTURE: &str = include_str!("fixtures/n8n/parity/unsupported/malformed-connection.json");
const PLAINTEXT_FIXTURE: &str = include_str!("fixtures/n8n/parity/unsupported/plaintext-node-credential.json");
const SCHEDULE_FIXTURE: &str = include_str!("fixtures/n8n/parity/unsupported/schedule-trigger.json");
const WEBHOOK_FIXTURE: &str = include_str!("fixtures/n8n/parity/unsupported/webhook-trigger.json");
const WEBHOOK_REPORT: &str = include_str!("fixtures/n8n/parity/unsupported/webhook-trigger.report.json");

const MAIN_SOURCE_PATH: &str = "src/main.rs";
const N8N_SOURCE_PATH: &str = "src/n8n_json.rs";
const EXECUTOR_SOURCE_PATH: &str = "src/engine/executor.rs";
const BOUNDARY_TESTS_PATH: &str = "tests/test_n8n_credential_boundaries.rs";
const N8N_TESTS_PATH: &str = "tests/test_n8n_json.rs";
const COMMAND_FIXTURE_PATH: &str = "tests/fixtures/n8n/parity/supported/command-pipeline.json";
const COMMAND_REPORT_PATH: &str = "tests/fixtures/n8n/parity/supported/command-pipeline.report.json";
const DEFAULT_HTTP_FIXTURE_PATH: &str = "tests/fixtures/n8n/parity/supported/default-http-method.json";
const HTTP_FIXTURE_PATH: &str = "tests/fixtures/n8n/parity/supported/http-credential-chain.json";
const HTTP_REPORT_PATH: &str = "tests/fixtures/n8n/parity/supported/http-credential-chain.report.json";
const EXPRESSION_FIXTURE_PATH: &str = "tests/fixtures/n8n/parity/unsupported/expression-url.json";
const MALFORMED_FIXTURE_PATH: &str = "tests/fixtures/n8n/parity/unsupported/malformed-connection.json";
const PLAINTEXT_FIXTURE_PATH: &str = "tests/fixtures/n8n/parity/unsupported/plaintext-node-credential.json";
const SCHEDULE_FIXTURE_PATH: &str = "tests/fixtures/n8n/parity/unsupported/schedule-trigger.json";
const WEBHOOK_FIXTURE_PATH: &str = "tests/fixtures/n8n/parity/unsupported/webhook-trigger.json";
const WEBHOOK_REPORT_PATH: &str = "tests/fixtures/n8n/parity/unsupported/webhook-trigger.report.json";

const SUBSYSTEM: &str = "credentials";
const EXACT_CLASSIFICATION: &str = "exact_maintained_evidence";
const PARTIAL_CLASSIFICATION: &str = "partial_evidence_only";
const UNASSESSED_CLASSIFICATION: &str = "unassessed";
const TOTAL_COUNT: usize = 340;
const REVIEWED_COUNT: usize = 13;
const EXACT_COUNT: usize = 4;
const PARTIAL_COUNT: usize = 9;
const UNASSESSED_COUNT: usize = 327;
const UNRELATED_MISSING_ID: &str = "lattice.agent_orchestration.authority.scenario.ambient-denied";
const UNEXPECTED_ADDED_ID: &str = "lattice.example.unaccepted.scenario.added";
const UNEXPECTED_REVIEW_ID: &str = concat!("n8n", "clone.credentials.unexpected.n8n-completion");
const DANGLING_ID: &str = "lattice.example.dangling.scenario.marker";

const EXPORT_ID: &str = concat!("n8n", "clone.credentials.n8n_json.placeholders.export");
const PLAINTEXT_ID: &str = concat!("n8n", "clone.credentials.n8n_json.placeholders.plaintext");
const REFERENCE_ID: &str = concat!("n8n", "clone.credentials.n8n_json.placeholders.reference");
const PARITY_ID: &str = concat!("n8n", "clone.credentials.n8n_parity_secret_safety");
const SELECTED_IDS: &[&str] = &[EXPORT_ID, PLAINTEXT_ID, REFERENCE_ID, PARITY_ID];
const EXPORT_MARKER: &str = concat!("// r[verify n8n", "clone.credentials.n8n_json.placeholders.export]\n");
const PLAINTEXT_MARKER: &str = concat!("// r[verify n8n", "clone.credentials.n8n_json.placeholders.plaintext]\n");
const REFERENCE_MARKER: &str = concat!("// r[verify n8n", "clone.credentials.n8n_json.placeholders.reference]\n");
const PARITY_MARKER: &str = concat!("// r[verify n8n", "clone.credentials.n8n_parity_secret_safety]\n");
const SINGLE_MARKER_COUNT: usize = 1;
const PARITY_MARKER_COUNT: usize = 2;

const EXPORT_EVIDENCE: &[&str] = &[MAIN_SOURCE_PATH, N8N_SOURCE_PATH, BOUNDARY_TESTS_PATH, N8N_TESTS_PATH];
const PLAINTEXT_EVIDENCE: &[&str] = &[
    MAIN_SOURCE_PATH,
    N8N_SOURCE_PATH,
    PLAINTEXT_FIXTURE_PATH,
    BOUNDARY_TESTS_PATH,
    N8N_TESTS_PATH,
];
const REFERENCE_EVIDENCE: &[&str] = &[
    EXECUTOR_SOURCE_PATH,
    N8N_SOURCE_PATH,
    BOUNDARY_TESTS_PATH,
    N8N_TESTS_PATH,
];
const PARITY_EVIDENCE: &[&str] = &[
    MAIN_SOURCE_PATH,
    N8N_SOURCE_PATH,
    COMMAND_FIXTURE_PATH,
    COMMAND_REPORT_PATH,
    DEFAULT_HTTP_FIXTURE_PATH,
    HTTP_FIXTURE_PATH,
    HTTP_REPORT_PATH,
    EXPRESSION_FIXTURE_PATH,
    MALFORMED_FIXTURE_PATH,
    PLAINTEXT_FIXTURE_PATH,
    SCHEDULE_FIXTURE_PATH,
    WEBHOOK_FIXTURE_PATH,
    WEBHOOK_REPORT_PATH,
    BOUNDARY_TESTS_PATH,
    N8N_TESTS_PATH,
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
            requirement_id: EXPORT_ID,
            classification: EXACT_CLASSIFICATION,
            rationale_code: "assertions.execution-save-raw-store-cli-export",
            evidence_paths: EXPORT_EVIDENCE,
        },
        ExpectedDecision {
            requirement_id: PLAINTEXT_ID,
            classification: EXACT_CLASSIFICATION,
            rationale_code: "assertions.cli-rejection-precedes-store",
            evidence_paths: PLAINTEXT_EVIDENCE,
        },
        ExpectedDecision {
            requirement_id: REFERENCE_ID,
            classification: EXACT_CLASSIFICATION,
            rationale_code: "assertions.imported-provider-execution-boundary",
            evidence_paths: REFERENCE_EVIDENCE,
        },
        ExpectedDecision {
            requirement_id: PARITY_ID,
            classification: EXACT_CLASSIFICATION,
            rationale_code: "assertions.bounded-negative-input-output-omission",
            evidence_paths: PARITY_EVIDENCE,
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
    assert_eq!(BOUNDARY_TESTS.matches(EXPORT_MARKER).count(), SINGLE_MARKER_COUNT);
    assert_eq!(BOUNDARY_TESTS.matches(PLAINTEXT_MARKER).count(), SINGLE_MARKER_COUNT);
    assert_eq!(BOUNDARY_TESTS.matches(REFERENCE_MARKER).count(), SINGLE_MARKER_COUNT);
    assert_eq!(BOUNDARY_TESTS.matches(PARITY_MARKER).count(), PARITY_MARKER_COUNT);
    BOUNDARY_TESTS
        .replace(EXPORT_MARKER, "")
        .replace(PLAINTEXT_MARKER, "")
        .replace(REFERENCE_MARKER, "")
        .replace(PARITY_MARKER, "")
}

fn actual_evidence_hashes() -> BTreeMap<&'static str, String> {
    let historical_boundary_tests = historical_boundary_tests();
    BTreeMap::from([
        (MAIN_SOURCE_PATH, blake3::hash(MAIN_SOURCE.as_bytes()).to_hex().to_string()),
        (N8N_SOURCE_PATH, blake3::hash(N8N_SOURCE.as_bytes()).to_hex().to_string()),
        (EXECUTOR_SOURCE_PATH, blake3::hash(EXECUTOR_SOURCE.as_bytes()).to_hex().to_string()),
        (BOUNDARY_TESTS_PATH, blake3::hash(historical_boundary_tests.as_bytes()).to_hex().to_string()),
        (N8N_TESTS_PATH, blake3::hash(N8N_TESTS.as_bytes()).to_hex().to_string()),
        (COMMAND_FIXTURE_PATH, blake3::hash(COMMAND_FIXTURE.as_bytes()).to_hex().to_string()),
        (COMMAND_REPORT_PATH, blake3::hash(COMMAND_REPORT.as_bytes()).to_hex().to_string()),
        (DEFAULT_HTTP_FIXTURE_PATH, blake3::hash(DEFAULT_HTTP_FIXTURE.as_bytes()).to_hex().to_string()),
        (HTTP_FIXTURE_PATH, blake3::hash(HTTP_FIXTURE.as_bytes()).to_hex().to_string()),
        (HTTP_REPORT_PATH, blake3::hash(HTTP_REPORT.as_bytes()).to_hex().to_string()),
        (EXPRESSION_FIXTURE_PATH, blake3::hash(EXPRESSION_FIXTURE.as_bytes()).to_hex().to_string()),
        (MALFORMED_FIXTURE_PATH, blake3::hash(MALFORMED_FIXTURE.as_bytes()).to_hex().to_string()),
        (PLAINTEXT_FIXTURE_PATH, blake3::hash(PLAINTEXT_FIXTURE.as_bytes()).to_hex().to_string()),
        (SCHEDULE_FIXTURE_PATH, blake3::hash(SCHEDULE_FIXTURE.as_bytes()).to_hex().to_string()),
        (WEBHOOK_FIXTURE_PATH, blake3::hash(WEBHOOK_FIXTURE.as_bytes()).to_hex().to_string()),
        (WEBHOOK_REPORT_PATH, blake3::hash(WEBHOOK_REPORT.as_bytes()).to_hex().to_string()),
    ])
}

// r[verify lattice.repository_validation.n8n_credential_completion]
// r[verify lattice.repository_validation.n8n_credential_completion.scenario.exact]
// r[verify lattice.repository_validation.n8n_credential_completion.scenario.boundaries]
// r[verify lattice.repository_validation.n8n_credential_completion.scenario.parity]
#[test]
fn exactly_four_n8n_rows_retain_their_historical_byte_bound_review() {
    let report: InventoryReport = serde_json::from_str(REVIEWED_INVENTORY).expect("inventory must decode");
    let rows = selected_review_rows(&report);
    let review_issues = validate_tranche(&rows, SUBSYSTEM, &expected_decisions());
    let baseline = coverage_snapshot(BASELINE_COVERAGE);
    let current = coverage_snapshot(CURRENT_COVERAGE);
    let coverage_issues = validate_unchanged_coverage(&baseline, &current);
    let historical_boundary_tests = historical_boundary_tests();
    let marker_issues = validate_no_selected_markers(&[historical_boundary_tests.as_str()], SELECTED_IDS);
    let actual_hashes = actual_evidence_hashes();
    let selected = SELECTED_IDS.iter().copied().collect::<BTreeSet<_>>();
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
    assert_eq!(report.summary.by_classification.get(PARTIAL_CLASSIFICATION), Some(&PARTIAL_COUNT));
    assert_eq!(report.summary.by_classification.get(UNASSESSED_CLASSIFICATION), Some(&UNASSESSED_COUNT));

    for identifier in SELECTED_IDS {
        assert!(current_missing.contains(identifier));
    }
    for row in &report.rows {
        if selected.contains(row.requirement_id.as_str()) {
            assert_eq!(row.classification, EXACT_CLASSIFICATION);
            for evidence in &row.evidence {
                assert_eq!(actual_hashes.get(evidence.path.as_str()), Some(&evidence.blake3));
            }
        } else if row.classification != UNASSESSED_CLASSIFICATION {
            assert_eq!(row.classification, PARTIAL_CLASSIFICATION);
        }
    }
}

// r[verify lattice.repository_validation.n8n_credential_completion.scenario.fail_closed]
#[test]
fn n8n_classification_marker_and_coverage_drift_fail_closed() {
    let report: InventoryReport = serde_json::from_str(REVIEWED_INVENTORY).expect("inventory must decode");
    let mut rows = selected_review_rows(&report);
    rows.retain(|row| row.requirement_id != EXPORT_ID);
    rows.iter_mut()
        .find(|row| row.requirement_id == PLAINTEXT_ID)
        .expect("plaintext row")
        .classification = PARTIAL_CLASSIFICATION.to_string();
    rows.iter_mut()
        .find(|row| row.requirement_id == REFERENCE_ID)
        .expect("reference row")
        .rationale_code = "invalid.rationale".to_string();
    rows.iter_mut().find(|row| row.requirement_id == PARITY_ID).expect("parity row").evidence_paths = Vec::new();
    rows.push(ReviewRow {
        requirement_id: UNEXPECTED_REVIEW_ID.to_string(),
        subsystem: SUBSYSTEM.to_string(),
        classification: EXACT_CLASSIFICATION.to_string(),
        rationale_code: "invalid.unexpected".to_string(),
        evidence_paths: vec![N8N_SOURCE_PATH.to_string()],
    });
    let review_issues = validate_tranche(&rows, SUBSYSTEM, &expected_decisions());
    let review_codes = issue_codes(&review_issues);

    let promoted_source = format!("// r[verify {EXPORT_ID}]");
    let marker_issues = validate_no_selected_markers(&[promoted_source.as_str()], &[EXPORT_ID, EXPORT_ID]);
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
    assert!(review_codes.contains("review.rationale"));
    assert!(review_codes.contains("review.evidence_paths"));
    assert!(marker_codes.contains("marker.promoted"));
    assert!(marker_codes.contains("marker.duplicate_selected"));
    assert!(coverage_codes.contains("coverage.removed"));
    assert!(coverage_codes.contains("coverage.added"));
    assert!(coverage_codes.contains("coverage.current_duplicate"));
    assert!(coverage_codes.contains("coverage.current_unsorted"));
    assert!(coverage_codes.contains("coverage.current_dangling"));
    assert!(coverage_codes.contains("coverage.current_valid"));
}

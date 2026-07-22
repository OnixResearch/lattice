#[path = "traceability_review/core.rs"]
mod core;

use core::CoverageSnapshot;
use core::ExpectedDecision;
use core::ReviewRow;
use core::validate_tranche;
use core::validate_unchanged_coverage;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

const BASELINE_COVERAGE: &str = include_str!("fixtures/traceability-candidate-promotion-current.json");
const CURRENT_COVERAGE: &str = include_str!("fixtures/traceability-operator-diagnostics-current.json");
const REVIEWED_INVENTORY: &str = include_str!("fixtures/traceability-operator-diagnostics-reviewed.json");
const DOCTOR_CORE_SOURCE: &str = include_str!("../src/doctor/core.rs");
const DOCTOR_TESTS: &str = include_str!("test_doctor.rs");
const DOCTOR_CORE_SOURCE_PATH: &str = "src/doctor/core.rs";
const DOCTOR_TESTS_PATH: &str = "tests/test_doctor.rs";
const SUBSYSTEM: &str = "operator-diagnostics";
const UNASSESSED_CLASSIFICATION: &str = "unassessed";
const EXACT_CLASSIFICATION: &str = "exact_maintained_evidence";
const PARTIAL_CLASSIFICATION: &str = "partial_evidence_only";
const NO_EXACT_CLASSIFICATION: &str = "no_exact_evidence_identified";
const TOTAL_COUNT: usize = 361;
const REVIEWED_COUNT: usize = 8;
const UNASSESSED_COUNT: usize = 353;
const EXACT_COUNT: usize = 8;
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
const UNEXPECTED_REVIEW_ID: &str = concat!("n8n", "clone.operator_diagnostics.unexpected");

#[derive(Debug, Clone, serde::Deserialize)]
struct InventoryReport {
    summary: InventorySummary,
    rows: Vec<InventoryRow>,
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
            requirement_id: DOCTOR_CORE_ID,
            classification: EXACT_CLASSIFICATION,
            rationale_code: "assertions.pure-core-deterministic",
            evidence_paths: &[DOCTOR_CORE_SOURCE_PATH],
        },
        ExpectedDecision {
            requirement_id: DOCTOR_CORE_FATAL_ID,
            classification: EXACT_CLASSIFICATION,
            rationale_code: "assertions.enumerated-failures-complete",
            evidence_paths: &[DOCTOR_CORE_SOURCE_PATH],
        },
        ExpectedDecision {
            requirement_id: DOCTOR_CORE_HEALTHY_ID,
            classification: EXACT_CLASSIFICATION,
            rationale_code: "assertions.healthy-findings-complete",
            evidence_paths: &[DOCTOR_CORE_SOURCE_PATH],
        },
        ExpectedDecision {
            requirement_id: DOCTOR_OUTPUT_ID,
            classification: EXACT_CLASSIFICATION,
            rationale_code: "assertions.deterministic-dual-renderers",
            evidence_paths: &[DOCTOR_CORE_SOURCE_PATH],
        },
        ExpectedDecision {
            requirement_id: DOCTOR_OUTPUT_JSON_ID,
            classification: EXACT_CLASSIFICATION,
            rationale_code: "assertions.mixed-json-output-complete",
            evidence_paths: &[DOCTOR_CORE_SOURCE_PATH],
        },
        ExpectedDecision {
            requirement_id: DOCTOR_OUTPUT_TEXT_ID,
            classification: EXACT_CLASSIFICATION,
            rationale_code: "assertions.mixed-text-output-complete",
            evidence_paths: &[DOCTOR_CORE_SOURCE_PATH],
        },
        ExpectedDecision {
            requirement_id: DOCTOR_REDACTION_ID,
            classification: EXACT_CLASSIFICATION,
            rationale_code: "assertions.sensitive-values-redacted",
            evidence_paths: &[DOCTOR_CORE_SOURCE_PATH, DOCTOR_TESTS_PATH],
        },
        ExpectedDecision {
            requirement_id: DOCTOR_REDACTION_JSON_ID,
            classification: EXACT_CLASSIFICATION,
            rationale_code: "assertions.json-sensitive-values-redacted",
            evidence_paths: &[DOCTOR_CORE_SOURCE_PATH],
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

fn actual_evidence_hashes() -> BTreeMap<&'static str, String> {
    BTreeMap::from([
        (DOCTOR_CORE_SOURCE_PATH, blake3::hash(DOCTOR_CORE_SOURCE.as_bytes()).to_hex().to_string()),
        (DOCTOR_TESTS_PATH, blake3::hash(DOCTOR_TESTS.as_bytes()).to_hex().to_string()),
    ])
}

// r[verify lattice.repository_validation.traceability_tranche_review.scenario.complete]
// r[verify lattice.repository_validation.traceability_tranche_review.scenario.evidence]
#[test]
fn operator_diagnostics_review_is_complete_bound_and_exact() {
    let report: InventoryReport = serde_json::from_str(REVIEWED_INVENTORY).expect("inventory must decode");
    let rows = review_rows(&report);
    let review_issues = validate_tranche(&rows, SUBSYSTEM, &expected_decisions());
    let baseline = coverage_snapshot(BASELINE_COVERAGE);
    let current = coverage_snapshot(CURRENT_COVERAGE);
    let coverage_issues = validate_unchanged_coverage(&baseline, &current);
    let actual_hashes = actual_evidence_hashes();

    assert!(review_issues.is_empty(), "review issues: {review_issues:?}");
    assert!(coverage_issues.is_empty(), "coverage issues: {coverage_issues:?}");
    assert_eq!(report.summary.total, TOTAL_COUNT);
    assert_eq!(report.summary.reviewed, REVIEWED_COUNT);
    assert_eq!(report.summary.unassessed, UNASSESSED_COUNT);
    assert_eq!(report.summary.by_classification.get(EXACT_CLASSIFICATION), Some(&EXACT_COUNT));
    assert!(!report.summary.by_classification.contains_key(PARTIAL_CLASSIFICATION));
    assert!(!report.summary.by_classification.contains_key(NO_EXACT_CLASSIFICATION));
    assert_eq!(report.summary.by_classification.get(UNASSESSED_CLASSIFICATION), Some(&UNASSESSED_COUNT));

    for row in report.rows.iter().filter(|row| row.subsystem == SUBSYSTEM) {
        for evidence in &row.evidence {
            assert_eq!(actual_hashes.get(evidence.path.as_str()), Some(&evidence.blake3));
        }
        assert!(DOCTOR_CORE_SOURCE.contains(&format!("r[verify {}]", row.requirement_id)));
    }
}

// r[verify lattice.repository_validation.traceability_tranche_review.scenario.fail_closed]
#[test]
fn classification_evidence_and_coverage_drift_fail_closed() {
    let report: InventoryReport = serde_json::from_str(REVIEWED_INVENTORY).expect("inventory must decode");
    let mut rows = review_rows(&report);
    rows.retain(|row| row.requirement_id != DOCTOR_CORE_ID);
    rows.iter_mut()
        .find(|row| row.requirement_id == DOCTOR_CORE_FATAL_ID)
        .expect("fatal row")
        .classification = PARTIAL_CLASSIFICATION.to_string();
    rows.iter_mut()
        .find(|row| row.requirement_id == DOCTOR_CORE_HEALTHY_ID)
        .expect("healthy row")
        .rationale_code = "invalid.rationale".to_string();
    rows.iter_mut()
        .find(|row| row.requirement_id == DOCTOR_OUTPUT_TEXT_ID)
        .expect("text row")
        .evidence_paths = Vec::new();
    rows.push(ReviewRow {
        requirement_id: UNEXPECTED_REVIEW_ID.to_string(),
        subsystem: SUBSYSTEM.to_string(),
        classification: NO_EXACT_CLASSIFICATION.to_string(),
        rationale_code: "invalid.unexpected".to_string(),
        evidence_paths: Vec::new(),
    });
    let review_issues = validate_tranche(&rows, SUBSYSTEM, &expected_decisions());
    let review_codes = issue_codes(&review_issues);

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
    assert!(coverage_codes.contains("coverage.removed"));
    assert!(coverage_codes.contains("coverage.added"));
    assert!(coverage_codes.contains("coverage.current_duplicate"));
    assert!(coverage_codes.contains("coverage.current_unsorted"));
    assert!(coverage_codes.contains("coverage.current_dangling"));
    assert!(coverage_codes.contains("coverage.current_valid"));
}

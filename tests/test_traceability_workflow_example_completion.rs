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

const BASELINE: &str = include_str!("fixtures/traceability-scheduling-promotion-current.json");
const CURRENT: &str = include_str!("fixtures/traceability-workflow-example-completion-current.json");
const REVIEWED: &str = include_str!("fixtures/traceability-workflow-example-completion-reviewed.json");
const POSITIVE_FIXTURE: &str = include_str!("../examples/workflows/selfie-to-fine-art.ncl");
const NEGATIVE_FIXTURE: &str = include_str!("../examples/workflows/invalid-unsupported-image-format.ncl");
const TEST_SOURCE: &str = include_str!("test_image_workflow_example.rs");
const POSITIVE_PATH: &str = "examples/workflows/selfie-to-fine-art.ncl";
const NEGATIVE_PATH: &str = "examples/workflows/invalid-unsupported-image-format.ncl";
const TEST_PATH: &str = "tests/test_image_workflow_example.rs";
const SUBSYSTEM: &str = "workflow-examples";
const EXACT: &str = "exact_maintained_evidence";
const PARTIAL: &str = "partial_evidence_only";
const INVALID_ID: &str = concat!("n8n", "clone.workflow_examples.image.invalid");
const SELFIE_ID: &str = concat!("n8n", "clone.workflow_examples.image.selfie_to_fine_art");
const IDS: &[&str] = &[INVALID_ID, SELFIE_ID];
const INVALID_MARKER: &str = concat!("// r[verify n8n", "clone.workflow_examples.image.invalid]\n");
const SELFIE_MARKER: &str = concat!("// r[verify n8n", "clone.workflow_examples.image.selfie_to_fine_art]\n");
const MARKER_COUNT: usize = 1;
const UNRELATED: &str = "lattice.agent_orchestration.authority.scenario.ambient-denied";
const ADDED: &str = "lattice.example.unaccepted.scenario.added";
const DANGLING: &str = "lattice.example.dangling.scenario.marker";

#[derive(Debug, serde::Deserialize)]
struct Report {
    rows: Vec<Row>,
}
#[derive(Debug, serde::Deserialize)]
struct Row {
    requirement_id: String,
    subsystem: String,
    classification: String,
    evidence: Vec<Evidence>,
    rationale_code: String,
}
#[derive(Debug, serde::Deserialize)]
struct Evidence {
    path: String,
    blake3: String,
}
#[derive(Debug, serde::Deserialize)]
struct Receipt {
    missing: Vec<String>,
    dangling: Vec<String>,
    valid: bool,
}

fn expected() -> Vec<ExpectedDecision> {
    vec![
        ExpectedDecision {
            requirement_id: INVALID_ID,
            classification: EXACT,
            rationale_code: "assertions.unsupported-format-without-partial-completion",
            evidence_paths: &[NEGATIVE_PATH, TEST_PATH],
        },
        ExpectedDecision {
            requirement_id: SELFIE_ID,
            classification: EXACT,
            rationale_code: "assertions.import-execute-and-image-output",
            evidence_paths: &[POSITIVE_PATH, TEST_PATH],
        },
    ]
}
fn rows(report: &Report) -> Vec<ReviewRow> {
    report
        .rows
        .iter()
        .map(|row| ReviewRow {
            requirement_id: row.requirement_id.clone(),
            subsystem: row.subsystem.clone(),
            classification: row.classification.clone(),
            rationale_code: row.rationale_code.clone(),
            evidence_paths: row.evidence.iter().map(|e| e.path.clone()).collect(),
        })
        .collect()
}
fn snapshot(source: &str) -> CoverageSnapshot {
    let receipt: Receipt = serde_json::from_str(source).expect("coverage decodes");
    CoverageSnapshot {
        missing: receipt.missing,
        dangling: receipt.dangling,
        valid: receipt.valid,
    }
}
fn codes(issues: &[core::ReviewIssue]) -> BTreeSet<&str> {
    issues.iter().map(|issue| issue.code).collect()
}
fn historical_test_source() -> String {
    assert_eq!(TEST_SOURCE.matches(INVALID_MARKER).count(), MARKER_COUNT);
    assert_eq!(TEST_SOURCE.matches(SELFIE_MARKER).count(), MARKER_COUNT);
    TEST_SOURCE.replace(INVALID_MARKER, "").replace(SELFIE_MARKER, "")
}

fn hashes() -> BTreeMap<&'static str, String> {
    let historical_test = historical_test_source();
    BTreeMap::from([
        (POSITIVE_PATH, blake3::hash(POSITIVE_FIXTURE.as_bytes()).to_hex().to_string()),
        (NEGATIVE_PATH, blake3::hash(NEGATIVE_FIXTURE.as_bytes()).to_hex().to_string()),
        (TEST_PATH, blake3::hash(historical_test.as_bytes()).to_hex().to_string()),
    ])
}

// r[verify lattice.repository_validation.workflow_example_completion]
// r[verify lattice.repository_validation.workflow_example_completion.scenario.fixture]
// r[verify lattice.repository_validation.workflow_example_completion.scenario.negative]
#[test]
fn both_workflow_example_rows_are_exact_byte_bound_and_unpromoted() {
    let report: Report = serde_json::from_str(REVIEWED).expect("inventory decodes");
    let review_issues = validate_tranche(&rows(&report), SUBSYSTEM, &expected());
    let coverage_issues = validate_unchanged_coverage(&snapshot(BASELINE), &snapshot(CURRENT));
    let historical_test = historical_test_source();
    let marker_issues =
        validate_no_selected_markers(&[POSITIVE_FIXTURE, NEGATIVE_FIXTURE, historical_test.as_str()], IDS);
    let actual = hashes();
    assert!(review_issues.is_empty(), "review issues: {review_issues:?}");
    assert!(coverage_issues.is_empty(), "coverage issues: {coverage_issues:?}");
    assert!(marker_issues.is_empty(), "marker issues: {marker_issues:?}");
    for row in report.rows.iter().filter(|row| IDS.contains(&row.requirement_id.as_str())) {
        for evidence in &row.evidence {
            assert_eq!(actual.get(evidence.path.as_str()), Some(&evidence.blake3));
        }
    }
}

#[test]
fn workflow_example_completion_drift_fails_closed() {
    let report: Report = serde_json::from_str(REVIEWED).expect("inventory decodes");
    let mut review_rows = rows(&report);
    review_rows.retain(|row| row.requirement_id != INVALID_ID);
    let selfie = review_rows.iter_mut().find(|row| row.requirement_id == SELFIE_ID).expect("selfie row");
    selfie.classification = PARTIAL.to_string();
    selfie.rationale_code = "invalid.rationale".to_string();
    selfie.evidence_paths = vec!["../unsafe".to_string()];
    let review_issues = validate_tranche(&review_rows, SUBSYSTEM, &expected());
    let review_codes = codes(&review_issues);
    let promoted = format!("// r[verify {INVALID_ID}]");
    let marker_issues = validate_no_selected_markers(&[promoted.as_str()], &[INVALID_ID, INVALID_ID]);
    let marker_codes = codes(&marker_issues);
    let baseline = snapshot(BASELINE);
    let mut current = snapshot(CURRENT);
    current.missing.retain(|id| id != UNRELATED);
    current.missing.push(ADDED.to_string());
    current.missing.push(ADDED.to_string());
    current.dangling.push(DANGLING.to_string());
    current.valid = true;
    let coverage_issues = validate_unchanged_coverage(&baseline, &current);
    let coverage_codes = codes(&coverage_issues);
    for code in [
        "review.missing",
        "review.classification",
        "review.rationale",
        "review.evidence_paths",
    ] {
        assert!(review_codes.contains(code));
    }
    assert!(marker_codes.contains("marker.promoted"));
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

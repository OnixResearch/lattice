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

const BASELINE_COVERAGE: &str = include_str!("fixtures/traceability-workflow-example-audit-current.json");
const CURRENT_COVERAGE: &str = include_str!("fixtures/traceability-node-template-prompt-audit-current.json");
const GENERATED_INVENTORY: &str = include_str!("../cairn-policy/generated/traceability-debt-inventory.json");
const APP_SOURCE: &str = include_str!("../src/tui/app.rs");
const PANELS_SOURCE: &str = include_str!("../src/tui/panels.rs");
const APP_SOURCE_PATH: &str = "src/tui/app.rs";
const PANELS_SOURCE_PATH: &str = "src/tui/panels.rs";
const SUBSYSTEM: &str = "node-templates";
const EXACT_CLASSIFICATION: &str = "exact_maintained_evidence";
const PARTIAL_CLASSIFICATION: &str = "partial_evidence_only";
const NO_EXACT_CLASSIFICATION: &str = "no_exact_evidence_identified";
const SELECTED_COUNT: usize = 3;
const PROMPT_ID: &str = concat!("n8n", "clone.node_templates.parameters.prompt");
const CANCEL_ID: &str = concat!("n8n", "clone.node_templates.parameters.prompt.cancel");
const REQUIRED_ID: &str = concat!("n8n", "clone.node_templates.parameters.prompt.required");
const SELECTED_IDS: &[&str] = &[PROMPT_ID, CANCEL_ID, REQUIRED_ID];
const UNRELATED_MISSING_ID: &str = "lattice.agent_orchestration.authority.scenario.ambient-denied";
const UNEXPECTED_ADDED_ID: &str = "lattice.example.unaccepted.scenario.added";
const DANGLING_ID: &str = "lattice.example.dangling.scenario.marker";
const UNEXPECTED_REVIEW_ID: &str = concat!("n8n", "clone.node_templates.parameters.prompt.unexpected");

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
            requirement_id: PROMPT_ID,
            classification: EXACT_CLASSIFICATION,
            rationale_code: "assertions.parameter-value-collected",
            evidence_paths: &[APP_SOURCE_PATH],
        },
        ExpectedDecision {
            requirement_id: CANCEL_ID,
            classification: EXACT_CLASSIFICATION,
            rationale_code: "assertions.cancel-preserves-workflow",
            evidence_paths: &[APP_SOURCE_PATH],
        },
        ExpectedDecision {
            requirement_id: REQUIRED_ID,
            classification: PARTIAL_CLASSIFICATION,
            rationale_code: "gap.rendered-required-prompt-unasserted",
            evidence_paths: &[APP_SOURCE_PATH, PANELS_SOURCE_PATH],
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
        (APP_SOURCE_PATH, blake3::hash(APP_SOURCE.as_bytes()).to_hex().to_string()),
        (PANELS_SOURCE_PATH, blake3::hash(PANELS_SOURCE.as_bytes()).to_hex().to_string()),
    ])
}

// r[verify lattice.repository_validation.node_template_prompt_audit]
// r[verify lattice.repository_validation.node_template_prompt_audit.scenario.complete]
// r[verify lattice.repository_validation.node_template_prompt_audit.scenario.evidence]
#[test]
fn node_template_prompt_audit_is_complete_byte_bound_and_unpromoted() {
    let report: InventoryReport = serde_json::from_str(GENERATED_INVENTORY).expect("inventory must decode");
    let review_issues = validate_tranche(&review_rows(&report), SUBSYSTEM, &expected_decisions());
    let baseline = coverage_snapshot(BASELINE_COVERAGE);
    let current = coverage_snapshot(CURRENT_COVERAGE);
    let coverage_issues = validate_unchanged_coverage(&baseline, &current);
    let marker_issues = validate_no_selected_markers(&[APP_SOURCE, PANELS_SOURCE], SELECTED_IDS);
    let actual_hashes = actual_evidence_hashes();
    let selected_rows = report.rows.iter().filter(|row| SELECTED_IDS.contains(&row.requirement_id.as_str()));

    assert!(review_issues.is_empty(), "review issues: {review_issues:?}");
    assert!(coverage_issues.is_empty(), "coverage issues: {coverage_issues:?}");
    assert!(marker_issues.is_empty(), "marker issues: {marker_issues:?}");
    assert_eq!(selected_rows.count(), SELECTED_COUNT);

    for row in report.rows.iter().filter(|row| SELECTED_IDS.contains(&row.requirement_id.as_str())) {
        for evidence in &row.evidence {
            assert_eq!(actual_hashes.get(evidence.path.as_str()), Some(&evidence.blake3));
        }
    }
}

// r[verify lattice.repository_validation.node_template_prompt_audit.scenario.fail_closed]
#[test]
fn node_template_prompt_classification_marker_evidence_and_coverage_drift_fail_closed() {
    let report: InventoryReport = serde_json::from_str(GENERATED_INVENTORY).expect("inventory must decode");
    let mut rows = review_rows(&report);
    rows.retain(|row| row.requirement_id != CANCEL_ID);
    let prompt = rows.iter_mut().find(|row| row.requirement_id == PROMPT_ID).expect("prompt row");
    prompt.classification = PARTIAL_CLASSIFICATION.to_string();
    prompt.rationale_code = "invalid.rationale".to_string();
    prompt.evidence_paths = vec!["../unsafe".to_string()];
    rows.push(ReviewRow {
        requirement_id: UNEXPECTED_REVIEW_ID.to_string(),
        subsystem: SUBSYSTEM.to_string(),
        classification: NO_EXACT_CLASSIFICATION.to_string(),
        rationale_code: "invalid.unexpected".to_string(),
        evidence_paths: Vec::new(),
    });
    let review_issues = validate_tranche(&rows, SUBSYSTEM, &expected_decisions());
    let review_codes = issue_codes(&review_issues);

    let promoted_source = format!("// r[verify {PROMPT_ID}]");
    let marker_issues = validate_no_selected_markers(&[promoted_source.as_str()], &[PROMPT_ID, PROMPT_ID]);
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

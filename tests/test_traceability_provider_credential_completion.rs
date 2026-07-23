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

const BASELINE_COVERAGE: &str = include_str!("fixtures/traceability-n8n-credential-promotion-current.json");
const CURRENT_COVERAGE: &str = include_str!("fixtures/traceability-provider-credential-completion-current.json");
const REVIEWED_INVENTORY: &str = include_str!("fixtures/traceability-provider-credential-completion-reviewed.json");
const CREDENTIAL_SOURCE: &str = include_str!("../src/credentials.rs");
const EXECUTOR_SOURCE: &str = include_str!("fixtures/traceability-reviewed-executor-source.txt");
const BOUNDARY_TESTS: &str = include_str!("test_provider_credential_boundaries.rs");

const CREDENTIAL_SOURCE_PATH: &str = "src/credentials.rs";
const EXECUTOR_SOURCE_PATH: &str = "src/engine/executor.rs";
const BOUNDARY_TESTS_PATH: &str = "tests/test_provider_credential_boundaries.rs";
const SUBSYSTEM: &str = "credentials";
const EXACT_CLASSIFICATION: &str = "exact_maintained_evidence";
const PARTIAL_CLASSIFICATION: &str = "partial_evidence_only";
const UNASSESSED_CLASSIFICATION: &str = "unassessed";
const TOTAL_COUNT: usize = 336;
const REVIEWED_COUNT: usize = 9;
const EXACT_COUNT: usize = 7;
const PARTIAL_COUNT: usize = 2;
const UNASSESSED_COUNT: usize = 327;
const UNRELATED_MISSING_ID: &str = "lattice.agent_orchestration.authority.scenario.ambient-denied";
const UNEXPECTED_ADDED_ID: &str = "lattice.example.unaccepted.scenario.added";
const DANGLING_ID: &str = "lattice.example.dangling.scenario.marker";
const UNEXPECTED_REVIEW_ID: &str = concat!("n8n", "clone.credentials.unexpected.provider-completion");

const CACHE_ID: &str = concat!("n8n", "clone.credentials.provider_cache_identity.no_plaintext");
const ERROR_ID: &str = concat!("n8n", "clone.credentials.provider_diagnostics.redaction.error");
const PROBE_ID: &str = concat!("n8n", "clone.credentials.provider_diagnostics.redaction.test");
const COMMAND_ID: &str = concat!("n8n", "clone.credentials.providers.command.resolve");
const TIMEOUT_ID: &str = concat!("n8n", "clone.credentials.providers.command.timeout");
const VAULT_ID: &str = concat!("n8n", "clone.credentials.providers.local_vault.resolve");
const TAMPER_ID: &str = concat!("n8n", "clone.credentials.providers.local_vault.tamper");
const SELECTED_IDS: &[&str] = &[
    CACHE_ID, ERROR_ID, PROBE_ID, COMMAND_ID, TIMEOUT_ID, VAULT_ID, TAMPER_ID,
];
const CACHE_MARKER: &str = concat!("// r[verify n8n", "clone.credentials.provider_cache_identity.no_plaintext]\n");
const ERROR_MARKER: &str = concat!("// r[verify n8n", "clone.credentials.provider_diagnostics.redaction.error]\n");
const PROBE_MARKER: &str = concat!("// r[verify n8n", "clone.credentials.provider_diagnostics.redaction.test]\n");
const COMMAND_MARKER: &str = concat!("// r[verify n8n", "clone.credentials.providers.command.resolve]\n");
const TIMEOUT_MARKER: &str = concat!("// r[verify n8n", "clone.credentials.providers.command.timeout]\n");
const VAULT_MARKER: &str = concat!("// r[verify n8n", "clone.credentials.providers.local_vault.resolve]\n");
const TAMPER_MARKER: &str = concat!("// r[verify n8n", "clone.credentials.providers.local_vault.tamper]\n");
const MARKER_COUNT: usize = 1;
const CACHE_EVIDENCE: &[&str] = &[EXECUTOR_SOURCE_PATH, BOUNDARY_TESTS_PATH];
const DIAGNOSTIC_EVIDENCE: &[&str] = &[CREDENTIAL_SOURCE_PATH, BOUNDARY_TESTS_PATH];
const EXECUTOR_EVIDENCE: &[&str] = &[CREDENTIAL_SOURCE_PATH, EXECUTOR_SOURCE_PATH, BOUNDARY_TESTS_PATH];

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
            requirement_id: CACHE_ID,
            classification: EXACT_CLASSIFICATION,
            rationale_code: "assertions.provider-key-field-and-value-independence",
            evidence_paths: CACHE_EVIDENCE,
        },
        ExpectedDecision {
            requirement_id: ERROR_ID,
            classification: EXACT_CLASSIFICATION,
            rationale_code: "assertions.fixed-category-safe-identity-render",
            evidence_paths: DIAGNOSTIC_EVIDENCE,
        },
        ExpectedDecision {
            requirement_id: PROBE_ID,
            classification: EXACT_CLASSIFICATION,
            rationale_code: "assertions.serialized-success-and-failure",
            evidence_paths: DIAGNOSTIC_EVIDENCE,
        },
        ExpectedDecision {
            requirement_id: COMMAND_ID,
            classification: EXACT_CLASSIFICATION,
            rationale_code: "assertions.executor-before-side-effect-and-output-omission",
            evidence_paths: EXECUTOR_EVIDENCE,
        },
        ExpectedDecision {
            requirement_id: TIMEOUT_ID,
            classification: EXACT_CLASSIFICATION,
            rationale_code: "assertions.executor-timeout-blocks-side-effect",
            evidence_paths: EXECUTOR_EVIDENCE,
        },
        ExpectedDecision {
            requirement_id: VAULT_ID,
            classification: EXACT_CLASSIFICATION,
            rationale_code: "assertions.workflow-file-executor-boundary",
            evidence_paths: EXECUTOR_EVIDENCE,
        },
        ExpectedDecision {
            requirement_id: TAMPER_ID,
            classification: EXACT_CLASSIFICATION,
            rationale_code: "assertions.rendered-tamper-blocks-side-effect",
            evidence_paths: EXECUTOR_EVIDENCE,
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
    for marker in [
        CACHE_MARKER,
        ERROR_MARKER,
        PROBE_MARKER,
        COMMAND_MARKER,
        TIMEOUT_MARKER,
        VAULT_MARKER,
        TAMPER_MARKER,
    ] {
        assert_eq!(BOUNDARY_TESTS.matches(marker).count(), MARKER_COUNT);
    }
    BOUNDARY_TESTS
        .replace(CACHE_MARKER, "")
        .replace(ERROR_MARKER, "")
        .replace(PROBE_MARKER, "")
        .replace(COMMAND_MARKER, "")
        .replace(TIMEOUT_MARKER, "")
        .replace(VAULT_MARKER, "")
        .replace(TAMPER_MARKER, "")
}

fn actual_evidence_hashes() -> BTreeMap<&'static str, String> {
    let historical_boundary_tests = historical_boundary_tests();
    BTreeMap::from([
        (CREDENTIAL_SOURCE_PATH, blake3::hash(CREDENTIAL_SOURCE.as_bytes()).to_hex().to_string()),
        (EXECUTOR_SOURCE_PATH, blake3::hash(EXECUTOR_SOURCE.as_bytes()).to_hex().to_string()),
        (BOUNDARY_TESTS_PATH, blake3::hash(historical_boundary_tests.as_bytes()).to_hex().to_string()),
    ])
}

// r[verify lattice.repository_validation.provider_credential_completion]
// r[verify lattice.repository_validation.provider_credential_completion.scenario.diagnostics]
// r[verify lattice.repository_validation.provider_credential_completion.scenario.boundaries]
// r[verify lattice.repository_validation.provider_credential_completion.scenario.exact]
#[test]
fn seven_provider_rows_retain_their_historical_exact_review() {
    let report: InventoryReport = serde_json::from_str(REVIEWED_INVENTORY).expect("inventory must decode");
    let rows = selected_review_rows(&report);
    let review_issues = validate_tranche(&rows, SUBSYSTEM, &expected_decisions());
    let baseline = coverage_snapshot(BASELINE_COVERAGE);
    let current = coverage_snapshot(CURRENT_COVERAGE);
    let coverage_issues = validate_unchanged_coverage(&baseline, &current);
    let historical_boundary_tests = historical_boundary_tests();
    let marker_issues = validate_no_selected_markers(
        &[CREDENTIAL_SOURCE, EXECUTOR_SOURCE, historical_boundary_tests.as_str()],
        SELECTED_IDS,
    );
    let hashes = actual_evidence_hashes();
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
                assert_eq!(hashes.get(evidence.path.as_str()), Some(&evidence.blake3));
            }
        } else if row.classification != UNASSESSED_CLASSIFICATION {
            assert_eq!(row.classification, PARTIAL_CLASSIFICATION);
        }
    }
}

// r[verify lattice.repository_validation.provider_credential_completion.scenario.fail_closed]
#[test]
fn provider_classification_marker_and_coverage_drift_fail_closed() {
    let report: InventoryReport = serde_json::from_str(REVIEWED_INVENTORY).expect("inventory must decode");
    let mut rows = selected_review_rows(&report);
    rows.retain(|row| row.requirement_id != CACHE_ID);
    rows.iter_mut().find(|row| row.requirement_id == ERROR_ID).expect("error row").classification =
        PARTIAL_CLASSIFICATION.to_string();
    rows.iter_mut().find(|row| row.requirement_id == PROBE_ID).expect("probe row").rationale_code =
        "invalid.rationale".to_string();
    rows.iter_mut().find(|row| row.requirement_id == COMMAND_ID).expect("command row").evidence_paths = Vec::new();
    rows.push(ReviewRow {
        requirement_id: UNEXPECTED_REVIEW_ID.to_string(),
        subsystem: SUBSYSTEM.to_string(),
        classification: EXACT_CLASSIFICATION.to_string(),
        rationale_code: "invalid.unexpected".to_string(),
        evidence_paths: vec![CREDENTIAL_SOURCE_PATH.to_string()],
    });
    let review_issues = validate_tranche(&rows, SUBSYSTEM, &expected_decisions());
    let review_codes = issue_codes(&review_issues);

    let promoted_source = format!("// r[verify {CACHE_ID}]");
    let marker_issues = validate_no_selected_markers(&[promoted_source.as_str()], &[CACHE_ID, CACHE_ID]);
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

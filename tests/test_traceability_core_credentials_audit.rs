#[path = "traceability_review/core.rs"]
mod core;

use core::CoverageSnapshot;
use core::ExpectedDecision;
use core::ReviewRow;
use core::validate_tranche;
use core::validate_unchanged_coverage;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

const BASELINE_COVERAGE: &str = include_str!("fixtures/traceability-operator-diagnostics-promotion-current.json");
const CURRENT_COVERAGE: &str = include_str!("fixtures/traceability-core-credentials-current.json");
const REVIEWED_INVENTORY: &str = include_str!("fixtures/traceability-core-credentials-reviewed.json");
const CREDENTIAL_SOURCE: &str = include_str!("../src/credentials.rs");
const EXECUTOR_SOURCE: &str = include_str!("../src/engine/executor.rs");
const WORKFLOW_FORMAT_SOURCE: &str = include_str!("../src/workflow_format.rs");
const WORKFLOW_VALIDATION_SOURCE: &str = include_str!("../src/workflow_validation.rs");
const CREDENTIAL_TESTS: &str = include_str!("test_credentials.rs");
const CREDENTIAL_SOURCE_PATH: &str = "src/credentials.rs";
const EXECUTOR_SOURCE_PATH: &str = "src/engine/executor.rs";
const WORKFLOW_FORMAT_SOURCE_PATH: &str = "src/workflow_format.rs";
const WORKFLOW_VALIDATION_SOURCE_PATH: &str = "src/workflow_validation.rs";
const CREDENTIAL_TESTS_PATH: &str = "tests/test_credentials.rs";
const SUBSYSTEM: &str = "credentials";
const UNASSESSED_CLASSIFICATION: &str = "unassessed";
const EXACT_CLASSIFICATION: &str = "exact_maintained_evidence";
const PARTIAL_CLASSIFICATION: &str = "partial_evidence_only";
const TOTAL_COUNT: usize = 353;
const REVIEWED_COUNT: usize = 12;
const UNASSESSED_COUNT: usize = 341;
const EXACT_COUNT: usize = 12;
const UNRELATED_MISSING_ID: &str = "lattice.agent_orchestration.authority.scenario.ambient-denied";
const UNEXPECTED_ADDED_ID: &str = "lattice.example.unaccepted.scenario.added";
const DANGLING_ID: &str = "lattice.example.dangling.scenario.marker";
const CACHE_IDENTITY_ID: &str = concat!("n8n", "clone.credentials.cache_identity.reference");
const PROVIDER_MISSING_ID: &str = concat!("n8n", "clone.credentials.provider_interface.missing");
const PROVIDER_RESOLVE_ID: &str = concat!("n8n", "clone.credentials.provider_interface.resolve");
const REGISTRY_RESOLVE_ID: &str = concat!("n8n", "clone.credentials.provider_registry.resolve");
const REGISTRY_UNKNOWN_ID: &str = concat!("n8n", "clone.credentials.provider_registry.unknown");
const ENV_MISSING_ID: &str = concat!("n8n", "clone.credentials.providers.env.missing");
const ENV_RESOLVE_ID: &str = concat!("n8n", "clone.credentials.providers.env.resolve");
const REDACTION_OUTPUT_ID: &str = concat!("n8n", "clone.credentials.redaction.output");
const REDACTION_VALIDATION_ID: &str = concat!("n8n", "clone.credentials.redaction.validation");
const REJECT_PLAINTEXT_ID: &str = concat!("n8n", "clone.credentials.secret_refs.model.reject_plaintext");
const STORE_REF_ID: &str = concat!("n8n", "clone.credentials.secret_refs.model.store_ref");
const MALFORMED_ID: &str = concat!("n8n", "clone.credentials.validation.malformed");
const UNEXPECTED_REVIEW_ID: &str = concat!("n8n", "clone.credentials.unexpected");

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
            requirement_id: CACHE_IDENTITY_ID,
            classification: EXACT_CLASSIFICATION,
            rationale_code: "assertions.reference-only-cache-key",
            evidence_paths: &[EXECUTOR_SOURCE_PATH, CREDENTIAL_TESTS_PATH],
        },
        ExpectedDecision {
            requirement_id: PROVIDER_MISSING_ID,
            classification: EXACT_CLASSIFICATION,
            rationale_code: "assertions.pre-runtime-missing-secret",
            evidence_paths: &[CREDENTIAL_SOURCE_PATH, EXECUTOR_SOURCE_PATH, CREDENTIAL_TESTS_PATH],
        },
        ExpectedDecision {
            requirement_id: PROVIDER_RESOLVE_ID,
            classification: EXACT_CLASSIFICATION,
            rationale_code: "assertions.execution-boundary-resolution",
            evidence_paths: &[CREDENTIAL_SOURCE_PATH, EXECUTOR_SOURCE_PATH, CREDENTIAL_TESTS_PATH],
        },
        ExpectedDecision {
            requirement_id: REGISTRY_RESOLVE_ID,
            classification: EXACT_CLASSIFICATION,
            rationale_code: "assertions.registered-provider-resolution",
            evidence_paths: &[CREDENTIAL_SOURCE_PATH, CREDENTIAL_TESTS_PATH],
        },
        ExpectedDecision {
            requirement_id: REGISTRY_UNKNOWN_ID,
            classification: EXACT_CLASSIFICATION,
            rationale_code: "assertions.pre-runtime-unknown-provider",
            evidence_paths: &[
                CREDENTIAL_SOURCE_PATH,
                WORKFLOW_VALIDATION_SOURCE_PATH,
                CREDENTIAL_TESTS_PATH,
            ],
        },
        ExpectedDecision {
            requirement_id: ENV_MISSING_ID,
            classification: EXACT_CLASSIFICATION,
            rationale_code: "assertions.pre-runtime-missing-env",
            evidence_paths: &[CREDENTIAL_SOURCE_PATH, EXECUTOR_SOURCE_PATH, CREDENTIAL_TESTS_PATH],
        },
        ExpectedDecision {
            requirement_id: ENV_RESOLVE_ID,
            classification: EXACT_CLASSIFICATION,
            rationale_code: "assertions.environment-secret-resolution",
            evidence_paths: &[CREDENTIAL_SOURCE_PATH, EXECUTOR_SOURCE_PATH, CREDENTIAL_TESTS_PATH],
        },
        ExpectedDecision {
            requirement_id: REDACTION_OUTPUT_ID,
            classification: EXACT_CLASSIFICATION,
            rationale_code: "assertions.all-maintained-surfaces-redacted",
            evidence_paths: &[CREDENTIAL_SOURCE_PATH, EXECUTOR_SOURCE_PATH, CREDENTIAL_TESTS_PATH],
        },
        ExpectedDecision {
            requirement_id: REDACTION_VALIDATION_ID,
            classification: EXACT_CLASSIFICATION,
            rationale_code: "assertions.validation-text-json-safe",
            evidence_paths: &[
                CREDENTIAL_SOURCE_PATH,
                WORKFLOW_VALIDATION_SOURCE_PATH,
                CREDENTIAL_TESTS_PATH,
            ],
        },
        ExpectedDecision {
            requirement_id: REJECT_PLAINTEXT_ID,
            classification: EXACT_CLASSIFICATION,
            rationale_code: "assertions.typed-rejection-before-store",
            evidence_paths: &[WORKFLOW_FORMAT_SOURCE_PATH, CREDENTIAL_TESTS_PATH],
        },
        ExpectedDecision {
            requirement_id: STORE_REF_ID,
            classification: EXACT_CLASSIFICATION,
            rationale_code: "assertions.post-resolution-save-export-safe",
            evidence_paths: &[
                CREDENTIAL_SOURCE_PATH,
                WORKFLOW_FORMAT_SOURCE_PATH,
                CREDENTIAL_TESTS_PATH,
            ],
        },
        ExpectedDecision {
            requirement_id: MALFORMED_ID,
            classification: EXACT_CLASSIFICATION,
            rationale_code: "assertions.fatal-malformed-reference",
            evidence_paths: &[
                CREDENTIAL_SOURCE_PATH,
                WORKFLOW_VALIDATION_SOURCE_PATH,
                CREDENTIAL_TESTS_PATH,
            ],
        },
    ]
}

fn review_rows(report: &InventoryReport) -> Vec<ReviewRow> {
    report
        .rows
        .iter()
        .filter(|row| row.classification != UNASSESSED_CLASSIFICATION)
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
        (CREDENTIAL_SOURCE_PATH, blake3::hash(CREDENTIAL_SOURCE.as_bytes()).to_hex().to_string()),
        (EXECUTOR_SOURCE_PATH, blake3::hash(EXECUTOR_SOURCE.as_bytes()).to_hex().to_string()),
        (WORKFLOW_FORMAT_SOURCE_PATH, blake3::hash(WORKFLOW_FORMAT_SOURCE.as_bytes()).to_hex().to_string()),
        (
            WORKFLOW_VALIDATION_SOURCE_PATH,
            blake3::hash(WORKFLOW_VALIDATION_SOURCE.as_bytes()).to_hex().to_string(),
        ),
        (CREDENTIAL_TESTS_PATH, blake3::hash(CREDENTIAL_TESTS.as_bytes()).to_hex().to_string()),
    ])
}

// r[verify lattice.repository_validation.credential_traceability_audit]
// r[verify lattice.repository_validation.credential_traceability_audit.scenario.complete]
// r[verify lattice.repository_validation.credential_traceability_audit.scenario.evidence]
#[test]
fn core_credential_review_is_complete_bound_and_exact() {
    let report: InventoryReport = serde_json::from_str(REVIEWED_INVENTORY).expect("inventory must decode");
    let rows = review_rows(&report);
    let review_issues = validate_tranche(&rows, SUBSYSTEM, &expected_decisions());
    let baseline = coverage_snapshot(BASELINE_COVERAGE);
    let current = coverage_snapshot(CURRENT_COVERAGE);
    let coverage_issues = validate_unchanged_coverage(&baseline, &current);
    let actual_hashes = actual_evidence_hashes();
    let current_missing = current.missing.iter().map(String::as_str).collect::<BTreeSet<_>>();
    let reviewed_ids =
        expected_decisions().into_iter().map(|decision| decision.requirement_id).collect::<BTreeSet<_>>();

    assert!(review_issues.is_empty(), "review issues: {review_issues:?}");
    assert!(coverage_issues.is_empty(), "coverage issues: {coverage_issues:?}");
    assert!(report.valid);
    assert!(report.issues.is_empty());
    assert_eq!(report.summary.total, TOTAL_COUNT);
    assert_eq!(report.summary.reviewed, REVIEWED_COUNT);
    assert_eq!(report.summary.unassessed, UNASSESSED_COUNT);
    assert_eq!(report.summary.by_classification.get(EXACT_CLASSIFICATION), Some(&EXACT_COUNT));
    assert!(!report.summary.by_classification.contains_key(PARTIAL_CLASSIFICATION));
    assert_eq!(report.summary.by_classification.get(UNASSESSED_CLASSIFICATION), Some(&UNASSESSED_COUNT));

    for decision in expected_decisions() {
        assert!(current_missing.contains(decision.requirement_id));
        let verification_marker = format!("r[verify {}]", decision.requirement_id);
        assert!(CREDENTIAL_TESTS.contains(&verification_marker));
    }
    for row in report.rows.iter().filter(|row| row.subsystem == SUBSYSTEM) {
        if reviewed_ids.contains(row.requirement_id.as_str()) {
            assert_ne!(row.classification, UNASSESSED_CLASSIFICATION);
            for evidence in &row.evidence {
                assert_eq!(actual_hashes.get(evidence.path.as_str()), Some(&evidence.blake3));
            }
        } else {
            assert_eq!(row.classification, UNASSESSED_CLASSIFICATION);
            assert!(row.evidence.is_empty());
            assert_eq!(row.rationale_code, "not-reviewed");
        }
    }
}

// r[verify lattice.repository_validation.credential_traceability_audit.scenario.fail_closed]
#[test]
fn credential_classification_evidence_and_coverage_drift_fail_closed() {
    let report: InventoryReport = serde_json::from_str(REVIEWED_INVENTORY).expect("inventory must decode");
    let mut rows = review_rows(&report);
    rows.retain(|row| row.requirement_id != CACHE_IDENTITY_ID);
    rows.iter_mut()
        .find(|row| row.requirement_id == PROVIDER_MISSING_ID)
        .expect("provider missing row")
        .classification = PARTIAL_CLASSIFICATION.to_string();
    rows.iter_mut()
        .find(|row| row.requirement_id == PROVIDER_RESOLVE_ID)
        .expect("provider resolve row")
        .rationale_code = "invalid.rationale".to_string();
    rows.iter_mut()
        .find(|row| row.requirement_id == STORE_REF_ID)
        .expect("store ref row")
        .evidence_paths = Vec::new();
    rows.push(ReviewRow {
        requirement_id: UNEXPECTED_REVIEW_ID.to_string(),
        subsystem: SUBSYSTEM.to_string(),
        classification: PARTIAL_CLASSIFICATION.to_string(),
        rationale_code: "invalid.unexpected".to_string(),
        evidence_paths: vec![CREDENTIAL_SOURCE_PATH.to_string()],
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

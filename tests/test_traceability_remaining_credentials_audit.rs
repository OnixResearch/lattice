#[path = "traceability_review/core.rs"]
mod core;

use core::CoverageSnapshot;
use core::ExpectedDecision;
use core::ReviewRow;
use core::validate_tranche;
use core::validate_unchanged_coverage;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

const BASELINE_COVERAGE: &str = include_str!("fixtures/traceability-core-credentials-promotion-current.json");
const CURRENT_COVERAGE: &str = include_str!("fixtures/traceability-remaining-credentials-current.json");
const REVIEWED_INVENTORY: &str = include_str!("fixtures/traceability-remaining-credentials-reviewed.json");
const CREDENTIAL_SOURCE: &str = include_str!("../src/credentials.rs");
const REVIEWED_CREDENTIAL_SOURCE_HEX: &str = include_str!("fixtures/traceability-reviewed-credentials-source.hex");
const EXECUTOR_SOURCE: &str = include_str!("../src/engine/executor.rs");
const N8N_SOURCE: &str = include_str!("../src/n8n_json.rs");
const TRIGGER_SOURCE: &str = include_str!("../src/triggers.rs");
const WEBHOOK_CONTROLS_SOURCE: &str = include_str!("../src/webhooks/controls.rs");
const WEBHOOK_FLOW_SOURCE: &str = include_str!("../src/webhooks/flow.rs");
const CREDENTIAL_TESTS: &str = include_str!("test_credentials.rs");
const N8N_TESTS: &str = include_str!("test_n8n_json.rs");
const TRIGGER_TESTS: &str = include_str!("test_triggers.rs");
const WEBHOOK_TESTS: &str = include_str!("webhooks.rs");
const PLAINTEXT_N8N_FIXTURE: &str = include_str!("fixtures/n8n/plaintext-credentials.json");
const PARITY_PLAINTEXT_FIXTURE: &str = include_str!("fixtures/n8n/parity/unsupported/plaintext-node-credential.json");
const EXAMPLES_README: &str = include_str!("../examples/README.md");
const COOKBOOK_WORKFLOW: &str = include_str!("../examples/workflows/webhook-cookbook.ncl");

const CREDENTIAL_SOURCE_PATH: &str = "src/credentials.rs";
const EXECUTOR_SOURCE_PATH: &str = "src/engine/executor.rs";
const N8N_SOURCE_PATH: &str = "src/n8n_json.rs";
const TRIGGER_SOURCE_PATH: &str = "src/triggers.rs";
const WEBHOOK_CONTROLS_SOURCE_PATH: &str = "src/webhooks/controls.rs";
const WEBHOOK_FLOW_SOURCE_PATH: &str = "src/webhooks/flow.rs";
const CREDENTIAL_TESTS_PATH: &str = "tests/test_credentials.rs";
const N8N_TESTS_PATH: &str = "tests/test_n8n_json.rs";
const TRIGGER_TESTS_PATH: &str = "tests/test_triggers.rs";
const WEBHOOK_TESTS_PATH: &str = "tests/webhooks.rs";
const PLAINTEXT_N8N_FIXTURE_PATH: &str = "tests/fixtures/n8n/plaintext-credentials.json";
const PARITY_PLAINTEXT_FIXTURE_PATH: &str = "tests/fixtures/n8n/parity/unsupported/plaintext-node-credential.json";
const EXAMPLES_README_PATH: &str = "examples/README.md";
const COOKBOOK_WORKFLOW_PATH: &str = "examples/workflows/webhook-cookbook.ncl";

const SUBSYSTEM: &str = "credentials";
const UNASSESSED_CLASSIFICATION: &str = "unassessed";
const EXACT_CLASSIFICATION: &str = "exact_maintained_evidence";
const PARTIAL_CLASSIFICATION: &str = "partial_evidence_only";
const TOTAL_COUNT: usize = 341;
const REVIEWED_COUNT: usize = 14;
const UNASSESSED_COUNT: usize = 327;
const EXACT_COUNT: usize = 1;
const PARTIAL_COUNT: usize = 13;
const UNRELATED_MISSING_ID: &str = "lattice.agent_orchestration.authority.scenario.ambient-denied";
const UNEXPECTED_ADDED_ID: &str = "lattice.example.unaccepted.scenario.added";
const DANGLING_ID: &str = "lattice.example.dangling.scenario.marker";
const UNEXPECTED_REVIEW_ID: &str = concat!("n8n", "clone.credentials.unexpected.remaining");

const N8N_EXPORT_ID: &str = concat!("n8n", "clone.credentials.n8n_json.placeholders.export");
const N8N_PLAINTEXT_ID: &str = concat!("n8n", "clone.credentials.n8n_json.placeholders.plaintext");
const N8N_REFERENCE_ID: &str = concat!("n8n", "clone.credentials.n8n_json.placeholders.reference");
const N8N_PARITY_ID: &str = concat!("n8n", "clone.credentials.n8n_parity_secret_safety");
const PROVIDER_CACHE_ID: &str = concat!("n8n", "clone.credentials.provider_cache_identity.no_plaintext");
const PROVIDER_ERROR_ID: &str = concat!("n8n", "clone.credentials.provider_diagnostics.redaction.error");
const PROVIDER_TEST_ID: &str = concat!("n8n", "clone.credentials.provider_diagnostics.redaction.test");
const COMMAND_RESOLVE_ID: &str = concat!("n8n", "clone.credentials.providers.command.resolve");
const COMMAND_TIMEOUT_ID: &str = concat!("n8n", "clone.credentials.providers.command.timeout");
const VAULT_RESOLVE_ID: &str = concat!("n8n", "clone.credentials.providers.local_vault.resolve");
const VAULT_TAMPER_ID: &str = concat!("n8n", "clone.credentials.providers.local_vault.tamper");
const WEBHOOK_REDACTION_ID: &str = concat!("n8n", "clone.credentials.webhook_auth.redaction");
const WEBHOOK_VALID_ID: &str = concat!("n8n", "clone.credentials.webhook_auth.valid");
const COOKBOOK_SAFETY_ID: &str = concat!("n8n", "clone.credentials.webhook_cookbook_secret_safety");
const PROMOTED_COOKBOOK_MARKER: &str =
    concat!("// r[verify n8n", "clone.credentials.webhook_cookbook_secret_safety]\n",);
const PROMOTED_COOKBOOK_MARKER_COUNT: usize = 2;

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
            requirement_id: N8N_EXPORT_ID,
            classification: PARTIAL_CLASSIFICATION,
            rationale_code: "assertions.credential-export-observation-missing",
            evidence_paths: &[N8N_SOURCE_PATH, N8N_TESTS_PATH],
        },
        ExpectedDecision {
            requirement_id: N8N_PLAINTEXT_ID,
            classification: PARTIAL_CLASSIFICATION,
            rationale_code: "assertions.plaintext-save-boundary-missing",
            evidence_paths: &[N8N_SOURCE_PATH, N8N_TESTS_PATH, PLAINTEXT_N8N_FIXTURE_PATH],
        },
        ExpectedDecision {
            requirement_id: N8N_REFERENCE_ID,
            classification: PARTIAL_CLASSIFICATION,
            rationale_code: "assertions.imported-placeholder-resolution-missing",
            evidence_paths: &[N8N_SOURCE_PATH, EXECUTOR_SOURCE_PATH, N8N_TESTS_PATH],
        },
        ExpectedDecision {
            requirement_id: N8N_PARITY_ID,
            classification: PARTIAL_CLASSIFICATION,
            rationale_code: "contract.fixture-sentinel-and-output-gaps",
            evidence_paths: &[N8N_SOURCE_PATH, N8N_TESTS_PATH, PARITY_PLAINTEXT_FIXTURE_PATH],
        },
        ExpectedDecision {
            requirement_id: PROVIDER_CACHE_ID,
            classification: PARTIAL_CLASSIFICATION,
            rationale_code: "assertions.reference-dimensions-incomplete",
            evidence_paths: &[EXECUTOR_SOURCE_PATH, CREDENTIAL_TESTS_PATH],
        },
        ExpectedDecision {
            requirement_id: PROVIDER_ERROR_ID,
            classification: PARTIAL_CLASSIFICATION,
            rationale_code: "assertions.error-fragment-source-missing",
            evidence_paths: &[CREDENTIAL_SOURCE_PATH, CREDENTIAL_TESTS_PATH],
        },
        ExpectedDecision {
            requirement_id: PROVIDER_TEST_ID,
            classification: PARTIAL_CLASSIFICATION,
            rationale_code: "assertions.render-persist-observation-missing",
            evidence_paths: &[CREDENTIAL_SOURCE_PATH, CREDENTIAL_TESTS_PATH],
        },
        ExpectedDecision {
            requirement_id: COMMAND_RESOLVE_ID,
            classification: PARTIAL_CLASSIFICATION,
            rationale_code: "assertions.executor-boundary-missing",
            evidence_paths: &[CREDENTIAL_SOURCE_PATH, CREDENTIAL_TESTS_PATH],
        },
        ExpectedDecision {
            requirement_id: COMMAND_TIMEOUT_ID,
            classification: PARTIAL_CLASSIFICATION,
            rationale_code: "assertions.no-runtime-observation-missing",
            evidence_paths: &[CREDENTIAL_SOURCE_PATH, CREDENTIAL_TESTS_PATH],
        },
        ExpectedDecision {
            requirement_id: VAULT_RESOLVE_ID,
            classification: PARTIAL_CLASSIFICATION,
            rationale_code: "assertions.workflow-file-observation-missing",
            evidence_paths: &[CREDENTIAL_SOURCE_PATH, CREDENTIAL_TESTS_PATH],
        },
        ExpectedDecision {
            requirement_id: VAULT_TAMPER_ID,
            classification: PARTIAL_CLASSIFICATION,
            rationale_code: "assertions.rendered-tamper-diagnostic-missing",
            evidence_paths: &[CREDENTIAL_SOURCE_PATH, CREDENTIAL_TESTS_PATH],
        },
        ExpectedDecision {
            requirement_id: WEBHOOK_REDACTION_ID,
            classification: PARTIAL_CLASSIFICATION,
            rationale_code: "assertions.safe-reference-context-missing",
            evidence_paths: &[
                TRIGGER_SOURCE_PATH,
                WEBHOOK_FLOW_SOURCE_PATH,
                TRIGGER_TESTS_PATH,
                WEBHOOK_TESTS_PATH,
            ],
        },
        ExpectedDecision {
            requirement_id: WEBHOOK_VALID_ID,
            classification: PARTIAL_CLASSIFICATION,
            rationale_code: "assertions.persistence-observation-missing",
            evidence_paths: &[
                TRIGGER_SOURCE_PATH,
                WEBHOOK_FLOW_SOURCE_PATH,
                TRIGGER_TESTS_PATH,
                WEBHOOK_TESTS_PATH,
            ],
        },
        ExpectedDecision {
            requirement_id: COOKBOOK_SAFETY_ID,
            classification: EXACT_CLASSIFICATION,
            rationale_code: "assertions.cookbook-artifacts-and-smoke-surfaces",
            evidence_paths: &[
                TRIGGER_SOURCE_PATH,
                WEBHOOK_CONTROLS_SOURCE_PATH,
                WEBHOOK_FLOW_SOURCE_PATH,
                WEBHOOK_TESTS_PATH,
                EXAMPLES_README_PATH,
                COOKBOOK_WORKFLOW_PATH,
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

fn evidence_sources() -> Vec<&'static str> {
    vec![
        CREDENTIAL_SOURCE,
        EXECUTOR_SOURCE,
        N8N_SOURCE,
        TRIGGER_SOURCE,
        WEBHOOK_CONTROLS_SOURCE,
        WEBHOOK_FLOW_SOURCE,
        CREDENTIAL_TESTS,
        N8N_TESTS,
        TRIGGER_TESTS,
        WEBHOOK_TESTS,
        PLAINTEXT_N8N_FIXTURE,
        PARITY_PLAINTEXT_FIXTURE,
        EXAMPLES_README,
        COOKBOOK_WORKFLOW,
    ]
}

fn historical_webhook_tests() -> String {
    assert_eq!(WEBHOOK_TESTS.matches(PROMOTED_COOKBOOK_MARKER).count(), PROMOTED_COOKBOOK_MARKER_COUNT);
    WEBHOOK_TESTS.replace(PROMOTED_COOKBOOK_MARKER, "")
}

fn actual_evidence_hashes() -> BTreeMap<&'static str, String> {
    let historical_webhook_tests = historical_webhook_tests();
    let reviewed_credential_source =
        hex::decode(REVIEWED_CREDENTIAL_SOURCE_HEX.trim()).expect("reviewed source hex decodes");
    BTreeMap::from([
        (CREDENTIAL_SOURCE_PATH, blake3::hash(&reviewed_credential_source).to_hex().to_string()),
        (EXECUTOR_SOURCE_PATH, blake3::hash(EXECUTOR_SOURCE.as_bytes()).to_hex().to_string()),
        (N8N_SOURCE_PATH, blake3::hash(N8N_SOURCE.as_bytes()).to_hex().to_string()),
        (TRIGGER_SOURCE_PATH, blake3::hash(TRIGGER_SOURCE.as_bytes()).to_hex().to_string()),
        (WEBHOOK_CONTROLS_SOURCE_PATH, blake3::hash(WEBHOOK_CONTROLS_SOURCE.as_bytes()).to_hex().to_string()),
        (WEBHOOK_FLOW_SOURCE_PATH, blake3::hash(WEBHOOK_FLOW_SOURCE.as_bytes()).to_hex().to_string()),
        (CREDENTIAL_TESTS_PATH, blake3::hash(CREDENTIAL_TESTS.as_bytes()).to_hex().to_string()),
        (N8N_TESTS_PATH, blake3::hash(N8N_TESTS.as_bytes()).to_hex().to_string()),
        (TRIGGER_TESTS_PATH, blake3::hash(TRIGGER_TESTS.as_bytes()).to_hex().to_string()),
        (WEBHOOK_TESTS_PATH, blake3::hash(historical_webhook_tests.as_bytes()).to_hex().to_string()),
        (PLAINTEXT_N8N_FIXTURE_PATH, blake3::hash(PLAINTEXT_N8N_FIXTURE.as_bytes()).to_hex().to_string()),
        (
            PARITY_PLAINTEXT_FIXTURE_PATH,
            blake3::hash(PARITY_PLAINTEXT_FIXTURE.as_bytes()).to_hex().to_string(),
        ),
        (EXAMPLES_README_PATH, blake3::hash(EXAMPLES_README.as_bytes()).to_hex().to_string()),
        (COOKBOOK_WORKFLOW_PATH, blake3::hash(COOKBOOK_WORKFLOW.as_bytes()).to_hex().to_string()),
    ])
}

// r[verify lattice.repository_validation.remaining_credential_audit]
// r[verify lattice.repository_validation.remaining_credential_audit.scenario.complete]
// r[verify lattice.repository_validation.remaining_credential_audit.scenario.evidence]
#[test]
fn remaining_credential_review_is_complete_bound_and_non_promoting() {
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
    assert_eq!(report.summary.by_classification.get(PARTIAL_CLASSIFICATION), Some(&PARTIAL_COUNT));
    assert_eq!(report.summary.by_classification.get(UNASSESSED_CLASSIFICATION), Some(&UNASSESSED_COUNT));
    assert!(!report.summary.by_classification.contains_key("no_exact_evidence_identified"));

    for decision in expected_decisions() {
        assert!(current_missing.contains(decision.requirement_id));
        let implementation_marker = format!("r[impl {}]", decision.requirement_id);
        let verification_marker = format!("r[verify {}]", decision.requirement_id);
        assert!(evidence_sources().iter().all(|source| !source.contains(&implementation_marker)));
        if decision.requirement_id == COOKBOOK_SAFETY_ID {
            assert_eq!(WEBHOOK_TESTS.matches(&verification_marker).count(), PROMOTED_COOKBOOK_MARKER_COUNT);
            assert!(!historical_webhook_tests().contains(&verification_marker));
        } else {
            assert!(evidence_sources().iter().all(|source| !source.contains(&verification_marker)));
        }
    }
    for row in &report.rows {
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

// r[verify lattice.repository_validation.remaining_credential_audit.scenario.fail_closed]
#[test]
fn remaining_credential_classification_evidence_and_coverage_drift_fail_closed() {
    let report: InventoryReport = serde_json::from_str(REVIEWED_INVENTORY).expect("inventory must decode");
    let mut rows = review_rows(&report);
    rows.retain(|row| row.requirement_id != N8N_EXPORT_ID);
    rows.iter_mut()
        .find(|row| row.requirement_id == N8N_PLAINTEXT_ID)
        .expect("n8n plaintext row")
        .classification = EXACT_CLASSIFICATION.to_string();
    rows.iter_mut()
        .find(|row| row.requirement_id == PROVIDER_CACHE_ID)
        .expect("provider cache row")
        .rationale_code = "invalid.rationale".to_string();
    rows.iter_mut()
        .find(|row| row.requirement_id == COOKBOOK_SAFETY_ID)
        .expect("cookbook safety row")
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

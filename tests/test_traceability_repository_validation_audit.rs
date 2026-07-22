#[path = "traceability_review/core.rs"]
mod core;

use core::CoverageSnapshot;
use core::ExpectedDecision;
use core::ReviewRow;
use core::validate_no_selected_markers;
use core::validate_tranche;
use core::validate_unchanged_coverage;
use std::collections::BTreeSet;

const BASELINE_COVERAGE: &str = include_str!("fixtures/traceability-workflow-example-promotion-current.json");
const CURRENT_COVERAGE: &str = include_str!("fixtures/traceability-repository-validation-audit-current.json");
const REVIEWED_INVENTORY: &str = include_str!("fixtures/traceability-repository-validation-audit-reviewed.json");
const README_SOURCE: &str = include_str!("fixtures/traceability-repository-validation-audit-reviewed-readme.txt");
const FLAKE_SOURCE: &str = include_str!("fixtures/traceability-repository-validation-audit-reviewed-flake.txt");
const CARGO_SOURCE: &str = include_str!("fixtures/traceability-repository-validation-audit-reviewed-cargo-toml.txt");
const N8N_TEST_SOURCE: &str =
    include_str!("fixtures/traceability-repository-validation-audit-reviewed-test-n8n-json.txt");
const JSON_FIXTURE_SOURCE: &str =
    include_str!("fixtures/traceability-repository-validation-audit-reviewed-shell-json.txt");
const ANIMUS_RESULT_SOURCE: &str =
    include_str!("fixtures/traceability-repository-validation-audit-reviewed-animus-result.txt");
const RATS_COVERAGE_SOURCE: &str =
    include_str!("fixtures/traceability-repository-validation-audit-reviewed-rats-coverage-baseline.txt");

const README_PATH: &str = "README.md";
const FLAKE_PATH: &str = "flake.nix";
const CARGO_PATH: &str = "Cargo.toml";
const N8N_TEST_PATH: &str = "tests/test_n8n_json.rs";
const JSON_FIXTURE_PATH: &str = "tests/fixtures/n8n/shell.json";
const ANIMUS_RESULT_PATH: &str = "vendor-cargo-deps/animus/crates/animus-adapters/src/effect/result.rs";
const RATS_COVERAGE_PATH: &str = "vendor-cargo-deps/rats/source/ratcore/src/ratatui/coverage/baseline.rs";
const SUBSYSTEM: &str = "repository-validation";
const EXACT: &str = "exact_maintained_evidence";
const PARTIAL: &str = "partial_evidence_only";
const NO_EXACT: &str = "no_exact_evidence_identified";
const SELECTED_COUNT: usize = 11;

const CLIPPY_ID: &str = concat!("n8n", "clone.repository_validation.clippy_scope");
const CLIPPY_VENDOR_ID: &str = concat!("n8n", "clone.repository_validation.clippy_scope.scenario.vendor-lints");
const FLAKE_ID: &str = concat!("n8n", "clone.repository_validation.flake_check");
const FLAKE_MISSING_KEY_ID: &str =
    concat!("n8n", "clone.repository_validation.flake_check.scenario.missing-signing-key");
const FLAKE_NORMAL_SIGNING_ID: &str = concat!("n8n", "clone.repository_validation.flake_check.scenario.normal-signing");
const FORMAT_ID: &str = concat!("n8n", "clone.repository_validation.format_scope");
const FORMAT_VENDOR_ID: &str = concat!("n8n", "clone.repository_validation.format_scope.scenario.vendor-format");
const TEST_SOURCE_ID: &str = concat!("n8n", "clone.repository_validation.test_source");
const TEST_JSON_ID: &str = concat!("n8n", "clone.repository_validation.test_source.scenario.json-fixture");
const VENDOR_ID: &str = concat!("n8n", "clone.repository_validation.vendor_snapshot");
const VENDOR_NESTED_ID: &str =
    concat!("n8n", "clone.repository_validation.vendor_snapshot.scenario.nested-output-name");
const SELECTED_IDS: &[&str] = &[
    CLIPPY_ID,
    CLIPPY_VENDOR_ID,
    FLAKE_ID,
    FLAKE_MISSING_KEY_ID,
    FLAKE_NORMAL_SIGNING_ID,
    FORMAT_ID,
    FORMAT_VENDOR_ID,
    TEST_SOURCE_ID,
    TEST_JSON_ID,
    VENDOR_ID,
    VENDOR_NESTED_ID,
];
const UNRELATED_MISSING_ID: &str = "lattice.agent_orchestration.authority.scenario.ambient-denied";
const UNEXPECTED_ADDED_ID: &str = "lattice.example.unaccepted.scenario.added";
const DANGLING_ID: &str = "lattice.example.dangling.scenario.marker";
const UNEXPECTED_REVIEW_ID: &str = concat!("n8n", "clone.repository_validation.unexpected.audit-row");

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
            requirement_id: CLIPPY_ID,
            classification: PARTIAL,
            rationale_code: "gap.no-direct-vendor-lint-fixture",
            evidence_paths: &[CARGO_PATH, FLAKE_PATH],
        },
        ExpectedDecision {
            requirement_id: CLIPPY_VENDOR_ID,
            classification: PARTIAL,
            rationale_code: "gap.no-triggering-vendor-lint-fixture",
            evidence_paths: &[CARGO_PATH, FLAKE_PATH],
        },
        ExpectedDecision {
            requirement_id: FLAKE_ID,
            classification: PARTIAL,
            rationale_code: "gap.no-hostile-host-signing-check",
            evidence_paths: &[README_PATH, FLAKE_PATH],
        },
        ExpectedDecision {
            requirement_id: FLAKE_MISSING_KEY_ID,
            classification: PARTIAL,
            rationale_code: "gap.no-missing-key-process-check",
            evidence_paths: &[README_PATH, FLAKE_PATH],
        },
        ExpectedDecision {
            requirement_id: FLAKE_NORMAL_SIGNING_ID,
            classification: PARTIAL,
            rationale_code: "gap.no-external-signing-policy-assertion",
            evidence_paths: &[README_PATH, FLAKE_PATH],
        },
        ExpectedDecision {
            requirement_id: FORMAT_ID,
            classification: PARTIAL,
            rationale_code: "gap.no-direct-vendor-format-fixture",
            evidence_paths: &[CARGO_PATH, FLAKE_PATH],
        },
        ExpectedDecision {
            requirement_id: FORMAT_VENDOR_ID,
            classification: PARTIAL,
            rationale_code: "gap.no-upstream-format-difference-assertion",
            evidence_paths: &[CARGO_PATH, FLAKE_PATH],
        },
        ExpectedDecision {
            requirement_id: TEST_SOURCE_ID,
            classification: EXACT,
            rationale_code: "assertions.nix-nextest-compiles-tracked-test-fixtures",
            evidence_paths: &[FLAKE_PATH, JSON_FIXTURE_PATH, N8N_TEST_PATH],
        },
        ExpectedDecision {
            requirement_id: TEST_JSON_ID,
            classification: EXACT,
            rationale_code: "assertions.filtered-json-fixture-compiles",
            evidence_paths: &[FLAKE_PATH, JSON_FIXTURE_PATH, N8N_TEST_PATH],
        },
        ExpectedDecision {
            requirement_id: VENDOR_ID,
            classification: PARTIAL,
            rationale_code: "gap.no-nested-vendor-source-closure-assertion",
            evidence_paths: &[CARGO_PATH, FLAKE_PATH, ANIMUS_RESULT_PATH, RATS_COVERAGE_PATH],
        },
        ExpectedDecision {
            requirement_id: VENDOR_NESTED_ID,
            classification: PARTIAL,
            rationale_code: "gap.no-git-and-nix-nested-path-assertion",
            evidence_paths: &[CARGO_PATH, FLAKE_PATH, ANIMUS_RESULT_PATH, RATS_COVERAGE_PATH],
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
        README_PATH => README_SOURCE,
        FLAKE_PATH => FLAKE_SOURCE,
        CARGO_PATH => CARGO_SOURCE,
        N8N_TEST_PATH => N8N_TEST_SOURCE,
        JSON_FIXTURE_PATH => JSON_FIXTURE_SOURCE,
        ANIMUS_RESULT_PATH => ANIMUS_RESULT_SOURCE,
        RATS_COVERAGE_PATH => RATS_COVERAGE_SOURCE,
        _ => panic!("unexpected evidence path: {path}"),
    }
}

// r[verify lattice.repository_validation.repository_checks_audit]
// r[verify lattice.repository_validation.repository_checks_audit.scenario.complete]
// r[verify lattice.repository_validation.repository_checks_audit.scenario.evidence]
#[test]
fn repository_check_audit_is_complete_byte_bound_and_unpromoted() {
    let report: InventoryReport = serde_json::from_str(REVIEWED_INVENTORY).expect("inventory must decode");
    let review_issues = validate_tranche(&review_rows(&report), SUBSYSTEM, &expected_decisions());
    let coverage_issues = validate_unchanged_coverage(&coverage(BASELINE_COVERAGE), &coverage(CURRENT_COVERAGE));
    let sources = [
        README_SOURCE,
        FLAKE_SOURCE,
        CARGO_SOURCE,
        N8N_TEST_SOURCE,
        JSON_FIXTURE_SOURCE,
        ANIMUS_RESULT_SOURCE,
        RATS_COVERAGE_SOURCE,
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

// r[verify lattice.repository_validation.repository_checks_audit.scenario.fail_closed]
#[test]
fn repository_check_audit_classification_marker_evidence_and_coverage_drift_fail_closed() {
    let report: InventoryReport = serde_json::from_str(REVIEWED_INVENTORY).expect("inventory must decode");
    let mut rows = review_rows(&report);
    rows.retain(|row| row.requirement_id != CLIPPY_ID);
    let flake = rows.iter_mut().find(|row| row.requirement_id == FLAKE_ID).expect("flake row");
    flake.classification = EXACT.to_string();
    flake.rationale_code = "invalid.rationale".to_string();
    flake.evidence_paths = vec!["../unsafe".to_string()];
    rows.push(ReviewRow {
        requirement_id: UNEXPECTED_REVIEW_ID.to_string(),
        subsystem: SUBSYSTEM.to_string(),
        classification: NO_EXACT.to_string(),
        rationale_code: "invalid.unexpected".to_string(),
        evidence_paths: Vec::new(),
    });
    let review_issues = validate_tranche(&rows, SUBSYSTEM, &expected_decisions());
    let review_codes = issue_codes(&review_issues);

    let promoted = format!("// r[verify {CLIPPY_ID}]");
    let marker_issues = validate_no_selected_markers(&[promoted.as_str()], &[CLIPPY_ID, CLIPPY_ID]);
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

#[path = "traceability_review/core.rs"]
mod core;

use core::CoverageSnapshot;
use core::ExpectedDecision;
use core::ReviewRow;
use core::validate_no_selected_markers;
use core::validate_tranche;
use core::validate_unchanged_coverage;
use std::borrow::Cow;
use std::collections::BTreeSet;

const BASELINE_COVERAGE: &str = include_str!("fixtures/traceability-repository-validation-audit-current.json");
const CURRENT_COVERAGE: &str = include_str!("fixtures/traceability-repository-validation-completion-current.json");
const REVIEWED_INVENTORY: &str = include_str!("fixtures/traceability-repository-validation-completion-reviewed.json");
const README_SOURCE: &str = include_str!("../README.md");
const FLAKE_SOURCE: &str = include_str!("fixtures/traceability-repository-validation-completion-reviewed-flake.txt");
const CARGO_SOURCE: &str = include_str!("../Cargo.toml");
const REPOSITORY_TEST_SOURCE: &str = include_str!("test_repository_validation.rs");
const JSON_FIXTURE_SOURCE: &str = include_str!("fixtures/n8n/shell.json");
const VENDOR_MANIFEST_SOURCE: &str = include_str!("fixtures/repository-validation/vendor-dependency/Cargo.toml");
const VENDOR_LIB_SOURCE: &str = include_str!("fixtures/repository-validation/vendor-dependency/src/lib.rs");
const VENDOR_RESULT_SOURCE: &str =
    include_str!("fixtures/repository-validation/vendor-dependency/src/coverage/result.rs");

const README_PATH: &str = "README.md";
const FLAKE_PATH: &str = "flake.nix";
const CARGO_PATH: &str = "Cargo.toml";
const REPOSITORY_TEST_PATH: &str = "tests/test_repository_validation.rs";
const JSON_FIXTURE_PATH: &str = "tests/fixtures/n8n/shell.json";
const VENDOR_MANIFEST_PATH: &str = "tests/fixtures/repository-validation/vendor-dependency/Cargo.toml";
const VENDOR_LIB_PATH: &str = "tests/fixtures/repository-validation/vendor-dependency/src/lib.rs";
const VENDOR_RESULT_PATH: &str = "tests/fixtures/repository-validation/vendor-dependency/src/coverage/result.rs";
const SUBSYSTEM: &str = "repository-validation";
const EXACT: &str = "exact_maintained_evidence";
const PARTIAL: &str = "partial_evidence_only";
const SELECTED_COUNT: usize = 11;
const ZERO_MARKERS: usize = 0;
const ONE_MARKER: usize = 1;

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
const SELECTED_MARKERS: &[&str] = &[
    concat!("// r[verify n8n", "clone.repository_validation.clippy_scope]\n"),
    concat!("// r[verify n8n", "clone.repository_validation.clippy_scope.scenario.vendor-lints]\n"),
    concat!("// r[verify n8n", "clone.repository_validation.flake_check]\n"),
    concat!("// r[verify n8n", "clone.repository_validation.flake_check.scenario.missing-signing-key]\n"),
    concat!("// r[verify n8n", "clone.repository_validation.flake_check.scenario.normal-signing]\n"),
    concat!("// r[verify n8n", "clone.repository_validation.format_scope]\n"),
    concat!("// r[verify n8n", "clone.repository_validation.format_scope.scenario.vendor-format]\n"),
    concat!("// r[verify n8n", "clone.repository_validation.test_source]\n"),
    concat!("// r[verify n8n", "clone.repository_validation.test_source.scenario.json-fixture]\n"),
    concat!("// r[verify n8n", "clone.repository_validation.vendor_snapshot]\n"),
    concat!("// r[verify n8n", "clone.repository_validation.vendor_snapshot.scenario.nested-output-name]\n"),
];
const UNRELATED_MISSING_ID: &str = "lattice.agent_orchestration.authority.scenario.ambient-denied";
const UNEXPECTED_ADDED_ID: &str = "lattice.example.unaccepted.scenario.added";
const DANGLING_ID: &str = "lattice.example.dangling.scenario.marker";

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
            classification: EXACT,
            rationale_code: "assertions.package-clippy-no-deps-with-deny-fixture",
            evidence_paths: &[CARGO_PATH, FLAKE_PATH, VENDOR_LIB_PATH, REPOSITORY_TEST_PATH],
        },
        ExpectedDecision {
            requirement_id: CLIPPY_VENDOR_ID,
            classification: EXACT,
            rationale_code: "assertions.vendor-clippy-deny-excluded",
            evidence_paths: &[CARGO_PATH, FLAKE_PATH, VENDOR_LIB_PATH, REPOSITORY_TEST_PATH],
        },
        ExpectedDecision {
            requirement_id: FLAKE_ID,
            classification: EXACT,
            rationale_code: "assertions.check-only-signing-override-and-external-policy",
            evidence_paths: &[README_PATH, FLAKE_PATH, REPOSITORY_TEST_PATH],
        },
        ExpectedDecision {
            requirement_id: FLAKE_MISSING_KEY_ID,
            classification: EXACT,
            rationale_code: "assertions.missing-key-overridden-for-full-check",
            evidence_paths: &[README_PATH, FLAKE_PATH, REPOSITORY_TEST_PATH],
        },
        ExpectedDecision {
            requirement_id: FLAKE_NORMAL_SIGNING_ID,
            classification: EXACT,
            rationale_code: "assertions.flake-signing-policy-absent",
            evidence_paths: &[README_PATH, FLAKE_PATH, REPOSITORY_TEST_PATH],
        },
        ExpectedDecision {
            requirement_id: FORMAT_ID,
            classification: EXACT,
            rationale_code: "assertions.package-format-excludes-vendor-fixture",
            evidence_paths: &[FLAKE_PATH, VENDOR_RESULT_PATH, REPOSITORY_TEST_PATH],
        },
        ExpectedDecision {
            requirement_id: FORMAT_VENDOR_ID,
            classification: EXACT,
            rationale_code: "assertions.nonconforming-vendor-format-excluded",
            evidence_paths: &[FLAKE_PATH, VENDOR_RESULT_PATH, REPOSITORY_TEST_PATH],
        },
        ExpectedDecision {
            requirement_id: TEST_SOURCE_ID,
            classification: EXACT,
            rationale_code: "assertions.nix-nextest-compiles-tracked-test-fixtures",
            evidence_paths: &[FLAKE_PATH, JSON_FIXTURE_PATH, REPOSITORY_TEST_PATH],
        },
        ExpectedDecision {
            requirement_id: TEST_JSON_ID,
            classification: EXACT,
            rationale_code: "assertions.filtered-json-fixture-compiles",
            evidence_paths: &[FLAKE_PATH, JSON_FIXTURE_PATH, REPOSITORY_TEST_PATH],
        },
        ExpectedDecision {
            requirement_id: VENDOR_ID,
            classification: EXACT,
            rationale_code: "assertions.git-and-nix-nested-source-admission",
            evidence_paths: &[
                CARGO_PATH,
                FLAKE_PATH,
                VENDOR_MANIFEST_PATH,
                VENDOR_RESULT_PATH,
                REPOSITORY_TEST_PATH,
            ],
        },
        ExpectedDecision {
            requirement_id: VENDOR_NESTED_ID,
            classification: EXACT,
            rationale_code: "assertions.nested-output-name-admitted",
            evidence_paths: &[
                CARGO_PATH,
                FLAKE_PATH,
                VENDOR_MANIFEST_PATH,
                VENDOR_RESULT_PATH,
                REPOSITORY_TEST_PATH,
            ],
        },
    ]
}

fn rows(report: &InventoryReport) -> Vec<ReviewRow> {
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

fn historical_repository_test() -> Cow<'static, str> {
    let marker_count =
        SELECTED_MARKERS.iter().map(|marker| REPOSITORY_TEST_SOURCE.matches(marker).count()).sum::<usize>();
    if marker_count == ZERO_MARKERS {
        return Cow::Borrowed(REPOSITORY_TEST_SOURCE);
    }
    for marker in SELECTED_MARKERS {
        assert_eq!(REPOSITORY_TEST_SOURCE.matches(marker).count(), ONE_MARKER);
    }
    let mut historical = REPOSITORY_TEST_SOURCE.to_string();
    for marker in SELECTED_MARKERS {
        historical = historical.replace(marker, "");
    }
    Cow::Owned(historical)
}

fn source(path: &str) -> Cow<'static, str> {
    match path {
        README_PATH => Cow::Borrowed(README_SOURCE),
        FLAKE_PATH => Cow::Borrowed(FLAKE_SOURCE),
        CARGO_PATH => Cow::Borrowed(CARGO_SOURCE),
        REPOSITORY_TEST_PATH => historical_repository_test(),
        JSON_FIXTURE_PATH => Cow::Borrowed(JSON_FIXTURE_SOURCE),
        VENDOR_MANIFEST_PATH => Cow::Borrowed(VENDOR_MANIFEST_SOURCE),
        VENDOR_LIB_PATH => Cow::Borrowed(VENDOR_LIB_SOURCE),
        VENDOR_RESULT_PATH => Cow::Borrowed(VENDOR_RESULT_SOURCE),
        _ => panic!("unexpected evidence path: {path}"),
    }
}

fn codes(issues: &[core::ReviewIssue]) -> BTreeSet<&str> {
    issues.iter().map(|issue| issue.code).collect()
}

// r[verify lattice.repository_validation.repository_checks_completion]
// r[verify lattice.repository_validation.repository_checks_completion.scenario.signing]
// r[verify lattice.repository_validation.repository_checks_completion.scenario.source_closure]
// r[verify lattice.repository_validation.repository_checks_completion.scenario.ownership]
// r[verify lattice.repository_validation.repository_checks_completion.scenario.exact]
#[test]
fn all_repository_validation_rows_are_exact_byte_bound_and_unpromoted() {
    let report: InventoryReport = serde_json::from_str(REVIEWED_INVENTORY).expect("inventory must decode");
    let review_issues = validate_tranche(&rows(&report), SUBSYSTEM, &expected_decisions());
    let coverage_issues = validate_unchanged_coverage(&coverage(BASELINE_COVERAGE), &coverage(CURRENT_COVERAGE));
    let historical_test = historical_repository_test();
    let sources = [
        README_SOURCE,
        FLAKE_SOURCE,
        CARGO_SOURCE,
        historical_test.as_ref(),
        JSON_FIXTURE_SOURCE,
        VENDOR_MANIFEST_SOURCE,
        VENDOR_LIB_SOURCE,
        VENDOR_RESULT_SOURCE,
    ];
    let marker_issues = validate_no_selected_markers(&sources, SELECTED_IDS);
    let selected = report.rows.iter().filter(|row| SELECTED_IDS.contains(&row.requirement_id.as_str()));

    assert!(review_issues.is_empty(), "review issues: {review_issues:?}");
    assert!(coverage_issues.is_empty(), "coverage issues: {coverage_issues:?}");
    assert!(marker_issues.is_empty(), "marker issues: {marker_issues:?}");
    assert_eq!(selected.count(), SELECTED_COUNT);
    for row in report.rows.iter().filter(|row| SELECTED_IDS.contains(&row.requirement_id.as_str())) {
        for evidence in &row.evidence {
            assert_eq!(blake3::hash(source(&evidence.path).as_bytes()).to_hex().to_string(), evidence.blake3);
        }
    }
}

// r[verify lattice.repository_validation.repository_checks_completion.scenario.fail_closed]
#[test]
fn repository_validation_completion_drift_fails_closed() {
    let report: InventoryReport = serde_json::from_str(REVIEWED_INVENTORY).expect("inventory must decode");
    let mut review_rows = rows(&report);
    review_rows.retain(|row| row.requirement_id != CLIPPY_ID);
    let flake = review_rows.iter_mut().find(|row| row.requirement_id == FLAKE_ID).expect("flake row");
    flake.classification = PARTIAL.to_string();
    flake.rationale_code = "invalid.rationale".to_string();
    flake.evidence_paths = vec!["../unsafe".to_string()];
    let review_issues = validate_tranche(&review_rows, SUBSYSTEM, &expected_decisions());
    let review_codes = codes(&review_issues);

    let promoted = format!("// r[verify {CLIPPY_ID}]");
    let marker_issues = validate_no_selected_markers(&[promoted.as_str()], &[CLIPPY_ID, CLIPPY_ID]);
    let marker_codes = codes(&marker_issues);

    let baseline = coverage(BASELINE_COVERAGE);
    let mut current = coverage(CURRENT_COVERAGE);
    current.missing.retain(|identifier| identifier != UNRELATED_MISSING_ID);
    current.missing.push(UNEXPECTED_ADDED_ID.to_string());
    current.missing.push(UNEXPECTED_ADDED_ID.to_string());
    current.dangling.push(DANGLING_ID.to_string());
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

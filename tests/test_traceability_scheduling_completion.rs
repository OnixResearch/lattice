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

const BASELINE_COVERAGE: &str = include_str!("fixtures/traceability-scheduling-audit-current.json");
const CURRENT_COVERAGE: &str = include_str!("fixtures/traceability-scheduling-completion-current.json");
const REVIEWED_INVENTORY: &str = include_str!("fixtures/traceability-scheduling-completion-reviewed.json");
const TRIGGER_TESTS: &str = include_str!("fixtures/traceability-workflow-format-audit-reviewed-triggers.txt");
const WEBHOOK_TESTS: &str = include_str!("fixtures/traceability-workflow-format-audit-reviewed-webhooks.txt");
const HEADLESS_SOURCE: &str = include_str!("fixtures/traceability-reviewed-headless-source.txt");
const TRIGGER_SOURCE: &str = include_str!("../src/triggers.rs");
const MAIN_SOURCE: &str = include_str!("fixtures/traceability-n8n-credential-completion-reviewed-main.txt");
const EXAMPLES_README: &str = include_str!("../examples/README.md");
const COOKBOOK_WORKFLOW: &str = include_str!("../examples/workflows/webhook-cookbook.ncl");
const TRIGGER_TESTS_PATH: &str = "tests/test_triggers.rs";
const WEBHOOK_TESTS_PATH: &str = "tests/webhooks.rs";
const HEADLESS_SOURCE_PATH: &str = "src/headless.rs";
const TRIGGER_SOURCE_PATH: &str = "src/triggers.rs";
const MAIN_SOURCE_PATH: &str = "src/main.rs";
const EXAMPLES_README_PATH: &str = "examples/README.md";
const COOKBOOK_WORKFLOW_PATH: &str = "examples/workflows/webhook-cookbook.ncl";
const SUBSYSTEM: &str = "scheduling";
const EXACT: &str = "exact_maintained_evidence";
const PARTIAL: &str = "partial_evidence_only";
const SELECTED_COUNT: usize = 7;
const DISABLED_ID: &str = concat!("n8n", "clone.scheduling.triggers.admission.disabled");
const HISTORY_ID: &str = concat!("n8n", "clone.scheduling.triggers.history.persist");
const COMPAT_ID: &str = concat!("n8n", "clone.scheduling.triggers.scheduler_compat.due");
const INVALID_ID: &str = concat!("n8n", "clone.scheduling.triggers.webhook.invalid");
const VALID_ID: &str = concat!("n8n", "clone.scheduling.triggers.webhook.valid");
const COOKBOOK_ID: &str = concat!("n8n", "clone.scheduling.webhook_cookbook_operator_flow");
const LISTENER_ID: &str = concat!("n8n", "clone.scheduling.webhook_listener.process.start");
const SELECTED_IDS: &[&str] = &[
    DISABLED_ID,
    HISTORY_ID,
    COMPAT_ID,
    INVALID_ID,
    VALID_ID,
    COOKBOOK_ID,
    LISTENER_ID,
];
const DISABLED_MARKER: &str = concat!("// r[verify n8n", "clone.scheduling.triggers.admission.disabled]\n");
const HISTORY_MARKER: &str = concat!("// r[verify n8n", "clone.scheduling.triggers.history.persist]\n");
const COMPAT_MARKER: &str = concat!("// r[verify n8n", "clone.scheduling.triggers.scheduler_compat.due]\n");
const INVALID_MARKER: &str = concat!("// r[verify n8n", "clone.scheduling.triggers.webhook.invalid]\n");
const VALID_MARKER: &str = concat!("// r[verify n8n", "clone.scheduling.triggers.webhook.valid]\n");
const COOKBOOK_MARKER: &str = concat!("// r[verify n8n", "clone.scheduling.webhook_cookbook_operator_flow]\n");
const LISTENER_MARKER: &str = concat!("// r[verify n8n", "clone.scheduling.webhook_listener.process.start]\n");
const SINGLE_MARKER_COUNT: usize = 1;
const VALID_TRIGGER_MARKER_COUNT: usize = 3;
const COOKBOOK_MARKER_COUNT: usize = 3;
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
            requirement_id: DISABLED_ID,
            classification: EXACT,
            rationale_code: "assertions.disabled-rejected-without-run",
            evidence_paths: &[WEBHOOK_TESTS_PATH],
        },
        ExpectedDecision {
            requirement_id: HISTORY_ID,
            classification: EXACT,
            rationale_code: "assertions.terminal-history-complete-and-safe",
            evidence_paths: &[HEADLESS_SOURCE_PATH, TRIGGER_SOURCE_PATH, TRIGGER_TESTS_PATH],
        },
        ExpectedDecision {
            requirement_id: COMPAT_ID,
            classification: EXACT,
            rationale_code: "assertions.full-legacy-adapted-plan-equality",
            evidence_paths: &[TRIGGER_TESTS_PATH],
        },
        ExpectedDecision {
            requirement_id: INVALID_ID,
            classification: EXACT,
            rationale_code: "assertions.invalid-webhooks-reject-without-run",
            evidence_paths: &[TRIGGER_TESTS_PATH, WEBHOOK_TESTS_PATH],
        },
        ExpectedDecision {
            requirement_id: VALID_ID,
            classification: EXACT,
            rationale_code: "assertions.valid-webhook-binds-and-runs",
            evidence_paths: &[TRIGGER_TESTS_PATH, WEBHOOK_TESTS_PATH],
        },
        ExpectedDecision {
            requirement_id: COOKBOOK_ID,
            classification: EXACT,
            rationale_code: "assertions.cookbook-import-dry-run-admit-reject-history",
            evidence_paths: &[EXAMPLES_README_PATH, COOKBOOK_WORKFLOW_PATH, WEBHOOK_TESTS_PATH],
        },
        ExpectedDecision {
            requirement_id: LISTENER_ID,
            classification: EXACT,
            rationale_code: "assertions.live-listener-process-accepts-and-persists",
            evidence_paths: &[MAIN_SOURCE_PATH, WEBHOOK_TESTS_PATH],
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

fn historical_trigger_tests() -> String {
    assert_eq!(TRIGGER_TESTS.matches(HISTORY_MARKER).count(), SINGLE_MARKER_COUNT);
    assert_eq!(TRIGGER_TESTS.matches(COMPAT_MARKER).count(), SINGLE_MARKER_COUNT);
    assert_eq!(TRIGGER_TESTS.matches(INVALID_MARKER).count(), SINGLE_MARKER_COUNT);
    assert_eq!(TRIGGER_TESTS.matches(VALID_MARKER).count(), VALID_TRIGGER_MARKER_COUNT);
    TRIGGER_TESTS
        .replace(HISTORY_MARKER, "")
        .replace(COMPAT_MARKER, "")
        .replace(INVALID_MARKER, "")
        .replace(VALID_MARKER, "")
}

fn historical_webhook_tests() -> String {
    assert_eq!(WEBHOOK_TESTS.matches(DISABLED_MARKER).count(), SINGLE_MARKER_COUNT);
    assert_eq!(WEBHOOK_TESTS.matches(INVALID_MARKER).count(), SINGLE_MARKER_COUNT);
    assert_eq!(WEBHOOK_TESTS.matches(COOKBOOK_MARKER).count(), COOKBOOK_MARKER_COUNT);
    assert_eq!(WEBHOOK_TESTS.matches(LISTENER_MARKER).count(), SINGLE_MARKER_COUNT);
    WEBHOOK_TESTS
        .replace(DISABLED_MARKER, "")
        .replace(INVALID_MARKER, "")
        .replace(COOKBOOK_MARKER, "")
        .replace(LISTENER_MARKER, "")
}

fn source(path: &str) -> Cow<'static, str> {
    match path {
        TRIGGER_TESTS_PATH => Cow::Owned(historical_trigger_tests()),
        WEBHOOK_TESTS_PATH => Cow::Owned(historical_webhook_tests()),
        HEADLESS_SOURCE_PATH => Cow::Borrowed(HEADLESS_SOURCE),
        TRIGGER_SOURCE_PATH => Cow::Borrowed(TRIGGER_SOURCE),
        MAIN_SOURCE_PATH => Cow::Borrowed(MAIN_SOURCE),
        EXAMPLES_README_PATH => Cow::Borrowed(EXAMPLES_README),
        COOKBOOK_WORKFLOW_PATH => Cow::Borrowed(COOKBOOK_WORKFLOW),
        _ => panic!("unexpected evidence path: {path}"),
    }
}

fn codes(issues: &[core::ReviewIssue]) -> BTreeSet<&str> {
    issues.iter().map(|issue| issue.code).collect()
}

// r[verify lattice.repository_validation.scheduling_completion]
// r[verify lattice.repository_validation.scheduling_completion.scenario.history]
// r[verify lattice.repository_validation.scheduling_completion.scenario.compatibility]
// r[verify lattice.repository_validation.scheduling_completion.scenario.listener]
// r[verify lattice.repository_validation.scheduling_completion.scenario.exact]
#[test]
fn all_scheduling_rows_are_exact_byte_bound_and_unpromoted() {
    let report: InventoryReport = serde_json::from_str(REVIEWED_INVENTORY).expect("inventory must decode");
    let review_issues = validate_tranche(&rows(&report), SUBSYSTEM, &expected_decisions());
    let coverage_issues = validate_unchanged_coverage(&coverage(BASELINE_COVERAGE), &coverage(CURRENT_COVERAGE));
    let historical_trigger_tests = historical_trigger_tests();
    let historical_webhook_tests = historical_webhook_tests();
    let sources = [
        historical_trigger_tests.as_str(),
        historical_webhook_tests.as_str(),
        HEADLESS_SOURCE,
        TRIGGER_SOURCE,
        MAIN_SOURCE,
        EXAMPLES_README,
        COOKBOOK_WORKFLOW,
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

#[test]
fn scheduling_completion_drift_fails_closed() {
    let report: InventoryReport = serde_json::from_str(REVIEWED_INVENTORY).expect("inventory must decode");
    let mut review_rows = rows(&report);
    review_rows.retain(|row| row.requirement_id != DISABLED_ID);
    let history = review_rows.iter_mut().find(|row| row.requirement_id == HISTORY_ID).expect("history row");
    history.classification = PARTIAL.to_string();
    history.rationale_code = "invalid.rationale".to_string();
    history.evidence_paths = vec!["../unsafe".to_string()];
    let review_issues = validate_tranche(&review_rows, SUBSYSTEM, &expected_decisions());
    let review_codes = codes(&review_issues);
    let promoted = format!("// r[verify {DISABLED_ID}]");
    let marker_issues = validate_no_selected_markers(&[promoted.as_str()], &[DISABLED_ID, DISABLED_ID]);
    let marker_codes = codes(&marker_issues);
    let baseline = coverage(BASELINE_COVERAGE);
    let mut current = coverage(CURRENT_COVERAGE);
    current.missing.retain(|id| id != UNRELATED_MISSING_ID);
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

#[path = "traceability_review/core.rs"]
mod core;

use core::CoverageSnapshot;
use core::ExpectedDecision;
use core::ReviewRow;
use std::collections::BTreeSet;

const BASELINE: &str = include_str!("fixtures/traceability-workflow-validation-audit-current.json");
const CURRENT: &str = include_str!("fixtures/traceability-workflow-validation-audit-current.json");
const REVIEWED: &str = include_str!("fixtures/traceability-workflow-validation-completion-reviewed.json");
const WORKFLOW: &str = include_str!("fixtures/traceability-workflow-validation-audit-reviewed-workflow-validation.txt");
const N8N: &str = include_str!("fixtures/traceability-workflow-validation-audit-reviewed-n8n-json.txt");
const REPLAY: &str = include_str!("fixtures/traceability-workflow-validation-audit-reviewed-replay.txt");
const CONTROL: &str = include_str!("fixtures/traceability-workflow-validation-audit-reviewed-control-flow.txt");
const RUNNER: &str = include_str!("fixtures/traceability-workflow-validation-completion-reviewed-runner.txt");
const VALIDATION: &str = include_str!("fixtures/traceability-workflow-validation-completion-reviewed-validation.txt");
const TESTS: &str = include_str!("fixtures/traceability-workflow-validation-completion-reviewed-tests.txt");

const WORKFLOW_PATH: &str = "tests/test_workflow_validation.rs";
const N8N_PATH: &str = "tests/test_n8n_json.rs";
const REPLAY_PATH: &str = "tests/test_replay.rs";
const CONTROL_PATH: &str = "tests/test_control_flow.rs";
const RUNNER_PATH: &str = "src/engine/runner.rs";
const VALIDATION_PATH: &str = "src/workflow_validation.rs";
const TESTS_PATH: &str = "tests/test_workflow_validation_completion.rs";
const SUBSYSTEM: &str = "workflow-validation";
const EXACT: &str = "exact_maintained_evidence";
const SELECTED_COUNT: usize = 29;
const UNRELATED: &str = "lattice.agent_orchestration.authority.scenario.ambient-denied";
const ADDED: &str = "lattice.example.unaccepted.scenario.added";
const DANGLING: &str = "lattice.example.dangling.scenario.marker";

const IDS: &[&str] = &[
    concat!("n8n", "clone.workflow_validation.credentials.providers.config"),
    concat!("n8n", "clone.workflow_validation.credentials.providers.field"),
    concat!("n8n", "clone.workflow_validation.credentials.providers.key"),
    concat!("n8n", "clone.workflow_validation.diagnostics.core.empty_command"),
    concat!("n8n", "clone.workflow_validation.diagnostics.core.missing_config"),
    concat!("n8n", "clone.workflow_validation.diagnostics.core.valid"),
    concat!("n8n", "clone.workflow_validation.diagnostics.negative.invalid_edge"),
    concat!("n8n", "clone.workflow_validation.generic_runner_admission.effects"),
    concat!("n8n", "clone.workflow_validation.generic_runner_admission.graph"),
    concat!("n8n", "clone.workflow_validation.generic_runner_admission.terminal_state"),
    concat!("n8n", "clone.workflow_validation.n8n_json.export_limits.node"),
    concat!("n8n", "clone.workflow_validation.n8n_json.unsupported.connection"),
    concat!("n8n", "clone.workflow_validation.n8n_json.unsupported.expression"),
    concat!("n8n", "clone.workflow_validation.n8n_json.unsupported.node_kind"),
    concat!("n8n", "clone.workflow_validation.n8n_parity_diagnostics.malformed_connection"),
    concat!("n8n", "clone.workflow_validation.n8n_parity_diagnostics.plaintext_credential"),
    concat!("n8n", "clone.workflow_validation.n8n_parity_diagnostics.unsupported_expression"),
    concat!("n8n", "clone.workflow_validation.n8n_parity_diagnostics.unsupported_node"),
    concat!("n8n", "clone.workflow_validation.node_kind_provider"),
    concat!("n8n", "clone.workflow_validation.replay_records.artifact"),
    concat!("n8n", "clone.workflow_validation.replay_records.order"),
    concat!("n8n", "clone.workflow_validation.replay_records.terminal"),
    concat!("n8n", "clone.workflow_validation.run_gate.block"),
    concat!("n8n", "clone.workflow_validation.run_gate.warning"),
    concat!("n8n", "clone.workflow_validation.subworkflows.mapping"),
    concat!("n8n", "clone.workflow_validation.subworkflows.missing"),
    concat!("n8n", "clone.workflow_validation.subworkflows.recursive"),
    concat!("n8n", "clone.workflow_validation.tui.panel.empty"),
    concat!("n8n", "clone.workflow_validation.tui.panel.open"),
];

fn rationale(id: &str) -> &'static str {
    match id {
        x if x == IDS[0] => "assertions.missing-provider-fatal-pre-effect",
        x if x == IDS[1] => "assertions.unsupported-field-fatal-pre-effect",
        x if x == IDS[2] => "assertions.malformed-key-fatal-safe-context",
        x if x == IDS[3] => "assertions.empty-command-fatal-code",
        x if x == IDS[4] => "assertions.missing-config-fatal-code",
        x if x == IDS[5] => "assertions.valid-workflow-no-fatal",
        x if x == IDS[6] => "assertions.invalid-edge-stable-code",
        x if x == IDS[7] => "assertions.unsupported-effect-policy-fatal",
        x if x == IDS[8] => "assertions.all-bounded-graph-classes-pre-effect",
        x if x == IDS[9] => "assertions.terminal-transition-preserves-workflow",
        x if x == IDS[10] => "assertions.unsupported-export-refused",
        x if x == IDS[11] => "assertions.malformed-connection-refused",
        x if x == IDS[12] => "assertions.unsupported-expression-refused",
        x if x == IDS[13] => "assertions.unsupported-node-refused",
        x if x == IDS[14] => "assertions.parity-malformed-golden-diagnostic",
        x if x == IDS[15] => "assertions.parity-plaintext-safe-diagnostic",
        x if x == IDS[16] => "assertions.parity-expression-golden-diagnostic",
        x if x == IDS[17] => "assertions.parity-node-golden-diagnostic",
        x if x == IDS[18] => "assertions.all-builtins-provider-and-runtime-admission",
        x if x == IDS[19] => "assertions.missing-artifact-recoverable-placeholder",
        x if x == IDS[20] => "assertions.duplicate-out-of-order-fatal-codes",
        x if x == IDS[21] => "assertions.terminal-replay-mutation-rejected",
        x if x == IDS[22] => "assertions.fatal-blocks-tui-and-headless-effects",
        x if x == IDS[23] => "assertions.warning-allows-run-gate",
        x if x == IDS[24] => "assertions.unknown-mapping-fatal-preserves-config",
        x if x == IDS[25] => "assertions.missing-subworkflow-rejected",
        x if x == IDS[26] => "assertions.recursive-subworkflow-rejected",
        x if x == IDS[27] => "assertions.ready-message-rendered",
        x if x == IDS[28] => "assertions.severity-location-action-rendered",
        _ => panic!("unexpected requirement id: {id}"),
    }
}

fn paths(id: &str) -> &'static [&'static str] {
    match id {
        x if [
            IDS[0], IDS[1], IDS[2], IDS[8], IDS[9], IDS[18], IDS[24], IDS[27], IDS[28],
        ]
        .contains(&x) =>
        {
            &[TESTS_PATH]
        }
        x if [IDS[3], IDS[4], IDS[5], IDS[6], IDS[22], IDS[23]].contains(&x) => &[WORKFLOW_PATH],
        x if x == IDS[7] => &[RUNNER_PATH, VALIDATION_PATH, TESTS_PATH],
        x if [IDS[10], IDS[11], IDS[12], IDS[13], IDS[14], IDS[15], IDS[16], IDS[17]].contains(&x) => &[N8N_PATH],
        x if [IDS[19], IDS[20], IDS[21]].contains(&x) => &[REPLAY_PATH],
        x if [IDS[25], IDS[26]].contains(&x) => &[CONTROL_PATH],
        _ => panic!("unexpected requirement id: {id}"),
    }
}

fn expected() -> Vec<ExpectedDecision> {
    IDS.iter()
        .map(|id| ExpectedDecision {
            requirement_id: id,
            classification: EXACT,
            rationale_code: rationale(id),
            evidence_paths: paths(id),
        })
        .collect()
}

#[derive(Debug, serde::Deserialize)]
struct InventoryReport {
    rows: Vec<InventoryRow>,
}
#[derive(Debug, serde::Deserialize)]
struct InventoryRow {
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
struct Coverage {
    missing: Vec<String>,
    dangling: Vec<String>,
    valid: bool,
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
    let value: Coverage = serde_json::from_str(source).expect("coverage decodes");
    CoverageSnapshot {
        missing: value.missing,
        dangling: value.dangling,
        valid: value.valid,
    }
}
fn source(path: &str) -> &'static str {
    match path {
        WORKFLOW_PATH => WORKFLOW,
        N8N_PATH => N8N,
        REPLAY_PATH => REPLAY,
        CONTROL_PATH => CONTROL,
        RUNNER_PATH => RUNNER,
        VALIDATION_PATH => VALIDATION,
        TESTS_PATH => TESTS,
        _ => panic!("unexpected evidence path: {path}"),
    }
}
fn codes(issues: &[core::ReviewIssue]) -> BTreeSet<&str> {
    issues.iter().map(|issue| issue.code).collect()
}

// r[verify lattice.repository_validation.workflow_validation_completion]
// r[verify lattice.repository_validation.workflow_validation_completion.scenario.behavior]
// r[verify lattice.repository_validation.workflow_validation_completion.scenario.evidence]
// r[verify lattice.repository_validation.workflow_validation_completion.scenario.classification]
#[test]
fn workflow_validation_completion_is_exact_byte_bound_and_unpromoted() {
    let report: InventoryReport = serde_json::from_str(REVIEWED).expect("inventory decodes");
    let expected = expected();
    let review = core::validate_tranche(&rows(&report), SUBSYSTEM, &expected);
    let unchanged = core::validate_unchanged_coverage(&coverage(BASELINE), &coverage(CURRENT));
    let markers = core::validate_no_selected_markers(&[WORKFLOW, N8N, REPLAY, CONTROL, RUNNER, VALIDATION, TESTS], IDS);
    assert!(review.is_empty(), "review issues: {review:?}");
    assert!(unchanged.is_empty(), "coverage issues: {unchanged:?}");
    assert!(markers.is_empty(), "marker issues: {markers:?}");
    assert_eq!(expected.len(), SELECTED_COUNT);
    for row in report.rows.iter().filter(|row| IDS.contains(&row.requirement_id.as_str())) {
        for evidence in &row.evidence {
            assert_eq!(blake3::hash(source(&evidence.path).as_bytes()).to_hex().to_string(), evidence.blake3);
        }
    }
}

// Negative coverage intentionally corrupts classification, evidence, marker, and coverage state.
#[test]
fn workflow_validation_completion_drift_fails_closed() {
    let report: InventoryReport = serde_json::from_str(REVIEWED).expect("inventory decodes");
    let expected = expected();
    let mut review_rows = rows(&report);
    review_rows.retain(|row| row.requirement_id != IDS[0]);
    let row = review_rows.iter_mut().find(|row| row.requirement_id == IDS[1]).expect("review row");
    row.classification = "partial_evidence_only".to_string();
    row.rationale_code = "invalid.rationale".to_string();
    row.evidence_paths = vec!["../unsafe".to_string()];
    let review_issues = core::validate_tranche(&review_rows, SUBSYSTEM, &expected);
    let review_codes = codes(&review_issues);
    for code in [
        "review.missing",
        "review.classification",
        "review.rationale",
        "review.evidence_paths",
    ] {
        assert!(review_codes.contains(code));
    }

    let marker = format!("// r[verify {}]", IDS[0]);
    let marker_issues = core::validate_no_selected_markers(&[marker.as_str()], &[IDS[0], IDS[0]]);
    let marker_codes = codes(&marker_issues);
    assert!(marker_codes.contains("marker.promoted"));
    assert!(marker_codes.contains("marker.duplicate_selected"));

    let baseline = coverage(BASELINE);
    let mut current = coverage(CURRENT);
    current.missing.retain(|id| id != UNRELATED);
    current.missing.push(ADDED.to_string());
    current.missing.push(ADDED.to_string());
    current.dangling.push(DANGLING.to_string());
    current.valid = true;
    let coverage_issues = core::validate_unchanged_coverage(&baseline, &current);
    let coverage_codes = codes(&coverage_issues);
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

#[allow(dead_code)]
#[path = "traceability_review/core.rs"]
mod core;

use core::CoverageSnapshot;
use std::collections::BTreeSet;

const BASELINE: &str = include_str!("fixtures/traceability-octet-baseline-migration-current.json");
const CURRENT: &str = include_str!("fixtures/traceability-workflow-execution-audit-current.json");
const REVIEWED: &str = include_str!("fixtures/traceability-workflow-execution-audit-reviewed.json");
const AGENT: &str = include_str!("fixtures/traceability-workflow-execution-audit-reviewed-test_agent_nodes.rs.txt");
const OPENAI: &str = include_str!("fixtures/traceability-workflow-execution-audit-reviewed-test_openai_agent.rs.txt");
const OPENAI_EXEC: &str = include_str!("fixtures/traceability-workflow-execution-audit-reviewed-openai-execution.txt");
const REPOSITORY: &str =
    include_str!("fixtures/traceability-workflow-execution-audit-reviewed-test_repository_validation.rs.txt");
const RUNNER: &str = include_str!("fixtures/traceability-workflow-execution-audit-reviewed-test_runner_core.rs.txt");
const EXPRESSION: &str =
    include_str!("fixtures/traceability-workflow-execution-audit-reviewed-test_expression_mapping.rs.txt");
const FILE: &str = include_str!("fixtures/traceability-workflow-execution-audit-reviewed-test_file_nodes.rs.txt");
const HEADLESS: &str = include_str!("fixtures/traceability-workflow-execution-audit-reviewed-test_headless.rs.txt");
const HTTP: &str = include_str!("fixtures/traceability-workflow-execution-audit-reviewed-test_http_node.rs.txt");
const IMAGE: &str = include_str!("fixtures/traceability-workflow-execution-audit-reviewed-test_image_nodes.rs.txt");
const VALUE: &str = include_str!("fixtures/traceability-workflow-execution-audit-reviewed-test_value.rs.txt");
const LLM: &str =
    include_str!("fixtures/traceability-workflow-execution-audit-reviewed-test_llm_operating_graph.rs.txt");
const EXECUTOR: &str = include_str!("fixtures/traceability-workflow-execution-audit-reviewed-test_executor.rs.txt");
const REPLAY: &str = include_str!("fixtures/traceability-workflow-execution-audit-reviewed-test_replay.rs.txt");
const CONTROL: &str = include_str!("fixtures/traceability-workflow-execution-audit-reviewed-test_control_flow.rs.txt");
const RATS: &str = include_str!("fixtures/traceability-workflow-execution-audit-reviewed-rats-source.txt");
const WORKFLOW_PREFIX: &str = concat!("n8n", "clone.workflow_execution.");
const SUBSYSTEM: &str = "workflow-execution";
const EXACT: &str = "exact_maintained_evidence";
const PARTIAL: &str = "partial_evidence_only";
const SELECTED_COUNT: usize = 83;
const PARTIAL_IDS: &[&str] = &[
    concat!("n8n", "clone.workflow_execution.rats_dependency"),
    concat!("n8n", "clone.workflow_execution.rats_dependency.scenario.source-completeness"),
    concat!("n8n", "clone.workflow_execution.subworkflows.events.correlation"),
    concat!("n8n", "clone.workflow_execution.subworkflows.invocation.outputs"),
];
const UNRELATED: &str = concat!("n8n", "clone.tui_ux.add_node.cancel");
const ADDED: &str = "lattice.example.unaccepted.scenario.added";
const DANGLING: &str = "lattice.example.dangling.scenario.marker";

#[derive(Debug, serde::Deserialize)]
struct InventoryReport {
    rows: Vec<InventoryRow>,
}
#[derive(Debug, Clone, serde::Deserialize)]
struct InventoryRow {
    requirement_id: String,
    subsystem: String,
    classification: String,
    evidence: Vec<Evidence>,
    rationale_code: String,
}
#[derive(Debug, Clone, serde::Deserialize)]
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
        "tests/test_agent_nodes.rs" => AGENT,
        "tests/test_openai_agent.rs" => OPENAI,
        "tests/openai/execution.rs" => OPENAI_EXEC,
        "tests/test_repository_validation.rs" => REPOSITORY,
        "tests/test_runner_core.rs" => RUNNER,
        "tests/test_expression_mapping.rs" => EXPRESSION,
        "tests/test_file_nodes.rs" => FILE,
        "tests/test_headless.rs" => HEADLESS,
        "tests/test_http_node.rs" => HTTP,
        "tests/test_image_nodes.rs" => IMAGE,
        "tests/test_value.rs" => VALUE,
        "tests/test_llm_operating_graph.rs" => LLM,
        "tests/test_executor.rs" => EXECUTOR,
        "tests/test_replay.rs" => REPLAY,
        "tests/test_control_flow.rs" => CONTROL,
        "vendor-cargo-deps/rats/source/SOURCE" => RATS,
        _ => panic!("unexpected path: {path}"),
    }
}

fn audit_issues(rows: &[InventoryRow], current: &CoverageSnapshot) -> BTreeSet<&'static str> {
    let mut issues = BTreeSet::new();
    let selected = rows.iter().filter(|row| row.subsystem == SUBSYSTEM).collect::<Vec<_>>();
    let observed = selected.iter().map(|row| row.requirement_id.as_str()).collect::<BTreeSet<_>>();
    let expected = current
        .missing
        .iter()
        .filter(|id| id.starts_with(WORKFLOW_PREFIX))
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    if observed != expected {
        issues.insert("review.identities");
    }
    if selected.len() != SELECTED_COUNT {
        issues.insert("review.count");
    }
    for row in selected {
        let partial = PARTIAL_IDS.contains(&row.requirement_id.as_str());
        let expected_class = if partial { PARTIAL } else { EXACT };
        if row.classification != expected_class {
            issues.insert("review.classification");
        }
        if row.evidence.is_empty() {
            issues.insert("review.evidence");
        }
        if partial && !row.rationale_code.starts_with("gap.") {
            issues.insert("review.rationale");
        }
        if !partial && !row.rationale_code.starts_with("assertions.") {
            issues.insert("review.rationale");
        }
        for evidence in &row.evidence {
            if blake3::hash(source(&evidence.path).as_bytes()).to_hex().to_string() != evidence.blake3 {
                issues.insert("review.evidence_hash");
            }
        }
    }
    issues
}

fn codes(issues: &[core::ReviewIssue]) -> BTreeSet<&str> {
    issues.iter().map(|issue| issue.code).collect()
}

// r[verify lattice.repository_validation.workflow_execution_audit]
// r[verify lattice.repository_validation.workflow_execution_audit.scenario.complete]
// r[verify lattice.repository_validation.workflow_execution_audit.scenario.evidence]
#[test]
fn workflow_execution_audit_is_complete_byte_bound_and_unpromoted() {
    let report: InventoryReport = serde_json::from_str(REVIEWED).expect("inventory decodes");
    let baseline = coverage(BASELINE);
    let current = coverage(CURRENT);
    let ids = report.rows.iter().map(|row| row.requirement_id.as_str()).collect::<Vec<_>>();
    let review = audit_issues(&report.rows, &current);
    let unchanged = core::validate_unchanged_coverage(&baseline, &current);
    let markers = core::validate_no_selected_markers(
        &[
            AGENT,
            OPENAI,
            OPENAI_EXEC,
            REPOSITORY,
            RUNNER,
            EXPRESSION,
            FILE,
            HEADLESS,
            HTTP,
            IMAGE,
            VALUE,
            LLM,
            EXECUTOR,
            REPLAY,
            CONTROL,
            RATS,
        ],
        &ids,
    );
    assert!(review.is_empty(), "review issues: {review:?}");
    assert!(unchanged.is_empty(), "coverage issues: {unchanged:?}");
    assert!(markers.is_empty(), "marker issues: {markers:?}");
}

// r[verify lattice.repository_validation.workflow_execution_audit.scenario.fail_closed]
#[test]
fn workflow_execution_audit_drift_fails_closed() {
    let mut report: InventoryReport = serde_json::from_str(REVIEWED).expect("inventory decodes");
    report.rows.remove(0);
    let row = report.rows.first_mut().expect("selected row");
    row.classification = PARTIAL.to_string();
    row.rationale_code = "invalid.rationale".to_string();
    row.evidence.clear();
    let issues = audit_issues(&report.rows, &coverage(CURRENT));
    for code in [
        "review.identities",
        "review.count",
        "review.classification",
        "review.evidence",
        "review.rationale",
    ] {
        assert!(issues.contains(code));
    }

    let marker = format!("// r[verify {}]", PARTIAL_IDS[0]);
    let marker_issues = core::validate_no_selected_markers(&[marker.as_str()], &[PARTIAL_IDS[0], PARTIAL_IDS[0]]);
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

#[allow(dead_code)]
#[path = "traceability_review/core.rs"]
mod core;

use core::CoverageSnapshot;
use std::collections::BTreeSet;

const BASELINE: &str = include_str!("fixtures/traceability-workflow-execution-audit-current.json");
const CURRENT: &str = include_str!("fixtures/traceability-workflow-execution-completion-current.json");
const AUDIT: &str = include_str!("fixtures/traceability-workflow-execution-audit-reviewed.json");
const COMPLETION: &str = include_str!("fixtures/traceability-workflow-execution-completion-reviewed.json");
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
const RATS_METADATA: &str = include_str!("fixtures/traceability-workflow-execution-audit-reviewed-rats-source.txt");
const RATS_COMPLETION: &str =
    include_str!("fixtures/traceability-workflow-execution-completion-reviewed-rats-source-closure.rs.txt");
const SUBWORKFLOW_COMPLETION: &str =
    include_str!("fixtures/traceability-workflow-execution-completion-reviewed-subworkflow-execution.rs.txt");
const SUBSYSTEM: &str = "workflow-execution";
const EXACT: &str = "exact_maintained_evidence";
const SELECTED_COUNT: usize = 83;
const UNRELATED: &str = concat!("n8n", "clone.tui_ux.add_node.cancel");
const ADDED: &str = "lattice.example.unaccepted.scenario.added";
const DANGLING: &str = "lattice.example.dangling.scenario.marker";

#[derive(Debug, Clone, serde::Deserialize)]
struct InventoryReport {
    rows: Vec<InventoryRow>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct InventoryRow {
    requirement_id: String,
    subsystem: String,
    classification: String,
    evidence: Vec<Evidence>,
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

fn selected(report: &InventoryReport) -> BTreeSet<&str> {
    report
        .rows
        .iter()
        .filter(|row| row.subsystem == SUBSYSTEM)
        .map(|row| row.requirement_id.as_str())
        .collect()
}

fn completion_issues(audit: &InventoryReport, completion: &InventoryReport) -> BTreeSet<&'static str> {
    let mut issues = BTreeSet::new();
    if selected(audit) != selected(completion) {
        issues.insert("completion.identities");
    }
    let rows = completion.rows.iter().filter(|row| row.subsystem == SUBSYSTEM).collect::<Vec<_>>();
    if rows.len() != SELECTED_COUNT {
        issues.insert("completion.count");
    }
    for row in rows {
        if row.classification != EXACT {
            issues.insert("completion.classification");
        }
        if row.evidence.is_empty() {
            issues.insert("completion.evidence");
        }
    }
    issues
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
        "vendor-cargo-deps/rats/source/SOURCE" => RATS_METADATA,
        "tests/test_rats_source_closure.rs" => RATS_COMPLETION,
        "tests/test_subworkflow_execution.rs" => SUBWORKFLOW_COMPLETION,
        _ => panic!("unexpected path: {path}"),
    }
}

fn codes(issues: &[core::ReviewIssue]) -> BTreeSet<&str> {
    issues.iter().map(|issue| issue.code).collect()
}

// r[verify lattice.repository_validation.workflow_execution_completion]
// r[verify lattice.repository_validation.workflow_execution_completion.scenario.rats]
// r[verify lattice.repository_validation.workflow_execution_completion.scenario.exact]
#[test]
fn workflow_execution_completion_is_exact_byte_bound_and_unpromoted() {
    let audit: InventoryReport = serde_json::from_str(AUDIT).expect("audit inventory decodes");
    let completion: InventoryReport = serde_json::from_str(COMPLETION).expect("completion inventory decodes");
    let ids = selected(&audit).into_iter().collect::<Vec<_>>();
    let issues = completion_issues(&audit, &completion);
    let unchanged = core::validate_unchanged_coverage(&coverage(BASELINE), &coverage(CURRENT));
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
            RATS_METADATA,
            RATS_COMPLETION,
            SUBWORKFLOW_COMPLETION,
        ],
        &ids,
    );
    assert!(issues.is_empty(), "completion issues: {issues:?}");
    assert!(unchanged.is_empty(), "coverage issues: {unchanged:?}");
    assert!(markers.is_empty(), "marker issues: {markers:?}");
    for row in completion.rows.iter().filter(|row| row.subsystem == SUBSYSTEM) {
        for evidence in &row.evidence {
            assert_eq!(blake3::hash(source(&evidence.path).as_bytes()).to_hex().to_string(), evidence.blake3);
        }
    }
}

// r[verify lattice.repository_validation.workflow_execution_completion.scenario.fail_closed]
#[test]
fn workflow_execution_completion_drift_fails_closed() {
    let audit: InventoryReport = serde_json::from_str(AUDIT).expect("audit inventory decodes");
    let mut completion: InventoryReport = serde_json::from_str(COMPLETION).expect("completion inventory decodes");
    let first_id = selected(&audit).into_iter().next().expect("selected id").to_string();
    completion.rows.retain(|row| row.requirement_id != first_id);
    let row = completion.rows.iter_mut().find(|row| row.subsystem == SUBSYSTEM).expect("completion row");
    row.classification = "partial_evidence_only".to_string();
    row.evidence.clear();
    let issues = completion_issues(&audit, &completion);
    for code in [
        "completion.identities",
        "completion.count",
        "completion.classification",
        "completion.evidence",
    ] {
        assert!(issues.contains(code));
    }

    let marker = format!("// r[verify {first_id}]");
    let marker_issues = core::validate_no_selected_markers(&[marker.as_str()], &[&first_id, &first_id]);
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

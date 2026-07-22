#[allow(dead_code)]
#[path = "traceability_review/core.rs"]
mod core;

use core::CoverageSnapshot;
use std::collections::BTreeSet;

const BASELINE: &str = include_str!("fixtures/traceability-agent-orchestration-audit-current.json");
const CURRENT: &str = include_str!("fixtures/traceability-agent-orchestration-audit-current.json");
const AUDIT: &str = include_str!("fixtures/traceability-agent-orchestration-audit-reviewed.json");
const COMPLETION: &str = include_str!("fixtures/traceability-agent-orchestration-completion-reviewed.json");
const CORE: &str = include_str!("fixtures/traceability-agent-orchestration-completion-reviewed-core.txt");
const CORE_SOURCE: &str = include_str!("fixtures/traceability-agent-orchestration-completion-reviewed-core-source.txt");
const STORE: &str = include_str!("fixtures/traceability-agent-orchestration-audit-reviewed-store.txt");
const FORMAT: &str = include_str!("fixtures/traceability-agent-orchestration-audit-reviewed-format.txt");
const SHELL: &str = include_str!("fixtures/traceability-agent-orchestration-audit-reviewed-shell.txt");
const CLI: &str = include_str!("fixtures/traceability-agent-orchestration-audit-reviewed-cli.txt");
const RUN_STORE: &str = include_str!("fixtures/traceability-agent-orchestration-audit-reviewed-run-store.txt");
const CORE_PATH: &str = "tests/test_orchestration_core.rs";
const CORE_SOURCE_PATH: &str = "src/orchestration/core.rs";
const STORE_PATH: &str = "tests/test_orchestration_store.rs";
const FORMAT_PATH: &str = "tests/test_orchestration_format.rs";
const SHELL_PATH: &str = "tests/test_orchestration_shell.rs";
const CLI_PATH: &str = "tests/test_orchestration_cli.rs";
const RUN_STORE_PATH: &str = "tests/test_store.rs";
const SUBSYSTEM: &str = "agent-orchestration";
const EXACT: &str = "exact_maintained_evidence";
const SELECTED_COUNT: usize = 39;
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
        CORE_PATH => CORE,
        CORE_SOURCE_PATH => CORE_SOURCE,
        STORE_PATH => STORE,
        FORMAT_PATH => FORMAT,
        SHELL_PATH => SHELL,
        CLI_PATH => CLI,
        RUN_STORE_PATH => RUN_STORE,
        _ => panic!("unexpected path: {path}"),
    }
}
fn codes(issues: &[core::ReviewIssue]) -> BTreeSet<&str> {
    issues.iter().map(|issue| issue.code).collect()
}

// r[verify lattice.repository_validation.agent_orchestration_completion]
// r[verify lattice.repository_validation.agent_orchestration_completion.scenario.behavior]
// r[verify lattice.repository_validation.agent_orchestration_completion.scenario.exact]
#[test]
fn agent_orchestration_completion_is_exact_byte_bound_and_unpromoted() {
    let audit: InventoryReport = serde_json::from_str(AUDIT).expect("audit inventory decodes");
    let completion: InventoryReport = serde_json::from_str(COMPLETION).expect("completion inventory decodes");
    let ids = selected(&audit).into_iter().collect::<Vec<_>>();
    let issues = completion_issues(&audit, &completion);
    let unchanged = core::validate_unchanged_coverage(&coverage(BASELINE), &coverage(CURRENT));
    let markers = core::validate_no_selected_markers(&[CORE, CORE_SOURCE, STORE, FORMAT, SHELL, CLI, RUN_STORE], &ids);
    assert!(issues.is_empty(), "completion issues: {issues:?}");
    assert!(unchanged.is_empty(), "coverage issues: {unchanged:?}");
    assert!(markers.is_empty(), "marker issues: {markers:?}");
    for row in completion.rows.iter().filter(|row| row.subsystem == SUBSYSTEM) {
        for evidence in &row.evidence {
            assert_eq!(blake3::hash(source(&evidence.path).as_bytes()).to_hex().to_string(), evidence.blake3);
        }
    }
}

// r[verify lattice.repository_validation.agent_orchestration_completion.scenario.fail_closed]
#[test]
fn agent_orchestration_completion_drift_fails_closed() {
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

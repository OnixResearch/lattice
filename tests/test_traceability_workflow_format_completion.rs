#[allow(dead_code)]
#[path = "traceability_review/core.rs"]
mod core;

use core::CoverageSnapshot;
use std::collections::BTreeSet;

const BASELINE: &str = include_str!("fixtures/traceability-workflow-format-audit-current.json");
const CURRENT: &str = include_str!("fixtures/traceability-workflow-format-completion-current.json");
const AUDIT: &str = include_str!("fixtures/traceability-workflow-format-audit-reviewed.json");
const COMPLETION: &str = include_str!("fixtures/traceability-workflow-format-completion-reviewed.json");
const WF: &str = include_str!("fixtures/traceability-workflow-format-completion-reviewed-workflow-format.txt");
const IMAGE: &str = include_str!("fixtures/traceability-workflow-format-completion-reviewed-image-example.txt");
const CONTROL: &str = include_str!("fixtures/traceability-workflow-format-completion-reviewed-control-flow.txt");
const CREDENTIALS: &str = include_str!("fixtures/traceability-workflow-format-completion-reviewed-credentials.txt");
const N8N: &str = include_str!("fixtures/traceability-workflow-format-completion-reviewed-n8n.txt");
const REPLAY: &str = include_str!("fixtures/traceability-workflow-format-completion-reviewed-replay.txt");
const TRIGGERS: &str = include_str!("fixtures/traceability-workflow-format-completion-reviewed-triggers.txt");
const NODE_KIND: &str = include_str!("fixtures/traceability-workflow-format-completion-reviewed-node-kind.txt");
const WEBHOOKS: &str = include_str!("fixtures/traceability-workflow-format-completion-reviewed-webhooks.txt");
const WF_PATH: &str = "tests/test_workflow_format.rs";
const IMAGE_PATH: &str = "tests/test_image_workflow_example.rs";
const CONTROL_PATH: &str = "tests/test_control_flow.rs";
const CREDENTIALS_PATH: &str = "tests/test_credentials.rs";
const N8N_PATH: &str = "tests/test_n8n_json.rs";
const REPLAY_PATH: &str = "tests/test_replay.rs";
const TRIGGERS_PATH: &str = "tests/test_triggers.rs";
const NODE_KIND_PATH: &str = "tests/test_node_kind_provider.rs";
const WEBHOOKS_PATH: &str = "tests/webhooks.rs";
const SUBSYSTEM: &str = "workflow-format";
const EXACT: &str = "exact_maintained_evidence";
const SELECTED_COUNT: usize = 41;
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
        WF_PATH => WF,
        IMAGE_PATH => IMAGE,
        CONTROL_PATH => CONTROL,
        CREDENTIALS_PATH => CREDENTIALS,
        N8N_PATH => N8N,
        REPLAY_PATH => REPLAY,
        TRIGGERS_PATH => TRIGGERS,
        NODE_KIND_PATH => NODE_KIND,
        WEBHOOKS_PATH => WEBHOOKS,
        _ => panic!("unexpected path: {path}"),
    }
}
fn codes(issues: &[core::ReviewIssue]) -> BTreeSet<&str> {
    issues.iter().map(|issue| issue.code).collect()
}

// r[verify lattice.repository_validation.workflow_format_completion]
// r[verify lattice.repository_validation.workflow_format_completion.scenario.behavior]
// r[verify lattice.repository_validation.workflow_format_completion.scenario.exact]
#[test]
fn workflow_format_completion_is_exact_byte_bound_and_unpromoted() {
    let audit: InventoryReport = serde_json::from_str(AUDIT).expect("audit inventory decodes");
    let completion: InventoryReport = serde_json::from_str(COMPLETION).expect("completion inventory decodes");
    let ids = selected(&audit).into_iter().collect::<Vec<_>>();
    let issues = completion_issues(&audit, &completion);
    let unchanged = core::validate_unchanged_coverage(&coverage(BASELINE), &coverage(CURRENT));
    let markers = core::validate_no_selected_markers(
        &[
            WF,
            IMAGE,
            CONTROL,
            CREDENTIALS,
            N8N,
            REPLAY,
            TRIGGERS,
            NODE_KIND,
            WEBHOOKS,
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

// r[verify lattice.repository_validation.workflow_format_completion.scenario.fail_closed]
#[test]
fn workflow_format_completion_drift_fails_closed() {
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

#[allow(dead_code)]
#[path = "traceability_review/core.rs"]
mod core;

use core::CoverageSnapshot;
use std::collections::BTreeSet;

const BASELINE: &str = include_str!("fixtures/traceability-agent-orchestration-promotion-current.json");
const CURRENT: &str = include_str!("fixtures/traceability-workflow-format-audit-current.json");
const REVIEWED: &str = include_str!("fixtures/traceability-workflow-format-audit-reviewed.json");
const WF: &str = include_str!("fixtures/traceability-workflow-format-audit-reviewed-workflow-format.txt");
const IMAGE: &str = include_str!("fixtures/traceability-workflow-format-audit-reviewed-image-example.txt");
const CONTROL: &str = include_str!("fixtures/traceability-workflow-format-audit-reviewed-control-flow.txt");
const CREDENTIALS: &str = include_str!("fixtures/traceability-workflow-format-audit-reviewed-credentials.txt");
const N8N: &str = include_str!("fixtures/traceability-workflow-format-audit-reviewed-n8n.txt");
const REPLAY: &str = include_str!("fixtures/traceability-workflow-format-audit-reviewed-replay.txt");
const TRIGGERS: &str = include_str!("fixtures/traceability-workflow-format-audit-reviewed-triggers.txt");
const NODE_KIND: &str = include_str!("fixtures/traceability-workflow-format-audit-reviewed-node-kind.txt");
const WEBHOOKS: &str = include_str!("fixtures/traceability-workflow-format-audit-reviewed-webhooks.txt");
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
const PARTIAL: &str = "partial_evidence_only";
const SELECTED_COUNT: usize = 41;
const UNRELATED: &str = concat!("n8n", "clone.tui_ux.add_node.cancel");
const ADDED: &str = "lattice.example.unaccepted.scenario.added";
const DANGLING: &str = "lattice.example.dangling.scenario.marker";

#[path = "traceability_workflow_format_ids.rs"]
mod workflow_format_ids;
use workflow_format_ids::IDS;
const PARTIAL_IDS: &[&str] = &[IDS[0], IDS[1], IDS[10], IDS[12], IDS[40]];

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

fn expected_paths(id: &str) -> &'static [&'static str] {
    match id {
        x if x == IDS[0] => &[WF_PATH, IMAGE_PATH],
        x if x == IDS[1] => &[IMAGE_PATH],
        x if (2..=7).contains(&IDS.iter().position(|candidate| *candidate == x).unwrap_or(usize::MAX)) => &[WF_PATH],
        x if [IDS[8], IDS[9], IDS[10]].contains(&x) => &[CONTROL_PATH],
        x if [IDS[11], IDS[12], IDS[13]].contains(&x) => &[CREDENTIALS_PATH],
        x if [IDS[14], IDS[15], IDS[16]].contains(&x) => &[WF_PATH],
        x if (17..=25).contains(&IDS.iter().position(|candidate| *candidate == x).unwrap_or(usize::MAX)) => &[N8N_PATH],
        x if (26..=31).contains(&IDS.iter().position(|candidate| *candidate == x).unwrap_or(usize::MAX)) => &[WF_PATH],
        x if x == IDS[32] => &[NODE_KIND_PATH, WF_PATH],
        x if [IDS[33], IDS[34], IDS[35]].contains(&x) => &[REPLAY_PATH],
        x if [IDS[36], IDS[37], IDS[38]].contains(&x) => &[TRIGGERS_PATH],
        x if [IDS[39], IDS[40]].contains(&x) => &[WEBHOOKS_PATH],
        _ => panic!("unexpected requirement: {id}"),
    }
}

fn partial_rationale(id: &str) -> &'static str {
    match id {
        x if [IDS[0], IDS[1]].contains(&x) => "gap.image-format-import-admission",
        x if x == IDS[10] => "gap.malformed-control-flow-import-unasserted",
        x if x == IDS[12] => "gap.reference-import-round-trip-unasserted",
        x if x == IDS[40] => "gap.cookbook-drift-negative-unasserted",
        _ => panic!("unexpected partial requirement: {id}"),
    }
}

fn audit_issues(rows: &[InventoryRow]) -> BTreeSet<&'static str> {
    let mut issues = BTreeSet::new();
    let selected = rows.iter().filter(|row| row.subsystem == SUBSYSTEM).collect::<Vec<_>>();
    let observed = selected.iter().map(|row| row.requirement_id.as_str()).collect::<BTreeSet<_>>();
    let expected = IDS.iter().copied().collect::<BTreeSet<_>>();
    if observed != expected {
        issues.insert("review.identities");
    }
    if selected.len() != SELECTED_COUNT {
        issues.insert("review.count");
    }
    for row in selected {
        let id = row.requirement_id.as_str();
        let expected_class = if PARTIAL_IDS.contains(&id) { PARTIAL } else { EXACT };
        if row.classification != expected_class {
            issues.insert("review.classification");
        }
        if PARTIAL_IDS.contains(&id) {
            if row.rationale_code != partial_rationale(id) {
                issues.insert("review.rationale");
            }
        } else if !row.rationale_code.starts_with("assertions.") {
            issues.insert("review.rationale");
        }
        let mut actual_paths = row.evidence.iter().map(|evidence| evidence.path.as_str()).collect::<Vec<_>>();
        actual_paths.sort_unstable();
        let mut expected = expected_paths(id).to_vec();
        expected.sort_unstable();
        if actual_paths != expected {
            issues.insert("review.evidence_paths");
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

// r[verify lattice.repository_validation.workflow_format_audit]
// r[verify lattice.repository_validation.workflow_format_audit.scenario.complete]
// r[verify lattice.repository_validation.workflow_format_audit.scenario.evidence]
#[test]
fn workflow_format_audit_is_complete_byte_bound_and_unpromoted() {
    let report: InventoryReport = serde_json::from_str(REVIEWED).expect("inventory decodes");
    let review = audit_issues(&report.rows);
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
        IDS,
    );
    assert!(review.is_empty(), "review issues: {review:?}");
    assert!(unchanged.is_empty(), "coverage issues: {unchanged:?}");
    assert!(markers.is_empty(), "marker issues: {markers:?}");
    for row in report.rows.iter().filter(|row| IDS.contains(&row.requirement_id.as_str())) {
        for evidence in &row.evidence {
            assert_eq!(blake3::hash(source(&evidence.path).as_bytes()).to_hex().to_string(), evidence.blake3);
        }
    }
}

// r[verify lattice.repository_validation.workflow_format_audit.scenario.fail_closed]
#[test]
fn workflow_format_audit_drift_fails_closed() {
    let report: InventoryReport = serde_json::from_str(REVIEWED).expect("inventory decodes");
    let mut rows = report.rows;
    rows.retain(|row| row.requirement_id != IDS[0]);
    let row = rows.iter_mut().find(|row| row.requirement_id == IDS[2]).expect("selected row");
    row.classification = PARTIAL.to_string();
    row.rationale_code = "invalid.rationale".to_string();
    row.evidence = Vec::new();
    let issues = audit_issues(&rows);
    for code in [
        "review.identities",
        "review.count",
        "review.classification",
        "review.rationale",
        "review.evidence_paths",
    ] {
        assert!(issues.contains(code));
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

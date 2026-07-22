#[path = "traceability_review/core.rs"]
mod core;

use core::CoverageSnapshot;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

const BASELINE: &str = include_str!("fixtures/traceability-workflow-validation-promotion-current.json");
const CURRENT: &str = include_str!("fixtures/traceability-agent-orchestration-audit-current.json");
const REVIEWED: &str = include_str!("fixtures/traceability-agent-orchestration-audit-reviewed.json");
const CORE: &str = include_str!("fixtures/traceability-agent-orchestration-audit-reviewed-core.txt");
const STORE: &str = include_str!("fixtures/traceability-agent-orchestration-audit-reviewed-store.txt");
const FORMAT: &str = include_str!("fixtures/traceability-agent-orchestration-audit-reviewed-format.txt");
const SHELL: &str = include_str!("fixtures/traceability-agent-orchestration-audit-reviewed-shell.txt");
const CLI: &str = include_str!("fixtures/traceability-agent-orchestration-audit-reviewed-cli.txt");
const RUN_STORE: &str = include_str!("fixtures/traceability-agent-orchestration-audit-reviewed-run-store.txt");
const CORE_PATH: &str = "tests/test_orchestration_core.rs";
const STORE_PATH: &str = "tests/test_orchestration_store.rs";
const FORMAT_PATH: &str = "tests/test_orchestration_format.rs";
const SHELL_PATH: &str = "tests/test_orchestration_shell.rs";
const CLI_PATH: &str = "tests/test_orchestration_cli.rs";
const RUN_STORE_PATH: &str = "tests/test_store.rs";
const SUBSYSTEM: &str = "agent-orchestration";
const EXACT: &str = "exact_maintained_evidence";
const PARTIAL: &str = "partial_evidence_only";
const SELECTED_COUNT: usize = 39;
const UNRELATED: &str = concat!("n8n", "clone.tui_ux.add_node.cancel");
const ADDED: &str = "lattice.example.unaccepted.scenario.added";
const DANGLING: &str = "lattice.example.dangling.scenario.marker";

const IDS: &[&str] = &[
    "lattice.agent_orchestration.authority.scenario.ambient-denied",
    "lattice.agent_orchestration.authority.scenario.approved",
    "lattice.agent_orchestration.authority.scenario.stale",
    "lattice.agent_orchestration.authority.scenario.validation",
    "lattice.agent_orchestration.budgets.scenario.block",
    "lattice.agent_orchestration.budgets.scenario.concurrent",
    "lattice.agent_orchestration.budgets.scenario.overage",
    "lattice.agent_orchestration.budgets.scenario.reserve",
    "lattice.agent_orchestration.cli.scenario.invalid",
    "lattice.agent_orchestration.cli.scenario.status",
    "lattice.agent_orchestration.controls.scenario.cancel",
    "lattice.agent_orchestration.controls.scenario.pause",
    "lattice.agent_orchestration.core.scenario.complete",
    "lattice.agent_orchestration.core.scenario.terminal-mutation",
    "lattice.agent_orchestration.durability.scenario.duplicate",
    "lattice.agent_orchestration.durability.scenario.external-effects",
    "lattice.agent_orchestration.durability.scenario.reconcile",
    "lattice.agent_orchestration.events.scenario.non-claim",
    "lattice.agent_orchestration.events.scenario.replay",
    "lattice.agent_orchestration.events.scenario.tampered",
    "lattice.agent_orchestration.format.scenario.n8n-boundary",
    "lattice.agent_orchestration.format.scenario.round-trip",
    "lattice.agent_orchestration.format.scenario.secret",
    "lattice.agent_orchestration.model.scenario.revision-drift",
    "lattice.agent_orchestration.model.scenario.valid",
    "lattice.agent_orchestration.rounds.scenario.execute",
    "lattice.agent_orchestration.rounds.scenario.hidden-cycle",
    "lattice.agent_orchestration.run_identity.scenario.concurrent",
    "lattice.agent_orchestration.run_identity.scenario.duplicate",
    "lattice.agent_orchestration.run_identity.scenario.foreign",
    "lattice.agent_orchestration.run_identity.scenario.match",
    "lattice.agent_orchestration.triggers.scenario.duplicate",
    "lattice.agent_orchestration.triggers.scenario.start",
    "lattice.agent_orchestration.unobserved_abandon.scenario.acknowledged",
    "lattice.agent_orchestration.unobserved_abandon.scenario.authority",
    "lattice.agent_orchestration.unobserved_abandon.scenario.identity",
    "lattice.agent_orchestration.unobserved_abandon.scenario.observed",
    "lattice.agent_orchestration.unobserved_abandon.scenario.planned",
    "lattice.agent_orchestration.unobserved_abandon.scenario.replay",
];
const PARTIAL_IDS: &[&str] = &[IDS[0], IDS[6], IDS[9], IDS[15], IDS[17], IDS[26]];

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
        x if [
            IDS[0], IDS[2], IDS[3], IDS[4], IDS[6], IDS[12], IDS[13], IDS[14], IDS[17], IDS[18], IDS[24], IDS[33],
            IDS[35], IDS[37],
        ]
        .contains(&x) =>
        {
            &[CORE_PATH]
        }
        x if [IDS[1], IDS[23], IDS[32], IDS[34]].contains(&x) => &[CORE_PATH, SHELL_PATH],
        x if x == IDS[5] => &[STORE_PATH],
        x if x == IDS[7] => &[CORE_PATH, STORE_PATH],
        x if [IDS[8], IDS[9]].contains(&x) => &[CLI_PATH],
        x if [IDS[10], IDS[11], IDS[38]].contains(&x) => &[CORE_PATH, CLI_PATH],
        x if [IDS[15], IDS[16], IDS[25], IDS[26], IDS[30], IDS[36]].contains(&x) => &[SHELL_PATH],
        x if x == IDS[19] => &[CORE_PATH, STORE_PATH],
        x if [IDS[20], IDS[21], IDS[22]].contains(&x) => &[FORMAT_PATH],
        x if [IDS[27], IDS[28]].contains(&x) => &[RUN_STORE_PATH],
        x if x == IDS[29] => &[SHELL_PATH, RUN_STORE_PATH],
        x if x == IDS[31] => &[CORE_PATH, STORE_PATH, SHELL_PATH],
        _ => panic!("unexpected requirement: {id}"),
    }
}

fn partial_rationale(id: &str) -> &'static str {
    match id {
        x if x == IDS[0] => "gap.inert-authority-shaped-output-unasserted",
        x if x == IDS[6] => "gap.overage-breach-and-nonclaim-unasserted",
        x if x == IDS[9] => "gap.complete-actionable-safe-status-unasserted",
        x if x == IDS[15] => "gap.remote-exactly-once-rendered-boundary-unasserted",
        x if x == IDS[17] => "gap.rendered-local-chain-nonclaim-unasserted",
        x if x == IDS[26] => "gap.feedback-cycle-continuation-unasserted",
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
        CORE_PATH => CORE,
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

// r[verify lattice.repository_validation.agent_orchestration_audit]
// r[verify lattice.repository_validation.agent_orchestration_audit.scenario.complete]
// r[verify lattice.repository_validation.agent_orchestration_audit.scenario.evidence]
#[test]
fn agent_orchestration_audit_is_complete_byte_bound_and_unpromoted() {
    let report: InventoryReport = serde_json::from_str(REVIEWED).expect("inventory decodes");
    let review = audit_issues(&report.rows);
    let unchanged = core::validate_unchanged_coverage(&coverage(BASELINE), &coverage(CURRENT));
    let markers = core::validate_no_selected_markers(&[CORE, STORE, FORMAT, SHELL, CLI, RUN_STORE], IDS);
    assert!(review.is_empty(), "review issues: {review:?}");
    assert!(unchanged.is_empty(), "coverage issues: {unchanged:?}");
    assert!(markers.is_empty(), "marker issues: {markers:?}");
    for row in report.rows.iter().filter(|row| IDS.contains(&row.requirement_id.as_str())) {
        for evidence in &row.evidence {
            assert_eq!(blake3::hash(source(&evidence.path).as_bytes()).to_hex().to_string(), evidence.blake3);
        }
    }
}

// r[verify lattice.repository_validation.agent_orchestration_audit.scenario.fail_closed]
#[test]
fn agent_orchestration_audit_drift_fails_closed() {
    let report: InventoryReport = serde_json::from_str(REVIEWED).expect("inventory decodes");
    let mut rows = report.rows;
    rows.retain(|row| row.requirement_id != IDS[0]);
    let row = rows.iter_mut().find(|row| row.requirement_id == IDS[1]).expect("selected row");
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

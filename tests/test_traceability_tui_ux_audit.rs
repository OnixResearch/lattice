#[path = "traceability_review/core.rs"]
mod core;
#[path = "traceability_tui_ux_ids.rs"]
mod tui_ux_ids;

use core::CoverageSnapshot;
use core::ExpectedDecision;
use core::ReviewRow;
use std::collections::BTreeSet;
use std::path::Path;

use tui_ux_ids::IDS;

const BASELINE: &str = include_str!("fixtures/traceability-workflow-execution-promotion-current.json");
const CURRENT: &str = include_str!("fixtures/traceability-tui-ux-audit-current.json");
const REVIEWED: &str = include_str!("fixtures/traceability-tui-ux-audit-reviewed.json");

const APP: &str = include_str!("fixtures/traceability-tui-ux-audit-reviewed-src-tui-app.rs.txt");
const INPUT: &str = include_str!("fixtures/traceability-tui-ux-audit-reviewed-src-tui-input.rs.txt");
const PANELS: &str = include_str!("fixtures/traceability-tui-ux-audit-reviewed-src-tui-panels.rs.txt");
const MAIN: &str = include_str!("fixtures/traceability-tui-ux-audit-reviewed-src-main.rs.txt");
const CONTROL: &str = include_str!("fixtures/traceability-tui-ux-audit-reviewed-test-control-flow.rs.txt");
const CREDENTIALS: &str = include_str!("fixtures/traceability-tui-ux-audit-reviewed-test-credentials.rs.txt");
const NODE_KIND: &str = include_str!("fixtures/traceability-tui-ux-audit-reviewed-test-node-kind-provider.rs.txt");
const REPLAY: &str = include_str!("fixtures/traceability-tui-ux-audit-reviewed-test-replay.rs.txt");
const RUN_HISTORY: &str = include_str!("fixtures/traceability-tui-ux-audit-reviewed-test-run-history-details.rs.txt");
const TRIGGERS: &str = include_str!("fixtures/traceability-tui-ux-audit-reviewed-test-triggers.rs.txt");
const WEBHOOKS: &str = include_str!("fixtures/traceability-tui-ux-audit-reviewed-webhooks.rs.txt");

const APP_PATH: &str = "src/tui/app.rs";
const INPUT_PATH: &str = "src/tui/input.rs";
const PANELS_PATH: &str = "src/tui/panels.rs";
const MAIN_PATH: &str = "src/main.rs";
const CONTROL_PATH: &str = "tests/test_control_flow.rs";
const CREDENTIALS_PATH: &str = "tests/test_credentials.rs";
const NODE_KIND_PATH: &str = "tests/test_node_kind_provider.rs";
const REPLAY_PATH: &str = "tests/test_replay.rs";
const RUN_HISTORY_PATH: &str = "tests/test_run_history_details.rs";
const TRIGGERS_PATH: &str = "tests/test_triggers.rs";
const WEBHOOKS_PATH: &str = "tests/webhooks.rs";

const SUBSYSTEM: &str = "tui-ux";
const EXACT: &str = "exact_maintained_evidence";
const PARTIAL: &str = "partial_evidence_only";
const SELECTED_COUNT: usize = 98;
const PARTIAL_COUNT: usize = 17;
const EXACT_COUNT: usize = SELECTED_COUNT - PARTIAL_COUNT;
const ADDED: &str = "lattice.example.unaccepted.scenario.added";
const DANGLING: &str = "lattice.example.dangling.scenario.marker";
const INVALID_PATH: &str = "../outside.rs";
const INVALID_HASH: &str = "invalid-hash";

const PARTIAL_IDS: &[&str] = &[
    concat!("n8n", "clone.tui_ux.agent_nodes.errors.invalid_target"),
    concat!("n8n", "clone.tui_ux.background_runs.long_running"),
    concat!("n8n", "clone.tui_ux.background_runs.responsive"),
    concat!("n8n", "clone.tui_ux.image_nodes.preview"),
    concat!("n8n", "clone.tui_ux.image_nodes.preview.canvas"),
    concat!("n8n", "clone.tui_ux.keyboard_first"),
    concat!("n8n", "clone.tui_ux.replay_debugger.active_run.stream"),
    concat!("n8n", "clone.tui_ux.replay_debugger.timeline.open"),
    concat!("n8n", "clone.tui_ux.run_history.details.empty_error.storage"),
    concat!("n8n", "clone.tui_ux.run_history.node_outputs.inspect"),
    concat!("n8n", "clone.tui_ux.run_history.node_outputs.missing"),
    concat!("n8n", "clone.tui_ux.run_history.rerun.unavailable"),
    concat!("n8n", "clone.tui_ux.run_history.timeline.show"),
    concat!("n8n", "clone.tui_ux.triggers.manage.error"),
    concat!("n8n", "clone.tui_ux.webhook_listener.cli.start"),
    concat!("n8n", "clone.tui_ux.workspace"),
    concat!("n8n", "clone.tui_ux.workspace.starts"),
];

#[derive(Debug, Clone, serde::Deserialize)]
struct InventoryReport {
    rows: Vec<InventoryRow>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct InventoryRow {
    requirement_id: String,
    subsystem: String,
    classification: String,
    rationale_code: String,
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct AuditIssue {
    code: &'static str,
    subject: String,
}

fn report() -> InventoryReport {
    serde_json::from_str(REVIEWED).expect("reviewed inventory decodes")
}

fn coverage(source: &str) -> CoverageSnapshot {
    let value: Coverage = serde_json::from_str(source).expect("coverage decodes");
    CoverageSnapshot {
        missing: value.missing,
        dangling: value.dangling,
        valid: value.valid,
    }
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

fn expected_decisions() -> Vec<ExpectedDecision> {
    let partial = PARTIAL_IDS.iter().copied().collect::<BTreeSet<_>>();
    IDS.iter()
        .map(|identifier| ExpectedDecision {
            requirement_id: identifier,
            classification: if partial.contains(identifier) { PARTIAL } else { EXACT },
            rationale_code: if partial.contains(identifier) {
                partial_rationale(identifier)
            } else {
                "assertions.direct-maintained-tui-evidence"
            },
            evidence_paths: evidence_paths(identifier),
        })
        .collect()
}

fn evidence_paths(identifier: &str) -> &'static [&'static str] {
    if identifier.starts_with(concat!("n8n", "clone.tui_ux.add_node."))
        || identifier.starts_with(concat!("n8n", "clone.tui_ux.agent_nodes."))
    {
        return &[APP_PATH];
    }
    if identifier.starts_with(concat!("n8n", "clone.tui_ux.background_runs."))
        || identifier.starts_with(concat!("n8n", "clone.tui_ux.unsaved."))
    {
        return &[APP_PATH, PANELS_PATH];
    }
    if identifier.starts_with(concat!("n8n", "clone.tui_ux.command_palette.")) {
        return &[INPUT_PATH];
    }
    if identifier == concat!("n8n", "clone.tui_ux.control_flow.properties") {
        return &[CONTROL_PATH];
    }
    if identifier == concat!("n8n", "clone.tui_ux.credentials.providers.edit") {
        return &[APP_PATH, INPUT_PATH];
    }
    if identifier == concat!("n8n", "clone.tui_ux.credentials.providers.test") {
        return &[CREDENTIALS_PATH];
    }
    if identifier.starts_with(concat!("n8n", "clone.tui_ux.empty_state."))
        || identifier.starts_with(concat!("n8n", "clone.tui_ux.help_footer."))
    {
        return &[PANELS_PATH];
    }
    if identifier.starts_with(concat!("n8n", "clone.tui_ux.image_nodes.")) {
        return &[APP_PATH, PANELS_PATH];
    }
    if identifier.starts_with(concat!("n8n", "clone.tui_ux.keyboard_first")) {
        return &[INPUT_PATH, PANELS_PATH];
    }
    if identifier.starts_with(concat!("n8n", "clone.tui_ux.mouse_wiring.")) {
        return &[INPUT_PATH];
    }
    if identifier == concat!("n8n", "clone.tui_ux.node_kind_provider") {
        return &[NODE_KIND_PATH, PANELS_PATH];
    }
    if identifier.starts_with(concat!("n8n", "clone.tui_ux.properties.")) {
        return &[APP_PATH, INPUT_PATH, PANELS_PATH];
    }
    if identifier.starts_with(concat!("n8n", "clone.tui_ux.replay_debugger.")) {
        return &[APP_PATH, PANELS_PATH, REPLAY_PATH];
    }
    if identifier.starts_with(concat!("n8n", "clone.tui_ux.run_history.")) {
        return &[APP_PATH, PANELS_PATH, RUN_HISTORY_PATH];
    }
    if identifier.starts_with(concat!("n8n", "clone.tui_ux.triggers.")) {
        return &[APP_PATH, TRIGGERS_PATH];
    }
    if identifier == concat!("n8n", "clone.tui_ux.webhook_listener.cli.start") {
        return &[MAIN_PATH, WEBHOOKS_PATH];
    }
    if identifier.starts_with(concat!("n8n", "clone.tui_ux.workflow_browser."))
        || identifier.starts_with(concat!("n8n", "clone.tui_ux.workspace"))
    {
        return &[APP_PATH, PANELS_PATH];
    }
    panic!("unmapped TUI requirement: {identifier}");
}

fn partial_rationale(identifier: &str) -> &'static str {
    match identifier {
        concat!("n8n", "clone.tui_ux.workspace") | concat!("n8n", "clone.tui_ux.workspace.starts") => {
            "gap.no-terminal-startup-layout-harness"
        }
        concat!("n8n", "clone.tui_ux.keyboard_first") => "gap.core-keyboard-surface-not-exhaustively-asserted",
        concat!("n8n", "clone.tui_ux.background_runs.long_running")
        | concat!("n8n", "clone.tui_ux.background_runs.responsive") => "gap.active-render-and-input-loop-unasserted",
        concat!("n8n", "clone.tui_ux.image_nodes.preview")
        | concat!("n8n", "clone.tui_ux.image_nodes.preview.canvas") => "gap.canvas-image-fallback-not-implemented",
        concat!("n8n", "clone.tui_ux.agent_nodes.errors.invalid_target") => "gap.tui-diagnostic-display-unasserted",
        concat!("n8n", "clone.tui_ux.replay_debugger.active_run.stream") => {
            "gap.events-persist-only-after-terminal-state"
        }
        concat!("n8n", "clone.tui_ux.replay_debugger.timeline.open") => "gap.historical-replay-open-not-asserted",
        concat!("n8n", "clone.tui_ux.run_history.details.empty_error.storage") => {
            "gap.storage-error-shell-not-injectable"
        }
        concat!("n8n", "clone.tui_ux.run_history.node_outputs.inspect") => {
            "gap.rendered-node-output-details-unasserted"
        }
        concat!("n8n", "clone.tui_ux.run_history.node_outputs.missing") => {
            "gap.selected-node-empty-output-message-unasserted"
        }
        concat!("n8n", "clone.tui_ux.run_history.rerun.unavailable") => "gap.unavailable-rerun-path-unasserted",
        concat!("n8n", "clone.tui_ux.run_history.timeline.show") => "gap.rendered-timeline-unasserted",
        concat!("n8n", "clone.tui_ux.triggers.manage.error") => "gap.recent-trigger-error-not-modeled",
        concat!("n8n", "clone.tui_ux.webhook_listener.cli.start") => "gap.start-status-output-unasserted",
        _ => panic!("missing partial rationale: {identifier}"),
    }
}

fn source(path: &str) -> Option<&'static str> {
    match path {
        APP_PATH => Some(APP),
        INPUT_PATH => Some(INPUT),
        PANELS_PATH => Some(PANELS),
        MAIN_PATH => Some(MAIN),
        CONTROL_PATH => Some(CONTROL),
        CREDENTIALS_PATH => Some(CREDENTIALS),
        NODE_KIND_PATH => Some(NODE_KIND),
        REPLAY_PATH => Some(REPLAY),
        RUN_HISTORY_PATH => Some(RUN_HISTORY),
        TRIGGERS_PATH => Some(TRIGGERS),
        WEBHOOKS_PATH => Some(WEBHOOKS),
        _ => None,
    }
}

fn validate_evidence(report: &InventoryReport) -> Vec<AuditIssue> {
    let mut issues = Vec::new();
    for row in &report.rows {
        if row.evidence.is_empty() {
            issues.push(AuditIssue {
                code: "evidence.empty",
                subject: row.requirement_id.clone(),
            });
        }
        for evidence in &row.evidence {
            let path = Path::new(&evidence.path);
            if path.is_absolute() || evidence.path.split('/').any(|component| component == "..") {
                issues.push(AuditIssue {
                    code: "evidence.unsafe_path",
                    subject: evidence.path.clone(),
                });
                continue;
            }
            let Some(source) = source(&evidence.path) else {
                issues.push(AuditIssue {
                    code: "evidence.unknown_path",
                    subject: evidence.path.clone(),
                });
                continue;
            };
            let actual = blake3::hash(source.as_bytes()).to_hex().to_string();
            if actual != evidence.blake3 {
                issues.push(AuditIssue {
                    code: "evidence.hash",
                    subject: evidence.path.clone(),
                });
            }
        }
    }
    issues.sort();
    issues.dedup();
    issues
}

fn review_codes(issues: &[core::ReviewIssue]) -> BTreeSet<&str> {
    issues.iter().map(|issue| issue.code).collect()
}

fn audit_codes(issues: &[AuditIssue]) -> BTreeSet<&str> {
    issues.iter().map(|issue| issue.code).collect()
}

fn snapshot_sources() -> [&'static str; 11] {
    [
        APP,
        INPUT,
        PANELS,
        MAIN,
        CONTROL,
        CREDENTIALS,
        NODE_KIND,
        REPLAY,
        RUN_HISTORY,
        TRIGGERS,
        WEBHOOKS,
    ]
}

// r[verify lattice.repository_validation.tui_ux_traceability_audit]
// r[verify lattice.repository_validation.tui_ux_traceability_audit.scenario.scope]
// r[verify lattice.repository_validation.tui_ux_traceability_audit.scenario.evidence]
// r[verify lattice.repository_validation.tui_ux_traceability_audit.scenario.classification]
#[test]
fn all_remaining_tui_ux_rows_are_byte_bound_and_honestly_classified() {
    let report = report();
    let expected = expected_decisions();
    let review_issues = core::validate_tranche(&review_rows(&report), SUBSYSTEM, &expected);
    let evidence_issues = validate_evidence(&report);
    let coverage_issues = core::validate_unchanged_coverage(&coverage(BASELINE), &coverage(CURRENT));
    let marker_issues = core::validate_no_selected_markers(&snapshot_sources(), IDS);
    let classifications = report.rows.iter().map(|row| row.classification.as_str()).collect::<Vec<_>>();

    assert!(review_issues.is_empty(), "review issues: {review_issues:?}");
    assert!(evidence_issues.is_empty(), "evidence issues: {evidence_issues:?}");
    assert!(coverage_issues.is_empty(), "coverage issues: {coverage_issues:?}");
    assert!(marker_issues.is_empty(), "marker issues: {marker_issues:?}");
    assert_eq!(IDS.len(), SELECTED_COUNT);
    assert_eq!(report.rows.len(), SELECTED_COUNT);
    assert_eq!(classifications.iter().filter(|classification| **classification == EXACT).count(), EXACT_COUNT);
    assert_eq!(classifications.iter().filter(|classification| **classification == PARTIAL).count(), PARTIAL_COUNT);
}

// r[verify lattice.repository_validation.tui_ux_traceability_audit.scenario.fail_closed]
#[test]
fn tui_ux_audit_drift_fails_closed() {
    let mut report = report();
    let removed = report.rows.remove(0);
    report.rows.push(report.rows[0].clone());
    let duplicated_index = report.rows.len() - 1;
    report.rows[duplicated_index].classification = "invalid-classification".to_string();
    report.rows[duplicated_index].rationale_code = "invalid-rationale".to_string();
    report.rows[duplicated_index].evidence[0].path = INVALID_PATH.to_string();
    report.rows[1].evidence[0].blake3 = INVALID_HASH.to_string();
    let expected = expected_decisions();
    let review_issues = core::validate_tranche(&review_rows(&report), SUBSYSTEM, &expected);
    let review_issue_codes = review_codes(&review_issues);
    let evidence_issues = validate_evidence(&report);
    let evidence_issue_codes = audit_codes(&evidence_issues);

    let baseline = coverage(BASELINE);
    let mut current = coverage(CURRENT);
    current.missing.retain(|identifier| identifier != IDS[0]);
    current.missing.push(ADDED.to_string());
    current.missing.push(ADDED.to_string());
    current.dangling.push(DANGLING.to_string());
    current.valid = true;
    let coverage_issues = core::validate_unchanged_coverage(&baseline, &current);
    let coverage_issue_codes = review_codes(&coverage_issues);
    let marker = format!("// r[verify {}]", IDS[0]);
    let marker_issues = core::validate_no_selected_markers(&[marker.as_str()], &[IDS[0], IDS[0]]);
    let marker_issue_codes = review_codes(&marker_issues);

    assert_eq!(removed.requirement_id, IDS[0]);
    for expected_code in [
        "review.missing",
        "review.duplicate",
        "review.classification",
        "review.rationale",
        "review.evidence_paths",
    ] {
        assert!(review_issue_codes.contains(expected_code));
    }
    assert!(evidence_issue_codes.contains("evidence.unsafe_path"));
    assert!(evidence_issue_codes.contains("evidence.hash"));
    for expected_code in [
        "coverage.removed",
        "coverage.added",
        "coverage.current_duplicate",
        "coverage.current_unsorted",
        "coverage.current_dangling",
        "coverage.current_valid",
    ] {
        assert!(coverage_issue_codes.contains(expected_code));
    }
    assert!(marker_issue_codes.contains("marker.promoted"));
    assert!(marker_issue_codes.contains("marker.duplicate_selected"));
}

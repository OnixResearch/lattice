#[allow(dead_code)]
#[path = "traceability_review/core.rs"]
mod core;
#[path = "traceability_tui_ux_ids.rs"]
mod tui_ux_ids;

use core::CoverageSnapshot;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::path::Path;

use tui_ux_ids::IDS;

const BASELINE: &str = include_str!("fixtures/traceability-tui-ux-audit-current.json");
const CURRENT: &str = include_str!("fixtures/traceability-tui-ux-completion-current.json");
const AUDIT: &str = include_str!("fixtures/traceability-tui-ux-audit-reviewed.json");
const COMPLETION: &str = include_str!("fixtures/traceability-tui-ux-completion-reviewed.json");

const APP: &str = include_str!("fixtures/traceability-tui-ux-completion-reviewed-src-tui-app.rs.txt");
const INPUT: &str = include_str!("fixtures/traceability-tui-ux-completion-reviewed-src-tui-input.rs.txt");
const PANELS: &str = include_str!("fixtures/traceability-tui-ux-completion-reviewed-src-tui-panels.rs.txt");
const MAIN: &str = include_str!("fixtures/traceability-tui-ux-completion-reviewed-src-main.rs.txt");
const CONTROL: &str = include_str!("fixtures/traceability-tui-ux-completion-reviewed-test-control-flow.rs.txt");
const CREDENTIALS: &str = include_str!("fixtures/traceability-tui-ux-completion-reviewed-test-credentials.rs.txt");
const NODE_KIND: &str = include_str!("fixtures/traceability-tui-ux-completion-reviewed-test-node-kind-provider.rs.txt");
const REPLAY: &str = include_str!("fixtures/traceability-tui-ux-completion-reviewed-test-replay.rs.txt");
const RUN_HISTORY: &str =
    include_str!("fixtures/traceability-tui-ux-completion-reviewed-test-run-history-details.rs.txt");
const TRIGGERS: &str = include_str!("fixtures/traceability-tui-ux-completion-reviewed-test-triggers.rs.txt");
const WEBHOOKS: &str = include_str!("fixtures/traceability-tui-ux-completion-reviewed-webhooks.rs.txt");
const COMPLETION_TEST: &str =
    include_str!("fixtures/traceability-tui-ux-completion-reviewed-test-tui-ux-completion.rs.txt");

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
const COMPLETION_TEST_PATH: &str = "tests/test_tui_ux_completion.rs";

const SUBSYSTEM: &str = "tui-ux";
const EXACT: &str = "exact_maintained_evidence";
const PARTIAL: &str = "partial_evidence_only";
const SELECTED_COUNT: usize = 98;
const AUDIT_PARTIAL_COUNT: usize = 17;
const AUDIT_EXACT_COUNT: usize = SELECTED_COUNT - AUDIT_PARTIAL_COUNT;
const ADDED: &str = "lattice.example.unaccepted.scenario.added";
const DANGLING: &str = "lattice.example.dangling.scenario.marker";
const INVALID_PATH: &str = "../unsafe.rs";
const INVALID_HASH: &str = "invalid-hash";

const COMPLETION_TEST_IDS: &[&str] = &[
    concat!("n8n", "clone.tui_ux.background_runs.long_running"),
    concat!("n8n", "clone.tui_ux.background_runs.responsive"),
    concat!("n8n", "clone.tui_ux.image_nodes.preview"),
    concat!("n8n", "clone.tui_ux.image_nodes.preview.canvas"),
    concat!("n8n", "clone.tui_ux.keyboard_first"),
    concat!("n8n", "clone.tui_ux.replay_debugger.active_run.stream"),
    concat!("n8n", "clone.tui_ux.workspace"),
    concat!("n8n", "clone.tui_ux.workspace.starts"),
];

#[derive(Debug, Clone, serde::Deserialize)]
struct Report {
    rows: Vec<Row>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct Row {
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
struct CompletionIssue {
    code: &'static str,
    subject: String,
}

fn report(source: &str) -> Report {
    serde_json::from_str(source).expect("review report decodes")
}

fn coverage(source: &str) -> CoverageSnapshot {
    let value: Coverage = serde_json::from_str(source).expect("coverage decodes");
    CoverageSnapshot {
        missing: value.missing,
        dangling: value.dangling,
        valid: value.valid,
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
        COMPLETION_TEST_PATH => Some(COMPLETION_TEST),
        _ => None,
    }
}

fn validate_completion(audit: &Report, completion: &Report) -> Vec<CompletionIssue> {
    let mut issues = Vec::new();
    let completion_test_ids = COMPLETION_TEST_IDS.iter().copied().collect::<BTreeSet<_>>();
    if completion_test_ids.len() != COMPLETION_TEST_IDS.len() {
        issues.push(issue("expectation.duplicate_completion_test_id", "completion-test-ids"));
    }
    let mut audit_by_id = BTreeMap::new();
    for row in &audit.rows {
        if audit_by_id.insert(row.requirement_id.as_str(), row).is_some() {
            issues.push(issue("audit.duplicate", &row.requirement_id));
        }
    }
    let mut completion_by_id = BTreeMap::new();
    for row in &completion.rows {
        if completion_by_id.insert(row.requirement_id.as_str(), row).is_some() {
            issues.push(issue("completion.duplicate", &row.requirement_id));
        }
        if row.subsystem != SUBSYSTEM {
            issues.push(issue("completion.subsystem", &row.requirement_id));
        }
        if row.classification != EXACT {
            issues.push(issue("completion.classification", &row.requirement_id));
        }
    }

    for identifier in IDS {
        let Some(audit_row) = audit_by_id.get(identifier) else {
            issues.push(issue("audit.missing", *identifier));
            continue;
        };
        let Some(completion_row) = completion_by_id.get(identifier) else {
            issues.push(issue("completion.missing", *identifier));
            continue;
        };
        let expected_rationale = if audit_row.classification == PARTIAL {
            "assertions.completed-maintained-tui-evidence"
        } else {
            "assertions.direct-maintained-tui-evidence"
        };
        if completion_row.rationale_code != expected_rationale {
            issues.push(issue("completion.rationale", *identifier));
        }
        let mut expected_paths = audit_row.evidence.iter().map(|evidence| evidence.path.as_str()).collect::<Vec<_>>();
        if completion_test_ids.contains(identifier) {
            expected_paths.push(COMPLETION_TEST_PATH);
        }
        expected_paths.sort_unstable();
        expected_paths.dedup();
        let mut actual_paths =
            completion_row.evidence.iter().map(|evidence| evidence.path.as_str()).collect::<Vec<_>>();
        actual_paths.sort_unstable();
        actual_paths.dedup();
        if actual_paths != expected_paths {
            issues.push(issue("completion.evidence_paths", *identifier));
        }
    }
    for identifier in completion_by_id.keys() {
        if !IDS.contains(identifier) {
            issues.push(issue("completion.unexpected", *identifier));
        }
    }
    issues.extend(validate_evidence(completion));
    issues.sort();
    issues.dedup();
    issues
}

fn validate_evidence(report: &Report) -> Vec<CompletionIssue> {
    let mut issues = Vec::new();
    for row in &report.rows {
        if row.evidence.is_empty() {
            issues.push(issue("evidence.empty", &row.requirement_id));
        }
        for evidence in &row.evidence {
            let path = Path::new(&evidence.path);
            if path.is_absolute() || evidence.path.split('/').any(|component| component == "..") {
                issues.push(issue("evidence.unsafe_path", &evidence.path));
                continue;
            }
            let Some(source) = source(&evidence.path) else {
                issues.push(issue("evidence.unknown_path", &evidence.path));
                continue;
            };
            if blake3::hash(source.as_bytes()).to_hex().to_string() != evidence.blake3 {
                issues.push(issue("evidence.hash", &evidence.path));
            }
        }
    }
    issues
}

fn issue(code: &'static str, subject: impl Into<String>) -> CompletionIssue {
    CompletionIssue {
        code,
        subject: subject.into(),
    }
}

fn completion_codes(issues: &[CompletionIssue]) -> BTreeSet<&str> {
    issues.iter().map(|issue| issue.code).collect()
}

fn review_codes(issues: &[core::ReviewIssue]) -> BTreeSet<&str> {
    issues.iter().map(|issue| issue.code).collect()
}

fn completion_sources() -> [&'static str; 12] {
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
        COMPLETION_TEST,
    ]
}

// r[verify lattice.repository_validation.tui_ux_evidence_completion]
// r[verify lattice.repository_validation.tui_ux_evidence_completion.scenario.exact]
// r[verify lattice.repository_validation.tui_ux_evidence_completion.scenario.coverage]
#[test]
fn all_tui_ux_rows_become_exact_without_changing_tracey_debt() {
    let audit = report(AUDIT);
    let completion = report(COMPLETION);
    let completion_issues = validate_completion(&audit, &completion);
    let coverage_issues = core::validate_unchanged_coverage(&coverage(BASELINE), &coverage(CURRENT));
    let marker_issues = core::validate_no_selected_markers(&completion_sources(), IDS);
    let audit_exact = audit.rows.iter().filter(|row| row.classification == EXACT).count();
    let audit_partial = audit.rows.iter().filter(|row| row.classification == PARTIAL).count();

    assert!(completion_issues.is_empty(), "completion issues: {completion_issues:?}");
    assert!(coverage_issues.is_empty(), "coverage issues: {coverage_issues:?}");
    assert!(marker_issues.is_empty(), "marker issues: {marker_issues:?}");
    assert_eq!(IDS.len(), SELECTED_COUNT);
    assert_eq!(audit.rows.len(), SELECTED_COUNT);
    assert_eq!(completion.rows.len(), SELECTED_COUNT);
    assert_eq!(audit_exact, AUDIT_EXACT_COUNT);
    assert_eq!(audit_partial, AUDIT_PARTIAL_COUNT);
    assert!(completion.rows.iter().all(|row| row.classification == EXACT));
}

// r[verify lattice.repository_validation.tui_ux_evidence_completion.scenario.fail_closed]
#[test]
fn tui_ux_completion_drift_fails_closed() {
    let audit = report(AUDIT);
    let mut completion = report(COMPLETION);
    let removed = completion.rows.remove(0);
    completion.rows.push(completion.rows[0].clone());
    let duplicate_index = completion.rows.len() - 1;
    completion.rows[duplicate_index].classification = PARTIAL.to_string();
    completion.rows[duplicate_index].rationale_code = "gap.stale".to_string();
    completion.rows[duplicate_index].evidence[0].path = INVALID_PATH.to_string();
    completion.rows[1].evidence[0].blake3 = INVALID_HASH.to_string();
    let completion_issues = validate_completion(&audit, &completion);
    let completion_issue_codes = completion_codes(&completion_issues);

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
    for expected in [
        "completion.missing",
        "completion.duplicate",
        "completion.classification",
        "completion.rationale",
        "completion.evidence_paths",
        "evidence.unsafe_path",
        "evidence.hash",
    ] {
        assert!(completion_issue_codes.contains(expected));
    }
    for expected in [
        "coverage.removed",
        "coverage.added",
        "coverage.current_duplicate",
        "coverage.current_unsorted",
        "coverage.current_dangling",
        "coverage.current_valid",
    ] {
        assert!(coverage_issue_codes.contains(expected));
    }
    assert!(marker_issue_codes.contains("marker.promoted"));
    assert!(marker_issue_codes.contains("marker.duplicate_selected"));
}

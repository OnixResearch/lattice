use crate::interaction_replay::{
    LeaderReplaySummary, LeaderReplayTarget, ReplayIssueCode, ReplayReport,
    leader_key_negative_replay_script, leader_key_positive_replay_script,
    leader_key_replay_summary, leader_key_replay_target, run_replay,
};

const POSITIVE_FIXTURE_ID: &str = "leader-key.workspace-action.baseline";
const NEGATIVE_FIXTURE_ID: &str = "leader-key.invalid.baseline";
const ACTION_WORKSPACE_SAVE: &str = "workspace.save";
const DISABLED_ACTION_DIAGNOSTIC: &str = "DisabledAction";
const EXPECTED_EVIDENCE_ROWS: usize = 2;

#[test]
fn leader_key_positive_fixture_replays_submenu_and_action() {
    let mut target = leader_key_replay_target();
    let script = leader_key_positive_replay_script();

    let outcome = run_replay(&script, &mut target);

    assert!(outcome.is_success());
    assert_eq!(
        outcome.state.get("selected_action"),
        Some(&ACTION_WORKSPACE_SAVE.to_string())
    );
    assert_eq!(outcome.state.get("visible"), Some(&false.to_string()));
    assert_eq!(outcome.state_hash, script.snapshots[0].blake3);
}

#[test]
fn leader_key_negative_fixture_reports_disabled_stale_and_unsupported() {
    let mut target = LeaderReplayTarget::new();
    let script = leader_key_negative_replay_script();

    let outcome = run_replay(&script, &mut target);

    assert_issue(&outcome, ReplayIssueCode::DisabledAction);
    assert_issue(&outcome, ReplayIssueCode::StaleFocus);
    assert_issue(&outcome, ReplayIssueCode::UnsupportedEvent);
    assert_eq!(outcome.state.get("selected_action"), Some(&"-".to_string()));
    assert_eq!(outcome.state.get("visible"), Some(&true.to_string()));
    assert_eq!(outcome.state_hash, script.snapshots[0].blake3);
}

#[test]
fn leader_key_evidence_includes_positive_and_negative_rows() {
    let evidence = leader_key_replay_summary();

    assert_eq!(evidence.rows.len(), EXPECTED_EVIDENCE_ROWS);
    assert!(has_positive_row(&evidence));
    assert!(has_negative_row(&evidence));
}

fn has_positive_row(evidence: &LeaderReplaySummary) -> bool {
    evidence
        .rows
        .iter()
        .any(|row| row.fixture_id == POSITIVE_FIXTURE_ID && row.diagnostics == "-")
}

fn has_negative_row(evidence: &LeaderReplaySummary) -> bool {
    evidence.rows.iter().any(|row| {
        row.fixture_id == NEGATIVE_FIXTURE_ID
            && row.diagnostics.contains(DISABLED_ACTION_DIAGNOSTIC)
    })
}

fn assert_issue(report: &ReplayReport, code: ReplayIssueCode) {
    assert!(report.issues.iter().any(|issue| issue.code == code));
}

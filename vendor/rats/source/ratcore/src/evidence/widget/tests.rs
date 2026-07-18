use super::*;

#[path = "leadertests.rs"]
mod leader;

const EXPECTED_WORKSPACE_SHELL_RENDERER_ARTIFACTS: usize = 4;
const WORKSPACE_SHELL_POSITIVE_TUI: &str = include_str!(
    "../../../../docs/generated/replay-snapshots/workspace-shell-positive.subwayrat.txt"
);
const WORKSPACE_SHELL_POSITIVE_DOM: &str = include_str!(
    "../../../../docs/generated/replay-snapshots/workspace-shell-positive.meteorite.html"
);
const WORKSPACE_SHELL_NEGATIVE_TUI: &str = include_str!(
    "../../../../docs/generated/replay-snapshots/workspace-shell-negative.subwayrat.txt"
);
const WORKSPACE_SHELL_NEGATIVE_DOM: &str = include_str!(
    "../../../../docs/generated/replay-snapshots/workspace-shell-negative.meteorite.html"
);

#[test]
fn tree_navigation_baseline_replays_and_hashes_state() {
    let mut target = tree_target();
    let script = tree_navigation_replay_script();

    let outcome = run_replay(&script, &mut target);

    assert!(outcome.is_success());
    assert_eq!(
        outcome.state.get("selected_node"),
        Some(&"src/lib.rs".to_string())
    );
    assert_eq!(outcome.state_hash, script.snapshots[0].blake3);
}

#[test]
fn command_palette_baseline_replays_filter_and_selection() {
    let mut target = command_palette_target();
    let script = command_palette_replay_script();

    let outcome = run_replay(&script, &mut target);

    assert!(outcome.is_success());
    assert_eq!(
        outcome.state.get("selected_action"),
        Some(&"action.open".to_string())
    );
    assert_eq!(outcome.state_hash, script.snapshots[0].blake3);
}

#[test]
fn replay_reports_boundaries_unknown_commands_and_malformed_text() {
    let mut tree = tree_target();
    let tree_script = ReplayScript::new(ReplayMetadata::new("tree.bad", "bad tree", "tree"))
        .with_event(ReplayEvent::Key {
            target: TARGET_TREE.to_string(),
            key: "ArrowUp".to_string(),
        })
        .with_event(ReplayEvent::Command {
            action_id: ACTION_UNKNOWN.to_string(),
        });

    let tree_outcome = run_replay(&tree_script, &mut tree);

    assert_issue(&tree_outcome, ReplayIssueCode::OutOfBoundsMovement);
    assert_issue(&tree_outcome, ReplayIssueCode::UnknownCommand);

    let mut palette = command_palette_target();
    let palette_script = ReplayScript::new(ReplayMetadata::new(
        "palette.bad",
        "bad palette",
        "command-palette",
    ))
    .with_event(ReplayEvent::Text {
        target: TARGET_COMMAND_PALETTE.to_string(),
        value: "bad\ntext".to_string(),
    });

    let palette_outcome = run_replay(&palette_script, &mut palette);

    assert_eq!(
        tree_outcome
            .issues
            .len()
            .saturating_add(palette_outcome.issues.len()),
        EXPECTED_WIDGET_REPLAY_ISSUES
    );
    assert!(!palette_outcome.is_success());
    assert_issue(&palette_outcome, ReplayIssueCode::MalformedText);
}

#[test]
fn workspace_shell_fixture_replays_waits_and_snapshot_evidence() {
    let mut target = workspace_shell_replay_target();
    let script = workspace_shell_positive_replay_script();

    let outcome = run_replay(&script, &mut target);

    assert!(outcome.is_success());
    assert_eq!(
        outcome.state.get("focus_pane"),
        Some(&"pane:task".to_string())
    );
    assert_eq!(
        outcome.state.get("workspace_status"),
        Some(&"working".to_string())
    );
}

#[test]
fn workspace_shell_negative_fixture_reports_stable_diagnostics() {
    let mut target = workspace_shell_replay_target();
    let script = workspace_shell_negative_replay_script();

    let outcome = run_replay(&script, &mut target);

    assert_issue(&outcome, ReplayIssueCode::DisabledAction);
    assert_issue(&outcome, ReplayIssueCode::UnsupportedWaitPredicate);
    assert_issue(&outcome, ReplayIssueCode::ImpossibleGeometry);
    assert_eq!(
        outcome.state.get("focus_pane"),
        Some(&"pane:editor".to_string())
    );
}

#[test]
fn workspace_shell_evidence_includes_positive_and_negative_rows() {
    let evidence = workspace_shell_replay_evidence();

    assert_eq!(evidence.rows.len(), 2);
    assert!(
        evidence
            .rows
            .iter()
            .any(|row| row.fixture_id == "workspace-shell.positive" && row.diagnostics == "-")
    );
    assert!(
        evidence
            .rows
            .iter()
            .any(|row| row.fixture_id == "workspace-shell.negative"
                && row.diagnostics.contains("DisabledAction"))
    );
}

#[test]
fn workspace_shell_renderer_snapshots_validate_and_match_checked_in_artifacts() {
    let evidence = workspace_shell_replay_evidence();
    let artifacts = workspace_shell_renderer_snapshots();
    let report = validate_workspace_shell_renderer_snapshots(&evidence.rows, &artifacts);

    assert!(report.is_valid());
    assert_eq!(artifacts.len(), EXPECTED_WORKSPACE_SHELL_RENDERER_ARTIFACTS);
    assert_artifact_bytes(
        &artifacts,
        "workspace-shell.positive",
        "subwayrat-tui",
        WORKSPACE_SHELL_POSITIVE_TUI.as_bytes(),
    );
    assert_artifact_bytes(
        &artifacts,
        "workspace-shell.positive",
        "meteorite-dom",
        WORKSPACE_SHELL_POSITIVE_DOM.as_bytes(),
    );
    assert_artifact_bytes(
        &artifacts,
        "workspace-shell.negative",
        "subwayrat-tui",
        WORKSPACE_SHELL_NEGATIVE_TUI.as_bytes(),
    );
    assert_artifact_bytes(
        &artifacts,
        "workspace-shell.negative",
        "meteorite-dom",
        WORKSPACE_SHELL_NEGATIVE_DOM.as_bytes(),
    );
}

#[test]
fn workspace_shell_renderer_snapshot_validation_rejects_bad_evidence() {
    let evidence = workspace_shell_replay_evidence();
    let mut stale = workspace_shell_renderer_snapshots()
        .first()
        .cloned()
        .expect("workspace renderer artifact exists");
    stale.blake3 = "stale".to_string();
    let duplicate = stale.clone();
    let unsupported = WorkspaceShellRendererSnapshot::new(
        "workspace-shell.missing",
        "unknown-renderer",
        "docs/generated/replay-snapshots/missing.unknown",
        "old-generator",
        Vec::<u8>::new(),
    );

    let report = validate_workspace_shell_renderer_snapshots(
        &evidence.rows,
        &[stale, duplicate, unsupported],
    );

    assert!(!report.is_valid());
    assert_snapshot_issue(
        &report,
        WorkspaceShellRendererSnapshotIssueCode::DuplicateArtifact,
    );
    assert_snapshot_issue(
        &report,
        WorkspaceShellRendererSnapshotIssueCode::MissingFixture,
    );
    assert_snapshot_issue(
        &report,
        WorkspaceShellRendererSnapshotIssueCode::UnsupportedRenderer,
    );
    assert_snapshot_issue(
        &report,
        WorkspaceShellRendererSnapshotIssueCode::UnsupportedGenerator,
    );
    assert_snapshot_issue(
        &report,
        WorkspaceShellRendererSnapshotIssueCode::EmptyArtifact,
    );
    assert_snapshot_issue(&report, WorkspaceShellRendererSnapshotIssueCode::StaleHash);
    assert_snapshot_issue(
        &report,
        WorkspaceShellRendererSnapshotIssueCode::MissingRenderer,
    );
}

#[test]
fn disabled_action_diagnostic_names_step_surface_and_action() {
    let mut target = command_palette_target();
    let script = ReplayScript::new(ReplayMetadata::new(
        "palette.disabled",
        "disabled palette action",
        "command-palette",
    ))
    .with_event(ReplayEvent::Command {
        action_id: ACTION_DISABLED.to_string(),
    });

    let outcome = run_replay(&script, &mut target);

    assert_issue(&outcome, ReplayIssueCode::DisabledAction);
    let issue = outcome
        .issues
        .iter()
        .find(|issue| issue.code == ReplayIssueCode::DisabledAction)
        .expect("disabled action diagnostic exists");
    assert!(issue.message.contains("step 1"));
    assert!(issue.message.contains(TARGET_COMMAND_PALETTE));
    assert!(issue.message.contains(ACTION_DISABLED));
    assert!(issue.message.contains("disabled state"));
}

fn tree_target() -> TreeReplayTarget {
    TreeReplayTarget::new(vec![
        ReplayTreeNode::new("src").with_child(ReplayTreeNode::new("src/lib.rs")),
        ReplayTreeNode::new("README.md"),
    ])
}

fn command_palette_target() -> CommandPaletteReplayTarget {
    CommandPaletteReplayTarget::new(vec![
        ReplayCommandAction::enabled("action.open", "Open file"),
        ReplayCommandAction::disabled(ACTION_DISABLED, "Build project"),
    ])
}

fn assert_issue(report: &ReplayReport, code: ReplayIssueCode) {
    assert!(report.issues.iter().any(|issue| issue.code == code));
}

fn assert_artifact_bytes(
    artifacts: &[WorkspaceShellRendererSnapshot],
    fixture_id: &str,
    renderer: &str,
    expected: &[u8],
) {
    let artifact = artifacts
        .iter()
        .find(|artifact| artifact.fixture_id == fixture_id && artifact.renderer == renderer)
        .expect("workspace shell renderer artifact exists");
    assert_eq!(artifact.bytes, expected);
}

fn assert_snapshot_issue(
    report: &WorkspaceShellRendererSnapshotReport,
    code: WorkspaceShellRendererSnapshotIssueCode,
) {
    assert!(report.issues.iter().any(|issue| issue.code == code));
}

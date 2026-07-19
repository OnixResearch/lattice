use super::*;

const WORKSPACE_LIVE: &str = "workspace:1";
const WORKSPACE_PERSISTED: &str = "workspace:persisted";
const TAB_LIVE: &str = "tab:1";
const TAB_PERSISTED: &str = "tab:persisted";
const PANE_EDITOR_LIVE: &str = "pane:editor";
const PANE_EDITOR_PERSISTED: &str = "pane:editor:persisted";
const PANE_LOG_LIVE: &str = "pane:log";
const PANE_LOG_PERSISTED: &str = "pane:log:persisted";
const PANE_TASK_LIVE: &str = "pane:task";
const PANE_TASK_PERSISTED: &str = "pane:task:persisted";
const ACTION_SPLIT: &str = "workspace.split.background";
const ACTION_DISABLED: &str = "workspace.close.disabled";
const ACTION_SHORTCUT_KEY: &str = "\\";
const ACTION_SHORTCUT_MODIFIER: &str = "Ctrl";
const DISABLED_REASON: &str = "pane is locked";

#[test]
fn model_constructs_focus_and_status_rollups() {
    let model = sample_model();

    let rollup = workspace_status_rollup(&model.workspaces[0]);

    assert_eq!(
        model.focus.as_ref().map(|focus| focus.pane.as_str()),
        Some(PANE_EDITOR_LIVE)
    );
    assert_eq!(rollup.working, 1);
    assert_eq!(rollup.blocked, 1);
    assert_eq!(rollup.done, 1);
    assert_eq!(rollup.idle, 1);
    assert_eq!(rollup.unknown, 1);
    assert_eq!(rollup.overall, WorkspaceTaskStatus::Blocked);
}

#[test]
fn duplicate_persisted_ids_and_stale_live_ids_fail_validation() {
    let duplicate_pane = WorkspaceShellPane::new(
        PANE_LOG_LIVE,
        PANE_EDITOR_PERSISTED,
        "duplicate",
        WorkspaceTaskStatus::Idle,
    );
    let model = WorkspaceShellModel::new(vec![WorkspaceShellWorkspace::new(
        WORKSPACE_LIVE,
        WORKSPACE_PERSISTED,
        "workspace",
        vec![WorkspaceShellTab::new(
            TAB_LIVE,
            TAB_PERSISTED,
            "main",
            vec![editor_pane(), duplicate_pane],
        )],
    )])
    .with_focus(WorkspaceShellFocus::new(
        WORKSPACE_LIVE,
        TAB_LIVE,
        PANE_TASK_LIVE,
    ))
    .with_retired_live_id(PANE_TASK_LIVE);

    let diagnostics = validate_workspace_shell(&model);

    assert_diagnostic(
        &diagnostics,
        WorkspaceShellDiagnosticCode::DuplicatePersistedId,
    );
    assert_diagnostic(&diagnostics, WorkspaceShellDiagnosticCode::StaleLiveId);
}

#[test]
fn reducer_splits_background_pane_without_stealing_focus() {
    let model = sample_model();
    let command = WorkspaceShellCommand::SplitPane {
        workspace: WORKSPACE_LIVE.to_string(),
        tab: TAB_LIVE.to_string(),
        target_pane: PANE_EDITOR_LIVE.to_string(),
        pane: task_pane(),
        axis: WorkspaceSplitAxis::Horizontal,
        focus_policy: WorkspaceFocusPolicy::Preserve,
    };

    let output = reduce_workspace_shell_command(&model, &command);

    assert!(output.diagnostics.is_empty());
    assert_eq!(output.effects.len(), 1);
    assert_eq!(
        output.model.focus.as_ref().map(|focus| focus.pane.as_str()),
        Some(PANE_EDITOR_LIVE)
    );
    assert!(
        output.model.workspaces[0].tabs[0]
            .panes
            .iter()
            .any(|pane| pane.live_id == PANE_TASK_LIVE)
    );
}

#[test]
fn invalid_command_preserves_state_and_emits_no_effects() {
    let model = sample_model().with_retired_live_id(PANE_TASK_LIVE);
    let command = WorkspaceShellCommand::FocusPane {
        workspace: WORKSPACE_LIVE.to_string(),
        tab: TAB_LIVE.to_string(),
        pane: PANE_TASK_LIVE.to_string(),
    };

    let output = reduce_workspace_shell_command(&model, &command);

    assert_eq!(output.model, model);
    assert!(output.effects.is_empty());
    assert_diagnostic(
        &output.diagnostics,
        WorkspaceShellDiagnosticCode::StaleLiveId,
    );
}

#[test]
fn restore_rejects_duplicate_persisted_ids_without_mutating_state() {
    let model = sample_model();
    let invalid_restore = WorkspaceShellModel::new(vec![WorkspaceShellWorkspace::new(
        WORKSPACE_LIVE,
        WORKSPACE_PERSISTED,
        "workspace",
        vec![WorkspaceShellTab::new(
            TAB_LIVE,
            TAB_PERSISTED,
            "main",
            vec![editor_pane(), editor_pane()],
        )],
    )]);
    let command = WorkspaceShellCommand::RestoreLayout {
        model: invalid_restore,
    };

    let output = reduce_workspace_shell_command(&model, &command);

    assert_eq!(output.model, model);
    assert!(output.effects.is_empty());
    assert_diagnostic(
        &output.diagnostics,
        WorkspaceShellDiagnosticCode::UnsupportedRestoreShape,
    );
}

#[test]
fn workspace_actions_project_consistent_metadata_across_surfaces() {
    let surfaces = sample_action_surfaces();

    let workspace_projection = surfaces
        .workspace
        .iter()
        .find(|action| action.action_id == ACTION_SPLIT)
        .expect("split action is projected");

    assert!(
        surfaces
            .keymap
            .iter()
            .any(|item| item.action_id == ACTION_SPLIT)
    );
    assert!(
        surfaces
            .menu
            .iter()
            .any(|item| item.action_id == ACTION_SPLIT)
    );
    assert!(
        surfaces
            .palette
            .iter()
            .any(|item| item.action_id == ACTION_SPLIT)
    );
    assert_eq!(
        surfaces.help.get(ACTION_SPLIT),
        Some(&"Ctrl+\\".to_string())
    );
    assert_eq!(
        workspace_projection.focus_policy,
        WorkspaceFocusPolicy::Preserve
    );
}

#[test]
fn disabled_workspace_action_fails_closed_in_all_projections() {
    let surfaces = sample_action_surfaces();
    let disabled = surfaces
        .workspace
        .iter()
        .find(|action| action.action_id == ACTION_DISABLED)
        .expect("disabled action is projected");

    assert!(!disabled.enabled);
    assert_eq!(disabled.disabled_reason.as_deref(), Some(DISABLED_REASON));
    assert!(disabled.diagnostic.is_some());
    assert!(
        surfaces
            .palette
            .iter()
            .any(|item| item.action_id == ACTION_DISABLED && !item.enabled)
    );
}

#[test]
fn view_model_contains_renderable_state_and_visible_diagnostics() {
    let model = sample_model().with_diagnostic(WorkspaceShellDiagnostic::new(
        WorkspaceShellDiagnosticCode::MissingStatusAdapter,
        PANE_LOG_LIVE,
        "status adapter missing",
    ));
    let actions = sample_action_surfaces();

    let view = build_workspace_shell_view_model(&model, &actions);

    assert_eq!(
        view.workspaces[0].tabs[0].panes[0].status,
        WorkspaceTaskStatus::Working.label()
    );
    assert!(view.workspaces[0].tabs[0].panes[0].focused);
    assert!(
        view.actions
            .iter()
            .any(|action| action.action_id == ACTION_SPLIT)
    );
    assert!(
        view.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.subject == PANE_LOG_LIVE)
    );
}

fn sample_model() -> WorkspaceShellModel {
    WorkspaceShellModel::new(vec![WorkspaceShellWorkspace::new(
        WORKSPACE_LIVE,
        WORKSPACE_PERSISTED,
        "workspace",
        vec![WorkspaceShellTab::new(
            TAB_LIVE,
            TAB_PERSISTED,
            "main",
            vec![
                editor_pane(),
                WorkspaceShellPane::new(
                    PANE_LOG_LIVE,
                    PANE_LOG_PERSISTED,
                    "logs",
                    WorkspaceTaskStatus::Blocked,
                )
                .with_diagnostic(WorkspaceShellDiagnostic::new(
                    WorkspaceShellDiagnosticCode::UnsupportedStatusProvider,
                    PANE_LOG_LIVE,
                    "unsupported provider",
                )),
                WorkspaceShellPane::new(
                    "pane:done",
                    "pane:done:persisted",
                    "done",
                    WorkspaceTaskStatus::Done,
                ),
                WorkspaceShellPane::new(
                    "pane:idle",
                    "pane:idle:persisted",
                    "idle",
                    WorkspaceTaskStatus::Idle,
                ),
                WorkspaceShellPane::new(
                    "pane:unknown",
                    "pane:unknown:persisted",
                    "unknown",
                    WorkspaceTaskStatus::Unknown,
                ),
            ],
        )],
    )])
}

fn editor_pane() -> WorkspaceShellPane {
    WorkspaceShellPane::new(
        PANE_EDITOR_LIVE,
        PANE_EDITOR_PERSISTED,
        "editor",
        WorkspaceTaskStatus::Working,
    )
    .with_content_summary("src/lib.rs")
}

fn task_pane() -> WorkspaceShellPane {
    WorkspaceShellPane::new(
        PANE_TASK_LIVE,
        PANE_TASK_PERSISTED,
        "task",
        WorkspaceTaskStatus::Idle,
    )
}

fn sample_action_surfaces() -> WorkspaceShellActionSurfaces {
    project_workspace_shell_actions(vec![
        WorkspaceShellActionDescriptor::new(
            ACTION_SPLIT,
            "Split Background Pane",
            WorkspaceFocusPolicy::Preserve,
        )
        .with_shortcut(crate::action_registry::ActionShortcut::global(
            ACTION_SHORTCUT_KEY,
            vec![ACTION_SHORTCUT_MODIFIER.to_string()],
        )),
        WorkspaceShellActionDescriptor::new(
            ACTION_DISABLED,
            "Close Locked Pane",
            WorkspaceFocusPolicy::Transfer,
        )
        .disabled(DISABLED_REASON),
    ])
}

fn assert_diagnostic(diagnostics: &[WorkspaceShellDiagnostic], code: WorkspaceShellDiagnosticCode) {
    assert!(diagnostics.iter().any(|diagnostic| diagnostic.code == code));
}

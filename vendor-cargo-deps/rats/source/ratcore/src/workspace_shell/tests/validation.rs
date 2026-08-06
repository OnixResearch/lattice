use super::*;

#[test]
fn duplicate_persisted_and_stale_live_ids_are_rejected() {
    let duplicate_pane =
        WorkspaceShellPane::new(PANE_LOG_LIVE, PANE_EDITOR_PERSISTED, "duplicate", WorkspaceTaskStatus::Idle);
    let model = WorkspaceShellModel::new(vec![WorkspaceShellWorkspace::new(
        WORKSPACE_LIVE,
        WORKSPACE_PERSISTED,
        "workspace",
        vec![WorkspaceShellTab::new(TAB_LIVE, TAB_PERSISTED, "main", vec![
            editor_pane(),
            duplicate_pane,
        ])],
    )])
    .with_focus(WorkspaceShellFocus::new(WORKSPACE_LIVE, TAB_LIVE, PANE_TASK_LIVE))
    .with_retired_live_id(WorkspacePaneId::new(PANE_TASK_LIVE));

    let diagnostics = validate_workspace_shell(&model);

    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.code == WorkspaceShellDiagnosticCode::DuplicatePersistedId })
    );
    assert!(diagnostics.iter().any(|diagnostic| diagnostic.code == WorkspaceShellDiagnosticCode::StaleLiveId));
}

#[test]
fn restore_rejects_duplicate_persisted_ids_without_mutating_state() {
    let model = sample_model();
    let invalid_restore = WorkspaceShellModel::new(vec![WorkspaceShellWorkspace::new(
        WORKSPACE_LIVE,
        WORKSPACE_PERSISTED,
        "workspace",
        vec![WorkspaceShellTab::new(TAB_LIVE, TAB_PERSISTED, "main", vec![
            editor_pane(),
            editor_pane(),
        ])],
    )]);
    let command = WorkspaceShellCommand::RestoreLayout { model: invalid_restore };

    let output = reduce_workspace_shell_command(&model, &command);

    assert_eq!(output.model, model);
    assert!(output.effects.is_empty());
    assert_diagnostic(&output.diagnostics, WorkspaceShellDiagnosticCode::UnsupportedRestoreShape);
}

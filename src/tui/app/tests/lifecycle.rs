use super::*;

// r[verify n8nclone.tui_ux.unsaved.mutation_marks_dirty]
#[test]
fn mutation_marks_workflow_dirty() {
    let temp_dir = tempfile::tempdir().expect("temporary app directory");
    let mut app = State::new(test_config(&temp_dir));

    assert!(!app.has_unsaved_changes());
    app.add_shell_node("Example", "echo example");

    assert!(app.has_unsaved_changes());
}

// r[verify n8nclone.tui_ux.unsaved.save_clears]
#[test]
fn successful_save_clears_dirty_state() {
    let temp_dir = tempfile::tempdir().expect("temporary app directory");
    let mut app = State::new(test_config(&temp_dir));
    app.add_shell_node("Example", "echo example");

    assert!(app.has_unsaved_changes());
    app.save_workflow();

    assert!(!app.has_unsaved_changes());
    assert!(!app.is_confirming_quit());
}

#[test]
fn app_creates_missing_data_directory_and_persists_workflow() {
    let temp_dir = tempfile::tempdir().expect("temporary app directory");
    let data_dir = temp_dir.path().join("nested").join("state");
    let mut app = State::new(AppConfig {
        data_dir: data_dir.clone(),
        ..AppConfig::default()
    });

    assert!(app.store.is_some(), "store opens in the created data directory");
    app.add_shell_node("Persist", "true");
    app.save_workflow();

    assert!(!app.has_unsaved_changes());
    assert!(crate::config::store_path(&data_dir).exists());
    assert!(app.store.as_ref().unwrap().load_workflow("untitled").unwrap().is_some());
}

// r[verify n8nclone.tui_ux.unsaved.failed_save]
#[test]
fn failed_save_keeps_dirty_state() {
    let temp_dir = tempfile::tempdir().expect("temporary app directory");
    let mut app = State::new(test_config(&temp_dir));
    app.add_shell_node("Example", "echo example");
    app.store = None;

    app.save_workflow();

    assert!(app.has_unsaved_changes());
}

// r[verify n8nclone.tui_ux.unsaved.quit_confirm]
#[test]
fn dirty_quit_requests_confirmation() {
    let temp_dir = tempfile::tempdir().expect("temporary app directory");
    let mut app = State::new(test_config(&temp_dir));
    app.add_shell_node("Example", "echo example");

    assert!(!app.request_quit());
    assert!(app.is_confirming_quit());

    app.cancel_quit_confirmation();
    assert!(!app.is_confirming_quit());
}

#[test]
fn clean_quit_exits_without_confirmation() {
    let temp_dir = tempfile::tempdir().expect("temporary app directory");
    let mut app = State::new(test_config(&temp_dir));

    assert!(app.request_quit());
    assert!(!app.is_confirming_quit());
}

// r[verify n8nclone.tui_ux.workflow_browser.open]
#[test]
fn workflow_browser_lists_saved_workflows() {
    let temp_dir = tempfile::tempdir().expect("temporary app directory");
    let mut app = State::new(test_config(&temp_dir));
    app.workflow.meta.name = "saved".to_string();
    app.save_workflow();

    app.open_workflow_browser();

    assert!(app.is_workflow_browser_open());
    assert_eq!(app.workflow_browser.names, vec!["saved".to_string()]);
    assert_eq!(app.workflow_browser.selected, 0);
}

// r[verify n8nclone.tui_ux.workflow_browser.empty]
#[test]
fn workflow_browser_reports_empty_store() {
    let temp_dir = tempfile::tempdir().expect("temporary app directory");
    let mut app = State::new(test_config(&temp_dir));

    app.open_workflow_browser();

    assert!(app.is_workflow_browser_open());
    assert!(app.workflow_browser.names.is_empty());
    assert_eq!(app.workflow_browser.message.as_deref(), Some("No saved workflows available"),);
}

// r[verify n8nclone.tui_ux.workflow_browser.load]
#[test]
fn workflow_browser_loads_selected_workflow() {
    let temp_dir = tempfile::tempdir().expect("temporary app directory");
    let mut app = State::new(test_config(&temp_dir));
    app.workflow.meta.name = "saved".to_string();
    app.add_shell_node("Saved", "echo saved");
    app.save_workflow();
    app.workflow = Workflow::new("current", crate::engine::workflow::WorkflowTimestamp::epoch());
    app.sync_graph_state();

    app.open_workflow_browser();
    app.confirm_workflow_browser_selection();

    assert_eq!(app.workflow.meta.name, "saved");
    assert_eq!(app.workflow.graph.node_count(), 1);
    assert!(!app.is_workflow_browser_open());
    assert!(!app.has_unsaved_changes());
}

// r[verify n8nclone.tui_ux.workflow_browser.load_failure]
#[test]
fn workflow_browser_load_failure_preserves_current_workflow() {
    let temp_dir = tempfile::tempdir().expect("temporary app directory");
    let mut app = State::new(test_config(&temp_dir));
    app.workflow = Workflow::new("current", crate::engine::workflow::WorkflowTimestamp::epoch());
    app.sync_graph_state();

    assert!(!app.load_workflow_by_name("missing"));

    assert_eq!(app.workflow.meta.name, "current");
    assert_eq!(app.workflow.graph.node_count(), 0);
}

#[test]
fn exhausted_run_identity_is_reported_before_starting_background_work() {
    let directory = tempfile::tempdir().expect("app directory");
    let mut app = State::new(test_config(&directory));
    app.next_tui_run_id = u64::MAX;
    let workflow = app.workflow.clone();
    assert!(!app.start_workflow_snapshot(workflow, "must not start"));
    assert!(!app.run_state.active);
    assert!(app.active_run.is_none());
    assert_eq!(app.next_tui_run_id, u64::MAX);
    assert!(output_text_lines(&app).iter().any(|line| line.contains("identity range exhausted")));
}

// r[verify lattice.tui_ux.aggregate_node_deletion]
// r[verify lattice.tui_ux.aggregate_node_deletion.scenario.missing]
#[test]
fn delete_selected_nodes_clears_active_editors_and_rejects_stale_selection() {
    let temp_dir = tempfile::tempdir().expect("temporary app directory");
    let mut app = State::new(test_config(&temp_dir));
    app.add_shell_node("Target", "echo target");
    let target = app.selected_node_id().expect("target selected");
    app.add_shell_node("Retained", "echo retained");
    let retained = app.selected_node_id().expect("retained selected");

    app.select_node(target);
    assert!(app.start_property_edit(PropertyEditField::Command));
    app.open_authoring("all");
    // Re-open property edit alongside authoring to verify both active editors clear.
    app.property_edit = Some(PropertyEditState {
        node_id: target,
        field: PropertyEditField::Command,
        original: "echo target".to_string(),
        draft: "echo pending".to_string(),
        workflow_snapshot: crate::workflow_format::workflow_to_portable(&app.workflow),
        authoring_snapshot: crate::authoring::identity("lattice.tui-authoring-snapshot.v1", &app.workflow).ok(),
    });
    assert_eq!(app.authoring.as_ref().and_then(|state| state.node_id), Some(target.0));
    app.clear_workflow_dirty();

    assert_eq!(app.delete_selected_nodes(), 1);
    assert!(app.property_edit.is_none(), "active property edit for deleted node must clear");
    assert!(app.authoring.is_none(), "active authoring browser for deleted node must clear");
    assert!(app.workflow.graph.node(target).is_none());
    assert!(app.workflow.node_config(target).is_none());
    assert!(app.workflow.graph.node(retained).is_some());
    assert!(app.has_unsaved_changes());

    app.clear_workflow_dirty();
    app.select_node(retained);
    app.open_authoring("all");
    let before = serde_json::to_value(&app.workflow).expect("workflow snapshot");
    app.graph_state
        .selected
        .insert(crate::tui::graph::subwayrat_node_id(crate::engine::workflow::NodeId(u64::MAX - 1)));
    assert_eq!(app.delete_selected_nodes(), 0);
    assert!(!app.has_unsaved_changes(), "stale deletion must not mark workflow dirty");
    assert_eq!(serde_json::to_value(&app.workflow).expect("workflow snapshot"), before);
    assert_eq!(app.authoring.as_ref().and_then(|state| state.node_id), Some(retained.0));
}

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

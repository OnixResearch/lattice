use super::*;

// r[verify n8nclone.tui_ux.run_history.list]
// r[verify n8nclone.tui_ux.run_history.persist]
// r[verify n8nclone.tui_ux.run_history.persist_record]
// r[verify n8nclone.tui_ux.workspace]
#[tokio::test]
async fn completed_background_run_is_persisted_to_history() {
    let temp_dir = tempfile::tempdir().expect("temporary app directory");
    let mut app = State::new(test_config(&temp_dir));
    app.add_condition_node("Ok", ConditionNodeConfig {
        branches: vec![ConditionBranch {
            label: "always".to_string(),
            predicate: control_flow::ConditionPredicate::Always,
        }],
    });

    assert!(app.start_workflow_run());
    for _ in 0..RUN_EVENT_POLL_LIMIT {
        app.poll_run_events();
        if !app.run_state.active {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(RUN_EVENT_POLL_DELAY_MS)).await;
    }
    app.open_run_history();

    assert!(app.is_run_history_open());
    assert_eq!(app.run_history.records.len(), 1);
    assert_eq!(app.run_history.records[0].node_count, 1);
    assert_eq!(app.run_history.records[0].failures, 0);
}

#[tokio::test]
async fn canceled_background_run_is_persisted_to_history() {
    let temp_dir = tempfile::tempdir().expect("temporary app directory");
    let mut app = State::new(test_config(&temp_dir));
    app.add_shell_node("Long", "sleep 1");

    assert!(app.start_workflow_run());
    assert!(app.cancel_active_run());
    app.open_run_history();

    assert_eq!(app.run_history.records.len(), 1);
    assert_eq!(app.run_history.records[0].node_count, 1);
}

#[test]
fn run_history_empty_and_inspect_states_are_visible() {
    let temp_dir = tempfile::tempdir().expect("temporary app directory");
    let mut app = State::new(test_config(&temp_dir));

    app.open_run_history();
    assert_eq!(app.run_history.message.as_deref(), Some("No run history available"),);

    let record = RunRecord {
        run_id: RunId(7),
        owner: crate::store::RunOwner::Unscoped,
        workflow_name: "wf".to_string(),
        started_epoch_ms: 0,
        finished_epoch_ms: Some(0),
        node_count: 2,
        cache_hits: 1,
        failures: 0,
    };
    app.run_history = RunHistoryState::open(vec![record], None);
    app.inspect_selected_run_history();

    assert_eq!(app.run_history.detail.as_deref(), Some("No run details available"));
    assert!(app.run_history.detail_record.is_none());
}

// r[verify n8nclone.tui_ux.run_history.details.empty_error.storage]
#[test]
fn run_detail_storage_failure_keeps_history_open_and_reports_safe_error() {
    let temp_dir = tempfile::tempdir().expect("temporary app directory");
    let mut app = State::new(test_config(&temp_dir));
    let record = RunRecord {
        run_id: RunId(SNAPSHOT_RUN_ID),
        owner: crate::store::RunOwner::Unscoped,
        workflow_name: SNAPSHOT_WORKFLOW_NAME.to_string(),
        started_epoch_ms: 0,
        finished_epoch_ms: Some(0),
        node_count: TEMPLATE_PARAMETER_COUNT,
        cache_hits: 0,
        failures: 0,
    };
    app.run_history = RunHistoryState::open(vec![record], None);

    app.apply_run_detail_load(crate::tui::app::model::receipts::classify_run_detail_load(Err(
        RUN_DETAIL_STORAGE_ERROR.to_string(),
    )));

    assert!(app.run_history.open);
    assert_eq!(app.run_history.records.len(), TEMPLATE_PARAMETER_COUNT);
    assert!(app.run_history.detail_record.is_none());
    assert_eq!(app.run_history.message.as_deref(), Some("Run details failed: storage offline"));
    assert!(output_text_lines(&app).iter().any(|line| line.contains(RUN_DETAIL_STORAGE_ERROR)));
}

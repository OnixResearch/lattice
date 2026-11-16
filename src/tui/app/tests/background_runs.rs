use super::*;

#[test]
fn finish_workflow_run_logs_executed_workflow_snapshot() {
    let temp_dir = tempfile::tempdir().expect("temporary app directory");
    let mut app = State::new(test_config(&temp_dir));
    let mut run_workflow = Workflow::new(SNAPSHOT_WORKFLOW_NAME, crate::engine::workflow::WorkflowTimestamp::epoch());
    let node_id = run_workflow.add_shell_node(SNAPSHOT_NODE_LABEL, SNAPSHOT_COMMAND);
    let node = run_workflow.graph.node(node_id).expect("node exists");
    let stdout_port = node.output_ports[SHELL_STDOUT_OUTPUT_INDEX].id;
    let exit_port = node.output_ports[SHELL_EXIT_OUTPUT_INDEX].id;
    let mut outputs = HashMap::new();
    outputs.insert(stdout_port, crate::engine::value::Value::from_string(SNAPSHOT_STDOUT.to_string()));
    outputs.insert(exit_port, crate::engine::value::Value::from_exit_code(EXIT_CODE_SUCCESS));
    let step = StepResult {
        node_id,
        status: NodeStatus::Completed { outputs, cached: false },
        input_hash: ContentHash::from_bytes(SNAPSHOT_INPUT_HASH_BYTES),
    };

    app.workflow = run_workflow.clone();
    app.workflow.graph.node_mut(node_id).expect("current node exists").label = MUTATED_NODE_LABEL.to_string();
    app.finish_workflow_run(RunId(SNAPSHOT_RUN_ID), &run_workflow, vec![step], vec![node_id]);

    let lines = output_text_lines(&app);
    assert!(lines.iter().any(|line| line.contains(SNAPSHOT_NODE_LABEL)), "lines: {lines:?}");
    assert!(!lines.iter().any(|line| line.contains(MUTATED_NODE_LABEL)), "lines: {lines:?}");
    assert!(lines.iter().any(|line| line == "  │ stdout: snapshot output"), "lines: {lines:?}");
    assert_eq!(app.run_output_lines.len(), SNAPSHOT_OUTPUT_RECORD_COUNT);
    assert!(
        app.run_output_lines.iter().any(|record| record.run_id == RunId(SNAPSHOT_RUN_ID)
            && record.node_id == node_id
            && record.text == SNAPSHOT_STDOUT_TEXT),
        "records: {:?}",
        app.run_output_lines
    );
}

// r[verify n8nclone.tui_ux.background_runs.status]
#[tokio::test]
async fn background_run_starts_without_blocking() {
    let temp_dir = tempfile::tempdir().expect("temporary app directory");
    let mut app = State::new(test_config(&temp_dir));
    app.add_shell_node("Long", "sleep 1");
    let node_id = app.graph_state.focused.expect("focused node");

    assert!(app.start_workflow_run());

    assert!(app.run_state.active);
    assert_eq!(app.run_state.node_statuses.get(&node_id.0), Some(&NodeRunStatus::Pending),);
    assert!(app.cancel_active_run());
}

// r[verify n8nclone.tui_ux.background_runs.cancel]
#[tokio::test]
async fn cancel_active_run_marks_pending_nodes_skipped() {
    let temp_dir = tempfile::tempdir().expect("temporary app directory");
    let mut app = State::new(test_config(&temp_dir));
    app.add_shell_node("Long", "sleep 1");
    let node_id = app.graph_state.focused.expect("focused node");

    assert!(app.start_workflow_run());
    assert!(app.cancel_active_run());

    assert!(!app.run_state.active);
    assert_eq!(app.run_state.node_statuses.get(&node_id.0), Some(&NodeRunStatus::Skipped),);
}

// r[verify n8nclone.tui_ux.background_runs.failure]
#[tokio::test]
async fn failed_background_run_marks_node_failed() {
    let temp_dir = tempfile::tempdir().expect("temporary app directory");
    let mut app = State::new(test_config(&temp_dir));
    let node_id = app.workflow.add_shell_node("Bad cwd", "echo no");
    app.workflow.set_node_config(node_id, NodeConfig::Shell {
        command: "echo no".to_string(),
        cwd: Some(temp_dir.path().join("missing").display().to_string()),
        env: HashMap::new(),
        timeout_secs: DEFAULT_SHELL_TIMEOUT_SECS,
        pipe_stdin: true,
    });
    app.sync_graph_state();
    app.select_node(node_id);

    assert!(app.start_workflow_run());
    for _ in 0..RUN_EVENT_POLL_LIMIT {
        app.poll_run_events();
        if !app.run_state.active {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(RUN_EVENT_POLL_DELAY_MS)).await;
    }

    assert!(!app.run_state.active);
    assert_eq!(app.run_state.node_statuses.get(&node_id.0), Some(&NodeRunStatus::Failed),);
}

// r[verify n8nclone.workflow_execution.image_input_nodes.production_io.tui]
#[tokio::test]
async fn tui_background_run_loads_local_image_with_default_executor() {
    let temp_dir = tempfile::tempdir().expect("temporary app directory");
    let image_path = temp_dir.path().join(TUI_LOCAL_IMAGE_FILE_NAME);
    std::fs::write(&image_path, TUI_LOCAL_PNG_FIXTURE).expect("TUI image fixture written");
    let mut app = State::new(test_config(&temp_dir));
    app.add_image_input_node("Local image", image_path.display().to_string());
    let node_id = app.graph_state.focused.expect("image node focused");

    assert!(app.start_workflow_run());
    drain_run(&mut app).await;

    assert!(!app.run_state.active);
    assert_eq!(app.run_state.node_statuses.get(&node_id.0), Some(&NodeRunStatus::Completed),);
    assert!(output_text_lines(&app).iter().any(|line| line.contains("image/png image")));
}

// r[verify n8nclone.tui_ux.agent_nodes.errors]
// r[verify n8nclone.tui_ux.agent_nodes.errors.missing_path]
// r[verify n8nclone.tui_ux.agent_nodes.errors.unsupported_format]
#[tokio::test]
async fn image_node_error_feedback_is_visible_after_run_failure() {
    let temp_dir = tempfile::tempdir().expect("temporary app directory");
    let mut missing_app = State::new(test_config(&temp_dir));
    missing_app.set_executor_factory(image_executor_factory(HashMap::new()));
    missing_app.add_image_input_node("Missing image", TUI_MISSING_IMAGE_PATH);
    let missing_node_id = missing_app.graph_state.focused.expect("missing image node focused");

    assert!(missing_app.start_workflow_run());
    drain_run(&mut missing_app).await;

    assert_eq!(missing_app.run_state.node_statuses.get(&missing_node_id.0), Some(&NodeRunStatus::Failed),);
    assert!(output_text_lines(&missing_app).iter().any(|line| line.contains("missing image")));

    let mut invalid_app = State::new(test_config(&temp_dir));
    invalid_app.set_executor_factory(image_executor_factory(
        [(TUI_BAD_IMAGE_PATH.to_string(), TUI_BAD_IMAGE_BYTES.to_vec())].into(),
    ));
    invalid_app.add_image_input_node("Bad image", TUI_BAD_IMAGE_PATH);
    let invalid_node_id = invalid_app.graph_state.focused.expect("invalid image node focused");

    assert!(invalid_app.start_workflow_run());
    drain_run(&mut invalid_app).await;

    assert_eq!(invalid_app.run_state.node_statuses.get(&invalid_node_id.0), Some(&NodeRunStatus::Failed),);
    assert!(output_text_lines(&invalid_app).iter().any(|line| line.contains("unsupported image format")));
}

// r[verify n8nclone.tui_ux.agent_nodes.errors]
// r[verify n8nclone.tui_ux.agent_nodes.errors.adapter]
#[tokio::test]
async fn agent_error_feedback_is_visible_after_run_failure() {
    let temp_dir = tempfile::tempdir().expect("temporary app directory");
    let mut app = State::new(test_config(&temp_dir));
    app.set_executor_factory(agent_error_executor_factory(Error::Adapter("runtime unavailable".to_string())));
    app.add_agent_node("Pi agent", crate::agent_node::Target::Pi, "summarize", crate::agent_node::OutputMode::Text);
    let authority = operator_authority(&app.workflow, temp_dir.path());
    app = app.with_authority_provider(std::sync::Arc::new(authority));
    let node_id = app.graph_state.focused.expect("focused node");

    assert!(app.start_workflow_run());
    drain_run(&mut app).await;

    assert!(!app.run_state.active);
    assert_eq!(app.run_state.node_statuses.get(&node_id.0), Some(&NodeRunStatus::Failed),);
    let lines = output_text_lines(&app);
    assert!(lines.iter().any(|line| line.contains("effect dispatch failed: AdapterRejected")), "{lines:?}");
    assert!(
        !lines.iter().any(|line| line.contains("runtime unavailable")),
        "raw adapter detail crossed the dispatch boundary: {lines:?}"
    );
    drop(app);

    let mut preflight_app = State::new(test_config(&temp_dir));
    preflight_app
        .set_executor_factory(agent_error_executor_factory(Error::PreflightRejected("model unavailable".to_string())));
    preflight_app.add_agent_node(
        "Preflight agent",
        crate::agent_node::Target::Clankers,
        "summarize",
        crate::agent_node::OutputMode::Text,
    );
    let preflight_authority = operator_authority(&preflight_app.workflow, temp_dir.path());
    preflight_app = preflight_app.with_authority_provider(std::sync::Arc::new(preflight_authority));
    let preflight_node_id = preflight_app.graph_state.focused.expect("preflight node focused");

    assert!(preflight_app.start_workflow_run());
    drain_run(&mut preflight_app).await;

    assert_eq!(preflight_app.run_state.node_statuses.get(&preflight_node_id.0), Some(&NodeRunStatus::Failed),);
    let preflight_lines = output_text_lines(&preflight_app);
    assert!(
        preflight_lines.iter().any(|line| line.contains("effect dispatch failed: AdapterRejected")),
        "{preflight_lines:?}"
    );
    assert!(
        !preflight_lines.iter().any(|line| line.contains("model unavailable")),
        "raw preflight detail crossed the dispatch boundary: {preflight_lines:?}"
    );
}

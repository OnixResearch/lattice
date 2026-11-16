//! Rows-specific live observation and selection regression coverage.

use super::*;
use crate::engine::executor::NodeProgress;

fn app() -> (tempfile::TempDir, State) {
    let directory = tempfile::tempdir().expect("app directory");
    let app = State::new(AppConfig {
        data_dir: directory.path().join("data"),
        ..AppConfig::default()
    });
    (directory, app)
}

fn active_observation_fixture(app: &mut State) -> mpsc::Sender<crate::tui::app::model::live::RunEvent> {
    let (sender, receiver) = mpsc::channel(RUN_EVENT_CAPACITY);
    app.run_state = RunState::idle();
    app.run_state.active = true;
    app.active_run = Some(crate::tui::app::model::live::ActiveRun {
        run_id: RunId(7),
        workflow: app.workflow.clone(),
        receiver,
        handle: tokio::spawn(async {}),
        cancellation: Arc::new(std::sync::atomic::AtomicBool::new(false)),
    });
    sender
}

// r[verify lattice.tui_ux.rows.metadata]
#[tokio::test]
async fn live_result_observation_updates_metadata_before_run_finishes() {
    let (_directory, mut app) = app();
    let id = app.workflow.add_agent_node(
        "observed",
        crate::agent_node::Target::Clankers,
        "inspect",
        crate::agent_node::OutputMode::Text,
    );
    let node = app.workflow.graph.node(id).expect("node");
    let output_port = node.output_ports.iter().find(|port| port.label != "metadata").expect("output").id;
    let metadata_port = node.output_ports.iter().find(|port| port.label == "metadata").expect("metadata").id;
    let step = StepResult {
        node_id: id,
        input_hash: crate::engine::ContentHash::from_bytes(b"live"),
        status: NodeStatus::Completed {
            cached: false,
            outputs: HashMap::from([
                (output_port, crate::engine::value::Value::from_string("live output".to_string())),
                (
                    metadata_port,
                    crate::engine::value::Value::from_json(
                        serde_json::json!({"usage": {"total_tokens": 123, "cost_microunits": 456}}),
                    ),
                ),
            ]),
        },
    };
    let sender = active_observation_fixture(&mut app);
    sender
        .try_send(crate::tui::app::model::live::RunEvent::NodeProgress(NodeProgress::Started { node_id: id }))
        .expect("started");
    app.poll_run_events();
    assert_eq!(app.run_state.node_statuses.get(&id.0), Some(&NodeRunStatus::Running));
    assert!(app.run_state.node_started.contains_key(&id.0));
    assert_eq!(app.run_state.node_start_times[&id.0].len(), 8);
    assert_eq!(app.run_state.node_attempts[&id.0], 1);

    sender
        .try_send(crate::tui::app::model::live::RunEvent::NodeProgress(NodeProgress::Finished {
            node_id: id,
            duration_millis: Some(42),
            result: step.clone(),
        }))
        .expect("finished");
    app.poll_run_events();
    assert!(app.run_state.active, "node observations must not terminate the run");
    assert_eq!(app.run_state.node_statuses[&id.0], NodeRunStatus::Completed);
    assert_eq!(app.run_state.node_durations_millis[&id.0], 42);
    assert!(!app.run_state.node_started.contains_key(&id.0));
    assert!(app.run_state.node_start_times.contains_key(&id.0));
    assert_eq!(app.run_state.node_tokens[&id.0], 123);
    assert_eq!(app.run_state.node_cost_microunits[&id.0], 456);
    assert!(app.run_output_lines.iter().any(|line| line.text == "live output"));
    let output_count = app.run_output_lines.len();
    sender
        .try_send(crate::tui::app::model::live::RunEvent::Completed {
            run_id: RunId(7),
            workflow: Box::new(app.workflow.clone()),
            results: vec![step],
            observed_completion_order: vec![id],
        })
        .expect("run complete");
    app.poll_run_events();
    assert!(!app.run_state.active);
    assert_eq!(app.run_output_lines.len(), output_count, "final receipt must not duplicate live output");
}

// r[verify lattice.tui_ux.rows.metadata]
#[tokio::test]
async fn live_failure_and_skip_are_never_reported_as_completed_or_attempted() {
    let (_directory, mut app) = app();
    let failed = app.workflow.add_shell_node("failed", "false");
    let skipped = app.workflow.add_shell_node("skipped", "true");
    let sender = active_observation_fixture(&mut app);
    sender
        .try_send(crate::tui::app::model::live::RunEvent::NodeProgress(NodeProgress::Started { node_id: failed }))
        .expect("start");
    for (id, status) in [
        (failed, NodeStatus::Failed {
            error: "Unknown: effect observation lost".to_string(),
        }),
        (skipped, NodeStatus::Skipped),
    ] {
        sender
            .try_send(crate::tui::app::model::live::RunEvent::NodeProgress(NodeProgress::Finished {
                node_id: id,
                duration_millis: (id == failed).then_some(12),
                result: StepResult {
                    node_id: id,
                    status,
                    input_hash: crate::engine::ContentHash::from_bytes(b"failure"),
                },
            }))
            .expect("result");
    }
    app.poll_run_events();
    assert!(app.run_state.active);
    assert_eq!(app.run_state.node_statuses[&failed.0], NodeRunStatus::Failed);
    assert_eq!(app.run_state.node_statuses[&skipped.0], NodeRunStatus::Skipped);
    assert_eq!(app.run_state.node_attempts[&failed.0], 1);
    assert!(!app.run_state.node_attempts.contains_key(&skipped.0));
    assert!(app.run_output_lines.is_empty());
    assert!(app.run_state.node_started.is_empty());
    assert!(!app.run_state.node_durations_millis.contains_key(&skipped.0));
}

// r[verify lattice.tui_ux.rows.selection]
#[test]
fn toggling_preserves_selected_identity_and_cancels_hidden_graph_gestures() {
    let (_directory, mut app) = app();
    app.add_shell_node("one", "true");
    app.add_shell_node("two", "true");
    let selected = app.graph_state.selected.clone();
    let focused = app.graph_state.focused;
    app.last_drag_canvas = Some((1, 2));
    app.toggle_canvas_view();
    assert_eq!(app.graph_state.selected, selected);
    assert_eq!(app.graph_state.focused, focused);
    assert_eq!(app.last_drag_canvas, None);
    app.select_row_boundary(true);
    let rows_selection = app.graph_state.selected.clone();
    app.toggle_canvas_view();
    assert_eq!(app.graph_state.selected, rows_selection);
    app.graph_state.focused = None;
    app.graph_state.selected.clear();
    app.select_row(1);
    let first = crate::tui::rows::layout(&app.workflow, 1).expect("valid workflow row layout").rows[0].node_id;
    assert_eq!(app.graph_state.focused, Some(crate::tui::graph::subwayrat_node_id(first)));
}

// r[verify lattice.tui_ux.rows.metadata]
#[tokio::test]
async fn real_executor_progress_preserves_completed_failed_and_skipped_results() {
    use crate::control_flow::ConditionBranch;
    use crate::control_flow::ConditionNodeConfig;
    use crate::control_flow::ConditionPredicate;
    use crate::control_flow::SubworkflowNodeConfig;
    use crate::control_flow::SubworkflowOutputMapping;

    let mut workflow = Workflow::new("rows-producer", crate::engine::workflow::WorkflowTimestamp::epoch());
    let condition = || ConditionNodeConfig {
        branches: vec![ConditionBranch {
            label: "match".to_string(),
            predicate: ConditionPredicate::StringEquals("ready".to_string()),
        }],
    };
    let completed = workflow.add_condition_node("no match completes", condition());
    // Recursive subworkflow admission fails without invoking a child or effect.
    let failed = workflow.add_subworkflow_node("recursive admission failure", SubworkflowNodeConfig {
        workflow_name: "rows-producer".to_string(),
        input_mappings: vec![],
        output_mappings: vec![SubworkflowOutputMapping {
            child_output: "stdout".to_string(),
            parent_output: "result".to_string(),
        }],
    });
    let skipped = workflow.add_condition_node("failed dependency", condition());
    let source = workflow.graph.node(failed).expect("source").output_ports[0].id;
    let target = workflow.graph.node(skipped).expect("target").input_ports[0].id;
    workflow.graph.add_edge(source, target).expect("dependency edge");
    let (sender, mut receiver) = crate::engine::executor::progress::channel(&workflow).expect("progress capacity");
    let mut executor = Executor::new().with_progress_sender(sender);
    let (_, results) = executor.run(&workflow, &HashMap::new()).await;
    let mut observed = HashMap::new();
    let mut started = std::collections::BTreeSet::new();
    while let Ok(progress) = receiver.try_recv() {
        match progress {
            NodeProgress::Started { node_id } => {
                started.insert(node_id);
            }
            NodeProgress::Finished { node_id, result, .. } => {
                observed.insert(node_id, result);
            }
        }
    }
    assert_eq!(observed.len(), results.len());
    assert!(matches!(observed[&completed].status, NodeStatus::Completed { .. }));
    assert!(matches!(observed[&failed].status, NodeStatus::Failed { .. }));
    assert!(matches!(observed[&skipped].status, NodeStatus::Skipped));
    assert!(!started.contains(&skipped));
    for result in results {
        assert_eq!(
            serde_json::to_value(&observed[&result.node_id].status).expect("observed status"),
            serde_json::to_value(&result.status).expect("final status")
        );
    }
}

#[tokio::test]
async fn lagging_ui_backpressures_completion_without_losing_observations() {
    let (_directory, mut app) = app();
    let node_id = app.workflow.add_shell_node("observed", "true");
    let sender = active_observation_fixture(&mut app);
    for _ in 0..RUN_EVENT_CAPACITY {
        sender
            .try_send(crate::tui::app::model::live::RunEvent::NodeProgress(NodeProgress::Started { node_id }))
            .expect("bounded observation slot");
    }
    assert_eq!(sender.capacity(), 0);
    let workflow = app.workflow.clone();
    let completion = tokio::spawn(async move {
        sender
            .send(crate::tui::app::model::live::RunEvent::Completed {
                run_id: RunId(7),
                workflow: Box::new(workflow),
                results: vec![StepResult {
                    node_id,
                    status: NodeStatus::Skipped,
                    input_hash: crate::engine::ContentHash::from_bytes(b"lagging-ui"),
                }],
                observed_completion_order: vec![node_id],
            })
            .await
    });
    app.poll_run_events();
    assert!(app.run_state.active);
    assert_eq!(app.run_state.node_attempts[&node_id.0], u32::try_from(RUN_EVENT_CAPACITY).expect("attempt count"));
    completion.await.expect("completion task").expect("completion retained");
    app.poll_run_events();
    assert!(!app.run_state.active);
    assert_eq!(app.run_state.node_statuses[&node_id.0], NodeRunStatus::Skipped);
}

#[tokio::test]
async fn incomplete_observations_do_not_leave_final_duration_running() {
    let (_directory, mut app) = app();
    let node_id = app.workflow.add_shell_node("observed", "true");
    let sender = active_observation_fixture(&mut app);
    sender
        .try_send(crate::tui::app::model::live::RunEvent::NodeProgress(NodeProgress::Started { node_id }))
        .expect("start");
    sender
        .try_send(crate::tui::app::model::live::RunEvent::ObservationIncomplete)
        .expect("incomplete diagnostic");
    app.poll_run_events();
    assert!(app.run_state.active, "observation faults must not cancel or retry execution");
    assert!(app.run_state.node_started.is_empty());
    assert!(!app.run_state.node_durations_millis.contains_key(&node_id.0));
}

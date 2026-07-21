//! Tests for detailed run-history persistence and inspection.

mod common;

use std::collections::HashMap;

use lattice::config::AppConfig;
use lattice::engine::executor::Executor;
use lattice::engine::executor::NodeStatus;
use lattice::engine::executor::RunId;
use lattice::engine::executor::StepResult;
use lattice::engine::value::ContentHash;
use lattice::engine::value::Value;
use lattice::engine::workflow::Workflow;
use lattice::run::history::detail_record_from_results;
use lattice::store::RunDetailRecord;
use lattice::store::RunNodeDetail;
use lattice::store::RunNodeDetailStatus;
use lattice::store::RunOutputDetail;
use lattice::store::RunOwner;
use lattice::store::RunRecord;
use lattice::tui::App;
use lattice::tui::app::run_detail_record_from_results as tui_run_detail_record_from_results;
use pretty_assertions::assert_eq;
use rat_nodegraph::model::NodeId;
use rat_nodegraph::model::PortId;
use tempfile::TempDir;

const RUN_ID_VALUE: u64 = 31;
const SECOND_RUN_ID_VALUE: u64 = 32;
const WORKFLOW_NAME: &str = "history-workflow";
const CURRENT_WORKFLOW_NAME: &str = "current-workflow";
const NODE_LABEL: &str = "history-node";
const NODE_COMMAND: &str = "printf history-output";
const OUTPUT_LABEL: &str = "stdout";
const OUTPUT_TEXT: &str = "history-output";
const STARTED_EPOCH_MS: i64 = 1_000;
const FINISHED_EPOCH_MS: i64 = 2_000;
const SINGLE_NODE_COUNT: usize = 1;
const NO_CACHE_HITS: usize = 0;
const NO_FAILURES: usize = 0;
const SENSITIVE_OUTPUT: &str = "live-secret-output";
const REDACTED_OUTPUT: &str = "[redacted]";
const INPUT_HASH_SEED: &[u8] = b"run-history-detail-input";

fn test_config(temp_dir: &TempDir) -> AppConfig {
    let data_dir = temp_dir.path().join("data");
    std::fs::create_dir_all(&data_dir).expect("create app data directory");
    AppConfig {
        data_dir,
        ..AppConfig::default()
    }
}

fn run_record(run_id: RunId, workflow_name: &str) -> RunRecord {
    RunRecord {
        run_id,
        owner: RunOwner::Unscoped,
        workflow_name: workflow_name.to_string(),
        started_epoch_ms: STARTED_EPOCH_MS,
        finished_epoch_ms: Some(FINISHED_EPOCH_MS),
        node_count: SINGLE_NODE_COUNT,
        cache_hits: NO_CACHE_HITS,
        failures: NO_FAILURES,
    }
}

fn detail_record(run_id: RunId) -> RunDetailRecord {
    RunDetailRecord {
        run_id,
        nodes: vec![RunNodeDetail {
            node_id: NodeId(RUN_ID_VALUE),
            node_label: NODE_LABEL.to_string(),
            status: RunNodeDetailStatus::Completed,
            cached: false,
            outputs: vec![RunOutputDetail {
                port_id: PortId(RUN_ID_VALUE),
                port_label: OUTPUT_LABEL.to_string(),
                text: OUTPUT_TEXT.to_string(),
            }],
            error: None,
        }],
    }
}

// r[verify n8nclone.tui_ux.run_history.details.persist]
#[test]
fn store_saves_and_loads_run_details() {
    let (store, _dir) = common::temp_store();
    let run_id = RunId(RUN_ID_VALUE);
    let detail = detail_record(run_id);

    store.save_run_details(&detail).expect("details save");
    let loaded = store.load_run_details(run_id).expect("details load");

    assert_eq!(loaded, Some(detail));
}

// r[verify n8nclone.tui_ux.run_history.details.empty_error]
#[test]
fn store_missing_run_details_returns_none() {
    let (store, _dir) = common::temp_store();

    let loaded = store.load_run_details(RunId(RUN_ID_VALUE)).expect("details load");

    assert!(loaded.is_none());
}

// r[verify n8nclone.tui_ux.run_history.details.persist]
// r[verify n8nclone.tui_ux.run_history.node_outputs]
#[tokio::test]
async fn run_detail_record_captures_node_outputs() {
    let (workflow, node_id) = common::standalone_node(NODE_LABEL, NODE_COMMAND);
    let mut executor = Executor::new();
    let (_executor_run_id, results) = executor.run(&workflow, &std::collections::HashMap::new()).await;

    let detail = detail_record_from_results(RunId(RUN_ID_VALUE), &workflow, &results);
    let tui_detail = tui_run_detail_record_from_results(RunId(RUN_ID_VALUE), &workflow, &results);

    assert_eq!(detail, tui_detail);
    assert_eq!(detail.nodes.len(), SINGLE_NODE_COUNT);
    assert_eq!(detail.nodes[0].node_id, node_id);
    assert_eq!(detail.nodes[0].status, RunNodeDetailStatus::Completed);
    assert!(detail.nodes[0].outputs.iter().any(|output| output.text == OUTPUT_TEXT));
}

// r[verify lattice.workflow_execution.headless_run_details]
// r[verify lattice.workflow_execution.headless_run_details.scenario.redaction]
#[test]
fn run_detail_record_replaces_sensitive_values_before_persistence() {
    let (workflow, node_id) = common::standalone_node(NODE_LABEL, NODE_COMMAND);
    let output_port = workflow.graph.node(node_id).expect("node must exist").output_ports[0].id;
    let results = vec![StepResult {
        node_id,
        status: NodeStatus::Completed {
            outputs: HashMap::from([(output_port, Value::from_sensitive_string(SENSITIVE_OUTPUT.to_string()))]),
            cached: false,
        },
        input_hash: ContentHash::from_bytes(INPUT_HASH_SEED),
    }];

    let detail = detail_record_from_results(RunId(RUN_ID_VALUE), &workflow, &results);
    let rendered = serde_json::to_string(&detail).expect("detail must serialize");

    assert_eq!(detail.nodes.len(), SINGLE_NODE_COUNT);
    assert_eq!(detail.nodes[0].outputs.len(), SINGLE_NODE_COUNT);
    assert_eq!(detail.nodes[0].outputs[0].text, REDACTED_OUTPUT);
    assert!(!rendered.contains(SENSITIVE_OUTPUT));
}

// r[verify n8nclone.tui_ux.run_history.timeline]
// r[verify n8nclone.tui_ux.run_history.node_outputs]
#[test]
fn app_inspects_run_history_details() {
    let temp_dir = tempfile::tempdir().expect("temporary app directory");
    let mut app = App::new(test_config(&temp_dir));
    let run_id = RunId(RUN_ID_VALUE);
    {
        let store = app.store.as_ref().expect("app store available");
        store.save_run(&run_record(run_id, WORKFLOW_NAME)).expect("run saves");
        store.save_run_details(&detail_record(run_id)).expect("details save");
    }

    app.open_run_history();
    app.inspect_selected_run_history();

    assert!(app.run_history.detail_record.is_some());
    assert_eq!(app.run_history.detail_record.as_ref().unwrap().nodes[0].node_label, NODE_LABEL);
}

// r[verify n8nclone.tui_ux.run_history.details.empty_error]
#[test]
fn app_reports_missing_run_history_details() {
    let temp_dir = tempfile::tempdir().expect("temporary app directory");
    let mut app = App::new(test_config(&temp_dir));
    let run_id = RunId(RUN_ID_VALUE);
    {
        let store = app.store.as_ref().expect("app store available");
        store.save_run(&run_record(run_id, WORKFLOW_NAME)).expect("run saves");
    }

    app.open_run_history();
    app.inspect_selected_run_history();

    assert!(app.run_history.detail_record.is_none());
    assert_eq!(app.run_history.message.as_deref(), Some("No run details available"));
}

// r[verify n8nclone.tui_ux.run_history.rerun]
#[tokio::test]
async fn app_reruns_history_workflow_without_replacing_current_workflow() {
    let temp_dir = tempfile::tempdir().expect("temporary app directory");
    let mut app = App::new(test_config(&temp_dir));
    let mut saved_workflow = Workflow::new(WORKFLOW_NAME);
    saved_workflow.add_shell_node(NODE_LABEL, NODE_COMMAND);
    {
        let store = app.store.as_ref().expect("app store available");
        store.save_workflow(&saved_workflow).expect("workflow saves");
        store.save_run(&run_record(RunId(SECOND_RUN_ID_VALUE), WORKFLOW_NAME)).expect("run saves");
    }
    app.workflow = Workflow::new(CURRENT_WORKFLOW_NAME);
    app.mark_workflow_dirty();

    app.open_run_history();
    assert!(app.rerun_selected_run_history());

    assert_eq!(app.workflow.meta.name, CURRENT_WORKFLOW_NAME);
    assert!(app.has_unsaved_changes());
    assert!(app.run_state.active);
    assert!(app.cancel_active_run());
}

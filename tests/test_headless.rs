//! Tests for headless workflow runs.

use std::process::Command;

use n8nclone::engine::executor::RunId;
use n8nclone::engine::workflow::Workflow;
use n8nclone::headless::HeadlessProcessResult;
use n8nclone::headless::HeadlessRunError;
use n8nclone::headless::HeadlessRunStatus;
use n8nclone::headless::process_result_for_summary;
use n8nclone::headless::render_run_summary_text;
use n8nclone::headless::run_saved_workflow;
use n8nclone::store::Store;
use rat_nodegraph::model::PortSpec;

const SUCCESS_WORKFLOW_NAME: &str = "headless-success";
const MISSING_WORKFLOW_NAME: &str = "missing-workflow";
const BROKEN_WORKFLOW_NAME: &str = "headless-broken";
const SUCCESS_NODE_LABEL: &str = "say hello";
const SUCCESS_COMMAND: &str = "printf hello";
const BROKEN_NODE_LABEL: &str = "broken";
const STRING_TYPE_TAG: &str = "string";
const SUCCESS_RUN_ID: u64 = 7;
const MISSING_RUN_ID: u64 = 11;
const FAILED_RUN_ID: u64 = 13;
const RECENT_RUN_LIMIT: usize = 8;
const SINGLE_NODE_COUNT: usize = 1;
const NO_FAILURES: usize = 0;
const ONE_FAILURE: usize = 1;
const DEFAULT_TIMEOUT_SECS: u64 = 60;
const MAX_PARALLELISM: usize = 4;
const CONFIG_FILE_NAME: &str = "config.ncl";
const STORE_FILE_NAME: &str = "n8nclone.redb";

fn temp_store() -> (Store, tempfile::TempDir) {
    let temp_dir = tempfile::tempdir().expect("temporary store directory");
    let store = Store::new(temp_dir.path().join("test.redb")).expect("store opens");
    (store, temp_dir)
}

fn write_test_config(temp_dir: &tempfile::TempDir) -> (std::path::PathBuf, std::path::PathBuf) {
    let data_dir = temp_dir.path().join("data");
    std::fs::create_dir_all(&data_dir).expect("data directory created");
    let config_path = temp_dir.path().join(CONFIG_FILE_NAME);
    let data_dir_string = data_dir.display().to_string();
    std::fs::write(
        &config_path,
        format!(
            "{{\n  data_dir = {:?},\n  default_timeout_secs = {DEFAULT_TIMEOUT_SECS},\n  shell = \"sh\",\n  max_parallelism = {MAX_PARALLELISM},\n}}\n",
            data_dir_string,
        ),
    )
    .expect("config written");
    (config_path, data_dir)
}

// r[verify n8nclone.workflow_execution.headless.run]
// r[verify n8nclone.workflow_execution.headless.summary]
// r[verify n8nclone.workflow_execution.headless.exit_status]
// r[verify n8nclone.workflow_execution.headless.history]
#[tokio::test]
async fn headless_run_saved_workflow_succeeds_and_persists_history() {
    let (store, _temp_dir) = temp_store();
    let mut workflow = Workflow::new(SUCCESS_WORKFLOW_NAME);
    workflow.add_shell_node(SUCCESS_NODE_LABEL, SUCCESS_COMMAND);
    store.save_workflow(&workflow).expect("workflow saves");

    let summary = run_saved_workflow(&store, SUCCESS_WORKFLOW_NAME, RunId(SUCCESS_RUN_ID))
        .await
        .expect("headless run succeeds");

    assert_eq!(summary.run_id, RunId(SUCCESS_RUN_ID));
    assert_eq!(summary.workflow_name, SUCCESS_WORKFLOW_NAME);
    assert_eq!(summary.status, HeadlessRunStatus::Succeeded);
    assert_eq!(summary.node_count, SINGLE_NODE_COUNT);
    assert_eq!(summary.failures, NO_FAILURES);
    assert_eq!(process_result_for_summary(&summary), HeadlessProcessResult::Success);
    assert!(render_run_summary_text(&summary).contains("succeeded"));

    let records = store.list_runs(RECENT_RUN_LIMIT).expect("run history lists");
    assert_eq!(records.len(), SINGLE_NODE_COUNT);
    assert_eq!(records[0].run_id, RunId(SUCCESS_RUN_ID));
    assert_eq!(records[0].failures, NO_FAILURES);
}

// r[verify n8nclone.workflow_execution.headless.run]
// r[verify n8nclone.workflow_execution.headless.summary]
// r[verify n8nclone.workflow_execution.headless.exit_status]
#[test]
fn cli_run_saved_workflow_emits_json_summary() {
    let temp_dir = tempfile::tempdir().expect("temporary cli directory");
    let (config_path, data_dir) = write_test_config(&temp_dir);
    let store = Store::new(data_dir.join(STORE_FILE_NAME)).expect("store opens");
    let mut workflow = Workflow::new(SUCCESS_WORKFLOW_NAME);
    workflow.add_shell_node(SUCCESS_NODE_LABEL, SUCCESS_COMMAND);
    store.save_workflow(&workflow).expect("workflow saves");
    drop(store);

    let output = Command::new(env!("CARGO_BIN_EXE_n8nclone"))
        .arg("--config")
        .arg(&config_path)
        .arg("run")
        .arg(SUCCESS_WORKFLOW_NAME)
        .arg("--output")
        .arg("json")
        .output()
        .expect("cli run executes");

    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let summary: n8nclone::headless::HeadlessRunSummary =
        serde_json::from_slice(&output.stdout).expect("json summary parses");
    assert_eq!(summary.workflow_name, SUCCESS_WORKFLOW_NAME);
    assert_eq!(summary.status, HeadlessRunStatus::Succeeded);
    assert_eq!(summary.failures, NO_FAILURES);
}

// r[verify n8nclone.workflow_execution.headless.errors]
#[tokio::test]
async fn headless_run_missing_workflow_fails_without_history_record() {
    let (store, _temp_dir) = temp_store();

    let error = run_saved_workflow(&store, MISSING_WORKFLOW_NAME, RunId(MISSING_RUN_ID))
        .await
        .expect_err("missing workflow fails");

    assert_eq!(error, HeadlessRunError::WorkflowNotFound(MISSING_WORKFLOW_NAME.to_string()));
    let records = store.list_runs(RECENT_RUN_LIMIT).expect("run history lists");
    assert!(records.is_empty());
}

// r[verify n8nclone.workflow_execution.headless.exit_status]
// r[verify n8nclone.workflow_execution.headless.errors]
#[tokio::test]
async fn headless_run_failed_node_returns_failure_summary() {
    let (store, _temp_dir) = temp_store();
    let mut workflow = Workflow::new(BROKEN_WORKFLOW_NAME);
    workflow
        .graph
        .add_node(BROKEN_NODE_LABEL, &[PortSpec::new("stdin", STRING_TYPE_TAG)], &[PortSpec::new(
            "stdout",
            STRING_TYPE_TAG,
        )]);
    store.save_workflow(&workflow).expect("workflow saves");

    let summary = run_saved_workflow(&store, BROKEN_WORKFLOW_NAME, RunId(FAILED_RUN_ID))
        .await
        .expect("headless run produces summary");

    assert_eq!(summary.status, HeadlessRunStatus::Failed);
    assert_eq!(summary.node_count, SINGLE_NODE_COUNT);
    assert_eq!(summary.failures, ONE_FAILURE);
    assert_eq!(process_result_for_summary(&summary), HeadlessProcessResult::Failure);

    let records = store.list_runs(RECENT_RUN_LIMIT).expect("run history lists");
    assert_eq!(records.len(), SINGLE_NODE_COUNT);
    assert_eq!(records[0].run_id, RunId(FAILED_RUN_ID));
    assert_eq!(records[0].failures, ONE_FAILURE);
}

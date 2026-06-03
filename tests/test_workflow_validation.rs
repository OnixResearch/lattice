//! Tests for pure workflow validation and run gates.

mod common;

use n8nclone::config::AppConfig;
use n8nclone::engine::executor::RunId;
use n8nclone::engine::workflow::Workflow;
use n8nclone::headless::HeadlessRunError;
use n8nclone::headless::run_saved_workflow;
use n8nclone::store::Store;
use n8nclone::tui::App;
use n8nclone::workflow_validation::DiagnosticSeverity;
use n8nclone::workflow_validation::WorkflowDiagnosticCode;
use n8nclone::workflow_validation::has_fatal_diagnostics;
use n8nclone::workflow_validation::validate_workflow;
use pretty_assertions::assert_eq;
use rat_nodegraph::model::PortSpec;
use serde_json::json;
use tempfile::TempDir;

const EMPTY_COMMAND: &str = "   ";
const VALID_COMMAND: &str = "printf ok";
const VALID_WORKFLOW_NAME: &str = "valid";
const INVALID_WORKFLOW_NAME: &str = "invalid";
const BROKEN_NODE_LABEL: &str = "broken";
const TYPE_TAG_STRING: &str = "string";
const INVALID_PORT_ID: u64 = 880_001;
const HEADLESS_RUN_ID: u64 = 21;
const SINGLE_ITEM_COUNT: usize = 1;

fn test_config(temp_dir: &TempDir) -> AppConfig {
    let data_dir = temp_dir.path().join("data");
    std::fs::create_dir_all(&data_dir).expect("create app data directory");
    AppConfig {
        data_dir,
        ..AppConfig::default()
    }
}

fn workflow_with_missing_config() -> Workflow {
    let mut workflow = Workflow::new(INVALID_WORKFLOW_NAME);
    workflow
        .graph
        .add_node(BROKEN_NODE_LABEL, &[PortSpec::new("stdin", TYPE_TAG_STRING)], &[PortSpec::new(
            "stdout",
            TYPE_TAG_STRING,
        )]);
    workflow
}

fn workflow_with_invalid_edge() -> Workflow {
    let (workflow, _source, _target) = common::two_node_pipeline();
    let mut value = serde_json::to_value(&workflow).expect("workflow serializes");
    value["graph"]["edges"] = json!([{ "source": INVALID_PORT_ID, "target": INVALID_PORT_ID }]);
    serde_json::from_value(value).expect("invalid edge fixture deserializes")
}

// r[verify n8nclone.workflow_validation.diagnostics.core]
#[test]
fn valid_workflow_has_no_fatal_diagnostics() {
    let mut workflow = Workflow::new(VALID_WORKFLOW_NAME);
    workflow.add_shell_node("valid", VALID_COMMAND);

    let diagnostics = validate_workflow(&workflow);

    assert!(!has_fatal_diagnostics(&diagnostics));
}

// r[verify n8nclone.workflow_validation.run_gate]
#[test]
fn warning_only_workflow_does_not_block_run_gate() {
    let workflow = Workflow::new(VALID_WORKFLOW_NAME);

    let diagnostics = validate_workflow(&workflow);

    assert_eq!(diagnostics.len(), SINGLE_ITEM_COUNT);
    assert_eq!(diagnostics[0].severity, DiagnosticSeverity::Warning);
    assert!(!has_fatal_diagnostics(&diagnostics));
}

// r[verify n8nclone.workflow_validation.diagnostics.core]
// r[verify n8nclone.workflow_validation.diagnostics.negative]
#[test]
fn empty_shell_command_is_fatal() {
    let mut workflow = Workflow::new(INVALID_WORKFLOW_NAME);
    workflow.add_shell_node(BROKEN_NODE_LABEL, EMPTY_COMMAND);

    let diagnostics = validate_workflow(&workflow);

    assert!(has_fatal_diagnostics(&diagnostics));
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.code == WorkflowDiagnosticCode::EmptyShellCommand && diagnostic.severity == DiagnosticSeverity::Fatal
    }));
}

// r[verify n8nclone.workflow_validation.diagnostics.core]
// r[verify n8nclone.workflow_validation.diagnostics.negative]
#[test]
fn missing_node_config_is_fatal() {
    let workflow = workflow_with_missing_config();

    let diagnostics = validate_workflow(&workflow);

    assert!(has_fatal_diagnostics(&diagnostics));
    assert!(diagnostics.iter().any(|diagnostic| diagnostic.code == WorkflowDiagnosticCode::MissingNodeConfig));
}

// r[verify n8nclone.workflow_validation.diagnostics.negative]
#[test]
fn invalid_edge_reference_is_fatal_and_does_not_panic() {
    let workflow = workflow_with_invalid_edge();

    let diagnostics = validate_workflow(&workflow);

    assert!(has_fatal_diagnostics(&diagnostics));
    assert!(diagnostics.iter().any(|diagnostic| diagnostic.code == WorkflowDiagnosticCode::EdgePortMissing));
}

// r[verify n8nclone.workflow_validation.run_gate]
// r[verify n8nclone.workflow_validation.tui.panel]
#[test]
fn tui_run_gate_opens_diagnostics_on_fatal_validation() {
    let temp_dir = tempfile::tempdir().expect("temporary app directory");
    let mut app = App::new(test_config(&temp_dir));
    app.add_shell_node(BROKEN_NODE_LABEL, EMPTY_COMMAND);

    assert!(!app.start_workflow_run());

    assert!(app.is_diagnostics_open());
    assert!(has_fatal_diagnostics(&app.diagnostics.diagnostics));
}

// r[verify n8nclone.workflow_validation.tui.panel]
#[test]
fn diagnostics_panel_opens_with_ready_state_for_valid_workflow() {
    let temp_dir = tempfile::tempdir().expect("temporary app directory");
    let mut app = App::new(test_config(&temp_dir));
    app.add_shell_node("valid", VALID_COMMAND);

    app.open_diagnostics();

    assert!(app.is_diagnostics_open());
    assert!(app.diagnostics.diagnostics.is_empty());
}

// r[verify n8nclone.workflow_validation.run_gate]
#[tokio::test]
async fn headless_run_gate_rejects_invalid_workflow_without_history() {
    let temp_dir = tempfile::tempdir().expect("temporary store directory");
    let store = Store::new(temp_dir.path().join("test.redb")).expect("store opens");
    let workflow = workflow_with_missing_config();
    store.save_workflow(&workflow).expect("workflow saves");

    let error = run_saved_workflow(&store, INVALID_WORKFLOW_NAME, RunId(HEADLESS_RUN_ID))
        .await
        .expect_err("validation blocks headless run");

    assert!(matches!(error, HeadlessRunError::Validation(_)));
    assert!(store.list_runs(SINGLE_ITEM_COUNT).expect("runs list").is_empty());
}

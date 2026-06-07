//! Tests for pure workflow validation and run gates.

mod common;

use n8nclone::config::AppConfig;
use n8nclone::engine::executor::RunId;
use n8nclone::engine::workflow::NodeRunnerSettings;
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
const SHELL_STDIN_INPUT_INDEX: usize = 0;
const MISSING_RUNTIME_KIND: &str = "missing-kind";
const MISSING_RUNTIME_KEY: &str = "missing-key";

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

fn workflow_with_cycle() -> Workflow {
    let (workflow, source, target) = common::two_node_pipeline();
    let mut value = serde_json::to_value(&workflow).expect("workflow serializes");
    let source_input = workflow.graph.node(source).expect("source node").input_ports[SHELL_STDIN_INPUT_INDEX].id.0;
    let target_stdout = common::stdout_port_id(&workflow, target).0;
    value["graph"]["edges"].as_array_mut().expect("edges array").push(json!({
        "source": target_stdout,
        "target": source_input,
    }));
    serde_json::from_value(value).expect("cyclic workflow fixture deserializes")
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
// r[verify n8nclone.workflow_validation.node_kind_provider.fallback]
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

// r[verify n8nclone.workflow_validation.generic_runner_admission]
#[test]
fn runner_cycle_admission_is_fatal_validation_diagnostic() {
    let workflow = workflow_with_cycle();

    let diagnostics = validate_workflow(&workflow);

    assert!(has_fatal_diagnostics(&diagnostics));
    assert!(diagnostics.iter().any(|diagnostic| diagnostic.code == WorkflowDiagnosticCode::RunnerCycleDetected));
}

// r[verify n8nclone.workflow_validation.generic_runner_admission.runtime]
// r[verify n8nclone.workflow_validation.node_kind_provider.runtime_binding]
#[test]
fn missing_runtime_binding_is_fatal_validation_diagnostic() {
    let mut workflow = Workflow::new(INVALID_WORKFLOW_NAME);
    let node_id = workflow.add_shell_node(BROKEN_NODE_LABEL, VALID_COMMAND);
    workflow.set_node_runner_settings(node_id, NodeRunnerSettings::new(MISSING_RUNTIME_KIND, MISSING_RUNTIME_KEY));

    let diagnostics = validate_workflow(&workflow);

    assert!(has_fatal_diagnostics(&diagnostics));
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == WorkflowDiagnosticCode::RunnerMissingRuntimeBinding)
    );
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

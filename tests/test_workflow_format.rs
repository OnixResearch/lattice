//! Tests for typed Nickel workflow import/export.

mod common;

use std::process::Command;

use n8nclone::engine::workflow::NodeConfig;
use n8nclone::engine::workflow::Workflow;
use n8nclone::store::Store;
use n8nclone::workflow_format::PortableEdge;
use n8nclone::workflow_format::WorkflowFormatError;
use n8nclone::workflow_format::load_workflow_nickel_file;
use n8nclone::workflow_format::portable_to_workflow;
use n8nclone::workflow_format::workflow_to_nickel;
use n8nclone::workflow_format::workflow_to_portable;
use pretty_assertions::assert_eq;

const CONFIG_FILE_NAME: &str = "config.ncl";
const STORE_FILE_NAME: &str = "n8nclone.redb";
const EXPORTED_WORKFLOW_FILE_NAME: &str = "workflow.ncl";
const IMPORTED_WORKFLOW_FILE_NAME: &str = "imported.ncl";
const MISSING_WORKFLOW_NAME: &str = "missing-workflow";
const DEFAULT_TIMEOUT_SECS: u64 = 60;
const MAX_PARALLELISM: usize = 4;
const IMPORTED_NODE_COUNT: usize = 2;
const IMPORTED_EDGE_COUNT: usize = 1;
const INVALID_PORT_ID: u64 = 999_001;

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

fn exported_then_loaded(workflow: &Workflow) -> Workflow {
    let temp_dir = tempfile::tempdir().expect("temporary workflow format directory");
    let workflow_path = temp_dir.path().join(EXPORTED_WORKFLOW_FILE_NAME);
    std::fs::write(&workflow_path, workflow_to_nickel(workflow)).expect("workflow Nickel written");
    load_workflow_nickel_file(&workflow_path).expect("workflow Nickel loads")
}

// r[verify n8nclone.workflow_format.nickel.export]
// r[verify n8nclone.workflow_format.nickel.import]
// r[verify n8nclone.workflow_format.nickel.round_trip]
#[test]
fn nickel_export_import_preserves_pipeline_shape_and_config() {
    let (workflow, _source, target) = common::two_node_pipeline();

    let imported = exported_then_loaded(&workflow);

    assert_eq!(imported.meta.name, workflow.meta.name);
    assert_eq!(imported.graph.node_count(), IMPORTED_NODE_COUNT);
    assert_eq!(imported.graph.edge_count(), IMPORTED_EDGE_COUNT);
    let target_node = imported.graph.nodes().find(|node| node.label == "grep-hello").expect("target node imported");
    match imported.node_config(target_node.id).expect("target config imported") {
        NodeConfig::Shell { command, .. } => assert_eq!(command, "grep hello"),
    }
    assert!(workflow.graph.node(target).is_some());
}

// r[verify n8nclone.workflow_format.nickel.export]
// r[verify n8nclone.workflow_format.nickel.import]
#[test]
fn cli_export_then_import_saves_workflow() {
    let temp_dir = tempfile::tempdir().expect("temporary cli directory");
    let (config_path, data_dir) = write_test_config(&temp_dir);
    let store = Store::new(data_dir.join(STORE_FILE_NAME)).expect("store opens");
    let (workflow, _source, _target) = common::two_node_pipeline();
    store.save_workflow(&workflow).expect("workflow saves");
    drop(store);

    let export_path = temp_dir.path().join(EXPORTED_WORKFLOW_FILE_NAME);
    let export_output = Command::new(env!("CARGO_BIN_EXE_n8nclone"))
        .arg("--config")
        .arg(&config_path)
        .arg("export")
        .arg("test-pipeline")
        .arg(&export_path)
        .output()
        .expect("export command runs");
    assert!(export_output.status.success(), "stderr: {}", String::from_utf8_lossy(&export_output.stderr));

    let import_path = temp_dir.path().join(IMPORTED_WORKFLOW_FILE_NAME);
    std::fs::copy(&export_path, &import_path).expect("copy exported workflow");
    let import_output = Command::new(env!("CARGO_BIN_EXE_n8nclone"))
        .arg("--config")
        .arg(&config_path)
        .arg("import")
        .arg(&import_path)
        .output()
        .expect("import command runs");
    assert!(import_output.status.success(), "stderr: {}", String::from_utf8_lossy(&import_output.stderr));

    let store = Store::new(data_dir.join(STORE_FILE_NAME)).expect("store reopens");
    let loaded = store.load_workflow("test-pipeline").expect("load imported workflow");
    assert!(loaded.is_some());
}

// r[verify n8nclone.workflow_format.nickel.export]
#[test]
fn cli_export_missing_workflow_writes_no_file() {
    let temp_dir = tempfile::tempdir().expect("temporary cli directory");
    let (config_path, _data_dir) = write_test_config(&temp_dir);
    let export_path = temp_dir.path().join(EXPORTED_WORKFLOW_FILE_NAME);

    let output = Command::new(env!("CARGO_BIN_EXE_n8nclone"))
        .arg("--config")
        .arg(&config_path)
        .arg("export")
        .arg(MISSING_WORKFLOW_NAME)
        .arg(&export_path)
        .output()
        .expect("export command runs");

    assert!(!output.status.success());
    assert!(!export_path.exists());
}

// r[verify n8nclone.workflow_format.nickel.invalid]
#[test]
fn nickel_import_rejects_missing_required_field() {
    let temp_dir = tempfile::tempdir().expect("temporary workflow format directory");
    let workflow_path = temp_dir.path().join(EXPORTED_WORKFLOW_FILE_NAME);
    std::fs::write(&workflow_path, "{ meta = { name = \"broken\" } }\n").expect("broken Nickel written");

    let error = load_workflow_nickel_file(&workflow_path).expect_err("missing fields rejected");

    assert!(matches!(error, WorkflowFormatError::Json(_)));
}

// r[verify n8nclone.workflow_format.nickel.invalid]
#[test]
fn portable_import_rejects_invalid_edge_reference() {
    let (workflow, _source, _target) = common::two_node_pipeline();
    let mut portable = workflow_to_portable(&workflow);
    portable.edges.push(PortableEdge {
        source: INVALID_PORT_ID,
        target: INVALID_PORT_ID,
    });

    let error = portable_to_workflow(portable).expect_err("invalid edge rejected");

    assert_eq!(error, WorkflowFormatError::UnknownEdgePort(INVALID_PORT_ID));
}

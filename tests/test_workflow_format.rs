//! Tests for typed Nickel workflow import/export.

mod common;

use std::process::Command;

use n8nclone::agent_node::AgentOutputMode;
use n8nclone::agent_node::AgentTarget;
use n8nclone::engine::runner::build_runner_plan;
use n8nclone::engine::workflow::NodeConfig;
use n8nclone::engine::workflow::NodeRunnerSettings;
use n8nclone::engine::workflow::Workflow;
use n8nclone::store::Store;
use n8nclone::workflow_format::PortableEdge;
use n8nclone::workflow_format::WorkflowFormatError;
use n8nclone::workflow_format::load_workflow_nickel_file;
use n8nclone::workflow_format::portable_to_workflow;
use n8nclone::workflow_format::workflow_to_nickel;
use n8nclone::workflow_format::workflow_to_portable;
use n8nclone::workflow_validation::validate_workflow;
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
const IMAGE_AGENT_NODE_COUNT: usize = 2;
const IMAGE_AGENT_EDGE_COUNT: usize = 1;
const INVALID_PORT_ID: u64 = 999_001;
const IMAGE_PATH: &str = "/tmp/selfie.png";
const STATIC_MISSING_IMAGE_FILE_NAME: &str = "missing-static-image.png";
const AGENT_PROMPT: &str = "Turn this selfie into fine art";
const CUSTOM_RUNTIME_KIND: &str = "shell";
const CUSTOM_RUNTIME_KEY: &str = "custom-shell";
const EFFECT_POLICY: &str = "process";
const CACHE_POLICY: &str = "inputs-and-config";
const RETRY_POLICY: &str = "never";
const ARTIFACT_POLICY: &str = "inline-small";

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

fn custom_runner_settings() -> NodeRunnerSettings {
    NodeRunnerSettings {
        runtime_kind: CUSTOM_RUNTIME_KIND.to_string(),
        runtime_key: CUSTOM_RUNTIME_KEY.to_string(),
        effect_policy: EFFECT_POLICY.to_string(),
        cache_policy: CACHE_POLICY.to_string(),
        retry_policy: RETRY_POLICY.to_string(),
        artifact_policy: ARTIFACT_POLICY.to_string(),
    }
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
        _ => panic!("expected shell config"),
    }
    assert!(workflow.graph.node(target).is_some());
}

// r[verify n8nclone.workflow_format.generic_pipeline_ir.nickel]
// r[verify n8nclone.workflow_format.generic_pipeline_ir.import]
#[test]
fn nickel_export_import_preserves_runner_metadata() {
    let (mut workflow, source, _target) = common::two_node_pipeline();
    workflow.set_node_runner_settings(source, custom_runner_settings());

    let portable = workflow_to_portable(&workflow);
    let config = portable.configs.iter().find(|config| config.node_id == source.0).expect("source config exported");
    assert_eq!(config.runtime_kind, CUSTOM_RUNTIME_KIND);
    assert_eq!(config.runtime_key, CUSTOM_RUNTIME_KEY);
    assert_eq!(config.effect_policy, EFFECT_POLICY);
    assert_eq!(config.cache_policy, CACHE_POLICY);
    assert_eq!(config.retry_policy, RETRY_POLICY);
    assert_eq!(config.artifact_policy, ARTIFACT_POLICY);

    let imported = exported_then_loaded(&workflow);
    let plan = build_runner_plan(&imported).plan.expect("imported workflow builds runner plan");
    let imported_source = plan.nodes.iter().find(|node| node.label == "echo-hello").expect("source node imported");
    assert_eq!(imported_source.runtime.kind, CUSTOM_RUNTIME_KIND);
    assert_eq!(imported_source.runtime.key, CUSTOM_RUNTIME_KEY);
    assert_eq!(imported_source.policy.effect_policy, EFFECT_POLICY);
}

// r[verify n8nclone.workflow_format.generic_pipeline_ir.compatibility]
#[test]
fn portable_import_defaults_empty_runner_metadata_for_older_workflows() {
    let (workflow, _source, _target) = common::two_node_pipeline();
    let mut portable = workflow_to_portable(&workflow);
    for config in &mut portable.configs {
        config.runtime_kind.clear();
        config.runtime_key.clear();
    }

    let imported = portable_to_workflow(portable).expect("older portable workflow imports");
    let diagnostics = validate_workflow(&imported);

    assert!(diagnostics.is_empty());
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
// r[verify n8nclone.workflow_format.agent_nodes.nickel]
#[test]
fn nickel_export_import_preserves_image_input_and_generic_agent_node() {
    let mut workflow = Workflow::new("image-agent-workflow");
    let image_id = workflow.add_image_input_node("Selfie", IMAGE_PATH);
    let agent_id =
        workflow.add_agent_node("Fine art agent", AgentTarget::Clankers, AGENT_PROMPT, AgentOutputMode::Image);
    let image_out = workflow.graph.node(image_id).expect("image node").output_ports[0].id;
    let agent_image_in = workflow.graph.node(agent_id).expect("agent node").input_ports[1].id;
    workflow.graph.add_edge(image_out, agent_image_in).expect("connect image to agent");

    let imported = exported_then_loaded(&workflow);

    assert_eq!(imported.graph.node_count(), IMAGE_AGENT_NODE_COUNT);
    assert_eq!(imported.graph.edge_count(), IMAGE_AGENT_EDGE_COUNT);
    let image_node = imported.graph.nodes().find(|node| node.label == "Selfie").expect("image node imported");
    let agent_node = imported.graph.nodes().find(|node| node.label == "Fine art agent").expect("agent node imported");
    match imported.node_config(image_node.id).expect("image config imported") {
        NodeConfig::ImageInput { path, .. } => assert_eq!(path, IMAGE_PATH),
        _ => panic!("expected image input config"),
    }
    match imported.node_config(agent_node.id).expect("agent config imported") {
        NodeConfig::Agent(settings) => {
            assert_eq!(settings.target, AgentTarget::Clankers);
            assert_eq!(settings.prompt, AGENT_PROMPT);
            assert_eq!(settings.output_mode, AgentOutputMode::Image);
        }
        _ => panic!("expected agent config"),
    }
}

// r[verify n8nclone.workflow_execution.image_input_nodes.production_io.no_import_io]
#[test]
fn image_workflow_static_import_export_and_validation_do_not_read_image_bytes() {
    let temp_dir = tempfile::tempdir().expect("temporary workflow format directory");
    let missing_path = temp_dir.path().join(STATIC_MISSING_IMAGE_FILE_NAME).display().to_string();
    let mut workflow = Workflow::new("static-image-workflow");
    let node_id = workflow.add_image_input_node("Missing static image", &missing_path);
    let workflow_path = temp_dir.path().join(EXPORTED_WORKFLOW_FILE_NAME);

    std::fs::write(&workflow_path, workflow_to_nickel(&workflow)).expect("workflow Nickel written");
    let imported =
        load_workflow_nickel_file(&workflow_path).expect("static image workflow imports without reading file");
    let diagnostics = validate_workflow(&imported);

    assert!(diagnostics.is_empty());
    match imported.node_config(node_id).expect("image config imported") {
        NodeConfig::ImageInput { path, .. } => assert_eq!(path, &missing_path),
        _ => panic!("expected image input config"),
    }
}

// r[verify n8nclone.workflow_format.node_kind_provider.handlers]
#[test]
fn portable_import_rejects_invalid_agent_target() {
    let mut workflow = Workflow::new("invalid-agent-workflow");
    workflow.add_agent_node("Agent", AgentTarget::Pi, AGENT_PROMPT, AgentOutputMode::Text);
    let mut portable = workflow_to_portable(&workflow);
    let config = portable.configs.first_mut().expect("agent config");
    config.agent_target = "unsupported".to_string();

    let error = portable_to_workflow(portable).expect_err("invalid agent target rejected");

    assert!(matches!(error, WorkflowFormatError::InvalidAgentTarget { .. }));
}

#[test]
fn portable_import_rejects_invalid_agent_output_mode() {
    let mut workflow = Workflow::new("invalid-agent-workflow");
    workflow.add_agent_node("Agent", AgentTarget::Pi, AGENT_PROMPT, AgentOutputMode::Text);
    let mut portable = workflow_to_portable(&workflow);
    let config = portable.configs.first_mut().expect("agent config");
    config.agent_output_mode = "unsupported".to_string();

    let error = portable_to_workflow(portable).expect_err("invalid agent output mode rejected");

    assert!(matches!(error, WorkflowFormatError::InvalidAgentOutputMode { .. }));
}

#[test]
fn portable_import_rejects_incompatible_image_agent_port() {
    let mut workflow = Workflow::new("invalid-image-port-workflow");
    let image_id = workflow.add_image_input_node("Selfie", IMAGE_PATH);
    let shell_id = workflow.add_shell_node("String consumer", "cat");
    let image_output = workflow.graph.node(image_id).expect("image node").output_ports[0].id;
    let shell_input = workflow.graph.node(shell_id).expect("shell node").input_ports[0].id;
    let mut portable = workflow_to_portable(&workflow);
    portable.edges.push(PortableEdge {
        source: image_output.0,
        target: shell_input.0,
    });

    let error = portable_to_workflow(portable).expect_err("incompatible image edge rejected");

    assert!(matches!(error, WorkflowFormatError::InvalidGraph(_)));
}

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

//! Tests for typed Nickel workflow import/export.

mod common;

use std::process::Command;

use lattice::agent_node::AgentOutputMode;
use lattice::agent_node::AgentTarget;
use lattice::control_flow::ConditionBranch;
use lattice::control_flow::ConditionNodeConfig;
use lattice::control_flow::ConditionPredicate;
use lattice::control_flow::MergeNodeConfig;
use lattice::control_flow::MergePolicy;
use lattice::control_flow::SubworkflowNodeConfig;
use lattice::credentials::NodeCredentialRefs;
use lattice::credentials::SecretRef;
use lattice::engine::runner::build_runner_plan;
use lattice::engine::workflow::NodeConfig;
use lattice::engine::workflow::NodeRunnerSettings;
use lattice::engine::workflow::Workflow;
use lattice::file_node::FileOperationKind;
use lattice::store::Store;
use lattice::workflow_format::MAX_MARKDOWN_AGENT_PROMPT_BYTES;
use lattice::workflow_format::PortableEdge;
use lattice::workflow_format::WorkflowFormatError;
use lattice::workflow_format::load_workflow_nickel_file;
use lattice::workflow_format::portable_to_workflow;
use lattice::workflow_format::workflow_to_nickel;
use lattice::workflow_format::workflow_to_portable;
use lattice::workflow_validation::validate_workflow;
use pretty_assertions::assert_eq;

const CONFIG_FILE_NAME: &str = "config.ncl";
const STORE_FILE_NAME: &str = "lattice.redb";
const EXPORTED_WORKFLOW_FILE_NAME: &str = "workflow.ncl";
const IMPORTED_WORKFLOW_FILE_NAME: &str = "imported.ncl";
const INVALID_IMAGE_WORKFLOW_PATH: &str = "examples/workflows/invalid-unsupported-image-format.ncl";
const INVALID_IMAGE_WORKFLOW_NAME: &str = "invalid-unsupported-image-format";
const EXISTING_WORKFLOW_NAME: &str = "existing-workflow";
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
const ROLE_PROMPT_PATH: &str = "roles/researcher.md";
const ROLE_PROMPT_BODY: &str = "# Researcher\n\nFind primary evidence and report uncertainty.\n";
const ROLE_PROMPT_LABEL: &str = "Researcher";
const OVERSIZED_PROMPT_EXTRA_BYTES: usize = 1;

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

fn markdown_agent_workflow(prompt_file: &str) -> Workflow {
    let mut workflow = Workflow::new("markdown-agent-workflow");
    let node_id = workflow.add_agent_node(ROLE_PROMPT_LABEL, AgentTarget::Pi, ROLE_PROMPT_BODY, AgentOutputMode::Text);
    let Some(NodeConfig::Agent(mut settings)) = workflow.node_config(node_id).cloned() else {
        panic!("agent config exists");
    };
    settings.prompt_file = Some(prompt_file.to_string());
    workflow.set_node_config(node_id, NodeConfig::Agent(settings));
    workflow
}

fn write_markdown_workflow(temp_dir: &tempfile::TempDir, prompt_file: &str) -> std::path::PathBuf {
    let workflow = markdown_agent_workflow(prompt_file);
    let workflow_path = temp_dir.path().join(EXPORTED_WORKFLOW_FILE_NAME);
    std::fs::write(&workflow_path, workflow_to_nickel(&workflow)).expect("Markdown workflow Nickel written");
    workflow_path
}

// r[verify n8nclone.workflow_format.nickel.export]
// r[verify n8nclone.workflow_format.nickel.import]
// r[verify n8nclone.workflow_format.nickel.round_trip]
// r[verify n8nclone.workflow_format.nickel.round_trip.preserve]
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

// r[verify n8nclone.workflow_format.generic_pipeline_ir]
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
// r[verify n8nclone.workflow_format.triggers.nickel.compatibility]
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
// r[verify n8nclone.workflow_format.nickel.export.saved]
// r[verify n8nclone.workflow_format.nickel.import.valid]
#[test]
fn cli_export_then_import_saves_workflow() {
    let temp_dir = tempfile::tempdir().expect("temporary cli directory");
    let (config_path, data_dir) = write_test_config(&temp_dir);
    let store = Store::new(data_dir.join(STORE_FILE_NAME)).expect("store opens");
    let (workflow, _source, _target) = common::two_node_pipeline();
    store.save_workflow(&workflow).expect("workflow saves");
    drop(store);

    let export_path = temp_dir.path().join(EXPORTED_WORKFLOW_FILE_NAME);
    let export_output = Command::new(env!("CARGO_BIN_EXE_lattice"))
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
    let import_output = Command::new(env!("CARGO_BIN_EXE_lattice"))
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
// r[verify n8nclone.workflow_format.nickel.export.missing]
#[test]
fn cli_export_missing_workflow_writes_no_file() {
    let temp_dir = tempfile::tempdir().expect("temporary cli directory");
    let (config_path, _data_dir) = write_test_config(&temp_dir);
    let export_path = temp_dir.path().join(EXPORTED_WORKFLOW_FILE_NAME);

    let output = Command::new(env!("CARGO_BIN_EXE_lattice"))
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
// r[verify n8nclone.workflow_format.agent_nodes.nickel.export]
// r[verify n8nclone.workflow_format.agent_nodes.nickel.import]
// r[verify n8nclone.workflow_format.agent_nodes.nickel.round_trip]
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

#[test]
fn nickel_export_import_preserves_agent_credential_references() {
    let mut workflow = Workflow::new("credentialed-agent-workflow");
    let agent_id = workflow.add_agent_node("Agent", AgentTarget::Pi, AGENT_PROMPT, AgentOutputMode::Text);
    let mut credentials = NodeCredentialRefs::default();
    credentials
        .env
        .insert("TOKEN".to_string(), SecretRef::new("env", "ARTIST_TOKEN", Some("access_token".to_string())));
    workflow.set_node_credentials(agent_id, credentials);

    let imported = exported_then_loaded(&workflow);
    let imported_agent = imported.graph.nodes().find(|node| node.label == "Agent").expect("agent imported");
    let imported_credentials = imported.node_credentials(imported_agent.id).expect("agent credentials imported");
    let secret_ref = imported_credentials.env.get("TOKEN").expect("token reference imported");

    assert_eq!(secret_ref.provider, "env");
    assert_eq!(secret_ref.key, "ARTIST_TOKEN");
    assert_eq!(secret_ref.field.as_deref(), Some("access_token"));
}

#[test]
fn nickel_round_trip_preserves_credentials_for_every_builtin_node_kind() {
    let mut workflow = Workflow::new("all-credentialed-kinds");
    let nodes = [
        ("Shell", workflow.add_shell_node("Shell", "true")),
        ("HTTP", workflow.add_http_node("HTTP", "GET", "https://example.com")),
        ("File", workflow.add_file_node("File", FileOperationKind::Read, "./input.txt")),
        ("Image", workflow.add_image_input_node("Image", "./input.png")),
        ("Agent", workflow.add_agent_node("Agent", AgentTarget::Pi, AGENT_PROMPT, AgentOutputMode::Text)),
        (
            "Condition",
            workflow.add_condition_node("Condition", ConditionNodeConfig {
                branches: vec![ConditionBranch {
                    label: "match".to_string(),
                    predicate: ConditionPredicate::Always,
                }],
            }),
        ),
        (
            "Merge",
            workflow.add_merge_node("Merge", MergeNodeConfig {
                policy: MergePolicy::All,
                required_inputs: vec!["input".to_string()],
            }),
        ),
        (
            "Subworkflow",
            workflow.add_subworkflow_node("Subworkflow", SubworkflowNodeConfig {
                workflow_name: "child".to_string(),
                input_mappings: Vec::new(),
                output_mappings: Vec::new(),
            }),
        ),
    ];
    for (label, node_id) in &nodes {
        let mut credentials = NodeCredentialRefs::default();
        credentials.env.insert("TOKEN".to_string(), SecretRef::new("env", format!("{label}_TOKEN"), None));
        workflow.set_node_credentials(*node_id, credentials);
    }

    let imported = exported_then_loaded(&workflow);

    for (label, _) in &nodes {
        let imported_node = imported.graph.nodes().find(|node| node.label == *label).expect("node imported");
        let credentials = imported.node_credentials(imported_node.id).expect("credentials imported");
        assert_eq!(credentials.env.get("TOKEN").expect("token reference").key, format!("{label}_TOKEN"));
    }
}

#[test]
fn aggregate_deletion_remains_valid_through_store_and_nickel_round_trips() {
    let temp_dir = tempfile::tempdir().expect("temporary deletion directory");
    let store = Store::new(temp_dir.path().join(STORE_FILE_NAME)).expect("store opens");
    let mut workflow = Workflow::new("deleted-node-round-trip");
    let removed = workflow.add_agent_node("Removed", AgentTarget::Pi, AGENT_PROMPT, AgentOutputMode::Text);
    let kept = workflow.add_shell_node("Kept", "true");
    let removed_output = workflow.graph.node(removed).expect("removed node").output_ports[0].id;
    let kept_input = workflow.graph.node(kept).expect("kept node").input_ports[0].id;
    workflow.graph.add_edge(removed_output, kept_input).expect("incident edge");
    let mut credentials = NodeCredentialRefs::default();
    credentials.env.insert("TOKEN".to_string(), SecretRef::new("env", "TOKEN", None));
    workflow.set_node_credentials(removed, credentials);
    workflow.set_node_runner_settings(removed, custom_runner_settings());

    workflow.remove_node(removed).expect("aggregate node removal");
    assert!(validate_workflow(&workflow).is_empty());
    store.save_workflow(&workflow).expect("cleaned workflow saves");
    let loaded = store
        .load_workflow("deleted-node-round-trip")
        .expect("cleaned workflow loads")
        .expect("saved workflow exists");
    let imported = exported_then_loaded(&loaded);

    assert_eq!(imported.graph.node_count(), 1);
    assert_eq!(imported.graph.edge_count(), 0);
    assert!(imported.graph.nodes().any(|node| node.label == "Kept"));
    assert!(!imported.graph.nodes().any(|node| node.label == "Removed"));
    assert_eq!(imported.configs.len(), imported.graph.node_count());
    assert!(imported.credentials.is_empty());
    assert!(
        imported
            .configs
            .keys()
            .all(|node_id| imported.graph.node(rat_nodegraph::model::NodeId(*node_id)).is_some())
    );
    assert!(
        imported
            .runner
            .keys()
            .all(|node_id| imported.graph.node(rat_nodegraph::model::NodeId(*node_id)).is_some())
    );
}

// r[verify n8nclone.workflow_format.markdown_agent_prompts.scenario.valid]
// r[verify n8nclone.workflow_format.markdown_agent_prompts.capability_read.scenario.success]
#[test]
fn markdown_agent_prompt_file_loads_and_round_trips_source() {
    let temp_dir = tempfile::tempdir().expect("temporary Markdown workflow directory");
    let role_path = temp_dir.path().join(ROLE_PROMPT_PATH);
    std::fs::create_dir_all(role_path.parent().expect("role parent directory")).expect("role directory created");
    std::fs::write(&role_path, ROLE_PROMPT_BODY).expect("role prompt written");
    let workflow_path = write_markdown_workflow(&temp_dir, ROLE_PROMPT_PATH);

    let imported = load_workflow_nickel_file(&workflow_path).expect("Markdown workflow imports");
    let node = imported.graph.nodes().find(|node| node.label == ROLE_PROMPT_LABEL).expect("role node imported");
    let NodeConfig::Agent(settings) = imported.node_config(node.id).expect("role config imported") else {
        panic!("expected agent config");
    };

    assert_eq!(settings.prompt, ROLE_PROMPT_BODY);
    assert_eq!(settings.prompt_file.as_deref(), Some(ROLE_PROMPT_PATH));
    let portable = workflow_to_portable(&imported);
    let config = portable.configs.first().expect("portable role config");
    assert!(config.agent_prompt.is_empty());
    assert_eq!(config.agent_prompt_file, vec![ROLE_PROMPT_PATH.to_string()]);
}

// r[verify n8nclone.workflow_format.markdown_agent_prompts.scenario.invalid]
// r[verify n8nclone.workflow_format.agent_nodes.invalid]
// r[verify n8nclone.workflow_format.agent_nodes.invalid.missing_prompt]
#[test]
fn markdown_agent_prompt_source_cardinality_fails_closed() {
    let workflow = markdown_agent_workflow(ROLE_PROMPT_PATH);

    let mut missing_source = workflow_to_portable(&workflow);
    let missing_config = missing_source.configs.first_mut().expect("missing-source config");
    missing_config.agent_prompt.clear();
    missing_config.agent_prompt_file.clear();
    assert!(matches!(
        portable_to_workflow(missing_source),
        Err(WorkflowFormatError::InvalidAgentPromptSource { .. })
    ));

    let mut multiple_sources = workflow_to_portable(&workflow);
    let multiple_config = multiple_sources.configs.first_mut().expect("multiple-source config");
    multiple_config.agent_prompt = ROLE_PROMPT_BODY.to_string();
    assert!(matches!(
        portable_to_workflow(multiple_sources),
        Err(WorkflowFormatError::InvalidAgentPromptSource { .. })
    ));

    let mut duplicate_files = workflow_to_portable(&workflow);
    duplicate_files.configs.first_mut().expect("duplicate-file config").agent_prompt_file =
        vec![ROLE_PROMPT_PATH.to_string(), ROLE_PROMPT_PATH.to_string()];
    assert!(matches!(
        portable_to_workflow(duplicate_files),
        Err(WorkflowFormatError::InvalidAgentPromptSource { .. })
    ));
}

// r[verify n8nclone.workflow_format.markdown_agent_prompts.scenario.invalid]
#[test]
fn unsafe_or_missing_markdown_agent_prompt_paths_fail_closed() {
    let path_cases = [
        "",
        "/tmp/absolute-role.md",
        "../escaping-role.md",
        "roles/not-markdown.txt",
    ];
    for prompt_file in path_cases {
        let temp_dir = tempfile::tempdir().expect("temporary invalid-path directory");
        let workflow_path = write_markdown_workflow(&temp_dir, prompt_file);
        assert!(matches!(
            load_workflow_nickel_file(&workflow_path),
            Err(WorkflowFormatError::InvalidAgentPromptSource { .. })
        ));
    }

    let temp_dir = tempfile::tempdir().expect("temporary missing-path directory");
    let workflow_path = write_markdown_workflow(&temp_dir, ROLE_PROMPT_PATH);
    assert!(matches!(
        load_workflow_nickel_file(&workflow_path),
        Err(WorkflowFormatError::AgentPromptFile { .. })
    ));
}

// r[verify n8nclone.workflow_format.markdown_agent_prompts.scenario.invalid]
#[test]
fn malformed_markdown_agent_prompt_content_fails_closed() {
    let temp_dir = tempfile::tempdir().expect("temporary malformed-prompt directory");
    let role_path = temp_dir.path().join(ROLE_PROMPT_PATH);
    std::fs::create_dir_all(role_path.parent().expect("role parent directory")).expect("role directory created");
    let workflow_path = write_markdown_workflow(&temp_dir, ROLE_PROMPT_PATH);

    std::fs::write(&role_path, "   \n").expect("empty role prompt written");
    assert!(matches!(
        load_workflow_nickel_file(&workflow_path),
        Err(WorkflowFormatError::AgentPromptFile { .. })
    ));

    let oversized_prompt = vec![b'x'; MAX_MARKDOWN_AGENT_PROMPT_BYTES + OVERSIZED_PROMPT_EXTRA_BYTES];
    std::fs::write(&role_path, oversized_prompt).expect("oversized role prompt written");
    assert!(matches!(
        load_workflow_nickel_file(&workflow_path),
        Err(WorkflowFormatError::AgentPromptFile { .. })
    ));

    std::fs::write(&role_path, [u8::MAX]).expect("invalid UTF-8 role prompt written");
    assert!(matches!(
        load_workflow_nickel_file(&workflow_path),
        Err(WorkflowFormatError::AgentPromptFile { .. })
    ));
}

#[cfg(unix)]
// r[verify n8nclone.workflow_format.markdown_agent_prompts.scenario.invalid]
// r[verify n8nclone.workflow_format.markdown_agent_prompts.capability_read.scenario.rejected]
#[test]
fn markdown_agent_prompt_symlink_escape_fails_closed() {
    use std::os::unix::fs::symlink;

    let temp_dir = tempfile::tempdir().expect("temporary symlink workflow directory");
    let outside_dir = tempfile::tempdir().expect("temporary outside directory");
    let outside_prompt = outside_dir.path().join("outside.md");
    std::fs::write(&outside_prompt, ROLE_PROMPT_BODY).expect("outside role prompt written");
    let role_path = temp_dir.path().join(ROLE_PROMPT_PATH);
    std::fs::create_dir_all(role_path.parent().expect("role parent directory")).expect("role directory created");
    symlink(&outside_prompt, &role_path).expect("escaping role symlink created");
    let workflow_path = write_markdown_workflow(&temp_dir, ROLE_PROMPT_PATH);

    assert!(matches!(
        load_workflow_nickel_file(&workflow_path),
        Err(WorkflowFormatError::AgentPromptFile { .. })
    ));
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

// r[verify n8nclone.workflow_format.agent_nodes.invalid]
// r[verify n8nclone.workflow_format.agent_nodes.invalid.image_format]
#[test]
fn cli_import_rejects_unsupported_declared_image_format_without_mutating_store() {
    let temp_dir = tempfile::tempdir().expect("temporary image import directory");
    let (config_path, data_dir) = write_test_config(&temp_dir);
    let store = Store::new(data_dir.join(STORE_FILE_NAME)).expect("store opens");
    store.save_workflow(&Workflow::new(EXISTING_WORKFLOW_NAME)).expect("existing workflow saves");
    drop(store);

    let output = Command::new(env!("CARGO_BIN_EXE_lattice"))
        .arg("--config")
        .arg(&config_path)
        .arg("import")
        .arg(INVALID_IMAGE_WORKFLOW_PATH)
        .output()
        .expect("invalid image import command runs");
    let stderr = String::from_utf8_lossy(&output.stderr);
    let store = Store::new(data_dir.join(STORE_FILE_NAME)).expect("store reopens");

    assert!(!output.status.success());
    assert!(stderr.contains("unsupported image format"), "stderr: {stderr}");
    assert!(store.load_workflow(EXISTING_WORKFLOW_NAME).expect("existing workflow loads").is_some());
    assert!(
        store
            .load_workflow(INVALID_IMAGE_WORKFLOW_NAME)
            .expect("invalid workflow lookup succeeds")
            .is_none()
    );
}

// r[verify n8nclone.workflow_format.node_kind_provider.handlers]
// r[verify n8nclone.workflow_format.agent_nodes.invalid]
// r[verify n8nclone.workflow_format.agent_nodes.invalid.target]
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

// r[verify n8nclone.workflow_format.agent_nodes.invalid]
// r[verify n8nclone.workflow_format.agent_nodes.invalid.port_type]
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

// r[verify n8nclone.workflow_format.nickel.import.invalid]
// r[verify n8nclone.workflow_format.nickel.invalid.missing_field]
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

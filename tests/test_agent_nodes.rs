//! Tests for generic agent operation nodes.

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::sync::Arc;

use n8nclone::agent_node::AgentAdapter;
use n8nclone::agent_node::AgentCommandOutput;
use n8nclone::agent_node::AgentCommandPlan;
use n8nclone::agent_node::AgentCommandRuntimeConfig;
use n8nclone::agent_node::AgentError;
use n8nclone::agent_node::AgentOutput;
use n8nclone::agent_node::AgentOutputMode;
use n8nclone::agent_node::AgentRequest;
use n8nclone::agent_node::AgentResponse;
use n8nclone::agent_node::AgentRuntimeConfig;
use n8nclone::agent_node::AgentTarget;
use n8nclone::agent_node::ProductionAgentAdapter;
use n8nclone::agent_node::decode_agent_command_output;
use n8nclone::agent_node::plan_agent_command;
use n8nclone::agent_node::render_agent_command_prompt;
use n8nclone::credentials::NodeCredentialRefs;
use n8nclone::credentials::SecretRef;
use n8nclone::engine::executor::Executor;
use n8nclone::engine::executor::NodeStatus;
use n8nclone::engine::executor::node_execution_cache_key;
use n8nclone::engine::value::ImageMetadata;
use n8nclone::engine::value::ImageValue;
use n8nclone::engine::value::TypeTag;
use n8nclone::engine::value::Value;
use n8nclone::engine::workflow::NodeConfig;
use n8nclone::engine::workflow::Workflow;
use n8nclone::workflow_validation::WorkflowDiagnosticCode;
use n8nclone::workflow_validation::validate_workflow;
use parking_lot::Mutex;
use pretty_assertions::assert_eq;

const AGENT_NODE_LABEL: &str = "agent";
const PROMPT_TEXT: &str = "Convert this selfie into fine art";
const PROMPT_TEXT_EDITED: &str = "Convert this selfie into ink art";
const TEXT_OUTPUT: &str = "done";
const IMAGE_MEDIA_TYPE: &str = "image/png";
const IMAGE_BYTES_A: &[u8] = b"\x89PNG\r\n\x1a\nagent-a";
const IMAGE_BYTES_B: &[u8] = b"\x89PNG\r\n\x1a\nagent-b";
const PROMPT_INPUT_INDEX: usize = 0;
const IMAGE_INPUT_INDEX: usize = 1;
const JSON_INPUT_INDEX: usize = 2;
const TEXT_OUTPUT_INDEX: usize = 0;
const IMAGE_OUTPUT_INDEX: usize = 2;
const METADATA_OUTPUT_INDEX: usize = 4;
const SECRET_ENV_NAME: &str = "AGENT_TOKEN";
const SECRET_PROVIDER: &str = "env";
const SECRET_KEY: &str = "agent_token";
const PI_RUNTIME_KEY: &str = "pi";
const CLANKERS_RUNTIME_KEY: &str = "clankers";
const PROMPT_FLAG: &str = "-p";
const PROMPT_ARG_TEMPLATE: &str = "{prompt}";
const MODEL_ARG_TEMPLATE: &str = "{model}";
const SESSION_ARG_TEMPLATE: &str = "{session}";
const OUTPUT_MODE_ARG_TEMPLATE: &str = "{output_mode}";
const FAKE_AGENT_FILE_NAME: &str = "agent-runtime.sh";
const SHELL_COMMAND: &str = "sh";
const RUNTIME_TEXT_OUTPUT: &str = "runtime text";
const RUNTIME_JSON_OUTPUT: &str = "{\"ok\":true}";
const INVALID_JSON_OUTPUT: &str = "not-json";
const HANDOFF_IMAGE_FILE_NAME: &str = "agent-output.png";
const HANDOFF_ARTIFACT_FILE_NAME: &str = "agent-output.bin";
const HANDOFF_MISSING_FILE_NAME: &str = "missing-output.png";
const HANDOFF_UNSUPPORTED_IMAGE_FILE_NAME: &str = "bad-output.img";
const HANDOFF_OUTSIDE_FILE_NAME: &str = "outside-output.png";
const HANDOFF_METADATA_LABEL: &str = "generated";
const HANDOFF_ARTIFACT_BYTES: &[u8] = b"artifact bytes";
const HANDOFF_BAD_IMAGE_BYTES: &[u8] = b"not an image";
const NONZERO_STDERR: &str = "runtime failed";
const MODEL_HINT: &str = "test-model";
const SESSION_ID: &str = "session-123";
const TOOL_POLICY: &str = "read-only";
const CONTEXT_REF: &str = "@context.md";
const EXECUTABLE_MODE: u32 = 0o755;
const NONZERO_EXIT_CODE: i32 = 17;
const SMALL_OUTPUT_LIMIT_BYTES: usize = 4;

#[derive(Clone)]
struct StaticAgentAdapter {
    response: AgentResponse,
    requests: Arc<Mutex<Vec<AgentRequest>>>,
}

impl StaticAgentAdapter {
    fn new(response: AgentResponse) -> Self {
        Self {
            response,
            requests: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn requests(&self) -> Vec<AgentRequest> {
        self.requests.lock().clone()
    }
}

impl AgentAdapter for StaticAgentAdapter {
    fn execute(&self, request: &AgentRequest) -> Result<AgentResponse, AgentError> {
        self.requests.lock().push(request.clone());
        Ok(self.response.clone())
    }
}

#[derive(Clone)]
struct ErrorAgentAdapter {
    error: AgentError,
    calls: Arc<Mutex<usize>>,
}

impl ErrorAgentAdapter {
    fn new(error: AgentError) -> Self {
        Self {
            error,
            calls: Arc::new(Mutex::new(0)),
        }
    }

    fn calls(&self) -> usize {
        *self.calls.lock()
    }
}

impl AgentAdapter for ErrorAgentAdapter {
    fn execute(&self, _request: &AgentRequest) -> Result<AgentResponse, AgentError> {
        *self.calls.lock() += 1;
        Err(self.error.clone())
    }
}

fn image_value(bytes: &[u8]) -> Value {
    Value::from_image(ImageValue::new(bytes.to_vec(), IMAGE_MEDIA_TYPE.to_string(), ImageMetadata::default()))
}

fn agent_request(target: AgentTarget, output_mode: AgentOutputMode) -> AgentRequest {
    AgentRequest {
        target,
        prompt: PROMPT_TEXT.to_string(),
        output_mode,
        model_hint: None,
        tool_policy: String::new(),
        context_refs: Vec::new(),
        cwd: None,
        session: None,
        text_inputs: Vec::new(),
        image_inputs: Vec::new(),
        json_inputs: Vec::new(),
    }
}

fn runtime_config(pi_runtime_command: Option<String>, clankers_runtime_command: Option<String>) -> AgentRuntimeConfig {
    let mut runtimes = BTreeMap::new();
    if let Some(command) = pi_runtime_command {
        runtimes.insert(PI_RUNTIME_KEY.to_string(), runtime_entry(command, &[PROMPT_FLAG, PROMPT_ARG_TEMPLATE]));
    }
    if let Some(command) = clankers_runtime_command {
        runtimes.insert(CLANKERS_RUNTIME_KEY.to_string(), runtime_entry(command, &[PROMPT_FLAG, PROMPT_ARG_TEMPLATE]));
    }
    AgentRuntimeConfig {
        runtimes,
        max_output_bytes: AgentRuntimeConfig::default().max_output_bytes,
    }
}

fn runtime_entry(command: impl Into<String>, args: &[&str]) -> AgentCommandRuntimeConfig {
    AgentCommandRuntimeConfig {
        command: command.into(),
        args: args.iter().map(|arg| (*arg).to_string()).collect(),
    }
}

#[cfg(unix)]
fn runtime_config_for_script(
    pi_runtime_script: Option<String>,
    clankers_runtime_script: Option<String>,
) -> AgentRuntimeConfig {
    let mut runtimes = BTreeMap::new();
    if let Some(script) = pi_runtime_script {
        runtimes.insert(PI_RUNTIME_KEY.to_string(), script_runtime_entry(script));
    }
    if let Some(script) = clankers_runtime_script {
        runtimes.insert(CLANKERS_RUNTIME_KEY.to_string(), script_runtime_entry(script));
    }
    AgentRuntimeConfig {
        runtimes,
        max_output_bytes: AgentRuntimeConfig::default().max_output_bytes,
    }
}

#[cfg(unix)]
fn script_runtime_entry(script: String) -> AgentCommandRuntimeConfig {
    AgentCommandRuntimeConfig {
        command: SHELL_COMMAND.to_string(),
        args: vec![script, PROMPT_FLAG.to_string(), PROMPT_ARG_TEMPLATE.to_string()],
    }
}

fn command_plan(target: AgentTarget, output_mode: AgentOutputMode) -> AgentCommandPlan {
    AgentCommandPlan {
        target,
        adapter_label: match target {
            AgentTarget::Pi => "pi",
            AgentTarget::Clankers => "clankers",
        },
        program: "agent".to_string(),
        args: Vec::new(),
        cwd: None,
        output_mode,
        max_output_bytes: AgentRuntimeConfig::default().max_output_bytes,
    }
}

#[cfg(unix)]
fn write_agent_script(temp_dir: &tempfile::TempDir, body: &str) -> String {
    let path = temp_dir.path().join(FAKE_AGENT_FILE_NAME);
    fs::write(&path, body).expect("fake agent script written");
    let mut permissions = fs::metadata(&path).expect("fake agent metadata").permissions();
    permissions.set_mode(EXECUTABLE_MODE);
    fs::set_permissions(&path, permissions).expect("fake agent made executable");
    path.display().to_string()
}

fn handoff_envelope(path: &str, media_type: Option<&str>) -> String {
    let envelope = serde_json::json!({
        "path": path,
        "media_type": media_type,
        "metadata": {"label": HANDOFF_METADATA_LABEL},
    });
    envelope.to_string()
}

// r[verify n8nclone.workflow_execution.agent_nodes.command_runtimes.config]
#[test]
fn default_agent_runtime_config_contains_command_presets() {
    let config = AgentRuntimeConfig::default();

    let pi = config.runtimes.get(PI_RUNTIME_KEY).expect("pi runtime preset");
    let clankers = config.runtimes.get(CLANKERS_RUNTIME_KEY).expect("clankers runtime preset");

    assert_eq!(pi.command, PI_RUNTIME_KEY);
    assert!(pi.args.iter().any(|arg| arg == PROMPT_ARG_TEMPLATE));
    assert_eq!(clankers.command, CLANKERS_RUNTIME_KEY);
    assert!(clankers.args.iter().any(|arg| arg == PROMPT_ARG_TEMPLATE));
}

// r[verify n8nclone.workflow_execution.agent_nodes.production_adapters.command]
// r[verify n8nclone.workflow_execution.agent_nodes.command_runtimes.planning]
#[test]
fn production_command_plan_uses_runtime_args_and_prompt_inputs() {
    let mut request = agent_request(AgentTarget::Pi, AgentOutputMode::Text);
    request.model_hint = Some(MODEL_HINT.to_string());
    request.session = Some(SESSION_ID.to_string());
    request.tool_policy = TOOL_POLICY.to_string();
    request.context_refs = vec![CONTEXT_REF.to_string()];
    request.text_inputs = vec!["extra prompt".to_string()];
    request.json_inputs = vec![serde_json::json!({"field": "value"})];
    let mut config = runtime_config(None, None);
    config.runtimes.insert(
        PI_RUNTIME_KEY.to_string(),
        runtime_entry("pi-test", &[
            "--model",
            MODEL_ARG_TEMPLATE,
            "--session",
            SESSION_ARG_TEMPLATE,
            "--mode",
            OUTPUT_MODE_ARG_TEMPLATE,
            "-p",
            PROMPT_ARG_TEMPLATE,
        ]),
    );

    let plan = plan_agent_command(&config, &request).expect("command plan builds");
    let prompt = render_agent_command_prompt(&request);

    assert_eq!(plan.program, "pi-test");
    assert!(plan.args.iter().any(|arg| arg == "--model"));
    assert!(plan.args.iter().any(|arg| arg == MODEL_HINT));
    assert!(plan.args.iter().any(|arg| arg == "--session"));
    assert!(plan.args.iter().any(|arg| arg == SESSION_ID));
    assert!(plan.args.iter().any(|arg| arg == "text"));
    assert!(plan.args.iter().any(|arg| arg == "-p"));
    assert!(prompt.contains(PROMPT_TEXT));
    assert!(prompt.contains("Text input"));
    assert!(prompt.contains("JSON input"));
    assert!(prompt.contains(TOOL_POLICY));
    assert!(prompt.contains(CONTEXT_REF));
}

// r[verify n8nclone.workflow_execution.agent_nodes.production_adapters.command]
// r[verify n8nclone.workflow_execution.agent_nodes.production_adapters.route]
#[cfg(unix)]
#[test]
fn production_pi_text_adapter_executes_command() {
    let temp_dir = tempfile::tempdir().expect("temporary fake agent directory");
    let command = write_agent_script(&temp_dir, &format!("#!/bin/sh\nprintf '{RUNTIME_TEXT_OUTPUT}'\n"));
    let adapter = ProductionAgentAdapter::new(runtime_config_for_script(Some(command), None));
    let request = agent_request(AgentTarget::Pi, AgentOutputMode::Text);

    let response = adapter.execute(&request).expect("production pi adapter runs");

    assert_eq!(response.output, AgentOutput::Text(RUNTIME_TEXT_OUTPUT.to_string()));
    assert_eq!(response.metadata["adapter"], "pi");
}

// r[verify n8nclone.workflow_execution.agent_nodes.production_adapters.command]
// r[verify n8nclone.workflow_execution.agent_nodes.production_adapters.route]
#[cfg(unix)]
#[test]
fn production_clankers_json_adapter_executes_command() {
    let temp_dir = tempfile::tempdir().expect("temporary fake agent directory");
    let command = write_agent_script(&temp_dir, &format!("#!/bin/sh\nprintf '{RUNTIME_JSON_OUTPUT}'\n"));
    let adapter = ProductionAgentAdapter::new(runtime_config_for_script(None, Some(command)));
    let request = agent_request(AgentTarget::Clankers, AgentOutputMode::Json);

    let response = adapter.execute(&request).expect("production clankers adapter runs");

    assert_eq!(response.output, AgentOutput::Json(serde_json::json!({"ok": true})));
    assert_eq!(response.metadata["adapter"], "clankers");
}

// r[verify n8nclone.workflow_execution.agent_nodes.production_adapters.errors]
// r[verify n8nclone.workflow_execution.agent_nodes.command_runtimes.errors]
#[test]
fn production_adapter_missing_runtime_rejects_before_spawn() {
    let config = runtime_config(None, None);
    let request = agent_request(AgentTarget::Pi, AgentOutputMode::Text);

    let error = plan_agent_command(&config, &request).expect_err("missing runtime rejected");

    assert!(error.to_string().contains("missing pi agent runtime configuration"), "error: {error}");
}

// r[verify n8nclone.workflow_execution.agent_nodes.production_adapters.errors]
// r[verify n8nclone.workflow_execution.agent_nodes.command_runtimes.errors]
#[test]
fn production_adapter_empty_command_rejects_before_spawn() {
    let config = runtime_config(Some("   ".to_string()), None);
    let request = agent_request(AgentTarget::Pi, AgentOutputMode::Text);

    let error = plan_agent_command(&config, &request).expect_err("empty command rejected");

    assert!(error.to_string().contains("missing pi agent runtime command"), "error: {error}");
}

// r[verify n8nclone.workflow_execution.agent_nodes.output_file_handoff.image]
#[cfg(unix)]
#[test]
fn production_image_handoff_loads_typed_image() {
    let temp_dir = tempfile::tempdir().expect("temporary image handoff directory");
    fs::write(temp_dir.path().join(HANDOFF_IMAGE_FILE_NAME), IMAGE_BYTES_A).expect("image handoff written");
    let envelope = handoff_envelope(HANDOFF_IMAGE_FILE_NAME, Some(IMAGE_MEDIA_TYPE));
    let command = write_agent_script(&temp_dir, &format!("#!/bin/sh\nprintf '{envelope}'\n"));
    let adapter = ProductionAgentAdapter::new(runtime_config_for_script(Some(command), None));
    let mut request = agent_request(AgentTarget::Pi, AgentOutputMode::Image);
    request.cwd = Some(temp_dir.path().display().to_string());

    let response = adapter.execute(&request).expect("production image handoff runs");

    match response.output {
        AgentOutput::Image(image) => {
            assert_eq!(image.media_type, IMAGE_MEDIA_TYPE);
            assert_eq!(image.bytes, IMAGE_BYTES_A);
        }
        other => panic!("expected image output, got {other:?}"),
    }
    assert_eq!(response.metadata["file_handoff"]["metadata"]["label"], HANDOFF_METADATA_LABEL);
}

// r[verify n8nclone.workflow_execution.agent_nodes.output_file_handoff.artifact]
#[cfg(unix)]
#[test]
fn production_artifact_handoff_loads_bytes() {
    let temp_dir = tempfile::tempdir().expect("temporary artifact handoff directory");
    fs::write(temp_dir.path().join(HANDOFF_ARTIFACT_FILE_NAME), HANDOFF_ARTIFACT_BYTES)
        .expect("artifact handoff written");
    let envelope = handoff_envelope(HANDOFF_ARTIFACT_FILE_NAME, None);
    let command = write_agent_script(&temp_dir, &format!("#!/bin/sh\nprintf '{envelope}'\n"));
    let adapter = ProductionAgentAdapter::new(runtime_config_for_script(None, Some(command)));
    let mut request = agent_request(AgentTarget::Clankers, AgentOutputMode::Artifact);
    request.cwd = Some(temp_dir.path().display().to_string());

    let response = adapter.execute(&request).expect("production artifact handoff runs");

    assert_eq!(response.output, AgentOutput::Artifact(HANDOFF_ARTIFACT_BYTES.to_vec()));
    assert_eq!(response.metadata["file_handoff"]["bytes"], HANDOFF_ARTIFACT_BYTES.len());
}

// r[verify n8nclone.workflow_execution.agent_nodes.output_file_handoff.safety]
#[test]
fn image_handoff_rejects_missing_file() {
    let temp_dir = tempfile::tempdir().expect("temporary missing handoff directory");
    let plan = AgentCommandPlan {
        cwd: Some(temp_dir.path().display().to_string()),
        ..command_plan(AgentTarget::Pi, AgentOutputMode::Image)
    };
    let output = AgentCommandOutput {
        status_code: Some(0),
        stdout: handoff_envelope(HANDOFF_MISSING_FILE_NAME, Some(IMAGE_MEDIA_TYPE)),
        stderr: String::new(),
    };

    let error = decode_agent_command_output(&plan, &output).expect_err("missing handoff rejected");

    assert!(error.to_string().contains("output file is missing"), "error: {error}");
}

// r[verify n8nclone.workflow_execution.agent_nodes.output_file_handoff.safety]
#[test]
fn image_handoff_rejects_unsupported_image_bytes() {
    let temp_dir = tempfile::tempdir().expect("temporary bad image handoff directory");
    fs::write(temp_dir.path().join(HANDOFF_UNSUPPORTED_IMAGE_FILE_NAME), HANDOFF_BAD_IMAGE_BYTES)
        .expect("bad image handoff written");
    let plan = AgentCommandPlan {
        cwd: Some(temp_dir.path().display().to_string()),
        ..command_plan(AgentTarget::Pi, AgentOutputMode::Image)
    };
    let output = AgentCommandOutput {
        status_code: Some(0),
        stdout: handoff_envelope(HANDOFF_UNSUPPORTED_IMAGE_FILE_NAME, Some(IMAGE_MEDIA_TYPE)),
        stderr: String::new(),
    };

    let error = decode_agent_command_output(&plan, &output).expect_err("unsupported image rejected");

    assert!(error.to_string().contains("unsupported image format"), "error: {error}");
}

// r[verify n8nclone.workflow_execution.agent_nodes.output_file_handoff.safety]
#[test]
fn image_handoff_rejects_media_type_mismatch() {
    let temp_dir = tempfile::tempdir().expect("temporary media mismatch handoff directory");
    fs::write(temp_dir.path().join(HANDOFF_IMAGE_FILE_NAME), IMAGE_BYTES_A).expect("image handoff written");
    let plan = AgentCommandPlan {
        cwd: Some(temp_dir.path().display().to_string()),
        ..command_plan(AgentTarget::Pi, AgentOutputMode::Image)
    };
    let output = AgentCommandOutput {
        status_code: Some(0),
        stdout: handoff_envelope(HANDOFF_IMAGE_FILE_NAME, Some("image/jpeg")),
        stderr: String::new(),
    };

    let error = decode_agent_command_output(&plan, &output).expect_err("media mismatch rejected");

    assert!(error.to_string().contains("media type mismatch"), "error: {error}");
}

// r[verify n8nclone.workflow_execution.agent_nodes.output_file_handoff.safety]
#[test]
fn artifact_handoff_rejects_oversized_file() {
    let temp_dir = tempfile::tempdir().expect("temporary oversized handoff directory");
    fs::write(temp_dir.path().join(HANDOFF_ARTIFACT_FILE_NAME), HANDOFF_ARTIFACT_BYTES)
        .expect("artifact handoff written");
    let plan = AgentCommandPlan {
        cwd: Some(temp_dir.path().display().to_string()),
        max_output_bytes: SMALL_OUTPUT_LIMIT_BYTES,
        ..command_plan(AgentTarget::Clankers, AgentOutputMode::Artifact)
    };
    let output = AgentCommandOutput {
        status_code: Some(0),
        stdout: handoff_envelope(HANDOFF_ARTIFACT_FILE_NAME, None),
        stderr: String::new(),
    };

    let error = decode_agent_command_output(&plan, &output).expect_err("oversized artifact rejected");

    assert!(error.to_string().contains("too large"), "error: {error}");
}

// r[verify n8nclone.workflow_execution.agent_nodes.output_file_handoff.safety]
#[test]
fn handoff_rejects_output_path_outside_working_directory() {
    let cwd_dir = tempfile::tempdir().expect("temporary cwd directory");
    let outside_dir = tempfile::tempdir().expect("temporary outside directory");
    let outside_path = outside_dir.path().join(HANDOFF_OUTSIDE_FILE_NAME);
    fs::write(&outside_path, IMAGE_BYTES_A).expect("outside handoff written");
    let plan = AgentCommandPlan {
        cwd: Some(cwd_dir.path().display().to_string()),
        ..command_plan(AgentTarget::Pi, AgentOutputMode::Image)
    };
    let output = AgentCommandOutput {
        status_code: Some(0),
        stdout: handoff_envelope(&outside_path.display().to_string(), Some(IMAGE_MEDIA_TYPE)),
        stderr: String::new(),
    };

    let error = decode_agent_command_output(&plan, &output).expect_err("outside path rejected");

    assert!(error.to_string().contains("outside working directory"), "error: {error}");
}

// r[verify n8nclone.workflow_execution.agent_nodes.output_file_handoff.safety]
#[test]
fn handoff_rejects_invalid_envelope_json() {
    let plan = command_plan(AgentTarget::Pi, AgentOutputMode::Image);
    let output = AgentCommandOutput {
        status_code: Some(0),
        stdout: INVALID_JSON_OUTPUT.to_string(),
        stderr: String::new(),
    };

    let error = decode_agent_command_output(&plan, &output).expect_err("invalid envelope rejected");

    assert!(error.to_string().contains("handoff envelope is invalid JSON"), "error: {error}");
}

// r[verify n8nclone.workflow_execution.agent_nodes.production_adapters.errors]
#[test]
fn decode_agent_command_output_rejects_invalid_json() {
    let plan = command_plan(AgentTarget::Pi, AgentOutputMode::Json);
    let output = AgentCommandOutput {
        status_code: Some(0),
        stdout: INVALID_JSON_OUTPUT.to_string(),
        stderr: String::new(),
    };

    let error = decode_agent_command_output(&plan, &output).expect_err("invalid JSON rejected");

    assert!(error.to_string().contains("invalid JSON"), "error: {error}");
}

// r[verify n8nclone.workflow_execution.agent_nodes.production_adapters.errors]
#[test]
fn decode_agent_command_output_reports_nonzero_stderr() {
    let plan = command_plan(AgentTarget::Clankers, AgentOutputMode::Text);
    let output = AgentCommandOutput {
        status_code: Some(NONZERO_EXIT_CODE),
        stdout: String::new(),
        stderr: NONZERO_STDERR.to_string(),
    };

    let error = decode_agent_command_output(&plan, &output).expect_err("nonzero exit rejected");

    assert!(error.to_string().contains(&format!("status {NONZERO_EXIT_CODE}")), "error: {error}");
    assert!(error.to_string().contains(NONZERO_STDERR), "error: {error}");
}

// r[verify n8nclone.workflow_execution.agent_nodes.config]
#[test]
fn add_agent_node_records_config_and_ports() {
    let mut workflow = Workflow::new("agent-workflow");
    let node_id = workflow.add_agent_node(AGENT_NODE_LABEL, AgentTarget::Pi, PROMPT_TEXT, AgentOutputMode::Image);
    let node = workflow.graph.node(node_id).expect("agent node exists");

    assert_eq!(node.input_ports.len(), JSON_INPUT_INDEX + 1);
    assert_eq!(node.output_ports.len(), METADATA_OUTPUT_INDEX + 1);
    assert_eq!(node.input_ports[IMAGE_INPUT_INDEX].type_tag, TypeTag::Image.as_str());
    assert_eq!(node.output_ports[IMAGE_OUTPUT_INDEX].type_tag, TypeTag::Image.as_str());
    match workflow.node_config(node_id).expect("agent config exists") {
        NodeConfig::Agent(settings) => {
            assert_eq!(settings.target, AgentTarget::Pi);
            assert_eq!(settings.prompt, PROMPT_TEXT);
            assert_eq!(settings.output_mode, AgentOutputMode::Image);
        }
        other => panic!("expected agent config, got {other:?}"),
    }
}

// r[verify n8nclone.workflow_execution.agent_nodes.pi_adapter]
// r[verify n8nclone.workflow_execution.agent_nodes.execute]
#[tokio::test]
async fn pi_agent_node_executes_text_request_through_adapter() {
    let adapter = StaticAgentAdapter::new(AgentResponse {
        output: AgentOutput::Text(TEXT_OUTPUT.to_string()),
        metadata: serde_json::json!({"adapter": "pi"}),
    });
    let observed_adapter = adapter.clone();
    let mut workflow = Workflow::new("agent-workflow");
    let node_id = workflow.add_agent_node(AGENT_NODE_LABEL, AgentTarget::Pi, PROMPT_TEXT, AgentOutputMode::Text);
    let node = workflow.graph.node(node_id).expect("agent node exists");
    let prompt_input = node.input_ports[PROMPT_INPUT_INDEX].id;
    let inputs = [(prompt_input, Value::from_string("extra prompt".to_string()))].into();
    let executor = Executor::new_with_agent_adapter(Box::new(adapter));

    let result = executor.execute_node(&workflow, node_id, &inputs).await;

    match result.status {
        NodeStatus::Completed { outputs, .. } => {
            let text = outputs.get(&node.output_ports[TEXT_OUTPUT_INDEX].id).expect("text output");
            let metadata = outputs.get(&node.output_ports[METADATA_OUTPUT_INDEX].id).expect("metadata output");
            assert_eq!(text.as_display_string(), TEXT_OUTPUT);
            assert!(metadata.as_display_string().contains("pi"));
        }
        other => panic!("expected completed agent node, got {other:?}"),
    }
    let requests = observed_adapter.requests();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].target, AgentTarget::Pi);
    assert_eq!(requests[0].prompt, PROMPT_TEXT);
    assert_eq!(requests[0].text_inputs, vec!["extra prompt".to_string()]);
}

// r[verify n8nclone.workflow_execution.agent_nodes.clankers_adapter]
// r[verify n8nclone.workflow_execution.agent_nodes.execute]
#[tokio::test]
async fn clankers_agent_node_executes_image_request_through_adapter() {
    let generated_image =
        ImageValue::new(IMAGE_BYTES_B.to_vec(), IMAGE_MEDIA_TYPE.to_string(), ImageMetadata::default());
    let adapter = StaticAgentAdapter::new(AgentResponse {
        output: AgentOutput::Image(generated_image),
        metadata: serde_json::json!({"adapter": "clankers"}),
    });
    let observed_adapter = adapter.clone();
    let mut workflow = Workflow::new("agent-workflow");
    let node_id = workflow.add_agent_node(AGENT_NODE_LABEL, AgentTarget::Clankers, PROMPT_TEXT, AgentOutputMode::Image);
    let node = workflow.graph.node(node_id).expect("agent node exists");
    let image_input = node.input_ports[IMAGE_INPUT_INDEX].id;
    let inputs = [(image_input, image_value(IMAGE_BYTES_A))].into();
    let executor = Executor::new_with_agent_adapter(Box::new(adapter));

    let result = executor.execute_node(&workflow, node_id, &inputs).await;

    match result.status {
        NodeStatus::Completed { outputs, .. } => {
            let image = outputs.get(&node.output_ports[IMAGE_OUTPUT_INDEX].id).expect("image output");
            assert_eq!(image.type_tag(), TypeTag::Image);
            assert!(image.as_display_string().contains(IMAGE_MEDIA_TYPE));
        }
        other => panic!("expected completed agent node, got {other:?}"),
    }
    let requests = observed_adapter.requests();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].target, AgentTarget::Clankers);
    assert_eq!(requests[0].image_inputs.len(), 1);
}

// r[verify n8nclone.workflow_execution.agent_nodes.execute]
#[tokio::test]
async fn output_mode_mismatch_marks_node_failed() {
    let adapter = StaticAgentAdapter::new(AgentResponse {
        output: AgentOutput::Text(TEXT_OUTPUT.to_string()),
        metadata: serde_json::json!({}),
    });
    let mut workflow = Workflow::new("agent-workflow");
    let node_id = workflow.add_agent_node(AGENT_NODE_LABEL, AgentTarget::Pi, PROMPT_TEXT, AgentOutputMode::Image);
    let executor = Executor::new_with_agent_adapter(Box::new(adapter));

    let result = executor.execute_node(&workflow, node_id, &HashMap::new()).await;

    match result.status {
        NodeStatus::Failed { error } => assert!(error.contains("output type mismatch"), "error: {error}"),
        other => panic!("expected failed agent node, got {other:?}"),
    }
}

// r[verify n8nclone.workflow_execution.agent_nodes.cache_key]
#[test]
fn agent_cache_key_changes_for_prompt_target_and_image_edits() {
    let mut workflow = Workflow::new("agent-workflow");
    let node_id = workflow.add_agent_node(AGENT_NODE_LABEL, AgentTarget::Pi, PROMPT_TEXT, AgentOutputMode::Image);
    let node = workflow.graph.node(node_id).expect("agent node exists");
    let image_input = node.input_ports[IMAGE_INPUT_INDEX].id;
    let inputs_a = [(image_input, image_value(IMAGE_BYTES_A))].into();
    let inputs_b = [(image_input, image_value(IMAGE_BYTES_B))].into();
    let original_config = workflow.node_config(node_id).expect("agent config");
    let original_key = node_execution_cache_key(node_id, original_config, &inputs_a);

    let prompt_config = NodeConfig::agent(AgentTarget::Pi, PROMPT_TEXT_EDITED, AgentOutputMode::Image);
    let target_config = NodeConfig::agent(AgentTarget::Clankers, PROMPT_TEXT, AgentOutputMode::Image);

    assert_ne!(original_key, node_execution_cache_key(node_id, &prompt_config, &inputs_a));
    assert_ne!(original_key, node_execution_cache_key(node_id, &target_config, &inputs_a));
    assert_ne!(original_key, node_execution_cache_key(node_id, original_config, &inputs_b));
}

// r[verify n8nclone.workflow_execution.agent_nodes.config]
#[test]
fn empty_agent_prompt_is_validation_diagnostic() {
    let mut workflow = Workflow::new("agent-workflow");
    workflow.add_agent_node(AGENT_NODE_LABEL, AgentTarget::Pi, "", AgentOutputMode::Text);

    let diagnostics = validate_workflow(&workflow);

    assert!(diagnostics.iter().any(|diagnostic| diagnostic.code == WorkflowDiagnosticCode::EmptyAgentPrompt));
}

// r[verify n8nclone.workflow_execution.agent_nodes.errors]
#[tokio::test]
async fn preflight_rejection_marks_node_failed() {
    let adapter = ErrorAgentAdapter::new(AgentError::PreflightRejected("missing model".to_string()));
    let mut workflow = Workflow::new("agent-workflow");
    let node_id = workflow.add_agent_node(AGENT_NODE_LABEL, AgentTarget::Pi, PROMPT_TEXT, AgentOutputMode::Text);
    let executor = Executor::new_with_agent_adapter(Box::new(adapter));

    let result = executor.execute_node(&workflow, node_id, &HashMap::new()).await;

    match result.status {
        NodeStatus::Failed { error } => assert!(error.contains("preflight rejected"), "error: {error}"),
        other => panic!("expected failed agent node, got {other:?}"),
    }
}

// r[verify n8nclone.workflow_execution.agent_nodes.errors]
#[tokio::test]
async fn session_failure_marks_node_failed() {
    let adapter = ErrorAgentAdapter::new(AgentError::Session("session unavailable".to_string()));
    let mut workflow = Workflow::new("agent-workflow");
    let node_id = workflow.add_agent_node(AGENT_NODE_LABEL, AgentTarget::Clankers, PROMPT_TEXT, AgentOutputMode::Text);
    let executor = Executor::new_with_agent_adapter(Box::new(adapter));

    let result = executor.execute_node(&workflow, node_id, &HashMap::new()).await;

    match result.status {
        NodeStatus::Failed { error } => assert!(error.contains("session unavailable"), "error: {error}"),
        other => panic!("expected failed agent node, got {other:?}"),
    }
}

// r[verify n8nclone.workflow_execution.agent_nodes.errors]
#[tokio::test]
async fn adapter_failure_marks_node_failed() {
    let adapter = ErrorAgentAdapter::new(AgentError::Adapter("adapter exploded".to_string()));
    let mut workflow = Workflow::new("agent-workflow");
    let node_id = workflow.add_agent_node(AGENT_NODE_LABEL, AgentTarget::Pi, PROMPT_TEXT, AgentOutputMode::Text);
    let executor = Executor::new_with_agent_adapter(Box::new(adapter));

    let result = executor.execute_node(&workflow, node_id, &HashMap::new()).await;

    match result.status {
        NodeStatus::Failed { error } => assert!(error.contains("adapter exploded"), "error: {error}"),
        other => panic!("expected failed agent node, got {other:?}"),
    }
}

// r[verify n8nclone.workflow_execution.agent_nodes.errors]
#[tokio::test]
async fn credential_failure_skips_agent_adapter_call() {
    let adapter = ErrorAgentAdapter::new(AgentError::Adapter("should not be called".to_string()));
    let observed_adapter = adapter.clone();
    let mut workflow = Workflow::new("agent-workflow");
    let node_id = workflow.add_agent_node(AGENT_NODE_LABEL, AgentTarget::Pi, PROMPT_TEXT, AgentOutputMode::Text);
    workflow.set_node_credentials(node_id, NodeCredentialRefs {
        env: [(SECRET_ENV_NAME.to_string(), SecretRef::new(SECRET_PROVIDER, SECRET_KEY, None))].into(),
    });
    let executor = Executor::new_with_agent_adapter(Box::new(adapter));

    let result = executor.execute_node(&workflow, node_id, &HashMap::new()).await;

    match result.status {
        NodeStatus::Failed { error } => assert!(error.contains("missing secret"), "error: {error}"),
        other => panic!("expected failed agent node, got {other:?}"),
    }
    assert_eq!(observed_adapter.calls(), 0);
}

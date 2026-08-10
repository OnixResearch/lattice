//! DAG executor — runs workflows in topological order.
//!
//! Two modes:
//!  - **Run**: async parallel execution of independent branches via tokio.
//!  - **Debug**: step-through, one node at a time, with pause after each.

use std::collections::BTreeMap;
use std::collections::HashMap;

use rat_nodegraph::model::Node;
use rat_nodegraph::model::NodeId;
use rat_nodegraph::model::PortId;
use serde::Deserialize;
use serde::Serialize;
use tracing::debug;
use tracing::error;
use tracing::info;

use crate::agent_node;
use crate::agent_node::AgentAdapter;
use crate::agent_node::AgentNodeSettings;
use crate::agent_node::AgentOutput;
use crate::agent_node::AgentRequest;
use crate::agent_node::AgentRuntimeConfig;
use crate::agent_node::ProductionAgentAdapter;
use crate::control_flow;
use crate::credentials;
use crate::credentials::CredentialProvider;
use crate::credentials::EmptyCredentialProvider;
use crate::credentials::NodeCredentialRefs;
use crate::engine::child::SubworkflowRunnerShell;
use crate::engine::runner;
use crate::engine::value::ContentHash;
use crate::engine::value::Value;
use crate::engine::workflow::NodeConfig;
use crate::engine::workflow::Workflow;
use crate::expressions;
use crate::expressions::ExpressionContext;
use crate::file_node::FileOperationKind;
use crate::file_node::FileOperationPlan;
use crate::file_node::FileOperationResult;
use crate::file_node::FileSystem;
use crate::http;
use crate::http::HttpClient;
use crate::http::HttpRequestPlan;
use crate::http::NoopHttpClient;
use crate::image_node;
use crate::image_node::ImageIo;
use crate::image_node::LocalImageIo;

const SHELL_STDOUT_OUTPUT_INDEX: usize = 0;
const SHELL_STDERR_OUTPUT_INDEX: usize = 1;
const SHELL_EXIT_OUTPUT_INDEX: usize = 2;
const SHELL_STDIN_INPUT_LABEL: &str = "stdin";
const SHELL_COMMAND_FLAG: &str = "-c";
const UNKNOWN_EXIT_CODE: i32 = -1;
const PROCESS_TASK_FAILURE_DIAGNOSTIC: &str = "bounded local process task failed";
const SHELL_ARGUMENT_COUNT: usize = 2;

type LocalProcessConfig = crate::process::core::RuntimeConfig;
const HTTP_BODY_INPUT_LABEL: &str = "body";
const FILE_CONTENT_INPUT_LABEL: &str = "content";
const FILE_CONTENT_OUTPUT_INDEX: usize = 0;
const FILE_METADATA_OUTPUT_INDEX: usize = 1;
const FILE_BYTES_OUTPUT_INDEX: usize = 2;
const IMAGE_VALUE_OUTPUT_INDEX: usize = 0;
const IMAGE_METADATA_OUTPUT_INDEX: usize = 1;
const AGENT_TEXT_OUTPUT_INDEX: usize = 0;
const AGENT_JSON_OUTPUT_INDEX: usize = 1;
const AGENT_IMAGE_OUTPUT_INDEX: usize = 2;
const AGENT_ARTIFACT_OUTPUT_INDEX: usize = 3;
const AGENT_METADATA_OUTPUT_INDEX: usize = 4;
const HTTP_STATUS_OUTPUT_INDEX: usize = 0;
const HTTP_HEADERS_OUTPUT_INDEX: usize = 1;
const HTTP_BODY_OUTPUT_INDEX: usize = 2;
const HASH_TAG_NODE_EXECUTION: &[u8] = b"node-execution:v1";
const HASH_TAG_NODE_ID: &[u8] = b"node-id";
const HASH_TAG_SHELL_CONFIG: &[u8] = b"shell-config";
const HASH_TAG_HTTP_CONFIG: &[u8] = b"http-config";
const HASH_TAG_FILE_CONFIG: &[u8] = b"file-config";
const HASH_TAG_IMAGE_INPUT_CONFIG: &[u8] = b"image-input-config";
const HASH_TAG_AGENT_CONFIG: &[u8] = b"agent-config";
const HASH_TAG_CONDITION_CONFIG: &[u8] = b"condition-config";
const HASH_TAG_MERGE_CONFIG: &[u8] = b"merge-config";
const HASH_TAG_SUBWORKFLOW_CONFIG: &[u8] = b"subworkflow-config";
const HASH_TAG_COMMAND: &[u8] = b"command";
const HASH_TAG_HTTP_METHOD: &[u8] = b"http-method";
const HASH_TAG_HTTP_URL: &[u8] = b"http-url";
const HASH_TAG_HTTP_HEADERS: &[u8] = b"http-headers";
const HASH_TAG_HTTP_BODY: &[u8] = b"http-body";
const HASH_TAG_FILE_OPERATION: &[u8] = b"file-operation";
const HASH_TAG_FILE_PATH: &[u8] = b"file-path";
const HASH_TAG_FILE_CONTENT: &[u8] = b"file-content";
const HASH_TAG_FILE_OVERWRITE: &[u8] = b"file-overwrite";
const HASH_TAG_FILE_ROOTS: &[u8] = b"file-roots";
const HASH_TAG_IMAGE_PATH: &[u8] = b"image-path";
const HASH_TAG_IMAGE_ROOTS: &[u8] = b"image-roots";
const HASH_TAG_IMAGE_MAX_BYTES: &[u8] = b"image-max-bytes";
const HASH_TAG_AGENT_TARGET: &[u8] = b"agent-target";
const HASH_TAG_AGENT_PROMPT: &[u8] = b"agent-prompt";
const HASH_TAG_AGENT_PROMPT_FILE: &[u8] = b"agent-prompt-file";
const HASH_TAG_AGENT_OUTPUT_MODE: &[u8] = b"agent-output-mode";
const HASH_TAG_AGENT_MODEL_HINT: &[u8] = b"agent-model-hint";
const HASH_TAG_AGENT_TOOL_POLICY: &[u8] = b"agent-tool-policy";
const HASH_TAG_AGENT_CONTEXT_REFS: &[u8] = b"agent-context-refs";
const HASH_TAG_AGENT_SESSION: &[u8] = b"agent-session";
const HASH_TAG_CWD: &[u8] = b"cwd";
const HASH_TAG_ENV: &[u8] = b"env";
const HASH_TAG_SECRET_ENV: &[u8] = b"secret-env";
const HASH_TAG_SECRET_PROVIDER: &[u8] = b"secret-provider";
const HASH_TAG_SECRET_KEY: &[u8] = b"secret-key";
const HASH_TAG_SECRET_FIELD: &[u8] = b"secret-field";
const HASH_TAG_TIMEOUT: &[u8] = b"timeout-secs";
const HASH_TAG_PIPE_STDIN: &[u8] = b"pipe-stdin";
const HASH_TAG_INPUTS: &[u8] = b"inputs";
const HASH_TAG_INPUT_PORT: &[u8] = b"input-port";
const HASH_TAG_INPUT_TYPE: &[u8] = b"input-type";
const HASH_TAG_INPUT_VALUE: &[u8] = b"input-value";
const HASH_TAG_SOME: &[u8] = b"some";
const HASH_TAG_NONE: &[u8] = b"none";
const HASH_TAG_TRUE: &[u8] = b"true";
const HASH_TAG_FALSE: &[u8] = b"false";
const HASH_TAG_BRANCH: &[u8] = b"branch";
const HASH_TAG_PREDICATE: &[u8] = b"predicate";
const HASH_TAG_MERGE_POLICY: &[u8] = b"merge-policy";
const HASH_TAG_SUBWORKFLOW_NAME: &[u8] = b"subworkflow-name";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ShellOutputPortIds {
    stdout: PortId,
    stderr: PortId,
    exit: PortId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ShellOutputValues {
    stdout: String,
    stderr: String,
    exit_code: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResolvedString {
    value: String,
    sensitive_fragments: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct HttpOutputPortIds {
    status: PortId,
    headers: PortId,
    body: PortId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FileOutputPortIds {
    content: PortId,
    metadata: PortId,
    bytes_written: PortId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ImageOutputPortIds {
    image: PortId,
    metadata: PortId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AgentOutputPortIds {
    text: PortId,
    json: PortId,
    image: PortId,
    artifact: PortId,
    metadata: PortId,
}

// r[impl n8nclone.workflow_execution.cache_key.shell_config]
pub fn node_execution_cache_key(node_id: NodeId, config: &NodeConfig, inputs: &HashMap<PortId, Value>) -> ContentHash {
    node_execution_cache_key_with_credentials(node_id, config, None, inputs)
}

// r[impl n8nclone.credentials.cache_identity]
pub fn node_execution_cache_key_with_credentials(
    node_id: NodeId,
    config: &NodeConfig,
    credentials: Option<&NodeCredentialRefs>,
    inputs: &HashMap<PortId, Value>,
) -> ContentHash {
    let mut hasher = blake3::Hasher::new();
    hasher.update(HASH_TAG_NODE_EXECUTION);
    update_u64(&mut hasher, HASH_TAG_NODE_ID, node_id.0);
    update_node_config_hash(&mut hasher, config);
    update_secret_refs_hash(&mut hasher, credentials);
    update_inputs_hash(&mut hasher, inputs);
    ContentHash(*hasher.finalize().as_bytes())
}

fn input_values_hash(inputs: &HashMap<PortId, Value>) -> ContentHash {
    let mut hasher = blake3::Hasher::new();
    update_inputs_hash(&mut hasher, inputs);
    ContentHash(*hasher.finalize().as_bytes())
}

fn update_node_config_hash(hasher: &mut blake3::Hasher, config: &NodeConfig) {
    match config {
        NodeConfig::Shell {
            command,
            cwd,
            env,
            timeout_secs,
            pipe_stdin,
        } => {
            hasher.update(HASH_TAG_SHELL_CONFIG);
            update_str(hasher, HASH_TAG_COMMAND, command);
            update_optional_str(hasher, HASH_TAG_CWD, cwd.as_deref());
            update_env(hasher, env);
            update_u64(hasher, HASH_TAG_TIMEOUT, *timeout_secs);
            update_bool(hasher, HASH_TAG_PIPE_STDIN, *pipe_stdin);
        }
        NodeConfig::Http {
            method,
            url,
            headers,
            body,
            timeout_secs,
        } => {
            hasher.update(HASH_TAG_HTTP_CONFIG);
            update_str(hasher, HASH_TAG_HTTP_METHOD, &http::normalized_method(method));
            update_str(hasher, HASH_TAG_HTTP_URL, url);
            update_env_with_tag(hasher, HASH_TAG_HTTP_HEADERS, headers);
            update_optional_str(hasher, HASH_TAG_HTTP_BODY, body.as_deref());
            update_u64(hasher, HASH_TAG_TIMEOUT, *timeout_secs);
        }
        NodeConfig::File {
            operation,
            path,
            content,
            overwrite,
            allowed_roots,
        } => {
            hasher.update(HASH_TAG_FILE_CONFIG);
            update_str(hasher, HASH_TAG_FILE_OPERATION, file_operation_label(*operation));
            update_str(hasher, HASH_TAG_FILE_PATH, path);
            update_optional_str(hasher, HASH_TAG_FILE_CONTENT, content.as_deref());
            update_bool(hasher, HASH_TAG_FILE_OVERWRITE, *overwrite);
            update_string_list_hash(hasher, HASH_TAG_FILE_ROOTS, allowed_roots);
        }
        NodeConfig::ImageInput {
            path,
            allowed_roots,
            max_bytes,
        } => {
            hasher.update(HASH_TAG_IMAGE_INPUT_CONFIG);
            update_str(hasher, HASH_TAG_IMAGE_PATH, path);
            update_string_list_hash(hasher, HASH_TAG_IMAGE_ROOTS, allowed_roots);
            update_usize(hasher, HASH_TAG_IMAGE_MAX_BYTES, *max_bytes);
        }
        NodeConfig::Agent(settings) => {
            hasher.update(HASH_TAG_AGENT_CONFIG);
            update_agent_settings_hash(hasher, settings);
        }
        NodeConfig::Condition(config) => {
            hasher.update(HASH_TAG_CONDITION_CONFIG);
            for branch in &config.branches {
                update_str(hasher, HASH_TAG_BRANCH, &branch.label);
                update_str(hasher, HASH_TAG_PREDICATE, &format!("{:?}", branch.predicate));
            }
        }
        NodeConfig::Merge(config) => {
            hasher.update(HASH_TAG_MERGE_CONFIG);
            update_str(hasher, HASH_TAG_MERGE_POLICY, &format!("{:?}", config.policy));
            update_ordered_string_list_hash(hasher, HASH_TAG_INPUTS, &config.required_inputs);
        }
        NodeConfig::Subworkflow(config) => {
            hasher.update(HASH_TAG_SUBWORKFLOW_CONFIG);
            update_str(hasher, HASH_TAG_SUBWORKFLOW_NAME, &config.workflow_name);
        }
    }
}

fn update_agent_settings_hash(hasher: &mut blake3::Hasher, settings: &AgentNodeSettings) {
    update_str(hasher, HASH_TAG_AGENT_TARGET, agent_node::agent_target_label(settings.target));
    update_str(hasher, HASH_TAG_AGENT_PROMPT, &settings.prompt);
    update_optional_str(hasher, HASH_TAG_AGENT_PROMPT_FILE, settings.prompt_file.as_deref());
    update_str(hasher, HASH_TAG_AGENT_OUTPUT_MODE, settings.output_mode.as_str());
    update_optional_str(hasher, HASH_TAG_AGENT_MODEL_HINT, settings.model_hint.as_deref());
    update_str(hasher, HASH_TAG_AGENT_TOOL_POLICY, &settings.tool_policy);
    update_ordered_string_list_hash(hasher, HASH_TAG_AGENT_CONTEXT_REFS, &settings.context_refs);
    update_optional_str(hasher, HASH_TAG_CWD, settings.cwd.as_deref());
    update_optional_str(hasher, HASH_TAG_AGENT_SESSION, settings.session.as_deref());
}

fn update_inputs_hash(hasher: &mut blake3::Hasher, inputs: &HashMap<PortId, Value>) {
    let mut input_pairs = inputs.iter().collect::<Vec<_>>();
    input_pairs.sort_by_key(|(port_id, _)| port_id.0);

    hasher.update(HASH_TAG_INPUTS);
    update_len(hasher, input_pairs.len());
    for (port_id, value) in input_pairs {
        update_u64(hasher, HASH_TAG_INPUT_PORT, port_id.0);
        update_str(hasher, HASH_TAG_INPUT_TYPE, value.type_tag().as_str());
        hasher.update(HASH_TAG_INPUT_VALUE);
        hasher.update(&value.hash.0);
    }
}

fn update_env(hasher: &mut blake3::Hasher, env: &HashMap<String, String>) {
    update_env_with_tag(hasher, HASH_TAG_ENV, env);
}

fn update_env_with_tag(hasher: &mut blake3::Hasher, tag: &[u8], env: &HashMap<String, String>) {
    let mut env_pairs = env.iter().collect::<Vec<_>>();
    env_pairs.sort_by_key(|(key, _)| *key);

    hasher.update(tag);
    update_len(hasher, env_pairs.len());
    for (key, value) in env_pairs {
        update_str(hasher, tag, key);
        update_str(hasher, tag, value);
    }
}

fn update_string_list_hash(hasher: &mut blake3::Hasher, tag: &[u8], values: &[String]) {
    let mut sorted_values = values.iter().collect::<Vec<_>>();
    sorted_values.sort();
    hasher.update(tag);
    update_len(hasher, sorted_values.len());
    for value in sorted_values {
        update_str(hasher, tag, value);
    }
}

fn update_ordered_string_list_hash(hasher: &mut blake3::Hasher, tag: &[u8], values: &[String]) {
    hasher.update(tag);
    update_len(hasher, values.len());
    for value in values {
        update_str(hasher, tag, value);
    }
}

fn update_usize(hasher: &mut blake3::Hasher, tag: &[u8], value: usize) {
    let encoded_value = u64::try_from(value).expect("hash input value fits in u64");
    update_u64(hasher, tag, encoded_value);
}

fn file_operation_label(operation: FileOperationKind) -> &'static str {
    match operation {
        FileOperationKind::Read => "read",
        FileOperationKind::Write => "write",
        FileOperationKind::Append => "append",
    }
}

fn resolve_config_string(input: &str, context: &ExpressionContext) -> Result<ResolvedString, String> {
    expressions::resolve_expression_string(input, context)
        .map(|value| ResolvedString {
            value: value.text,
            sensitive_fragments: value.sensitive_fragments,
        })
        .map_err(|error| error.to_string())
}

fn resolve_optional_config_string(
    input: &Option<String>,
    context: &ExpressionContext,
) -> Result<Option<ResolvedString>, String> {
    input.as_ref().map(|value| resolve_config_string(value, context)).transpose()
}

fn resolve_config_map(
    input: &HashMap<String, String>,
    context: &ExpressionContext,
) -> Result<(HashMap<String, String>, Vec<String>), String> {
    let mut resolved = HashMap::new();
    let mut sensitive_fragments = Vec::new();
    for (key, value) in input {
        let resolved_value = resolve_config_string(value, context)?;
        sensitive_fragments.extend(resolved_value.sensitive_fragments);
        resolved.insert(key.clone(), resolved_value.value);
    }
    Ok((resolved, sensitive_fragments))
}

fn resolve_config_list(input: &[String], context: &ExpressionContext) -> Result<Vec<String>, String> {
    let mut resolved = Vec::new();
    for value in input {
        resolved.push(resolve_config_string(value, context)?.value);
    }
    Ok(resolved)
}

fn collect_input_sensitive_fragments(inputs: &HashMap<PortId, Value>) -> Vec<String> {
    inputs.values().flat_map(|value| value.sensitive_fragments.clone()).collect()
}

fn expression_context_from_workflow_outputs(
    workflow: &Workflow,
    port_values: &HashMap<PortId, Value>,
) -> ExpressionContext {
    let mut context = ExpressionContext::new(HashMap::new());
    for node in workflow.graph.nodes() {
        for port in &node.output_ports {
            if let Some(value) = port_values.get(&port.id) {
                context.insert(node.id.0, port.id, value.clone());
            }
        }
    }
    context
}

fn update_secret_refs_hash(hasher: &mut blake3::Hasher, credentials: Option<&NodeCredentialRefs>) {
    let mut secret_pairs =
        credentials.map(|credentials| credentials.env.iter().collect::<Vec<_>>()).unwrap_or_default();
    secret_pairs.sort_by_key(|(env_name, _)| *env_name);

    hasher.update(HASH_TAG_SECRET_ENV);
    update_len(hasher, secret_pairs.len());
    for (env_name, secret_ref) in secret_pairs {
        update_str(hasher, HASH_TAG_SECRET_ENV, env_name);
        update_str(hasher, HASH_TAG_SECRET_PROVIDER, &secret_ref.provider);
        update_str(hasher, HASH_TAG_SECRET_KEY, &secret_ref.key);
        update_optional_str(hasher, HASH_TAG_SECRET_FIELD, secret_ref.field.as_deref());
    }
}

fn update_optional_str(hasher: &mut blake3::Hasher, tag: &[u8], value: Option<&str>) {
    hasher.update(tag);
    match value {
        Some(value) => update_str(hasher, HASH_TAG_SOME, value),
        None => {
            hasher.update(HASH_TAG_NONE);
        }
    }
}

fn update_str(hasher: &mut blake3::Hasher, tag: &[u8], value: &str) {
    hasher.update(tag);
    update_len(hasher, value.len());
    hasher.update(value.as_bytes());
}

fn update_bool(hasher: &mut blake3::Hasher, tag: &[u8], value: bool) {
    hasher.update(tag);
    hasher.update(if value { HASH_TAG_TRUE } else { HASH_TAG_FALSE });
}

fn update_u64(hasher: &mut blake3::Hasher, tag: &[u8], value: u64) {
    hasher.update(tag);
    hasher.update(&value.to_le_bytes());
}

fn update_len(hasher: &mut blake3::Hasher, value: usize) {
    let encoded_len = u64::try_from(value).expect("hash input length fits in u64");
    hasher.update(&encoded_len.to_le_bytes());
}

fn shell_stdin_value<'a>(node: &Node, inputs: &'a HashMap<PortId, Value>) -> Option<&'a Value> {
    let labeled_stdin = node
        .input_ports
        .iter()
        .find(|port| port.label == SHELL_STDIN_INPUT_LABEL)
        .and_then(|port| inputs.get(&port.id));
    if labeled_stdin.is_some() {
        return labeled_stdin;
    }

    node.input_ports.iter().find_map(|port| inputs.get(&port.id))
}

fn http_body_value<'a>(node: &Node, inputs: &'a HashMap<PortId, Value>) -> Option<&'a Value> {
    let labeled_body = node
        .input_ports
        .iter()
        .find(|port| port.label == HTTP_BODY_INPUT_LABEL)
        .and_then(|port| inputs.get(&port.id));
    if labeled_body.is_some() {
        return labeled_body;
    }

    node.input_ports.iter().find_map(|port| inputs.get(&port.id))
}

fn http_request_plan(
    node: &Node,
    method: &str,
    url: &str,
    headers: &HashMap<String, String>,
    body: &Option<String>,
    timeout_secs: u64,
    inputs: &HashMap<PortId, Value>,
) -> HttpRequestPlan {
    let resolved_body = body.clone().or_else(|| http_body_value(node, inputs).map(Value::as_display_string));
    HttpRequestPlan {
        method: http::normalized_method(method),
        url: url.to_string(),
        headers: headers.clone(),
        body: resolved_body,
        timeout_secs,
    }
}

fn file_content_value<'a>(node: &Node, inputs: &'a HashMap<PortId, Value>) -> Option<&'a Value> {
    let labeled_content = node
        .input_ports
        .iter()
        .find(|port| port.label == FILE_CONTENT_INPUT_LABEL)
        .and_then(|port| inputs.get(&port.id));
    if labeled_content.is_some() {
        return labeled_content;
    }

    node.input_ports.iter().find_map(|port| inputs.get(&port.id))
}

fn file_operation_plan(
    node: &Node,
    operation: FileOperationKind,
    path: &str,
    content: &Option<String>,
    overwrite: bool,
    allowed_roots: &[String],
    inputs: &HashMap<PortId, Value>,
) -> FileOperationPlan {
    let resolved_content = content.clone().or_else(|| file_content_value(node, inputs).map(Value::as_display_string));
    FileOperationPlan {
        kind: operation,
        path: path.to_string(),
        content: resolved_content,
        overwrite,
        allowed_roots: allowed_roots.to_vec(),
    }
}

fn ordered_input_values(inputs: &HashMap<PortId, Value>) -> Vec<Value> {
    let mut input_pairs = inputs.iter().collect::<Vec<_>>();
    input_pairs.sort_by_key(|(port_id, _)| port_id.0);
    input_pairs.into_iter().map(|(_, value)| value.clone()).collect()
}

// r[impl n8nclone.workflow_execution.agent_nodes.execute]
fn agent_request_from_settings(
    settings: &AgentNodeSettings,
    prompt: String,
    model_hint: Option<String>,
    tool_policy: String,
    context_refs: Vec<String>,
    cwd: Option<String>,
    session: Option<String>,
    inputs: &HashMap<PortId, Value>,
) -> AgentRequest {
    let ordered_values = ordered_input_values(inputs);
    let (text_inputs, image_inputs, json_inputs) = agent_node::request_inputs_from_values(&ordered_values);
    AgentRequest {
        target: settings.target,
        prompt,
        output_mode: settings.output_mode,
        model_hint,
        tool_policy,
        context_refs,
        cwd,
        session,
        text_inputs,
        image_inputs,
        json_inputs,
    }
}

fn shell_output_port_ids(node: &Node) -> Result<ShellOutputPortIds, String> {
    let stdout = node
        .output_ports
        .get(SHELL_STDOUT_OUTPUT_INDEX)
        .map(|port| port.id)
        .ok_or_else(|| format!("node '{}' is missing stdout output port", node.label))?;
    let stderr = node
        .output_ports
        .get(SHELL_STDERR_OUTPUT_INDEX)
        .map(|port| port.id)
        .ok_or_else(|| format!("node '{}' is missing stderr output port", node.label))?;
    let exit = node
        .output_ports
        .get(SHELL_EXIT_OUTPUT_INDEX)
        .map(|port| port.id)
        .ok_or_else(|| format!("node '{}' is missing exit output port", node.label))?;

    Ok(ShellOutputPortIds { stdout, stderr, exit })
}

fn shell_outputs_by_real_port(ports: ShellOutputPortIds, outputs: ShellOutputValues) -> HashMap<PortId, Value> {
    let mut mapped = HashMap::new();
    mapped.insert(ports.stdout, Value::from_string(outputs.stdout));
    mapped.insert(ports.stderr, Value::from_string(outputs.stderr));
    mapped.insert(ports.exit, Value::from_exit_code(outputs.exit_code));
    mapped
}

fn http_output_port_ids(node: &Node) -> Result<HttpOutputPortIds, String> {
    let status = node
        .output_ports
        .get(HTTP_STATUS_OUTPUT_INDEX)
        .map(|port| port.id)
        .ok_or_else(|| format!("node '{}' is missing HTTP status output port", node.label))?;
    let headers = node
        .output_ports
        .get(HTTP_HEADERS_OUTPUT_INDEX)
        .map(|port| port.id)
        .ok_or_else(|| format!("node '{}' is missing HTTP headers output port", node.label))?;
    let body = node
        .output_ports
        .get(HTTP_BODY_OUTPUT_INDEX)
        .map(|port| port.id)
        .ok_or_else(|| format!("node '{}' is missing HTTP body output port", node.label))?;
    Ok(HttpOutputPortIds { status, headers, body })
}

fn http_outputs_by_real_port(ports: HttpOutputPortIds, response: http::HttpResponse) -> HashMap<PortId, Value> {
    let mut mapped = HashMap::new();
    mapped.insert(ports.status, Value::from_string(response.status.to_string()));
    mapped.insert(ports.headers, Value::from_json(serde_json::json!(response.headers)));
    mapped.insert(ports.body, Value::from_string(response.body));
    mapped
}

fn file_output_port_ids(node: &Node) -> Result<FileOutputPortIds, String> {
    let content = node
        .output_ports
        .get(FILE_CONTENT_OUTPUT_INDEX)
        .map(|port| port.id)
        .ok_or_else(|| format!("node '{}' is missing file content output port", node.label))?;
    let metadata = node
        .output_ports
        .get(FILE_METADATA_OUTPUT_INDEX)
        .map(|port| port.id)
        .ok_or_else(|| format!("node '{}' is missing file metadata output port", node.label))?;
    let bytes_written = node
        .output_ports
        .get(FILE_BYTES_OUTPUT_INDEX)
        .map(|port| port.id)
        .ok_or_else(|| format!("node '{}' is missing file bytes output port", node.label))?;
    Ok(FileOutputPortIds {
        content,
        metadata,
        bytes_written,
    })
}

fn file_outputs_by_real_port(
    ports: FileOutputPortIds,
    path: &str,
    result: FileOperationResult,
) -> HashMap<PortId, Value> {
    let mut mapped = HashMap::new();
    match result {
        FileOperationResult::Read { content, bytes } => {
            mapped.insert(ports.content, Value::from_string(content));
            mapped.insert(ports.metadata, Value::from_json(serde_json::json!({ "path": path, "bytes": bytes })));
            mapped.insert(ports.bytes_written, Value::from_string(String::new()));
        }
        FileOperationResult::Written { bytes } => {
            mapped.insert(ports.content, Value::from_string(String::new()));
            mapped.insert(ports.metadata, Value::from_json(serde_json::json!({ "path": path, "bytes": bytes })));
            mapped.insert(ports.bytes_written, Value::from_string(bytes.to_string()));
        }
    }
    mapped
}

fn image_output_port_ids(node: &Node) -> Result<ImageOutputPortIds, String> {
    let image = node
        .output_ports
        .get(IMAGE_VALUE_OUTPUT_INDEX)
        .map(|port| port.id)
        .ok_or_else(|| format!("node '{}' is missing image output port", node.label))?;
    let metadata = node
        .output_ports
        .get(IMAGE_METADATA_OUTPUT_INDEX)
        .map(|port| port.id)
        .ok_or_else(|| format!("node '{}' is missing image metadata output port", node.label))?;
    Ok(ImageOutputPortIds { image, metadata })
}

fn image_outputs_by_real_port(
    ports: ImageOutputPortIds,
    image: crate::engine::value::ImageValue,
) -> HashMap<PortId, Value> {
    let mut mapped = HashMap::new();
    let metadata = serde_json::json!({
        "media_type": image.media_type.clone(),
        "bytes": image.bytes.len(),
        "width": image.metadata.width,
        "height": image.metadata.height,
        "source_label": image.metadata.source_label.clone(),
    });
    mapped.insert(ports.image, Value::from_image(image));
    mapped.insert(ports.metadata, Value::from_json(metadata));
    mapped
}

fn agent_output_port_ids(node: &Node) -> Result<AgentOutputPortIds, String> {
    let text = node
        .output_ports
        .get(AGENT_TEXT_OUTPUT_INDEX)
        .map(|port| port.id)
        .ok_or_else(|| format!("node '{}' is missing agent text output port", node.label))?;
    let json = node
        .output_ports
        .get(AGENT_JSON_OUTPUT_INDEX)
        .map(|port| port.id)
        .ok_or_else(|| format!("node '{}' is missing agent JSON output port", node.label))?;
    let image = node
        .output_ports
        .get(AGENT_IMAGE_OUTPUT_INDEX)
        .map(|port| port.id)
        .ok_or_else(|| format!("node '{}' is missing agent image output port", node.label))?;
    let artifact = node
        .output_ports
        .get(AGENT_ARTIFACT_OUTPUT_INDEX)
        .map(|port| port.id)
        .ok_or_else(|| format!("node '{}' is missing agent artifact output port", node.label))?;
    let metadata = node
        .output_ports
        .get(AGENT_METADATA_OUTPUT_INDEX)
        .map(|port| port.id)
        .ok_or_else(|| format!("node '{}' is missing agent metadata output port", node.label))?;
    Ok(AgentOutputPortIds {
        text,
        json,
        image,
        artifact,
        metadata,
    })
}

fn agent_outputs_by_real_port(
    ports: AgentOutputPortIds,
    response: agent_node::AgentResponse,
) -> HashMap<PortId, Value> {
    let mut mapped = HashMap::new();
    match response.output {
        AgentOutput::Text(text) => {
            mapped.insert(ports.text, Value::from_string(text));
        }
        AgentOutput::Json(json) => {
            mapped.insert(ports.json, Value::from_json(json));
        }
        AgentOutput::Image(image) => {
            mapped.insert(ports.image, Value::from_image(image));
        }
        AgentOutput::Artifact(bytes) => {
            mapped.insert(ports.artifact, Value::from_bytes(bytes));
        }
    }
    mapped.insert(ports.metadata, Value::from_json(response.metadata));
    mapped
}

fn first_input_value(inputs: &HashMap<PortId, Value>) -> Option<&Value> {
    inputs.iter().min_by_key(|(port_id, _)| port_id.0).map(|(_, value)| value)
}

// r[impl n8nclone.workflow_execution.control_flow.condition]
fn condition_outputs_by_real_port(
    node: &Node,
    matched_labels: &[String],
    value: Option<&Value>,
) -> HashMap<PortId, Value> {
    let matched = matched_labels.iter().collect::<std::collections::BTreeSet<_>>();
    node.output_ports
        .iter()
        .filter(|port| matched.contains(&port.label))
        .map(|port| {
            let output = value.cloned().unwrap_or_else(|| {
                Value::from_json(serde_json::json!({
                    "branch": port.label,
                    "matched": true,
                }))
            });
            (port.id, output)
        })
        .collect()
}

fn merge_inputs_by_label(node: &Node, inputs: &HashMap<PortId, Value>) -> Vec<control_flow::MergeInputValue> {
    node.input_ports
        .iter()
        .enumerate()
        .filter_map(|(sequence, port)| {
            inputs.get(&port.id).cloned().map(|value| control_flow::MergeInputValue {
                label: port.label.clone(),
                sequence: u64::try_from(sequence).expect("input port sequence fits in u64"),
                value,
            })
        })
        .collect()
}

// r[impl n8nclone.workflow_execution.control_flow.merge]
fn merge_outputs_by_real_port(
    node: &Node,
    policy: control_flow::MergePolicy,
    selected: Vec<control_flow::MergeInputValue>,
) -> HashMap<PortId, Value> {
    let Some(output_port) = node.output_ports.first() else {
        return HashMap::new();
    };
    let output_value = merge_output_value(policy, selected);
    HashMap::from([(output_port.id, output_value)])
}

fn merge_output_value(policy: control_flow::MergePolicy, selected: Vec<control_flow::MergeInputValue>) -> Value {
    if policy != control_flow::MergePolicy::Collect && selected.len() == 1 {
        return selected.into_iter().next().expect("selected merge value exists").value;
    }
    let sensitive_fragments =
        selected.iter().flat_map(|input| input.value.sensitive_fragments.clone()).collect::<Vec<_>>();
    let values = selected
        .into_iter()
        .map(|input| {
            serde_json::json!({
                "label": input.label,
                "sequence": input.sequence,
                "value": value_to_json(&input.value),
            })
        })
        .collect::<Vec<_>>();
    Value::from_json(serde_json::json!(values)).with_sensitive_fragments(sensitive_fragments)
}

fn value_to_json(value: &Value) -> serde_json::Value {
    match &value.data {
        crate::engine::ValueData::String(value) => serde_json::json!(value),
        crate::engine::ValueData::Bytes(value) => serde_json::json!(value),
        crate::engine::ValueData::Json(value) => value.clone(),
        crate::engine::ValueData::Image(image) => serde_json::json!({
            "media_type": image.media_type,
            "summary": image.summary(),
        }),
        crate::engine::ValueData::ExitCode(value) => serde_json::json!(value),
        crate::engine::ValueData::Error { message, stderr } => serde_json::json!({
            "message": message,
            "stderr": stderr,
        }),
    }
}

// r[impl n8nclone.workflow_execution.runtime_registry.typed_output_contract]
fn completed_step_result(
    node_id: NodeId,
    input_hash: ContentHash,
    node: &Node,
    outputs: HashMap<PortId, Value>,
    cached: bool,
) -> StepResult {
    match admit_completed_outputs(node, &outputs) {
        Ok(()) => StepResult {
            node_id,
            status: NodeStatus::Completed { outputs, cached },
            input_hash,
        },
        Err(error) => StepResult {
            node_id,
            status: NodeStatus::Failed { error },
            input_hash,
        },
    }
}

fn admit_completed_outputs(node: &Node, outputs: &HashMap<PortId, Value>) -> Result<(), String> {
    let runner_node = runner_node_from_graph_node(node);
    let output_shapes = outputs
        .iter()
        .map(|(port_id, value)| runner::RuntimeOutputShape {
            port_id: port_id.0,
            type_tag: value.type_tag().as_str().to_string(),
        })
        .collect::<Vec<_>>();
    runner::admit_runtime_outputs(&runner_node, &output_shapes).map_err(format_runner_output_diagnostics)
}

fn runner_node_from_graph_node(node: &Node) -> runner::RunnerNode {
    runner::RunnerNode {
        id: node.id.0,
        label: node.label.clone(),
        kind: String::new(),
        runtime: runner::RuntimeBinding {
            kind: String::new(),
            key: String::new(),
        },
        policy: runner::RunnerPolicy {
            effect_policy: String::new(),
            cache_policy: String::new(),
            retry_policy: String::new(),
            artifact_policy: String::new(),
        },
        input_ports: Vec::new(),
        output_ports: node
            .output_ports
            .iter()
            .map(|port| runner::RunnerPort {
                id: port.id.0,
                label: port.label.clone(),
                direction: runner::RunnerPortDirection::Output,
                type_tag: port.type_tag.clone(),
            })
            .collect(),
        credential_ref_count: 0,
    }
}

fn format_runner_output_diagnostics(diagnostics: Vec<runner::RunnerDiagnostic>) -> String {
    diagnostics.into_iter().map(|diagnostic| diagnostic.message).collect::<Vec<_>>().join("; ")
}

/// Unique identifier for an execution run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RunId(pub u64);

impl std::fmt::Display for RunId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "run:{}", self.0)
    }
}

/// Execution mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionMode {
    /// Run all independent branches in parallel.
    Run,
    /// Step through one node at a time.
    Debug,
}

/// Per-node execution status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeStatus {
    Pending,
    Running,
    Completed {
        outputs: HashMap<PortId, Value>,
        cached: bool,
    },
    Failed {
        error: String,
    },
    Skipped,
}

/// Result of executing a single step (one node).
#[derive(Debug, Clone)]
pub struct StepResult {
    pub node_id: NodeId,
    pub status: NodeStatus,
    pub input_hash: ContentHash,
}

struct RuntimeParts {
    credential_provider: Box<dyn CredentialProvider>,
    http_client: Box<dyn HttpClient>,
    file_system: Box<dyn FileSystem>,
    image_io: Box<dyn ImageIo>,
    agent_adapter: Box<dyn AgentAdapter>,
    local_process: LocalProcessConfig,
}

/// The executor.
pub struct Executor {
    next_run_id: u64,
    credential_provider: Box<dyn CredentialProvider>,
    http_client: Box<dyn HttpClient>,
    file_system: Box<dyn FileSystem>,
    image_io: Box<dyn ImageIo>,
    agent_adapter: Box<dyn AgentAdapter>,
    local_process: LocalProcessConfig,
    subworkflow_runner_shell: Box<dyn SubworkflowRunnerShell>,
    cancellation: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

// r[impl n8nclone.workflow_execution.image_input_nodes.production_io.default_executor]
fn default_image_io() -> Box<dyn ImageIo> {
    Box::<LocalImageIo>::default()
}

// r[impl n8nclone.workflow_execution.file_nodes.interface]
// r[impl n8nclone.workflow_execution.llm_operating_graph.scenario.production-file-effects]
fn default_file_system() -> Box<dyn FileSystem> {
    Box::new(crate::file_node::LocalFs)
}

// r[impl n8nclone.workflow_execution.agent_nodes.production_adapters.default_executor]
fn default_agent_adapter() -> Box<dyn AgentAdapter> {
    Box::<ProductionAgentAdapter>::default()
}

fn default_subworkflow_runner_shell() -> Box<dyn SubworkflowRunnerShell> {
    Box::new(crate::engine::child::UnavailableSubworkflowRunnerShell)
}

impl Executor {
    fn with_components(components: RuntimeParts) -> Self {
        Self {
            next_run_id: 0,
            credential_provider: components.credential_provider,
            http_client: components.http_client,
            file_system: components.file_system,
            image_io: components.image_io,
            agent_adapter: components.agent_adapter,
            local_process: components.local_process,
            subworkflow_runner_shell: default_subworkflow_runner_shell(),
            cancellation: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    pub fn new() -> Self {
        Self::with_components(RuntimeParts {
            credential_provider: Box::<EmptyCredentialProvider>::default(),
            http_client: Box::<NoopHttpClient>::default(),
            file_system: default_file_system(),
            image_io: default_image_io(),
            agent_adapter: default_agent_adapter(),
            local_process: LocalProcessConfig::default(),
        })
    }

    pub fn new_with_credential_provider(credential_provider: Box<dyn CredentialProvider>) -> Self {
        Self::with_components(RuntimeParts {
            credential_provider,
            http_client: Box::<NoopHttpClient>::default(),
            file_system: default_file_system(),
            image_io: default_image_io(),
            agent_adapter: default_agent_adapter(),
            local_process: LocalProcessConfig::default(),
        })
    }

    // r[impl n8nclone.workflow_execution.agent_nodes.production_adapters.config]
    pub fn new_with_agent_runtime_config(agent_runtime: AgentRuntimeConfig) -> Self {
        let credential_env_keys: Vec<String> = agent_runtime
            .openai_compatible
            .values()
            .filter_map(|runtime| runtime.api_key_env.clone())
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();
        let credential_provider =
            crate::credentials::EnvironmentCredentialProvider::from_current_env(&credential_env_keys);
        let local_process = agent_runtime.local_process.clone();
        Self::with_components(RuntimeParts {
            credential_provider: Box::new(credential_provider),
            http_client: Box::<NoopHttpClient>::default(),
            file_system: default_file_system(),
            image_io: default_image_io(),
            agent_adapter: Box::new(ProductionAgentAdapter::new(agent_runtime)),
            local_process,
        })
    }

    pub fn new_with_http_client(http_client: Box<dyn HttpClient>) -> Self {
        Self::with_components(RuntimeParts {
            credential_provider: Box::<EmptyCredentialProvider>::default(),
            http_client,
            file_system: default_file_system(),
            image_io: default_image_io(),
            agent_adapter: default_agent_adapter(),
            local_process: LocalProcessConfig::default(),
        })
    }

    pub fn new_with_file_system(file_system: Box<dyn FileSystem>) -> Self {
        Self::with_components(RuntimeParts {
            credential_provider: Box::<EmptyCredentialProvider>::default(),
            http_client: Box::<NoopHttpClient>::default(),
            file_system,
            image_io: default_image_io(),
            agent_adapter: default_agent_adapter(),
            local_process: LocalProcessConfig::default(),
        })
    }

    pub fn new_with_image_io(image_io: Box<dyn ImageIo>) -> Self {
        Self::with_components(RuntimeParts {
            credential_provider: Box::<EmptyCredentialProvider>::default(),
            http_client: Box::<NoopHttpClient>::default(),
            file_system: default_file_system(),
            image_io,
            agent_adapter: default_agent_adapter(),
            local_process: LocalProcessConfig::default(),
        })
    }

    pub fn new_with_agent_adapter(agent_adapter: Box<dyn AgentAdapter>) -> Self {
        Self::with_components(RuntimeParts {
            credential_provider: Box::<EmptyCredentialProvider>::default(),
            http_client: Box::<NoopHttpClient>::default(),
            file_system: default_file_system(),
            image_io: default_image_io(),
            agent_adapter,
            local_process: LocalProcessConfig::default(),
        })
    }

    pub fn new_with_subworkflow_runner_shell(subworkflow_runner_shell: Box<dyn SubworkflowRunnerShell>) -> Self {
        Self::new().with_subworkflow_runner_shell(subworkflow_runner_shell)
    }

    pub fn with_subworkflow_runner_shell(mut self, subworkflow_runner_shell: Box<dyn SubworkflowRunnerShell>) -> Self {
        self.subworkflow_runner_shell = subworkflow_runner_shell;
        self
    }

    pub fn with_cancellation(mut self, cancellation: std::sync::Arc<std::sync::atomic::AtomicBool>) -> Self {
        self.cancellation = cancellation;
        self
    }

    pub fn new_with_clients(
        credential_provider: Box<dyn CredentialProvider>,
        http_client: Box<dyn HttpClient>,
        file_system: Box<dyn FileSystem>,
    ) -> Self {
        Self::with_components(RuntimeParts {
            credential_provider,
            http_client,
            file_system,
            image_io: default_image_io(),
            agent_adapter: default_agent_adapter(),
            local_process: LocalProcessConfig::default(),
        })
    }

    pub fn new_with_runtime_clients(
        credential_provider: Box<dyn CredentialProvider>,
        http_client: Box<dyn HttpClient>,
        file_system: Box<dyn FileSystem>,
        image_io: Box<dyn ImageIo>,
        agent_adapter: Box<dyn AgentAdapter>,
    ) -> Self {
        Self::with_components(RuntimeParts {
            credential_provider,
            http_client,
            file_system,
            image_io,
            agent_adapter,
            local_process: LocalProcessConfig::default(),
        })
    }

    /// Execute a single node given its input values.
    pub async fn execute_node(
        &self,
        workflow: &Workflow,
        node_id: NodeId,
        inputs: &HashMap<PortId, Value>,
    ) -> StepResult {
        self.execute_node_with_expression_context(
            workflow,
            node_id,
            inputs,
            &ExpressionContext::new(HashMap::new()),
            None,
        )
        .await
    }

    async fn execute_node_with_expression_context(
        &self,
        workflow: &Workflow,
        node_id: NodeId,
        inputs: &HashMap<PortId, Value>,
        expression_context: &ExpressionContext,
        parent_run_id: Option<RunId>,
    ) -> StepResult {
        let node = match workflow.graph.node(node_id) {
            Some(n) => n,
            None => {
                return StepResult {
                    node_id,
                    status: NodeStatus::Failed {
                        error: "node not found in graph".into(),
                    },
                    input_hash: ContentHash::from_bytes(b""),
                };
            }
        };

        let fallback_input_hash = input_values_hash(inputs);

        let config = match workflow.node_config(node_id) {
            Some(c) => c,
            None => {
                return StepResult {
                    node_id,
                    status: NodeStatus::Failed {
                        error: format!("no config for node {:?} '{}'", node_id, node.label),
                    },
                    input_hash: fallback_input_hash,
                };
            }
        };
        let node_credentials = workflow.node_credentials(node_id);
        let input_hash = node_execution_cache_key_with_credentials(node_id, config, node_credentials, inputs);

        info!(node = %node.label, "executing node");

        match config {
            NodeConfig::Shell {
                command,
                cwd,
                env,
                timeout_secs,
                pipe_stdin,
            } => {
                let output_ports = match shell_output_port_ids(node) {
                    Ok(ports) => ports,
                    Err(error) => {
                        return StepResult {
                            node_id,
                            status: NodeStatus::Failed { error },
                            input_hash,
                        };
                    }
                };
                let secret_env = match credentials::resolve_secret_env(
                    node_credentials.unwrap_or(&NodeCredentialRefs::default()),
                    self.credential_provider.as_ref(),
                ) {
                    Ok(secret_env) => secret_env,
                    Err(error) => {
                        return StepResult {
                            node_id,
                            status: NodeStatus::Failed {
                                error: error.to_string(),
                            },
                            input_hash,
                        };
                    }
                };
                let resolved_command = match resolve_config_string(command, expression_context) {
                    Ok(value) => value,
                    Err(error) => {
                        return StepResult {
                            node_id,
                            status: NodeStatus::Failed { error },
                            input_hash,
                        };
                    }
                };
                let resolved_cwd = match resolve_optional_config_string(cwd, expression_context) {
                    Ok(value) => value,
                    Err(error) => {
                        return StepResult {
                            node_id,
                            status: NodeStatus::Failed { error },
                            input_hash,
                        };
                    }
                };
                let (resolved_env, env_sensitive_fragments) = match resolve_config_map(env, expression_context) {
                    Ok(value) => value,
                    Err(error) => {
                        return StepResult {
                            node_id,
                            status: NodeStatus::Failed { error },
                            input_hash,
                        };
                    }
                };
                let mut sensitive_fragments = secret_env.secret_values.clone();
                sensitive_fragments.extend(resolved_command.sensitive_fragments.clone());
                sensitive_fragments.extend(env_sensitive_fragments);
                sensitive_fragments.extend(collect_input_sensitive_fragments(inputs));
                let resolved_cwd_value = resolved_cwd.map(|value| {
                    sensitive_fragments.extend(value.sensitive_fragments);
                    value.value
                });
                let merged_env = merged_shell_env(&resolved_env, &secret_env.env);
                let stdin_value = shell_stdin_value(node, inputs);
                let result = execute_shell(
                    ShellExecutionInput {
                        command: &resolved_command.value,
                        cwd: &resolved_cwd_value,
                        environment: &merged_env,
                        timeout_secs: *timeout_secs,
                        should_pipe_stdin: *pipe_stdin,
                        stdin_value,
                    },
                    &self.local_process,
                    std::sync::Arc::clone(&self.cancellation),
                )
                .await;

                match result {
                    Ok(shell_outputs) => completed_step_result(
                        node_id,
                        input_hash,
                        node,
                        shell_outputs_by_real_port(
                            output_ports,
                            redact_shell_outputs(shell_outputs, &sensitive_fragments),
                        ),
                        false,
                    ),
                    Err(e) => StepResult {
                        node_id,
                        status: NodeStatus::Failed { error: e.to_string() },
                        input_hash,
                    },
                }
            }
            NodeConfig::Http {
                method,
                url,
                headers,
                body,
                timeout_secs,
            } => {
                let output_ports = match http_output_port_ids(node) {
                    Ok(ports) => ports,
                    Err(error) => {
                        return StepResult {
                            node_id,
                            status: NodeStatus::Failed { error },
                            input_hash,
                        };
                    }
                };
                let resolved_method = match resolve_config_string(method, expression_context) {
                    Ok(value) => value,
                    Err(error) => {
                        return StepResult {
                            node_id,
                            status: NodeStatus::Failed { error },
                            input_hash,
                        };
                    }
                };
                let resolved_url = match resolve_config_string(url, expression_context) {
                    Ok(value) => value,
                    Err(error) => {
                        return StepResult {
                            node_id,
                            status: NodeStatus::Failed { error },
                            input_hash,
                        };
                    }
                };
                let (resolved_headers, _) = match resolve_config_map(headers, expression_context) {
                    Ok(value) => value,
                    Err(error) => {
                        return StepResult {
                            node_id,
                            status: NodeStatus::Failed { error },
                            input_hash,
                        };
                    }
                };
                let resolved_body = match resolve_optional_config_string(body, expression_context) {
                    Ok(value) => value.map(|value| value.value),
                    Err(error) => {
                        return StepResult {
                            node_id,
                            status: NodeStatus::Failed { error },
                            input_hash,
                        };
                    }
                };
                let request = http_request_plan(
                    node,
                    &resolved_method.value,
                    &resolved_url.value,
                    &resolved_headers,
                    &resolved_body,
                    *timeout_secs,
                    inputs,
                );
                match self.http_client.execute(&request) {
                    Ok(response) => completed_step_result(
                        node_id,
                        input_hash,
                        node,
                        http_outputs_by_real_port(output_ports, response),
                        false,
                    ),
                    Err(error) => StepResult {
                        node_id,
                        status: NodeStatus::Failed {
                            error: error.to_string(),
                        },
                        input_hash,
                    },
                }
            }
            NodeConfig::File {
                operation,
                path,
                content,
                overwrite,
                allowed_roots,
            } => {
                let output_ports = match file_output_port_ids(node) {
                    Ok(ports) => ports,
                    Err(error) => {
                        return StepResult {
                            node_id,
                            status: NodeStatus::Failed { error },
                            input_hash,
                        };
                    }
                };
                let resolved_path = match resolve_config_string(path, expression_context) {
                    Ok(value) => value,
                    Err(error) => {
                        return StepResult {
                            node_id,
                            status: NodeStatus::Failed { error },
                            input_hash,
                        };
                    }
                };
                let resolved_content = match resolve_optional_config_string(content, expression_context) {
                    Ok(value) => value.map(|value| value.value),
                    Err(error) => {
                        return StepResult {
                            node_id,
                            status: NodeStatus::Failed { error },
                            input_hash,
                        };
                    }
                };
                let mut resolved_roots = Vec::new();
                for root in allowed_roots {
                    match resolve_config_string(root, expression_context) {
                        Ok(value) => resolved_roots.push(value.value),
                        Err(error) => {
                            return StepResult {
                                node_id,
                                status: NodeStatus::Failed { error },
                                input_hash,
                            };
                        }
                    }
                }
                let plan = file_operation_plan(
                    node,
                    *operation,
                    &resolved_path.value,
                    &resolved_content,
                    *overwrite,
                    &resolved_roots,
                    inputs,
                );
                match self.file_system.execute(&plan) {
                    Ok(result) => completed_step_result(
                        node_id,
                        input_hash,
                        node,
                        file_outputs_by_real_port(output_ports, &resolved_path.value, result),
                        false,
                    ),
                    Err(error) => StepResult {
                        node_id,
                        status: NodeStatus::Failed {
                            error: error.to_string(),
                        },
                        input_hash,
                    },
                }
            }
            NodeConfig::ImageInput {
                path,
                allowed_roots,
                max_bytes,
            } => {
                let output_ports = match image_output_port_ids(node) {
                    Ok(ports) => ports,
                    Err(error) => {
                        return StepResult {
                            node_id,
                            status: NodeStatus::Failed { error },
                            input_hash,
                        };
                    }
                };
                let resolved_path = match resolve_config_string(path, expression_context) {
                    Ok(value) => value,
                    Err(error) => {
                        return StepResult {
                            node_id,
                            status: NodeStatus::Failed { error },
                            input_hash,
                        };
                    }
                };
                let resolved_roots = match resolve_config_list(allowed_roots, expression_context) {
                    Ok(value) => value,
                    Err(error) => {
                        return StepResult {
                            node_id,
                            status: NodeStatus::Failed { error },
                            input_hash,
                        };
                    }
                };
                let plan = image_node::image_input_plan(&resolved_path.value, &resolved_roots, *max_bytes);
                match self.image_io.load(&plan) {
                    Ok(image) => completed_step_result(
                        node_id,
                        input_hash,
                        node,
                        image_outputs_by_real_port(output_ports, image),
                        false,
                    ),
                    Err(error) => StepResult {
                        node_id,
                        status: NodeStatus::Failed {
                            error: error.to_string(),
                        },
                        input_hash,
                    },
                }
            }
            NodeConfig::Agent(settings) => {
                let secret_env = match credentials::resolve_secret_env(
                    node_credentials.unwrap_or(&NodeCredentialRefs::default()),
                    self.credential_provider.as_ref(),
                ) {
                    Ok(secret_env) => secret_env,
                    Err(error) => {
                        return StepResult {
                            node_id,
                            status: NodeStatus::Failed {
                                error: error.to_string(),
                            },
                            input_hash,
                        };
                    }
                };
                let output_ports = match agent_output_port_ids(node) {
                    Ok(ports) => ports,
                    Err(error) => {
                        return StepResult {
                            node_id,
                            status: NodeStatus::Failed { error },
                            input_hash,
                        };
                    }
                };
                let resolved_prompt = match resolve_config_string(&settings.prompt, expression_context) {
                    Ok(value) => value,
                    Err(error) => {
                        return StepResult {
                            node_id,
                            status: NodeStatus::Failed { error },
                            input_hash,
                        };
                    }
                };
                let resolved_model_hint = match resolve_optional_config_string(&settings.model_hint, expression_context)
                {
                    Ok(value) => value.map(|value| value.value),
                    Err(error) => {
                        return StepResult {
                            node_id,
                            status: NodeStatus::Failed { error },
                            input_hash,
                        };
                    }
                };
                let resolved_tool_policy = match resolve_config_string(&settings.tool_policy, expression_context) {
                    Ok(value) => value.value,
                    Err(error) => {
                        return StepResult {
                            node_id,
                            status: NodeStatus::Failed { error },
                            input_hash,
                        };
                    }
                };
                let resolved_context_refs = match resolve_config_list(&settings.context_refs, expression_context) {
                    Ok(value) => value,
                    Err(error) => {
                        return StepResult {
                            node_id,
                            status: NodeStatus::Failed { error },
                            input_hash,
                        };
                    }
                };
                let resolved_cwd = match resolve_optional_config_string(&settings.cwd, expression_context) {
                    Ok(value) => value.map(|value| value.value),
                    Err(error) => {
                        return StepResult {
                            node_id,
                            status: NodeStatus::Failed { error },
                            input_hash,
                        };
                    }
                };
                let resolved_session = match resolve_optional_config_string(&settings.session, expression_context) {
                    Ok(value) => value.map(|value| value.value),
                    Err(error) => {
                        return StepResult {
                            node_id,
                            status: NodeStatus::Failed { error },
                            input_hash,
                        };
                    }
                };
                let request = agent_request_from_settings(
                    settings,
                    resolved_prompt.value,
                    resolved_model_hint,
                    resolved_tool_policy,
                    resolved_context_refs,
                    resolved_cwd,
                    resolved_session,
                    inputs,
                );
                let agent_context = crate::agent_node::ExecutionContext::new(secret_env.env);
                match self.agent_adapter.execute(&request, &agent_context) {
                    Ok(response) => match agent_node::validate_agent_response_mode(&response, settings.output_mode) {
                        Ok(()) => completed_step_result(
                            node_id,
                            input_hash,
                            node,
                            agent_outputs_by_real_port(output_ports, response),
                            false,
                        ),
                        Err(error) => StepResult {
                            node_id,
                            status: NodeStatus::Failed {
                                error: error.to_string(),
                            },
                            input_hash,
                        },
                    },
                    Err(error) => StepResult {
                        node_id,
                        status: NodeStatus::Failed {
                            error: error.to_string(),
                        },
                        input_hash,
                    },
                }
            }
            NodeConfig::Condition(config) => {
                let value = first_input_value(inputs);
                let matched_labels = control_flow::evaluate_condition(config, value);
                completed_step_result(
                    node_id,
                    input_hash,
                    node,
                    condition_outputs_by_real_port(node, &matched_labels, value),
                    false,
                )
            }
            NodeConfig::Merge(config) => {
                let decision = match control_flow::merge_decision(config, merge_inputs_by_label(node, inputs)) {
                    Ok(decision) => decision,
                    Err(error) => {
                        return StepResult {
                            node_id,
                            status: NodeStatus::Failed {
                                error: error.to_string(),
                            },
                            input_hash,
                        };
                    }
                };
                if !decision.ready {
                    return StepResult {
                        node_id,
                        status: NodeStatus::Skipped,
                        input_hash,
                    };
                }
                completed_step_result(
                    node_id,
                    input_hash,
                    node,
                    merge_outputs_by_real_port(node, config.policy, decision.selected),
                    false,
                )
            }
            NodeConfig::Subworkflow(config) => {
                if let Err(error) =
                    control_flow::validate_subworkflow_admission(config, std::slice::from_ref(&workflow.meta.name))
                {
                    return StepResult {
                        node_id,
                        status: NodeStatus::Failed {
                            error: error.to_string(),
                        },
                        input_hash,
                    };
                }
                let Some(parent_run_id) = parent_run_id else {
                    return StepResult {
                        node_id,
                        status: NodeStatus::Failed {
                            error: "subworkflow execution requires a parent run identity".to_string(),
                        },
                        input_hash,
                    };
                };
                let invocation = match crate::engine::child::build_invocation(
                    &workflow.meta.name,
                    parent_run_id,
                    node,
                    config,
                    inputs,
                ) {
                    Ok(invocation) => invocation,
                    Err(error) => {
                        return StepResult {
                            node_id,
                            status: NodeStatus::Failed {
                                error: error.to_string(),
                            },
                            input_hash,
                        };
                    }
                };
                let execution = match self.subworkflow_runner_shell.execute(&invocation).await {
                    Ok(execution) => execution,
                    Err(error) => {
                        return StepResult {
                            node_id,
                            status: NodeStatus::Failed {
                                error: error.to_string(),
                            },
                            input_hash,
                        };
                    }
                };
                match crate::engine::child::validate_and_map_execution(node, config, &invocation, execution) {
                    Ok(outputs) => completed_step_result(node_id, input_hash, node, outputs, false),
                    Err(error) => StepResult {
                        node_id,
                        status: NodeStatus::Failed {
                            error: error.to_string(),
                        },
                        input_hash,
                    },
                }
            }
        }
    }

    /// Run the full workflow (topological order, parallel where possible).
    pub async fn run(
        &mut self,
        workflow: &Workflow,
        cache: &HashMap<ContentHash, HashMap<PortId, Value>>,
    ) -> (RunId, Vec<StepResult>) {
        self.run_with_bound_inputs(workflow, cache, &HashMap::new()).await
    }

    pub async fn run_with_named_inputs(
        &mut self,
        workflow: &Workflow,
        cache: &HashMap<ContentHash, HashMap<PortId, Value>>,
        named_inputs: &BTreeMap<String, Value>,
    ) -> Result<(RunId, Vec<StepResult>), crate::engine::child::SubworkflowExecutionError> {
        let bound_inputs = crate::engine::child::bind_named_inputs(workflow, named_inputs)?;
        Ok(self.run_with_bound_inputs(workflow, cache, &bound_inputs).await)
    }

    async fn run_with_bound_inputs(
        &mut self,
        workflow: &Workflow,
        cache: &HashMap<ContentHash, HashMap<PortId, Value>>,
        bound_inputs: &HashMap<PortId, Value>,
    ) -> (RunId, Vec<StepResult>) {
        let run_id = RunId(self.next_run_id);
        self.next_run_id += 1;

        let (runner_plan, topo) = match admitted_runner_execution(workflow) {
            Some(admitted) => admitted,
            None => {
                error!("workflow failed shared graph admission, cannot execute");
                return (run_id, Vec::new());
            }
        };

        info!(run = %run_id, nodes = topo.len(), "starting workflow execution");

        let maximum_step_count = topo.len();
        let mut results = Vec::with_capacity(maximum_step_count);
        let mut port_values: HashMap<PortId, Value> = HashMap::new();
        let mut runner_states = BTreeMap::new();

        for node_id in topo {
            assert!(
                runner_states.len() < maximum_step_count,
                "runner state count must remain bounded by admitted nodes"
            );

            // Gather inputs from upstream edges and admitted workflow-bound inputs.
            let mut inputs = gather_inputs(workflow, node_id, &port_values);
            if let Some(node) = workflow.graph.node(node_id) {
                for port in &node.input_ports {
                    if let Some(value) = bound_inputs.get(&port.id) {
                        inputs.insert(port.id, value.clone());
                    }
                }
            }

            // Check cache.
            let input_hash = match workflow.node_config(node_id) {
                Some(config) => node_execution_cache_key_with_credentials(
                    node_id,
                    config,
                    workflow.node_credentials(node_id),
                    &inputs,
                ),
                None => input_values_hash(&inputs),
            };

            let decision = runner::durable_runner_decision(&runner_plan, &runner_states, node_id.0);
            if decision != runner::DurableRunnerDecision::Execute {
                debug!(node = ?node_id, ?decision, "shared graph lifecycle blocked node execution");
                runner_states.insert(node_id.0, runner::RunnerStepState::Cancelled);
                results.push(StepResult {
                    node_id,
                    status: NodeStatus::Skipped,
                    input_hash,
                });
                continue;
            }

            if let Some(cached_outputs) = cache.get(&input_hash) {
                debug!(node = ?node_id, "cache hit, skipping execution");
                let Some(node) = workflow.graph.node(node_id) else {
                    continue;
                };
                let step = completed_step_result(node_id, input_hash, node, cached_outputs.clone(), true);
                if let NodeStatus::Completed { ref outputs, .. } = step.status {
                    for (port_id, value) in outputs {
                        port_values.insert(*port_id, value.clone());
                    }
                }
                runner_states.insert(node_id.0, runner::RunnerStepState::Completed);
                results.push(step);
                continue;
            }

            let expression_context = expression_context_from_workflow_outputs(workflow, &port_values);
            let step = self
                .execute_node_with_expression_context(workflow, node_id, &inputs, &expression_context, Some(run_id))
                .await;

            // Store outputs for downstream nodes.
            if let NodeStatus::Completed { ref outputs, .. } = step.status {
                for (port_id, value) in outputs {
                    port_values.insert(*port_id, value.clone());
                }
            }

            runner_states.insert(node_id.0, runner_step_state(&step.status));
            results.push(step);
        }

        info!(run = %run_id, steps = results.len(), "workflow execution complete");
        (run_id, results)
    }
}

impl Default for Executor {
    fn default() -> Self {
        Self::new()
    }
}

fn merged_shell_env(base: &HashMap<String, String>, secret_env: &HashMap<String, String>) -> HashMap<String, String> {
    let mut merged = base.clone();
    for (key, value) in secret_env {
        merged.insert(key.clone(), value.clone());
    }
    merged
}

fn redact_shell_outputs(outputs: ShellOutputValues, secret_values: &[String]) -> ShellOutputValues {
    ShellOutputValues {
        stdout: credentials::redact_text(&outputs.stdout, secret_values),
        stderr: credentials::redact_text(&outputs.stderr, secret_values),
        exit_code: outputs.exit_code,
    }
}

struct ShellExecutionInput<'a> {
    command: &'a str,
    cwd: &'a Option<String>,
    environment: &'a HashMap<String, String>,
    timeout_secs: u64,
    should_pipe_stdin: bool,
    stdin_value: Option<&'a Value>,
}

/// Execute a shell command through the shared blocking bounded-process shell.
// r[impl lattice.workflow_execution.bounded_process_compatibility]
async fn execute_shell(
    input: ShellExecutionInput<'_>,
    config: &LocalProcessConfig,
    cancellation: std::sync::Arc<std::sync::atomic::AtomicBool>,
) -> Result<ShellOutputValues, crate::agent_node::ProcessError> {
    let stdin = if input.should_pipe_stdin {
        input.stdin_value.map(Value::as_display_string).unwrap_or_default().into_bytes()
    } else {
        Vec::new()
    };
    let request = crate::agent_node::ProcessRequest {
        program: config.shell_program.clone(),
        args: vec![SHELL_COMMAND_FLAG.to_owned(), input.command.to_owned()],
        cwd: input.cwd.clone(),
        env: input.environment.iter().map(|(key, value)| (key.clone(), value.clone())).collect(),
        stdin,
        timeout_secs: input.timeout_secs,
        output_limit_bytes: config.limits.stdout_max_bytes.max(config.limits.stderr_max_bytes),
    };
    debug_assert_eq!(request.args.len(), SHELL_ARGUMENT_COUNT);
    debug_assert_eq!(request.program, config.shell_program);
    let runner = crate::agent_node::ProductionProcessRunner::new(config.clone());
    let output = tokio::task::spawn_blocking(move || runner.run_with_cancellation(&request, &cancellation))
        .await
        .map_err(|_| crate::agent_node::ProcessError::Spawn(PROCESS_TASK_FAILURE_DIAGNOSTIC.to_owned()))??;

    Ok(ShellOutputValues {
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        exit_code: output.status_code.unwrap_or(UNKNOWN_EXIT_CODE),
    })
}

/// Gather input values for a node from upstream edge outputs.
fn gather_inputs(workflow: &Workflow, node_id: NodeId, port_values: &HashMap<PortId, Value>) -> HashMap<PortId, Value> {
    let mut inputs = HashMap::new();

    let node = match workflow.graph.node(node_id) {
        Some(n) => n,
        None => return inputs,
    };

    for input_port in &node.input_ports {
        // Find the edge targeting this input port.
        for edge in workflow.graph.edges() {
            if edge.target == input_port.id
                && let Some(value) = port_values.get(&edge.source)
            {
                inputs.insert(input_port.id, value.clone());
            }
        }
    }

    inputs
}

fn runner_step_state(status: &NodeStatus) -> runner::RunnerStepState {
    match status {
        NodeStatus::Completed { .. } => runner::RunnerStepState::Completed,
        NodeStatus::Failed { .. } => runner::RunnerStepState::Failed,
        NodeStatus::Skipped => runner::RunnerStepState::Cancelled,
        NodeStatus::Pending => runner::RunnerStepState::Pending,
        NodeStatus::Running => runner::RunnerStepState::Running,
    }
}

/// Admit the workflow through the shared graph core and return its stable order.
// r[impl n8nclone.workflow_execution.runner_core.compatibility]
// r[impl n8nclone.workflow_execution.trait_boundaries.compatibility]
// r[impl n8nclone.workflow_execution.seaglass_graph_core]
fn admitted_runner_execution(workflow: &Workflow) -> Option<(runner::RunnerWorkflow, Vec<NodeId>)> {
    let plan = runner::build_runner_plan(workflow).plan?;
    match runner::admit_runner_plan(&plan) {
        runner::RunnerAdmission::Accepted { topo_order } => Some((plan, topo_order.into_iter().map(NodeId).collect())),
        runner::RunnerAdmission::Rejected { .. } => None,
    }
}

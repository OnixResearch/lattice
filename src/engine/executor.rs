//! DAG executor — runs workflows in topological order.
//!
//! Two modes:
//!  - **Run**: async parallel execution of independent branches via tokio.
//!  - **Debug**: step-through, one node at a time, with pause after each.

use std::collections::HashMap;

use rat_nodegraph::model::Node;
use rat_nodegraph::model::NodeId;
use rat_nodegraph::model::PortId;
use serde::Deserialize;
use serde::Serialize;
use tokio::process::Command;
use tracing::debug;
use tracing::error;
use tracing::info;

use crate::credentials;
use crate::credentials::CredentialProvider;
use crate::credentials::EmptyCredentialProvider;
use crate::credentials::NodeCredentialRefs;
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
use crate::file_node::NoopFileSystem;
use crate::http;
use crate::http::HttpClient;
use crate::http::HttpRequestPlan;
use crate::http::NoopHttpClient;

const SHELL_STDOUT_OUTPUT_INDEX: usize = 0;
const SHELL_STDERR_OUTPUT_INDEX: usize = 1;
const SHELL_EXIT_OUTPUT_INDEX: usize = 2;
const SHELL_STDIN_INPUT_LABEL: &str = "stdin";
const HTTP_BODY_INPUT_LABEL: &str = "body";
const FILE_CONTENT_INPUT_LABEL: &str = "content";
const FILE_CONTENT_OUTPUT_INDEX: usize = 0;
const FILE_METADATA_OUTPUT_INDEX: usize = 1;
const FILE_BYTES_OUTPUT_INDEX: usize = 2;
const HTTP_STATUS_OUTPUT_INDEX: usize = 0;
const HTTP_HEADERS_OUTPUT_INDEX: usize = 1;
const HTTP_BODY_OUTPUT_INDEX: usize = 2;
const HASH_TAG_NODE_EXECUTION: &[u8] = b"node-execution:v1";
const HASH_TAG_NODE_ID: &[u8] = b"node-id";
const HASH_TAG_SHELL_CONFIG: &[u8] = b"shell-config";
const HASH_TAG_HTTP_CONFIG: &[u8] = b"http-config";
const HASH_TAG_FILE_CONFIG: &[u8] = b"file-config";
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
    }
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

fn collect_input_sensitive_fragments(inputs: &HashMap<PortId, Value>) -> Vec<String> {
    inputs.values().flat_map(|value| value.sensitive_fragments.clone()).collect()
}

fn expression_context_from_workflow_outputs(
    workflow: &Workflow,
    port_values: &HashMap<PortId, Value>,
) -> ExpressionContext {
    let mut context = ExpressionContext::default();
    for node in workflow.graph.nodes() {
        for port in &node.output_ports {
            if let Some(value) = port_values.get(&port.id) {
                context.insert(node.id.0, port.id.0, value.clone());
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

/// The executor.
pub struct Executor {
    next_run_id: u64,
    credential_provider: Box<dyn CredentialProvider>,
    http_client: Box<dyn HttpClient>,
    file_system: Box<dyn FileSystem>,
}

impl Executor {
    pub fn new() -> Self {
        Self {
            next_run_id: 0,
            credential_provider: Box::<EmptyCredentialProvider>::default(),
            http_client: Box::<NoopHttpClient>::default(),
            file_system: Box::<NoopFileSystem>::default(),
        }
    }

    pub fn new_with_credential_provider(credential_provider: Box<dyn CredentialProvider>) -> Self {
        Self {
            next_run_id: 0,
            credential_provider,
            http_client: Box::<NoopHttpClient>::default(),
            file_system: Box::<NoopFileSystem>::default(),
        }
    }

    pub fn new_with_http_client(http_client: Box<dyn HttpClient>) -> Self {
        Self {
            next_run_id: 0,
            credential_provider: Box::<EmptyCredentialProvider>::default(),
            http_client,
            file_system: Box::<NoopFileSystem>::default(),
        }
    }

    pub fn new_with_file_system(file_system: Box<dyn FileSystem>) -> Self {
        Self {
            next_run_id: 0,
            credential_provider: Box::<EmptyCredentialProvider>::default(),
            http_client: Box::<NoopHttpClient>::default(),
            file_system,
        }
    }

    pub fn new_with_clients(
        credential_provider: Box<dyn CredentialProvider>,
        http_client: Box<dyn HttpClient>,
        file_system: Box<dyn FileSystem>,
    ) -> Self {
        Self {
            next_run_id: 0,
            credential_provider,
            http_client,
            file_system,
        }
    }

    /// Execute a single node given its input values.
    pub async fn execute_node(
        &self,
        workflow: &Workflow,
        node_id: NodeId,
        inputs: &HashMap<PortId, Value>,
    ) -> StepResult {
        self.execute_node_with_expression_context(workflow, node_id, inputs, &ExpressionContext::default())
            .await
    }

    async fn execute_node_with_expression_context(
        &self,
        workflow: &Workflow,
        node_id: NodeId,
        inputs: &HashMap<PortId, Value>,
        expression_context: &ExpressionContext,
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
                    &resolved_command.value,
                    &resolved_cwd_value,
                    &merged_env,
                    *timeout_secs,
                    *pipe_stdin,
                    stdin_value,
                )
                .await;

                match result {
                    Ok(shell_outputs) => StepResult {
                        node_id,
                        status: NodeStatus::Completed {
                            outputs: shell_outputs_by_real_port(
                                output_ports,
                                redact_shell_outputs(shell_outputs, &sensitive_fragments),
                            ),
                            cached: false,
                        },
                        input_hash,
                    },
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
                    Ok(response) => StepResult {
                        node_id,
                        status: NodeStatus::Completed {
                            outputs: http_outputs_by_real_port(output_ports, response),
                            cached: false,
                        },
                        input_hash,
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
                    Ok(result) => StepResult {
                        node_id,
                        status: NodeStatus::Completed {
                            outputs: file_outputs_by_real_port(output_ports, &resolved_path.value, result),
                            cached: false,
                        },
                        input_hash,
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
        }
    }

    /// Run the full workflow (topological order, parallel where possible).
    pub async fn run(
        &mut self,
        workflow: &Workflow,
        cache: &HashMap<ContentHash, HashMap<PortId, Value>>,
    ) -> (RunId, Vec<StepResult>) {
        let run_id = RunId(self.next_run_id);
        self.next_run_id += 1;

        let topo = match topological_sort(workflow) {
            Some(order) => order,
            None => {
                error!("workflow contains cycles, cannot execute");
                return (run_id, Vec::new());
            }
        };

        info!(run = %run_id, nodes = topo.len(), "starting workflow execution");

        let mut results = Vec::new();
        let mut port_values: HashMap<PortId, Value> = HashMap::new();

        for node_id in topo {
            // Gather inputs from upstream edges.
            let inputs = gather_inputs(workflow, node_id, &port_values);

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

            if let Some(cached_outputs) = cache.get(&input_hash) {
                debug!(node = ?node_id, "cache hit, skipping execution");
                for (port_id, value) in cached_outputs {
                    port_values.insert(*port_id, value.clone());
                }
                results.push(StepResult {
                    node_id,
                    status: NodeStatus::Completed {
                        outputs: cached_outputs.clone(),
                        cached: true,
                    },
                    input_hash,
                });
                continue;
            }

            let expression_context = expression_context_from_workflow_outputs(workflow, &port_values);
            let step = self.execute_node_with_expression_context(workflow, node_id, &inputs, &expression_context).await;

            // Store outputs for downstream nodes.
            if let NodeStatus::Completed { ref outputs, .. } = step.status {
                for (port_id, value) in outputs {
                    port_values.insert(*port_id, value.clone());
                }
            }

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

/// Execute a shell command, returning output values keyed by port.
async fn execute_shell(
    command: &str,
    cwd: &Option<String>,
    env: &HashMap<String, String>,
    timeout_secs: u64,
    pipe_stdin: bool,
    stdin_value: Option<&Value>,
) -> Result<ShellOutputValues, Box<dyn std::error::Error + Send + Sync>> {
    let mut cmd = Command::new("sh");
    cmd.arg("-c").arg(command);

    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }

    for (k, v) in env {
        cmd.env(k, v);
    }

    if pipe_stdin {
        cmd.stdin(std::process::Stdio::piped());
    } else {
        cmd.stdin(std::process::Stdio::null());
    }
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    let mut child = cmd.spawn()?;

    // Pipe stdin from the node's selected stdin input value.
    if pipe_stdin
        && let Some(stdin_val) = stdin_value
        && let Some(ref mut stdin) = child.stdin
    {
        use tokio::io::AsyncWriteExt;
        let data = stdin_val.as_display_string();
        let _ = stdin.write_all(data.as_bytes()).await;
        // Drop stdin to close the pipe.
        drop(child.stdin.take());
    }

    let output = if timeout_secs > 0 {
        tokio::time::timeout(std::time::Duration::from_secs(timeout_secs), child.wait_with_output())
            .await
            .map_err(|_| -> Box<dyn std::error::Error + Send + Sync> { "command timed out".into() })??
    } else {
        child.wait_with_output().await?
    };

    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let exit_code = output.status.code().unwrap_or(-1);

    Ok(ShellOutputValues {
        stdout,
        stderr,
        exit_code,
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

/// Topological sort of the workflow graph. Returns None if cycles exist.
fn topological_sort(workflow: &Workflow) -> Option<Vec<NodeId>> {
    let node_ids = workflow.graph.node_ids();
    let mut in_degree: HashMap<NodeId, usize> = HashMap::new();
    let mut adjacency: HashMap<NodeId, Vec<NodeId>> = HashMap::new();

    for &id in &node_ids {
        in_degree.insert(id, 0);
        adjacency.insert(id, Vec::new());
    }

    for edge in workflow.graph.edges() {
        let src_node = workflow.graph.port_owner(edge.source);
        let tgt_node = workflow.graph.port_owner(edge.target);
        if let (Some(src), Some(tgt)) = (src_node, tgt_node) {
            adjacency.entry(src).or_default().push(tgt);
            *in_degree.entry(tgt).or_default() += 1;
        }
    }

    let mut queue: Vec<NodeId> = in_degree.iter().filter(|(_, deg)| **deg == 0).map(|(&id, _)| id).collect();
    queue.sort(); // deterministic order

    let mut order = Vec::new();

    while let Some(node) = queue.pop() {
        order.push(node);
        if let Some(neighbors) = adjacency.get(&node) {
            for &neighbor in neighbors {
                if let Some(deg) = in_degree.get_mut(&neighbor) {
                    *deg -= 1;
                    if *deg == 0 {
                        queue.push(neighbor);
                    }
                }
            }
        }
    }

    if order.len() == node_ids.len() {
        Some(order)
    } else {
        None
    }
}

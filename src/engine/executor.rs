//! DAG executor — runs workflows in topological order.
//!
//! Two modes:
//!  - **Run**: async parallel execution of independent branches via tokio.
//!  - **Debug**: step-through, one node at a time, with pause after each.

use std::collections::BTreeMap;
use std::collections::HashMap;

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
use crate::engine::workflow::Node;
use crate::engine::workflow::NodeConfig;
use crate::engine::workflow::PortId;
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
const SHELL_SUCCESS_EXIT_CODE: i32 = 0;
const PROCESS_TASK_FAILURE_DIAGNOSTIC: &str = "bounded local process task failed";
const SHELL_ARGUMENT_COUNT: usize = 2;
const CHOREGRAPH_NODE_PREFIX: &str = "lattice-node/";
const EFFECT_TRANSITION: &str = "ready->start";
const EFFECT_RESOURCE_FILE_PREFIX: &str = "lattice://file/";
const EFFECT_RESOURCE_PROCESS_PREFIX: &str = "lattice://process/";
const EFFECT_RESOURCE_NETWORK_PREFIX: &str = "lattice://network/";
const EFFECT_OPERATION_SHELL: &str = "lattice/shell";
const EFFECT_OPERATION_HTTP: &str = "lattice/http";
const EFFECT_OPERATION_FILE: &str = "lattice/file";
const EFFECT_OPERATION_TYPED: &str = "lattice/operation";
const EFFECT_ABILITY_PROCESS: &str = "process/execute";
const EFFECT_ABILITY_NETWORK: &str = "network/request";
const EFFECT_ABILITY_FILE_READ: &str = "file/read";
const EFFECT_ABILITY_FILE_WRITE: &str = "file/write";
const EFFECT_ABILITY_FILE_APPEND: &str = "file/append";
const AUTHORIZED_FILE_ROOT_COUNT: usize = 1;
const BYTES_PER_KIBIBYTE: usize = 1_024;
const KIBIBYTES_PER_MEBIBYTE: usize = 1_024;
const AUTHORIZED_FILE_READ_LIMIT_MEBIBYTES: usize = 64;
const AUTHORIZED_FILE_READ_MAX_BYTES: usize = AUTHORIZED_FILE_READ_LIMIT_MEBIBYTES
    .saturating_mul(KIBIBYTES_PER_MEBIBYTE)
    .saturating_mul(BYTES_PER_KIBIBYTE);
const AUTHORIZED_EFFECT_BINDING_MISMATCH: &str = "authorized effect binding does not match the selected executor node";
const AUTHORIZED_EFFECT_REQUEST_MISMATCH: &str = "authorized effect request does not match the resolved adapter plan";
const AUTHORIZED_FILE_ROOT_MISMATCH: &str = "authorized file effects require one configured root identifier";
const AUTHORIZED_FILE_READ_UTF8_ERROR: &str = "authorized file read output is not valid UTF-8";
const AUTHORIZED_EFFECTFUL_NODE_REQUIRED: &str = "authorized execution requires an effectful shell, HTTP, or file node";
const AUTHORIZED_WORKFLOW_CONTEXT_REQUIRED: &str =
    "product effect execution requires a current Choregraph authority context";
const CHOREGRAPH_EXECUTION_FAILURE: &str = "Choregraph product execution failed";
const APPROVAL_REQUIRES_DURABLE_SHELL: &str = "approval nodes require the durable workflow approval shell";

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
const HASH_TAG_OPERATION_CONFIG: &[u8] = b"operation-config";
const HASH_TAG_CONDITION_CONFIG: &[u8] = b"condition-config";
const HASH_TAG_MERGE_CONFIG: &[u8] = b"merge-config";
const HASH_TAG_SUBWORKFLOW_CONFIG: &[u8] = b"subworkflow-config";
const HASH_TAG_APPROVAL_CONFIG: &[u8] = b"approval-config";
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
const HASH_TAG_EXPRESSION_VALUES: &[u8] = b"expression-values";
const HASH_TAG_EXPRESSION_NODE: &[u8] = b"expression-node";
const HASH_TAG_EXPRESSION_PORT: &[u8] = b"expression-port";
const HASH_TAG_EXPRESSION_VALUE: &[u8] = b"expression-value";
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
pub fn node_execution_cache_key(
    node_id: crate::engine::workflow::NodeId,
    config: &NodeConfig,
    inputs: &HashMap<PortId, Value>,
) -> ContentHash {
    node_execution_cache_key_with_credentials(node_id, config, None, inputs)
}

// r[impl n8nclone.credentials.cache_identity]
pub fn node_execution_cache_key_with_credentials(
    node_id: crate::engine::workflow::NodeId,
    config: &NodeConfig,
    credentials: Option<&NodeCredentialRefs>,
    inputs: &HashMap<PortId, Value>,
) -> ContentHash {
    node_execution_cache_key_with_expression_values(node_id, config, credentials, inputs, &BTreeMap::new())
}

pub(super) fn node_execution_cache_key_with_expression_values(
    node_id: crate::engine::workflow::NodeId,
    config: &NodeConfig,
    credentials: Option<&NodeCredentialRefs>,
    inputs: &HashMap<PortId, Value>,
    expression_values: &BTreeMap<(u64, expressions::ExpressionPortId), Value>,
) -> ContentHash {
    let mut hasher = blake3::Hasher::new();
    hasher.update(HASH_TAG_NODE_EXECUTION);
    update_u64(&mut hasher, HASH_TAG_NODE_ID, node_id.0);
    update_node_config_hash(&mut hasher, config);
    update_secret_refs_hash(&mut hasher, credentials);
    update_inputs_hash(&mut hasher, inputs);
    update_expression_values_hash(&mut hasher, expression_values);
    ContentHash(*hasher.finalize().as_bytes())
}

pub(super) fn input_values_hash(inputs: &HashMap<PortId, Value>) -> ContentHash {
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
        NodeConfig::Operation(plan) => {
            hasher.update(HASH_TAG_OPERATION_CONFIG);
            update_str(hasher, HASH_TAG_OPERATION_CONFIG, &plan.identity);
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
        NodeConfig::Approval(config) => {
            hasher.update(HASH_TAG_APPROVAL_CONFIG);
            update_str(hasher, HASH_TAG_APPROVAL_CONFIG, config.gate_id.as_str());
            update_str(hasher, HASH_TAG_APPROVAL_CONFIG, &config.safe_summary);
            update_ordered_string_list_hash(hasher, HASH_TAG_INPUTS, &config.selected_input_ports);
            for artifact in &config.required_artifacts {
                update_str(hasher, HASH_TAG_APPROVAL_CONFIG, artifact.as_str());
            }
            update_str(hasher, HASH_TAG_APPROVAL_CONFIG, &format!("{:?}", config.authority));
            update_str(hasher, HASH_TAG_APPROVAL_CONFIG, &format!("{:?}", config.expiry));
            update_str(hasher, HASH_TAG_APPROVAL_CONFIG, &format!("{:?}", config.resume));
            update_str(hasher, HASH_TAG_APPROVAL_CONFIG, &format!("{:?}", config.terminal));
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

fn update_expression_values_hash(
    hasher: &mut blake3::Hasher,
    values: &BTreeMap<(u64, expressions::ExpressionPortId), Value>,
) {
    if values.is_empty() {
        return;
    }
    hasher.update(HASH_TAG_EXPRESSION_VALUES);
    update_len(hasher, values.len());
    for ((node_id, port_id), value) in values {
        update_u64(hasher, HASH_TAG_EXPRESSION_NODE, *node_id);
        update_u64(hasher, HASH_TAG_EXPRESSION_PORT, port_id.0);
        hasher.update(HASH_TAG_EXPRESSION_VALUE);
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

// r[impl lattice.workflow_validation.explicit_dependencies.context]
pub(super) fn expression_values_for_consumer(
    plan: &runner::RunnerWorkflow,
    consumer_node: crate::engine::workflow::NodeId,
    port_values: &HashMap<PortId, Value>,
) -> Result<BTreeMap<(u64, expressions::ExpressionPortId), Value>, String> {
    plan.expression_dependencies
        .iter()
        .filter(|dependency| dependency.consumer_node == consumer_node.0)
        .map(|dependency| {
            let port_id = PortId(dependency.producer_output_port.0);
            let value = port_values.get(&port_id).ok_or_else(|| {
                format!(
                    "{}: expression reference node:{}:port:{} is unavailable",
                    dependency.config_field_path, dependency.producer_node, dependency.producer_output_port.0
                )
            })?;
            Ok(((dependency.producer_node, dependency.producer_output_port), value.clone()))
        })
        .collect()
}

pub(super) fn expression_context_from_admitted_values(
    values: &BTreeMap<(u64, expressions::ExpressionPortId), Value>,
) -> ExpressionContext {
    ExpressionContext::new(values.iter().map(|(key, value)| (*key, value.clone())).collect())
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

fn authorized_file_request(
    plan: &FileOperationPlan,
) -> Result<crate::engine::effect::runtime::FileAdapterRequest, String> {
    let [root_id] = plan.allowed_roots.as_slice() else {
        return Err(AUTHORIZED_FILE_ROOT_MISMATCH.to_owned());
    };
    debug_assert_eq!(plan.allowed_roots.len(), AUTHORIZED_FILE_ROOT_COUNT);
    Ok(crate::engine::effect::runtime::FileAdapterRequest {
        root_id: root_id.clone(),
        relative_path: plan.path.clone(),
        content: plan.content.as_ref().map(|content| content.as_bytes().to_vec()),
        overwrite: plan.overwrite,
        read_max_bytes: AUTHORIZED_FILE_READ_MAX_BYTES,
    })
}

fn authorized_file_output(
    output: crate::engine::effect::runtime::FileAdapterOutput,
) -> Result<FileOperationResult, String> {
    match output {
        crate::engine::effect::runtime::FileAdapterOutput::Read(bytes) => {
            let byte_count = bytes.len();
            let content = String::from_utf8(bytes).map_err(|_| AUTHORIZED_FILE_READ_UTF8_ERROR.to_owned())?;
            Ok(FileOperationResult::Read {
                content,
                bytes: byte_count,
            })
        }
        crate::engine::effect::runtime::FileAdapterOutput::Written { bytes } => {
            Ok(FileOperationResult::Written { bytes })
        }
    }
}

pub(super) fn effect_operation(config: &NodeConfig) -> Option<&'static str> {
    match config {
        NodeConfig::Shell { .. } => Some(EFFECT_OPERATION_SHELL),
        NodeConfig::Http { .. } => Some(EFFECT_OPERATION_HTTP),
        NodeConfig::File { .. } => Some(EFFECT_OPERATION_FILE),
        NodeConfig::Operation(_) => Some(EFFECT_OPERATION_TYPED),
        NodeConfig::ImageInput { .. }
        | NodeConfig::Agent(_)
        | NodeConfig::Condition(_)
        | NodeConfig::Merge(_)
        | NodeConfig::Subworkflow(_)
        | NodeConfig::Approval(_) => None,
    }
}

fn effect_ability(config: &NodeConfig) -> Option<&'static str> {
    match config {
        NodeConfig::Shell { .. } => Some(EFFECT_ABILITY_PROCESS),
        NodeConfig::Http { .. } => Some(EFFECT_ABILITY_NETWORK),
        NodeConfig::File { operation, .. } => Some(match operation {
            FileOperationKind::Read => EFFECT_ABILITY_FILE_READ,
            FileOperationKind::Write => EFFECT_ABILITY_FILE_WRITE,
            FileOperationKind::Append => EFFECT_ABILITY_FILE_APPEND,
        }),
        NodeConfig::Operation(_) => Some(EFFECT_ABILITY_PROCESS),
        NodeConfig::ImageInput { .. }
        | NodeConfig::Agent(_)
        | NodeConfig::Condition(_)
        | NodeConfig::Merge(_)
        | NodeConfig::Subworkflow(_)
        | NodeConfig::Approval(_) => None,
    }
}

fn effect_resource(config: &NodeConfig, node_id: crate::engine::workflow::NodeId) -> Option<String> {
    let prefix = match config {
        NodeConfig::Shell { .. } => EFFECT_RESOURCE_PROCESS_PREFIX,
        NodeConfig::Http { .. } => EFFECT_RESOURCE_NETWORK_PREFIX,
        NodeConfig::File { .. } => EFFECT_RESOURCE_FILE_PREFIX,
        NodeConfig::Operation(_) => EFFECT_RESOURCE_PROCESS_PREFIX,
        NodeConfig::ImageInput { .. }
        | NodeConfig::Agent(_)
        | NodeConfig::Condition(_)
        | NodeConfig::Merge(_)
        | NodeConfig::Subworkflow(_)
        | NodeConfig::Approval(_) => return None,
    };
    Some(format!("{prefix}{}", node_id.0))
}

fn authorized_binding_matches_node(
    binding: &crate::engine::effect::authority::IntentBinding,
    node_id: crate::engine::workflow::NodeId,
    config: &NodeConfig,
) -> bool {
    binding.node.as_str() == format!("{CHOREGRAPH_NODE_PREFIX}{}", node_id.0)
        && binding.transition.as_str() == EFFECT_TRANSITION
        && effect_operation(config).is_some_and(|operation| binding.operation.as_str() == operation)
        && effect_ability(config).is_some_and(|ability| binding.ability.as_str() == ability)
        && effect_resource(config, node_id).is_some_and(|resource| binding.resource.as_str() == resource)
}

fn authorized_request_matches_payload(
    binding: &crate::engine::effect::authority::IntentBinding,
    payload_identity: &str,
) -> bool {
    binding.payload.as_str() == payload_identity
        && binding.request.as_str() == crate::engine::effect::runtime::dispatch_request_identity(payload_identity)
}

fn dispatch_error(error: crate::engine::effect::dispatch::Error) -> String {
    format!("effect dispatch failed: {error:?}")
}

pub(super) fn choregraph_execution_error(error: crate::engine::effect::execution::Error) -> String {
    format!("{CHOREGRAPH_EXECUTION_FAILURE}: {error}")
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
pub(super) fn completed_step_result(
    node_id: crate::engine::workflow::NodeId,
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    pub node_id: crate::engine::workflow::NodeId,
    pub status: NodeStatus,
    pub input_hash: ContentHash,
}

#[derive(Debug, Clone)]
pub struct ApprovalSuspensionPoint {
    pub node_id: crate::engine::workflow::NodeId,
    pub inputs: HashMap<PortId, Value>,
}

#[derive(Debug, Clone)]
pub struct ApprovalRunOutcome {
    pub run_id: RunId,
    pub results: Vec<StepResult>,
    pub suspension: Option<ApprovalSuspensionPoint>,
}

pub struct ApprovalResumeInput<'a> {
    pub workflow: &'a Workflow,
    pub cache: &'a HashMap<ContentHash, HashMap<PortId, Value>>,
    pub context: &'a crate::engine::effect::execution::RunContext,
    pub checkpoint: &'a [StepResult],
    pub resumed_gate: crate::engine::workflow::NodeId,
    pub resumed_gate_outputs: &'a HashMap<PortId, Value>,
}

#[derive(Debug, Clone, Copy)]
struct ExecutionControl<'a> {
    checkpoint: &'a [StepResult],
    stop_at_approval: bool,
    resumed_gate: Option<crate::engine::workflow::NodeId>,
    resumed_gate_outputs: Option<&'a HashMap<PortId, Value>>,
}

impl ExecutionControl<'_> {
    const fn ordinary() -> Self {
        Self {
            checkpoint: &[],
            stop_at_approval: false,
            resumed_gate: None,
            resumed_gate_outputs: None,
        }
    }
}

struct RuntimeParts {
    credential_provider: Box<dyn CredentialProvider>,
    http_client: Box<dyn HttpClient>,
    file_system: Box<dyn FileSystem>,
    image_io: Box<dyn ImageIo>,
    agent_adapter: Box<dyn AgentAdapter>,
    local_process: LocalProcessConfig,
}

pub(super) enum EffectExecution<'a> {
    Compatibility,
    Exact {
        binding: &'a crate::engine::effect::authority::IntentBinding,
        dispatch: &'a crate::engine::effect::dispatch::Shell,
    },
    Product(&'a std::sync::Arc<parking_lot::Mutex<crate::engine::effect::execution::Session>>),
}

pub(super) struct NodeExecution<'a> {
    pub(super) workflow: &'a Workflow,
    pub(super) node_id: crate::engine::workflow::NodeId,
    pub(super) inputs: &'a HashMap<PortId, Value>,
    pub(super) expression_context: &'a ExpressionContext,
    pub(super) input_hash: Option<ContentHash>,
    pub(super) parent_run_id: Option<RunId>,
    pub(super) effect: EffectExecution<'a>,
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
    require_successful_shells: bool,
    subworkflow_runner_shell: Box<dyn SubworkflowRunnerShell>,
    cancellation: std::sync::Arc<std::sync::atomic::AtomicBool>,
    pub(super) frontier_policy: lattice_core::scheduling::CapacityPolicy,
    frontier_conflicts: BTreeMap<u64, lattice_core::scheduling::ConflictState>,
    observed_completion_order: Vec<crate::engine::workflow::NodeId>,
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
            require_successful_shells: false,
            subworkflow_runner_shell: default_subworkflow_runner_shell(),
            cancellation: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            frontier_policy: crate::engine::frontier::serial_capacity_policy(),
            frontier_conflicts: BTreeMap::new(),
            observed_completion_order: Vec::new(),
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

    // r[impl lattice.workflow_interchange.shell_success]
    pub(crate) fn with_required_shell_success(mut self) -> Self {
        self.require_successful_shells = true;
        self
    }

    pub fn with_frontier_policy(
        mut self,
        policy: lattice_core::scheduling::CapacityPolicy,
    ) -> Result<Self, lattice_core::scheduling::PolicyError> {
        lattice_core::scheduling::validate_policy(&policy)?;
        self.frontier_policy = policy;
        Ok(self)
    }

    pub fn with_resolved_resource_alias(
        mut self,
        node_id: crate::engine::workflow::NodeId,
        alias: impl Into<String>,
    ) -> Result<Self, &'static str> {
        let alias = alias.into();
        if alias.trim().is_empty() {
            return Err("resource alias must not be empty");
        }
        self.frontier_conflicts.insert(node_id.0, lattice_core::scheduling::ConflictState::Resolved(alias));
        Ok(self)
    }

    // r[impl lattice.workflow_execution.ready_frontier.determinism]
    pub fn observed_completion_order(&self) -> &[crate::engine::workflow::NodeId] {
        &self.observed_completion_order
    }

    pub fn frontier_policy_identity(&self) -> Result<String, serde_json::Error> {
        crate::engine::frontier::capacity_policy_identity(&self.frontier_policy)
    }

    pub(super) fn cancellation_requested(&self) -> bool {
        self.cancellation.load(std::sync::atomic::Ordering::Acquire)
    }

    pub(super) fn frontier_conflict(
        &self,
        node_id: crate::engine::workflow::NodeId,
    ) -> Option<&lattice_core::scheduling::ConflictState> {
        self.frontier_conflicts.get(&node_id.0)
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

    /// Execute a single node through the compatibility adapter surface.
    ///
    /// Product effect execution must use [`Self::execute_authorized_node`].
    pub async fn execute_node(
        &self,
        workflow: &Workflow,
        node_id: crate::engine::workflow::NodeId,
        inputs: &HashMap<PortId, Value>,
    ) -> StepResult {
        self.execute_node_with_expression_context(NodeExecution {
            workflow,
            node_id,
            inputs,
            expression_context: &ExpressionContext::new(HashMap::new()),
            input_hash: None,
            parent_run_id: None,
            effect: EffectExecution::Compatibility,
        })
        .await
    }

    // r[impl lattice.choregraph_effect_authority.binding]
    // r[impl lattice.choregraph_effect_authority.adapters]
    /// Execute one effectful node through its exact Choregraph V2 binding.
    pub async fn execute_authorized_node(
        &self,
        workflow: &Workflow,
        node_id: crate::engine::workflow::NodeId,
        inputs: &HashMap<PortId, Value>,
        binding: &crate::engine::effect::authority::IntentBinding,
        dispatch: &crate::engine::effect::dispatch::Shell,
    ) -> StepResult {
        if workflow.node_config(node_id).and_then(effect_operation).is_none() {
            return StepResult {
                node_id,
                status: NodeStatus::Failed {
                    error: AUTHORIZED_EFFECTFUL_NODE_REQUIRED.to_owned(),
                },
                input_hash: input_values_hash(inputs),
            };
        }
        self.execute_node_with_expression_context(NodeExecution {
            workflow,
            node_id,
            inputs,
            expression_context: &ExpressionContext::new(HashMap::new()),
            input_hash: None,
            parent_run_id: None,
            effect: EffectExecution::Exact { binding, dispatch },
        })
        .await
    }

    pub(super) async fn execute_node_with_expression_context(&self, execution: NodeExecution<'_>) -> StepResult {
        let NodeExecution {
            workflow,
            node_id,
            inputs,
            expression_context,
            input_hash: precomputed_input_hash,
            parent_run_id,
            effect,
        } = execution;
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

        let fallback_input_hash = precomputed_input_hash.unwrap_or_else(|| input_values_hash(inputs));

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
        let input_hash = precomputed_input_hash
            .unwrap_or_else(|| node_execution_cache_key_with_credentials(node_id, config, node_credentials, inputs));
        if let EffectExecution::Exact { binding, .. } = &effect
            && !authorized_binding_matches_node(binding, node_id, config)
        {
            return StepResult {
                node_id,
                status: NodeStatus::Failed {
                    error: AUTHORIZED_EFFECT_BINDING_MISMATCH.to_owned(),
                },
                input_hash,
            };
        }

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
                let shell_input = ShellExecutionInput {
                    command: &resolved_command.value,
                    cwd: &resolved_cwd_value,
                    environment: &merged_env,
                    timeout_secs: *timeout_secs,
                    should_pipe_stdin: *pipe_stdin,
                    stdin_value,
                };
                let result = match effect {
                    EffectExecution::Exact { binding, dispatch } => {
                        let request = shell_process_request(&shell_input, &self.local_process);
                        let payload = crate::engine::effect::runtime::process_plan_identity(&request);
                        if !authorized_request_matches_payload(binding, &payload) {
                            Err(AUTHORIZED_EFFECT_REQUEST_MISMATCH.to_owned())
                        } else {
                            let runtime = crate::engine::effect::runtime::Adapters::new_with_cancellation(
                                dispatch,
                                self.http_client.as_ref(),
                                &self.local_process,
                                self.cancellation.as_ref(),
                            );
                            dispatch.dispatch(&runtime, binding.clone(), &request).map_err(dispatch_error)
                        }
                    }
                    EffectExecution::Product(session) => {
                        let request = shell_process_request(&shell_input, &self.local_process);
                        let payload = crate::engine::effect::runtime::process_plan_identity(&request);
                        let prepared = {
                            let mut session = session.lock();
                            session.bind_effect(node_id, &payload).map(|binding| (binding, session.dispatch_handle()))
                        };
                        match prepared {
                            Ok((binding, dispatch)) => {
                                let local_process = self.local_process.clone();
                                let cancellation = std::sync::Arc::clone(&self.cancellation);
                                match tokio::task::spawn_blocking(move || {
                                    let http_client = NoopHttpClient;
                                    let runtime = crate::engine::effect::runtime::Adapters::new_with_cancellation(
                                        dispatch.as_ref(),
                                        &http_client,
                                        &local_process,
                                        cancellation.as_ref(),
                                    );
                                    dispatch.dispatch(&runtime, binding, &request).map_err(dispatch_error)
                                })
                                .await
                                {
                                    Ok(result) => result,
                                    Err(_) => Err(PROCESS_TASK_FAILURE_DIAGNOSTIC.to_owned()),
                                }
                            }
                            Err(error) => Err(choregraph_execution_error(error)),
                        }
                    }
                    EffectExecution::Compatibility => {
                        execute_shell(shell_input, &self.local_process, std::sync::Arc::clone(&self.cancellation))
                            .await
                            .map_err(|error| error.to_string())
                    }
                };

                match result {
                    Ok(output)
                        if self.require_successful_shells && output.status_code != Some(SHELL_SUCCESS_EXIT_CODE) =>
                    {
                        StepResult {
                            node_id,
                            status: NodeStatus::Failed {
                                error: format!(
                                    "shell command exited with status {}",
                                    output.status_code.unwrap_or(UNKNOWN_EXIT_CODE)
                                ),
                            },
                            input_hash,
                        }
                    }
                    Ok(output) => completed_step_result(
                        node_id,
                        input_hash,
                        node,
                        shell_outputs_by_real_port(
                            output_ports,
                            redact_shell_outputs(shell_output_values(output), &sensitive_fragments),
                        ),
                        false,
                    ),
                    Err(error) => StepResult {
                        node_id,
                        status: NodeStatus::Failed { error },
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
                let response = match effect {
                    EffectExecution::Exact { binding, dispatch } => {
                        let payload = crate::engine::effect::runtime::network_plan_identity(&request);
                        if !authorized_request_matches_payload(binding, &payload) {
                            Err(AUTHORIZED_EFFECT_REQUEST_MISMATCH.to_owned())
                        } else {
                            let runtime = crate::engine::effect::runtime::Adapters::new(
                                dispatch,
                                self.http_client.as_ref(),
                                &self.local_process,
                            );
                            dispatch.dispatch(&runtime, binding.clone(), &request).map_err(dispatch_error)
                        }
                    }
                    EffectExecution::Product(session) => {
                        let payload = crate::engine::effect::runtime::network_plan_identity(&request);
                        let prepared = {
                            let mut session = session.lock();
                            session.bind_effect(node_id, &payload).map(|binding| (binding, session.dispatch_handle()))
                        };
                        match prepared {
                            Ok((binding, dispatch)) => {
                                let runtime = crate::engine::effect::runtime::Adapters::new(
                                    dispatch.as_ref(),
                                    self.http_client.as_ref(),
                                    &self.local_process,
                                );
                                dispatch.dispatch(&runtime, binding, &request).map_err(dispatch_error)
                            }
                            Err(error) => Err(choregraph_execution_error(error)),
                        }
                    }
                    EffectExecution::Compatibility => {
                        self.http_client.execute(&request).map_err(|error| error.to_string())
                    }
                };
                match response {
                    Ok(response) => completed_step_result(
                        node_id,
                        input_hash,
                        node,
                        http_outputs_by_real_port(output_ports, response),
                        false,
                    ),
                    Err(error) => StepResult {
                        node_id,
                        status: NodeStatus::Failed { error },
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
                let result = match effect {
                    EffectExecution::Exact { binding, dispatch } => {
                        authorized_file_request(&plan).and_then(|request| {
                            let payload = crate::engine::effect::runtime::file_plan_identity(&request);
                            if !authorized_request_matches_payload(binding, &payload) {
                                return Err(AUTHORIZED_EFFECT_REQUEST_MISMATCH.to_owned());
                            }
                            let runtime = crate::engine::effect::runtime::Adapters::new(
                                dispatch,
                                self.http_client.as_ref(),
                                &self.local_process,
                            );
                            dispatch
                                .dispatch(&runtime, binding.clone(), &request)
                                .map_err(dispatch_error)
                                .and_then(authorized_file_output)
                        })
                    }
                    EffectExecution::Product(session) => authorized_file_request(&plan).and_then(|request| {
                        let payload = crate::engine::effect::runtime::file_plan_identity(&request);
                        let (binding, dispatch) = {
                            let mut session = session.lock();
                            let binding = session.bind_effect(node_id, &payload).map_err(choregraph_execution_error)?;
                            (binding, session.dispatch_handle())
                        };
                        let runtime = crate::engine::effect::runtime::Adapters::new(
                            dispatch.as_ref(),
                            self.http_client.as_ref(),
                            &self.local_process,
                        );
                        dispatch
                            .dispatch(&runtime, binding, &request)
                            .map_err(dispatch_error)
                            .and_then(authorized_file_output)
                    }),
                    EffectExecution::Compatibility => {
                        self.file_system.execute(&plan).map_err(|error| error.to_string())
                    }
                };
                match result {
                    Ok(result) => completed_step_result(
                        node_id,
                        input_hash,
                        node,
                        file_outputs_by_real_port(output_ports, &resolved_path.value, result),
                        false,
                    ),
                    Err(error) => StepResult {
                        node_id,
                        status: NodeStatus::Failed { error },
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
            NodeConfig::Operation(plan) => {
                let Some(output_port) = node.output_ports.first().map(|port| port.id) else {
                    return StepResult {
                        node_id,
                        status: NodeStatus::Failed {
                            error: "operation node output port is missing".to_string(),
                        },
                        input_hash,
                    };
                };
                let request =
                    match crate::operation::runtime::prepare_process_request(plan, self.credential_provider.as_ref()) {
                        Ok(request) => request,
                        Err(observation) => {
                            return StepResult {
                                node_id,
                                status: NodeStatus::Failed {
                                    error: observation.diagnostic,
                                },
                                input_hash,
                            };
                        }
                    };
                let payload = crate::engine::effect::runtime::process_plan_identity(&request.process);
                let dispatched = match effect {
                    EffectExecution::Exact { binding, dispatch } => {
                        if !authorized_request_matches_payload(binding, &payload) {
                            Err(AUTHORIZED_EFFECT_REQUEST_MISMATCH.to_string())
                        } else {
                            let runtime = crate::engine::effect::runtime::Adapters::new_with_cancellation(
                                dispatch,
                                self.http_client.as_ref(),
                                &self.local_process,
                                self.cancellation.as_ref(),
                            );
                            dispatch.dispatch(&runtime, binding.clone(), &request).map_err(dispatch_error)
                        }
                    }
                    EffectExecution::Product(session) => {
                        let prepared = {
                            let mut session = session.lock();
                            session
                                .bind_effect(node_id, &payload)
                                .map(|binding| (binding, session.dispatch_handle()))
                                .map_err(choregraph_execution_error)
                        };
                        prepared.and_then(|(binding, dispatch)| {
                            let runtime = crate::engine::effect::runtime::Adapters::new_with_cancellation(
                                dispatch.as_ref(),
                                self.http_client.as_ref(),
                                &self.local_process,
                                self.cancellation.as_ref(),
                            );
                            dispatch.dispatch(&runtime, binding, &request).map_err(dispatch_error)
                        })
                    }
                    EffectExecution::Compatibility => Err("operation execution requires product authority".to_string()),
                };
                let observation = match dispatched {
                    Ok(Ok(output)) => {
                        crate::operation::runtime::classify_process_output(plan, output, &request.secret_values)
                            .unwrap_or_else(std::convert::identity)
                    }
                    Ok(Err(error)) => crate::operation::runtime::process_failure(error),
                    Err(_) => {
                        crate::operation::runtime::unknown("operation authority or process outcome is unavailable")
                    }
                };
                match (observation.class, observation.output) {
                    (crate::operation::runtime::ObservationClass::Completed, Some(output)) => {
                        let mut outputs = HashMap::new();
                        outputs.insert(output_port, Value::from_json(output));
                        completed_step_result(node_id, input_hash, node, outputs, false)
                    }
                    _ => StepResult {
                        node_id,
                        status: NodeStatus::Failed {
                            error: observation.diagnostic,
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
            NodeConfig::Approval(_) => StepResult {
                node_id,
                status: NodeStatus::Failed {
                    error: APPROVAL_REQUIRES_DURABLE_SHELL.to_owned(),
                },
                input_hash,
            },
        }
    }

    /// Run the full workflow through the explicit compatibility adapter surface.
    ///
    /// Product entry points must use [`Self::run_authorized`].
    pub async fn run(
        &mut self,
        workflow: &Workflow,
        cache: &HashMap<ContentHash, HashMap<PortId, Value>>,
    ) -> (RunId, Vec<StepResult>) {
        self.run_with_bound_inputs(workflow, cache, &HashMap::new(), None).await
    }

    // r[impl lattice.choregraph_effect_authority.binding]
    // r[impl lattice.choregraph_effect_authority.denial]
    /// Run a product workflow through one run-scoped Choregraph authority context.
    pub async fn run_authorized(
        &mut self,
        workflow: &Workflow,
        cache: &HashMap<ContentHash, HashMap<PortId, Value>>,
        context: &crate::engine::effect::execution::RunContext,
    ) -> (RunId, Vec<StepResult>) {
        self.run_with_bound_inputs(workflow, cache, &HashMap::new(), Some(context)).await
    }

    pub async fn run_authorized_until_approval(
        &mut self,
        workflow: &Workflow,
        cache: &HashMap<ContentHash, HashMap<PortId, Value>>,
        context: &crate::engine::effect::execution::RunContext,
    ) -> ApprovalRunOutcome {
        self.run_with_bound_inputs_control(workflow, cache, &HashMap::new(), Some(context), ExecutionControl {
            checkpoint: &[],
            stop_at_approval: true,
            resumed_gate: None,
            resumed_gate_outputs: None,
        })
        .await
    }

    pub async fn resume_authorized_after_approval(&mut self, input: ApprovalResumeInput<'_>) -> ApprovalRunOutcome {
        self.run_with_bound_inputs_control(
            input.workflow,
            input.cache,
            &HashMap::new(),
            Some(input.context),
            ExecutionControl {
                checkpoint: input.checkpoint,
                stop_at_approval: true,
                resumed_gate: Some(input.resumed_gate),
                resumed_gate_outputs: Some(input.resumed_gate_outputs),
            },
        )
        .await
    }

    pub async fn run_with_named_inputs(
        &mut self,
        workflow: &Workflow,
        cache: &HashMap<ContentHash, HashMap<PortId, Value>>,
        named_inputs: &BTreeMap<String, Value>,
    ) -> Result<(RunId, Vec<StepResult>), crate::engine::child::SubworkflowExecutionError> {
        let bound_inputs = crate::engine::child::bind_named_inputs(workflow, named_inputs)?;
        Ok(self.run_with_bound_inputs(workflow, cache, &bound_inputs, None).await)
    }

    pub async fn run_authorized_with_named_inputs(
        &mut self,
        workflow: &Workflow,
        cache: &HashMap<ContentHash, HashMap<PortId, Value>>,
        named_inputs: &BTreeMap<String, Value>,
        context: &crate::engine::effect::execution::RunContext,
    ) -> Result<(RunId, Vec<StepResult>), crate::engine::child::SubworkflowExecutionError> {
        let bound_inputs = crate::engine::child::bind_named_inputs(workflow, named_inputs)?;
        Ok(self.run_with_bound_inputs(workflow, cache, &bound_inputs, Some(context)).await)
    }

    async fn run_with_bound_inputs(
        &mut self,
        workflow: &Workflow,
        cache: &HashMap<ContentHash, HashMap<PortId, Value>>,
        bound_inputs: &HashMap<PortId, Value>,
        product_context: Option<&crate::engine::effect::execution::RunContext>,
    ) -> (RunId, Vec<StepResult>) {
        let outcome = self
            .run_with_bound_inputs_control(workflow, cache, bound_inputs, product_context, ExecutionControl::ordinary())
            .await;
        (outcome.run_id, outcome.results)
    }

    async fn run_with_bound_inputs_control(
        &mut self,
        workflow: &Workflow,
        cache: &HashMap<ContentHash, HashMap<PortId, Value>>,
        bound_inputs: &HashMap<PortId, Value>,
        product_context: Option<&crate::engine::effect::execution::RunContext>,
        control: ExecutionControl<'_>,
    ) -> ApprovalRunOutcome {
        self.observed_completion_order.clear();
        let run_id = RunId(self.next_run_id);
        self.next_run_id = self.next_run_id.saturating_add(1);
        let product_session = match product_context {
            Some(context) => {
                let session = if control.checkpoint.is_empty() {
                    crate::engine::effect::execution::Session::new(workflow, context)
                } else {
                    crate::engine::effect::execution::Session::new_with_checkpoint(
                        workflow,
                        context,
                        control.checkpoint,
                    )
                };
                match session {
                    Ok(session) => Some(std::sync::Arc::new(parking_lot::Mutex::new(session))),
                    Err(error) => {
                        let diagnostic = choregraph_execution_error(error);
                        return ApprovalRunOutcome {
                            run_id,
                            results: failed_product_run(workflow, &diagnostic),
                            suspension: None,
                        };
                    }
                }
            }
            None => None,
        };

        let (runner_plan, topo) = match admitted_runner_execution(workflow) {
            Some(admitted) => admitted,
            None => {
                error!("workflow failed shared graph admission, cannot execute");
                return ApprovalRunOutcome {
                    run_id,
                    results: Vec::new(),
                    suspension: None,
                };
            }
        };

        info!(run = %run_id, nodes = topo.len(), "starting workflow execution");

        let has_approval_gate =
            topo.iter().any(|node_id| matches!(workflow.node_config(*node_id), Some(NodeConfig::Approval(_))));
        if (!control.stop_at_approval || !has_approval_gate) && control.checkpoint.is_empty() {
            let outcome = crate::engine::frontier::runtime::run(crate::engine::frontier::runtime::RunRequest {
                executor: self,
                workflow,
                cache,
                bound_inputs,
                run_id,
                runner_plan: &runner_plan,
                canonical_order: &topo,
                product_session: product_session.clone(),
            })
            .await;
            self.observed_completion_order = outcome.observed_completion_order;
            info!(run = %run_id, steps = outcome.results.len(), "workflow frontier execution complete");
            return ApprovalRunOutcome {
                run_id,
                results: outcome.results,
                suspension: None,
            };
        }

        let maximum_step_count = topo.len();
        let mut results = Vec::with_capacity(maximum_step_count);
        let mut port_values: HashMap<PortId, Value> = HashMap::new();
        assert!(
            control.checkpoint.len() <= maximum_step_count,
            "approval checkpoint cannot contain more nodes than the admitted workflow"
        );
        let mut runner_states = control
            .checkpoint
            .iter()
            .map(|step| (step.node_id.0, runner_step_state(&step.status)))
            .collect::<BTreeMap<_, _>>();
        let mut checkpoint_nodes = std::collections::HashSet::with_capacity(control.checkpoint.len());
        for step in control.checkpoint {
            checkpoint_nodes.insert(step.node_id);
            if let NodeStatus::Completed { outputs, .. } = &step.status {
                for (port_id, value) in outputs {
                    port_values.insert(*port_id, value.clone());
                }
            }
            results.push(step.clone());
        }

        for node_id in topo {
            if checkpoint_nodes.contains(&node_id) {
                continue;
            }
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

            let base_input_hash = match workflow.node_config(node_id) {
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
                if let Some(session) = product_session.as_ref()
                    && let Err(error) = session.lock().cancel(node_id)
                {
                    results.push(failed_product_step(node_id, base_input_hash, &choregraph_execution_error(error)));
                    continue;
                }
                results.push(StepResult {
                    node_id,
                    status: NodeStatus::Skipped,
                    input_hash: base_input_hash,
                });
                continue;
            }

            let expression_values = match expression_values_for_consumer(&runner_plan, node_id, &port_values) {
                Ok(values) => values,
                Err(error) => {
                    let step = failed_product_step(node_id, base_input_hash, &error);
                    runner_states.insert(node_id.0, runner_step_state(&step.status));
                    results.push(step);
                    continue;
                }
            };
            let input_hash = match workflow.node_config(node_id) {
                Some(config) => node_execution_cache_key_with_expression_values(
                    node_id,
                    config,
                    workflow.node_credentials(node_id),
                    &inputs,
                    &expression_values,
                ),
                None => base_input_hash,
            };
            let expression_context = expression_context_from_admitted_values(&expression_values);

            if matches!(workflow.node_config(node_id), Some(NodeConfig::Approval(_))) && control.stop_at_approval {
                if control.resumed_gate == Some(node_id) {
                    let Some(node) = workflow.graph.node(node_id) else {
                        results.push(failed_product_step(
                            node_id,
                            input_hash,
                            "approval node is missing from workflow graph",
                        ));
                        continue;
                    };
                    let Some(outputs) = control.resumed_gate_outputs else {
                        results.push(failed_product_step(node_id, input_hash, "approved resume inputs are missing"));
                        continue;
                    };
                    if !approval_gate_outputs_valid(node, outputs) {
                        results.push(failed_product_step(
                            node_id,
                            input_hash,
                            "approved resume outputs do not match the gate schema",
                        ));
                        continue;
                    }
                    if let Some(session) = product_session.as_ref()
                        && {
                            let mut session = session.lock();
                            session.mark_ready(node_id).is_err() || session.start_pure(node_id).is_err()
                        }
                    {
                        let step = failed_product_step(node_id, input_hash, CHOREGRAPH_EXECUTION_FAILURE);
                        runner_states.insert(node_id.0, runner_step_state(&step.status));
                        results.push(step);
                        continue;
                    }
                    let step = completed_step_result(node_id, input_hash, node, outputs.clone(), false);
                    let step = match product_session.as_ref() {
                        Some(session) => match session.lock().finish(node_id, &step.status) {
                            Ok(()) => step,
                            Err(error) => failed_product_step(node_id, input_hash, &choregraph_execution_error(error)),
                        },
                        None => step,
                    };
                    if let NodeStatus::Completed { outputs, .. } = &step.status {
                        for (port_id, value) in outputs {
                            port_values.insert(*port_id, value.clone());
                        }
                    }
                    runner_states.insert(node_id.0, runner_step_state(&step.status));
                    results.push(step);
                    continue;
                }
                return ApprovalRunOutcome {
                    run_id,
                    results,
                    suspension: Some(ApprovalSuspensionPoint { node_id, inputs }),
                };
            }

            if let Some(session) = product_session.as_ref()
                && let Err(error) = session.lock().mark_ready(node_id)
            {
                let step = failed_product_step(node_id, input_hash, &choregraph_execution_error(error));
                runner_states.insert(node_id.0, runner_step_state(&step.status));
                results.push(step);
                continue;
            }

            if let Some(cached_outputs) = cache.get(&input_hash) {
                debug!(node = ?node_id, "cache hit, skipping execution");
                let Some(node) = workflow.graph.node(node_id) else {
                    continue;
                };
                if let Some(session) = product_session.as_ref()
                    && let Err(error) = session.lock().start_pure(node_id)
                {
                    let step = failed_product_step(node_id, input_hash, &choregraph_execution_error(error));
                    runner_states.insert(node_id.0, runner_step_state(&step.status));
                    results.push(step);
                    continue;
                }
                let step = completed_step_result(node_id, input_hash, node, cached_outputs.clone(), true);
                if let Some(session) = product_session.as_ref()
                    && let Err(error) = session.lock().finish(node_id, &step.status)
                {
                    let step = failed_product_step(node_id, input_hash, &choregraph_execution_error(error));
                    runner_states.insert(node_id.0, runner_step_state(&step.status));
                    results.push(step);
                    continue;
                }
                if let NodeStatus::Completed { ref outputs, .. } = step.status {
                    for (port_id, value) in outputs {
                        port_values.insert(*port_id, value.clone());
                    }
                }
                runner_states.insert(node_id.0, runner::RunnerStepState::Completed);
                results.push(step);
                continue;
            }

            let has_effectful_config = workflow.node_config(node_id).and_then(effect_operation).is_some();
            if !has_effectful_config
                && let Some(session) = product_session.as_ref()
                && let Err(error) = session.lock().start_pure(node_id)
            {
                let step = failed_product_step(node_id, input_hash, &choregraph_execution_error(error));
                runner_states.insert(node_id.0, runner_step_state(&step.status));
                results.push(step);
                continue;
            }
            let effect = match product_session.as_ref() {
                Some(session) => EffectExecution::Product(session),
                None => EffectExecution::Compatibility,
            };
            let step = self
                .execute_node_with_expression_context(NodeExecution {
                    workflow,
                    node_id,
                    inputs: &inputs,
                    expression_context: &expression_context,
                    input_hash: Some(input_hash),
                    parent_run_id: Some(run_id),
                    effect,
                })
                .await;
            let step = match product_session.as_ref() {
                Some(session) => match session.lock().finish(node_id, &step.status) {
                    Ok(()) => step,
                    Err(error) => failed_product_step(node_id, input_hash, &choregraph_execution_error(error)),
                },
                None => step,
            };

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
        self.observed_completion_order = results.iter().map(|step| step.node_id).collect();
        ApprovalRunOutcome {
            run_id,
            results,
            suspension: None,
        }
    }
}

impl Default for Executor {
    fn default() -> Self {
        Self::new()
    }
}

fn failed_product_run(workflow: &Workflow, diagnostic: &str) -> Vec<StepResult> {
    workflow
        .graph
        .nodes()
        .map(|node| failed_product_step(node.id, ContentHash::from_bytes(b""), diagnostic))
        .collect()
}

pub(super) fn failed_product_step(
    node_id: crate::engine::workflow::NodeId,
    input_hash: ContentHash,
    diagnostic: &str,
) -> StepResult {
    StepResult {
        node_id,
        status: NodeStatus::Failed {
            error: diagnostic.to_owned(),
        },
        input_hash,
    }
}

#[must_use]
pub fn authority_context_required_diagnostic() -> &'static str {
    AUTHORIZED_WORKFLOW_CONTEXT_REQUIRED
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
fn shell_process_request(
    input: &ShellExecutionInput<'_>,
    config: &LocalProcessConfig,
) -> crate::agent_node::ProcessRequest {
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
    request
}

fn shell_output_values(output: crate::agent_node::ProcessOutput) -> ShellOutputValues {
    ShellOutputValues {
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        exit_code: output.status_code.unwrap_or(UNKNOWN_EXIT_CODE),
    }
}

async fn execute_shell(
    input: ShellExecutionInput<'_>,
    config: &LocalProcessConfig,
    cancellation: std::sync::Arc<std::sync::atomic::AtomicBool>,
) -> Result<crate::agent_node::ProcessOutput, crate::agent_node::ProcessError> {
    let request = shell_process_request(&input, config);
    let runner = crate::agent_node::ProductionProcessRunner::new(config.clone());
    let output = tokio::task::spawn_blocking(move || runner.run_with_cancellation(&request, &cancellation))
        .await
        .map_err(|_| crate::agent_node::ProcessError::Spawn(PROCESS_TASK_FAILURE_DIAGNOSTIC.to_owned()))??;

    Ok(output)
}

/// Gather input values for a node from upstream edge outputs.
pub(super) fn gather_inputs(
    workflow: &Workflow,
    node_id: crate::engine::workflow::NodeId,
    port_values: &HashMap<PortId, Value>,
) -> HashMap<PortId, Value> {
    let Some(node) = workflow.graph.node(node_id) else {
        return HashMap::new();
    };
    let mut inputs = HashMap::with_capacity(node.input_ports.len());

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

fn approval_gate_outputs_valid(node: &Node, outputs: &HashMap<PortId, Value>) -> bool {
    if outputs.len() != node.output_ports.len() {
        return false;
    }
    node.output_ports.iter().all(|port| {
        outputs.get(&port.id).is_some_and(|value| {
            port.type_tag == crate::engine::value::TypeTag::Any.as_str() || port.type_tag == value.type_tag().as_str()
        })
    })
}

pub(super) fn runner_step_state(status: &NodeStatus) -> runner::RunnerStepState {
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
fn admitted_runner_execution(
    workflow: &Workflow,
) -> Option<(runner::RunnerWorkflow, Vec<crate::engine::workflow::NodeId>)> {
    let plan = runner::build_runner_plan(workflow).plan?;
    match runner::admit_runner_plan(&plan) {
        runner::RunnerAdmission::Accepted { topo_order } => {
            Some((plan, topo_order.into_iter().map(crate::engine::workflow::NodeId).collect()))
        }
        runner::RunnerAdmission::Rejected { .. } => None,
    }
}

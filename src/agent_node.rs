//! Generic agent node request planning and adapter interface.

use std::collections::BTreeMap;
use std::io::ErrorKind;
use std::path::Path;
use std::process::Command;

use serde::Deserialize;
use serde::Serialize;

use crate::engine::value::ImageValue;
use crate::engine::value::Value;
use crate::engine::value::ValueData;
use crate::image_node;
use crate::openai::Transport;

const DEFAULT_PI_RUNTIME_KEY: &str = "pi";
const DEFAULT_CLANKERS_RUNTIME_KEY: &str = "clankers";
const DEFAULT_ANIMUS_RUNTIME_KEY: &str = "animus";
const DEFAULT_PI_COMMAND: &str = "pi";
const DEFAULT_CLANKERS_COMMAND: &str = "clankers";
const PRINT_MODE_FLAG: &str = "-p";
const NO_SESSION_FLAG: &str = "--no-session";
const PROMPT_PLACEHOLDER: &str = "{prompt}";
const MODEL_PLACEHOLDER: &str = "{model}";
const SESSION_PLACEHOLDER: &str = "{session}";
const OUTPUT_MODE_PLACEHOLDER: &str = "{output_mode}";
const TEXT_INPUT_HEADING: &str = "Text input";
const JSON_INPUT_HEADING: &str = "JSON input";
const IMAGE_INPUT_HEADING: &str = "Image input";
const CONTEXT_REFS_HEADING: &str = "Context references";
const TOOL_POLICY_HEADING: &str = "Tool policy";
const MODEL_HINT_HEADING: &str = "Model hint";
const DISPLAY_INDEX_BASE: usize = 1;
const ERROR_OUTPUT_LIMIT_CHARS: usize = 2048;
const BYTES_PER_KIB: usize = 1024;
const KIB_PER_MIB: usize = 1024;
const DEFAULT_AGENT_OUTPUT_BYTE_BUDGET_MIB: usize = 64;
const DEFAULT_AGENT_OUTPUT_MAX_BYTES: usize = DEFAULT_AGENT_OUTPUT_BYTE_BUDGET_MIB * KIB_PER_MIB * BYTES_PER_KIB;
const HANDOFF_PATH_FIELD: &str = "path";
const HANDOFF_MEDIA_TYPE_FIELD: &str = "media_type";
const HANDOFF_METADATA_FIELD: &str = "metadata";
const CURRENT_DIRECTORY: &str = ".";
const AGENT_COMMAND_NO_TIMEOUT_SECS: u64 = 0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentTarget {
    Pi,
    Clankers,
    Animus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentOutputMode {
    Text,
    Json,
    Image,
    Artifact,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentNodeSettings {
    pub target: AgentTarget,
    pub prompt: String,
    #[serde(default = "absent_prompt_file")]
    pub prompt_file: Option<String>,
    pub output_mode: AgentOutputMode,
    pub model_hint: Option<String>,
    pub tool_policy: String,
    pub context_refs: Vec<String>,
    pub cwd: Option<String>,
    pub session: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentRuntimeConfig {
    #[serde(default = "default_agent_runtimes")]
    pub runtimes: BTreeMap<String, AgentCommandRuntimeConfig>,
    #[serde(default = "empty_openai_runtimes")]
    pub openai_compatible: BTreeMap<String, crate::openai::RuntimeConfig>,
    #[serde(default = "default_agent_output_max_bytes")]
    pub max_output_bytes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentCommandRuntimeConfig {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
}

impl Default for AgentRuntimeConfig {
    fn default() -> Self {
        Self {
            runtimes: default_agent_runtimes(),
            openai_compatible: BTreeMap::new(),
            max_output_bytes: default_agent_output_max_bytes(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentRequest {
    pub target: AgentTarget,
    pub prompt: String,
    pub output_mode: AgentOutputMode,
    pub model_hint: Option<String>,
    pub tool_policy: String,
    pub context_refs: Vec<String>,
    pub cwd: Option<String>,
    pub session: Option<String>,
    pub text_inputs: Vec<String>,
    pub image_inputs: Vec<ImageValue>,
    pub json_inputs: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentResponse {
    pub output: AgentOutput,
    pub metadata: serde_json::Value,
}

/// Execution-only secrets for an agent adapter call.
#[derive(Default)]
pub struct ExecutionContext {
    secret_env: BTreeMap<String, String>,
    secret_values: Vec<String>,
}

impl ExecutionContext {
    pub fn new(secret_env: std::collections::HashMap<String, String>) -> Self {
        let secret_values = secret_env.values().filter(|value| !value.is_empty()).cloned().collect();
        Self {
            secret_env: secret_env.into_iter().collect(),
            secret_values,
        }
    }

    pub fn secret(&self, binding: &str) -> Option<&str> {
        self.secret_env.get(binding).map(String::as_str)
    }

    pub fn redact(&self, value: &str) -> String {
        crate::credentials::redact_text(value, &self.secret_values)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AgentOutput {
    Text(String),
    Json(serde_json::Value),
    Image(ImageValue),
    Artifact(Vec<u8>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentError {
    UnsupportedTarget(String),
    MissingPrompt,
    PreflightRejected(String),
    Credential(String),
    Session(String),
    Adapter(String),
    OutputTypeMismatch {
        expected: AgentOutputMode,
        actual: AgentOutputMode,
    },
}

impl std::fmt::Display for AgentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedTarget(target) => write!(f, "unsupported agent target: {target}"),
            Self::MissingPrompt => write!(f, "generic agent node prompt is empty"),
            Self::PreflightRejected(message) => write!(f, "agent preflight rejected request: {message}"),
            Self::Credential(message) => write!(f, "agent credential error: {message}"),
            Self::Session(message) => write!(f, "agent session error: {message}"),
            Self::Adapter(message) => write!(f, "agent adapter error: {message}"),
            Self::OutputTypeMismatch { expected, actual } => {
                write!(f, "agent output type mismatch: expected {}, got {}", expected.as_str(), actual.as_str())
            }
        }
    }
}

impl std::error::Error for AgentError {}

pub trait AgentAdapter: Send + Sync {
    fn execute(&self, request: &AgentRequest, context: &ExecutionContext) -> Result<AgentResponse, AgentError>;
}

#[derive(Debug, Default)]
pub struct NoopAgentAdapter;

impl AgentAdapter for NoopAgentAdapter {
    fn execute(&self, request: &AgentRequest, _context: &ExecutionContext) -> Result<AgentResponse, AgentError> {
        Err(AgentError::UnsupportedTarget(agent_target_label(request.target).to_string()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentCommandPlan {
    pub target: AgentTarget,
    pub adapter_label: &'static str,
    pub program: String,
    pub args: Vec<String>,
    pub cwd: Option<String>,
    pub output_mode: AgentOutputMode,
    pub max_output_bytes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentCommandOutput {
    pub status_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessRequest {
    pub program: String,
    pub args: Vec<String>,
    pub cwd: Option<String>,
    pub env: BTreeMap<String, String>,
    pub stdin: Vec<u8>,
    pub timeout_secs: u64,
    pub output_limit_bytes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessOutput {
    pub status_code: Option<i32>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessError {
    Spawn(String),
    Timeout,
    OutputLimitExceeded { limit: usize, actual: usize },
}

impl std::fmt::Display for ProcessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Spawn(message) => write!(f, "process spawn failed: {message}"),
            Self::Timeout => write!(f, "process timed out"),
            Self::OutputLimitExceeded { limit, actual } => {
                write!(f, "process output exceeded limit {limit} bytes with {actual} bytes")
            }
        }
    }
}

impl std::error::Error for ProcessError {}

// r[impl n8nclone.workflow_execution.trait_boundaries.process_runner]
pub trait ProcessRunner {
    fn run(&self, request: &ProcessRequest) -> Result<ProcessOutput, ProcessError>;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ProductionProcessRunner;

// r[impl n8nclone.workflow_execution.trait_boundaries.process_runner]
impl ProcessRunner for ProductionProcessRunner {
    fn run(&self, request: &ProcessRequest) -> Result<ProcessOutput, ProcessError> {
        run_process_request(request)
    }
}

#[derive(Debug, Clone)]
pub struct FakeProcessRunner {
    result: Result<ProcessOutput, ProcessError>,
}

impl FakeProcessRunner {
    pub fn new(result: Result<ProcessOutput, ProcessError>) -> Self {
        Self { result }
    }
}

// r[impl n8nclone.workflow_execution.trait_boundaries.process_runner]
impl ProcessRunner for FakeProcessRunner {
    fn run(&self, _request: &ProcessRequest) -> Result<ProcessOutput, ProcessError> {
        self.result.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct AgentOutputFileEnvelope {
    pub path: String,
    #[serde(default)]
    pub media_type: Option<String>,
    #[serde(default)]
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AgentOutputFile {
    path_label: String,
    bytes: Vec<u8>,
}

enum RuntimeRoute<'a> {
    Command,
    OpenAi(&'a crate::openai::RuntimeConfig),
    Animus(&'a crate::openai::RuntimeConfig),
}

// r[impl n8nclone.workflow_execution.agent_nodes.openai_compatible.scenario.route]
// r[impl lattice.agent_runtime.animus_leaf]
// r[impl lattice.agent_runtime.animus_compatibility]
fn runtime_route_for_target(config: &AgentRuntimeConfig, target: AgentTarget) -> Result<RuntimeRoute<'_>, AgentError> {
    let target_label = agent_target_label(target);
    let has_command_runtime = config.runtimes.contains_key(target_label);
    let openai_runtime = config.openai_compatible.get(target_label);
    match (has_command_runtime, openai_runtime, target) {
        (true, None, AgentTarget::Pi | AgentTarget::Clankers) => Ok(RuntimeRoute::Command),
        (false, Some(runtime), AgentTarget::Pi | AgentTarget::Clankers) => Ok(RuntimeRoute::OpenAi(runtime)),
        (false, Some(runtime), AgentTarget::Animus) => Ok(RuntimeRoute::Animus(runtime)),
        (false, None, _) => {
            Err(AgentError::PreflightRejected(format!("missing {target_label} agent runtime configuration")))
        }
        (true, Some(_), _) => {
            Err(AgentError::PreflightRejected(format!("ambiguous {target_label} agent runtime configuration")))
        }
        (true, None, AgentTarget::Animus) => Err(AgentError::PreflightRejected(
            "Animus target requires an OpenAI-compatible model runtime and cannot use a command runtime".to_string(),
        )),
    }
}

#[derive(Clone)]
pub struct ProductionAgentAdapter {
    config: AgentRuntimeConfig,
    openai_transport: std::sync::Arc<dyn Transport>,
}

impl ProductionAgentAdapter {
    pub fn new(config: AgentRuntimeConfig) -> Self {
        Self {
            config,
            openai_transport: crate::openai::default_transport(),
        }
    }

    pub fn new_with_openai_transport(
        config: AgentRuntimeConfig,
        openai_transport: std::sync::Arc<dyn Transport>,
    ) -> Self {
        Self {
            config,
            openai_transport,
        }
    }
}

impl Default for ProductionAgentAdapter {
    fn default() -> Self {
        Self::new(AgentRuntimeConfig::default())
    }
}

impl AgentAdapter for ProductionAgentAdapter {
    fn execute(&self, request: &AgentRequest, context: &ExecutionContext) -> Result<AgentResponse, AgentError> {
        match runtime_route_for_target(&self.config, request.target)? {
            RuntimeRoute::Command => {
                let plan = plan_agent_command(&self.config, request)?;
                let output = run_agent_command(&plan)?;
                decode_agent_command_output(&plan, &output)
            }
            RuntimeRoute::OpenAi(runtime) => {
                let global_output_max_bytes = global_output_max_bytes(&self.config)?;
                crate::openai::execute(
                    runtime,
                    request,
                    context,
                    global_output_max_bytes,
                    self.openai_transport.as_ref(),
                )
            }
            RuntimeRoute::Animus(runtime) => crate::animus_agent::execute(
                runtime,
                request,
                context,
                global_output_max_bytes(&self.config)?,
                self.openai_transport.as_ref(),
            ),
        }
    }
}

fn global_output_max_bytes(config: &AgentRuntimeConfig) -> Result<u64, AgentError> {
    u64::try_from(config.max_output_bytes)
        .map_err(|_| AgentError::PreflightRejected("global agent output bound is unsupported".to_string()))
}

fn absent_prompt_file() -> Option<String> {
    None
}

fn default_agent_runtimes() -> BTreeMap<String, AgentCommandRuntimeConfig> {
    BTreeMap::from([
        (DEFAULT_PI_RUNTIME_KEY.to_string(), AgentCommandRuntimeConfig {
            command: DEFAULT_PI_COMMAND.to_string(),
            args: vec![
                NO_SESSION_FLAG.to_string(),
                PRINT_MODE_FLAG.to_string(),
                PROMPT_PLACEHOLDER.to_string(),
            ],
        }),
        (DEFAULT_CLANKERS_RUNTIME_KEY.to_string(), AgentCommandRuntimeConfig {
            command: DEFAULT_CLANKERS_COMMAND.to_string(),
            args: vec![PRINT_MODE_FLAG.to_string(), PROMPT_PLACEHOLDER.to_string()],
        }),
    ])
}

fn empty_openai_runtimes() -> BTreeMap<String, crate::openai::RuntimeConfig> {
    BTreeMap::new()
}

fn default_agent_output_max_bytes() -> usize {
    DEFAULT_AGENT_OUTPUT_MAX_BYTES
}

impl AgentNodeSettings {
    pub fn pi(prompt: impl Into<String>, output_mode: AgentOutputMode) -> Self {
        Self::new(AgentTarget::Pi, prompt, output_mode)
    }

    pub fn clankers(prompt: impl Into<String>, output_mode: AgentOutputMode) -> Self {
        Self::new(AgentTarget::Clankers, prompt, output_mode)
    }

    pub fn animus(prompt: impl Into<String>, output_mode: AgentOutputMode) -> Self {
        Self::new(AgentTarget::Animus, prompt, output_mode)
    }

    pub fn new(target: AgentTarget, prompt: impl Into<String>, output_mode: AgentOutputMode) -> Self {
        Self {
            target,
            prompt: prompt.into(),
            prompt_file: None,
            output_mode,
            model_hint: None,
            tool_policy: String::new(),
            context_refs: Vec::new(),
            cwd: None,
            session: None,
        }
    }
}

impl AgentOutputMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Json => "json",
            Self::Image => "image",
            Self::Artifact => "artifact",
        }
    }
}

impl std::fmt::Display for AgentOutputMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl AgentOutput {
    pub fn output_mode(&self) -> AgentOutputMode {
        match self {
            Self::Text(_) => AgentOutputMode::Text,
            Self::Json(_) => AgentOutputMode::Json,
            Self::Image(_) => AgentOutputMode::Image,
            Self::Artifact(_) => AgentOutputMode::Artifact,
        }
    }
}

pub fn agent_target_label(target: AgentTarget) -> &'static str {
    match target {
        AgentTarget::Pi => DEFAULT_PI_RUNTIME_KEY,
        AgentTarget::Clankers => DEFAULT_CLANKERS_RUNTIME_KEY,
        AgentTarget::Animus => DEFAULT_ANIMUS_RUNTIME_KEY,
    }
}

pub fn parse_agent_target(value: &str) -> Result<AgentTarget, AgentError> {
    match value {
        DEFAULT_PI_RUNTIME_KEY => Ok(AgentTarget::Pi),
        DEFAULT_CLANKERS_RUNTIME_KEY => Ok(AgentTarget::Clankers),
        DEFAULT_ANIMUS_RUNTIME_KEY => Ok(AgentTarget::Animus),
        _ => Err(AgentError::UnsupportedTarget(value.to_string())),
    }
}

pub fn parse_agent_output_mode(value: &str) -> Result<AgentOutputMode, AgentError> {
    match value {
        "text" => Ok(AgentOutputMode::Text),
        "json" => Ok(AgentOutputMode::Json),
        "image" => Ok(AgentOutputMode::Image),
        "artifact" => Ok(AgentOutputMode::Artifact),
        _ => Err(AgentError::Adapter(format!("unsupported agent output mode: {value}"))),
    }
}

// r[impl n8nclone.workflow_execution.agent_nodes.production_adapters.command]
// r[impl n8nclone.workflow_execution.runtime_registry.command_runtime]
// r[impl n8nclone.workflow_execution.agent_nodes.output_file_handoff.contract]
pub fn plan_agent_command(config: &AgentRuntimeConfig, request: &AgentRequest) -> Result<AgentCommandPlan, AgentError> {
    let runtime = command_runtime_for_target(config, request.target)?;
    let program = runtime_command_for_target(runtime, request.target)?;
    let prompt = render_agent_command_prompt(request);
    let args = command_args_for_request(runtime, request, &prompt);
    Ok(AgentCommandPlan {
        target: request.target,
        adapter_label: agent_target_label(request.target),
        program,
        args,
        cwd: request.cwd.clone(),
        output_mode: request.output_mode,
        max_output_bytes: config.max_output_bytes,
    })
}

fn command_runtime_for_target(
    config: &AgentRuntimeConfig,
    target: AgentTarget,
) -> Result<&AgentCommandRuntimeConfig, AgentError> {
    let target_label = agent_target_label(target);
    config
        .runtimes
        .get(target_label)
        .ok_or_else(|| AgentError::PreflightRejected(format!("missing {target_label} agent runtime configuration")))
}

fn runtime_command_for_target(runtime: &AgentCommandRuntimeConfig, target: AgentTarget) -> Result<String, AgentError> {
    let Some(command) = non_empty_trimmed(&runtime.command) else {
        return Err(AgentError::PreflightRejected(format!(
            "missing {} agent runtime command",
            agent_target_label(target)
        )));
    };
    Ok(command.to_string())
}

fn non_empty_trimmed(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    if trimmed.is_empty() { None } else { Some(trimmed) }
}

fn command_args_for_request(runtime: &AgentCommandRuntimeConfig, request: &AgentRequest, prompt: &str) -> Vec<String> {
    runtime.args.iter().filter_map(|arg| render_command_arg_template(arg, request, prompt)).collect()
}

fn render_command_arg_template(template: &str, request: &AgentRequest, prompt: &str) -> Option<String> {
    let rendered = template
        .replace(PROMPT_PLACEHOLDER, prompt)
        .replace(MODEL_PLACEHOLDER, request.model_hint.as_deref().unwrap_or_default())
        .replace(SESSION_PLACEHOLDER, request.session.as_deref().unwrap_or_default())
        .replace(OUTPUT_MODE_PLACEHOLDER, request.output_mode.as_str());
    non_empty_trimmed(&rendered).map(str::to_string)
}

pub fn render_agent_command_prompt(request: &AgentRequest) -> String {
    let mut sections = Vec::new();
    sections.push(request.prompt.clone());
    append_text_input_sections(&mut sections, &request.text_inputs);
    append_json_input_sections(&mut sections, &request.json_inputs);
    append_image_input_sections(&mut sections, &request.image_inputs);
    append_optional_list_section(&mut sections, CONTEXT_REFS_HEADING, &request.context_refs);
    append_optional_string_section(&mut sections, TOOL_POLICY_HEADING, &request.tool_policy);
    if let Some(model_hint) = request.model_hint.as_ref() {
        append_optional_string_section(&mut sections, MODEL_HINT_HEADING, model_hint);
    }
    sections.join("\n\n")
}

fn append_text_input_sections(sections: &mut Vec<String>, inputs: &[String]) {
    for (index, input) in inputs.iter().enumerate() {
        sections.push(format!("{TEXT_INPUT_HEADING} {}:\n{input}", index + DISPLAY_INDEX_BASE));
    }
}

fn append_json_input_sections(sections: &mut Vec<String>, inputs: &[serde_json::Value]) {
    for (index, input) in inputs.iter().enumerate() {
        let rendered = serde_json::to_string_pretty(input).unwrap_or_else(|_| input.to_string());
        sections.push(format!("{JSON_INPUT_HEADING} {}:\n{rendered}", index + DISPLAY_INDEX_BASE));
    }
}

fn append_image_input_sections(sections: &mut Vec<String>, inputs: &[ImageValue]) {
    for (index, input) in inputs.iter().enumerate() {
        let source = input.metadata.source_label.as_deref().unwrap_or("inline image");
        sections.push(format!(
            "{IMAGE_INPUT_HEADING} {}: {} at {}",
            index + DISPLAY_INDEX_BASE,
            input.summary(),
            source
        ));
    }
}

fn append_optional_list_section(sections: &mut Vec<String>, heading: &str, values: &[String]) {
    if values.is_empty() {
        return;
    }
    let rendered = values.iter().map(|value| format!("- {value}")).collect::<Vec<_>>().join("\n");
    sections.push(format!("{heading}:\n{rendered}"));
}

fn append_optional_string_section(sections: &mut Vec<String>, heading: &str, value: &str) {
    if value.trim().is_empty() {
        return;
    }
    sections.push(format!("{heading}:\n{value}"));
}

fn run_agent_command(plan: &AgentCommandPlan) -> Result<AgentCommandOutput, AgentError> {
    run_agent_command_with_runner(plan, &ProductionProcessRunner)
}

// r[impl n8nclone.workflow_execution.trait_boundaries.process_runner]
pub fn run_agent_command_with_runner(
    plan: &AgentCommandPlan,
    runner: &impl ProcessRunner,
) -> Result<AgentCommandOutput, AgentError> {
    let request = process_request_for_agent_command(plan);
    let output = runner.run(&request).map_err(|error| {
        AgentError::Adapter(format!("failed to run {} agent command '{}': {error}", plan.adapter_label, plan.program))
    })?;
    Ok(agent_command_output_from_process_output(output))
}

fn process_request_for_agent_command(plan: &AgentCommandPlan) -> ProcessRequest {
    ProcessRequest {
        program: plan.program.clone(),
        args: plan.args.clone(),
        cwd: plan.cwd.clone(),
        env: BTreeMap::new(),
        stdin: Vec::new(),
        timeout_secs: AGENT_COMMAND_NO_TIMEOUT_SECS,
        output_limit_bytes: plan.max_output_bytes,
    }
}

fn agent_command_output_from_process_output(output: ProcessOutput) -> AgentCommandOutput {
    AgentCommandOutput {
        status_code: output.status_code,
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    }
}

fn run_process_request(request: &ProcessRequest) -> Result<ProcessOutput, ProcessError> {
    let mut command = Command::new(&request.program);
    command.args(&request.args);
    if let Some(cwd) = request.cwd.as_ref().filter(|value| !value.trim().is_empty()) {
        command.current_dir(cwd);
    }
    for (key, value) in &request.env {
        command.env(key, value);
    }
    if !request.stdin.is_empty() {
        command.stdin(std::process::Stdio::piped());
    }
    let output = command.output().map_err(|error| ProcessError::Spawn(error.to_string()))?;
    let total_output_bytes = output.stdout.len().saturating_add(output.stderr.len());
    if total_output_bytes > request.output_limit_bytes {
        return Err(ProcessError::OutputLimitExceeded {
            limit: request.output_limit_bytes,
            actual: total_output_bytes,
        });
    }
    Ok(ProcessOutput {
        status_code: output.status.code(),
        stdout: output.stdout,
        stderr: output.stderr,
    })
}

// r[impl n8nclone.workflow_execution.agent_nodes.production_adapters.command]
// r[impl n8nclone.workflow_execution.runtime_registry.command_runtime]
// r[impl n8nclone.workflow_execution.agent_nodes.output_file_handoff.contract]
// r[impl n8nclone.workflow_execution.agent_nodes.output_file_handoff.image]
// r[impl n8nclone.workflow_execution.agent_nodes.output_file_handoff.artifact]
pub fn decode_agent_command_output(
    plan: &AgentCommandPlan,
    output: &AgentCommandOutput,
) -> Result<AgentResponse, AgentError> {
    if output.status_code != Some(0) {
        return Err(AgentError::Adapter(format!(
            "{} agent command exited with status {}; stderr: {}",
            plan.adapter_label,
            status_label(output.status_code),
            bounded_error_text(&output.stderr)
        )));
    }

    match plan.output_mode {
        AgentOutputMode::Text => Ok(AgentResponse {
            output: AgentOutput::Text(output.stdout.trim_end().to_string()),
            metadata: command_metadata(plan, output, None),
        }),
        AgentOutputMode::Json => {
            let json = serde_json::from_str(output.stdout.trim()).map_err(|error| {
                AgentError::Adapter(format!("{} agent command returned invalid JSON: {error}", plan.adapter_label))
            })?;
            Ok(AgentResponse {
                output: AgentOutput::Json(json),
                metadata: command_metadata(plan, output, None),
            })
        }
        AgentOutputMode::Image => decode_image_file_handoff(plan, output),
        AgentOutputMode::Artifact => decode_artifact_file_handoff(plan, output),
    }
}

fn decode_image_file_handoff(
    plan: &AgentCommandPlan,
    output: &AgentCommandOutput,
) -> Result<AgentResponse, AgentError> {
    let envelope = parse_agent_output_file_envelope(plan, &output.stdout)?;
    let output_file = load_agent_output_file(plan, &envelope)?;
    let image =
        image_node::image_value_from_loaded_bytes(&output_file.path_label, output_file.bytes, plan.max_output_bytes)
            .map_err(|error| AgentError::Adapter(error.to_string()))?;
    validate_declared_media_type(plan, envelope.media_type.as_deref(), &image.media_type)?;
    let metadata =
        command_metadata(plan, output, Some(file_metadata(&envelope, &output_file.path_label, image.bytes.len())));
    Ok(AgentResponse {
        output: AgentOutput::Image(image),
        metadata,
    })
}

fn decode_artifact_file_handoff(
    plan: &AgentCommandPlan,
    output: &AgentCommandOutput,
) -> Result<AgentResponse, AgentError> {
    let envelope = parse_agent_output_file_envelope(plan, &output.stdout)?;
    let output_file = load_agent_output_file(plan, &envelope)?;
    let metadata = command_metadata(
        plan,
        output,
        Some(file_metadata(&envelope, &output_file.path_label, output_file.bytes.len())),
    );
    Ok(AgentResponse {
        output: AgentOutput::Artifact(output_file.bytes),
        metadata,
    })
}

pub fn parse_agent_output_file_envelope(
    plan: &AgentCommandPlan,
    stdout: &str,
) -> Result<AgentOutputFileEnvelope, AgentError> {
    let envelope = serde_json::from_str::<AgentOutputFileEnvelope>(stdout.trim()).map_err(|error| {
        AgentError::Adapter(format!(
            "{} agent {} handoff envelope is invalid JSON: {error}",
            plan.adapter_label,
            plan.output_mode.as_str()
        ))
    })?;
    if envelope.path.trim().is_empty() {
        return Err(AgentError::Adapter(format!(
            "{} agent {} handoff envelope must include a non-empty {HANDOFF_PATH_FIELD}",
            plan.adapter_label,
            plan.output_mode.as_str()
        )));
    }
    Ok(envelope)
}

// r[impl n8nclone.workflow_execution.agent_nodes.output_file_handoff.capability_read]
fn load_agent_output_file(
    plan: &AgentCommandPlan,
    envelope: &AgentOutputFileEnvelope,
) -> Result<AgentOutputFile, AgentError> {
    debug_assert!(!envelope.path.trim().is_empty());
    let resolved = resolve_handoff_path(plan, &envelope.path)?;
    let base = plan.cwd.as_deref().unwrap_or(CURRENT_DIRECTORY);
    let root = crate::capability::open_root(&resolved).map_err(|error| {
        AgentError::Adapter(format!("{} agent working directory is unavailable: {base}: {error}", plan.adapter_label))
    })?;
    let file = crate::capability::open_relative_read(&root, resolved.relative())
        .map_err(|error| output_file_io_error(plan, &envelope.path, error))?;
    let metadata = file.metadata().map_err(|error| output_file_io_error(plan, &envelope.path, error))?;
    let metadata_bytes = usize::try_from(metadata.len()).map_err(|_| {
        AgentError::Adapter(format!(
            "{} agent output file is too large to address on this platform: {}",
            plan.adapter_label, envelope.path
        ))
    })?;
    validate_output_file_size(plan, &envelope.path, metadata_bytes)?;
    let bytes = crate::capability::read_bounded(file, plan.max_output_bytes)
        .map_err(|error| output_file_io_error(plan, &envelope.path, error))?;
    validate_output_file_size(plan, &envelope.path, bytes.len())?;
    debug_assert!(bytes.len() <= plan.max_output_bytes);
    Ok(AgentOutputFile {
        path_label: envelope.path.clone(),
        bytes,
    })
}

fn resolve_handoff_path(plan: &AgentCommandPlan, path: &str) -> Result<crate::capability::RootedPath, AgentError> {
    let base = plan.cwd.as_deref().unwrap_or(CURRENT_DIRECTORY);
    crate::capability::rooted_relative_path(Path::new(base), Path::new(path)).ok_or_else(|| {
        AgentError::Adapter(format!("{} agent output file is outside working directory: {path}", plan.adapter_label))
    })
}

fn validate_output_file_size(plan: &AgentCommandPlan, path: &str, bytes: usize) -> Result<(), AgentError> {
    if bytes <= plan.max_output_bytes {
        return Ok(());
    }
    Err(AgentError::Adapter(format!(
        "{} agent output file is too large: {path} has {bytes} bytes, max {}",
        plan.adapter_label, plan.max_output_bytes
    )))
}

fn output_file_io_error(plan: &AgentCommandPlan, path: &str, error: std::io::Error) -> AgentError {
    match error.kind() {
        ErrorKind::NotFound => {
            AgentError::Adapter(format!("{} agent output file is missing: {path}", plan.adapter_label))
        }
        ErrorKind::PermissionDenied => {
            AgentError::Adapter(format!("{} agent output file permission denied: {path}", plan.adapter_label))
        }
        _ => AgentError::Adapter(format!("{} agent output file I/O error: {path}: {error}", plan.adapter_label)),
    }
}

fn validate_declared_media_type(
    plan: &AgentCommandPlan,
    declared: Option<&str>,
    sniffed: &str,
) -> Result<(), AgentError> {
    let Some(declared) = declared else {
        return Ok(());
    };
    if declared == sniffed {
        return Ok(());
    }
    Err(AgentError::Adapter(format!(
        "{} agent image media type mismatch: declared {declared}, sniffed {sniffed}",
        plan.adapter_label
    )))
}

fn command_metadata(
    plan: &AgentCommandPlan,
    output: &AgentCommandOutput,
    file_metadata: Option<serde_json::Value>,
) -> serde_json::Value {
    let mut metadata = serde_json::json!({
        "adapter": plan.adapter_label,
        "target": agent_target_label(plan.target),
        "stdout_bytes": output.stdout.len(),
    });
    if let Some(file_metadata) = file_metadata {
        metadata["file_handoff"] = file_metadata;
    }
    metadata
}

fn file_metadata(envelope: &AgentOutputFileEnvelope, path_label: &str, bytes: usize) -> serde_json::Value {
    serde_json::json!({
        HANDOFF_PATH_FIELD: path_label,
        "bytes": bytes,
        HANDOFF_MEDIA_TYPE_FIELD: envelope.media_type.clone(),
        HANDOFF_METADATA_FIELD: envelope.metadata.clone(),
    })
}

fn status_label(status_code: Option<i32>) -> String {
    match status_code {
        Some(code) => code.to_string(),
        None => "terminated by signal".to_string(),
    }
}

fn bounded_error_text(text: &str) -> String {
    text.chars().take(ERROR_OUTPUT_LIMIT_CHARS).collect()
}

// r[impl n8nclone.workflow_execution.agent_nodes.execute]
pub fn request_inputs_from_values(values: &[Value]) -> (Vec<String>, Vec<ImageValue>, Vec<serde_json::Value>) {
    let mut text_inputs = Vec::new();
    let mut image_inputs = Vec::new();
    let mut json_inputs = Vec::new();

    for value in values {
        match &value.data {
            ValueData::String(text) => text_inputs.push(text.clone()),
            ValueData::Image(image) => image_inputs.push(image.clone()),
            ValueData::Json(json) => json_inputs.push(json.clone()),
            ValueData::Bytes(_) | ValueData::ExitCode(_) | ValueData::Error { .. } => {}
        }
    }

    (text_inputs, image_inputs, json_inputs)
}

// r[impl n8nclone.workflow_execution.agent_nodes.execute]
pub fn validate_agent_response_mode(response: &AgentResponse, expected: AgentOutputMode) -> Result<(), AgentError> {
    let actual = response.output.output_mode();
    if actual == expected {
        Ok(())
    } else {
        Err(AgentError::OutputTypeMismatch { expected, actual })
    }
}

//! Generic agent node request planning and adapter interface.

use std::process::Command;

use serde::Deserialize;
use serde::Serialize;

use crate::engine::value::ImageValue;
use crate::engine::value::Value;
use crate::engine::value::ValueData;

const DEFAULT_PI_COMMAND: &str = "pi";
const DEFAULT_CLANKERS_COMMAND: &str = "clankers";
const PRINT_MODE_FLAG: &str = "-p";
const MODEL_FLAG: &str = "--model";
const SESSION_FLAG: &str = "--session";
const NO_SESSION_FLAG: &str = "--no-session";
const TEXT_INPUT_HEADING: &str = "Text input";
const JSON_INPUT_HEADING: &str = "JSON input";
const IMAGE_INPUT_HEADING: &str = "Image input";
const CONTEXT_REFS_HEADING: &str = "Context references";
const TOOL_POLICY_HEADING: &str = "Tool policy";
const MODEL_HINT_HEADING: &str = "Model hint";
const DISPLAY_INDEX_BASE: usize = 1;
const ERROR_OUTPUT_LIMIT_CHARS: usize = 2048;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentTarget {
    Pi,
    Clankers,
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
    pub output_mode: AgentOutputMode,
    pub model_hint: Option<String>,
    pub tool_policy: String,
    pub context_refs: Vec<String>,
    pub cwd: Option<String>,
    pub session: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentRuntimeConfig {
    #[serde(default = "default_pi_command")]
    pub pi_command: Option<String>,
    #[serde(default = "default_clankers_command")]
    pub clankers_command: Option<String>,
}

impl Default for AgentRuntimeConfig {
    fn default() -> Self {
        Self {
            pi_command: default_pi_command(),
            clankers_command: default_clankers_command(),
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
    fn execute(&self, request: &AgentRequest) -> Result<AgentResponse, AgentError>;
}

#[derive(Debug, Default)]
pub struct NoopAgentAdapter;

impl AgentAdapter for NoopAgentAdapter {
    fn execute(&self, request: &AgentRequest) -> Result<AgentResponse, AgentError> {
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentCommandOutput {
    pub status_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Clone, Default)]
pub struct ProductionAgentAdapter {
    config: AgentRuntimeConfig,
}

impl ProductionAgentAdapter {
    pub fn new(config: AgentRuntimeConfig) -> Self {
        Self { config }
    }
}

impl AgentAdapter for ProductionAgentAdapter {
    fn execute(&self, request: &AgentRequest) -> Result<AgentResponse, AgentError> {
        let plan = plan_agent_command(&self.config, request)?;
        let output = run_agent_command(&plan)?;
        decode_agent_command_output(&plan, &output)
    }
}

fn default_pi_command() -> Option<String> {
    Some(DEFAULT_PI_COMMAND.to_string())
}

fn default_clankers_command() -> Option<String> {
    Some(DEFAULT_CLANKERS_COMMAND.to_string())
}

impl AgentNodeSettings {
    pub fn pi(prompt: impl Into<String>, output_mode: AgentOutputMode) -> Self {
        Self::new(AgentTarget::Pi, prompt, output_mode)
    }

    pub fn clankers(prompt: impl Into<String>, output_mode: AgentOutputMode) -> Self {
        Self::new(AgentTarget::Clankers, prompt, output_mode)
    }

    pub fn new(target: AgentTarget, prompt: impl Into<String>, output_mode: AgentOutputMode) -> Self {
        Self {
            target,
            prompt: prompt.into(),
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
        AgentTarget::Pi => "pi",
        AgentTarget::Clankers => "clankers",
    }
}

pub fn parse_agent_target(value: &str) -> Result<AgentTarget, AgentError> {
    match value {
        "pi" => Ok(AgentTarget::Pi),
        "clankers" => Ok(AgentTarget::Clankers),
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
pub fn plan_agent_command(config: &AgentRuntimeConfig, request: &AgentRequest) -> Result<AgentCommandPlan, AgentError> {
    validate_command_supported_output_mode(request.target, request.output_mode)?;
    let program = command_for_target(config, request.target)?;
    let prompt = render_agent_command_prompt(request);
    let args = command_args_for_request(request, &prompt);
    Ok(AgentCommandPlan {
        target: request.target,
        adapter_label: agent_target_label(request.target),
        program,
        args,
        cwd: request.cwd.clone(),
        output_mode: request.output_mode,
    })
}

fn validate_command_supported_output_mode(target: AgentTarget, output_mode: AgentOutputMode) -> Result<(), AgentError> {
    match output_mode {
        AgentOutputMode::Text | AgentOutputMode::Json => Ok(()),
        AgentOutputMode::Image | AgentOutputMode::Artifact => Err(AgentError::Adapter(format!(
            "production {} command adapter does not support {} output mode",
            agent_target_label(target),
            output_mode.as_str()
        ))),
    }
}

fn command_for_target(config: &AgentRuntimeConfig, target: AgentTarget) -> Result<String, AgentError> {
    let command = match target {
        AgentTarget::Pi => &config.pi_command,
        AgentTarget::Clankers => &config.clankers_command,
    };
    let Some(command) = command.as_ref().map(|value| value.trim()).filter(|value| !value.is_empty()) else {
        return Err(AgentError::PreflightRejected(format!(
            "missing {} agent command configuration",
            agent_target_label(target)
        )));
    };
    Ok(command.to_string())
}

fn command_args_for_request(request: &AgentRequest, prompt: &str) -> Vec<String> {
    match request.target {
        AgentTarget::Pi => pi_command_args(request, prompt),
        AgentTarget::Clankers => clankers_command_args(request, prompt),
    }
}

fn pi_command_args(request: &AgentRequest, prompt: &str) -> Vec<String> {
    let mut args = Vec::new();
    if let Some(model_hint) = request.model_hint.as_ref().filter(|value| !value.trim().is_empty()) {
        args.push(MODEL_FLAG.to_string());
        args.push(model_hint.clone());
    }
    if let Some(session) = request.session.as_ref().filter(|value| !value.trim().is_empty()) {
        args.push(SESSION_FLAG.to_string());
        args.push(session.clone());
    } else {
        args.push(NO_SESSION_FLAG.to_string());
    }
    args.push(PRINT_MODE_FLAG.to_string());
    args.push(prompt.to_string());
    args
}

fn clankers_command_args(request: &AgentRequest, prompt: &str) -> Vec<String> {
    let mut args = Vec::new();
    if let Some(model_hint) = request.model_hint.as_ref().filter(|value| !value.trim().is_empty()) {
        args.push(MODEL_FLAG.to_string());
        args.push(model_hint.clone());
    }
    args.push(PRINT_MODE_FLAG.to_string());
    args.push(prompt.to_string());
    args
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
    let mut command = Command::new(&plan.program);
    command.args(&plan.args);
    if let Some(cwd) = plan.cwd.as_ref().filter(|value| !value.trim().is_empty()) {
        command.current_dir(cwd);
    }
    let output = command.output().map_err(|error| {
        AgentError::Adapter(format!("failed to spawn {} agent command '{}': {error}", plan.adapter_label, plan.program))
    })?;
    Ok(AgentCommandOutput {
        status_code: output.status.code(),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    })
}

// r[impl n8nclone.workflow_execution.agent_nodes.production_adapters.command]
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

    let metadata = serde_json::json!({
        "adapter": plan.adapter_label,
        "target": agent_target_label(plan.target),
        "stdout_bytes": output.stdout.len(),
    });
    match plan.output_mode {
        AgentOutputMode::Text => Ok(AgentResponse {
            output: AgentOutput::Text(output.stdout.trim_end().to_string()),
            metadata,
        }),
        AgentOutputMode::Json => {
            let json = serde_json::from_str(output.stdout.trim()).map_err(|error| {
                AgentError::Adapter(format!("{} agent command returned invalid JSON: {error}", plan.adapter_label))
            })?;
            Ok(AgentResponse {
                output: AgentOutput::Json(json),
                metadata,
            })
        }
        AgentOutputMode::Image | AgentOutputMode::Artifact => Err(AgentError::Adapter(format!(
            "production {} command adapter does not support {} output mode",
            plan.adapter_label,
            plan.output_mode.as_str()
        ))),
    }
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

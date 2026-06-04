//! Generic agent node request planning and adapter interface.

use serde::Deserialize;
use serde::Serialize;

use crate::engine::value::ImageValue;
use crate::engine::value::Value;
use crate::engine::value::ValueData;

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

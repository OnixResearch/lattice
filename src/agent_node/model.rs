const DEFAULT_PI_RUNTIME_KEY: &str = "pi";
const DEFAULT_CLANKERS_RUNTIME_KEY: &str = "clankers";
const DEFAULT_ANIMUS_RUNTIME_KEY: &str = "animus";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Target {
    Pi,
    Clankers,
    Animus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum OutputMode {
    Text,
    Json,
    Image,
    Artifact,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Request {
    pub target: Target,
    pub prompt: String,
    pub output_mode: OutputMode,
    pub model_hint: Option<String>,
    pub tool_policy: String,
    pub context_refs: Vec<String>,
    pub cwd: Option<String>,
    pub session: Option<String>,
    pub text_inputs: Vec<String>,
    pub image_inputs: Vec<crate::engine::value::ImageValue>,
    pub json_inputs: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Response {
    pub output: Output,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Output {
    Text(String),
    Json(serde_json::Value),
    Image(crate::engine::value::ImageValue),
    Artifact(Vec<u8>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    UnsupportedTarget(String),
    MissingPrompt,
    PreflightRejected(String),
    Credential(String),
    Session(String),
    Adapter(String),
    OutputTypeMismatch { expected: OutputMode, actual: OutputMode },
}

impl std::fmt::Display for Error {
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

impl std::error::Error for Error {}

impl OutputMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Json => "json",
            Self::Image => "image",
            Self::Artifact => "artifact",
        }
    }
}

impl std::fmt::Display for OutputMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Output {
    pub fn output_mode(&self) -> OutputMode {
        match self {
            Self::Text(_) => OutputMode::Text,
            Self::Json(_) => OutputMode::Json,
            Self::Image(_) => OutputMode::Image,
            Self::Artifact(_) => OutputMode::Artifact,
        }
    }
}

pub fn target_label(target: Target) -> &'static str {
    match target {
        Target::Pi => DEFAULT_PI_RUNTIME_KEY,
        Target::Clankers => DEFAULT_CLANKERS_RUNTIME_KEY,
        Target::Animus => DEFAULT_ANIMUS_RUNTIME_KEY,
    }
}

pub fn parse_target(value: &str) -> Result<Target, Error> {
    match value {
        DEFAULT_PI_RUNTIME_KEY => Ok(Target::Pi),
        DEFAULT_CLANKERS_RUNTIME_KEY => Ok(Target::Clankers),
        DEFAULT_ANIMUS_RUNTIME_KEY => Ok(Target::Animus),
        _ => Err(Error::UnsupportedTarget(value.to_string())),
    }
}

pub fn parse_output_mode(value: &str) -> Result<OutputMode, Error> {
    match value {
        "text" => Ok(OutputMode::Text),
        "json" => Ok(OutputMode::Json),
        "image" => Ok(OutputMode::Image),
        "artifact" => Ok(OutputMode::Artifact),
        _ => Err(Error::Adapter(format!("unsupported agent output mode: {value}"))),
    }
}

// r[impl n8nclone.workflow_execution.agent_nodes.execute]
pub fn request_inputs_from_values(
    values: &[crate::engine::value::Value],
) -> (Vec<String>, Vec<crate::engine::value::ImageValue>, Vec<serde_json::Value>) {
    let counts = InputCounts::from_values(values);
    let mut text_inputs = Vec::with_capacity(counts.text_count);
    let mut image_inputs = Vec::with_capacity(counts.image_count);
    let mut json_inputs = Vec::with_capacity(counts.json_count);
    for value in values {
        match &value.data {
            crate::engine::value::ValueData::String(text) => text_inputs.push(text.clone()),
            crate::engine::value::ValueData::Image(image) => image_inputs.push(image.clone()),
            crate::engine::value::ValueData::Json(json) => json_inputs.push(json.clone()),
            crate::engine::value::ValueData::Bytes(_)
            | crate::engine::value::ValueData::ExitCode(_)
            | crate::engine::value::ValueData::Error { .. } => {}
        }
    }
    (text_inputs, image_inputs, json_inputs)
}

#[derive(Default)]
struct InputCounts {
    text_count: usize,
    image_count: usize,
    json_count: usize,
}

impl InputCounts {
    fn from_values(values: &[crate::engine::value::Value]) -> Self {
        let mut counts = Self::default();
        for value in values {
            match &value.data {
                crate::engine::value::ValueData::String(_) => counts.text_count = counts.text_count.saturating_add(1),
                crate::engine::value::ValueData::Image(_) => counts.image_count = counts.image_count.saturating_add(1),
                crate::engine::value::ValueData::Json(_) => counts.json_count = counts.json_count.saturating_add(1),
                crate::engine::value::ValueData::Bytes(_)
                | crate::engine::value::ValueData::ExitCode(_)
                | crate::engine::value::ValueData::Error { .. } => {}
            }
        }
        counts
    }
}

// r[impl n8nclone.workflow_execution.agent_nodes.execute]
pub fn validate_response_mode(response: &Response, expected: OutputMode) -> Result<(), Error> {
    let actual = response.output.output_mode();
    if actual == expected {
        Ok(())
    } else {
        Err(Error::OutputTypeMismatch { expected, actual })
    }
}

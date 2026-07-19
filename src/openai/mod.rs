//! OpenAI-compatible chat-completions planning, bounded transport, and decoding.

mod core;
mod transport;

pub use core::decode_response;
pub use core::execute;
pub use core::plan_chat;

pub use transport::ReqwestClient;

const DEFAULT_TIMEOUT_SECS: u64 = 120;
const DEFAULT_MAX_TOKENS: u32 = 2_048;
const DEFAULT_TEMPERATURE_MILLIUNITS: u16 = 0;
const DEFAULT_REQUEST_MAX_BYTES: u64 = 1_048_576;
const DEFAULT_RESPONSE_MAX_BYTES: u64 = 1_048_576;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RuntimeConfig {
    pub endpoint: String,
    pub model: String,
    #[serde(default = "absent_string")]
    pub api_key_env: Option<String>,
    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
    #[serde(default = "default_temperature_milliunits")]
    pub temperature_milliunits: u16,
    #[serde(default = "default_request_max_bytes")]
    pub max_request_bytes: u64,
    #[serde(default = "default_response_max_bytes")]
    pub max_response_bytes: u64,
    #[serde(default = "absent_bool")]
    pub enable_thinking: Option<bool>,
    #[serde(default = "empty_string")]
    pub system_prompt: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatPlan {
    pub adapter_label: &'static str,
    pub endpoint: String,
    pub model: String,
    pub api_key_env: Option<String>,
    pub timeout_secs: u64,
    pub max_response_bytes: u64,
    pub output_mode: crate::agent_node::AgentOutputMode,
    pub body: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpResponse {
    pub status: u16,
    pub body: Vec<u8>,
}

#[derive(Debug, serde::Deserialize)]
struct ResponseEnvelope {
    #[serde(default = "absent_string")]
    model: Option<String>,
    #[serde(default = "empty_choices")]
    choices: Vec<Choice>,
    #[serde(default = "null_json")]
    usage: serde_json::Value,
}

#[derive(Debug, serde::Deserialize)]
struct Choice {
    message: Message,
    #[serde(default = "absent_string")]
    finish_reason: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct Message {
    #[serde(default = "absent_string")]
    content: Option<String>,
}

pub trait Transport: Send + Sync {
    fn execute(
        &self,
        plan: &ChatPlan,
        bearer_token: Option<&str>,
    ) -> Result<HttpResponse, crate::agent_node::AgentError>;
}

pub fn default_transport() -> std::sync::Arc<dyn Transport> {
    std::sync::Arc::new(ReqwestClient)
}

const fn default_timeout_secs() -> u64 {
    DEFAULT_TIMEOUT_SECS
}

const fn default_max_tokens() -> u32 {
    DEFAULT_MAX_TOKENS
}

const fn default_temperature_milliunits() -> u16 {
    DEFAULT_TEMPERATURE_MILLIUNITS
}

const fn default_request_max_bytes() -> u64 {
    DEFAULT_REQUEST_MAX_BYTES
}

const fn default_response_max_bytes() -> u64 {
    DEFAULT_RESPONSE_MAX_BYTES
}

fn absent_string() -> Option<String> {
    None
}

fn empty_choices() -> Vec<Choice> {
    Vec::new()
}

const fn null_json() -> serde_json::Value {
    serde_json::Value::Null
}

const fn absent_bool() -> Option<bool> {
    None
}

fn empty_string() -> String {
    String::new()
}

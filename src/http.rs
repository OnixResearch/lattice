//! HTTP request planning and client interface.

use std::collections::HashMap;

use serde::Deserialize;
use serde::Serialize;

const SUPPORTED_METHODS: &[&str] = &["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD"];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HttpRequestPlan {
    pub method: String,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
    pub timeout_secs: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HttpError {
    InvalidRequest(String),
    Timeout(String),
    Transport(String),
}

impl std::fmt::Display for HttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidRequest(message) => write!(f, "invalid HTTP request: {message}"),
            Self::Timeout(message) => write!(f, "HTTP request timed out: {message}"),
            Self::Transport(message) => write!(f, "HTTP transport error: {message}"),
        }
    }
}

impl std::error::Error for HttpError {}

pub trait HttpClient: Send + Sync {
    fn execute(&self, request: &HttpRequestPlan) -> Result<HttpResponse, HttpError>;
}

#[derive(Debug, Default)]
pub struct NoopHttpClient;

impl HttpClient for NoopHttpClient {
    fn execute(&self, _request: &HttpRequestPlan) -> Result<HttpResponse, HttpError> {
        Err(HttpError::Transport("no HTTP client configured".to_string()))
    }
}

#[derive(Debug, Clone)]
pub enum StaticHttpResult {
    Response(HttpResponse),
    Error(HttpError),
}

#[derive(Debug, Clone, Default)]
pub struct StaticHttpClient {
    responses: HashMap<String, StaticHttpResult>,
}

impl StaticHttpClient {
    pub fn new(responses: HashMap<String, StaticHttpResult>) -> Self {
        Self { responses }
    }
}

impl HttpClient for StaticHttpClient {
    fn execute(&self, request: &HttpRequestPlan) -> Result<HttpResponse, HttpError> {
        self.responses
            .get(&request.url)
            .cloned()
            .ok_or_else(|| HttpError::Transport(format!("no fixture for {}", request.url)))
            .and_then(|result| match result {
                StaticHttpResult::Response(response) => Ok(response),
                StaticHttpResult::Error(error) => Err(error),
            })
    }
}

pub fn normalized_method(method: &str) -> String {
    method.trim().to_ascii_uppercase()
}

// r[impl n8nclone.workflow_execution.http_node.config]
pub fn validate_http_method(method: &str) -> bool {
    let normalized = normalized_method(method);
    SUPPORTED_METHODS.contains(&normalized.as_str())
}

// r[impl n8nclone.workflow_execution.http_node.config]
pub fn validate_http_url(url: &str) -> bool {
    let Ok(parsed) = url::Url::parse(url) else {
        return false;
    };
    matches!(parsed.scheme(), "http" | "https")
}

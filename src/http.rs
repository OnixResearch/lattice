//! HTTP request planning and client interface.

mod native;

pub use native::NativeHttpClient;

pub const MAX_URL_BYTES: usize = 8_192;
pub const MAX_REQUEST_BODY_BYTES: usize = 1_048_576;
pub const MAX_RESPONSE_BODY_BYTES: usize = 8_388_608;
pub const MAX_HEADER_COUNT: usize = 64;
pub const MAX_HEADER_BYTES: usize = 16_384;
pub const MAX_TIMEOUT_SECS: u64 = 300;
pub const MAX_CONNECT_TIMEOUT_SECS: u64 = 10;

/// A caller may narrow, but never enlarge, native transport ceilings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HttpBounds {
    pub response_body_bytes: usize,
    pub header_bytes: usize,
    /// Covers the entire DNS, TCP and TLS connection establishment together.
    pub connect_timeout_secs: u64,
}

impl Default for HttpBounds {
    fn default() -> Self {
        Self {
            response_body_bytes: MAX_RESPONSE_BODY_BYTES,
            header_bytes: MAX_HEADER_BYTES,
            connect_timeout_secs: MAX_CONNECT_TIMEOUT_SECS,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpLimit {
    Url,
    RequestBody,
    ResponseBody,
    RequestHeaderCount,
    RequestHeaderBytes,
    ResponseHeaderCount,
    ResponseHeaderBytes,
    Deadline,
}

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
    LimitExceeded(HttpLimit),
    Cancelled,
}

impl std::fmt::Display for HttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidRequest(message) => write!(f, "invalid HTTP request: {message}"),
            Self::Timeout(message) => write!(f, "HTTP request timed out: {message}"),
            Self::Transport(message) => write!(f, "HTTP transport error: {message}"),
            Self::LimitExceeded(limit) => write!(f, "HTTP limit exceeded: {limit:?}"),
            Self::Cancelled => write!(f, "HTTP request cancelled; remote outcome may be unknown"),
        }
    }
}

impl std::error::Error for HttpError {}

pub trait HttpClient: Send + Sync {
    fn execute(&self, request: &HttpRequestPlan) -> Result<HttpResponse, HttpError>;

    /// Embeddings without in-flight cancellation still reject pre-cancelled work.
    /// Native transport overrides this to drop in-flight local I/O as well.
    fn execute_with_cancellation(
        &self,
        request: &HttpRequestPlan,
        cancellation: Option<&std::sync::atomic::AtomicBool>,
    ) -> Result<HttpResponse, HttpError> {
        if cancellation.is_some_and(|flag| flag.load(std::sync::atomic::Ordering::Acquire)) {
            return Err(HttpError::Cancelled);
        }
        self.execute(request)
    }

    /// Explicit bounds cannot be silently ignored by an embedding client.
    fn execute_bounded(
        &self,
        _request: &HttpRequestPlan,
        _bounds: &HttpBounds,
        _cancellation: Option<&std::sync::atomic::AtomicBool>,
    ) -> Result<HttpResponse, HttpError> {
        Err(HttpError::InvalidRequest("bounded HTTP execution unsupported".to_owned()))
    }
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
            .ok_or_else(|| HttpError::Transport("no HTTP fixture configured".to_owned()))
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
        && parsed.host_str().is_some()
        && parsed.username().is_empty()
        && parsed.password().is_none()
        && parsed.fragment().is_none()
}

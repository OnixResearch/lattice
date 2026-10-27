//! Native private gateway transport. Pure DTOs and exact canonical bindings;
//! execution, authority, journal state and signing remain in the gateway shell.
use alloc::string::String;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

pub const API_VERSION: u32 = 1;
pub const FRAME_BYTES_MAX: u32 = 1_048_576;
pub const REQUEST_BYTES_MAX: u32 = 131_072;
pub const EVENT_PAGE_MAX: u32 = 64;
pub const LOG_WINDOW_MAX: u32 = 65_536;
pub const SIGNED_RESPONSE_SCHEMA: &str = "mantle-gateway-signed-response-v1";
pub const RESPONSE_DOMAIN: &[u8] = b"mantle-gateway-signed-response-v1\0";
pub const COMPLETION_DOMAIN: &[u8] = b"mantle-gateway-completion-v1\0";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    Queued,
    Running,
    Succeeded,
    Failed,
    Cancelled,
}
impl Phase {
    pub const fn terminal(self) -> bool {
        matches!(self, Self::Succeeded | Self::Failed | Self::Cancelled)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Observation {
    pub phase: Phase,
    pub fence: u64,
    pub result_evidence: Option<[u8; 32]>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Completion {
    pub version: u32,
    pub event_id: [u8; 32],
    pub attempt_id: String,
    pub request_digest: [u8; 32],
    pub terminal: Phase,
    pub result_evidence: Option<[u8; 32]>,
    pub policy_version: [u8; 32],
    pub sequence: u64,
    pub producer: [u8; 32],
    pub signature: Vec<u8>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Event {
    pub sequence: u64,
    pub phase: Phase,
    pub completion: Option<Completion>,
    /// Exact event-time observation. Legacy journals may lack it; such events
    /// cannot be promoted into the signed observation protocol.
    #[serde(default)]
    pub observation: Option<Observation>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Status {
    pub attempt_id: String,
    pub request_digest: [u8; 32],
    pub observation: Observation,
    pub cancel_requested: bool,
    pub sequence: u64,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Page<T> {
    pub values: Vec<T>,
    pub next_cursor: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LogWindow {
    pub bytes: Vec<u8>,
    pub next_cursor: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Usage {
    pub admitted: u64,
    pub active: u64,
    pub completed: u64,
    pub cpu_millis: u64,
    pub output_bytes: u64,
    pub accounting: Option<UsageAccounting>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UsageAccounting {
    pub schedule_version: u32,
    pub cpu_ms_per_unit: u64,
    pub memory_byte_ms_per_unit: u64,
    pub executor_output_bytes_per_unit: u64,
    pub project_limit_units: u64,
    pub account_limit_units: u64,
    pub project_reason_counts: UsageReasonCounts,
    pub account_reason_counts: UsageReasonCounts,
    pub unit: String,
    pub window_id: String,
    pub window_start_s: u64,
    pub window_end_s: u64,
    pub project_reserved_units: u64,
    pub project_charged_units: u64,
    pub account_reserved_units: u64,
    pub account_charged_units: u64,
    pub memory_measurement: String,
    pub output_measurement: String,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UsageReasonCounts {
    pub reserved: u64,
    pub settled: u64,
    pub cancelled_without_final_usage: u64,
    pub worker_lost: u64,
    pub partial_observation: u64,
    pub untrusted_observation: u64,
    pub usage_exceeds_reservation: u64,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Response {
    Submitted { attempt_id: String, recovered: bool },
    Status(Status),
    Logs(LogWindow),
    Events(Page<Event>),
    CancelRequested { attempt_id: String, terminal: bool },
    Result(Option<Completion>),
    Usage(Usage),
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "operation", rename_all = "kebab-case", deny_unknown_fields)]
pub enum Request {
    Submit {
        drv_path: String,
        idempotency_key: String,
    },
    Status {
        attempt_id: String,
    },
    Logs {
        attempt_id: String,
        cursor: Option<String>,
        bytes: u32,
    },
    Events {
        attempt_id: String,
        cursor: Option<String>,
        items: u32,
    },
    Cancel {
        attempt_id: String,
        fence: u64,
    },
    Result {
        attempt_id: String,
    },
    Usage {
        items: u32,
    },
}
impl Request {
    pub fn target<'a>(&'a self, scope: &'a str) -> &'a str {
        match self {
            Self::Submit { drv_path, .. } => drv_path,
            Self::Status { attempt_id }
            | Self::Logs { attempt_id, .. }
            | Self::Events { attempt_id, .. }
            | Self::Cancel { attempt_id, .. }
            | Self::Result { attempt_id } => attempt_id,
            Self::Usage { .. } => scope,
        }
    }
    pub fn attempt(&self) -> Option<&str> {
        match self {
            Self::Submit { .. } | Self::Usage { .. } => None,
            _ => Some(self.target("")),
        }
    }
}

/// Entire actual native response, authenticated after native authorization and
/// host/journal execution. No bearer or holder signature is retained here.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SignedResponse {
    pub schema: String,
    pub service: String,
    pub project: String,
    pub owner: String,
    pub request_digest: [u8; 32],
    pub created_seconds: u64,
    pub status: Option<Status>,
    pub producer: [u8; 32],
    pub response: Response,
    pub signature: Vec<u8>,
}

pub fn request_digest(request: &Request, scope: &str, generation: u64) -> Result<[u8; 32], serde_json::Error> {
    let bytes = match request {
        Request::Submit {
            drv_path,
            idempotency_key,
        } => serde_json::to_vec(&("submit", drv_path, idempotency_key)),
        Request::Status { attempt_id } => serde_json::to_vec(&("status", attempt_id)),
        Request::Logs {
            attempt_id,
            cursor,
            bytes,
        } => serde_json::to_vec(&("logs", attempt_id, cursor, bytes)),
        Request::Events {
            attempt_id,
            cursor,
            items,
        } => serde_json::to_vec(&("events", attempt_id, cursor, items)),
        Request::Cancel { attempt_id, fence } => serde_json::to_vec(&("cancel", attempt_id, fence)),
        Request::Result { attempt_id } => serde_json::to_vec(&("result", attempt_id)),
        Request::Usage { items } => serde_json::to_vec(&("usage", scope, items)),
    }?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"mantle-gateway-operation-v1\0");
    hasher.update(&generation.to_le_bytes());
    hasher.update(&bytes);
    Ok(*hasher.finalize().as_bytes())
}

pub fn response_digest(response: &SignedResponse) -> Result<[u8; 32], serde_json::Error> {
    let bytes = serde_json::to_vec(&(
        &response.schema,
        &response.service,
        &response.project,
        &response.owner,
        response.request_digest,
        response.created_seconds,
        response.producer,
        &response.response,
        &response.status,
    ))?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(RESPONSE_DOMAIN);
    hasher.update(&bytes);
    Ok(*hasher.finalize().as_bytes())
}

pub fn completion_identity(event: &Completion) -> Result<[u8; 32], serde_json::Error> {
    let bytes = serde_json::to_vec(&(
        event.version,
        &event.attempt_id,
        event.request_digest,
        event.terminal,
        event.result_evidence,
        event.policy_version,
        event.sequence,
        event.producer,
    ))?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(COMPLETION_DOMAIN);
    hasher.update(&bytes);
    Ok(*hasher.finalize().as_bytes())
}

/// Stable event identity independent of response-signing time and page shape.
/// Native terminal completions supply their independently signed identity.
pub fn event_identity(
    status: &Status,
    phase: Phase,
    sequence: u64,
    evidence: Option<[u8; 32]>,
) -> Result<[u8; 32], serde_json::Error> {
    let bytes = serde_json::to_vec(&(
        &status.attempt_id,
        status.request_digest,
        status.observation.fence,
        phase,
        sequence,
        evidence,
    ))?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"mantle-gateway-observed-event-v1\0");
    hasher.update(&bytes);
    Ok(*hasher.finalize().as_bytes())
}

pub fn safe_segment(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 256
        && value != "."
        && value != ".."
        && value.bytes().all(|byte| byte.is_ascii_alphanumeric() || b"_+.-=".contains(&byte))
}

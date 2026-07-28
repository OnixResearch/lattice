pub const DEFAULT_BIND_ADDR: &str = "127.0.0.1:8080";
pub const DEFAULT_MAX_BODY_BYTES: usize = 1_048_576;
pub const DEFAULT_HISTORY_LOOKBACK_RECORDS: usize = 4096;

pub(crate) const HTTP_READ_CHUNK_BYTES: usize = 1024;
pub(crate) const MAX_HTTP_HEADER_BYTES: usize = 16_384;
pub(crate) const MAX_HTTP_HEADER_LINES: usize = 128;
pub(crate) const MAX_QUERY_PAIR_COUNT: usize = 128;
pub(crate) const DRY_RUN_DIAGNOSTICS_PER_TRIGGER: usize = 4;
pub(crate) const HTTP_STATUS_ACCEPTED: u16 = 202;
pub(crate) const HTTP_STATUS_BAD_REQUEST: u16 = 400;
pub(crate) const HTTP_STATUS_UNAUTHORIZED: u16 = 401;
pub(crate) const HTTP_STATUS_NOT_FOUND: u16 = 404;
pub(crate) const HTTP_STATUS_METHOD_NOT_ALLOWED: u16 = 405;
pub(crate) const HTTP_STATUS_CONFLICT: u16 = 409;
pub(crate) const HTTP_STATUS_PAYLOAD_TOO_LARGE: u16 = 413;
pub(crate) const HTTP_STATUS_UNSUPPORTED_MEDIA_TYPE: u16 = 415;
pub(crate) const HTTP_STATUS_INTERNAL_SERVER_ERROR: u16 = 500;
pub(crate) const HTTP_VERSION_PREFIX: &str = "HTTP/";
pub(crate) const HTTP_HEADER_TERMINATOR: &[u8] = b"\r\n\r\n";
pub(crate) const HTTP_LINE_TERMINATOR: &str = "\r\n";
pub(crate) const HTTP_HEADER_SEPARATOR: &str = ":";
pub(crate) const CONTENT_LENGTH_HEADER: &str = "content-length";
pub(crate) const CONTENT_TYPE_HEADER: &str = "content-type";
pub(crate) const IDEMPOTENCY_KEY_HEADER: &str = "idempotency-key";
pub(crate) const JSON_CONTENT_TYPE: &str = "application/json";
pub(crate) const JSON_CONTENT_TYPE_SUFFIX: &str = "+json";
pub(crate) const HTTP_CONTENT_TYPE_JSON: &str = "application/json";
pub(crate) const HTTP_CONNECTION_CLOSE: &str = "close";
pub(crate) const HTTP_REASON_ACCEPTED: &str = "Accepted";
pub(crate) const HTTP_REASON_BAD_REQUEST: &str = "Bad Request";
pub(crate) const HTTP_REASON_UNAUTHORIZED: &str = "Unauthorized";
pub(crate) const HTTP_REASON_NOT_FOUND: &str = "Not Found";
pub(crate) const HTTP_REASON_METHOD_NOT_ALLOWED: &str = "Method Not Allowed";
pub(crate) const HTTP_REASON_CONFLICT: &str = "Conflict";
pub(crate) const HTTP_REASON_PAYLOAD_TOO_LARGE: &str = "Payload Too Large";
pub(crate) const HTTP_REASON_UNSUPPORTED_MEDIA_TYPE: &str = "Unsupported Media Type";
pub(crate) const HTTP_REASON_INTERNAL_SERVER_ERROR: &str = "Internal Server Error";
pub(crate) const STATUS_ACCEPTED: &str = "accepted";
pub(crate) const STATUS_REJECTED: &str = "rejected";
pub(crate) const LISTENER_STATUS_LISTENING: &str = "listening";
pub(crate) const LISTENER_STATUS_DRY_RUN_READY: &str = "dry-run-ready";
pub(crate) const LISTENER_STATUS_DRY_RUN_BLOCKED: &str = "dry-run-blocked";
pub(crate) const STATUS_LINE_PREFIX: &str = "webhook-listener";
pub(crate) const DEFAULT_SAFE_REJECTION_MESSAGE: &str = "webhook request rejected";
pub(crate) const SAFE_NOT_FOUND_MESSAGE: &str = "no enabled webhook trigger matched the request path";
pub(crate) const SAFE_BAD_REQUEST_MESSAGE: &str = "webhook request is malformed";
pub(crate) const SAFE_MALFORMED_JSON_MESSAGE: &str = "webhook JSON body is malformed";
pub(crate) const SAFE_BODY_TOO_LARGE_MESSAGE: &str = "webhook request body exceeds the configured limit";
pub(crate) const SAFE_METHOD_MISMATCH_MESSAGE: &str = "webhook method is not allowed";
pub(crate) const SAFE_CONTENT_TYPE_MESSAGE: &str = "webhook content type is not accepted";
pub(crate) const SAFE_AUTH_MESSAGE: &str = "webhook authentication failed";
pub(crate) const SAFE_MISSING_SECRET_MESSAGE: &str = "webhook authentication secret is unavailable";
pub(crate) const SAFE_DUPLICATE_MESSAGE: &str = "webhook idempotency key was already accepted";
pub(crate) const SAFE_OVERLAP_MESSAGE: &str = "webhook trigger already has a running workflow";
pub(crate) const SAFE_DISABLED_MESSAGE: &str = "webhook trigger is disabled";
pub(crate) const SAFE_MISSING_IDEMPOTENCY_MESSAGE: &str = "webhook idempotency key is required";
pub(crate) const SAFE_PAYLOAD_BINDING_MESSAGE: &str = "webhook payload could not be bound";
pub(crate) const SAFE_EXECUTION_FAILED_MESSAGE: &str = "webhook workflow execution failed";
pub(crate) const DRY_RUN_CREDENTIAL_CODE: &str = "credential_unavailable";
pub(crate) const DRY_RUN_DEFINITION_CODE: &str = "definition_invalid";
pub(crate) const DRY_RUN_READY_MESSAGE: &str = "listener configuration is ready";
pub(crate) const DRY_RUN_BLOCKED_MESSAGE: &str = "listener configuration has diagnostics";
pub(crate) const EMPTY_TRIGGER_ID: &str = "-";

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanningCode {
    InvalidRequest,
    BodyTooLarge,
    MalformedJson,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RejectionCode {
    NotFound,
    InvalidRequest,
    BodyTooLarge,
    MalformedJson,
    MethodMismatch,
    UnsupportedContentType,
    MissingAuthSecret,
    InvalidAuth,
    MissingIdempotencyKey,
    DuplicateIdempotencyKey,
    OverlapPrevented,
    MissingPayloadField,
    PayloadTypeMismatch,
    Disabled,
    ExecutionFailed,
}

pub(crate) fn rejection_code_for_planning(code: PlanningCode) -> RejectionCode {
    match code {
        PlanningCode::InvalidRequest => RejectionCode::InvalidRequest,
        PlanningCode::BodyTooLarge => RejectionCode::BodyTooLarge,
        PlanningCode::MalformedJson => RejectionCode::MalformedJson,
    }
}

pub(crate) fn rejection_code_for_trigger(code: crate::triggers::TriggerDiagnosticCode) -> RejectionCode {
    match code {
        crate::triggers::TriggerDiagnosticCode::EmptyTriggerId
        | crate::triggers::TriggerDiagnosticCode::EmptyWorkflowName
        | crate::triggers::TriggerDiagnosticCode::MissingWorkflow
        | crate::triggers::TriggerDiagnosticCode::MalformedSecretRef => RejectionCode::InvalidRequest,
        crate::triggers::TriggerDiagnosticCode::Disabled => RejectionCode::Disabled,
        crate::triggers::TriggerDiagnosticCode::MethodMismatch => RejectionCode::MethodMismatch,
        crate::triggers::TriggerDiagnosticCode::PathMismatch => RejectionCode::NotFound,
        crate::triggers::TriggerDiagnosticCode::UnsupportedContentType => RejectionCode::UnsupportedContentType,
        crate::triggers::TriggerDiagnosticCode::MissingAuthSecret => RejectionCode::MissingAuthSecret,
        crate::triggers::TriggerDiagnosticCode::InvalidAuth => RejectionCode::InvalidAuth,
        crate::triggers::TriggerDiagnosticCode::MissingIdempotencyKey => RejectionCode::MissingIdempotencyKey,
        crate::triggers::TriggerDiagnosticCode::DuplicateIdempotencyKey => RejectionCode::DuplicateIdempotencyKey,
        crate::triggers::TriggerDiagnosticCode::OverlapPrevented => RejectionCode::OverlapPrevented,
        crate::triggers::TriggerDiagnosticCode::MissingPayloadField => RejectionCode::MissingPayloadField,
        crate::triggers::TriggerDiagnosticCode::PayloadTypeMismatch => RejectionCode::PayloadTypeMismatch,
    }
}

pub(crate) fn safe_message_for_rejection(code: RejectionCode) -> &'static str {
    match code {
        RejectionCode::NotFound => SAFE_NOT_FOUND_MESSAGE,
        RejectionCode::InvalidRequest => SAFE_BAD_REQUEST_MESSAGE,
        RejectionCode::BodyTooLarge => SAFE_BODY_TOO_LARGE_MESSAGE,
        RejectionCode::MalformedJson => SAFE_MALFORMED_JSON_MESSAGE,
        RejectionCode::MethodMismatch => SAFE_METHOD_MISMATCH_MESSAGE,
        RejectionCode::UnsupportedContentType => SAFE_CONTENT_TYPE_MESSAGE,
        RejectionCode::MissingAuthSecret => SAFE_MISSING_SECRET_MESSAGE,
        RejectionCode::InvalidAuth => SAFE_AUTH_MESSAGE,
        RejectionCode::MissingIdempotencyKey => SAFE_MISSING_IDEMPOTENCY_MESSAGE,
        RejectionCode::DuplicateIdempotencyKey => SAFE_DUPLICATE_MESSAGE,
        RejectionCode::OverlapPrevented => SAFE_OVERLAP_MESSAGE,
        RejectionCode::MissingPayloadField | RejectionCode::PayloadTypeMismatch => SAFE_PAYLOAD_BINDING_MESSAGE,
        RejectionCode::Disabled => SAFE_DISABLED_MESSAGE,
        RejectionCode::ExecutionFailed => SAFE_EXECUTION_FAILED_MESSAGE,
    }
}

#[cfg(target_pointer_width = "64")]
pub(crate) fn count_as_u64(count: usize) -> u64 {
    u64::from_ne_bytes(count.to_ne_bytes())
}

#[cfg(target_pointer_width = "32")]
pub(crate) fn count_as_u64(count: usize) -> u64 {
    u64::from(u32::from_ne_bytes(count.to_ne_bytes()))
}

#[cfg(target_pointer_width = "16")]
pub(crate) fn count_as_u64(count: usize) -> u64 {
    u64::from(u16::from_ne_bytes(count.to_ne_bytes()))
}

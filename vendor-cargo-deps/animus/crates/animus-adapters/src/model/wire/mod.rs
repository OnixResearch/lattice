pub(crate) mod failure;
pub(crate) mod fixture;
pub(crate) mod parts;
#[cfg(test)]
mod tests;
pub(crate) mod types;
pub(crate) mod validate;

pub(crate) const MODEL_CALL_ID_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("model_call_id");
pub(crate) const BODY_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("model_stream_body");
pub(crate) const TOOL_CALL_ID_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("tool_call_id");
pub(crate) const FAILURE_REASON_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("model_stream_failure_reason");
pub(crate) const CANCELLATION_REASON_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("model_stream_cancellation_reason");

/// Stable contract identity for public model stream frame DTOs.
pub const MODEL_STREAM_FRAME_CONTRACT_IDENTITY: &str = "animus.model-stream-frame.v1";

pub(crate) const MIN_ORDER: u64 = 1;
pub(crate) const RECORD_SEPARATOR: char = '\n';
pub(crate) const FIELD_SEPARATOR: char = '|';
pub(crate) const KIND_INDEX: usize = 0;
pub(crate) const CALL_INDEX: usize = 1;
pub(crate) const ORDER_INDEX: usize = 2;
pub(crate) const BODY_INDEX: usize = 3;
pub(crate) const TOOL_ID_INDEX: usize = 3;
pub(crate) const TOOL_BODY_INDEX: usize = 4;
pub(crate) const USAGE_INPUT_INDEX: usize = 3;
pub(crate) const USAGE_OUTPUT_INDEX: usize = 4;
pub(crate) const STANDARD_FIELD_COUNT: usize = 4;
pub(crate) const TERMINAL_FIELD_COUNT: usize = 3;
pub(crate) const TERMINAL_REASON_FIELD_COUNT: usize = 4;
pub(crate) const TOOL_FIELD_COUNT: usize = 5;
pub(crate) const USAGE_FIELD_COUNT: usize = 5;
pub(crate) const KIND_CONTENT: &str = "content";
pub(crate) const KIND_THOUGHT: &str = "thought";
pub(crate) const KIND_TOOL: &str = "tool";
pub(crate) const KIND_USAGE: &str = "usage";
pub(crate) const KIND_FINISH: &str = "finish";
pub(crate) const KIND_CANCELLED: &str = "cancelled";
pub(crate) const KIND_FAILURE: &str = "failure";
pub(crate) const MALFORMED_EMPTY_CHUNKS: &str = "stream fixture chunks must not be empty";
pub(crate) const MALFORMED_TRUNCATED_RECORD: &str =
    "stream fixture ended before a record separator";
pub(crate) const MALFORMED_FIELD_COUNT: &str = "stream fixture record has the wrong field count";
pub(crate) const MALFORMED_RECORD_KIND: &str = "stream fixture record kind is unsupported";
pub(crate) const MALFORMED_ORDER: &str = "stream fixture order is not a valid integer";
pub(crate) const MALFORMED_USAGE: &str = "stream fixture usage is not a valid integer";

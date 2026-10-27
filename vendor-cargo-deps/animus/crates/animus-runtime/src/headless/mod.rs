#[allow(module_file_count, non_trait_imports)]
mod control;
mod events;
#[allow(non_trait_imports)]
mod fault;
#[allow(excessive_file_length)]
mod protocol;
mod request;
mod sink;
#[allow(non_trait_imports)]
mod structure;
#[cfg(test)]
mod tests;

const OBJECTIVE_FIELD: &str = "objective";
const RUN_IDENTITY_FIELD: &str = "run_identity";
const SESSION_ID_FIELD: &str = "session_id";
const WORKSPACE_ID_FIELD: &str = "workspace_id";
const INPUT_REFERENCE_FIELD: &str = "input_reference";
const PROMPT_FIELD: &str = "prompt";
const CONTEXT_FIELD: &str = "external_context_handle";
const POLICY_PROFILE_FIELD: &str = "policy_profile";
const CAPABILITY_FIELD: &str = "allowed_capability";
const SANDBOX_FIELD: &str = "sandbox_profile";
const OUTPUT_MODE_FIELD: &str = "output_mode";
const SCHEMA_ID_FIELD: &str = "structured_schema_id";
const VALIDATION_PLAN_FIELD: &str = "validation_plan";
const NONINTERACTIVE_BEHAVIOR_FIELD: &str = "noninteractive_behavior";
const THREAD_ID_FIELD: &str = "thread_id";
const LEDGER_HASH_FIELD: &str = "ledger_hash";
const EVENT_KIND_FIELD: &str = "event_kind";
const EVENT_BODY_FIELD: &str = "event_body";
const EVENT_REDACTION_FIELD: &str = "event_redaction";
const EVENT_SEQUENCE_FIELD: &str = "event_sequence";
const PROJECTION_FIELD: &str = "event_projection";
const STRUCTURED_FIELD: &str = "structured_output_field";
const COMPLETION_STATUS_FIELD: &str = "completion_status";
const EVIDENCE_REFERENCE_FIELD: &str = "evidence_reference";
const FINAL_RESPONSE_FIELD: &str = "final_response";

const SANDBOX_READ_ONLY: &str = "read_only_workspace";
const SANDBOX_WORKSPACE_WRITE: &str = "workspace_write";
const SANDBOX_FULL_ACCESS: &str = "full_access";
const OUTPUT_TEXT: &str = "text";
const OUTPUT_STRUCTURED: &str = "structured";
const NONINTERACTIVE_FAIL_CLOSED: &str = "fail_closed";
const NONINTERACTIVE_PREAUTHORIZED: &str = "preauthorized";
const REDACTION_VISIBLE: &str = "visible";
const REDACTION_REDACTED: &str = "redacted";
const REDACTION_HASH_ONLY: &str = "hash_only";
const PROJECTION_JSONL: &str = "jsonl";
const PROJECTION_PRESERVES: &str = "preserves";
const PRESERVES_EVENT_TAG: &str = "headless-event";
const STATUS_SUCCESS: &str = "success";
const STATUS_DENIED: &str = "denied";
const STATUS_PROMPT_REQUIRED: &str = "prompt_required";
const STATUS_INPUT_REQUIRED: &str = "input_required";
const STATUS_VALIDATION_FAILED: &str = "validation_failed";
const STATUS_SERVICE_UNAVAILABLE: &str = "service_unavailable";
const STATUS_CANCELLED: &str = "cancelled";
const STATUS_EXECUTION_FAILED: &str = "execution_failed";
const KIND_THREAD_STARTED: &str = "thread_started";
const KIND_TURN_STARTED: &str = "turn_started";
const KIND_ITEM_CREATED: &str = "item_created";
const KIND_PROMPT_ASSEMBLY: &str = "prompt_assembly";
const KIND_MODEL_DISPATCH: &str = "model_dispatch";
const KIND_MODEL_STREAM_START: &str = "model_stream_start";
const KIND_MODEL_STREAM_FRAME: &str = "model_stream_frame";
const KIND_MODEL_STREAM_END: &str = "model_stream_end";
const KIND_MODEL_STREAM_FAILURE: &str = "model_stream_failure";
const KIND_TOOL_DISPATCH: &str = "tool_dispatch";
const KIND_POLICY_DECISION: &str = "policy_decision";
const KIND_BACKGROUND_WORK: &str = "background_work";
const KIND_VALIDATION: &str = "validation";
const KIND_RECEIPT: &str = "receipt";
const KIND_FINAL_RESPONSE: &str = "final_response";
const KIND_HOOK_OUTCOME: &str = "hook_outcome";
const KIND_USAGE: &str = "usage";
const KIND_COMPLETION: &str = "completion";
const KIND_FAILURE: &str = "failure";

const AUTOMATION_EVENT_VERSION: u16 = 1;
const MIN_EVENT_SEQUENCE: u64 = 1;
const DEFAULT_MAX_EVENT_BODY_BYTES: usize = 4096;
const DEFAULT_EVENT_VISIBLE_BYTES: usize = 256;

type Text = String;
type Collection<T> = Vec<T>;

pub use control::{
    RuntimeThreadController, ThreadSnapshot, ThreadStatus, validate_thread_cancellation,
    validate_thread_resume,
};
pub use events::{
    AutomationEnvelope, AutomationEventDraft, AutomationEventKind, EventPayloadSummary,
    EventProjection, EventRedaction, EventSequence, build_automation_stream, project_event,
    summarize_event_payload,
};
pub use fault::AutomationError;
pub use protocol::{
    HeadlessCommand, HeadlessCommandDraft, HeadlessCommandKind, HeadlessProtocolError,
    HeadlessProtocolErrorKind, HeadlessProtocolEvent, HeadlessProtocolEventClass, HeadlessQueueing,
    HeadlessRecordBatch, HeadlessResponse, HeadlessResponseStatus, HeadlessRuntimePhase,
    acknowledge_headless_command, map_runtime_events_to_headless, parse_headless_command_record,
    split_lf_records,
};
pub use request::{
    NoninteractiveBehavior, OutputPreference, RunIdentity, RunRequest, RunRequestDraft,
    SandboxProfile, ThreadResume, ThreadResumeDraft,
};
pub use sink::{
    CompletionReceipt, CompletionReceiptDraft, CompletionStatus, FinalOutput,
    prompt_required_completion, select_final_output,
};
pub use structure::{
    StructuredOutputDraft, StructuredOutputFailure, StructuredOutputReport, StructuredOutputStatus,
    StructuredSchemaDraft, validate_structured_output,
};

#[derive(Clone, Copy)]
struct TextField<'a> {
    field: &'static str,
    value: &'a str,
}

#[derive(Clone, Copy)]
struct OptionalTextField<'a> {
    field: &'static str,
    value: Option<&'a str>,
}

fn required_text(input: TextField<'_>) -> Result<Text, AutomationError> {
    let normalized = input.value.trim();
    if normalized.is_empty() {
        return Err(AutomationError::EmptyField { field: input.field });
    }
    Ok(Text::from(normalized))
}

fn optional_text(input: OptionalTextField<'_>) -> Option<Text> {
    input.value.and_then(|present| {
        let normalized = present.trim();
        if normalized.is_empty() {
            None
        } else {
            Some(Text::from(normalized))
        }
    })
}

fn required_optional_text(input: OptionalTextField<'_>) -> Result<Text, AutomationError> {
    optional_text(input).ok_or(AutomationError::EmptyField { field: input.field })
}

fn unique_texts(field: &'static str, values: &[&str]) -> Result<Collection<Text>, AutomationError> {
    let mut normalized = Collection::with_capacity(values.len());
    for value in values {
        let text = required_text(TextField { field, value })?;
        if normalized.iter().any(|existing| existing == &text) {
            return Err(AutomationError::DuplicateValue { field, value: text });
        }
        normalized.push(text);
    }
    Ok(normalized)
}

fn required_unique_texts(
    field: &'static str,
    values: &[&str],
) -> Result<Collection<Text>, AutomationError> {
    if values.is_empty() {
        return Err(AutomationError::EmptyField { field });
    }
    unique_texts(field, values)
}

fn unsupported(input: TextField<'_>) -> AutomationError {
    AutomationError::UnsupportedValue {
        field: input.field,
        value: Text::from(input.value),
    }
}

fn bounded_body(value: &str, max_bytes: usize) -> Result<Text, AutomationError> {
    let body = required_text(TextField {
        field: EVENT_BODY_FIELD,
        value,
    })?;
    let actual_bytes = body.len();
    if actual_bytes > max_bytes {
        return Err(AutomationError::EventBodyTooLarge {
            max_bytes,
            actual_bytes,
        });
    }
    Ok(body)
}

fn hash_text(value: &str) -> Text {
    blake3::hash(value.as_bytes()).to_hex().to_string()
}

fn prefix_at_utf8_boundary(value: &str, max_bytes: usize) -> &str {
    if value.len() <= max_bytes {
        return value;
    }
    let mut end = max_bytes;
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    &value[..end]
}

fn contains_text(values: &[Text], needle: &str) -> bool {
    values.iter().any(|value| value.as_str() == needle)
}

fn push_json_escaped(output: &mut Text, value: &str) {
    for character in value.chars() {
        match character {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            safe => output.push(safe),
        }
    }
}

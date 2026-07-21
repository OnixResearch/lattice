use super::super::*;

pub(super) const OBJECTIVE: &str = "summarize-status";
pub(super) const SESSION_ID: &str = "session-1";
pub(super) const WORKSPACE_ID: &str = "workspace-1";
pub(super) const INPUT_REFERENCE: &str = "stdin:b3:ddd";
pub(super) const DUPLICATE_INPUT_REFERENCE: &str = "stdin:b3:ddd";
pub(super) const PROMPT: &str = "Summarize the repository status.";
pub(super) const CONTEXT_HANDLE: &str = "context:b3:aaa";
pub(super) const DUPLICATE_CONTEXT: &str = "context:b3:aaa";
pub(super) const POLICY_PROFILE: &str = "ci-safe";
pub(super) const CAPABILITY_READ: &str = "workspace-read";
pub(super) const DUPLICATE_CAPABILITY: &str = "workspace-read";
pub(super) const VALIDATION_PLAN: &str = "cargo-test-headless";
pub(super) const SCHEMA_ID: &str = "schema.final.answer";
pub(super) const THREAD_ID: &str = "thread-1";
pub(super) const OTHER_THREAD_ID: &str = "thread-2";
pub(super) const LEDGER_HASH: &str =
    "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
pub(super) const OTHER_LEDGER_HASH: &str =
    "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
pub(super) const EVENT_BODY: &str = "model dispatched";
pub(super) const PROMPT_BODY: &str = "prompt assembled";
pub(super) const BACKGROUND_BODY: &str = "background queue idle";
pub(super) const VALIDATION_BODY: &str = "validation passed";
pub(super) const RECEIPT_BODY: &str = "receipt:b3:eee";
pub(super) const PROGRESS_BODY: &str = "progress: still working";
pub(super) const FINAL_BODY: &str = "final answer";
pub(super) const FAILURE_BODY: &str = "model failed";
pub(super) const REQUIRED_FIELD: &str = "answer";
pub(super) const EXTRA_FIELD: &str = "debug";
pub(super) const EVIDENCE_REFERENCE: &str = "evidence:b3:fff";
pub(super) const UNSUPPORTED_PROJECTION: &str = "debug_text";
pub(super) const THREAD_SEQUENCE: u64 = 1;
pub(super) const TURN_SEQUENCE: u64 = 2;
pub(super) const PROMPT_SEQUENCE: u64 = 3;
pub(super) const MODEL_SEQUENCE: u64 = 4;
pub(super) const STREAM_START_SEQUENCE: u64 = 5;
pub(super) const STREAM_FRAME_SEQUENCE: u64 = 6;
pub(super) const STREAM_END_SEQUENCE: u64 = 7;
pub(super) const BACKGROUND_SEQUENCE: u64 = 8;
pub(super) const VALIDATION_SEQUENCE: u64 = 9;
pub(super) const RECEIPT_SEQUENCE: u64 = 10;
pub(super) const FINAL_RESPONSE_SEQUENCE: u64 = 11;
pub(super) const FINAL_SEQUENCE: u64 = 12;
pub(super) const REORDERED_SEQUENCE: u64 = 2;
pub(super) const FIRST_EVENT_INDEX: usize = 0;
pub(super) const MODEL_EVENT_INDEX: usize = 3;
pub(super) const STREAM_START_EVENT_INDEX: usize = 4;
pub(super) const STREAM_END_EVENT_INDEX: usize = 6;
pub(super) const BACKGROUND_EVENT_INDEX: usize = 7;
pub(super) const VALIDATION_EVENT_INDEX: usize = 8;
pub(super) const RECEIPT_EVENT_INDEX: usize = 9;
pub(super) const FINAL_RESPONSE_EVENT_INDEX: usize = 10;
pub(super) const EXPECTED_STREAM_LEN: usize = 12;
pub(super) const PROJECTED_STREAM_EVENT_COUNT: usize = 4;
pub(super) const OVERSIZED_EVENT_BODY_BYTES: usize = 4097;
pub(super) const STREAM_CALL_ID: &str = "call-1";
pub(super) const STREAM_FIRST_ORDER: u64 = 1;
pub(super) const STREAM_SECOND_ORDER: u64 = 2;
pub(super) const STREAM_THIRD_ORDER: u64 = 3;

pub(super) fn base_request<'a>() -> RunRequestDraft<'a> {
    RunRequestDraft {
        objective: OBJECTIVE,
        session_id: SESSION_ID,
        workspace_id: WORKSPACE_ID,
        input_references: &[INPUT_REFERENCE],
        prompt: PROMPT,
        external_context_handles: &[CONTEXT_HANDLE],
        policy_profile: POLICY_PROFILE,
        allowed_capabilities: &[CAPABILITY_READ],
        sandbox_profile: SandboxProfile::ReadOnlyWorkspace.as_str(),
        output_mode: OUTPUT_TEXT,
        structured_schema_id: None,
        validation_plan: VALIDATION_PLAN,
        noninteractive_behavior: NoninteractiveBehavior::FailClosed.as_str(),
        resume: None,
    }
}

pub(super) fn event_body_over_limit() -> Text {
    "x".repeat(OVERSIZED_EVENT_BODY_BYTES)
}

pub(super) fn visible_event(
    sequence: u64,
    kind: AutomationEventKind,
    body: &str,
) -> AutomationEventDraft<'_> {
    AutomationEventDraft {
        sequence,
        kind: kind.as_str(),
        redaction: EventRedaction::Visible.as_str(),
        body,
    }
}

pub(super) fn valid_stream() -> Collection<AutomationEnvelope> {
    build_automation_stream(&[
        visible_event(
            THREAD_SEQUENCE,
            AutomationEventKind::ThreadStarted,
            "thread started",
        ),
        visible_event(
            TURN_SEQUENCE,
            AutomationEventKind::ItemCreated,
            PROGRESS_BODY,
        ),
        visible_event(
            PROMPT_SEQUENCE,
            AutomationEventKind::PromptAssembly,
            PROMPT_BODY,
        ),
        visible_event(
            MODEL_SEQUENCE,
            AutomationEventKind::ModelDispatch,
            EVENT_BODY,
        ),
        visible_event(
            STREAM_START_SEQUENCE,
            AutomationEventKind::ModelStreamStart,
            STREAM_CALL_ID,
        ),
        visible_event(
            STREAM_FRAME_SEQUENCE,
            AutomationEventKind::ModelStreamFrame,
            "token delta",
        ),
        visible_event(
            STREAM_END_SEQUENCE,
            AutomationEventKind::ModelStreamEnd,
            "finish",
        ),
        visible_event(
            BACKGROUND_SEQUENCE,
            AutomationEventKind::BackgroundWork,
            BACKGROUND_BODY,
        ),
        visible_event(
            VALIDATION_SEQUENCE,
            AutomationEventKind::Validation,
            VALIDATION_BODY,
        ),
        visible_event(RECEIPT_SEQUENCE, AutomationEventKind::Receipt, RECEIPT_BODY),
        visible_event(
            FINAL_RESPONSE_SEQUENCE,
            AutomationEventKind::FinalResponse,
            FINAL_BODY,
        ),
        visible_event(FINAL_SEQUENCE, AutomationEventKind::Completion, FINAL_BODY),
    ])
    .expect("valid stream should build")
}

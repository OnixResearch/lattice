use super::super::*;

const REQUEST_ID: &str = "req-1";
const PROMPT_PAYLOAD: &str = "prompt-summary";
const STEER_PAYLOAD: &str = "steer-summary";
const TOOL_NAME: &str = "search";
const TOOL_INPUT: &str = "hidden tool args";
const TOOL_ARGUMENT_NAME: &str = "query";
const TOOL_SCHEMA: &str = "schema.search.v1";
const TOOL_CALL_ID: &str = "tool-call-1";
const TOOL_VISIBLE_BUDGET: u32 = 64;
const TOOL_MUTATION_TARGET: &str = "resource:file:README";
const TOOL_CAPABILITY: &str = "tool.read";
const TOOL_DENIAL: &str = "policy_denied";
const QUEUE_ITEM_ID: &str = "queue.one";
const QUEUE_ENTRY_ID: &str = "entry.queue.one";
const QUEUE_IDENTITY: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const PRIVATE_QUEUE_TEXT: &str = "private queue text";
const TOOL_ARGUMENT_COUNT: usize = 1;
const FIRST_TURN_INDEX: usize = 0;
const FIRST_EVENT_INDEX: usize = 0;
const SECOND_EVENT_INDEX: usize = 1;
const FIRST_SEQUENCE: u64 = 1;
const EXPECTED_COMPLETE_RECORDS: usize = 2;
const FIRST_RECORD: &str = "req-1|prompt|new_turn|payload";
const SECOND_RECORD_PREFIX: &str = "req-2|prompt|new_turn|payload";
const TRUNCATED_RECORD: &str = "req-3|prompt|new_turn";
const UNICODE_SEPARATOR: char = '\u{2028}';
const MALFORMED_RECORD: &str = "not-enough-fields";
const UNSAFE_RECORD: &str = "req-4|prompt|new_turn|RAW_PROMPT: secret=abc";

#[test]
fn prompt_acknowledgement_preserves_request_id_and_events_stream_separately() {
    let command = command(
        HeadlessCommandKind::Prompt,
        Some(HeadlessQueueing::NewTurn),
        PROMPT_PAYLOAD,
    );
    let response = acknowledge_headless_command(&command, HeadlessRuntimePhase::Idle);
    let events = map_runtime_events_to_headless(&[crate::RuntimeEvent::TurnStarted {
        turn_index: FIRST_TURN_INDEX,
    }])
    .expect("runtime event should project");

    assert_eq!(response.request_id(), Some(REQUEST_ID));
    assert_eq!(response.status(), HeadlessResponseStatus::Accepted);
    assert_eq!(events[FIRST_EVENT_INDEX].request_id(), None);
    assert_eq!(events[FIRST_EVENT_INDEX].sequence(), FIRST_SEQUENCE);
    assert_eq!(events[FIRST_EVENT_INDEX].turn_id(), Some("turn-0"));
}

#[test]
fn lf_framing_tolerates_crlf_and_preserves_unicode_separators() {
    let input = format!(
        "{FIRST_RECORD}\r\n{SECOND_RECORD_PREFIX}{UNICODE_SEPARATOR}still-payload\n{TRUNCATED_RECORD}"
    );
    let batch = split_lf_records(input.as_str());

    assert_eq!(batch.records().len(), EXPECTED_COMPLETE_RECORDS);
    assert_eq!(batch.records()[FIRST_EVENT_INDEX], FIRST_RECORD);
    assert_eq!(
        batch.records()[SECOND_EVENT_INDEX],
        format!("{SECOND_RECORD_PREFIX}{UNICODE_SEPARATOR}still-payload")
    );
    assert_eq!(batch.remainder(), TRUNCATED_RECORD);
}

#[test]
fn active_turn_prompt_requires_explicit_steer_or_follow_up_semantics() {
    let ambiguous = command(HeadlessCommandKind::Prompt, None, PROMPT_PAYLOAD);
    let rejected = acknowledge_headless_command(&ambiguous, HeadlessRuntimePhase::ActiveTurn);
    let steering = command(
        HeadlessCommandKind::Prompt,
        Some(HeadlessQueueing::Steer),
        STEER_PAYLOAD,
    );
    let accepted = acknowledge_headless_command(&steering, HeadlessRuntimePhase::ActiveTurn);

    assert_eq!(rejected.status(), HeadlessResponseStatus::Rejected);
    assert_eq!(
        rejected.error().expect("rejection has error").kind(),
        HeadlessProtocolErrorKind::InvalidState
    );
    assert_eq!(accepted.status(), HeadlessResponseStatus::Accepted);
}

#[test]
fn queue_mutation_events_project_safe_frontend_metadata() {
    // r[verify agent_harness_runtime.coding_session_queue_cas.verification]
    let events =
        map_runtime_events_to_headless(&[crate::RuntimeEvent::SessionQueueMutationObserved {
            mutation: crate::coding_session_host::CodingSessionQueueMutationEvent {
                operation: crate::coding_session_host::CodingSessionQueueMutationOperation::Edit,
                status: "conflict",
                item_id: String::from(QUEUE_ITEM_ID),
                entry_id: String::from(QUEUE_ENTRY_ID),
                position: Some(0),
                generation: crate::coding_session_host::CodingSessionQueueGeneration::new(1),
                queue_identity: String::from(QUEUE_IDENTITY),
                conflict: Some(
                    crate::coding_session_host::CodingSessionQueueConflictKind::StaleText,
                ),
                redaction_marker: "private_queue_text_omitted",
            },
        }])
        .expect("queue mutation event should project");

    assert_eq!(events.len(), 1);
    assert!(
        events[FIRST_EVENT_INDEX]
            .summary()
            .contains("edit conflict generation=1")
    );
    assert!(
        !events[FIRST_EVENT_INDEX]
            .summary()
            .contains(PRIVATE_QUEUE_TEXT)
    );
}

#[test]
fn runtime_tool_events_carry_shared_safe_tool_correlation() {
    let request = tool_request();
    let events = map_runtime_events_to_headless(&[
        crate::RuntimeEvent::ToolRequested {
            request: request.clone(),
        },
        crate::RuntimeEvent::ToolDenied {
            request,
            reason: String::from(TOOL_DENIAL),
        },
    ])
    .expect("tool events should project");

    assert_eq!(
        events[FIRST_EVENT_INDEX].tool_call_id(),
        Some("tool:search")
    );
    assert_eq!(
        events[FIRST_EVENT_INDEX].tool_call_id(),
        events[SECOND_EVENT_INDEX].tool_call_id()
    );
    assert!(!events[FIRST_EVENT_INDEX].summary().contains(TOOL_INPUT));
    assert_eq!(events[SECOND_EVENT_INDEX].summary(), TOOL_DENIAL);
}

fn tool_request() -> animus_adapters::ToolRequest {
    let fields = [animus_adapters::tool::ArgumentFieldDraft {
        name: Some(TOOL_ARGUMENT_NAME),
        value: Some(TOOL_INPUT),
    }];
    let arguments = animus_adapters::tool::PreparedArguments::prepare(
        &animus_adapters::tool::ArgumentPreparationRequest {
            current_schema_identity: TOOL_SCHEMA,
            allowed_field_names: &[TOOL_ARGUMENT_NAME],
            compatibility_rules: &[],
            fields: &fields,
        },
    )
    .expect("arguments should validate");
    assert_eq!(arguments.schema_identity(), TOOL_SCHEMA);
    assert_eq!(arguments.fields().len(), TOOL_ARGUMENT_COUNT);
    let request = animus_adapters::ToolRequest::from_draft(&animus_adapters::tool::RequestDraft {
        name: Some(TOOL_NAME),
        arguments: Some(&arguments),
        correlation_id: Some(TOOL_CALL_ID),
        output_policy: animus_adapters::tool::OutputPolicy {
            visible_char_budget: TOOL_VISIBLE_BUDGET,
        },
        mutation_targets: &[TOOL_MUTATION_TARGET],
        required_capability_labels: &[TOOL_CAPABILITY],
        approval_posture: animus_adapters::tool::ApprovalPosture::PreApproved.as_str(),
    })
    .expect("tool request should validate");
    assert_eq!(request.name(), TOOL_NAME);
    assert_eq!(request.correlation_id(), TOOL_CALL_ID);
    request
}

#[test]
fn malformed_and_unsafe_records_return_bounded_errors_without_echoing_payloads() {
    let malformed = parse_headless_command_record(MALFORMED_RECORD)
        .expect_err("malformed record should be rejected");
    let unsafe_payload =
        parse_headless_command_record(UNSAFE_RECORD).expect_err("unsafe record should be rejected");

    assert_eq!(malformed.status(), HeadlessResponseStatus::Rejected);
    assert_eq!(
        malformed.error().expect("malformed has error").kind(),
        HeadlessProtocolErrorKind::Parse
    );
    assert_eq!(
        malformed.error().expect("malformed has error").message(),
        "malformed_record"
    );
    assert_eq!(unsafe_payload.status(), HeadlessResponseStatus::Rejected);
    assert_eq!(
        unsafe_payload.error().expect("unsafe has error").message(),
        "malformed_record"
    );
    assert!(
        !unsafe_payload
            .error()
            .expect("unsafe has error")
            .message()
            .contains("secret")
    );
}

fn command(
    kind: HeadlessCommandKind,
    queueing: Option<HeadlessQueueing>,
    payload: &str,
) -> HeadlessCommand {
    HeadlessCommand::validate(&HeadlessCommandDraft {
        request_id: REQUEST_ID,
        kind: kind.as_str(),
        queueing: queueing.map(HeadlessQueueing::as_str),
        payload,
    })
    .expect("command should validate")
}

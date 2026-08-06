use super::super::*;
use super::support::*;

#[test]
fn stream_projection_metadata_is_stable() {
    let stream = valid_stream();
    let model_event = &stream[MODEL_EVENT_INDEX];
    let jsonl = project_event(model_event, EventProjection::Jsonl.as_str()).expect("jsonl projection should work");
    let preserves =
        project_event(model_event, EventProjection::Preserves.as_str()).expect("preserves projection should work");

    assert_eq!(stream.len(), EXPECTED_STREAM_LEN);
    assert_eq!(stream[FIRST_EVENT_INDEX].sequence().value(), THREAD_SEQUENCE);
    assert_eq!(model_event.redaction(), EventRedaction::Visible);
    assert_eq!(model_event.body_hash(), hash_text(EVENT_BODY));
    assert_eq!(model_event.jsonl_line(), jsonl);
    assert!(jsonl.contains(EVENT_BODY));
    assert_eq!(model_event.preserves_tag(), AutomationEventKind::ModelDispatch.as_str());
    assert!(preserves.contains(PRESERVES_EVENT_TAG));
}

#[test]
fn stream_kinds_cover_boundaries() {
    let stream = valid_stream();

    assert_eq!(stream[STREAM_START_EVENT_INDEX].kind(), AutomationEventKind::ModelStreamStart);
    assert_eq!(stream[STREAM_END_EVENT_INDEX].preserves_tag(), AutomationEventKind::ModelStreamEnd.as_str());
    assert_eq!(stream[BACKGROUND_EVENT_INDEX].kind(), AutomationEventKind::BackgroundWork);
    assert_eq!(stream[VALIDATION_EVENT_INDEX].kind(), AutomationEventKind::Validation);
    assert_eq!(stream[RECEIPT_EVENT_INDEX].kind(), AutomationEventKind::Receipt);
    assert_eq!(stream[FINAL_RESPONSE_EVENT_INDEX].kind(), AutomationEventKind::FinalResponse);
}

#[test]
fn invalid_stream_and_projection_fail_closed() {
    let body = event_body_over_limit();
    let reordered_error = build_automation_stream(&[
        visible_event(MODEL_SEQUENCE, AutomationEventKind::ModelDispatch, EVENT_BODY),
        visible_event(REORDERED_SEQUENCE, AutomationEventKind::ToolDispatch, "tool dispatched"),
    ])
    .expect_err("out-of-order event should fail");
    let body_too_large_error =
        AutomationEnvelope::validate(&visible_event(THREAD_SEQUENCE, AutomationEventKind::Usage, body.as_str()))
            .expect_err("oversized event body should fail");
    let stream = valid_stream();
    let projection_error = project_event(&stream[MODEL_EVENT_INDEX], UNSUPPORTED_PROJECTION)
        .expect_err("unsupported projection should fail safely");

    assert_eq!(reordered_error, AutomationError::OutOfOrderEvent {
        previous: MODEL_SEQUENCE,
        next: REORDERED_SEQUENCE,
    });
    assert_eq!(body_too_large_error, AutomationError::EventBodyTooLarge {
        max_bytes: DEFAULT_MAX_EVENT_BODY_BYTES,
        actual_bytes: OVERSIZED_EVENT_BODY_BYTES,
    });
    assert_eq!(projection_error, AutomationError::UnsupportedValue {
        field: PROJECTION_FIELD,
        value: Text::from(UNSUPPORTED_PROJECTION),
    });
}

#[test]
fn large_event_payloads_use_visible_windows_and_hashes() {
    let body = event_body_over_limit();
    let summary = summarize_event_payload(body.as_str()).expect("summary should build");

    assert!(summary.truncated());
    assert_eq!(
        summary.actual_bytes(),
        u64::try_from(OVERSIZED_EVENT_BODY_BYTES).expect("byte count should fit u64")
    );
    assert_eq!(
        summary.visible_bytes(),
        u64::try_from(DEFAULT_EVENT_VISIBLE_BYTES).expect("byte count should fit u64")
    );
    assert_eq!(summary.body_hash(), hash_text(body.as_str()));
    assert_ne!(summary.visible_body(), body.as_str());
}

#[test]
fn runtime_projects_model_stream_frames_before_completion() {
    let frames = stream_frames();
    let events = crate::project_model_stream_events(&frames).expect("stream should project");

    assert_eq!(events.len(), PROJECTED_STREAM_EVENT_COUNT);
    assert_eq!(events[FIRST_EVENT_INDEX].kind(), crate::RuntimeEventKind::ModelStreamStarted);
    let terminal = events.last().expect("projected stream should have terminal event");
    assert!(matches!(
        terminal,
        crate::RuntimeEvent::ModelStreamEnded { status, .. }
            if *status == animus_adapters::ModelStreamFrameKind::Finish
    ));
}

#[test]
fn runtime_rejects_malformed_model_stream_projection() {
    let frames = vec![content_frame(STREAM_FIRST_ORDER)];
    let error = crate::project_model_stream_events(&frames).expect_err("missing finish should fail");

    assert_eq!(error, animus_adapters::ModelStreamError::MissingTerminalFrame);
}

fn stream_frames() -> Vec<animus_adapters::ModelStreamFrame> {
    vec![
        content_frame(STREAM_FIRST_ORDER),
        thought_frame(STREAM_SECOND_ORDER),
        finish_frame(STREAM_THIRD_ORDER),
    ]
}

fn content_frame(order: u64) -> animus_adapters::ModelStreamFrame {
    animus_adapters::ModelStreamFrame::from_draft(animus_adapters::ModelStreamFrameDraft {
        call_id: STREAM_CALL_ID,
        order,
        kind: animus_adapters::ModelStreamFrameKind::ContentDelta,
        body: Some("token"),
        tool_call_id: None,
        usage: None,
        visibility: animus_adapters::ModelStreamVisibility::AssistantVisible,
    })
    .expect("content frame should validate")
}

fn thought_frame(order: u64) -> animus_adapters::ModelStreamFrame {
    animus_adapters::ModelStreamFrame::from_draft(animus_adapters::ModelStreamFrameDraft {
        call_id: STREAM_CALL_ID,
        order,
        kind: animus_adapters::ModelStreamFrameKind::ThoughtDelta,
        body: Some("redacted thought"),
        tool_call_id: None,
        usage: None,
        visibility: animus_adapters::ModelStreamVisibility::Redacted,
    })
    .expect("thought frame should validate")
}

fn finish_frame(order: u64) -> animus_adapters::ModelStreamFrame {
    animus_adapters::ModelStreamFrame::from_draft(animus_adapters::ModelStreamFrameDraft {
        call_id: STREAM_CALL_ID,
        order,
        kind: animus_adapters::ModelStreamFrameKind::Finish,
        body: None,
        tool_call_id: None,
        usage: None,
        visibility: animus_adapters::ModelStreamVisibility::Redacted,
    })
    .expect("finish frame should validate")
}

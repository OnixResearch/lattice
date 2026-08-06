const CALL_ID: &str = "call-1";
const TOOL_ID: &str = "tool-1";
const FIRST_ORDER: u64 = 1;
const SECOND_ORDER: u64 = 2;
const THIRD_ORDER: u64 = 3;
const FOURTH_ORDER: u64 = 4;
const FIFTH_ORDER: u64 = 5;
const THOUGHT_FRAME_INDEX: usize = 1;
const TOOL_FRAME_INDEX: usize = 2;
const EXPECTED_FRAME_COUNT: usize = 5;
const INPUT_TOKENS: u32 = 7;
const OUTPUT_TOKENS: u32 = 11;
const TOTAL_TOKENS: u32 = 18;

#[test]
fn frames_preserve_order_and_visible_final_text() {
    let frames = alloc::vec![
        content_frame(FIRST_ORDER, "Hel"),
        thought_frame(SECOND_ORDER, "private reasoning"),
        content_frame(THIRD_ORDER, "lo"),
        usage_frame(FOURTH_ORDER),
        finish_frame(FIFTH_ORDER),
    ];

    super::validate::frames(&frames).expect("stream should validate");
    let response = super::validate::assemble(&frames).expect("visible content should assemble");

    assert_eq!(response.message(), "Hello");
    assert_eq!(super::types::Frame::contract_identity(), super::MODEL_STREAM_FRAME_CONTRACT_IDENTITY);
    assert_eq!(frames[THOUGHT_FRAME_INDEX].visibility(), super::parts::Visibility::Redacted);
    assert_eq!(usage_frame(FOURTH_ORDER).usage().expect("usage should exist").total_tokens(), TOTAL_TOKENS);
}

#[test]
fn parser_accepts_fragmented_chunks() {
    let frames = super::fixture::parse(&[
        "content|call-1|1|Hel",
        "lo\nthought|call-1|2|private\n",
        "tool|call-1|3|tool-1|{\"name\":\"read\"}\nusage|call-1|4|7|11\nfinish|call-1|5\n",
    ])
    .expect("fragmented fixture should parse");
    let tool = &frames[TOOL_FRAME_INDEX];

    assert_eq!(frames.len(), EXPECTED_FRAME_COUNT);
    assert_eq!(tool.kind(), super::parts::Kind::ToolCallDelta);
    assert_eq!(tool.tool_call_id().expect("tool id").as_str(), TOOL_ID);
    assert_eq!(super::validate::assemble(&frames).expect("response should assemble").message(), "Hello");
}

#[test]
fn thought_frames_cannot_be_assistant_visible_by_default() {
    let error = super::types::Frame::from_draft(super::types::Draft {
        call_id: CALL_ID,
        order: FIRST_ORDER,
        kind: super::parts::Kind::ThoughtDelta,
        body: Some("private reasoning"),
        tool_call_id: None,
        usage: None,
        visibility: super::parts::Visibility::AssistantVisible,
    })
    .expect_err("thought visibility should fail closed");

    assert_eq!(error, super::failure::Error::ThoughtVisibilityLeak);
}

#[test]
fn fixture_without_terminal_frame_fails_closed() {
    let error = super::fixture::parse(&["content|call-1|1|partial\n"]).expect_err("missing finish should fail");

    assert_eq!(error, super::failure::Error::MissingTerminalFrame);
}

#[test]
fn fixture_rejects_non_increasing_order() {
    let error = super::fixture::parse(&["content|call-1|1|first\ncontent|call-1|1|again\nfinish|call-1|2\n"])
        .expect_err("non-increasing order should fail");

    assert!(matches!(error, super::failure::Error::OutOfOrderFrame { .. }));
}

#[test]
fn fixture_rejects_truncated_chunks() {
    let error = super::fixture::parse(&["content|call-1|1|partial"]).expect_err("truncated chunk should fail");

    assert_eq!(error, super::failure::Error::MalformedFixture {
        reason: super::MALFORMED_TRUNCATED_RECORD,
    });
}

#[test]
fn terminal_failure_does_not_assemble_success() {
    let frames = alloc::vec![content_frame(FIRST_ORDER, "partial"), failure_frame(SECOND_ORDER)];
    let error = super::validate::assemble(&frames).expect_err("failure is not final success");

    assert_eq!(error, super::failure::Error::TerminalFrameNotFinish {
        kind: super::parts::Kind::Failure,
    });
}

fn content_frame(order: u64, body: &str) -> super::types::Frame {
    super::types::Frame::from_draft(super::types::Draft {
        call_id: CALL_ID,
        order,
        kind: super::parts::Kind::ContentDelta,
        body: Some(body),
        tool_call_id: None,
        usage: None,
        visibility: super::parts::Visibility::AssistantVisible,
    })
    .expect("content frame should validate")
}

fn thought_frame(order: u64, body: &str) -> super::types::Frame {
    super::types::Frame::from_draft(super::types::Draft {
        call_id: CALL_ID,
        order,
        kind: super::parts::Kind::ThoughtDelta,
        body: Some(body),
        tool_call_id: None,
        usage: None,
        visibility: super::parts::Visibility::Redacted,
    })
    .expect("thought frame should validate")
}

fn usage_frame(order: u64) -> super::types::Frame {
    super::types::Frame::from_draft(super::types::Draft {
        call_id: CALL_ID,
        order,
        kind: super::parts::Kind::Usage,
        body: None,
        tool_call_id: None,
        usage: Some(super::parts::Usage::new(INPUT_TOKENS, OUTPUT_TOKENS)),
        visibility: super::parts::Visibility::Redacted,
    })
    .expect("usage frame should validate")
}

fn finish_frame(order: u64) -> super::types::Frame {
    super::types::Frame::from_draft(super::types::Draft {
        call_id: CALL_ID,
        order,
        kind: super::parts::Kind::Finish,
        body: None,
        tool_call_id: None,
        usage: None,
        visibility: super::parts::Visibility::Redacted,
    })
    .expect("finish frame should validate")
}

fn failure_frame(order: u64) -> super::types::Frame {
    super::types::Frame::from_draft(super::types::Draft {
        call_id: CALL_ID,
        order,
        kind: super::parts::Kind::Failure,
        body: Some("provider_failure"),
        tool_call_id: None,
        usage: None,
        visibility: super::parts::Visibility::Redacted,
    })
    .expect("failure frame should validate")
}

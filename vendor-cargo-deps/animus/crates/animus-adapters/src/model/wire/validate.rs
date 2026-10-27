pub(super) fn frame_from_draft(
    draft: super::types::Draft<'_>,
) -> Result<super::types::Frame, super::failure::Error> {
    let call_id = super::parts::CallId::new(draft.call_id)?;
    let order = super::parts::Order::new(draft.order)?;
    let body = body(draft.kind, draft.body)?;
    let tool_call_id = tool_call_id(draft.kind, draft.tool_call_id)?;
    let usage = usage(draft.kind, draft.usage)?;
    visibility(draft.kind, draft.visibility)?;
    Ok(super::types::Frame {
        call_id,
        order,
        kind: draft.kind,
        body,
        tool_call_id,
        usage,
        visibility: draft.visibility,
    })
}

fn body(
    kind: super::parts::Kind,
    value: Option<&str>,
) -> Result<Option<alloc::string::String>, super::failure::Error> {
    match (kind.requires_body(), value) {
        (true, Some(raw)) => {
            let field = super::types::body_field_for(kind);
            Ok(Some(crate::validation::normalized_adapter_field(
                field, raw,
            )?))
        }
        (true, None) => Err(super::failure::Error::MissingBody { kind }),
        (false, Some(_raw)) => Err(super::failure::Error::UnexpectedBody { kind }),
        (false, None) => Ok(None),
    }
}

fn tool_call_id(
    kind: super::parts::Kind,
    value: Option<&str>,
) -> Result<Option<super::parts::ToolCallId>, super::failure::Error> {
    match (kind.requires_tool_call_id(), value) {
        (true, Some(raw)) => Ok(Some(super::parts::ToolCallId::new(raw)?)),
        (true, None) => Err(super::failure::Error::MissingToolCallId),
        (false, Some(_raw)) => Err(super::failure::Error::UnexpectedToolCallId { kind }),
        (false, None) => Ok(None),
    }
}

const fn usage(
    kind: super::parts::Kind,
    value: Option<super::parts::Usage>,
) -> Result<Option<super::parts::Usage>, super::failure::Error> {
    match (kind.allows_usage(), value) {
        (true, Some(raw)) => Ok(Some(raw)),
        (true, None) => Err(super::failure::Error::MissingUsage),
        (false, Some(_raw)) => Err(super::failure::Error::UnexpectedUsage { kind }),
        (false, None) => Ok(None),
    }
}

const fn visibility(
    kind: super::parts::Kind,
    value: super::parts::Visibility,
) -> Result<(), super::failure::Error> {
    match (kind, value) {
        (super::parts::Kind::ThoughtDelta, super::parts::Visibility::AssistantVisible) => {
            Err(super::failure::Error::ThoughtVisibilityLeak)
        }
        _allowed => Ok(()),
    }
}

/// Validates stream ordering, model-call correlation, and terminal status.
///
/// # Errors
///
/// Returns [`super::failure::Error`] when the stream is empty, mixes call IDs,
/// loses strict order, places frames after a terminal event, or omits a terminal event.
pub(crate) fn frames(frames: &[super::types::Frame]) -> Result<(), super::failure::Error> {
    assert!(
        frames.len() <= crate::validation::MAX_ADAPTER_FIELD_CHARS,
        "stream frame count fits adapter diagnostic bounds"
    );
    let Some(first) = frames.first() else {
        return Err(super::failure::Error::EmptyStream);
    };
    sequence(frames, first.call_id().as_str())
}

fn sequence(
    frames: &[super::types::Frame],
    expected_call_id: &str,
) -> Result<(), super::failure::Error> {
    assert!(
        !expected_call_id.is_empty(),
        "validated first frame carries a call id"
    );
    assert!(!frames.is_empty(), "sequence receives at least one frame");
    let mut previous_order = None;
    let mut terminal_kind = None;
    for frame in frames {
        check_call(frame, expected_call_id)?;
        check_terminal_position(terminal_kind)?;
        check_order(frame, previous_order)?;
        previous_order = Some(frame.order().value());
        if frame.kind().is_terminal() {
            terminal_kind = Some(frame.kind());
        }
    }
    terminal_kind.map_or(Err(super::failure::Error::MissingTerminalFrame), |_kind| {
        Ok(())
    })
}

fn check_call(
    frame: &super::types::Frame,
    expected_call_id: &str,
) -> Result<(), super::failure::Error> {
    assert!(
        !frame.call_id().as_str().is_empty(),
        "validated stream frame carries a call id"
    );
    if frame.call_id().as_str() == expected_call_id {
        return Ok(());
    }
    Err(super::failure::Error::MixedModelCall {
        expected: alloc::string::String::from(expected_call_id),
        actual: alloc::string::String::from(frame.call_id().as_str()),
    })
}

const fn check_terminal_position(
    kind: Option<super::parts::Kind>,
) -> Result<(), super::failure::Error> {
    if let Some(kind) = kind {
        return Err(super::failure::Error::TerminalFrameNotLast { kind });
    }
    Ok(())
}

fn check_order(
    frame: &super::types::Frame,
    previous_order: Option<u64>,
) -> Result<(), super::failure::Error> {
    let Some(previous) = previous_order else {
        return Ok(());
    };
    let next = frame.order().value();
    if previous < next {
        return Ok(());
    }
    Err(super::failure::Error::OutOfOrderFrame { previous, next })
}

/// Assembles the default assistant-visible final response from visible content frames only.
///
/// # Errors
///
/// Returns [`super::failure::Error`] when the stream is invalid, did not finish
/// successfully, or visible content fails response validation.
pub(crate) fn assemble(
    items: &[super::types::Frame],
) -> Result<super::super::Response, super::failure::Error> {
    assert!(
        items.len() <= crate::validation::MAX_ADAPTER_FIELD_CHARS,
        "visible assembly frame count fits adapter diagnostic bounds"
    );
    frames(items)?;
    finish_frame(items.last().ok_or(super::failure::Error::EmptyStream)?)?;
    super::super::Response::new(&visible_message(items)).map_err(super::failure::Error::from)
}

fn finish_frame(frame: &super::types::Frame) -> Result<(), super::failure::Error> {
    if frame.kind() == super::parts::Kind::Finish {
        return Ok(());
    }
    Err(super::failure::Error::TerminalFrameNotFinish { kind: frame.kind() })
}

fn visible_message(frames: &[super::types::Frame]) -> alloc::string::String {
    let mut message = alloc::string::String::new();
    for frame in frames {
        if frame.is_default_final_visible() {
            if let Some(body) = frame.body() {
                message.push_str(body);
            }
        }
    }
    message
}

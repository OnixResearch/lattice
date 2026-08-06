/// Parses raw provider-stream fixture chunks into canonical stream frames.
///
/// The fixture grammar is line-oriented and intentionally small so provider
/// adapters can feed raw SSE or byte chunks through their production parser and
/// compare the canonical frame output here: `content|call|order|text`,
/// `thought|call|order|text`, `tool|call|order|tool_id|text`,
/// `usage|call|order|input_tokens|output_tokens`, `finish|call|order`,
/// `cancelled|call|order|reason`, and `failure|call|order|reason`.
///
/// # Errors
///
/// Returns [`super::failure::Error`] when chunks are empty, truncated,
/// malformed, or yield an invalid stream.
pub(crate) fn parse(chunks: &[&str]) -> Result<alloc::vec::Vec<super::types::Frame>, super::failure::Error> {
    check_chunks(chunks)?;
    let buffer = buffer(chunks);
    check_complete(buffer.as_str())?;
    let frames = records(buffer.as_str())?;
    super::validate::frames(&frames)?;
    Ok(frames)
}

fn check_chunks(chunks: &[&str]) -> Result<(), super::failure::Error> {
    assert!(
        chunks.len() <= crate::validation::MAX_ADAPTER_FIELD_CHARS,
        "fixture chunk count fits adapter diagnostic bounds"
    );
    if chunks.is_empty() {
        return Err(super::failure::Error::MalformedFixture {
            reason: super::MALFORMED_EMPTY_CHUNKS,
        });
    }
    Ok(())
}

fn buffer(chunks: &[&str]) -> alloc::string::String {
    assert!(!chunks.is_empty(), "non-empty chunks are checked by caller");
    let mut buffer = alloc::string::String::new();
    for chunk in chunks {
        buffer.push_str(chunk);
    }
    buffer
}

fn check_complete(buffer: &str) -> Result<(), super::failure::Error> {
    if buffer.ends_with(super::RECORD_SEPARATOR) {
        return Ok(());
    }
    Err(super::failure::Error::MalformedFixture {
        reason: super::MALFORMED_TRUNCATED_RECORD,
    })
}

fn records(buffer: &str) -> Result<alloc::vec::Vec<super::types::Frame>, super::failure::Error> {
    assert!(buffer.ends_with(super::RECORD_SEPARATOR), "fixture buffer has a record separator");
    let mut frames = alloc::vec::Vec::with_capacity(record_count(buffer));
    for record in buffer.split(super::RECORD_SEPARATOR) {
        if !record.is_empty() {
            frames.push(record_fields(record)?);
        }
    }
    Ok(frames)
}

fn record_count(buffer: &str) -> usize {
    buffer.split(super::RECORD_SEPARATOR).filter(|record| !record.is_empty()).count()
}

fn record_fields(record: &str) -> Result<super::types::Frame, super::failure::Error> {
    assert!(!record.is_empty(), "fixture record is non-empty");
    let fields: alloc::vec::Vec<&str> = record.split(super::FIELD_SEPARATOR).collect();
    assert!(!fields.is_empty(), "split fixture record has a kind field");
    let Some(kind) = fields.get(super::KIND_INDEX) else {
        return Err(super::failure::Error::MalformedFixture {
            reason: super::MALFORMED_FIELD_COUNT,
        });
    };

    match *kind {
        super::KIND_CONTENT => text(&fields, super::parts::Kind::ContentDelta),
        super::KIND_THOUGHT => text(&fields, super::parts::Kind::ThoughtDelta),
        super::KIND_TOOL => tool(&fields),
        super::KIND_USAGE => usage(&fields),
        super::KIND_FINISH => terminal(&fields, super::parts::Kind::Finish),
        super::KIND_CANCELLED => reason(&fields, super::parts::Kind::Cancelled),
        super::KIND_FAILURE => reason(&fields, super::parts::Kind::Failure),
        _unknown => Err(super::failure::Error::MalformedFixture {
            reason: super::MALFORMED_RECORD_KIND,
        }),
    }
}

fn text(fields: &[&str], kind: super::parts::Kind) -> Result<super::types::Frame, super::failure::Error> {
    if fields.len() != super::STANDARD_FIELD_COUNT {
        return Err(super::failure::Error::MalformedFixture {
            reason: super::MALFORMED_FIELD_COUNT,
        });
    }
    super::types::Frame::from_draft(super::types::Draft {
        call_id: fields[super::CALL_INDEX],
        order: order(fields[super::ORDER_INDEX])?,
        kind,
        body: Some(fields[super::BODY_INDEX]),
        tool_call_id: None,
        usage: None,
        visibility: text_visibility(kind),
    })
}

const fn text_visibility(kind: super::parts::Kind) -> super::parts::Visibility {
    match kind {
        super::parts::Kind::ContentDelta => super::parts::Visibility::AssistantVisible,
        super::parts::Kind::ThoughtDelta
        | super::parts::Kind::ToolCallDelta
        | super::parts::Kind::Usage
        | super::parts::Kind::Finish
        | super::parts::Kind::Cancelled
        | super::parts::Kind::Failure => super::parts::Visibility::Redacted,
    }
}

fn tool(fields: &[&str]) -> Result<super::types::Frame, super::failure::Error> {
    if fields.len() != super::TOOL_FIELD_COUNT {
        return Err(super::failure::Error::MalformedFixture {
            reason: super::MALFORMED_FIELD_COUNT,
        });
    }
    super::types::Frame::from_draft(super::types::Draft {
        call_id: fields[super::CALL_INDEX],
        order: order(fields[super::ORDER_INDEX])?,
        kind: super::parts::Kind::ToolCallDelta,
        body: Some(fields[super::TOOL_BODY_INDEX]),
        tool_call_id: Some(fields[super::TOOL_ID_INDEX]),
        usage: None,
        visibility: super::parts::Visibility::Redacted,
    })
}

fn usage(fields: &[&str]) -> Result<super::types::Frame, super::failure::Error> {
    if fields.len() != super::USAGE_FIELD_COUNT {
        return Err(super::failure::Error::MalformedFixture {
            reason: super::MALFORMED_FIELD_COUNT,
        });
    }
    super::types::Frame::from_draft(super::types::Draft {
        call_id: fields[super::CALL_INDEX],
        order: order(fields[super::ORDER_INDEX])?,
        kind: super::parts::Kind::Usage,
        body: None,
        tool_call_id: None,
        usage: Some(super::parts::Usage::new(
            usage_count(fields[super::USAGE_INPUT_INDEX])?,
            usage_count(fields[super::USAGE_OUTPUT_INDEX])?,
        )),
        visibility: super::parts::Visibility::Redacted,
    })
}

fn terminal(fields: &[&str], kind: super::parts::Kind) -> Result<super::types::Frame, super::failure::Error> {
    if fields.len() != super::TERMINAL_FIELD_COUNT {
        return Err(super::failure::Error::MalformedFixture {
            reason: super::MALFORMED_FIELD_COUNT,
        });
    }
    super::types::Frame::from_draft(super::types::Draft {
        call_id: fields[super::CALL_INDEX],
        order: order(fields[super::ORDER_INDEX])?,
        kind,
        body: None,
        tool_call_id: None,
        usage: None,
        visibility: super::parts::Visibility::Redacted,
    })
}

fn reason(fields: &[&str], kind: super::parts::Kind) -> Result<super::types::Frame, super::failure::Error> {
    if fields.len() != super::TERMINAL_REASON_FIELD_COUNT {
        return Err(super::failure::Error::MalformedFixture {
            reason: super::MALFORMED_FIELD_COUNT,
        });
    }
    super::types::Frame::from_draft(super::types::Draft {
        call_id: fields[super::CALL_INDEX],
        order: order(fields[super::ORDER_INDEX])?,
        kind,
        body: Some(fields[super::BODY_INDEX]),
        tool_call_id: None,
        usage: None,
        visibility: super::parts::Visibility::Redacted,
    })
}

fn order(raw: &str) -> Result<u64, super::failure::Error> {
    raw.parse::<u64>().map_err(|_error| super::failure::Error::MalformedFixture {
        reason: super::MALFORMED_ORDER,
    })
}

fn usage_count(raw: &str) -> Result<u32, super::failure::Error> {
    raw.parse::<u32>().map_err(|_error| super::failure::Error::MalformedFixture {
        reason: super::MALFORMED_USAGE,
    })
}

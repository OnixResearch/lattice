/// Projects runtime events into headless protocol events in supplied order.
///
/// # Errors
///
/// Returns [`HeadlessProtocolError`] when an event contains unsafe metadata or
/// when the event sequence overflows.
pub fn map_runtime_events_to_headless(
    events: &[crate::RuntimeEvent],
) -> Result<Collection<HeadlessProtocolEvent>, HeadlessProtocolError> {
    let mut projected = Collection::with_capacity(events.len());
    for (index, event) in events.iter().enumerate() {
        projected.push(HeadlessProtocolEvent::from_runtime_event(
            index_to_sequence(index)?,
            event,
        )?);
    }
    Ok(projected)
}

fn index_to_sequence(index: usize) -> Result<u64, HeadlessProtocolError> {
    let index_value = u64::try_from(index).map_err(|_overflow| {
        HeadlessProtocolError::new(
            None,
            HeadlessProtocolErrorKind::RuntimeFailure,
            EVENT_SEQUENCE_FIELD,
        )
    })?;
    index_value
        .checked_add(super::MIN_EVENT_SEQUENCE)
        .ok_or_else(|| {
            HeadlessProtocolError::new(
                None,
                HeadlessProtocolErrorKind::RuntimeFailure,
                EVENT_SEQUENCE_FIELD,
            )
        })
}

fn rejected_from_error(error: HeadlessProtocolError) -> HeadlessResponse {
    let request_id = error.request_id().map(Text::from);
    HeadlessResponse {
        request_id,
        status: HeadlessResponseStatus::Rejected,
        error: Some(error),
    }
}

fn malformed_record_response(request_id: Option<&str>) -> HeadlessResponse {
    HeadlessResponse::rejected(
        request_id,
        HeadlessProtocolError::new(
            request_id,
            HeadlessProtocolErrorKind::Parse,
            MALFORMED_RECORD_SUMMARY,
        ),
    )
}

fn safe_text(field: &'static str, value: &str) -> Result<Text, HeadlessProtocolError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(HeadlessProtocolError {
            request_id: None,
            kind: HeadlessProtocolErrorKind::InvalidCommand,
            message: Text::from(field),
        });
    }
    let lower = trimmed.to_ascii_lowercase();
    if contains_leak_marker(lower.as_str()) {
        return Err(HeadlessProtocolError {
            request_id: None,
            kind: HeadlessProtocolErrorKind::Parse,
            message: Text::from(MALFORMED_RECORD_SUMMARY),
        });
    }
    Ok(Text::from(trimmed))
}

fn contains_leak_marker(value: &str) -> bool {
    value.contains(LEAK_RAW_PROMPT)
        || value.contains(LEAK_PROVIDER_PAYLOAD)
        || value.contains(LEAK_CREDENTIAL)
        || value.contains(LEAK_AUTHORIZATION)
        || value.contains(LEAK_FULL_PATH)
        || value.contains(LEAK_UNBOUNDED_OUTPUT)
}

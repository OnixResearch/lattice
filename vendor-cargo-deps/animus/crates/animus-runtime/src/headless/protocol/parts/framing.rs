/// Validates command queueing and returns the acknowledgement response.
#[must_use]
pub fn acknowledge_headless_command(
    command: &HeadlessCommand,
    phase: HeadlessRuntimePhase,
) -> HeadlessResponse {
    if command.kind().is_prompt_like() && phase == HeadlessRuntimePhase::ActiveTurn {
        let queueing = command.queueing();
        if !matches!(
            queueing,
            Some(HeadlessQueueing::Steer | HeadlessQueueing::FollowUp)
        ) {
            return HeadlessResponse::rejected(
                Some(command.request_id()),
                HeadlessProtocolError::new(
                    Some(command.request_id()),
                    HeadlessProtocolErrorKind::InvalidState,
                    AMBIGUOUS_ACTIVE_PROMPT_SUMMARY,
                ),
            );
        }
    }
    HeadlessResponse::accepted(command.request_id())
}

/// Parses one pipe-shaped command record for conformance fixtures.
///
/// # Errors
///
/// Returns a rejected [`HeadlessResponse`] when framing produced a malformed or
/// unsafe command record. The response never echoes the malformed payload.
pub fn parse_headless_command_record(record: &str) -> Result<HeadlessCommand, HeadlessResponse> {
    let parts: Collection<&str> = record.split(FIELD_SEPARATOR).collect();
    if parts.len() != RECORD_FIELD_COUNT {
        return Err(malformed_record_response(None));
    }
    let queueing = if parts[2].is_empty() {
        None
    } else {
        Some(parts[2])
    };
    HeadlessCommand::validate(&HeadlessCommandDraft {
        request_id: parts[0],
        kind: parts[1],
        queueing,
        payload: parts[3],
    })
    .map_err(rejected_from_error)
}

/// Splits strict LF-delimited protocol records from a buffer.
#[must_use]
pub fn split_lf_records(buffer: &str) -> HeadlessRecordBatch {
    let mut records = Collection::new();
    let mut start = 0;
    for (index, character) in buffer.char_indices() {
        if character == LF_DELIMITER {
            let raw = &buffer[start..index];
            records.push(Text::from(raw.strip_suffix(CR_TRAILER).unwrap_or(raw)));
            start = index + character.len_utf8();
        }
    }
    HeadlessRecordBatch {
        records,
        remainder: Text::from(&buffer[start..]),
    }
}

/// Records split from a headless protocol buffer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HeadlessRecordBatch {
    records: Collection<Text>,
    remainder: Text,
}

impl HeadlessRecordBatch {
    /// Returns complete LF-delimited records.
    #[must_use]
    pub fn records(&self) -> &[Text] {
        self.records.as_slice()
    }

    /// Returns the trailing incomplete record data.
    #[must_use]
    pub fn remainder(&self) -> &str {
        self.remainder.as_str()
    }
}

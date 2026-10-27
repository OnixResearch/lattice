/// Bounded summary for an event payload that may be too large to embed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventPayloadSummary {
    visible_body: super::super::Text,
    actual_bytes: u64,
    visible_bytes: u64,
    body_hash: super::super::Text,
    is_truncated: bool,
}

impl EventPayloadSummary {
    /// Returns the bounded visible body prefix.
    #[must_use]
    pub fn visible_body(&self) -> &str {
        self.visible_body.as_str()
    }

    /// Returns the original payload byte length.
    #[must_use]
    pub const fn actual_bytes(&self) -> u64 {
        self.actual_bytes
    }

    /// Returns the visible payload byte length.
    #[must_use]
    pub const fn visible_bytes(&self) -> u64 {
        self.visible_bytes
    }

    /// Returns the BLAKE3 hash of the original payload.
    #[must_use]
    pub fn body_hash(&self) -> &str {
        self.body_hash.as_str()
    }

    /// Returns true when the original payload exceeded the visible budget.
    #[must_use]
    pub const fn truncated(&self) -> bool {
        self.is_truncated
    }
}

/// Builds a bounded safe summary for a potentially large event payload.
///
/// # Errors
///
/// Returns [`super::super::AutomationError`] when the payload is empty.
///
/// # Panics
///
/// Panics if the computed visible payload length exceeds the original body
/// length, which would indicate invalid UTF-8 boundary truncation.
pub fn summarize_event_payload(
    value: &str,
) -> Result<EventPayloadSummary, super::super::AutomationError> {
    let body = super::super::required_text(super::super::TextField {
        field: super::super::EVENT_BODY_FIELD,
        value,
    })?;
    let visible_body = super::super::Text::from(super::super::prefix_at_utf8_boundary(
        body.as_str(),
        super::super::DEFAULT_EVENT_VISIBLE_BYTES,
    ));
    let actual_bytes = body.len() as u64;
    let visible_bytes = visible_body.len() as u64;
    let is_truncated = actual_bytes > visible_bytes;
    let body_hash = super::super::hash_text(body.as_str());
    assert!(actual_bytes >= visible_bytes);
    assert_eq!(is_truncated, actual_bytes > visible_bytes);
    Ok(EventPayloadSummary {
        visible_body,
        actual_bytes,
        visible_bytes,
        body_hash,
        is_truncated,
    })
}

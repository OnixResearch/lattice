mod body;
mod order;
mod projection;
mod taxonomy;

pub use body::{EventPayloadSummary, summarize_event_payload};
pub use order::EventSequence;
pub use projection::{EventProjection, project_event};
pub use taxonomy::{AutomationEventKind, EventRedaction};

/// Borrowed automation event fields before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AutomationEventDraft<'a> {
    /// Monotonic runtime-order sequence.
    pub sequence: u64,
    /// Event kind label.
    pub kind: &'a str,
    /// Safe redaction marker for the event body.
    pub redaction: &'a str,
    /// Bounded machine-readable body.
    pub body: &'a str,
}

/// Versioned append-only automation event envelope.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AutomationEnvelope {
    version: u16,
    sequence: EventSequence,
    kind: AutomationEventKind,
    redaction: EventRedaction,
    body_hash: super::Text,
    body: super::Text,
}

impl AutomationEnvelope {
    /// Validates one automation event envelope.
    ///
    /// # Errors
    ///
    /// Returns [`super::AutomationError`] when the sequence is zero, kind or
    /// redaction marker is unsupported, or the event body is empty or exceeds
    /// the event budget.
    pub fn validate(draft: &AutomationEventDraft<'_>) -> Result<Self, super::AutomationError> {
        let sequence = EventSequence::new(draft.sequence)?;
        let kind = AutomationEventKind::parse(draft.kind).ok_or_else(|| {
            super::unsupported(super::TextField {
                field: super::EVENT_KIND_FIELD,
                value: draft.kind,
            })
        })?;
        let redaction = EventRedaction::parse(draft.redaction).ok_or_else(|| {
            super::unsupported(super::TextField {
                field: super::EVENT_REDACTION_FIELD,
                value: draft.redaction,
            })
        })?;
        let body = super::bounded_body(draft.body, super::DEFAULT_MAX_EVENT_BODY_BYTES)?;
        let body_hash = super::hash_text(body.as_str());
        Ok(Self {
            version: super::AUTOMATION_EVENT_VERSION,
            sequence,
            kind,
            redaction,
            body_hash,
            body,
        })
    }

    /// Returns the event schema version.
    #[must_use]
    pub const fn version(&self) -> u16 {
        self.version
    }

    /// Returns the monotonic event sequence.
    #[must_use]
    pub const fn sequence(&self) -> EventSequence {
        self.sequence
    }

    /// Returns the event kind.
    #[must_use]
    pub const fn kind(&self) -> AutomationEventKind {
        self.kind
    }

    /// Returns the event redaction marker.
    #[must_use]
    pub const fn redaction(&self) -> EventRedaction {
        self.redaction
    }

    /// Returns the BLAKE3 hash of the bounded event body.
    #[must_use]
    pub fn body_hash(&self) -> &str {
        self.body_hash.as_str()
    }

    /// Returns the bounded event body.
    #[must_use]
    pub fn body(&self) -> &str {
        self.body.as_str()
    }

    /// Renders a stable JSONL record for shell consumers.
    #[must_use]
    pub fn jsonl_line(&self) -> super::Text {
        let mut line = super::Text::new();
        line.push('{');
        line.push_str("\"version\":");
        line.push_str(&self.version.to_string());
        line.push_str(",\"sequence\":");
        line.push_str(&self.sequence.value().to_string());
        line.push_str(",\"kind\":\"");
        line.push_str(self.kind.as_str());
        line.push_str("\",\"redaction\":\"");
        line.push_str(self.redaction.as_str());
        line.push_str("\",\"body_hash\":\"");
        line.push_str(self.body_hash.as_str());
        line.push_str("\",\"body\":\"");
        super::push_json_escaped(&mut line, self.body.as_str());
        line.push_str("\"}");
        line
    }

    /// Returns a stable Preserves-compatible tag label.
    #[must_use]
    pub const fn preserves_tag(&self) -> &'static str {
        self.kind.as_str()
    }

    /// Renders a stable Preserves-compatible metadata projection.
    #[must_use]
    pub fn preserves_record(&self) -> super::Text {
        let mut record = super::Text::new();
        record.push('(');
        record.push_str(super::PRESERVES_EVENT_TAG);
        record.push(' ');
        record.push_str(self.kind.as_str());
        record.push(' ');
        record.push_str(&self.sequence.value().to_string());
        record.push(' ');
        record.push_str(self.redaction.as_str());
        record.push(' ');
        record.push_str(self.body_hash.as_str());
        record.push(')');
        record
    }
}

/// Builds an ordered append-only automation stream from runtime-order drafts.
///
/// # Errors
///
/// Returns [`super::AutomationError`] when an event is invalid or sequences are
/// not strictly increasing.
pub fn build_automation_stream(
    drafts: &[AutomationEventDraft<'_>],
) -> Result<super::Collection<AutomationEnvelope>, super::AutomationError> {
    let mut stream = super::Collection::with_capacity(drafts.len());
    let mut previous_sequence: Option<EventSequence> = None;
    for draft in drafts {
        let envelope = AutomationEnvelope::validate(draft)?;
        if let Some(previous) = previous_sequence {
            if previous >= envelope.sequence() {
                return Err(super::AutomationError::OutOfOrderEvent {
                    previous: previous.value(),
                    next: envelope.sequence().value(),
                });
            }
        }
        previous_sequence = Some(envelope.sequence());
        stream.push(envelope);
    }
    Ok(stream)
}

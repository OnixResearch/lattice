/// Structured append-only history event kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EventKind {
    /// Tool call admission.
    Call,
    /// Typed tool result.
    Result,
    /// Execution span metadata.
    Span,
}

/// Borrowed event facts before append.
#[derive(Clone, Copy)]
pub struct EventDraft<'a> {
    /// Event kind.
    pub kind: EventKind,
    /// Stable call correlation identity.
    pub call_id: &'a str,
    /// Optional span identity.
    pub span_id: Option<&'a str>,
    /// Required only for result events.
    pub result: Option<&'a super::PersistedRecord>,
}

/// One admitted structured history event.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Event {
    /// Monotonic zero-based sequence.
    pub sequence: u64,
    /// Event kind.
    pub kind: EventKind,
    /// Stable call correlation identity.
    pub call_id: alloc::string::String,
    /// Optional span identity.
    pub span_id: Option<alloc::string::String>,
    /// Structured result projection.
    pub result: Option<super::PersistedRecord>,
}

/// Explicit history bound.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HistoryPolicy {
    /// Maximum retained events.
    pub max_events: u32,
}

/// Bounded append-only structured history.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct History {
    policy: HistoryPolicy,
    events: alloc::vec::Vec<Event>,
}

impl History {
    /// Builds an empty bounded history.
    ///
    /// # Errors
    ///
    /// Returns [`super::Error::InvalidHistory`] for a zero event bound.
    pub fn new(policy: HistoryPolicy) -> Result<Self, super::Error> {
        if policy.max_events == 0 {
            return Err(super::Error::InvalidHistory);
        }
        Ok(Self {
            policy,
            events: alloc::vec::Vec::new(),
        })
    }

    /// Appends one structured event without exposing mutation or replay hooks.
    ///
    /// # Errors
    ///
    /// Returns [`super::Error::InvalidHistory`] for malformed event shape or a
    /// full history.
    pub fn append(&mut self, draft: EventDraft<'_>) -> Result<&Event, super::Error> {
        validate_draft(&draft)?;
        let current_count =
            u32::try_from(self.events.len()).map_err(|_| super::Error::InvalidHistory)?;
        if current_count >= self.policy.max_events {
            return Err(super::Error::InvalidHistory);
        }
        let sequence =
            u64::try_from(self.events.len()).map_err(|_| super::Error::InvalidHistory)?;
        self.events.push(Event {
            sequence,
            kind: draft.kind,
            call_id: alloc::string::String::from(draft.call_id),
            span_id: draft.span_id.map(alloc::string::String::from),
            result: draft.result.cloned(),
        });
        self.events.last().ok_or(super::Error::InvalidHistory)
    }

    /// Returns immutable structured events in append order.
    #[must_use]
    pub fn events(&self) -> &[Event] {
        &self.events
    }
}

/// Renders history as a pure query over recorded data.
///
/// This function has no tool executor or effect port and cannot rerun a tool.
#[must_use]
pub fn render_history(history: &History) -> alloc::vec::Vec<alloc::string::String> {
    history.events().iter().map(render_event).collect()
}

fn validate_draft(draft: &EventDraft<'_>) -> Result<(), super::Error> {
    if draft.call_id.trim().is_empty() || draft.call_id.chars().any(char::is_control) {
        return Err(super::Error::InvalidHistory);
    }
    let is_shape_valid = match draft.kind {
        EventKind::Call => draft.span_id.is_none() && draft.result.is_none(),
        EventKind::Result => draft.span_id.is_none() && draft.result.is_some(),
        EventKind::Span => draft.span_id.is_some() && draft.result.is_none(),
    };
    if !is_shape_valid {
        return Err(super::Error::InvalidHistory);
    }
    Ok(())
}

fn render_event(event: &Event) -> alloc::string::String {
    use core::fmt::Write as _;
    let mut line = alloc::string::String::new();
    let kind = match event.kind {
        EventKind::Call => "call",
        EventKind::Result => "result",
        EventKind::Span => "span",
    };
    let _written = write!(line, "{}:{}:{}", event.sequence, kind, event.call_id);
    if let Some(span_id) = &event.span_id {
        let _written = write!(line, ":{span_id}");
    }
    if let Some(result) = &event.result {
        let _written = write!(line, ":{}", result.datum_identity);
    }
    line
}

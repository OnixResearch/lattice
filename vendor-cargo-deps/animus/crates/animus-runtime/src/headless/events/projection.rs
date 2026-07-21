/// Supported event projection formats.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EventProjection {
    /// JSON Lines shell projection.
    Jsonl,
    /// Preserves-compatible metadata projection.
    Preserves,
}

impl EventProjection {
    fn parse(value: &str) -> Option<Self> {
        match value {
            super::super::PROJECTION_JSONL => Some(Self::Jsonl),
            super::super::PROJECTION_PRESERVES => Some(Self::Preserves),
            _unsupported => None,
        }
    }

    /// Returns the stable projection label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Jsonl => super::super::PROJECTION_JSONL,
            Self::Preserves => super::super::PROJECTION_PRESERVES,
        }
    }
}

/// Projects one validated event into the requested boundary format.
///
/// # Errors
///
/// Returns [`super::super::AutomationError`] when the projection format is
/// unsupported.
pub fn project_event(
    envelope: &super::AutomationEnvelope,
    projection: &str,
) -> Result<super::super::Text, super::super::AutomationError> {
    match EventProjection::parse(projection) {
        Some(EventProjection::Jsonl) => Ok(envelope.jsonl_line()),
        Some(EventProjection::Preserves) => Ok(envelope.preserves_record()),
        None => Err(super::super::unsupported(super::super::TextField {
            field: super::super::PROJECTION_FIELD,
            value: projection,
        })),
    }
}

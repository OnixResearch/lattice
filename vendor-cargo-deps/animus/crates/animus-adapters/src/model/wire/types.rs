/// Borrowed stream-frame fields before adapter-boundary validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Draft<'a> {
    /// Model-call correlation identifier.
    pub call_id: &'a str,
    /// Per-call order value.
    pub order: u64,
    /// Stream frame kind.
    pub kind: super::parts::Kind,
    /// Optional text body for text, tool, cancellation, and failure frames.
    pub body: Option<&'a str>,
    /// Optional tool-call identifier for tool-call deltas.
    pub tool_call_id: Option<&'a str>,
    /// Optional usage metadata for usage frames.
    pub usage: Option<super::parts::Usage>,
    /// Default sink visibility for text frames.
    pub visibility: super::parts::Visibility,
}

/// Canonical model stream frame emitted by model adapters.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Frame {
    pub(super) call_id: super::parts::CallId,
    pub(super) order: super::parts::Order,
    pub(super) kind: super::parts::Kind,
    pub(super) body: Option<alloc::string::String>,
    pub(super) tool_call_id: Option<super::parts::ToolCallId>,
    pub(super) usage: Option<super::parts::Usage>,
    pub(super) visibility: super::parts::Visibility,
}

impl Frame {
    /// Validates a model stream frame draft.
    ///
    /// # Errors
    ///
    /// Returns [`super::failure::Error`] when the frame has invalid identifiers,
    /// invalid order, missing required fields, unexpected fields, or unsafe
    /// thought visibility.
    pub fn from_draft(draft: Draft<'_>) -> Result<Self, super::failure::Error> {
        super::validate::frame_from_draft(draft)
    }

    /// Returns the model-call correlation identifier.
    #[must_use]
    pub const fn call_id(&self) -> &super::parts::CallId {
        &self.call_id
    }

    /// Returns the per-call order value.
    #[must_use]
    pub const fn order(&self) -> super::parts::Order {
        self.order
    }

    /// Returns the stream frame kind.
    #[must_use]
    pub const fn kind(&self) -> super::parts::Kind {
        self.kind
    }

    /// Returns the optional stream body.
    #[must_use]
    pub fn body(&self) -> Option<&str> {
        self.body.as_deref()
    }

    /// Returns the optional tool-call identifier.
    #[must_use]
    pub const fn tool_call_id(&self) -> Option<&super::parts::ToolCallId> {
        self.tool_call_id.as_ref()
    }

    /// Returns optional usage metadata.
    #[must_use]
    pub const fn usage(&self) -> Option<super::parts::Usage> {
        self.usage
    }

    /// Returns default sink visibility metadata.
    #[must_use]
    pub const fn visibility(&self) -> super::parts::Visibility {
        self.visibility
    }

    /// Returns the stable model stream frame contract identity.
    #[must_use]
    pub const fn contract_identity() -> &'static str {
        super::MODEL_STREAM_FRAME_CONTRACT_IDENTITY
    }

    /// Returns true when the default final-answer assembler may include this body.
    #[must_use]
    pub const fn is_default_final_visible(&self) -> bool {
        match self.kind {
            super::parts::Kind::ContentDelta => self.visibility.permits_default_final_text(),
            super::parts::Kind::ThoughtDelta
            | super::parts::Kind::ToolCallDelta
            | super::parts::Kind::Usage
            | super::parts::Kind::Finish
            | super::parts::Kind::Cancelled
            | super::parts::Kind::Failure => false,
        }
    }
}

pub(super) fn body_field_for(kind: super::parts::Kind) -> crate::validation::FieldName {
    match kind {
        super::parts::Kind::Cancelled => super::CANCELLATION_REASON_FIELD,
        super::parts::Kind::Failure => super::FAILURE_REASON_FIELD,
        super::parts::Kind::ContentDelta
        | super::parts::Kind::ThoughtDelta
        | super::parts::Kind::ToolCallDelta
        | super::parts::Kind::Usage
        | super::parts::Kind::Finish => super::BODY_FIELD,
    }
}

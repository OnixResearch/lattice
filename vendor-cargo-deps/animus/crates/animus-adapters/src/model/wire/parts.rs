/// Stable model-call correlation identifier carried by every stream frame.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CallId(alloc::string::String);

impl CallId {
    /// Builds a model-call identifier after adapter-boundary validation.
    ///
    /// # Errors
    ///
    /// Returns [`super::failure::Error`] when the identifier is empty, too long,
    /// or contains a NUL character.
    pub fn new(raw: &str) -> Result<Self, super::failure::Error> {
        Ok(Self(crate::validation::normalized_adapter_field(super::MODEL_CALL_ID_FIELD, raw)?))
    }

    /// Returns the model-call identifier text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

/// Stable tool-call correlation identifier for streamed tool-call deltas.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToolCallId(alloc::string::String);

impl ToolCallId {
    /// Builds a tool-call identifier after adapter-boundary validation.
    ///
    /// # Errors
    ///
    /// Returns [`super::failure::Error`] when the identifier is empty, too long,
    /// or contains a NUL character.
    pub fn new(raw: &str) -> Result<Self, super::failure::Error> {
        Ok(Self(crate::validation::normalized_adapter_field(super::TOOL_CALL_ID_FIELD, raw)?))
    }

    /// Returns the tool-call identifier text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

/// Strictly increasing per-call stream order value.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Order {
    value: u64,
}

impl Order {
    /// Builds a stream order value.
    ///
    /// # Errors
    ///
    /// Returns [`super::failure::Error`] when the value is below the first
    /// allowed stream order.
    pub const fn new(value: u64) -> Result<Self, super::failure::Error> {
        if value < super::MIN_ORDER {
            return Err(super::failure::Error::InvalidOrder { value });
        }
        Ok(Self { value })
    }

    /// Returns the raw stream order value.
    #[must_use]
    pub const fn value(self) -> u64 {
        self.value
    }
}

/// Canonical model stream frame kind at the adapter boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Kind {
    /// Assistant-visible content token or delta text.
    ContentDelta,
    /// Provider-exposed thought or reasoning delta text.
    ThoughtDelta,
    /// Tool-call argument/name delta text.
    ToolCallDelta,
    /// Usage update emitted before or at finish.
    Usage,
    /// Successful terminal frame.
    Finish,
    /// Cancelled terminal frame.
    Cancelled,
    /// Failed terminal frame.
    Failure,
}

impl Kind {
    /// Returns the stable stream-frame label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ContentDelta => "content_delta",
            Self::ThoughtDelta => "thought_delta",
            Self::ToolCallDelta => "tool_call_delta",
            Self::Usage => "usage",
            Self::Finish => "finish",
            Self::Cancelled => "cancelled",
            Self::Failure => "failure",
        }
    }

    /// Returns true when the frame kind terminates a model stream.
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        match self {
            Self::Finish | Self::Cancelled | Self::Failure => true,
            Self::ContentDelta | Self::ThoughtDelta | Self::ToolCallDelta | Self::Usage => false,
        }
    }

    pub(super) const fn requires_body(self) -> bool {
        match self {
            Self::ContentDelta | Self::ThoughtDelta | Self::ToolCallDelta | Self::Cancelled | Self::Failure => true,
            Self::Usage | Self::Finish => false,
        }
    }

    pub(super) const fn allows_usage(self) -> bool {
        match self {
            Self::Usage => true,
            Self::ContentDelta
            | Self::ThoughtDelta
            | Self::ToolCallDelta
            | Self::Finish
            | Self::Cancelled
            | Self::Failure => false,
        }
    }

    pub(super) const fn requires_tool_call_id(self) -> bool {
        match self {
            Self::ToolCallDelta => true,
            Self::ContentDelta | Self::ThoughtDelta | Self::Usage | Self::Finish | Self::Cancelled | Self::Failure => {
                false
            }
        }
    }
}

/// Default sink visibility attached to streamed text channels.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Visibility {
    /// Text may appear in assistant-visible final output by default.
    AssistantVisible,
    /// Text requires explicit host policy before any safe projection.
    PolicyGated,
    /// Text is redacted or omitted by default sinks.
    Redacted,
}

impl Visibility {
    /// Returns the stable visibility label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::AssistantVisible => "assistant_visible",
            Self::PolicyGated => "policy_gated",
            Self::Redacted => "redacted",
        }
    }

    /// Returns true when the default final-answer assembler may include text.
    #[must_use]
    pub const fn permits_default_final_text(self) -> bool {
        match self {
            Self::AssistantVisible => true,
            Self::PolicyGated | Self::Redacted => false,
        }
    }
}

/// Usage counts supplied by a model stream.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Usage {
    input_tokens: u32,
    output_tokens: u32,
}

impl Usage {
    /// Builds usage metadata from already-bounded token counts.
    #[must_use]
    pub const fn new(input_tokens: u32, output_tokens: u32) -> Self {
        Self {
            input_tokens,
            output_tokens,
        }
    }

    /// Returns input-token usage.
    #[must_use]
    pub const fn input_tokens(self) -> u32 {
        self.input_tokens
    }

    /// Returns output-token usage.
    #[must_use]
    pub const fn output_tokens(self) -> u32 {
        self.output_tokens
    }

    /// Returns saturating total-token usage.
    #[must_use]
    pub const fn total_tokens(self) -> u32 {
        self.input_tokens.saturating_add(self.output_tokens)
    }
}

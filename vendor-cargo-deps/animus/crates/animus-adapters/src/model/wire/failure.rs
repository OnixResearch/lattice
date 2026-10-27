/// Model streaming validation, parsing, and assembly failures.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    /// A nested adapter field failed validation.
    InvalidField {
        /// Field validation error.
        error: crate::AdapterContractError,
    },
    /// Per-call order was below the first allowed stream order.
    InvalidOrder {
        /// Invalid raw order value.
        value: u64,
    },
    /// A frame body was required but absent.
    MissingBody {
        /// Frame kind missing a body.
        kind: super::parts::Kind,
    },
    /// A frame body was supplied for a kind that forbids bodies.
    UnexpectedBody {
        /// Frame kind with an unexpected body.
        kind: super::parts::Kind,
    },
    /// A tool-call delta lacked its tool-call identifier.
    MissingToolCallId,
    /// A non-tool-call frame supplied a tool-call identifier.
    UnexpectedToolCallId {
        /// Frame kind with an unexpected tool-call identifier.
        kind: super::parts::Kind,
    },
    /// A usage frame lacked usage metadata.
    MissingUsage,
    /// A non-usage frame supplied usage metadata.
    UnexpectedUsage {
        /// Frame kind with unexpected usage metadata.
        kind: super::parts::Kind,
    },
    /// A thought frame attempted to opt into assistant-visible final text.
    ThoughtVisibilityLeak,
    /// The stream contained no frames.
    EmptyStream,
    /// One stream mixed frames from different model-call identifiers.
    MixedModelCall {
        /// First observed model-call identifier.
        expected: alloc::string::String,
        /// Later model-call identifier.
        actual: alloc::string::String,
    },
    /// Per-call order was not strictly increasing.
    OutOfOrderFrame {
        /// Previous order value.
        previous: u64,
        /// Next order value.
        next: u64,
    },
    /// No terminal frame was present.
    MissingTerminalFrame,
    /// A terminal frame was followed by more frames.
    TerminalFrameNotLast {
        /// Terminal frame kind that appeared before the end.
        kind: super::parts::Kind,
    },
    /// Final response assembly requires a successful finish frame.
    TerminalFrameNotFinish {
        /// Terminal frame kind observed.
        kind: super::parts::Kind,
    },
    /// Raw fixture input was malformed.
    MalformedFixture {
        /// Safe failure reason.
        reason: &'static str,
    },
}

impl From<crate::AdapterContractError> for Error {
    fn from(error: crate::AdapterContractError) -> Self {
        Self::InvalidField { error }
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidField { error } => {
                write!(formatter, "invalid model stream field: {error}")
            }
            Self::InvalidOrder { value } => write!(
                formatter,
                "model stream order must be at least {}; got {value}",
                super::MIN_ORDER
            ),
            Self::MissingBody { kind } => {
                write!(formatter, "{} frame requires a body", kind.as_str())
            }
            Self::UnexpectedBody { kind } => {
                write!(formatter, "{} frame does not accept a body", kind.as_str())
            }
            Self::MissingToolCallId => {
                write!(formatter, "tool-call stream frame requires a tool id")
            }
            Self::UnexpectedToolCallId { kind } => write!(
                formatter,
                "{} frame does not accept a tool-call identifier",
                kind.as_str()
            ),
            Self::MissingUsage => write!(formatter, "usage stream frame requires usage metadata"),
            Self::UnexpectedUsage { kind } => write!(
                formatter,
                "{} frame does not accept usage metadata",
                kind.as_str()
            ),
            Self::ThoughtVisibilityLeak => write!(
                formatter,
                "thought stream frames must not be assistant-visible by default"
            ),
            Self::EmptyStream => write!(formatter, "model stream must contain at least one frame"),
            Self::MixedModelCall { expected, actual } => write!(
                formatter,
                "model stream mixed call ids: expected {expected}, got {actual}"
            ),
            Self::OutOfOrderFrame { previous, next } => write!(
                formatter,
                "model stream order must increase: previous {previous}, next {next}"
            ),
            Self::MissingTerminalFrame => {
                write!(formatter, "model stream is missing a terminal frame")
            }
            Self::TerminalFrameNotLast { kind } => write!(
                formatter,
                "terminal {} frame must be the last stream frame",
                kind.as_str()
            ),
            Self::TerminalFrameNotFinish { kind } => write!(
                formatter,
                "final response assembly requires finish, got {}",
                kind.as_str()
            ),
            Self::MalformedFixture { reason } => {
                write!(formatter, "malformed stream fixture: {reason}")
            }
        }
    }
}

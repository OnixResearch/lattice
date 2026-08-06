/// Errors raised while parsing or building safe artifact envelopes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    /// An artifact hash failed deterministic validation.
    InvalidHash {
        /// Boundary field name.
        field: &'static str,
        /// Human-readable validation reason.
        reason: &'static str,
    },
    /// An artifact payload exceeded `MAX_PAYLOAD_CHARS`.
    PayloadTooLong {
        /// Maximum accepted character count.
        max_chars: usize,
        /// Actual character count.
        actual_chars: usize,
    },
    /// An artifact payload contained a NUL character.
    PayloadContainsNul {
        /// Byte index where the NUL character starts.
        byte_index: usize,
    },
}

impl core::fmt::Display for Error {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidHash { field, reason } => {
                write!(formatter, "{field} is invalid: {reason}")
            }
            Self::PayloadTooLong {
                max_chars,
                actual_chars,
            } => write!(formatter, "artifact payload must be at most {max_chars} characters; got {actual_chars}"),
            Self::PayloadContainsNul { byte_index } => {
                write!(formatter, "artifact payload contains a NUL character at byte index {byte_index}")
            }
        }
    }
}

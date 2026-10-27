/// Errors raised by context-reference parsing, planning, budgeting, or metadata validation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContextReferenceError {
    /// The reference did not start with the required sigil.
    MissingSigil,
    /// The reference had no target after trimming.
    EmptyReference,
    /// The reference kind is unsupported.
    UnsupportedKind {
        /// Human-readable reason.
        reason: &'static str,
    },
    /// The line range could not be parsed safely.
    MalformedRange {
        /// Human-readable reason.
        reason: &'static str,
    },
    /// The target attempted absolute, home-relative, or parent traversal.
    PathEscape {
        /// Human-readable reason.
        reason: &'static str,
    },
    /// The target contained unsafe authority metadata.
    UnsafeTarget {
        /// Human-readable reason.
        reason: &'static str,
    },
    /// The active policy denied expansion.
    ResolverDenied,
    /// A required resolver service was absent.
    ResolverMissing,
    /// A resolver service reported an unavailable backend.
    ResolverUnavailable,
    /// A resolver returned malformed output for the requested role.
    ResolverMalformedOutput,
    /// A resolver output exceeded host or policy bounds before budgeting.
    ResolverOversized,
    /// A configured budget was invalid.
    InvalidBudget,
    /// A count could not fit the public DTO bounds.
    CountOverflow,
    /// Safe metadata validation rejected the provenance record.
    SafeMetadata(crate::safe::receipt::Error),
}

impl core::fmt::Display for ContextReferenceError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::MissingSigil => formatter.write_str("context reference is missing @ sigil"),
            Self::EmptyReference => formatter.write_str(crate::prompt::base::EMPTY_TARGET_REASON),
            Self::UnsupportedKind { reason }
            | Self::MalformedRange { reason }
            | Self::PathEscape { reason }
            | Self::UnsafeTarget { reason } => formatter.write_str(reason),
            Self::ResolverDenied => {
                formatter.write_str("context reference resolver denied by policy")
            }
            Self::ResolverMissing => formatter.write_str("context reference resolver is missing"),
            Self::ResolverUnavailable => {
                formatter.write_str("context reference resolver is unavailable")
            }
            Self::ResolverMalformedOutput => {
                formatter.write_str("context reference resolver returned malformed output")
            }
            Self::ResolverOversized => {
                formatter.write_str("context reference resolver output is oversized")
            }
            Self::InvalidBudget => formatter.write_str("context reference budget must be positive"),
            Self::CountOverflow => formatter.write_str("context reference count overflowed u32"),
            Self::SafeMetadata(error) => write!(formatter, "{error}"),
        }
    }
}

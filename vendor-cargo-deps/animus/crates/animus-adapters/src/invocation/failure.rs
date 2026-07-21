const FAILURE_CLASS_ABSENT: &str = "absent";
const FAILURE_CLASS_INVALID_INPUT: &str = "invalid_input";
const FAILURE_CLASS_MISSING_EXECUTOR: &str = "missing_executor";
const FAILURE_CLASS_DENIED: &str = "denied";
const FAILURE_CLASS_DENIED_CAPABILITY: &str = "denied_capability";
const FAILURE_CLASS_APPROVAL_DENIED: &str = "approval_denied";
const FAILURE_CLASS_UNAVAILABLE: &str = "unavailable";
const FAILURE_CLASS_MALFORMED: &str = "malformed_output";
const FAILURE_CLASS_UNSAFE_OUTPUT_REFERENCE: &str = "unsafe_output_reference";
const FAILURE_CLASS_UNBOUNDED_OVERSIZED_OUTPUT: &str = "unbounded_oversized_output";
const FAILURE_CLASS_MUTATION_CONFLICT: &str = "mutation_conflict";
const FAILURE_CLASS_FALLBACK_DENIED: &str = "fallback_denied";
const FAILURE_CLASS_CANCELLED: &str = "cancelled";
const FAILURE_CLASS_TIMED_OUT: &str = "timed_out";
const FAILURE_CLASS_EXECUTION: &str = "execution_failure";

/// Typed fail-closed class for an adapter call that did not complete.
#[allow(path_segment_repetition)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdapterCallFailureClass {
    /// No matching service was injected for the required capability.
    Absent,
    /// Request or definition shape was invalid.
    InvalidInput,
    /// No executor was injected for the known tool.
    MissingExecutor,
    /// Capability or policy denied dispatch before completion.
    Denied,
    /// Exact capability policy denied dispatch.
    DeniedCapability,
    /// Approval policy denied dispatch.
    ApprovalDenied,
    /// A matching service existed but could not accept work.
    Unavailable,
    /// The adapter returned data that failed boundary validation.
    MalformedOutput,
    /// Output reference metadata was unsafe.
    UnsafeOutputReference,
    /// Output was oversized and not bounded.
    UnboundedOversizedOutput,
    /// Mutation-window coordination detected a conflict.
    MutationConflict,
    /// Fallback was requested and denied.
    FallbackDenied,
    /// The call was cancelled before completion.
    Cancelled,
    /// The call exceeded its caller-owned deadline.
    TimedOut,
    /// Legacy or typed adapter execution failed without a more specific class.
    ExecutionFailure,
}

impl AdapterCallFailureClass {
    /// Returns the stable fail-closed class label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Absent => FAILURE_CLASS_ABSENT,
            Self::InvalidInput => FAILURE_CLASS_INVALID_INPUT,
            Self::MissingExecutor => FAILURE_CLASS_MISSING_EXECUTOR,
            Self::Denied => FAILURE_CLASS_DENIED,
            Self::DeniedCapability => FAILURE_CLASS_DENIED_CAPABILITY,
            Self::ApprovalDenied => FAILURE_CLASS_APPROVAL_DENIED,
            Self::Unavailable => FAILURE_CLASS_UNAVAILABLE,
            Self::MalformedOutput => FAILURE_CLASS_MALFORMED,
            Self::UnsafeOutputReference => FAILURE_CLASS_UNSAFE_OUTPUT_REFERENCE,
            Self::UnboundedOversizedOutput => FAILURE_CLASS_UNBOUNDED_OVERSIZED_OUTPUT,
            Self::MutationConflict => FAILURE_CLASS_MUTATION_CONFLICT,
            Self::FallbackDenied => FAILURE_CLASS_FALLBACK_DENIED,
            Self::Cancelled => FAILURE_CLASS_CANCELLED,
            Self::TimedOut => FAILURE_CLASS_TIMED_OUT,
            Self::ExecutionFailure => FAILURE_CLASS_EXECUTION,
        }
    }

    /// Maps legacy injected-service outcomes into adapter-call failure classes.
    #[must_use]
    pub const fn from_outcome_kind(outcome: crate::OutcomeKind) -> Option<Self> {
        match outcome {
            crate::OutcomeKind::Absent => Some(Self::Absent),
            crate::OutcomeKind::Unavailable => Some(Self::Unavailable),
            crate::OutcomeKind::Denied => Some(Self::Denied),
            crate::OutcomeKind::Cancelled => Some(Self::Cancelled),
            crate::OutcomeKind::TimedOut => Some(Self::TimedOut),
            crate::OutcomeKind::MalformedOutput => Some(Self::MalformedOutput),
            crate::OutcomeKind::Completed => None,
        }
    }
}

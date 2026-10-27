/// Typed outcome class for an injected service call.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OutcomeKind {
    /// No service was injected for the required boundary.
    Absent,
    /// A service exists but is currently unavailable.
    Unavailable,
    /// Capability or policy denied the requested effect.
    Denied,
    /// The request was cancelled before completion.
    Cancelled,
    /// The request exceeded its caller-owned deadline.
    TimedOut,
    /// The service returned data that failed boundary validation.
    MalformedOutput,
    /// The service completed successfully.
    Completed,
}

impl OutcomeKind {
    /// Returns a stable outcome label for receipts and diagnostics.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Absent => "absent",
            Self::Unavailable => "unavailable",
            Self::Denied => "denied",
            Self::Cancelled => "cancelled",
            Self::TimedOut => "timed_out",
            Self::MalformedOutput => "malformed_output",
            Self::Completed => "completed",
        }
    }

    /// Returns whether this outcome represents a completed effect.
    #[must_use]
    pub const fn is_completed(self) -> bool {
        matches!(self, Self::Completed)
    }
}

/// Reason class for an injected-service outcome.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Reason {
    /// The required service was not injected.
    NotInjected,
    /// The host service reported unavailable capacity.
    HostUnavailable,
    /// Capability evaluation denied the request.
    CapabilityDenied,
    /// Runtime policy denied the request.
    PolicyDenied,
    /// The request was cancelled.
    Cancelled,
    /// The request timed out.
    DeadlineExceeded,
    /// The service returned invalid boundary data.
    InvalidBoundaryPayload,
    /// The broker returned no actionable confirmation decision.
    NoDecision,
    /// The confirmation approval did not match the pending request.
    MismatchedConfirmation,
    /// The confirmation approval expired before evaluation.
    StaleConfirmation,
    /// The service completed.
    Completed,
}

impl Reason {
    /// Returns a stable reason label for receipts and diagnostics.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NotInjected => "not_injected",
            Self::HostUnavailable => "host_unavailable",
            Self::CapabilityDenied => "capability_denied",
            Self::PolicyDenied => "policy_denied",
            Self::Cancelled => "cancelled",
            Self::DeadlineExceeded => "deadline_exceeded",
            Self::InvalidBoundaryPayload => "invalid_boundary_payload",
            Self::NoDecision => "no_decision",
            Self::MismatchedConfirmation => "mismatched_confirmation",
            Self::StaleConfirmation => "stale_confirmation",
            Self::Completed => "completed",
        }
    }
}

/// Result of an injected service call, including non-success outcomes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ServiceOutcome<T> {
    /// Service completed and returned a typed value.
    Completed {
        /// Completed value.
        value: T,
        /// Redacted service receipt.
        receipt: super::Receipt,
    },
    /// Required service was absent.
    Absent {
        /// Redacted service receipt.
        receipt: super::Receipt,
    },
    /// Required service was unavailable.
    Unavailable {
        /// Redacted service receipt.
        receipt: super::Receipt,
    },
    /// Required service was denied by capability or policy.
    Denied {
        /// Denied value, when a decision object is useful to callers.
        value: Option<T>,
        /// Redacted service receipt.
        receipt: super::Receipt,
    },
    /// Required service was cancelled.
    Cancelled {
        /// Redacted service receipt.
        receipt: super::Receipt,
    },
    /// Required service timed out.
    TimedOut {
        /// Redacted service receipt.
        receipt: super::Receipt,
    },
    /// Required service returned malformed output.
    MalformedOutput {
        /// Redacted service receipt.
        receipt: super::Receipt,
    },
}

impl<T> ServiceOutcome<T> {
    /// Builds a completed service outcome.
    #[must_use]
    pub fn completed(value: T, receipt: super::Receipt) -> Self {
        assert_specific_outcome(&receipt, OutcomeKind::Completed);
        Self::Completed { value, receipt }
    }

    /// Builds an absent service outcome.
    #[must_use]
    pub fn absent(receipt: super::Receipt) -> Self {
        assert_specific_outcome(&receipt, OutcomeKind::Absent);
        Self::Absent { receipt }
    }

    /// Builds an unavailable service outcome.
    #[must_use]
    pub fn unavailable(receipt: super::Receipt) -> Self {
        assert_specific_outcome(&receipt, OutcomeKind::Unavailable);
        Self::Unavailable { receipt }
    }

    /// Builds a denied service outcome.
    #[must_use]
    pub fn denied(value: Option<T>, receipt: super::Receipt) -> Self {
        assert_specific_outcome(&receipt, OutcomeKind::Denied);
        Self::Denied { value, receipt }
    }

    /// Builds a cancelled service outcome.
    #[must_use]
    pub fn cancelled(receipt: super::Receipt) -> Self {
        assert_specific_outcome(&receipt, OutcomeKind::Cancelled);
        Self::Cancelled { receipt }
    }

    /// Builds a timed-out service outcome.
    #[must_use]
    pub fn timed_out(receipt: super::Receipt) -> Self {
        assert_specific_outcome(&receipt, OutcomeKind::TimedOut);
        Self::TimedOut { receipt }
    }

    /// Builds a malformed-output service outcome.
    #[must_use]
    pub fn malformed_output(receipt: super::Receipt) -> Self {
        assert_specific_outcome(&receipt, OutcomeKind::MalformedOutput);
        Self::MalformedOutput { receipt }
    }

    /// Returns the outcome class.
    #[must_use]
    pub const fn kind(&self) -> OutcomeKind {
        match self {
            Self::Completed { .. } => OutcomeKind::Completed,
            Self::Absent { .. } => OutcomeKind::Absent,
            Self::Unavailable { .. } => OutcomeKind::Unavailable,
            Self::Denied { .. } => OutcomeKind::Denied,
            Self::Cancelled { .. } => OutcomeKind::Cancelled,
            Self::TimedOut { .. } => OutcomeKind::TimedOut,
            Self::MalformedOutput { .. } => OutcomeKind::MalformedOutput,
        }
    }

    /// Returns whether this service call completed successfully.
    #[must_use]
    pub const fn is_completed(&self) -> bool {
        self.kind().is_completed()
    }

    /// Returns the redacted receipt.
    #[must_use]
    pub const fn receipt(&self) -> &super::Receipt {
        match self {
            Self::Completed { receipt, .. }
            | Self::Absent { receipt }
            | Self::Unavailable { receipt }
            | Self::Denied { receipt, .. }
            | Self::Cancelled { receipt }
            | Self::TimedOut { receipt }
            | Self::MalformedOutput { receipt } => receipt,
        }
    }
}

fn assert_specific_outcome(receipt: &super::Receipt, expected: OutcomeKind) {
    assert_eq!(
        receipt.outcome(),
        expected,
        "service outcome must match receipt outcome"
    );
}

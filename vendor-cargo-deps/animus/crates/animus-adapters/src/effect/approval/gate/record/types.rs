/// Protected action class that needs confirmation before dispatch.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConfirmationActionKind {
    /// Model/provider dispatch action.
    ModelDispatch,
    /// Tool execution action.
    ToolExecution,
    /// Host filesystem, process, network, terminal, or persistence effect.
    HostEffect,
    /// Session or provenance persistence action.
    SessionPersistence,
    /// Remote or cross-session admission action.
    RemoteAdmission,
    /// Workflow or automation action.
    WorkflowAutomation,
    /// Runtime-policy or internal guard boundary.
    RuntimeBoundary,
}

impl ConfirmationActionKind {
    /// Returns a stable action-kind label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ModelDispatch => "model_dispatch",
            Self::ToolExecution => "tool_execution",
            Self::HostEffect => "host_effect",
            Self::SessionPersistence => "session_persistence",
            Self::RemoteAdmission => "remote_admission",
            Self::WorkflowAutomation => "workflow_automation",
            Self::RuntimeBoundary => "runtime_boundary",
        }
    }

    /// Maps a broad capability class to a protected action kind.
    #[must_use]
    pub const fn from_capability(capability: super::super::CapabilityClass) -> Self {
        match capability {
            super::super::CapabilityClass::ModelDispatch => Self::ModelDispatch,
            super::super::CapabilityClass::ToolExecution => Self::ToolExecution,
            super::super::CapabilityClass::HostEffect => Self::HostEffect,
            super::super::CapabilityClass::SessionPersistence => Self::SessionPersistence,
            super::super::CapabilityClass::Confirmation
            | super::super::CapabilityClass::Cancellation
            | super::super::CapabilityClass::RuntimePolicy => Self::RuntimeBoundary,
        }
    }

    /// Returns the broad capability class associated with this protected action.
    #[must_use]
    pub const fn capability(self) -> super::super::CapabilityClass {
        match self {
            Self::ModelDispatch => super::super::CapabilityClass::ModelDispatch,
            Self::ToolExecution => super::super::CapabilityClass::ToolExecution,
            Self::HostEffect => super::super::CapabilityClass::HostEffect,
            Self::SessionPersistence => super::super::CapabilityClass::SessionPersistence,
            Self::RemoteAdmission | Self::WorkflowAutomation | Self::RuntimeBoundary => {
                super::super::CapabilityClass::RuntimePolicy
            }
        }
    }
}

/// Confirmation strength required by policy for a protected action.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConfirmationLevel {
    /// Host may record an informational acknowledgement.
    Notice,
    /// Host must collect an explicit approval or denial.
    Explicit,
    /// Host should apply its highest-risk approval path.
    HighRisk,
}

impl ConfirmationLevel {
    /// Returns a stable confirmation-level label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Notice => "notice",
            Self::Explicit => "explicit",
            Self::HighRisk => "high_risk",
        }
    }
}

/// Monotonic host-provided tick used for pure expiry checks.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ConfirmationTick {
    value: u64,
}

impl ConfirmationTick {
    /// Builds a confirmation tick from host-owned monotonic state.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self { value }
    }

    /// Returns the host-owned tick value.
    #[must_use]
    pub const fn value(self) -> u64 {
        self.value
    }
}

/// Optional expiry attached to a confirmation request and approval binding.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConfirmationExpiry {
    expires_at: ConfirmationTick,
}

impl ConfirmationExpiry {
    /// Builds an expiry that remains fresh through `expires_at`.
    #[must_use]
    pub const fn new(expires_at: ConfirmationTick) -> Self {
        Self { expires_at }
    }

    /// Returns the latest host tick accepted for this confirmation.
    #[must_use]
    pub const fn expires_at(self) -> ConfirmationTick {
        self.expires_at
    }

    /// Returns whether `current_tick` is later than the accepted expiry tick.
    #[must_use]
    pub const fn is_stale(self, current_tick: ConfirmationTick) -> bool {
        current_tick.value() > self.expires_at.value()
    }
}

/// Final confirmation evaluation status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConfirmationStatus {
    /// Confirmation approved dispatch.
    Approved,
    /// Confirmation denied dispatch or failed closed.
    Denied,
}

impl ConfirmationStatus {
    /// Returns a stable status label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Approved => "approved",
            Self::Denied => "denied",
        }
    }
}

/// Typed confirmation issue code.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConfirmationIssue {
    /// No issue; dispatch may proceed.
    None,
    /// Broker was not injected.
    BrokerAbsent,
    /// Broker was unavailable.
    BrokerUnavailable,
    /// Broker denied the action.
    BrokerDenied,
    /// Broker call was cancelled.
    BrokerCancelled,
    /// Broker call timed out.
    BrokerTimedOut,
    /// Broker output failed boundary validation.
    MalformedDecision,
    /// Broker returned no actionable decision.
    NoDecision,
    /// Approval request ID did not match the pending request.
    MismatchedRequest,
    /// Approval action kind did not match the pending request.
    MismatchedAction,
    /// Approval resource did not match the pending request.
    MismatchedResource,
    /// Approval ability did not match the pending request.
    MismatchedAbility,
    /// Approval policy hash did not match the pending request.
    MismatchedPolicy,
    /// Approval expiry did not match the pending request.
    MismatchedExpiry,
    /// Approval or pending request expired before evaluation.
    StaleDecision,
}

impl ConfirmationIssue {
    /// Returns a stable issue label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::BrokerAbsent => "broker_absent",
            Self::BrokerUnavailable => "broker_unavailable",
            Self::BrokerDenied => "broker_denied",
            Self::BrokerCancelled => "broker_cancelled",
            Self::BrokerTimedOut => "broker_timed_out",
            Self::MalformedDecision => "malformed_decision",
            Self::NoDecision => "no_decision",
            Self::MismatchedRequest => "mismatched_request",
            Self::MismatchedAction => "mismatched_action",
            Self::MismatchedResource => "mismatched_resource",
            Self::MismatchedAbility => "mismatched_ability",
            Self::MismatchedPolicy => "mismatched_policy",
            Self::MismatchedExpiry => "mismatched_expiry",
            Self::StaleDecision => "stale_decision",
        }
    }
}

/// Confirmation broker class recorded in safe receipts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConfirmationBrokerClass {
    /// No injected broker was present and default-deny handled the request.
    DefaultDeny,
    /// Host-injected broker supplied the outcome.
    Injected,
}

impl ConfirmationBrokerClass {
    /// Returns a stable broker-class label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DefaultDeny => "default_deny",
            Self::Injected => "injected",
        }
    }
}

/// Timing class recorded in confirmation receipts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConfirmationTimingClass {
    /// Timing did not apply to this denial.
    NotApplicable,
    /// Approval was evaluated before expiry.
    Fresh,
    /// Approval was stale at evaluation.
    Expired,
    /// Broker call timed out.
    TimedOut,
    /// Broker call was cancelled.
    Cancelled,
}

impl ConfirmationTimingClass {
    /// Returns a stable timing label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NotApplicable => "not_applicable",
            Self::Fresh => "fresh",
            Self::Expired => "expired",
            Self::TimedOut => "timed_out",
            Self::Cancelled => "cancelled",
        }
    }
}

/// Expiry status recorded in confirmation receipts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConfirmationExpiryClass {
    /// Request did not carry an expiry.
    NoExpiry,
    /// Request expiry was still active.
    Active,
    /// Request expiry had elapsed.
    Expired,
}

impl ConfirmationExpiryClass {
    /// Returns a stable expiry label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NoExpiry => "no_expiry",
            Self::Active => "active",
            Self::Expired => "expired",
        }
    }
}

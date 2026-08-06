const CONFIRMATION_APPROVED: &str = "approved";
const CONFIRMATION_DENIED: &str = "denied";
const CONFIRMATION_MISSING: &str = "missing";
const CONFIRMATION_NOT_REQUIRED: &str = "not_required";
const MUTATION_PROTECTED: &str = "protected";
const MUTATION_UNPROTECTED: &str = "unprotected";
const POLICY_DISABLED: &str = "disabled";
const POLICY_BEST_EFFORT: &str = "best_effort";
const POLICY_STRICT: &str = "strict";
const SERVICE_INJECTED: &str = "injected";
const SERVICE_MISSING: &str = "missing";
const SERVICE_UNAVAILABLE: &str = "unavailable";
const SERVICE_DENIED: &str = "denied";
const SERVICE_UNSUPPORTED: &str = "unsupported";

/// Rollback confirmation status supplied by the host or operator.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RollbackConfirmationStatus {
    /// Exact rollback target was approved.
    Approved,
    /// Rollback was explicitly denied.
    Denied,
    /// No confirmation decision is available.
    Missing,
    /// Confirmation is not required for the operation class.
    NotRequired,
}
impl RollbackConfirmationStatus {
    /// Parses a confirmation-status label.
    ///
    /// # Errors
    /// Returns [`super::CheckpointError`] when the value is empty, malformed, or unknown.
    pub fn parse(value: &str) -> Result<Self, super::CheckpointError> {
        super::core::parse_enum(super::CHECKPOINT_CONFIRMATION_FIELD, value, |normalized| match normalized {
            CONFIRMATION_APPROVED => Some(Self::Approved),
            CONFIRMATION_DENIED => Some(Self::Denied),
            CONFIRMATION_MISSING => Some(Self::Missing),
            CONFIRMATION_NOT_REQUIRED => Some(Self::NotRequired),
            _ => None,
        })
    }

    /// Returns the stable confirmation label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Approved => CONFIRMATION_APPROVED,
            Self::Denied => CONFIRMATION_DENIED,
            Self::Missing => CONFIRMATION_MISSING,
            Self::NotRequired => CONFIRMATION_NOT_REQUIRED,
        }
    }
}

/// Whether a mutation is covered by checkpoint policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MutationProtection {
    /// Mutation requires checkpoint policy evaluation.
    Protected,
    /// Mutation is outside checkpoint policy scope.
    Unprotected,
}
impl MutationProtection {
    /// Parses a mutation-protection label.
    ///
    /// # Errors
    /// Returns [`super::CheckpointError`] when the value is empty, malformed, or unknown.
    pub fn parse(value: &str) -> Result<Self, super::CheckpointError> {
        super::core::parse_enum(super::CHECKPOINT_MUTATION_FIELD, value, |normalized| match normalized {
            MUTATION_PROTECTED => Some(Self::Protected),
            MUTATION_UNPROTECTED => Some(Self::Unprotected),
            _ => None,
        })
    }

    /// Returns the stable mutation-protection label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Protected => MUTATION_PROTECTED,
            Self::Unprotected => MUTATION_UNPROTECTED,
        }
    }
}

/// Policy for checkpointing protected mutations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CheckpointPolicy {
    /// Checkpoints are disabled.
    Disabled,
    /// Checkpoint failures are recorded but do not block dispatch.
    BestEffort,
    /// Checkpoint failures block dispatch before mutation.
    Strict,
}
impl CheckpointPolicy {
    /// Parses a checkpoint-policy label.
    ///
    /// # Errors
    /// Returns [`super::CheckpointError`] when the value is empty, malformed, or unknown.
    pub fn parse(value: &str) -> Result<Self, super::CheckpointError> {
        super::core::parse_enum(super::CHECKPOINT_POLICY_FIELD, value, |normalized| match normalized {
            POLICY_DISABLED => Some(Self::Disabled),
            POLICY_BEST_EFFORT => Some(Self::BestEffort),
            POLICY_STRICT => Some(Self::Strict),
            _ => None,
        })
    }

    /// Returns the stable policy label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Disabled => POLICY_DISABLED,
            Self::BestEffort => POLICY_BEST_EFFORT,
            Self::Strict => POLICY_STRICT,
        }
    }
}

/// Host-injected checkpoint service availability state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CheckpointServiceState {
    /// Service is injected and available.
    Injected,
    /// Service is not injected.
    Missing,
    /// Service is injected but unavailable.
    Unavailable,
    /// Service or policy denied checkpointing.
    Denied,
    /// Service does not support the requested operation.
    Unsupported,
}
impl CheckpointServiceState {
    /// Parses a checkpoint-service state label.
    ///
    /// # Errors
    /// Returns [`super::CheckpointError`] when the value is empty, malformed, or unknown.
    pub fn parse(value: &str) -> Result<Self, super::CheckpointError> {
        super::core::parse_enum(super::CHECKPOINT_SERVICE_FIELD, value, |normalized| match normalized {
            SERVICE_INJECTED => Some(Self::Injected),
            SERVICE_MISSING => Some(Self::Missing),
            SERVICE_UNAVAILABLE => Some(Self::Unavailable),
            SERVICE_DENIED => Some(Self::Denied),
            SERVICE_UNSUPPORTED => Some(Self::Unsupported),
            _ => None,
        })
    }

    /// Returns the stable service-state label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Injected => SERVICE_INJECTED,
            Self::Missing => SERVICE_MISSING,
            Self::Unavailable => SERVICE_UNAVAILABLE,
            Self::Denied => SERVICE_DENIED,
            Self::Unsupported => SERVICE_UNSUPPORTED,
        }
    }

    pub(super) const fn failure_class(self) -> super::CheckpointFailureClass {
        match self {
            Self::Injected => super::CheckpointFailureClass::None,
            Self::Missing => super::CheckpointFailureClass::MissingService,
            Self::Unavailable => super::CheckpointFailureClass::Unavailable,
            Self::Denied => super::CheckpointFailureClass::Denied,
            Self::Unsupported => super::CheckpointFailureClass::UnsupportedOperation,
        }
    }
}

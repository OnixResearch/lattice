/// Pure input for protected mutation checkpoint evaluation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProtectedMutationInput {
    /// Checkpoint policy.
    pub policy: super::CheckpointPolicy,
    /// Whether the requested mutation is protected.
    pub protection: super::MutationProtection,
    /// Host-injected checkpoint service state.
    pub service_state: super::CheckpointServiceState,
}

/// Pure decision for protected mutation checkpoint evaluation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProtectedMutationDecision {
    is_dispatch_allowed: bool,
    is_checkpoint_required: bool,
    failure_class: super::CheckpointFailureClass,
}

impl ProtectedMutationDecision {
    /// Returns true when the protected mutation may dispatch.
    #[must_use]
    pub const fn dispatch_allowed(self) -> bool {
        self.is_dispatch_allowed
    }

    /// Returns true when a pre-mutation checkpoint should be requested.
    #[must_use]
    pub const fn checkpoint_required(self) -> bool {
        self.is_checkpoint_required
    }

    /// Returns the checkpoint failure class.
    #[must_use]
    pub const fn failure_class(self) -> super::CheckpointFailureClass {
        self.failure_class
    }
}

/// Evaluates checkpoint policy before dispatching a mutating action.
///
/// # Panics
///
/// Panics if checkpoint enum variants change without updating the policy matrix.
#[must_use]
pub fn evaluate_protected_mutation(input: ProtectedMutationInput) -> ProtectedMutationDecision {
    assert!(
        matches!(input.protection, super::MutationProtection::Protected | super::MutationProtection::Unprotected),
        "mutation protection policy matrix is exhaustive"
    );
    assert!(
        matches!(
            input.policy,
            super::CheckpointPolicy::Disabled | super::CheckpointPolicy::BestEffort | super::CheckpointPolicy::Strict
        ),
        "checkpoint policy matrix is exhaustive"
    );
    if matches!(input.protection, super::MutationProtection::Unprotected)
        || matches!(input.policy, super::CheckpointPolicy::Disabled)
    {
        return ProtectedMutationDecision {
            is_dispatch_allowed: true,
            is_checkpoint_required: false,
            failure_class: super::CheckpointFailureClass::None,
        };
    }
    let failure_class = input.service_state.failure_class();
    let is_checkpoint_required = matches!(input.service_state, super::CheckpointServiceState::Injected);
    let is_dispatch_allowed = matches!(input.policy, super::CheckpointPolicy::BestEffort)
        || matches!(failure_class, super::CheckpointFailureClass::None);
    ProtectedMutationDecision {
        is_dispatch_allowed,
        is_checkpoint_required,
        failure_class,
    }
}

/// Pure input for rollback confirmation and freshness validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RollbackValidationInput<'a> {
    /// Target checkpoint metadata.
    pub target: &'a super::CheckpointRecord,
    /// Host/operator confirmation status.
    pub confirmation: super::RollbackConfirmationStatus,
    /// Workspace hash expected immediately before rollback.
    pub expected_current_hash: &'a super::CheckpointHash,
    /// Workspace hash observed immediately before rollback.
    pub observed_current_hash: &'a super::CheckpointHash,
}

/// Pure rollback validation result.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RollbackValidation {
    is_restore_allowed: bool,
    status: super::CheckpointStatus,
    failure_class: super::CheckpointFailureClass,
}

impl RollbackValidation {
    /// Returns true only when the checkpoint service may be asked to restore.
    #[must_use]
    pub const fn restore_allowed(self) -> bool {
        self.is_restore_allowed
    }

    /// Returns the safe rollback validation status.
    #[must_use]
    pub const fn status(self) -> super::CheckpointStatus {
        self.status
    }

    /// Returns the rollback failure class.
    #[must_use]
    pub const fn failure_class(self) -> super::CheckpointFailureClass {
        self.failure_class
    }
}

/// Validates rollback confirmation and freshness before backend mutation.
///
/// # Panics
///
/// Panics if callers bypass the validated checkpoint and hash constructors before
/// invoking the pure rollback validator.
#[must_use]
pub fn validate_rollback(input: RollbackValidationInput<'_>) -> RollbackValidation {
    assert!(!input.target.checkpoint_id().is_empty(), "rollback target must have a validated checkpoint id");
    assert!(!input.expected_current_hash.as_str().is_empty(), "rollback expected hash must be validated");
    if !matches!(input.confirmation, super::RollbackConfirmationStatus::Approved) {
        return RollbackValidation {
            is_restore_allowed: false,
            status: super::CheckpointStatus::Blocked,
            failure_class: super::CheckpointFailureClass::UnconfirmedRollback,
        };
    }
    if input.observed_current_hash != input.expected_current_hash {
        return RollbackValidation {
            is_restore_allowed: false,
            status: super::CheckpointStatus::Blocked,
            failure_class: super::CheckpointFailureClass::StaleWorkspace,
        };
    }
    RollbackValidation {
        is_restore_allowed: true,
        status: super::CheckpointStatus::RolledBack,
        failure_class: super::CheckpointFailureClass::None,
    }
}

/// Typed request for a host-injected checkpoint service.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckpointServiceRequest {
    operation: super::CheckpointOperation,
    target: Option<super::CheckpointRecord>,
}

impl CheckpointServiceRequest {
    /// Builds a checkpoint service request.
    #[must_use]
    pub const fn new(operation: super::CheckpointOperation, target: Option<super::CheckpointRecord>) -> Self {
        Self { operation, target }
    }

    /// Returns the requested operation.
    #[must_use]
    pub const fn operation(&self) -> super::CheckpointOperation {
        self.operation
    }

    /// Returns the optional target checkpoint metadata.
    #[must_use]
    pub const fn target(&self) -> Option<&super::CheckpointRecord> {
        self.target.as_ref()
    }
}

/// Typed response from a host-injected checkpoint service.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckpointServiceResponse {
    status: super::CheckpointStatus,
    failure_class: super::CheckpointFailureClass,
    receipt: super::CheckpointReceipt,
}

impl CheckpointServiceResponse {
    /// Builds a checkpoint service response.
    #[must_use]
    pub const fn new(
        status: super::CheckpointStatus,
        failure_class: super::CheckpointFailureClass,
        receipt: super::CheckpointReceipt,
    ) -> Self {
        Self {
            status,
            failure_class,
            receipt,
        }
    }

    /// Returns the response status.
    #[must_use]
    pub const fn status(&self) -> super::CheckpointStatus {
        self.status
    }

    /// Returns the response failure class.
    #[must_use]
    pub const fn failure_class(&self) -> super::CheckpointFailureClass {
        self.failure_class
    }

    /// Returns the safe response receipt.
    #[must_use]
    pub const fn receipt(&self) -> &super::CheckpointReceipt {
        &self.receipt
    }
}

/// Host-injected checkpoint service boundary.
pub trait CheckpointService: crate::ComponentAdapter<CheckpointServiceRequest, CheckpointServiceResponse> {}

impl<T> CheckpointService for T where T: crate::ComponentAdapter<CheckpointServiceRequest, CheckpointServiceResponse> {}

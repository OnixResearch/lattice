/// Borrowed policy input for gateway validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PolicyInput<'a> {
    /// Normalized delivery request.
    pub request: &'a ArtifactDeliveryRequest,
    /// Host active-context availability.
    pub context_status: DeliveryContextStatus,
    /// Host toolset availability.
    pub toolset_status: DeliveryToolsetStatus,
    /// Forbidden raw destination value.
    pub raw_destination: Option<&'a str>,
    /// Forbidden raw credential value.
    pub raw_credential: Option<&'a str>,
    /// Forbidden raw header value.
    pub raw_header: Option<&'a str>,
    /// Forbidden cloud object URL.
    pub cloud_object_url: Option<&'a str>,
    /// Forbidden webhook URL.
    pub webhook_url: Option<&'a str>,
    /// Forbidden raw platform payload.
    pub platform_payload: Option<&'a str>,
}

/// Safe gateway policy decision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyDecision {
    status: DeliveryPolicyStatus,
    failure_class: Option<DeliveryFailureClass>,
    retryability: DeliveryRetryability,
}

impl PolicyDecision {
    /// Returns the policy status.
    #[must_use]
    pub const fn status(&self) -> DeliveryPolicyStatus {
        self.status
    }

    /// Returns the policy failure class when denied.
    #[must_use]
    pub const fn failure_class(&self) -> Option<DeliveryFailureClass> {
        self.failure_class
    }

    /// Returns retryability for the policy outcome.
    #[must_use]
    pub const fn retryability(&self) -> DeliveryRetryability {
        self.retryability
    }
}

/// Validates gateway policy before adapter execution.
///
/// # Errors
///
/// This function is total over validated input and returns denied decisions for
/// unsafe requests instead of raising recoverable errors.
///
/// # Panics
///
/// Panics if a request previously normalized by this module no longer preserves
/// its non-empty safe artifact handle or toolset label invariants.
// r[impl agent_harness_runtime.artifact_delivery_gateway.policy]
#[must_use]
pub fn evaluate_policy(input: PolicyInput<'_>) -> PolicyDecision {
    assert!(
        !input.request.artifact_handle().is_empty(),
        "delivery policy requires a normalized artifact handle"
    );
    assert!(
        !input.request.toolset_label().is_empty(),
        "delivery policy requires a normalized toolset label"
    );
    if has_raw_destination_material(&input) {
        return denied(DeliveryFailureClass::UnsafeDestination);
    }
    if input.toolset_status == DeliveryToolsetStatus::Disabled {
        return denied(DeliveryFailureClass::DisabledToolset);
    }
    if !input.request.target_kind().reusable_policy_supports() {
        return denied(DeliveryFailureClass::UnsupportedTarget);
    }
    if input.request.target_kind().requires_active_context()
        && (input.context_status == DeliveryContextStatus::Missing
            || input.request.active_context_label().is_none())
    {
        return denied(DeliveryFailureClass::MissingContext);
    }
    PolicyDecision {
        status: DeliveryPolicyStatus::Approved,
        failure_class: None,
        retryability: DeliveryRetryability::Retryable,
    }
}

fn denied(failure_class: DeliveryFailureClass) -> PolicyDecision {
    PolicyDecision {
        status: DeliveryPolicyStatus::Denied,
        failure_class: Some(failure_class),
        retryability: DeliveryRetryability::NonRetryable,
    }
}

fn has_raw_destination_material(input: &PolicyInput<'_>) -> bool {
    input.raw_destination.is_some()
        || input.raw_credential.is_some()
        || input.raw_header.is_some()
        || input.cloud_object_url.is_some()
        || input.webhook_url.is_some()
        || input.platform_payload.is_some()
}

/// Borrowed adapter outcome before safe attempt recording.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DeliveryAdapterOutcomeDraft<'a> {
    /// Adapter outcome status.
    pub status: DeliveryAdapterOutcomeStatus,
    /// Optional safe platform handle supplied by the host adapter.
    pub platform_handle: Option<&'a str>,
    /// Optional safe adapter failure class.
    pub failure_class: Option<DeliveryFailureClass>,
}

/// Borrowed input for adapter-backed attempt recording.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DeliveryExecutionInput<'a> {
    /// Normalized delivery request.
    pub request: &'a ArtifactDeliveryRequest,
    /// Policy decision produced immediately before execution.
    pub policy: &'a PolicyDecision,
    /// Allocated attempt ID.
    pub attempt_id: &'a str,
    /// Injected adapter availability.
    pub adapter_availability: DeliveryAdapterAvailability,
    /// Adapter outcome when an injected adapter actually ran.
    pub adapter_outcome: Option<DeliveryAdapterOutcomeDraft<'a>>,
}

/// Safe outbox record for one delivery attempt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeliveryOutboxRecord {
    attempt_id: alloc::string::String,
    status: DeliveryAttemptStatus,
    retryability: DeliveryRetryability,
    target_kind: DeliveryTargetKind,
    artifact_kind: DeliveryArtifactKind,
    safe_artifact_label: alloc::string::String,
    artifact_hash: crate::artifact::Hash,
    adapter_availability: DeliveryAdapterAvailability,
    adapter_outcome: Option<DeliveryAdapterOutcomeStatus>,
    platform_handle: Option<alloc::string::String>,
    error_class: Option<DeliveryFailureClass>,
}

impl DeliveryOutboxRecord {
    /// Returns the attempt ID.
    #[must_use]
    pub fn attempt_id(&self) -> &str {
        self.attempt_id.as_str()
    }

    /// Returns the attempt status.
    #[must_use]
    pub const fn status(&self) -> DeliveryAttemptStatus {
        self.status
    }

    /// Returns retryability.
    #[must_use]
    pub const fn retryability(&self) -> DeliveryRetryability {
        self.retryability
    }

    /// Returns the target kind.
    #[must_use]
    pub const fn target_kind(&self) -> DeliveryTargetKind {
        self.target_kind
    }

    /// Returns the artifact kind.
    #[must_use]
    pub const fn artifact_kind(&self) -> DeliveryArtifactKind {
        self.artifact_kind
    }

    /// Returns the safe artifact label.
    #[must_use]
    pub fn safe_artifact_label(&self) -> &str {
        self.safe_artifact_label.as_str()
    }

    /// Returns the artifact hash.
    #[must_use]
    pub const fn artifact_hash(&self) -> &crate::artifact::Hash {
        &self.artifact_hash
    }

    /// Returns the adapter availability class.
    #[must_use]
    pub const fn adapter_availability(&self) -> DeliveryAdapterAvailability {
        self.adapter_availability
    }

    /// Returns the adapter outcome when present.
    #[must_use]
    pub const fn adapter_outcome(&self) -> Option<DeliveryAdapterOutcomeStatus> {
        self.adapter_outcome
    }

    /// Returns a safe platform handle when present.
    #[must_use]
    pub fn platform_handle(&self) -> Option<&str> {
        self.platform_handle.as_deref()
    }

    /// Returns the error class when the attempt failed.
    #[must_use]
    pub const fn error_class(&self) -> Option<DeliveryFailureClass> {
        self.error_class
    }
}


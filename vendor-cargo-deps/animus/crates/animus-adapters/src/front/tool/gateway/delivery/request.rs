/// Active-context state supplied by the host shell.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeliveryContextStatus {
    /// Required active context is present.
    Active,
    /// Required active context is missing or unavailable.
    Missing,
}

/// Toolset state supplied by host policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeliveryToolsetStatus {
    /// Requested toolset/capability is enabled.
    Enabled,
    /// Requested toolset/capability is disabled.
    Disabled,
}

/// Attempt status stored in the safe outbox.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeliveryAttemptStatus {
    /// Attempt was rejected or failed before adapter success.
    Failed,
    /// Attempt was delivered by an injected adapter.
    Succeeded,
}

impl DeliveryAttemptStatus {
    /// Returns the stable attempt-status label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Failed => "failed",
            Self::Succeeded => "succeeded",
        }
    }
}

/// Safe failure class for delivery policy, adapter, and retry outcomes.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum DeliveryFailureClass {
    /// Request contained invalid or unsupported metadata.
    InvalidRequest,
    /// Target kind is unsupported by reusable gateway policy.
    UnsupportedTarget,
    /// Required active context was missing.
    MissingContext,
    /// Request included raw destination, credential, header, URL, or payload data.
    UnsafeDestination,
    /// Requested toolset or capability is disabled.
    DisabledToolset,
    /// No injected adapter existed for the selected target.
    MissingAdapter,
    /// Adapter was unavailable.
    AdapterUnavailable,
    /// Adapter denied delivery.
    AdapterDenied,
    /// Adapter returned malformed output.
    MalformedAdapterOutput,
    /// Adapter reported a retryable failure.
    RetryableAdapterFailure,
    /// Adapter reported a non-retryable failure.
    NonRetryableAdapterFailure,
    /// Current policy denied retry.
    CurrentPolicyDenied,
    /// Attempt is not retryable.
    NotRetryable,
}

impl DeliveryFailureClass {
    /// Returns the stable failure label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InvalidRequest => "invalid-request",
            Self::UnsupportedTarget => "unsupported-target",
            Self::MissingContext => "missing-context",
            Self::UnsafeDestination => "unsafe-destination",
            Self::DisabledToolset => "disabled-toolset",
            Self::MissingAdapter => "missing-adapter",
            Self::AdapterUnavailable => "adapter-unavailable",
            Self::AdapterDenied => "adapter-denied",
            Self::MalformedAdapterOutput => "malformed-adapter-output",
            Self::RetryableAdapterFailure => "retryable-adapter-failure",
            Self::NonRetryableAdapterFailure => "non-retryable-adapter-failure",
            Self::CurrentPolicyDenied => "current-policy-denied",
            Self::NotRetryable => "not-retryable",
        }
    }
}

/// Borrowed artifact delivery request before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ArtifactDeliveryRequestDraft<'a> {
    /// Opaque host artifact handle, not raw artifact bytes.
    pub artifact_handle: &'a str,
    /// Lowercase BLAKE3 artifact hash.
    pub artifact_hash: &'a str,
    /// Artifact kind.
    pub artifact_kind: DeliveryArtifactKind,
    /// Delivery target kind.
    pub target_kind: DeliveryTargetKind,
    /// Optional active context label supplied by the host shell.
    pub active_context_label: Option<&'a str>,
    /// Safe artifact label for receipts and UI.
    pub safe_artifact_label: &'a str,
    /// Requested toolset label.
    pub toolset_label: &'a str,
    /// Requested capability class.
    pub capability: DeliveryCapabilityKind,
    /// Latest policy decision status when known.
    pub policy_status: DeliveryPolicyStatus,
    /// Delivery attempt ID when allocated by the shell.
    pub attempt_id: Option<&'a str>,
    /// Retryability requested or recorded for the attempt.
    pub retryability: DeliveryRetryability,
    /// Latest adapter outcome status when known.
    pub adapter_outcome: Option<DeliveryAdapterOutcomeStatus>,
    /// Forbidden raw artifact bytes, rejected when present.
    pub raw_artifact_bytes: Option<&'a [u8]>,
}

/// Validated artifact delivery request metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArtifactDeliveryRequest {
    artifact_handle: alloc::string::String,
    artifact_hash: crate::artifact::Hash,
    artifact_kind: DeliveryArtifactKind,
    target_kind: DeliveryTargetKind,
    active_context_label: Option<alloc::string::String>,
    safe_artifact_label: alloc::string::String,
    toolset_label: alloc::string::String,
    capability: DeliveryCapabilityKind,
    policy_status: DeliveryPolicyStatus,
    attempt_id: Option<alloc::string::String>,
    retryability: DeliveryRetryability,
    adapter_outcome: Option<DeliveryAdapterOutcomeStatus>,
}

impl ArtifactDeliveryRequest {
    /// Validates and normalizes a delivery request without embedding raw artifact bytes.
    ///
    /// # Errors
    ///
    /// Returns [`DeliveryError`] when safe text, hash, or no-raw-byte
    /// invariants fail.
    // r[impl agent_harness_runtime.artifact_delivery_gateway.delivery_dto]
    pub fn normalize(
        draft: &ArtifactDeliveryRequestDraft<'_>,
    ) -> Result<Self, DeliveryError> {
        if draft.raw_artifact_bytes.is_some() {
            return Err(DeliveryError::RawArtifactBytesEmbedded);
        }
        Ok(Self {
            artifact_handle: normalize(ARTIFACT_HANDLE_FIELD, draft.artifact_handle)?,
            artifact_hash: crate::artifact::Hash::new(draft.artifact_hash)
                .map_err(DeliveryError::InvalidHash)?,
            artifact_kind: draft.artifact_kind,
            target_kind: draft.target_kind,
            active_context_label: normalize_optional(
                CONTEXT_LABEL_FIELD,
                draft.active_context_label,
            )?,
            safe_artifact_label: normalize(ARTIFACT_LABEL_FIELD, draft.safe_artifact_label)?,
            toolset_label: normalize(TOOLSET_LABEL_FIELD, draft.toolset_label)?,
            capability: draft.capability,
            policy_status: draft.policy_status,
            attempt_id: normalize_optional(ATTEMPT_ID_FIELD, draft.attempt_id)?,
            retryability: draft.retryability,
            adapter_outcome: draft.adapter_outcome,
        })
    }

    /// Returns the opaque artifact handle.
    #[must_use]
    pub fn artifact_handle(&self) -> &str {
        self.artifact_handle.as_str()
    }

    /// Returns the artifact hash.
    #[must_use]
    pub const fn artifact_hash(&self) -> &crate::artifact::Hash {
        &self.artifact_hash
    }

    /// Returns the artifact kind.
    #[must_use]
    pub const fn artifact_kind(&self) -> DeliveryArtifactKind {
        self.artifact_kind
    }

    /// Returns the target kind.
    #[must_use]
    pub const fn target_kind(&self) -> DeliveryTargetKind {
        self.target_kind
    }

    /// Returns the active context label when present.
    #[must_use]
    pub fn active_context_label(&self) -> Option<&str> {
        self.active_context_label.as_deref()
    }

    /// Returns the safe artifact label.
    #[must_use]
    pub fn safe_artifact_label(&self) -> &str {
        self.safe_artifact_label.as_str()
    }

    /// Returns the requested toolset label.
    #[must_use]
    pub fn toolset_label(&self) -> &str {
        self.toolset_label.as_str()
    }

    /// Returns the requested capability.
    #[must_use]
    pub const fn capability(&self) -> DeliveryCapabilityKind {
        self.capability
    }

    /// Returns the policy status carried on the request.
    #[must_use]
    pub const fn policy_status(&self) -> DeliveryPolicyStatus {
        self.policy_status
    }

    /// Returns the attempt ID when present.
    #[must_use]
    pub fn attempt_id(&self) -> Option<&str> {
        self.attempt_id.as_deref()
    }

    /// Returns retryability metadata.
    #[must_use]
    pub const fn retryability(&self) -> DeliveryRetryability {
        self.retryability
    }

    /// Returns the adapter outcome status when present.
    #[must_use]
    pub const fn adapter_outcome(&self) -> Option<DeliveryAdapterOutcomeStatus> {
        self.adapter_outcome
    }
}


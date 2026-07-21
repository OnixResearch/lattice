
const ARTIFACT_HANDLE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("delivery_artifact_handle");
const ARTIFACT_LABEL_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("delivery_artifact_label");
const CONTEXT_LABEL_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("delivery_context_label");
const TOOLSET_LABEL_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("delivery_toolset_label");
const ATTEMPT_ID_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("delivery_attempt_id");
const PLATFORM_HANDLE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("delivery_platform_handle");
const CANONICAL_SEPARATOR: &str = "|";
const RECEIPT_CANONICAL_VERSION: &str = "delivery-receipt-v1";

/// Artifact class used by delivery policy and receipts.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum DeliveryArtifactKind {
    /// Text or markdown artifact.
    Text,
    /// Image artifact.
    Image,
    /// Audio artifact.
    Audio,
    /// Video artifact.
    Video,
    /// Archive artifact.
    Archive,
    /// Generic file artifact.
    File,
    /// Host-defined artifact class.
    Other,
}

impl DeliveryArtifactKind {
    /// Returns the stable artifact-kind label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Image => "image",
            Self::Audio => "audio",
            Self::Video => "video",
            Self::Archive => "archive",
            Self::File => "file",
            Self::Other => "other",
        }
    }
}

/// Delivery target class before host-specific adapter resolution.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum DeliveryTargetKind {
    /// Current local/session UI or workspace context.
    LocalSession,
    /// Host-managed opaque active context handle.
    HostContext,
    /// Matrix-room delivery, unsupported by reusable policy by default.
    MatrixRoom,
    /// Webhook delivery, unsupported by reusable policy by default.
    Webhook,
    /// Cloud object delivery, unsupported by reusable policy by default.
    CloudObject,
    /// Explicitly unsupported platform target.
    UnsupportedPlatform,
}

impl DeliveryTargetKind {
    /// Returns the stable target-kind label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::LocalSession => "local-session",
            Self::HostContext => "host-context",
            Self::MatrixRoom => "matrix-room",
            Self::Webhook => "webhook",
            Self::CloudObject => "cloud-object",
            Self::UnsupportedPlatform => "unsupported-platform",
        }
    }

    const fn requires_active_context(self) -> bool {
        match self {
            Self::LocalSession | Self::HostContext => true,
            Self::MatrixRoom | Self::Webhook | Self::CloudObject | Self::UnsupportedPlatform => {
                false
            }
        }
    }

    const fn reusable_policy_supports(self) -> bool {
        match self {
            Self::LocalSession | Self::HostContext => true,
            Self::MatrixRoom | Self::Webhook | Self::CloudObject | Self::UnsupportedPlatform => {
                false
            }
        }
    }
}

/// Delivery capability requested from a gateway adapter.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum DeliveryCapabilityKind {
    /// Display or attach an artifact to the current local session.
    SessionLocal,
    /// Save an artifact into a host-owned destination context.
    SaveArtifact,
    /// Attach an artifact to a host-managed platform context.
    AttachArtifact,
    /// Send an artifact through an external platform adapter.
    SendExternal,
}

impl DeliveryCapabilityKind {
    /// Returns the stable capability label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SessionLocal => "session-local",
            Self::SaveArtifact => "save-artifact",
            Self::AttachArtifact => "attach-artifact",
            Self::SendExternal => "send-external",
        }
    }
}

/// Policy status for a delivery request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeliveryPolicyStatus {
    /// Policy has not yet evaluated the request.
    Pending,
    /// Policy approved adapter execution.
    Approved,
    /// Policy denied adapter execution.
    Denied,
}

impl DeliveryPolicyStatus {
    /// Returns the stable policy-status label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Approved => "approved",
            Self::Denied => "denied",
        }
    }
}

/// Retry class for delivery attempts and failures.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeliveryRetryability {
    /// Attempt may be retried after current policy revalidation.
    Retryable,
    /// Attempt must not be retried.
    NonRetryable,
}

impl DeliveryRetryability {
    /// Returns the stable retryability label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Retryable => "retryable",
            Self::NonRetryable => "non-retryable",
        }
    }
}

/// Status returned by a host delivery adapter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeliveryAdapterOutcomeStatus {
    /// Adapter reports successful delivery.
    Succeeded,
    /// Adapter reports a retryable failure.
    RetryableFailure,
    /// Adapter reports a non-retryable failure.
    NonRetryableFailure,
    /// Adapter output could not be trusted or parsed.
    MalformedOutput,
}

impl DeliveryAdapterOutcomeStatus {
    /// Returns the stable adapter-outcome label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Succeeded => "succeeded",
            Self::RetryableFailure => "retryable-failure",
            Self::NonRetryableFailure => "non-retryable-failure",
            Self::MalformedOutput => "malformed-output",
        }
    }
}

/// Availability class for an injected delivery adapter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeliveryAdapterAvailability {
    /// Adapter is available for the selected target kind.
    Available,
    /// No adapter was injected for the selected target kind.
    Missing,
    /// Adapter exists but is temporarily unavailable.
    Unavailable,
    /// Adapter or host policy denied execution.
    Denied,
}

impl DeliveryAdapterAvailability {
    /// Returns the stable adapter-availability label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Available => "available",
            Self::Missing => "missing",
            Self::Unavailable => "unavailable",
            Self::Denied => "denied",
        }
    }
}


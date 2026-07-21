use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::num::NonZeroU16;

const WINDOW_ID_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("audit_window_id");
const MAX_AUDIT_WORK_SUMMARIES: usize = 6;
const MAX_AUDIT_FAILURE_SUMMARIES: usize = 8;
const CANONICAL_SEPARATOR: &str = "|";
const CANONICAL_VERSION_PREFIX: &str = "audit-receipt-v1";
const ZERO_AUDIT_COUNT: u32 = 0;

const _: () = assert!(MAX_AUDIT_WORK_SUMMARIES > 0);
const _: () = assert!(MAX_AUDIT_FAILURE_SUMMARIES > 0);

/// Runtime work category summarized by an audit snapshot.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum AuditWorkKind {
    /// Model/provider call work.
    ModelCall,
    /// Tool dispatch or execution work.
    ToolCall,
    /// Human or policy confirmation work.
    Confirmation,
    /// Background task work.
    BackgroundWork,
    /// Delivery attempt work.
    DeliveryAttempt,
    /// Retry work.
    Retry,
}

impl AuditWorkKind {
    /// Returns the stable work-kind label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ModelCall => "model-call",
            Self::ToolCall => "tool-call",
            Self::Confirmation => "confirmation",
            Self::BackgroundWork => "background-work",
            Self::DeliveryAttempt => "delivery-attempt",
            Self::Retry => "retry",
        }
    }
}

/// Safe status class for projected audit state.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum AuditStatusClass {
    /// No pending, completed, or failed work exists for the summary.
    Idle,
    /// Pending work exists.
    Pending,
    /// Completed work exists and no pending or failed work dominates.
    Completed,
    /// Failed work exists and dominates the status summary.
    Failed,
}

impl AuditStatusClass {
    /// Returns the stable status label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Pending => "pending",
            Self::Completed => "completed",
            Self::Failed => "failed",
        }
    }
}

/// Host-provided timing bucket for already-summarized duration data.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum AuditTimingBucket {
    /// No duration was observed.
    None,
    /// Immediate work.
    Immediate,
    /// Interactive latency work.
    Interactive,
    /// Background duration work.
    Background,
    /// Slow or over-budget duration work.
    Slow,
}

impl AuditTimingBucket {
    /// Returns the stable timing-bucket label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Immediate => "immediate",
            Self::Interactive => "interactive",
            Self::Background => "background",
            Self::Slow => "slow",
        }
    }
}

/// Safe failure class counted in audit snapshots.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum AuditFailureClass {
    /// Invalid input prevented work from running.
    InvalidInput,
    /// Capability or policy denied the work.
    DeniedCapability,
    /// Work timed out.
    Timeout,
    /// Work was cancelled.
    Cancellation,
    /// Work produced malformed output.
    MalformedOutput,
    /// Required service was unavailable.
    UnavailableService,
}

impl AuditFailureClass {
    /// Returns the stable failure-class label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InvalidInput => "invalid-input",
            Self::DeniedCapability => "denied-capability",
            Self::Timeout => "timeout",
            Self::Cancellation => "cancellation",
            Self::MalformedOutput => "malformed-output",
            Self::UnavailableService => "unavailable-service",
        }
    }
}

/// Host-owned observability boundary category.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuditHostBoundary {
    /// Metrics sink.
    MetricsSink,
    /// Telemetry exporter.
    TelemetryExporter,
    /// Host clock.
    Clock,
    /// Persistent storage.
    PersistentStorage,
    /// User-interface rendering.
    UiRendering,
}

/// Host-owned observability boundary status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuditHostBoundaryStatus {
    /// Host boundary was available.
    Available,
    /// Host boundary was unavailable.
    Unavailable,
}

/// Host-owned boundary outcome kept outside reusable projection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AuditHostBoundaryOutcome {
    boundary: AuditHostBoundary,
    status: AuditHostBoundaryStatus,
}

impl AuditHostBoundaryOutcome {
    /// Builds a host-owned boundary outcome.
    #[must_use]
    pub const fn new(boundary: AuditHostBoundary, status: AuditHostBoundaryStatus) -> Self {
        Self { boundary, status }
    }

    /// Returns the host-owned boundary category.
    #[must_use]
    pub const fn boundary(self) -> AuditHostBoundary {
        self.boundary
    }

    /// Returns the host-owned boundary status.
    #[must_use]
    pub const fn status(self) -> AuditHostBoundaryStatus {
        self.status
    }
}

/// Borrowed audit projection policy fields before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AuditProjectionPolicyDraft<'a> {
    /// Non-zero audit schema version.
    pub schema_version: u16,
    /// Maximum public pending count per work kind.
    pub public_pending_limit: u32,
    /// Lowercase BLAKE3 policy hash.
    pub policy_hash: &'a str,
}

/// Validated audit projection policy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuditProjectionPolicy {
    schema_version: NonZeroU16,
    public_pending_limit: u32,
    policy_hash: crate::artifact::Hash,
}

impl AuditProjectionPolicy {
    /// Validates audit projection policy metadata.
    ///
    /// # Errors
    ///
    /// Returns [`AuditReceiptError`] when schema or policy hash metadata is invalid.
    pub fn validate(draft: &AuditProjectionPolicyDraft<'_>) -> Result<Self, AuditReceiptError> {
        let schema_version =
            NonZeroU16::new(draft.schema_version).ok_or(AuditReceiptError::ZeroSchemaVersion)?;
        Ok(Self {
            schema_version,
            public_pending_limit: draft.public_pending_limit,
            policy_hash: crate::artifact::Hash::new(draft.policy_hash)
                .map_err(AuditReceiptError::InvalidHash)?,
        })
    }

    /// Returns the audit schema version.
    #[must_use]
    pub const fn schema_version(&self) -> NonZeroU16 {
        self.schema_version
    }

    /// Returns the configured public pending limit.
    #[must_use]
    pub const fn public_pending_limit(&self) -> u32 {
        self.public_pending_limit
    }

    /// Returns the policy identity.
    #[must_use]
    pub const fn policy_hash(&self) -> &crate::artifact::Hash {
        &self.policy_hash
    }
}

/// Borrowed event window identity before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AuditWindowDraft<'a> {
    /// Safe event window label or opaque identifier.
    pub window_id: &'a str,
    /// Lowercase BLAKE3 event window hash.
    pub event_window_hash: &'a str,
}

/// Service boundary that may perform or decide a host-owned effect.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ServiceKind {
    /// Model/provider dispatch service.
    ModelProvider,
    /// Tool execution service.
    ToolExecutor,
    /// Host-owned side-effect service.
    HostEffect,
    /// Capability decision service.
    CapabilityDecision,
    /// Cancellation check service.
    Cancellation,
    /// Confirmation broker service.
    Confirmation,
    /// Runtime policy check service.
    RuntimePolicy,
    /// Session ledger replay service.
    SessionLedger,
}

impl ServiceKind {
    /// Returns a stable service label for receipts and diagnostics.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ModelProvider => "model_provider",
            Self::ToolExecutor => "tool_executor",
            Self::HostEffect => "host_effect",
            Self::CapabilityDecision => "capability_decision",
            Self::Cancellation => "cancellation",
            Self::Confirmation => "confirmation",
            Self::RuntimePolicy => "runtime_policy",
            Self::SessionLedger => "session_ledger",
        }
    }
}

/// Capability class evaluated before a host-owned effect can run.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CapabilityClass {
    /// Model or provider dispatch.
    ModelDispatch,
    /// Tool invocation.
    ToolExecution,
    /// Host filesystem, process, network, terminal, or persistence effect.
    HostEffect,
    /// Human or host confirmation.
    Confirmation,
    /// Cancellation observation.
    Cancellation,
    /// Runtime policy decision.
    RuntimePolicy,
    /// Session or provenance persistence.
    SessionPersistence,
}

impl CapabilityClass {
    /// Returns a stable capability label for receipts and diagnostics.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ModelDispatch => "model_dispatch",
            Self::ToolExecution => "tool_execution",
            Self::HostEffect => "host_effect",
            Self::Confirmation => "confirmation",
            Self::Cancellation => "cancellation",
            Self::RuntimePolicy => "runtime_policy",
            Self::SessionPersistence => "session_persistence",
        }
    }
}

/// Redaction marker proving receipts did not capture raw payloads.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RedactionMarker {
    /// Payloads were not captured at all.
    PayloadNotCaptured,
    /// Only a safe hash of payload content was captured.
    PayloadHashOnly,
    /// Only bounded counts were captured.
    CountsOnly,
}

impl RedactionMarker {
    /// Returns a stable redaction label for receipts and diagnostics.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PayloadNotCaptured => "payload_not_captured",
            Self::PayloadHashOnly => "payload_hash_only",
            Self::CountsOnly => "counts_only",
        }
    }
}

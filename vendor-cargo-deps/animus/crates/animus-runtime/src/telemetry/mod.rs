mod dto;
mod observability;

pub use dto::RuntimeEvent;
/// Root telemetry projection for safe observability records.
pub type RuntimeObservabilityProjection = observability::Projection;

/// Projects a complete run report into safe observability records.
///
/// # Errors
///
/// Returns [`animus_adapters::ObservabilityError`] when generated labels fail validation.
pub fn project_run_report_observability(
    report: &crate::RunReport,
) -> Result<RuntimeObservabilityProjection, animus_adapters::ObservabilityError> {
    observability::project_report(report)
}

/// Projects one runtime event into safe observability records.
///
/// # Errors
///
/// Returns [`animus_adapters::ObservabilityError`] when generated labels fail validation.
pub fn project_runtime_event_observability(
    event: &RuntimeEvent,
    sequence: u32,
) -> Result<RuntimeObservabilityProjection, animus_adapters::ObservabilityError> {
    observability::project_runtime_event(event, sequence)
}

/// Adapter component kind used in runtime events.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdapterComponent {
    /// Model adapter component.
    Model,
    /// Tool adapter component.
    Tool,
}

impl AdapterComponent {
    /// Returns the stable adapter-component label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Model => "model",
            Self::Tool => "tool",
        }
    }
}

/// Loop decision class selected by the fabric.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LoopDecisionKind {
    /// Model dispatch decision.
    Model,
    /// Tool dispatch decision.
    Tool,
    /// Final response decision.
    Respond,
    /// Stop decision.
    Stop,
}

impl LoopDecisionKind {
    /// Returns the stable decision label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Model => "model",
            Self::Tool => "tool",
            Self::Respond => "respond",
            Self::Stop => "stop",
        }
    }
}

/// Terminal runtime status class emitted at shutdown.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeTerminalKind {
    /// Run completed with a final response.
    FinalResponse,
    /// Run stopped without a final response.
    Stopped,
    /// Run exhausted its configured turn budget.
    MaxTurnsExceeded,
    /// Run failed closed.
    Failed,
}

impl RuntimeTerminalKind {
    /// Returns the stable terminal status label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::FinalResponse => "final_response",
            Self::Stopped => "stopped",
            Self::MaxTurnsExceeded => "max_turns_exceeded",
            Self::Failed => "failed",
        }
    }

    pub(crate) fn from_status(status: &crate::RuntimeStatus) -> Self {
        match status {
            crate::RuntimeStatus::FinalResponse { .. } => Self::FinalResponse,
            crate::RuntimeStatus::Stopped => Self::Stopped,
            crate::RuntimeStatus::MaxTurnsExceeded { .. } => Self::MaxTurnsExceeded,
            crate::RuntimeStatus::Failed { .. } => Self::Failed,
        }
    }
}

/// Stable runtime event kind for headless consumers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeEventKind {
    /// Runtime start event.
    RuntimeStarted,
    /// Compatibility loop-start event.
    LoopStarted,
    /// Context replay event.
    ContextReplayed,
    /// Turn start event.
    TurnStarted,
    /// Loop decision event.
    LoopDecision,
    /// Model request event.
    ModelRequested,
    /// Model stream-start event.
    ModelStreamStarted,
    /// Model stream-frame event.
    ModelStreamFrame,
    /// Model stream-end event.
    ModelStreamEnded,
    /// Model stream-failure event.
    ModelStreamFailed,
    /// Provider-neutral retry progress event.
    ProviderRetryProgress,
    /// Model response event.
    ModelResponded,
    /// Model denial event.
    ModelDenied,
    /// Tool request event.
    ToolRequested,
    /// Tool response event.
    ToolResponded,
    /// Tool denial event.
    ToolDenied,
    /// Final response event.
    FinalResponse,
    /// Stop event.
    Stopped,
    /// Max-turn exhaustion event.
    MaxTurnsExceeded,
    /// Typed adapter outcome classification event.
    AdapterOutcomeClassified,
    /// Adapter error event.
    AdapterError,
    /// Fabric error event.
    FabricError,
    /// Hook block event.
    HookBlocked,
    /// Hook failure event.
    HookFailed,
    /// Lifecycle hook outcome event.
    LifecycleHookOutcome,
    /// Steel policy denial event.
    SteelPolicyDenied,
    /// Sandbox escalation event.
    SandboxEscalation,
    /// Cancellation event.
    Cancelled,
    /// Compaction event.
    CompactionRequested,
    /// Shutdown event.
    Shutdown,
}

impl RuntimeEventKind {
    /// Returns the stable event-kind label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RuntimeStarted => "runtime_started",
            Self::LoopStarted => "loop_started",
            Self::ContextReplayed => "context_replayed",
            Self::TurnStarted => "turn_started",
            Self::LoopDecision => "loop_decision",
            Self::ModelRequested => "model_requested",
            Self::ModelStreamStarted => "model_stream_started",
            Self::ModelStreamFrame => "model_stream_frame",
            Self::ModelStreamEnded => "model_stream_ended",
            Self::ModelStreamFailed => "model_stream_failed",
            Self::ProviderRetryProgress => "provider_retry_progress",
            Self::ModelResponded => "model_responded",
            Self::ModelDenied => "model_denied",
            Self::ToolRequested => "tool_requested",
            Self::ToolResponded => "tool_responded",
            Self::ToolDenied => "tool_denied",
            Self::FinalResponse => "final_response",
            Self::Stopped => "stopped",
            Self::MaxTurnsExceeded => "max_turns_exceeded",
            Self::AdapterOutcomeClassified => "adapter_outcome_classified",
            Self::AdapterError => "adapter_error",
            Self::FabricError => "fabric_error",
            Self::HookBlocked => "hook_blocked",
            Self::HookFailed => "hook_failed",
            Self::LifecycleHookOutcome => "lifecycle_hook_outcome",
            Self::SteelPolicyDenied => "steel_policy_denied",
            Self::SandboxEscalation => "sandbox_escalation",
            Self::Cancelled => "cancelled",
            Self::CompactionRequested => "compaction_requested",
            Self::Shutdown => "shutdown",
        }
    }
}

/// Projects canonical model stream frames into runtime-order stream events.
///
/// # Errors
///
/// Returns [`animus_adapters::ModelStreamError`] when the stream is invalid.
pub fn project_model_stream_events(
    frames: &[animus_adapters::ModelStreamFrame],
) -> Result<Vec<RuntimeEvent>, animus_adapters::ModelStreamError> {
    dto::project_model_stream_events(frames)
}

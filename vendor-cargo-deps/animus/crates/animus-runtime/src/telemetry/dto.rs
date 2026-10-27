const STREAM_PROJECTION_EXTRA_EVENTS: usize = 2;

/// Ordered runtime event emitted by the agent loop.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RuntimeEvent {
    /// Runtime started for a validated runtime input summary.
    RuntimeStarted {
        /// Deterministic input summary made available to the runtime.
        summary: String,
    },
    /// The loop started for a validated runtime input summary.
    LoopStarted {
        /// Deterministic input summary made available to the loop fabric.
        summary: String,
    },
    /// Context replay boundary was observed.
    ContextReplayed {
        /// Deterministic context summary for headless replay consumers.
        summary: String,
    },
    /// A loop-fabric turn started.
    TurnStarted {
        /// Zero-based deterministic turn index.
        turn_index: usize,
    },
    /// The loop fabric selected a decision for the current turn.
    LoopDecision {
        /// Zero-based deterministic turn index.
        turn_index: usize,
        /// Decision class selected by the fabric.
        decision: super::LoopDecisionKind,
    },
    /// A validated model request is about to cross the model adapter boundary.
    ModelRequested {
        /// Validated model request.
        request: animus_adapters::ModelRequest,
    },
    /// A model stream started for a correlated model call.
    ModelStreamStarted {
        /// Stable model-call correlation identifier.
        call_id: animus_adapters::ModelStreamCallId,
    },
    /// A non-terminal model stream frame was projected before completion.
    ModelStreamFrame {
        /// Canonical model stream frame.
        frame: animus_adapters::ModelStreamFrame,
    },
    /// A model stream ended without a stream failure.
    ModelStreamEnded {
        /// Stable model-call correlation identifier.
        call_id: animus_adapters::ModelStreamCallId,
        /// Terminal stream status.
        status: animus_adapters::ModelStreamFrameKind,
    },
    /// A model stream failed with a bounded safe reason.
    ModelStreamFailed {
        /// Stable model-call correlation identifier.
        call_id: animus_adapters::ModelStreamCallId,
        /// Safe failure reason.
        reason: String,
    },
    /// Provider-neutral progress for a bounded same-plan retry.
    ProviderRetryProgress {
        /// Safe retry progress metadata, never assistant transcript content.
        progress: crate::provider_retry::ProviderRetryProgressEvent,
    },
    /// The model adapter returned a validated response.
    ModelResponded {
        /// Validated model response.
        response: animus_adapters::ModelResponse,
    },
    /// A model guard denied dispatch before adapter execution.
    ModelDenied {
        /// Denied model request.
        request: animus_adapters::ModelRequest,
        /// Safe denial reason.
        reason: String,
    },
    /// A validated tool request is about to cross the tool adapter boundary.
    ToolRequested {
        /// Validated tool request.
        request: animus_adapters::ToolRequest,
    },
    /// The tool adapter returned a validated response.
    ToolResponded {
        /// Validated tool response.
        response: animus_adapters::ToolResponse,
    },
    /// A tool guard denied dispatch before adapter execution.
    ToolDenied {
        /// Denied tool request.
        request: animus_adapters::ToolRequest,
        /// Safe denial reason.
        reason: String,
    },
    /// The loop fabric selected a final response.
    FinalResponse {
        /// Validated final model response.
        response: animus_adapters::ModelResponse,
    },
    /// The loop fabric requested host-owned completion evaluation.
    CompletionCandidateRequested {
        /// BLAKE3 identity of the exact request spelling.
        request_hash: String,
    },
    /// The loop fabric selected a stop decision without a final response.
    Stopped,
    /// The configured maximum-turn limit was reached before completion.
    MaxTurnsExceeded {
        /// Configured maximum number of fabric turns.
        max_turns: usize,
    },
    /// A typed adapter outcome failed closed before completion.
    AdapterOutcomeClassified {
        /// Adapter component that failed closed.
        component: super::AdapterComponent,
        /// Stable fail-closed class.
        class: animus_adapters::AdapterCallFailureClass,
        /// Bounded safe outcome message.
        message: String,
    },
    /// An adapter returned an error and the runtime halted.
    AdapterError {
        /// Adapter component that failed.
        component: super::AdapterComponent,
        /// Human-readable adapter error message.
        message: String,
    },
    /// The loop fabric returned an error and the runtime halted.
    FabricError {
        /// Human-readable fabric error message.
        message: String,
    },
    /// Host middleware blocked a guarded boundary.
    HookBlocked {
        /// Boundary that was blocked.
        boundary: crate::HookBoundary,
        /// Middleware hook label.
        hook: String,
        /// Safe block reason.
        reason: String,
    },
    /// Host middleware failed at a guarded boundary.
    HookFailed {
        /// Boundary that failed.
        boundary: crate::HookBoundary,
        /// Middleware hook label.
        hook: String,
        /// Safe failure message.
        message: String,
    },
    /// Typed lifecycle hook outcome with redacted input metadata.
    LifecycleHookOutcome {
        /// Safe lifecycle hook outcome report.
        outcome: crate::LifecycleHookOutcome,
    },
    /// Steel policy denied activation before Steel evaluation.
    SteelPolicyDenied {
        /// Safe selected profile label.
        profile: String,
        /// BLAKE3 hash of observed script source.
        script_hash: String,
        /// BLAKE3 hash of safe receipt fields.
        receipt_hash: String,
        /// Typed denial reason label.
        reason: String,
    },
    /// Sandbox escalation policy decision was recorded with redacted evidence.
    SandboxEscalation {
        /// Exec-policy decision for the escalation.
        decision: animus_adapters::ExecPolicyDecision,
        /// Matched rule identities.
        rule_ids: Vec<String>,
        /// BLAKE3 command fingerprints only; raw command bodies are omitted.
        command_fingerprints: Vec<String>,
        /// Active sandbox label before escalation.
        active_sandbox: String,
        /// Requested capability outside the active sandbox.
        requested_capability: String,
        /// Safe escalation reason label.
        escalation_reason: String,
        /// Redaction marker proving command bodies were not embedded.
        redaction_marker: String,
    },
    /// A host cancellation boundary was observed.
    Cancelled {
        /// Safe cancellation reason.
        reason: String,
    },
    /// A compaction boundary was requested.
    CompactionRequested {
        /// Safe compaction summary.
        summary: String,
    },
    /// Coding-session lifecycle state changed without exposing transcript bodies.
    SessionStateChanged {
        /// Stable session identity.
        session_id: String,
        /// Active run identity when one exists.
        run_id: Option<String>,
        /// Safe lifecycle status.
        status: String,
        /// Bounded pending queue count.
        queue_count: u32,
    },
    /// A queue compare-and-swap result was observed without private text.
    SessionQueueMutationObserved {
        /// Bounded queue mutation metadata.
        mutation: crate::coding_session_host::CodingSessionQueueMutationEvent,
    },
    /// A coding-session message crossed its declared durable boundary.
    SessionCommitObserved {
        /// Stable session identity.
        session_id: String,
        /// Correlated run identity.
        run_id: String,
        /// Committed entry identity.
        entry_id: String,
        /// Safe committed message kind.
        message_kind: String,
        /// Committed BLAKE3 head identity.
        head_hash: String,
        /// Logical durable cursor after commit.
        durable_cursor: u64,
    },
    /// Coding-session recovery produced one validated metadata-only observation.
    SessionRecoveryObserved {
        /// Stable session identity.
        session_id: String,
        /// Safe recovery status.
        status: String,
        /// Committed BLAKE3 head identity.
        head_hash: String,
        /// Recovered active leaf identity when present.
        active_leaf_id: Option<String>,
    },
    /// Session authority-profile continuity or binding was observed.
    SessionAuthorityProfileObserved {
        /// Stable session identity.
        session_id: String,
        /// Stable branch identity.
        branch_id: String,
        /// Safe admission or binding status.
        status: String,
        /// Current requested profile BLAKE3 identity.
        profile_identity: String,
        /// Durable binding identity when one exists.
        binding_identity: Option<String>,
        /// Safe authorization decision identity when one exists.
        authorization_identity: Option<String>,
        /// Current realized-enforcement observation identity when one exists.
        enforcement_observation_identity: Option<String>,
        /// Bounded safe component classes that changed.
        changed_components: Vec<String>,
    },
    /// Runtime completed and emitted a terminal status class.
    Shutdown {
        /// Terminal status class.
        status: super::RuntimeTerminalKind,
    },
}

impl RuntimeEvent {
    /// Returns the stable kind for this event.
    #[must_use]
    pub fn kind(&self) -> super::RuntimeEventKind {
        match self {
            Self::RuntimeStarted { .. } => super::RuntimeEventKind::RuntimeStarted,
            Self::LoopStarted { .. } => super::RuntimeEventKind::LoopStarted,
            Self::ContextReplayed { .. } => super::RuntimeEventKind::ContextReplayed,
            Self::TurnStarted { .. } => super::RuntimeEventKind::TurnStarted,
            Self::LoopDecision { .. } => super::RuntimeEventKind::LoopDecision,
            Self::ModelRequested { .. } => super::RuntimeEventKind::ModelRequested,
            Self::ModelStreamStarted { .. } => super::RuntimeEventKind::ModelStreamStarted,
            Self::ModelStreamFrame { .. } => super::RuntimeEventKind::ModelStreamFrame,
            Self::ModelStreamEnded { .. } => super::RuntimeEventKind::ModelStreamEnded,
            Self::ModelStreamFailed { .. } => super::RuntimeEventKind::ModelStreamFailed,
            Self::ProviderRetryProgress { .. } => super::RuntimeEventKind::ProviderRetryProgress,
            Self::ModelResponded { .. } => super::RuntimeEventKind::ModelResponded,
            Self::ModelDenied { .. } => super::RuntimeEventKind::ModelDenied,
            Self::ToolRequested { .. } => super::RuntimeEventKind::ToolRequested,
            Self::ToolResponded { .. } => super::RuntimeEventKind::ToolResponded,
            Self::ToolDenied { .. } => super::RuntimeEventKind::ToolDenied,
            Self::FinalResponse { .. } => super::RuntimeEventKind::FinalResponse,
            Self::CompletionCandidateRequested { .. } => {
                super::RuntimeEventKind::CompletionCandidateRequested
            }
            Self::Stopped => super::RuntimeEventKind::Stopped,
            Self::MaxTurnsExceeded { .. } => super::RuntimeEventKind::MaxTurnsExceeded,
            Self::AdapterOutcomeClassified { .. } => {
                super::RuntimeEventKind::AdapterOutcomeClassified
            }
            Self::AdapterError { .. } => super::RuntimeEventKind::AdapterError,
            Self::FabricError { .. } => super::RuntimeEventKind::FabricError,
            Self::HookBlocked { .. } => super::RuntimeEventKind::HookBlocked,
            Self::HookFailed { .. } => super::RuntimeEventKind::HookFailed,
            Self::LifecycleHookOutcome { .. } => super::RuntimeEventKind::LifecycleHookOutcome,
            Self::SteelPolicyDenied { .. } => super::RuntimeEventKind::SteelPolicyDenied,
            Self::SandboxEscalation { .. } => super::RuntimeEventKind::SandboxEscalation,
            Self::Cancelled { .. } => super::RuntimeEventKind::Cancelled,
            Self::CompactionRequested { .. } => super::RuntimeEventKind::CompactionRequested,
            Self::SessionStateChanged { .. } => super::RuntimeEventKind::SessionStateChanged,
            Self::SessionQueueMutationObserved { .. } => {
                super::RuntimeEventKind::SessionQueueMutationObserved
            }
            Self::SessionCommitObserved { .. } => super::RuntimeEventKind::SessionCommitObserved,
            Self::SessionRecoveryObserved { .. } => {
                super::RuntimeEventKind::SessionRecoveryObserved
            }
            Self::SessionAuthorityProfileObserved { .. } => {
                super::RuntimeEventKind::SessionAuthorityProfileObserved
            }
            Self::Shutdown { .. } => super::RuntimeEventKind::Shutdown,
        }
    }
}

pub(super) fn project_model_stream_events(
    frames: &[animus_adapters::ModelStreamFrame],
) -> Result<Vec<RuntimeEvent>, animus_adapters::ModelStreamError> {
    animus_adapters::validate_model_stream_frames(frames)?;
    let call_id = first_stream_call_id(frames)?;
    let mut events = Vec::with_capacity(stream_projection_capacity(frames.len()));
    events.push(RuntimeEvent::ModelStreamStarted { call_id });

    for frame in frames {
        events.push(project_stream_frame_event(frame));
    }

    Ok(events)
}

fn first_stream_call_id(
    frames: &[animus_adapters::ModelStreamFrame],
) -> Result<animus_adapters::ModelStreamCallId, animus_adapters::ModelStreamError> {
    let Some(first) = frames.first() else {
        return Err(animus_adapters::ModelStreamError::EmptyStream);
    };
    Ok(first.call_id().clone())
}

const fn stream_projection_capacity(frame_count: usize) -> usize {
    frame_count.saturating_add(STREAM_PROJECTION_EXTRA_EVENTS)
}

fn project_stream_frame_event(frame: &animus_adapters::ModelStreamFrame) -> RuntimeEvent {
    match frame.kind() {
        animus_adapters::ModelStreamFrameKind::Failure => RuntimeEvent::ModelStreamFailed {
            call_id: frame.call_id().clone(),
            reason: terminal_reason(frame),
        },
        animus_adapters::ModelStreamFrameKind::Finish
        | animus_adapters::ModelStreamFrameKind::Cancelled => RuntimeEvent::ModelStreamEnded {
            call_id: frame.call_id().clone(),
            status: frame.kind(),
        },
        animus_adapters::ModelStreamFrameKind::ContentDelta
        | animus_adapters::ModelStreamFrameKind::ThoughtDelta
        | animus_adapters::ModelStreamFrameKind::ToolCallDelta
        | animus_adapters::ModelStreamFrameKind::Usage => RuntimeEvent::ModelStreamFrame {
            frame: frame.clone(),
        },
    }
}

fn terminal_reason(frame: &animus_adapters::ModelStreamFrame) -> String {
    frame
        .body()
        .map_or_else(|| String::from(frame.kind().as_str()), String::from)
}

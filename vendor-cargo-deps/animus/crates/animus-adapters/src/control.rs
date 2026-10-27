//! Transport-neutral agent pause, resume, cancellation, and wait contracts.

/// Maximum accepted byte length for agent-control run and command identities.
pub const MAX_AGENT_CONTROL_ID_BYTES: usize = 128;

const RUN_ID_FIELD: &str = "agent_control_run_id";
const COMMAND_ID_FIELD: &str = "agent_control_command_id";

/// Agent-control identity validation error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AgentControlIdentityError {
    /// The identity was empty.
    Empty {
        /// Stable field label.
        field: &'static str,
    },
    /// The identity exceeded the byte bound.
    TooLong {
        /// Stable field label.
        field: &'static str,
        /// Maximum accepted byte count.
        max_bytes: usize,
        /// Actual byte count.
        actual_bytes: usize,
    },
    /// The identity contained a character outside the safe identity alphabet.
    UnsupportedCharacter {
        /// Stable field label.
        field: &'static str,
    },
}

impl core::fmt::Display for AgentControlIdentityError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Empty { field } => write!(formatter, "{field} must not be empty"),
            Self::TooLong {
                field,
                max_bytes,
                actual_bytes,
            } => write!(
                formatter,
                "{field} must be at most {max_bytes} bytes; got {actual_bytes}"
            ),
            Self::UnsupportedCharacter { field } => {
                write!(formatter, "{field} contains an unsupported character")
            }
        }
    }
}

/// Validated exact runtime identity for agent control.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct AgentControlRunId(alloc::string::String);

impl AgentControlRunId {
    /// Validates and stores an exact runtime identity.
    ///
    /// # Errors
    ///
    /// Returns [`AgentControlIdentityError`] for empty, oversized, or unsafe identities.
    pub fn new(value: &str) -> Result<Self, AgentControlIdentityError> {
        validate_identity(&IdentityDraft {
            field: RUN_ID_FIELD,
            value,
        })?;
        Ok(Self(alloc::string::String::from(value)))
    }

    /// Returns the validated identity.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

/// Validated idempotence identity for one agent-control command.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct AgentControlCommandId(alloc::string::String);

impl AgentControlCommandId {
    /// Validates and stores a command identity.
    ///
    /// # Errors
    ///
    /// Returns [`AgentControlIdentityError`] for empty, oversized, or unsafe identities.
    pub fn new(value: &str) -> Result<Self, AgentControlIdentityError> {
        validate_identity(&IdentityDraft {
            field: COMMAND_ID_FIELD,
            value,
        })?;
        Ok(Self(alloc::string::String::from(value)))
    }

    /// Returns the validated identity.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

/// Desired agent-control action from an external publisher.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AgentControlAction {
    /// Request cooperative pause at the next safe checkpoint.
    Pause,
    /// Return a paused or pause-requested run to runnable state.
    ResumeRun,
    /// Request monotonic cancellation.
    Cancel,
}

impl AgentControlAction {
    /// Returns the stable action label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pause => "pause",
            Self::ResumeRun => "resume_run",
            Self::Cancel => "cancel",
        }
    }
}

/// Typed external command for one exact run revision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentControlCommand {
    run_id: AgentControlRunId,
    command_id: AgentControlCommandId,
    expected_revision: u64,
    action: AgentControlAction,
}

impl AgentControlCommand {
    /// Builds an exact revision-fenced command.
    #[must_use]
    pub const fn new(
        run_id: AgentControlRunId,
        command_id: AgentControlCommandId,
        expected_revision: u64,
        action: AgentControlAction,
    ) -> Self {
        Self {
            run_id,
            command_id,
            expected_revision,
            action,
        }
    }

    /// Returns the target run identity.
    #[must_use]
    pub const fn run_id(&self) -> &AgentControlRunId {
        &self.run_id
    }

    /// Returns the idempotence identity.
    #[must_use]
    pub const fn command_id(&self) -> &AgentControlCommandId {
        &self.command_id
    }

    /// Returns the required current revision.
    #[must_use]
    pub const fn expected_revision(&self) -> u64 {
        self.expected_revision
    }

    /// Returns the requested action.
    #[must_use]
    pub const fn action(&self) -> AgentControlAction {
        self.action
    }
}

/// Current phase of one controlled run.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AgentControlPhase {
    /// The run can start work.
    Running,
    /// A pause is pending until the next safe checkpoint.
    PauseRequested,
    /// The run is blocked at a safe checkpoint.
    Paused,
    /// Cancellation is monotonic and the run cannot resume.
    Cancelling,
    /// The run reached a terminal state.
    Terminal,
}

impl AgentControlPhase {
    /// Returns the stable phase label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::PauseRequested => "pause_requested",
            Self::Paused => "paused",
            Self::Cancelling => "cancelling",
            Self::Terminal => "terminal",
        }
    }
}

/// Safe runtime boundary where control state can block new work.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AgentControlCheckpointKind {
    /// Before loop-fabric evaluation.
    BeforeFabric,
    /// Immediately before model dispatch.
    BeforeModel,
    /// Immediately before tool dispatch.
    BeforeTool,
    /// Immediately before summarizer dispatch.
    BeforeSummarizer,
    /// After an admitted external effect returns and is reduced.
    AfterEffect,
}

impl AgentControlCheckpointKind {
    /// Returns the stable checkpoint label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::BeforeFabric => "before_fabric",
            Self::BeforeModel => "before_model",
            Self::BeforeTool => "before_tool",
            Self::BeforeSummarizer => "before_summarizer",
            Self::AfterEffect => "after_effect",
        }
    }
}

/// Exact safe-checkpoint observation supplied to a control waiter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentControlCheckpoint {
    run_id: AgentControlRunId,
    kind: AgentControlCheckpointKind,
}

impl AgentControlCheckpoint {
    /// Builds a checkpoint for one exact run.
    #[must_use]
    pub const fn new(run_id: AgentControlRunId, kind: AgentControlCheckpointKind) -> Self {
        Self { run_id, kind }
    }

    /// Returns the exact run identity.
    #[must_use]
    pub const fn run_id(&self) -> &AgentControlRunId {
        &self.run_id
    }

    /// Returns the safe-checkpoint class.
    #[must_use]
    pub const fn kind(&self) -> AgentControlCheckpointKind {
        self.kind
    }
}

/// Runtime permit returned by an injected control waiter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AgentControlPermit {
    /// Continue normal runtime work.
    Continue,
    /// Stop before starting new work.
    Cancel,
}

/// Safe control event class published after an admitted transition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AgentControlEventKind {
    /// A pause command was admitted.
    PauseRequested,
    /// The run reached a paused safe checkpoint.
    Paused,
    /// A resume command was admitted.
    ResumeRequested,
    /// A cancellation command was admitted.
    CancellationRequested,
    /// The run became terminal.
    Terminal,
}

impl AgentControlEventKind {
    /// Returns the stable event label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PauseRequested => "pause_requested",
            Self::Paused => "paused",
            Self::ResumeRequested => "resume_requested",
            Self::CancellationRequested => "cancellation_requested",
            Self::Terminal => "terminal",
        }
    }
}

/// Metadata-only event for one admitted control transition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentControlEvent {
    run_id: AgentControlRunId,
    command_id: Option<AgentControlCommandId>,
    revision: u64,
    kind: AgentControlEventKind,
    phase: AgentControlPhase,
    checkpoint: Option<AgentControlCheckpointKind>,
}

impl AgentControlEvent {
    /// Builds a metadata-only event for an admitted external command.
    #[must_use]
    pub fn from_command(
        command: &AgentControlCommand,
        revision: u64,
        kind: AgentControlEventKind,
        phase: AgentControlPhase,
    ) -> Self {
        Self {
            run_id: command.run_id().clone(),
            command_id: Some(command.command_id().clone()),
            revision,
            kind,
            phase,
            checkpoint: None,
        }
    }

    /// Builds a metadata-only paused event for a safe checkpoint.
    #[must_use]
    pub const fn from_checkpoint(
        run_id: AgentControlRunId,
        revision: u64,
        phase: AgentControlPhase,
        checkpoint: AgentControlCheckpointKind,
    ) -> Self {
        Self {
            run_id,
            command_id: None,
            revision,
            kind: AgentControlEventKind::Paused,
            phase,
            checkpoint: Some(checkpoint),
        }
    }

    /// Builds a metadata-only terminal event.
    #[must_use]
    pub const fn terminal(run_id: AgentControlRunId, revision: u64) -> Self {
        Self {
            run_id,
            command_id: None,
            revision,
            kind: AgentControlEventKind::Terminal,
            phase: AgentControlPhase::Terminal,
            checkpoint: None,
        }
    }

    /// Returns the exact run identity.
    #[must_use]
    pub const fn run_id(&self) -> &AgentControlRunId {
        &self.run_id
    }

    /// Returns the command identity when a command caused the transition.
    #[must_use]
    pub const fn command_id(&self) -> Option<&AgentControlCommandId> {
        self.command_id.as_ref()
    }

    /// Returns the resulting state revision.
    #[must_use]
    pub const fn revision(&self) -> u64 {
        self.revision
    }

    /// Returns the event class.
    #[must_use]
    pub const fn kind(&self) -> AgentControlEventKind {
        self.kind
    }

    /// Returns the resulting phase.
    #[must_use]
    pub const fn phase(&self) -> AgentControlPhase {
        self.phase
    }

    /// Returns the checkpoint when one caused the transition.
    #[must_use]
    pub const fn checkpoint(&self) -> Option<AgentControlCheckpointKind> {
        self.checkpoint
    }
}

/// Typed wait failure that prevents implicit dispatch fallback.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AgentControlWaitError {
    /// The checkpoint targets a different run.
    RunMismatch,
    /// The explicit control host closed.
    Closed,
    /// The host synchronization primitive failed.
    SynchronizationFailed,
    /// The injected control service is unavailable.
    Unavailable,
}

impl core::fmt::Display for AgentControlWaitError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str(match self {
            Self::RunMismatch => "agent control run mismatch",
            Self::Closed => "agent control host closed",
            Self::SynchronizationFailed => "agent control synchronization failed",
            Self::Unavailable => "agent control service unavailable",
        })
    }
}

/// Implementation-free host port that blocks at safe checkpoints when required.
pub trait AgentControlWait {
    /// Waits until the exact run can continue or must cancel.
    ///
    /// # Errors
    ///
    /// Returns [`AgentControlWaitError`] when the control host cannot safely decide.
    fn wait_until_runnable(
        &mut self,
        checkpoint: &AgentControlCheckpoint,
    ) -> Result<AgentControlPermit, AgentControlWaitError>;
}

struct IdentityDraft<'a> {
    field: &'static str,
    value: &'a str,
}

fn validate_identity(draft: &IdentityDraft<'_>) -> Result<(), AgentControlIdentityError> {
    if draft.value.is_empty() {
        return Err(AgentControlIdentityError::Empty { field: draft.field });
    }
    if draft.value.len() > MAX_AGENT_CONTROL_ID_BYTES {
        return Err(AgentControlIdentityError::TooLong {
            field: draft.field,
            max_bytes: MAX_AGENT_CONTROL_ID_BYTES,
            actual_bytes: draft.value.len(),
        });
    }
    if !draft.value.bytes().all(is_safe_identity_byte) {
        return Err(AgentControlIdentityError::UnsupportedCharacter { field: draft.field });
    }
    Ok(())
}

const fn is_safe_identity_byte(value: u8) -> bool {
    value.is_ascii_alphanumeric() || matches!(value, b'.' | b'-' | b'_' | b':')
}

#[cfg(test)]
mod tests {
    use super::*;

    const RUN_ID: &str = "run.control-1";
    const COMMAND_ID: &str = "command.pause-1";
    const UNSAFE_ID: &str = "run/private";
    const INITIAL_REVISION: u64 = 0;

    #[test]
    fn valid_control_contract_preserves_exact_typed_fields() {
        // r[verify agent_harness_runtime.agent_control.port.injected]
        // r[verify agent_harness_runtime.agent_control.evidence.safe]
        let run_id = AgentControlRunId::new(RUN_ID).expect("run id must validate");
        let command_id = AgentControlCommandId::new(COMMAND_ID).expect("command id must validate");
        let command = AgentControlCommand::new(
            run_id.clone(),
            command_id.clone(),
            INITIAL_REVISION,
            AgentControlAction::Pause,
        );
        let checkpoint =
            AgentControlCheckpoint::new(run_id.clone(), AgentControlCheckpointKind::BeforeModel);
        let event = AgentControlEvent::from_command(
            &command,
            INITIAL_REVISION,
            AgentControlEventKind::PauseRequested,
            AgentControlPhase::PauseRequested,
        );

        assert_eq!(command.run_id().as_str(), RUN_ID);
        assert_eq!(command.command_id().as_str(), COMMAND_ID);
        assert_eq!(checkpoint.kind(), AgentControlCheckpointKind::BeforeModel);
        assert_eq!(event.kind(), AgentControlEventKind::PauseRequested);
        assert_eq!(event.phase(), AgentControlPhase::PauseRequested);
    }

    #[test]
    fn empty_oversized_and_unsafe_control_identities_fail_closed() {
        // r[verify agent_harness_runtime.agent_control.verification.negative]
        let oversized_bytes = "x".repeat(MAX_AGENT_CONTROL_ID_BYTES.saturating_add(1));

        assert!(matches!(
            AgentControlRunId::new(""),
            Err(AgentControlIdentityError::Empty { .. })
        ));
        assert!(matches!(
            AgentControlCommandId::new(oversized_bytes.as_str()),
            Err(AgentControlIdentityError::TooLong { .. })
        ));
        assert!(matches!(
            AgentControlRunId::new(UNSAFE_ID),
            Err(AgentControlIdentityError::UnsupportedCharacter { .. })
        ));
    }
}

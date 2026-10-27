//! Pure revisioned agent-control state and transition reducers.

#[allow(unused_imports)]
use crate::prelude::*;

const REVISION_INCREMENT: u64 = 1;
const INITIAL_REVISION: u64 = 0;

/// Pure state for one exact controlled run.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentControlState {
    run_id: animus_adapters::AgentControlRunId,
    phase: animus_adapters::AgentControlPhase,
    revision: u64,
    last_command: Option<LastAgentControlCommand>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct LastAgentControlCommand {
    command_id: animus_adapters::AgentControlCommandId,
    expected_revision: u64,
    action: animus_adapters::AgentControlAction,
}

impl AgentControlState {
    /// Builds a running state for one exact run.
    #[must_use]
    pub const fn new(run_id: animus_adapters::AgentControlRunId) -> Self {
        Self {
            run_id,
            phase: animus_adapters::AgentControlPhase::Running,
            revision: INITIAL_REVISION,
            last_command: None,
        }
    }

    /// Returns the exact run identity.
    #[must_use]
    pub const fn run_id(&self) -> &animus_adapters::AgentControlRunId {
        &self.run_id
    }

    /// Returns the current control phase.
    #[must_use]
    pub const fn phase(&self) -> animus_adapters::AgentControlPhase {
        self.phase
    }

    /// Returns the current monotonic revision.
    #[must_use]
    pub const fn revision(&self) -> u64 {
        self.revision
    }
}

/// Result class for one command reduction.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AgentControlCommandDisposition {
    /// The command changed or reaffirmed current desired state.
    Applied,
    /// The exact prior command was observed again without state mutation.
    Duplicate,
}

/// Pure command reduction result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentControlCommandTransition {
    state: AgentControlState,
    disposition: AgentControlCommandDisposition,
    event: Option<animus_adapters::AgentControlEvent>,
}

impl AgentControlCommandTransition {
    /// Returns the resulting state.
    #[must_use]
    pub const fn state(&self) -> &AgentControlState {
        &self.state
    }

    /// Consumes the transition and returns the resulting state.
    #[must_use]
    pub fn into_state(self) -> AgentControlState {
        self.state
    }

    /// Returns the command disposition.
    #[must_use]
    pub const fn disposition(&self) -> AgentControlCommandDisposition {
        self.disposition
    }

    /// Returns the emitted event when state was admitted.
    #[must_use]
    pub const fn event(&self) -> Option<&animus_adapters::AgentControlEvent> {
        self.event.as_ref()
    }
}

/// Decision produced at a safe runtime checkpoint.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AgentControlCheckpointDecision {
    /// The runtime can continue immediately.
    Continue,
    /// The host must block until state changes.
    Wait,
    /// The runtime must stop before new work starts.
    Cancel,
}

/// Pure checkpoint reduction result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentControlCheckpointTransition {
    state: AgentControlState,
    decision: AgentControlCheckpointDecision,
    event: Option<animus_adapters::AgentControlEvent>,
}

impl AgentControlCheckpointTransition {
    /// Returns the resulting state.
    #[must_use]
    pub const fn state(&self) -> &AgentControlState {
        &self.state
    }

    /// Consumes the transition and returns the resulting state.
    #[must_use]
    pub fn into_state(self) -> AgentControlState {
        self.state
    }

    /// Returns the checkpoint decision.
    #[must_use]
    pub const fn decision(&self) -> AgentControlCheckpointDecision {
        self.decision
    }

    /// Returns the emitted event when the checkpoint changed state.
    #[must_use]
    pub const fn event(&self) -> Option<&animus_adapters::AgentControlEvent> {
        self.event.as_ref()
    }
}

/// Pure terminal reduction result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentControlTerminalTransition {
    state: AgentControlState,
    event: Option<animus_adapters::AgentControlEvent>,
}

impl AgentControlTerminalTransition {
    /// Returns the resulting state.
    #[must_use]
    pub const fn state(&self) -> &AgentControlState {
        &self.state
    }

    /// Consumes the transition and returns the resulting state.
    #[must_use]
    pub fn into_state(self) -> AgentControlState {
        self.state
    }

    /// Returns the terminal event when the state changed.
    #[must_use]
    pub const fn event(&self) -> Option<&animus_adapters::AgentControlEvent> {
        self.event.as_ref()
    }
}

/// Pure agent-control reduction error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AgentControlReductionError {
    /// The supplied command or checkpoint targets another run.
    RunMismatch,
    /// The command expected a stale or future revision.
    StaleRevision {
        /// Revision required by the command.
        expected: u64,
        /// Current state revision.
        actual: u64,
    },
    /// One command identity was reused with different command facts.
    CommandIdentityConflict,
    /// Pause or resume was attempted after cancellation became monotonic.
    Cancelling,
    /// A command was attempted after terminal state.
    Terminal,
    /// The revision counter cannot advance safely.
    RevisionOverflow,
}

impl core::fmt::Display for AgentControlReductionError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::RunMismatch => formatter.write_str("agent control run mismatch"),
            Self::StaleRevision { expected, actual } => write!(
                formatter,
                "agent control revision mismatch: expected {expected}, current {actual}"
            ),
            Self::CommandIdentityConflict => {
                formatter.write_str("agent control command identity conflict")
            }
            Self::Cancelling => formatter.write_str("agent control is cancelling"),
            Self::Terminal => formatter.write_str("agent control is terminal"),
            Self::RevisionOverflow => formatter.write_str("agent control revision overflow"),
        }
    }
}

/// Applies one external command to supplied control state.
///
/// # Errors
///
/// Returns [`AgentControlReductionError`] for stale, conflicting, or terminal commands.
pub fn reduce_agent_control_command(
    state: &AgentControlState,
    command: &animus_adapters::AgentControlCommand,
) -> Result<AgentControlCommandTransition, AgentControlReductionError> {
    // r[impl agent_harness_runtime.agent_control.state]
    debug_assert!(!state.run_id().as_str().is_empty());
    debug_assert!(!command.run_id().as_str().is_empty());
    validate_run(state.run_id(), command.run_id())?;
    if let Some(disposition) = duplicate_disposition(state, command)? {
        return Ok(AgentControlCommandTransition {
            state: state.clone(),
            disposition,
            event: None,
        });
    }
    if command.expected_revision() != state.revision() {
        return Err(AgentControlReductionError::StaleRevision {
            expected: command.expected_revision(),
            actual: state.revision(),
        });
    }

    let next_phase = reduce_command_phase(state.phase(), command.action())?;
    let next_revision = increment_revision(state.revision())?;
    let next = AgentControlState {
        run_id: state.run_id.clone(),
        phase: next_phase,
        revision: next_revision,
        last_command: Some(LastAgentControlCommand {
            command_id: command.command_id().clone(),
            expected_revision: command.expected_revision(),
            action: command.action(),
        }),
    };
    let event = command_event(&next, command);
    Ok(AgentControlCommandTransition {
        state: next,
        disposition: AgentControlCommandDisposition::Applied,
        event: Some(event),
    })
}

/// Applies one safe-checkpoint observation to supplied control state.
///
/// # Errors
///
/// Returns [`AgentControlReductionError`] for run mismatch or revision overflow.
pub fn reduce_agent_control_checkpoint(
    state: &AgentControlState,
    checkpoint: &animus_adapters::AgentControlCheckpoint,
) -> Result<AgentControlCheckpointTransition, AgentControlReductionError> {
    // r[impl agent_harness_runtime.agent_control.runtime.gate]
    debug_assert!(!state.run_id().as_str().is_empty());
    debug_assert!(!checkpoint.run_id().as_str().is_empty());
    validate_run(state.run_id(), checkpoint.run_id())?;
    match state.phase() {
        animus_adapters::AgentControlPhase::Running => Ok(checkpoint_transition(
            state.clone(),
            AgentControlCheckpointDecision::Continue,
            None,
        )),
        animus_adapters::AgentControlPhase::PauseRequested => {
            let next_revision = increment_revision(state.revision())?;
            let next = AgentControlState {
                run_id: state.run_id.clone(),
                phase: animus_adapters::AgentControlPhase::Paused,
                revision: next_revision,
                last_command: state.last_command.clone(),
            };
            let event = animus_adapters::AgentControlEvent::from_checkpoint(
                next.run_id.clone(),
                next.revision,
                next.phase,
                checkpoint.kind(),
            );
            Ok(checkpoint_transition(
                next,
                AgentControlCheckpointDecision::Wait,
                Some(event),
            ))
        }
        animus_adapters::AgentControlPhase::Paused => Ok(checkpoint_transition(
            state.clone(),
            AgentControlCheckpointDecision::Wait,
            None,
        )),
        animus_adapters::AgentControlPhase::Cancelling
        | animus_adapters::AgentControlPhase::Terminal => Ok(checkpoint_transition(
            state.clone(),
            AgentControlCheckpointDecision::Cancel,
            None,
        )),
    }
}

/// Marks supplied control state terminal.
///
/// # Errors
///
/// Returns [`AgentControlReductionError::RevisionOverflow`] when revision cannot advance.
pub fn reduce_agent_control_terminal(
    state: &AgentControlState,
) -> Result<AgentControlTerminalTransition, AgentControlReductionError> {
    debug_assert!(!state.run_id().as_str().is_empty());
    debug_assert_eq!(state.run_id(), &state.run_id);
    if state.phase() == animus_adapters::AgentControlPhase::Terminal {
        return Ok(AgentControlTerminalTransition {
            state: state.clone(),
            event: None,
        });
    }
    let next_revision = increment_revision(state.revision())?;
    let next = AgentControlState {
        run_id: state.run_id.clone(),
        phase: animus_adapters::AgentControlPhase::Terminal,
        revision: next_revision,
        last_command: state.last_command.clone(),
    };
    let event = animus_adapters::AgentControlEvent::terminal(next.run_id.clone(), next.revision);
    Ok(AgentControlTerminalTransition {
        state: next,
        event: Some(event),
    })
}

fn duplicate_disposition(
    state: &AgentControlState,
    command: &animus_adapters::AgentControlCommand,
) -> Result<Option<AgentControlCommandDisposition>, AgentControlReductionError> {
    let Some(last) = state.last_command.as_ref() else {
        return Ok(None);
    };
    if &last.command_id != command.command_id() {
        return Ok(None);
    }
    if last.expected_revision == command.expected_revision() && last.action == command.action() {
        return Ok(Some(AgentControlCommandDisposition::Duplicate));
    }
    Err(AgentControlReductionError::CommandIdentityConflict)
}

const fn reduce_command_phase(
    current: animus_adapters::AgentControlPhase,
    action: animus_adapters::AgentControlAction,
) -> Result<animus_adapters::AgentControlPhase, AgentControlReductionError> {
    match (current, action) {
        (animus_adapters::AgentControlPhase::Terminal, _) => {
            Err(AgentControlReductionError::Terminal)
        }
        (
            animus_adapters::AgentControlPhase::Cancelling,
            animus_adapters::AgentControlAction::Pause
            | animus_adapters::AgentControlAction::ResumeRun,
        ) => Err(AgentControlReductionError::Cancelling),
        (
            animus_adapters::AgentControlPhase::Cancelling
            | animus_adapters::AgentControlPhase::Running
            | animus_adapters::AgentControlPhase::PauseRequested
            | animus_adapters::AgentControlPhase::Paused,
            animus_adapters::AgentControlAction::Cancel,
        ) => Ok(animus_adapters::AgentControlPhase::Cancelling),
        (
            animus_adapters::AgentControlPhase::Running,
            animus_adapters::AgentControlAction::Pause,
        ) => Ok(animus_adapters::AgentControlPhase::PauseRequested),
        (
            animus_adapters::AgentControlPhase::PauseRequested
            | animus_adapters::AgentControlPhase::Paused,
            animus_adapters::AgentControlAction::Pause,
        ) => Ok(current),
        (
            animus_adapters::AgentControlPhase::Running
            | animus_adapters::AgentControlPhase::PauseRequested
            | animus_adapters::AgentControlPhase::Paused,
            animus_adapters::AgentControlAction::ResumeRun,
        ) => Ok(animus_adapters::AgentControlPhase::Running),
    }
}

fn command_event(
    state: &AgentControlState,
    command: &animus_adapters::AgentControlCommand,
) -> animus_adapters::AgentControlEvent {
    let kind = match command.action() {
        animus_adapters::AgentControlAction::Pause => {
            animus_adapters::AgentControlEventKind::PauseRequested
        }
        animus_adapters::AgentControlAction::ResumeRun => {
            animus_adapters::AgentControlEventKind::ResumeRequested
        }
        animus_adapters::AgentControlAction::Cancel => {
            animus_adapters::AgentControlEventKind::CancellationRequested
        }
    };
    animus_adapters::AgentControlEvent::from_command(command, state.revision, kind, state.phase)
}

fn checkpoint_transition(
    state: AgentControlState,
    decision: AgentControlCheckpointDecision,
    event: Option<animus_adapters::AgentControlEvent>,
) -> AgentControlCheckpointTransition {
    AgentControlCheckpointTransition {
        state,
        decision,
        event,
    }
}

fn validate_run(
    expected: &animus_adapters::AgentControlRunId,
    actual: &animus_adapters::AgentControlRunId,
) -> Result<(), AgentControlReductionError> {
    if expected == actual {
        Ok(())
    } else {
        Err(AgentControlReductionError::RunMismatch)
    }
}

fn increment_revision(revision: u64) -> Result<u64, AgentControlReductionError> {
    revision
        .checked_add(REVISION_INCREMENT)
        .ok_or(AgentControlReductionError::RevisionOverflow)
}

#[cfg(test)]
mod tests {
    use super::*;

    const RUN_ID: &str = "run.control";
    const OTHER_RUN_ID: &str = "run.other";
    const PAUSE_COMMAND_ID: &str = "command.pause";
    const RESUME_COMMAND_ID: &str = "command.resume";
    const CANCEL_COMMAND_ID: &str = "command.cancel";
    const CONFLICTING_COMMAND_ID: &str = "command.conflict";
    const FIRST_REVISION: u64 = 1;
    const PAUSED_REVISION: u64 = 2;
    const RESUMED_REVISION: u64 = 3;

    #[test]
    fn pause_checkpoint_and_resume_follow_revisioned_state() {
        // r[verify agent_harness_runtime.agent_control.state.pause]
        // r[verify agent_harness_runtime.agent_control.verification.positive]
        let initial = state();
        let paused_requested = reduce_agent_control_command(
            &initial,
            &command(
                &initial,
                PAUSE_COMMAND_ID,
                animus_adapters::AgentControlAction::Pause,
            ),
        )
        .expect("pause must apply");
        assert_eq!(
            paused_requested.state().phase(),
            animus_adapters::AgentControlPhase::PauseRequested
        );
        assert_eq!(paused_requested.state().revision(), FIRST_REVISION);

        let checkpoint = checkpoint(animus_adapters::AgentControlCheckpointKind::BeforeModel);
        let paused = reduce_agent_control_checkpoint(paused_requested.state(), &checkpoint)
            .expect("checkpoint must apply");
        assert_eq!(paused.decision(), AgentControlCheckpointDecision::Wait);
        assert_eq!(
            paused.state().phase(),
            animus_adapters::AgentControlPhase::Paused
        );
        assert_eq!(paused.state().revision(), PAUSED_REVISION);

        let resumed = reduce_agent_control_command(
            paused.state(),
            &command(
                paused.state(),
                RESUME_COMMAND_ID,
                animus_adapters::AgentControlAction::ResumeRun,
            ),
        )
        .expect("resume must apply");
        assert_eq!(
            resumed.state().phase(),
            animus_adapters::AgentControlPhase::Running
        );
        assert_eq!(resumed.state().revision(), RESUMED_REVISION);
        assert_eq!(
            reduce_agent_control_checkpoint(resumed.state(), &checkpoint)
                .expect("running checkpoint must apply")
                .decision(),
            AgentControlCheckpointDecision::Continue
        );
    }

    #[test]
    fn exact_duplicate_is_idempotent_but_conflicting_reuse_fails() {
        // r[verify agent_harness_runtime.agent_control.state.stale]
        let initial = state();
        let pause = command(
            &initial,
            CONFLICTING_COMMAND_ID,
            animus_adapters::AgentControlAction::Pause,
        );
        let applied =
            reduce_agent_control_command(&initial, &pause).expect("first command must apply");
        let duplicate = reduce_agent_control_command(applied.state(), &pause)
            .expect("exact duplicate must be idempotent");
        let conflicting = animus_adapters::AgentControlCommand::new(
            run_id(RUN_ID),
            command_id(CONFLICTING_COMMAND_ID),
            initial.revision(),
            animus_adapters::AgentControlAction::Cancel,
        );

        assert_eq!(
            duplicate.disposition(),
            AgentControlCommandDisposition::Duplicate
        );
        assert_eq!(duplicate.state(), applied.state());
        assert_eq!(duplicate.event(), None);
        assert_eq!(
            reduce_agent_control_command(applied.state(), &conflicting),
            Err(AgentControlReductionError::CommandIdentityConflict)
        );
    }

    #[test]
    fn stale_wrong_run_and_post_cancel_commands_fail_closed() {
        // r[verify agent_harness_runtime.agent_control.concurrency.cancel]
        // r[verify agent_harness_runtime.agent_control.verification.negative]
        let initial = state();
        let stale = animus_adapters::AgentControlCommand::new(
            run_id(RUN_ID),
            command_id(PAUSE_COMMAND_ID),
            FIRST_REVISION,
            animus_adapters::AgentControlAction::Pause,
        );
        let wrong_run = animus_adapters::AgentControlCommand::new(
            run_id(OTHER_RUN_ID),
            command_id(PAUSE_COMMAND_ID),
            initial.revision(),
            animus_adapters::AgentControlAction::Pause,
        );
        assert!(matches!(
            reduce_agent_control_command(&initial, &stale),
            Err(AgentControlReductionError::StaleRevision { .. })
        ));
        assert_eq!(
            reduce_agent_control_command(&initial, &wrong_run),
            Err(AgentControlReductionError::RunMismatch)
        );

        let cancelling = reduce_agent_control_command(
            &initial,
            &command(
                &initial,
                CANCEL_COMMAND_ID,
                animus_adapters::AgentControlAction::Cancel,
            ),
        )
        .expect("cancel must apply");
        assert_eq!(
            reduce_agent_control_command(
                cancelling.state(),
                &command(
                    cancelling.state(),
                    RESUME_COMMAND_ID,
                    animus_adapters::AgentControlAction::ResumeRun,
                ),
            ),
            Err(AgentControlReductionError::Cancelling)
        );
        assert_eq!(
            reduce_agent_control_checkpoint(
                cancelling.state(),
                &checkpoint(animus_adapters::AgentControlCheckpointKind::BeforeTool),
            )
            .expect("cancelling checkpoint must apply")
            .decision(),
            AgentControlCheckpointDecision::Cancel
        );
    }

    #[test]
    fn terminal_state_is_monotonic() {
        // r[verify agent_harness_runtime.agent_control.verification.negative]
        let initial = state();
        let terminal = reduce_agent_control_terminal(&initial).expect("terminal must apply");
        let duplicate_terminal =
            reduce_agent_control_terminal(terminal.state()).expect("terminal must be idempotent");

        assert_eq!(
            terminal.state().phase(),
            animus_adapters::AgentControlPhase::Terminal
        );
        assert!(duplicate_terminal.event().is_none());
        assert_eq!(
            reduce_agent_control_command(
                terminal.state(),
                &command(
                    terminal.state(),
                    PAUSE_COMMAND_ID,
                    animus_adapters::AgentControlAction::Pause,
                ),
            ),
            Err(AgentControlReductionError::Terminal)
        );
    }

    fn state() -> AgentControlState {
        AgentControlState::new(run_id(RUN_ID))
    }

    fn run_id(value: &str) -> animus_adapters::AgentControlRunId {
        animus_adapters::AgentControlRunId::new(value).expect("run id must validate")
    }

    fn command_id(value: &str) -> animus_adapters::AgentControlCommandId {
        animus_adapters::AgentControlCommandId::new(value).expect("command id must validate")
    }

    fn command(
        state: &AgentControlState,
        id: &str,
        action: animus_adapters::AgentControlAction,
    ) -> animus_adapters::AgentControlCommand {
        animus_adapters::AgentControlCommand::new(
            state.run_id().clone(),
            command_id(id),
            state.revision(),
            action,
        )
    }

    fn checkpoint(
        kind: animus_adapters::AgentControlCheckpointKind,
    ) -> animus_adapters::AgentControlCheckpoint {
        animus_adapters::AgentControlCheckpoint::new(run_id(RUN_ID), kind)
    }
}

const DEFAULT_MAX_TURNS_COUNT: usize = 16;
const DEFAULT_MAX_TURNS_POLICY_COUNT: u32 = 16;
const MIN_MAX_TURNS_COUNT: usize = 1;
const MIN_MAX_TURNS_POLICY_COUNT: u32 = 1;
const SUMMARY_FIELD: &str = "summary";
#[cfg(test)]
const MAX_TURNS_FIELD: &str = "max_turns";
#[cfg(test)]
const MAX_TURNS_TOO_LARGE_REASON: &str = "max turns must fit runtime policy budget";

/// Explicit maximum fabric-turn budget.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MaxTurns {
    count: usize,
    #[cfg_attr(not(feature = "steel"), allow(dead_code))]
    policy_count: u32,
}

impl MaxTurns {
    const fn from_valid_count(count: usize, policy_count: u32) -> Self {
        Self {
            count,
            policy_count,
        }
    }

    pub(crate) const fn count(self) -> usize {
        self.count
    }

    #[cfg(feature = "steel")]
    pub(crate) const fn policy_count(self) -> u32 {
        self.policy_count
    }

    #[cfg(test)]
    fn from_count(count: usize) -> Result<Self, RuntimeError> {
        if count < MIN_MAX_TURNS_COUNT {
            return Err(RuntimeError::InvalidConfig {
                field: MAX_TURNS_FIELD,
                reason: "max turns must be at least one",
            });
        }
        let Ok(policy_count) = u32::try_from(count) else {
            return Err(RuntimeError::InvalidConfig {
                field: MAX_TURNS_FIELD,
                reason: MAX_TURNS_TOO_LARGE_REASON,
            });
        };

        Ok(Self {
            count,
            policy_count,
        })
    }
}

/// Default maximum number of fabric turns before the runtime fails closed.
pub const DEFAULT_MAX_TURNS: MaxTurns =
    MaxTurns::from_valid_count(DEFAULT_MAX_TURNS_COUNT, DEFAULT_MAX_TURNS_POLICY_COUNT);

/// Smallest accepted maximum-turn budget.
pub const MIN_MAX_TURNS: MaxTurns =
    MaxTurns::from_valid_count(MIN_MAX_TURNS_COUNT, MIN_MAX_TURNS_POLICY_COUNT);

const _: () = assert!(DEFAULT_MAX_TURNS_COUNT >= MIN_MAX_TURNS_COUNT);
const _: () = assert!(DEFAULT_MAX_TURNS_POLICY_COUNT >= MIN_MAX_TURNS_POLICY_COUNT);

/// Runtime configuration for one agent-loop execution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RuntimeConfig {
    max_turns: MaxTurns,
}

impl RuntimeConfig {
    /// Builds runtime configuration with an explicit non-zero max-turn budget.
    #[must_use]
    pub const fn new(max_turns: MaxTurns) -> Self {
        Self { max_turns }
    }

    /// Builds runtime configuration from a raw turn count for tests.
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeError`] when `max_turns` is zero.
    #[cfg(test)]
    pub(crate) fn from_max_turn_count(max_turns: usize) -> Result<Self, RuntimeError> {
        Ok(Self {
            max_turns: MaxTurns::from_count(max_turns)?,
        })
    }

    /// Returns the maximum number of fabric turns allowed for this run.
    #[must_use]
    pub const fn max_turns(self) -> MaxTurns {
        self.max_turns
    }
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            max_turns: DEFAULT_MAX_TURNS,
        }
    }
}

/// Runtime construction and validation errors raised before a loop can run.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RuntimeError {
    /// A required runtime input field contained no user-visible characters.
    EmptyInput {
        /// Input field name.
        field: &'static str,
    },
    /// A runtime configuration field was invalid.
    InvalidConfig {
        /// Configuration field name.
        field: &'static str,
        /// Human-readable validation reason.
        reason: &'static str,
    },
    /// A valid [`RuntimeInput`] could not be converted into a loop turn.
    LoopTurn {
        /// Human-readable conversion reason.
        message: String,
    },
}

impl core::fmt::Display for RuntimeError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::EmptyInput { field } => {
                write!(formatter, "runtime input {field} must not be empty")
            }
            Self::InvalidConfig { field, reason } => {
                write!(formatter, "runtime config {field} is invalid: {reason}")
            }
            Self::LoopTurn { message } => {
                write!(
                    formatter,
                    "runtime input could not form a loop turn: {message}"
                )
            }
        }
    }
}

impl std::error::Error for RuntimeError {}

/// Typed failure status for a completed runtime report.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RuntimeFailure {
    /// The loop fabric failed to load, evaluate, or return a valid decision.
    Fabric {
        /// Human-readable failure message.
        message: String,
    },
    /// The model adapter rejected or failed a request.
    ModelAdapter {
        /// Human-readable failure message.
        message: String,
    },
    /// The tool adapter rejected or failed a request.
    ToolAdapter {
        /// Human-readable failure message.
        message: String,
    },
    /// Steel policy denied activation before fabric evaluation.
    SteelPolicy {
        /// Human-readable failure message.
        message: String,
    },
    /// Host middleware blocked or failed a guarded boundary.
    Middleware {
        /// Guarded boundary that failed safe.
        boundary: crate::HookBoundary,
        /// Human-readable failure message.
        message: String,
    },
}

impl RuntimeFailure {
    /// Returns the failure message.
    #[must_use]
    pub fn message(&self) -> &str {
        match self {
            Self::Fabric { message }
            | Self::ModelAdapter { message }
            | Self::ToolAdapter { message }
            | Self::SteelPolicy { message }
            | Self::Middleware { message, .. } => message.as_str(),
        }
    }
}

/// Validated input for one runtime-loop execution.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeInput {
    summary: String,
}

impl RuntimeInput {
    /// Builds runtime input from a deterministic summary string.
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeError`] when the summary is empty after trimming.
    pub fn new(summary: &str) -> Result<Self, RuntimeError> {
        if summary.trim().is_empty() {
            return Err(RuntimeError::EmptyInput {
                field: SUMMARY_FIELD,
            });
        }

        Ok(Self {
            summary: String::from(summary),
        })
    }

    /// Builds runtime input from deterministic core state.
    #[must_use]
    pub fn from_state(state: &animus_core::AnimusState) -> Self {
        Self {
            summary: state.summary(),
        }
    }

    /// Returns the summary made available to the loop fabric.
    #[must_use]
    pub fn summary(&self) -> &str {
        self.summary.as_str()
    }
}

/// Final runtime-loop status.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RuntimeStatus {
    /// The loop produced a final response.
    FinalResponse {
        /// Validated final response.
        response: animus_adapters::ModelResponse,
    },
    /// The loop stopped without a final response.
    Stopped,
    /// The runtime reached its configured max-turn budget and failed closed.
    MaxTurnsExceeded {
        /// Configured maximum number of fabric turns.
        max_turns: usize,
    },
    /// The runtime halted because a fabric or adapter boundary failed.
    Failed {
        /// Typed runtime failure.
        failure: RuntimeFailure,
    },
}

/// Complete result returned by one runtime-loop execution.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RunReport {
    events: Vec<crate::RuntimeEvent>,
    status: RuntimeStatus,
}

impl RunReport {
    /// Builds a result from ordered runtime events and final status.
    #[must_use]
    pub fn new(events: Vec<crate::RuntimeEvent>, status: RuntimeStatus) -> Self {
        Self { events, status }
    }

    /// Returns ordered runtime events.
    #[must_use]
    pub fn events(&self) -> &[crate::RuntimeEvent] {
        self.events.as_slice()
    }

    /// Returns the final runtime status.
    #[must_use]
    pub const fn status(&self) -> &RuntimeStatus {
        &self.status
    }

    /// Consumes the result and returns ordered runtime events.
    #[must_use]
    pub fn into_events(self) -> Vec<crate::RuntimeEvent> {
        self.events
    }
}

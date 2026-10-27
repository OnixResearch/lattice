mod boundary;
mod denial;
#[cfg(feature = "steel")]
mod halt;

pub use boundary::{AdapterOutcome, AdapterStep, apply_adapter_outcome};
pub use denial::{complete_with_guard_block, complete_with_guard_failure};
#[cfg(feature = "steel")]
pub use halt::complete_with_steel_policy_denial;

/// Runtime plan before evaluating the next loop-fabric turn.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TurnPlan {
    /// Evaluate the loop fabric with this turn input.
    Evaluate {
        /// State that will consume the fabric decision.
        state: TurnState,
        /// Typed loop-fabric turn input.
        turn: crate::RuntimeLoopTurn,
    },
    /// Runtime execution has completed.
    Complete {
        /// Final runtime report.
        report: crate::RunReport,
    },
}

/// Pending adapter request selected by a validated loop decision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AdapterRequest {
    /// Model adapter request.
    Model {
        /// Validated model request.
        request: animus_adapters::ModelRequest,
    },
    /// Tool adapter request.
    Tool {
        /// Validated tool request.
        request: animus_adapters::ToolRequest,
    },
}

impl AdapterRequest {
    /// Returns the guarded middleware boundary for this adapter request.
    #[must_use]
    pub const fn boundary(&self) -> crate::HookBoundary {
        match self {
            Self::Model { .. } => crate::HookBoundary::ModelDispatch,
            Self::Tool { .. } => crate::HookBoundary::ToolDispatch,
        }
    }
}

/// Step returned after applying a loop decision to pure state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DecisionStep {
    /// A model or tool adapter must be called by the imperative shell.
    Dispatch {
        /// State after recording the requested adapter event.
        state: TurnState,
        /// Adapter request to dispatch.
        request: AdapterRequest,
    },
    /// Runtime execution has completed.
    Complete {
        /// Final runtime report.
        report: crate::RunReport,
    },
}

/// Pure deterministic runtime-loop state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TurnState {
    summary: String,
    turns_completed: usize,
    last_model_message: Option<String>,
    last_tool_output: Option<String>,
    events: Vec<crate::RuntimeEvent>,
}

impl TurnState {
    /// Builds initial turn state from validated runtime input.
    #[must_use]
    pub fn new(input: &crate::RuntimeInput) -> Self {
        Self {
            summary: String::from(input.summary()),
            turns_completed: 0,
            last_model_message: None,
            last_tool_output: None,
            events: vec![
                crate::RuntimeEvent::RuntimeStarted {
                    summary: String::from(input.summary()),
                },
                crate::RuntimeEvent::ContextReplayed {
                    summary: String::from(input.summary()),
                },
                crate::RuntimeEvent::LoopStarted {
                    summary: String::from(input.summary()),
                },
            ],
        }
    }

    /// Returns the input summary.
    #[must_use]
    pub fn summary(&self) -> &str {
        self.summary.as_str()
    }

    /// Returns the number of completed fabric decisions.
    #[must_use]
    const fn turns_completed(&self) -> usize {
        self.turns_completed
    }

    /// Returns the last model message, if any.
    #[must_use]
    pub fn last_model_message(&self) -> Option<&str> {
        self.last_model_message.as_deref()
    }

    /// Returns the last tool output, if any.
    #[must_use]
    pub fn last_tool_output(&self) -> Option<&str> {
        self.last_tool_output.as_deref()
    }

    /// Returns ordered runtime events accumulated so far.
    #[must_use]
    pub fn events(&self) -> &[crate::RuntimeEvent] {
        self.events.as_slice()
    }

    fn next_turn(&self) -> Result<crate::RuntimeLoopTurn, crate::RuntimeError> {
        crate::RuntimeLoopTurn::new(
            self.summary(),
            self.last_model_message(),
            self.last_tool_output(),
        )
        .map_err(|source| crate::RuntimeError::LoopTurn {
            message: source.to_string(),
        })
    }

    fn complete(mut self, status: crate::RuntimeStatus) -> crate::RunReport {
        let terminal = crate::RuntimeTerminalKind::from_status(&status);
        self.events
            .push(crate::RuntimeEvent::Shutdown { status: terminal });
        crate::RunReport::new(self.events, status)
    }

    fn with_event(mut self, event: crate::RuntimeEvent) -> Self {
        self.events.push(event);
        self
    }
}

/// Plans the next fabric turn or max-turn completion without side effects.
#[must_use]
pub fn plan_next_turn(config: crate::RuntimeConfig, state: TurnState) -> TurnPlan {
    if state.turns_completed() >= config.max_turns().count() {
        let max_turns = config.max_turns().count();
        let state = state.with_event(crate::RuntimeEvent::MaxTurnsExceeded { max_turns });
        return TurnPlan::Complete {
            report: state.complete(crate::RuntimeStatus::MaxTurnsExceeded { max_turns }),
        };
    }

    let turn_index = state.turns_completed();
    match state.next_turn() {
        Ok(turn) => TurnPlan::Evaluate {
            state: state.with_event(crate::RuntimeEvent::TurnStarted { turn_index }),
            turn,
        },
        Err(error) => TurnPlan::Complete {
            report: complete_with_fabric_error(state, error.to_string()),
        },
    }
}

/// Applies a validated loop decision to pure state.
#[must_use]
pub fn apply_decision(state: TurnState, decision: crate::RuntimeLoopDecision) -> DecisionStep {
    match decision {
        crate::RuntimeLoopDecision::CallModel { request } => {
            let turn_index = state.turns_completed();
            let state = state
                .with_event(crate::RuntimeEvent::LoopDecision {
                    turn_index,
                    decision: crate::LoopDecisionKind::Model,
                })
                .with_event(crate::RuntimeEvent::ModelRequested {
                    request: request.clone(),
                });
            DecisionStep::Dispatch {
                state,
                request: AdapterRequest::Model { request },
            }
        }
        crate::RuntimeLoopDecision::CallTool { request } => {
            let turn_index = state.turns_completed();
            let state = state
                .with_event(crate::RuntimeEvent::LoopDecision {
                    turn_index,
                    decision: crate::LoopDecisionKind::Tool,
                })
                .with_event(crate::RuntimeEvent::ToolRequested {
                    request: request.clone(),
                });
            DecisionStep::Dispatch {
                state,
                request: AdapterRequest::Tool { request },
            }
        }
        crate::RuntimeLoopDecision::Respond { response } => {
            let turn_index = state.turns_completed();
            let state = state
                .with_event(crate::RuntimeEvent::LoopDecision {
                    turn_index,
                    decision: crate::LoopDecisionKind::Respond,
                })
                .with_event(crate::RuntimeEvent::FinalResponse {
                    response: response.clone(),
                });
            DecisionStep::Complete {
                report: state.complete(crate::RuntimeStatus::FinalResponse { response }),
            }
        }
        crate::RuntimeLoopDecision::CompletionCandidate { candidate } => {
            complete_candidate(state, candidate)
        }
        crate::RuntimeLoopDecision::Stop => {
            let turn_index = state.turns_completed();
            let state = state
                .with_event(crate::RuntimeEvent::LoopDecision {
                    turn_index,
                    decision: crate::LoopDecisionKind::Stop,
                })
                .with_event(crate::RuntimeEvent::Stopped);
            DecisionStep::Complete {
                report: state.complete(crate::RuntimeStatus::Stopped),
            }
        }
    }
}

fn complete_candidate(
    state: TurnState,
    candidate: crate::RuntimeCompletionCandidate,
) -> DecisionStep {
    let turn_index = state.turns_completed();
    let request_hash = blake3::hash(candidate.request().as_bytes())
        .to_hex()
        .to_string();
    let state = state
        .with_event(crate::RuntimeEvent::LoopDecision {
            turn_index,
            decision: crate::LoopDecisionKind::CompletionCandidate,
        })
        .with_event(crate::RuntimeEvent::CompletionCandidateRequested { request_hash });
    DecisionStep::Complete {
        report: state.complete(crate::RuntimeStatus::CompletionCandidate { candidate }),
    }
}

/// Completes a runtime report after a fabric failure.
#[must_use]
pub fn complete_with_fabric_error(state: TurnState, message: String) -> crate::RunReport {
    let state = state.with_event(crate::RuntimeEvent::FabricError {
        message: message.clone(),
    });
    state.complete(crate::RuntimeStatus::Failed {
        failure: crate::RuntimeFailure::Fabric { message },
    })
}

pub(crate) fn complete_with_agent_control_stop(
    state: TurnState,
    reason: &'static str,
) -> crate::RunReport {
    let state = state.with_event(crate::RuntimeEvent::Cancelled {
        reason: String::from(reason),
    });
    state.complete(crate::RuntimeStatus::Stopped)
}

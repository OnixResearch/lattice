const CHOREGRAPH_NODE_PREFIX: &str = "lattice-node/";
const MAX_EVENTS_PER_OUTCOME: usize = 3;
const MAX_SUSPENSION_EVENTS: usize = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdapterErrorCode {
    GraphAdmission,
    UnknownNode,
    TransitionDenied,
    UnexpectedIntent,
    StateMismatch,
    ProfileMismatch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdapterError {
    pub code: AdapterErrorCode,
    pub message: String,
}

impl std::fmt::Display for AdapterError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for AdapterError {}

pub fn events_before_suspension(
    completed: &[crate::workflow::gate::model::PersistedNodeOutcome],
) -> Vec<crate::workflow::gate::model::ApprovalGraphEvent> {
    let event_reservation = completed.len().saturating_mul(MAX_EVENTS_PER_OUTCOME);
    let mut events = Vec::with_capacity(event_reservation);
    for node in completed {
        events.push(event(node.node_id, crate::workflow::gate::model::ApprovalGraphAction::MarkReady));
        match node.state {
            crate::workflow::gate::model::PersistedNodeState::Completed => {
                events.push(event(node.node_id, crate::workflow::gate::model::ApprovalGraphAction::Start));
                events.push(event(node.node_id, crate::workflow::gate::model::ApprovalGraphAction::Complete));
            }
            crate::workflow::gate::model::PersistedNodeState::Failed => {
                events.push(event(node.node_id, crate::workflow::gate::model::ApprovalGraphAction::Start));
                events.push(event(node.node_id, crate::workflow::gate::model::ApprovalGraphAction::Fail));
            }
            crate::workflow::gate::model::PersistedNodeState::Cancelled => {
                events.push(event(node.node_id, crate::workflow::gate::model::ApprovalGraphAction::Cancel))
            }
        }
    }
    debug_assert!(events.len() <= event_reservation);
    debug_assert!(events.iter().all(|event| event.suspension_class.is_none()));
    events
}

// r[impl lattice.workflow_approval.suspension]
pub fn apply_suspend(
    workflow: &crate::engine::workflow::Workflow,
    request: &mut crate::workflow::gate::model::ApprovalRequest,
) -> Result<(), AdapterError> {
    require_profile(request)?;
    request.graph_events.reserve(MAX_SUSPENSION_EVENTS);
    let machine = approval_machine(workflow)?;
    let mut state = replay(&machine, &request.graph_events)?;
    let gate = machine_node_id(&machine, request.node_id)?;
    if state.state(&gate) != Some(choregraph_core::LocalState::Ready) {
        let ready = crate::workflow::gate::model::ApprovalGraphEvent {
            node_id: request.node_id,
            action: crate::workflow::gate::model::ApprovalGraphAction::MarkReady,
            suspension_class: None,
            suspension_reference: None,
        };
        state = apply_event(&machine, &state, &ready)?;
        request.graph_events.push(ready);
    }
    let suspend = crate::workflow::gate::model::ApprovalGraphEvent {
        node_id: request.node_id,
        action: crate::workflow::gate::model::ApprovalGraphAction::Suspend,
        suspension_class: Some(crate::workflow::gate::model::CHOREGRAPH_SUSPENSION_CLASS.to_owned()),
        suspension_reference: Some(request.suspension_reference.clone()),
    };
    let state = apply_event(&machine, &state, &suspend)?;
    if state.state(&gate) != Some(choregraph_core::LocalState::Suspended) {
        return Err(error(
            AdapterErrorCode::StateMismatch,
            "Choregraph did not produce the required suspended gate state",
        ));
    }
    request.graph_events.push(suspend);
    debug_assert_eq!(state.state(&gate), Some(choregraph_core::LocalState::Suspended));
    debug_assert!(
        request
            .graph_events
            .last()
            .is_some_and(|event| event.action == crate::workflow::gate::model::ApprovalGraphAction::Suspend)
    );
    Ok(())
}

// r[impl lattice.workflow_approval.resume]
pub fn apply_resume_signal(
    workflow: &crate::engine::workflow::Workflow,
    request: &mut crate::workflow::gate::model::ApprovalRequest,
) -> Result<(), AdapterError> {
    require_profile(request)?;
    request.graph_events.reserve(1);
    let machine = approval_machine(workflow)?;
    let state = replay(&machine, &request.graph_events)?;
    let gate = machine_node_id(&machine, request.node_id)?;
    if state.state(&gate) != Some(choregraph_core::LocalState::Suspended) {
        return Err(error(
            AdapterErrorCode::StateMismatch,
            "persisted approval graph does not reconstruct the suspended gate state",
        ));
    }
    let resume = crate::workflow::gate::model::ApprovalGraphEvent {
        node_id: request.node_id,
        action: crate::workflow::gate::model::ApprovalGraphAction::ResumeSignal,
        suspension_class: Some(crate::workflow::gate::model::CHOREGRAPH_SUSPENSION_CLASS.to_owned()),
        suspension_reference: Some(request.suspension_reference.clone()),
    };
    let resumed = apply_event(&machine, &state, &resume)?;
    if resumed.state(&gate) != Some(choregraph_core::LocalState::Ready) {
        return Err(error(
            AdapterErrorCode::StateMismatch,
            "Choregraph did not produce the required resumed gate state",
        ));
    }
    request.graph_events.push(resume);
    debug_assert_eq!(resumed.state(&gate), Some(choregraph_core::LocalState::Ready));
    debug_assert!(
        request
            .graph_events
            .last()
            .is_some_and(|event| event.action == crate::workflow::gate::model::ApprovalGraphAction::ResumeSignal)
    );
    Ok(())
}

pub fn apply_execution_outcomes(
    workflow: &crate::engine::workflow::Workflow,
    request: &mut crate::workflow::gate::model::ApprovalRequest,
    outcomes: &[crate::workflow::gate::model::PersistedNodeOutcome],
) -> Result<(), AdapterError> {
    require_profile(request)?;
    let event_reservation = outcomes.len().saturating_mul(MAX_EVENTS_PER_OUTCOME);
    request.graph_events.reserve(event_reservation);
    let initial_event_count = request.graph_events.len();
    let machine = approval_machine(workflow)?;
    let mut state = replay(&machine, &request.graph_events)?;
    for outcome in outcomes {
        let machine_node = machine_node_id(&machine, outcome.node_id)?;
        let target = match outcome.state {
            crate::workflow::gate::model::PersistedNodeState::Completed => choregraph_core::LocalState::Completed,
            crate::workflow::gate::model::PersistedNodeState::Failed => choregraph_core::LocalState::Failed,
            crate::workflow::gate::model::PersistedNodeState::Cancelled => choregraph_core::LocalState::Cancelled,
        };
        if state.state(&machine_node) == Some(target) {
            continue;
        }
        if outcome.state == crate::workflow::gate::model::PersistedNodeState::Cancelled {
            let event = event(outcome.node_id, crate::workflow::gate::model::ApprovalGraphAction::Cancel);
            state = apply_event(&machine, &state, &event)?;
            request.graph_events.push(event);
        } else {
            if state.state(&machine_node) == Some(choregraph_core::LocalState::Pending) {
                let ready = event(outcome.node_id, crate::workflow::gate::model::ApprovalGraphAction::MarkReady);
                state = apply_event(&machine, &state, &ready)?;
                request.graph_events.push(ready);
            }
            if state.state(&machine_node) == Some(choregraph_core::LocalState::Ready) {
                let start = event(outcome.node_id, crate::workflow::gate::model::ApprovalGraphAction::Start);
                state = apply_event(&machine, &state, &start)?;
                request.graph_events.push(start);
            }
            let terminal_action = match outcome.state {
                crate::workflow::gate::model::PersistedNodeState::Completed => {
                    crate::workflow::gate::model::ApprovalGraphAction::Complete
                }
                crate::workflow::gate::model::PersistedNodeState::Failed => {
                    crate::workflow::gate::model::ApprovalGraphAction::Fail
                }
                crate::workflow::gate::model::PersistedNodeState::Cancelled => {
                    return Err(error(
                        AdapterErrorCode::StateMismatch,
                        "cancelled outcome reached the non-cancelled transition path",
                    ));
                }
            };
            let terminal = event(outcome.node_id, terminal_action);
            state = apply_event(&machine, &state, &terminal)?;
            request.graph_events.push(terminal);
        }
        if state.state(&machine_node) != Some(target) {
            return Err(error(
                AdapterErrorCode::StateMismatch,
                "Choregraph execution outcome did not reach its required terminal state",
            ));
        }
    }
    debug_assert!(request.graph_events.len() >= initial_event_count);
    debug_assert!(request.graph_events.len() <= initial_event_count.saturating_add(event_reservation));
    Ok(())
}

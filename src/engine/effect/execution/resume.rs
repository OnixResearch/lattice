//! Session restoration from a durable executor checkpoint.

impl super::Session {
    pub fn new_with_checkpoint(
        workflow: &crate::engine::workflow::Workflow,
        run: &super::RunContext,
        checkpoint: &[crate::engine::executor::StepResult],
    ) -> Result<Self, super::Error> {
        let mut session = Self::new(workflow, run)?;
        let mut states = session
            .machine
            .node_ids()
            .iter()
            .cloned()
            .map(|node_id| (node_id, choregraph_core::model::semantics::LocalState::Pending))
            .collect::<std::collections::BTreeMap<_, _>>();
        let mut observed = std::collections::BTreeSet::new();
        for step in checkpoint {
            if !observed.insert(step.node_id) {
                return Err(super::Error::InvalidCheckpoint);
            }
            let expected = super::node_reference(step.node_id);
            let machine_node = session
                .machine
                .node_ids()
                .iter()
                .find(|node_id| node_id.as_str() == expected)
                .cloned()
                .ok_or(super::Error::InvalidCheckpoint)?;
            states.insert(machine_node, restored_local_state(&step.status)?);
        }
        debug_assert_eq!(observed.len(), checkpoint.len(), "checkpoint steps are unique");
        debug_assert_eq!(states.len(), session.machine.node_ids().len(), "restoration only replaces machine nodes");
        session.state = choregraph_core::model::automaton::MachineState::new(states);
        Ok(session)
    }
}

/// Only terminal executor outcomes can be restored into the machine state.
fn restored_local_state(
    status: &crate::engine::executor::NodeStatus,
) -> Result<choregraph_core::model::semantics::LocalState, super::Error> {
    match status {
        crate::engine::executor::NodeStatus::Completed { .. } => {
            Ok(choregraph_core::model::semantics::LocalState::Completed)
        }
        crate::engine::executor::NodeStatus::Failed { .. } => Ok(choregraph_core::model::semantics::LocalState::Failed),
        crate::engine::executor::NodeStatus::Skipped => Ok(choregraph_core::model::semantics::LocalState::Cancelled),
        crate::engine::executor::NodeStatus::Pending | crate::engine::executor::NodeStatus::Running => {
            Err(super::Error::InvalidCheckpoint)
        }
    }
}

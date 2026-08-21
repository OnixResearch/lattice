//! Run-scoped Choregraph authority composition for product workflow execution.

mod provider;

pub use provider::RunContextProvider;
pub use provider::StoreAttempts;
pub use provider::UnavailableAttempts;
pub use provider::UnavailableRunContextSource;

const CHOREGRAPH_NODE_PREFIX: &str = "lattice-node/";
const EFFECT_DECLARATION_PREFIX: &str = "lattice-effect-";
const EXPECTED_EFFECT_BINDING_COUNT: usize = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Binding(super::authority::BindingError),
    Lowering(String),
    MachineIdentity,
    WorkflowRevision,
    TransitionDenied,
    UnexpectedBindings,
    InvalidCheckpoint,
}

impl std::fmt::Display for Error {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Binding(error) => write!(formatter, "invalid effect identity: {error:?}"),
            Self::Lowering(error) => write!(formatter, "Choregraph V2 lowering failed: {error}"),
            Self::MachineIdentity => write!(formatter, "canonical Choregraph machine identity failed"),
            Self::WorkflowRevision => write!(formatter, "workflow revision identity failed"),
            Self::TransitionDenied => write!(formatter, "Choregraph denied the product execution transition"),
            Self::UnexpectedBindings => write!(formatter, "Choregraph returned an unexpected concrete intent set"),
            Self::InvalidCheckpoint => write!(formatter, "persisted workflow checkpoint is not admissible"),
        }
    }
}

impl std::error::Error for Error {}

impl From<super::authority::BindingError> for Error {
    fn from(error: super::authority::BindingError) -> Self {
        Self::Binding(error)
    }
}

pub struct RunContext {
    run: super::authority::RunIdentity,
    authority: super::authority::AuthorityIdentity,
    scope: super::authority::CollaborationScopeBinding,
    dispatch: std::sync::Arc<super::dispatch::Shell>,
}

impl RunContext {
    #[must_use]
    pub fn new(
        run: super::authority::RunIdentity,
        authority: super::authority::AuthorityIdentity,
        scope: super::authority::CollaborationScopeBinding,
        dispatch: std::sync::Arc<super::dispatch::Shell>,
    ) -> Self {
        Self {
            run,
            authority,
            scope,
            dispatch,
        }
    }
}

pub struct Session {
    machine: choregraph_core::AdmittedMachine,
    state: choregraph_core::MachineState,
    context: super::choregraph::TransitionContext,
    dispatch: std::sync::Arc<super::dispatch::Shell>,
    ready_nodes: std::collections::BTreeSet<u64>,
    started_nodes: std::collections::BTreeSet<u64>,
}

impl Session {
    pub fn new(workflow: &crate::engine::workflow::Workflow, run: &RunContext) -> Result<Self, Error> {
        let portable = crate::workflow_format::workflow_to_portable(workflow);
        let lowered = crate::workflow_format::choregraph::lower_portable(&portable)
            .map_err(|error| Error::Lowering(error.to_string()))?;
        let artifact = choregraph_kamacite::build_concrete_machine_artifact(&lowered.machine)
            .map_err(|_| Error::MachineIdentity)?;
        let machine_reference = format!("{}:{}", artifact.identity.algorithm, artifact.identity.hex);
        let revision = crate::orchestration::workflow_revision(workflow).map_err(|_| Error::WorkflowRevision)?;
        let workflow_reference = format!("blake3:{}", revision.content_hash);
        let context = super::choregraph::TransitionContext {
            machine: super::authority::MachineIdentity::new(machine_reference)?,
            workflow_revision: super::authority::WorkflowRevisionIdentity::new(workflow_reference)?,
            run: run.run.clone(),
            authority: run.authority.clone(),
            scope: run.scope.clone(),
        };
        let state = lowered.machine.initial_state();
        Ok(Self {
            machine: lowered.machine,
            state,
            context,
            dispatch: std::sync::Arc::clone(&run.dispatch),
            ready_nodes: std::collections::BTreeSet::new(),
            started_nodes: std::collections::BTreeSet::new(),
        })
    }

    pub fn new_with_checkpoint(
        workflow: &crate::engine::workflow::Workflow,
        run: &RunContext,
        checkpoint: &[crate::engine::executor::StepResult],
    ) -> Result<Self, Error> {
        let mut session = Self::new(workflow, run)?;
        let mut states = session
            .machine
            .node_ids()
            .iter()
            .cloned()
            .map(|node_id| (node_id, choregraph_core::LocalState::Pending))
            .collect::<std::collections::BTreeMap<_, _>>();
        let mut observed = std::collections::BTreeSet::new();
        for step in checkpoint {
            if !observed.insert(step.node_id) {
                return Err(Error::InvalidCheckpoint);
            }
            let expected = node_reference(step.node_id);
            let machine_node = session
                .machine
                .node_ids()
                .iter()
                .find(|node_id| node_id.as_str() == expected)
                .cloned()
                .ok_or(Error::InvalidCheckpoint)?;
            let state = match step.status {
                crate::engine::executor::NodeStatus::Completed { .. } => choregraph_core::LocalState::Completed,
                crate::engine::executor::NodeStatus::Failed { .. } => choregraph_core::LocalState::Failed,
                crate::engine::executor::NodeStatus::Skipped => choregraph_core::LocalState::Cancelled,
                crate::engine::executor::NodeStatus::Pending | crate::engine::executor::NodeStatus::Running => {
                    return Err(Error::InvalidCheckpoint);
                }
            };
            states.insert(machine_node, state);
        }
        session.state = choregraph_core::MachineState::new(states);
        Ok(session)
    }

    pub fn ready_frontier(&self) -> Result<Vec<crate::engine::workflow::NodeId>, Error> {
        let ready = choregraph_core::ready_nodes(&self.machine, &self.state).map_err(|_| Error::TransitionDenied)?;
        ready
            .into_iter()
            .map(|node_id| {
                node_id
                    .as_str()
                    .strip_prefix(CHOREGRAPH_NODE_PREFIX)
                    .and_then(|value| value.parse::<u64>().ok())
                    .map(crate::engine::workflow::NodeId)
                    .ok_or(Error::TransitionDenied)
            })
            .collect()
    }

    pub fn mark_ready(&mut self, node_id: crate::engine::workflow::NodeId) -> Result<(), Error> {
        let event = choregraph_core::ConcreteEvent::new(node_reference(node_id), choregraph_core::Action::MarkReady);
        self.apply_without_bindings(&event)?;
        self.ready_nodes.insert(node_id.0);
        Ok(())
    }

    pub fn start_pure(&mut self, node_id: crate::engine::workflow::NodeId) -> Result<(), Error> {
        let event = choregraph_core::ConcreteEvent::new(node_reference(node_id), choregraph_core::Action::Start);
        self.apply_without_bindings(&event)?;
        self.ready_nodes.remove(&node_id.0);
        self.started_nodes.insert(node_id.0);
        Ok(())
    }

    pub fn bind_effect(
        &mut self,
        node_id: crate::engine::workflow::NodeId,
        payload_reference: &str,
    ) -> Result<super::authority::IntentBinding, Error> {
        let event = super::choregraph::requested_event(super::choregraph::EventRequest {
            declaration_key: format!("{EFFECT_DECLARATION_PREFIX}{}", node_id.0),
            node_id: node_reference(node_id),
            source_node_id: node_id.0.to_string(),
            action: choregraph_core::Action::Start,
            request_ref: super::runtime::dispatch_request_identity(payload_reference),
            payload_ref: payload_reference.to_owned(),
        });
        let bindings = self.apply(&event)?;
        if bindings.len() != EXPECTED_EFFECT_BINDING_COUNT {
            return Err(Error::UnexpectedBindings);
        }
        let binding = bindings.into_iter().next().ok_or(Error::UnexpectedBindings)?;
        self.ready_nodes.remove(&node_id.0);
        self.started_nodes.insert(node_id.0);
        Ok(binding)
    }

    pub fn finish(
        &mut self,
        node_id: crate::engine::workflow::NodeId,
        status: &crate::engine::executor::NodeStatus,
    ) -> Result<(), Error> {
        if self.ready_nodes.contains(&node_id.0) {
            return self.cancel(node_id);
        }
        if !self.started_nodes.contains(&node_id.0) {
            return Ok(());
        }
        let action = match status {
            crate::engine::executor::NodeStatus::Completed { .. } => choregraph_core::Action::Complete,
            crate::engine::executor::NodeStatus::Skipped => choregraph_core::Action::Cancel,
            crate::engine::executor::NodeStatus::Failed { .. }
            | crate::engine::executor::NodeStatus::Pending
            | crate::engine::executor::NodeStatus::Running => choregraph_core::Action::Fail,
        };
        let event = choregraph_core::ConcreteEvent::new(node_reference(node_id), action);
        self.apply_without_bindings(&event)?;
        self.started_nodes.remove(&node_id.0);
        Ok(())
    }

    pub fn cancel(&mut self, node_id: crate::engine::workflow::NodeId) -> Result<(), Error> {
        let event = choregraph_core::ConcreteEvent::new(node_reference(node_id), choregraph_core::Action::Cancel);
        self.apply_without_bindings(&event)?;
        self.ready_nodes.remove(&node_id.0);
        self.started_nodes.remove(&node_id.0);
        Ok(())
    }

    #[must_use]
    pub fn dispatch(&self) -> &super::dispatch::Shell {
        self.dispatch.as_ref()
    }

    #[must_use]
    pub fn dispatch_handle(&self) -> std::sync::Arc<super::dispatch::Shell> {
        std::sync::Arc::clone(&self.dispatch)
    }

    fn apply_without_bindings(&mut self, event: &choregraph_core::ConcreteEvent) -> Result<(), Error> {
        let bindings = self.apply(event)?;
        if !bindings.is_empty() {
            return Err(Error::UnexpectedBindings);
        }
        Ok(())
    }

    fn apply(&mut self, event: &choregraph_core::ConcreteEvent) -> Result<Vec<super::authority::IntentBinding>, Error> {
        match super::choregraph::transition(&self.machine, &self.state, event, &self.context) {
            super::choregraph::TransitionResult::Accepted { next, bindings } => {
                self.state = next;
                Ok(bindings)
            }
            super::choregraph::TransitionResult::Denied => Err(Error::TransitionDenied),
        }
    }
}

fn node_reference(node_id: crate::engine::workflow::NodeId) -> String {
    format!("{CHOREGRAPH_NODE_PREFIX}{}", node_id.0)
}

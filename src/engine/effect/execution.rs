//! Run-scoped Choregraph authority composition for product workflow execution.

mod provider;
mod resume;

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
    AuthoritySource,
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
            Self::AuthoritySource => write!(formatter, "operator effect authority is unavailable or invalid"),
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
    machine: choregraph_core::model::automaton::AdmittedMachine,
    state: choregraph_core::model::automaton::MachineState,
    context: super::choregraph::TransitionContext,
    dispatch: std::sync::Arc<super::dispatch::Shell>,
    ready_nodes: std::collections::BTreeSet<u64>,
    started_nodes: std::collections::BTreeSet<u64>,
    construction_effects:
        std::collections::BTreeMap<u64, choregraph_nickel::construction::model::projection::ExpandedEffect>,
}

impl Session {
    pub fn new(workflow: &crate::engine::workflow::Workflow, run: &RunContext) -> Result<Self, Error> {
        let (machine, construction_effects) = match crate::authoring::construction::admit_runtime(workflow)
            .map_err(|error| Error::Lowering(error.into()))?
        {
            Some(admitted) => {
                let effects =
                    admitted.projection.effects.iter().map(|effect| (effect.node_id, effect.clone())).collect();
                (admitted.machine, effects)
            }
            None => {
                if !workflow.control_dependencies.is_empty() {
                    return Err(Error::Lowering("control dependencies require v3 construction admission".into()));
                }
                let portable = crate::workflow_format::workflow_to_portable(workflow);
                let lowered = crate::workflow_format::choregraph::lower_portable(&portable)
                    .map_err(|error| Error::Lowering(error.to_string()))?;
                (lowered.machine, std::collections::BTreeMap::new())
            }
        };
        let machine_reference = canonical_machine_identity(&machine)?;
        let revision = crate::orchestration::workflow_revision(workflow).map_err(|_| Error::WorkflowRevision)?;
        let workflow_reference = format!("blake3:{}", revision.content_hash);
        let context = super::choregraph::TransitionContext {
            machine: machine_reference,
            workflow_revision: super::authority::WorkflowRevisionIdentity::new(workflow_reference)?,
            run: run.run.clone(),
            authority: run.authority.clone(),
            scope: run.scope.clone(),
        };
        let state = machine.initial_state();
        Ok(Self {
            machine,
            state,
            context,
            dispatch: std::sync::Arc::clone(&run.dispatch),
            ready_nodes: std::collections::BTreeSet::new(),
            started_nodes: std::collections::BTreeSet::new(),
            construction_effects,
        })
    }

    pub fn ready_frontier(&self) -> Result<Vec<crate::engine::workflow::NodeId>, Error> {
        let ready =
            choregraph_core::analytics::ready_nodes(&self.machine, &self.state).map_err(|_| Error::TransitionDenied)?;
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
        let event = choregraph_core::concrete::event::ConcreteEvent::new(
            node_reference(node_id),
            choregraph_core::model::semantics::Action::MarkReady,
        );
        self.apply_without_bindings(&event)?;
        self.ready_nodes.insert(node_id.0);
        Ok(())
    }

    pub fn start_pure(&mut self, node_id: crate::engine::workflow::NodeId) -> Result<(), Error> {
        let event = choregraph_core::concrete::event::ConcreteEvent::new(
            node_reference(node_id),
            choregraph_core::model::semantics::Action::Start,
        );
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
        let request_ref = super::runtime::dispatch_request_identity(payload_reference);
        let event = if self.construction_effects.is_empty() {
            super::choregraph::requested_event(super::choregraph::EventRequest {
                declaration_key: format!("{EFFECT_DECLARATION_PREFIX}{}", node_id.0),
                node_id: node_reference(node_id),
                source_node_id: node_id.0.to_string(),
                action: choregraph_core::model::semantics::Action::Start,
                request_ref,
                payload_ref: payload_reference.to_owned(),
            })
        } else {
            let effect = self.construction_effects.get(&node_id.0).ok_or(Error::UnexpectedBindings)?;
            let references = super::choregraph::InvocationReferences {
                request_ref,
                payload_ref: payload_reference.to_owned(),
            };
            super::choregraph::requested_construction_event(effect, references).ok_or(Error::UnexpectedBindings)?
        };
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
            crate::engine::executor::NodeStatus::Completed { .. } => {
                choregraph_core::model::semantics::Action::Complete
            }
            crate::engine::executor::NodeStatus::Skipped => choregraph_core::model::semantics::Action::Cancel,
            crate::engine::executor::NodeStatus::Failed { .. }
            | crate::engine::executor::NodeStatus::Pending
            | crate::engine::executor::NodeStatus::Running => choregraph_core::model::semantics::Action::Fail,
        };
        let event = choregraph_core::concrete::event::ConcreteEvent::new(node_reference(node_id), action);
        self.apply_without_bindings(&event)?;
        self.started_nodes.remove(&node_id.0);
        Ok(())
    }

    pub fn cancel(&mut self, node_id: crate::engine::workflow::NodeId) -> Result<(), Error> {
        let event = choregraph_core::concrete::event::ConcreteEvent::new(
            node_reference(node_id),
            choregraph_core::model::semantics::Action::Cancel,
        );
        self.apply_without_bindings(&event)?;
        self.ready_nodes.remove(&node_id.0);
        self.started_nodes.remove(&node_id.0);
        Ok(())
    }

    /// Existing admitted run provenance, not an authority grant.
    pub(crate) fn run_identity(&self) -> &super::authority::RunIdentity {
        &self.context.run
    }

    #[must_use]
    pub fn dispatch(&self) -> &super::dispatch::Shell {
        self.dispatch.as_ref()
    }

    #[must_use]
    pub fn dispatch_handle(&self) -> std::sync::Arc<super::dispatch::Shell> {
        std::sync::Arc::clone(&self.dispatch)
    }

    fn apply_without_bindings(&mut self, event: &choregraph_core::concrete::event::ConcreteEvent) -> Result<(), Error> {
        let bindings = self.apply(event)?;
        if !bindings.is_empty() {
            return Err(Error::UnexpectedBindings);
        }
        Ok(())
    }

    fn apply(
        &mut self,
        event: &choregraph_core::concrete::event::ConcreteEvent,
    ) -> Result<Vec<super::authority::IntentBinding>, Error> {
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

/// Canonical product machine identity shared by execution and read-only inspection.
pub(crate) fn canonical_machine_identity(
    machine: &choregraph_core::model::automaton::AdmittedMachine,
) -> Result<super::authority::MachineIdentity, Error> {
    let artifact = choregraph_kamacite::concrete::conversion::build_concrete_machine_artifact(machine)
        .map_err(|_| Error::MachineIdentity)?;
    super::authority::MachineIdentity::new(format!("{}:{}", artifact.identity.algorithm, artifact.identity.hex))
        .map_err(Error::from)
}

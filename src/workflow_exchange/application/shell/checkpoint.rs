#![allow(
    unknown_lints,
    assertion_density,
    non_trait_imports,
    path_segment_repetition,
    reason = "the handler's checkpoint restore keeps admission, adoption and worker start visible"
)]

//! Application checkpoint binding, offer and restore of the exchange handler (Lattice #12).

use super::ApplicationError;
use super::ApplicationHandler;
use super::DurableExecutionState;
use super::WorkflowRoute;

/// Start facts of a worker: defaults for a fresh dispatch, carried values for an adoption.
#[derive(Default)]
pub(super) struct WorkerStart {
    pub(super) seed: crate::checkpoint::HookSeed,
    pub(super) restored: Vec<crate::engine::executor::StepResult>,
    pub(super) is_cancelled: bool,
}

impl ApplicationHandler {
    /// Bind exactly one route to its opaque C1 profile identity and declared checkpoint.
    pub fn with_application_checkpoint(
        mut self,
        binding: crate::checkpoint::CheckpointBinding,
    ) -> Result<Self, ApplicationError> {
        crate::checkpoint::validate_binding(&binding).map_err(|_| ApplicationError::Profile)?;
        let matching = self.profile.routes().iter().filter(|route| binds_route(&binding, route)).count();
        if matching != 1 {
            return Err(ApplicationError::Route);
        }
        self.checkpoint = Some(binding);
        Ok(self)
    }

    /// `Unavailable` unless a bound route's profile declares a checkpoint.
    #[must_use]
    pub fn checkpoint_offer(&self) -> crate::checkpoint::CheckpointOffer {
        crate::checkpoint::checkpoint_offer(self.checkpoint.as_ref())
    }

    // r[impl lattice.application_checkpoint.restore]
    // r[impl lattice.application_checkpoint.unavailable]
    /// Adopt a committed checkpoint under a Kiln admission into this handler's fresh store.
    ///
    /// Nothing runs here: the serving exchange starts the adopted execution
    /// ([`Self::resume_adopted`]). Uncertain nodes are adopted as `Unknown`, so nothing is ever
    /// scheduled for them. An identical restart is idempotent only while its adoption is present.
    pub fn restore_checkpoint(
        &mut self,
        source: &crate::checkpoint::CheckpointRoot,
        request: &crate::checkpoint::RestartRequest,
        now_unix_ms: u64,
    ) -> Result<crate::checkpoint::RestartReceipt, crate::checkpoint::Refusal> {
        use crate::checkpoint::Refusal;
        let binding = self
            .checkpoint
            .clone()
            .filter(|binding| binding.checkpoint != crate::checkpoint::CheckpointProfile::None)
            .ok_or(Refusal::CheckpointUnavailable)?;
        let admitted = match crate::checkpoint::admit_from_root(source, request, &self.store, &binding, now_unix_ms)? {
            crate::checkpoint::Admission::Idempotent(receipt) => {
                self.store
                    .load_workflow_exchange(&request.execution)
                    .map_err(|_| Refusal::Store)?
                    .ok_or(Refusal::Store)?;
                return Ok(receipt);
            }
            crate::checkpoint::Admission::Fresh(admitted) => admitted,
        };
        let (route, workflow, revision) =
            self.instantiated_route(&admitted.workflow_request).ok_or(Refusal::CrossedRoute)?;
        let adopted = crate::checkpoint::adopt(
            &crate::checkpoint::AdoptionTarget {
                store: &self.store,
                source,
                route: &route,
                binding: &binding,
                workflow: &workflow,
                execution_workflow_revision: &revision,
                handler_profile_identity: &self.profile_identity,
            },
            &admitted,
        )?;
        Ok(adopted.adoption.receipt)
    }

    // r[impl lattice.application_checkpoint.resume]
    /// Start every adopted execution that never started, publishing under this handler's binding.
    ///
    /// The worker's `Queued` to `Running` transition is a compare-and-swap in the single-writer
    /// store, so exactly one process runs an adopted execution. An adoption that cannot be
    /// resumed here is recorded `Unknown`.
    pub fn resume_adopted(&mut self) -> Result<(), ApplicationError> {
        let Some(binding) = self.checkpoint.clone() else {
            return Ok(());
        };
        let root = crate::checkpoint::CheckpointRoot::new(binding.root.clone());
        for adoption in self.store.list_workflow_exchange_adoptions().map_err(|_| ApplicationError::Store)? {
            let record = self.store.load_workflow_exchange(&adoption.execution).map_err(|_| ApplicationError::Store)?;
            let Some(record) = record.filter(|record| record.state == DurableExecutionState::Queued) else {
                continue;
            };
            let resumable = (record.handler_profile_identity == self.profile_identity)
                .then(|| self.instantiated_route(&record.request))
                .flatten()
                .filter(|(route, _workflow, revision)| {
                    binds_route(&binding, route) && *revision == record.workflow_revision
                })
                .zip(crate::checkpoint::adopted_steps(&root, &adoption).ok());
            let Some(((route, workflow, _revision), restored)) = resumable else {
                self.mark_execution_unknown(adoption.execution.as_str());
                continue;
            };
            let start = WorkerStart {
                seed: crate::checkpoint::HookSeed {
                    incarnation: adoption.receipt.restarted_incarnation,
                    consumed: adoption.consumed,
                    cancellation_requested: adoption.cancellation_requested,
                    completed: adoption.completed_nodes,
                },
                restored,
                is_cancelled: record.cancellation_requested,
            };
            self.spawn_worker(record, route, workflow, start);
        }
        Ok(())
    }

    /// The route a request selects, with its saved workflow validated and instantiated.
    fn instantiated_route(
        &self,
        request: &lattice_workflow_contract::WorkflowRequest,
    ) -> Option<(WorkflowRoute, crate::engine::workflow::Workflow, lattice_workflow_contract::Identity)> {
        let route = self.profile.select_route(request).ok()?.clone();
        let workflow = self.store.load_workflow(&route.workflow_name).ok()??;
        let facts = super::super::worker::workflow_facts(&workflow).ok()?;
        crate::workflow_exchange::validate_route(&route, request, &facts).ok()?;
        let (workflow, revision) = crate::workflow_exchange::instantiate_workflow(&route, request, workflow).ok()?;
        Some((route, workflow, revision))
    }

    /// The handler's checkpoint binding when it binds `route`.
    pub(super) fn route_checkpoint(&self, route: &WorkflowRoute) -> Option<crate::checkpoint::CheckpointBinding> {
        self.checkpoint.clone().filter(|binding| binds_route(binding, route))
    }
}

fn binds_route(binding: &crate::checkpoint::CheckpointBinding, route: &WorkflowRoute) -> bool {
    route.graph_identity == binding.graph_identity && route.workflow_revision == binding.workflow_revision
}

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
        let matching = self.profile.routes.iter().filter(|route| binds_route(&binding, route)).count();
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
    /// Restore a committed checkpoint under a Kiln admission into this handler's fresh store.
    ///
    /// Uncertain nodes are adopted as `Unknown` and nothing is scheduled; otherwise the remaining
    /// nodes run on a worker that publishes this incarnation's own checkpoints.
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
            crate::checkpoint::Admission::Idempotent(receipt) => return Ok(receipt),
            crate::checkpoint::Admission::Fresh(admitted) => admitted,
        };
        let route = crate::workflow_exchange::select_route(&self.profile, &admitted.workflow_request)
            .map_err(|_| Refusal::CrossedRoute)?
            .clone();
        let workflow = self
            .store
            .load_workflow(&route.workflow_name)
            .map_err(|_| Refusal::Store)?
            .ok_or(Refusal::CrossedRoute)?;
        let facts = super::super::worker::workflow_facts(&workflow).map_err(|_| Refusal::CrossedWorkflowRevision)?;
        crate::workflow_exchange::validate_route(&route, &admitted.workflow_request, &facts)
            .map_err(|_| Refusal::CrossedRoute)?;
        let (workflow, revision) =
            crate::workflow_exchange::instantiate_workflow(&route, &admitted.workflow_request, workflow)
                .map_err(|_| Refusal::CrossedRoute)?;
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
        let receipt = adopted.adoption.receipt.clone();
        if adopted.record.state == DurableExecutionState::Queued {
            let seed = crate::checkpoint::HookSeed {
                incarnation: receipt.restarted_incarnation,
                consumed: adopted.adoption.consumed,
                cancellation_requested: adopted.adoption.cancellation_requested,
                completed: admitted.outcomes.iter().map(|(node, _outcome)| node.clone()).collect(),
            };
            let is_cancelled = adopted.record.cancellation_requested;
            self.spawn_worker(adopted.record, route, workflow, WorkerStart {
                seed,
                restored: adopted.steps,
                is_cancelled,
            });
        }
        Ok(receipt)
    }

    /// The handler's checkpoint binding when it binds `route`.
    pub(super) fn route_checkpoint(&self, route: &WorkflowRoute) -> Option<crate::checkpoint::CheckpointBinding> {
        self.checkpoint.clone().filter(|binding| binds_route(binding, route))
    }
}

fn binds_route(binding: &crate::checkpoint::CheckpointBinding, route: &WorkflowRoute) -> bool {
    route.graph_identity == binding.graph_identity && route.workflow_revision == binding.workflow_revision
}

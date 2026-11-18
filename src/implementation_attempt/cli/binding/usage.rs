//! Host-owned `Scope::Attempt` usage observation with frozen binding provenance.
//!
//! It is persisted before the attempt result is published. Reserved claims stay
//! labelled `reserved`; host duration appears only when both endpoints were
//! observed; model and device usage stay unknown.

// r[impl lattice.execution_binding.usage_provenance.scenario.bound]
// r[impl lattice.execution_binding.usage_provenance.scenario.shared_allocation]

/// Host clock readings around the owned worker step; either may be missing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in super::super) struct WorkerWindow {
    pub(in super::super) started_epoch_ms: Option<u64>,
    pub(in super::super) finished_epoch_ms: Option<u64>,
}

impl WorkerWindow {
    fn duration_millis(self) -> Option<u64> {
        self.finished_epoch_ms?.checked_sub(self.started_epoch_ms?)
    }
}

/// Identities frozen before spawn for the attempt usage observation.
#[derive(Debug, Clone)]
pub(in super::super) struct WorkerReceipt {
    admitted: lattice_core::execution_binding::admission::bound::Admitted,
    request_identity: String,
}

impl WorkerReceipt {
    pub(in super::super) fn new(
        admitted: lattice_core::execution_binding::admission::bound::Admitted,
        request_identity: String,
    ) -> Self {
        Self {
            admitted,
            request_identity,
        }
    }

    pub(in super::super) fn observation(&self, window: WorkerWindow) -> crate::orchestration::CommandWorkerObservation {
        let duration_millis = window.duration_millis();
        let (provenance, coverage) = if duration_millis.is_some() {
            (
                crate::orchestration::UsageProvenance::HostMeasured,
                lattice_core::execution_binding::projection::accounting::WorkerCoverage::HostMeasured,
            )
        } else {
            (
                crate::orchestration::UsageProvenance::Unknown,
                lattice_core::execution_binding::projection::accounting::WorkerCoverage::Unknown,
            )
        };
        let attempt = &self.admitted.attempt;
        crate::orchestration::CommandWorkerObservation {
            schema: crate::orchestration::USAGE_OBSERVATION_SCHEMA.to_owned(),
            scope: crate::orchestration::Scope::Attempt,
            workflow_revision: attempt.envelope_identity.clone(),
            instance_id: attempt.work_id.clone(),
            dispatch_id: self.request_identity.clone(),
            run_id: 0,
            node_id: 0,
            invocation_id: self.request_identity.clone(),
            worker_profile: self.admitted.profile_revision.clone(),
            attempt_id: attempt.attempt_id.clone(),
            agent_invocations: None,
            model_requests: None,
            subagent_calls: None,
            tokens: None,
            cost_microunits: None,
            duration_millis,
            provenance,
            complete: false,
            accounting: None,
            execution_binding: Some(lattice_core::execution_binding::projection::accounting::provenance(
                &self.admitted,
                &self.request_identity,
                coverage,
            )),
        }
    }

    /// Persist before the attempt result; replay of identical content is idempotent.
    pub(in super::super) fn persist(&self, store: &crate::store::Store, window: WorkerWindow) -> Result<(), String> {
        store.save_command_worker_usage(&self.observation(window)).map_err(|error| error.to_string())
    }

    pub(in super::super) fn request_identity(&self) -> &str {
        &self.request_identity
    }
}

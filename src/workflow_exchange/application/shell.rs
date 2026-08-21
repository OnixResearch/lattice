#![allow(
    unknown_lints,
    assertion_density,
    non_trait_imports,
    path_segment_repetition,
    reason = "the imperative handler keeps durable admission and transport operation dispatch visible"
)]

use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::thread::JoinHandle;

use lattice_workflow_contract::ControlAction;
use lattice_workflow_contract::Identity;
use lattice_workflow_contract::WorkflowAccepted;
use lattice_workflow_contract::WorkflowRequest;
use lattice_workflow_transport::ExchangeSuccess;
use lattice_workflow_transport::HandlerError;

use super::AdmissionResolution;
use super::ApplicationError;
use super::DurableExecutionState;
use super::ExecutionRecord;
use super::HandlerProfile;
use super::WorkflowRoute;

const WORKER_THREAD_PREFIX: &str = "lattice-exchange";

struct Worker {
    cancellation: Arc<AtomicBool>,
    handle: JoinHandle<()>,
}

pub struct ApplicationHandler {
    store: crate::store::Store,
    profile: HandlerProfile,
    profile_identity: Identity,
    agent_runtime: crate::agent_node::AgentRuntimeConfig,
    workers: BTreeMap<String, Worker>,
}

impl ApplicationHandler {
    pub fn new(
        store: crate::store::Store,
        profile: HandlerProfile,
        agent_runtime: crate::agent_node::AgentRuntimeConfig,
    ) -> Result<Self, ApplicationError> {
        super::validate_handler_profile(&profile).map_err(|_| ApplicationError::Profile)?;
        let profile_identity = super::handler_profile_identity(&profile).map_err(|_| ApplicationError::Profile)?;
        Ok(Self {
            store,
            profile,
            profile_identity,
            agent_runtime,
            workers: BTreeMap::new(),
        })
    }

    #[must_use]
    pub const fn profile(&self) -> &HandlerProfile {
        &self.profile
    }

    pub fn wait_for_workers(&mut self) {
        let workers = std::mem::take(&mut self.workers);
        for (execution_identity, worker) in workers {
            if worker.handle.join().is_err() {
                self.mark_execution_unknown(&execution_identity);
            }
        }
    }

    pub(super) fn refresh_workers(&mut self) {
        let finished = self
            .workers
            .iter()
            .filter(|(_identity, worker)| worker.handle.is_finished())
            .map(|(identity, _worker)| identity.clone())
            .collect::<Vec<_>>();
        for identity in finished {
            if let Some(worker) = self.workers.remove(&identity)
                && worker.handle.join().is_err()
            {
                self.mark_execution_unknown(&identity);
            }
        }
    }

    fn mark_execution_unknown(&self, execution_identity: &str) {
        let Ok(identity) = Identity::new(execution_identity.to_string()) else {
            return;
        };
        let Ok(Some(current)) = self.store.load_workflow_exchange(&identity) else {
            return;
        };
        let Ok(next) = super::mark_unknown(&current) else {
            return;
        };
        if self.store.save_workflow_exchange_transition(&current, &next).is_err() {
            tracing::warn!("workflow exchange handler could not persist unknown state");
        }
    }

    pub(super) fn dispatch(&mut self, request: &WorkflowRequest) -> Result<WorkflowAccepted, HandlerError> {
        if let Some(existing) = self
            .store
            .load_workflow_exchange_by_request(&request.identity)
            .map_err(|_| HandlerError::Unavailable)?
        {
            if existing.request == *request {
                return Ok(existing.accepted);
            }
            return Err(HandlerError::Rejected);
        }
        let maximum_active =
            usize::try_from(self.profile.maximum_active_executions).map_err(|_| HandlerError::Unavailable)?;
        if self.workers.len() >= maximum_active {
            return Err(HandlerError::Unavailable);
        }
        let route = super::select_route(&self.profile, request).map_err(|_| HandlerError::Rejected)?.clone();
        let workflow = self
            .store
            .load_workflow(&route.workflow_name)
            .map_err(|_| HandlerError::Unavailable)?
            .ok_or(HandlerError::NotFound)?;
        let facts = super::worker::workflow_facts(&workflow).map_err(|_| HandlerError::Rejected)?;
        super::validate_route(&route, request, &facts).map_err(|_| HandlerError::Rejected)?;
        let (workflow, execution_workflow_revision) =
            super::instantiate_workflow(&route, request, workflow).map_err(|_| HandlerError::Rejected)?;
        let resolution = self
            .store
            .admit_workflow_exchange(
                request.clone(),
                &route,
                execution_workflow_revision,
                self.profile_identity.clone(),
            )
            .map_err(map_admission_error)?;
        match resolution {
            AdmissionResolution::Existing(record) => Ok(record.accepted),
            AdmissionResolution::Inserted(record) => {
                let accepted = record.accepted.clone();
                self.spawn_worker(record, route, workflow);
                Ok(accepted)
            }
        }
    }

    fn spawn_worker(
        &mut self,
        record: ExecutionRecord,
        route: WorkflowRoute,
        workflow: crate::engine::workflow::Workflow,
    ) {
        let execution_identity = record.accepted.execution_identity.as_str().to_string();
        let cancellation = Arc::new(AtomicBool::new(false));
        let worker_cancellation = Arc::clone(&cancellation);
        let store = self.store.clone();
        let agent_runtime = self.agent_runtime.clone();
        let worker_name = format!("{WORKER_THREAD_PREFIX}-{}", record.run_id);
        let spawn = std::thread::Builder::new().name(worker_name).spawn(move || {
            super::worker::run(super::worker::WorkerInput {
                store,
                admitted: record,
                authority_mode: route.effect_authority,
                workflow,
                agent_runtime,
                cancellation: worker_cancellation,
            });
        });
        match spawn {
            Ok(handle) => {
                self.workers.insert(execution_identity, Worker { cancellation, handle });
            }
            Err(_) => self.mark_execution_unknown(&execution_identity),
        }
    }

    pub(super) fn recover_dispatch(&self, request: &WorkflowRequest) -> Result<WorkflowAccepted, HandlerError> {
        let record = self
            .store
            .load_workflow_exchange_by_request(&request.identity)
            .map_err(|_| HandlerError::Unavailable)?
            .ok_or(HandlerError::NotFound)?;
        if record.request != *request {
            return Err(HandlerError::Rejected);
        }
        Ok(record.accepted)
    }

    fn linked_record(
        &mut self,
        request: &WorkflowRequest,
        accepted: &WorkflowAccepted,
    ) -> Result<ExecutionRecord, HandlerError> {
        let mut record = self
            .store
            .load_workflow_exchange(&accepted.execution_identity)
            .map_err(|_| HandlerError::Unavailable)?
            .ok_or(HandlerError::NotFound)?;
        if record.request != *request || record.accepted != *accepted {
            return Err(HandlerError::Rejected);
        }
        let execution_identity = accepted.execution_identity.as_str();
        if !record.state.is_terminal()
            && record.state != DurableExecutionState::Unknown
            && !self.workers.contains_key(execution_identity)
        {
            let unknown = super::mark_unknown(&record).map_err(|_| HandlerError::Rejected)?;
            self.store
                .save_workflow_exchange_transition(&record, &unknown)
                .map_err(|_| HandlerError::Unavailable)?;
            record = unknown;
        }
        Ok(record)
    }

    pub(super) fn observe(
        &mut self,
        profile: &lattice_workflow_contract::CapabilityProfile,
        request: &WorkflowRequest,
        accepted: &WorkflowAccepted,
    ) -> Result<lattice_workflow_contract::ExecutionObservation, HandlerError> {
        let record = self.linked_record(request, accepted)?;
        let observation = super::observation(&record).map_err(|_| HandlerError::Rejected)?;
        lattice_workflow_contract::validate_observation(profile, request, accepted, &observation)
            .map_err(|_| HandlerError::Rejected)?;
        Ok(observation)
    }

    pub(super) fn control(
        &mut self,
        profile: &lattice_workflow_contract::CapabilityProfile,
        request: &WorkflowRequest,
        accepted: &WorkflowAccepted,
        current_sequence: u64,
        control: &lattice_workflow_contract::ControlRequest,
    ) -> Result<lattice_workflow_contract::ExecutionObservation, HandlerError> {
        let current = self.linked_record(request, accepted)?;
        if current.sequence != current_sequence || current_sequence != control.expected_sequence {
            return Err(HandlerError::Rejected);
        }
        if control.action != ControlAction::Cancel {
            return Err(HandlerError::Rejected);
        }
        let worker = self.workers.get(accepted.execution_identity.as_str()).ok_or(HandlerError::Unavailable)?;
        let next = super::request_cancellation(&current, current_sequence).map_err(|_| HandlerError::Rejected)?;
        self.store
            .save_workflow_exchange_transition(&current, &next)
            .map_err(|_| HandlerError::Unavailable)?;
        worker.cancellation.store(true, Ordering::SeqCst);
        let observation = super::observation(&next).map_err(|_| HandlerError::Rejected)?;
        lattice_workflow_contract::validate_observation(profile, request, accepted, &observation)
            .map_err(|_| HandlerError::Rejected)?;
        Ok(observation)
    }

    pub(super) fn read_log(
        &mut self,
        profile: &lattice_workflow_contract::CapabilityProfile,
        request: &WorkflowRequest,
        accepted: &WorkflowAccepted,
        log_read: &lattice_workflow_contract::LogReadRequest,
    ) -> Result<ExchangeSuccess, HandlerError> {
        let record = self.linked_record(request, accepted)?;
        let observation = super::observation(&record).map_err(|_| HandlerError::Rejected)?;
        lattice_workflow_contract::validate_observation(profile, request, accepted, &observation)
            .map_err(|_| HandlerError::Rejected)?;
        let bytes =
            super::retained_log(&record, &log_read.node_id, log_read.stream, log_read.cursor, log_read.maximum_bytes)
                .map_err(|_| HandlerError::NotFound)?
                .to_vec();
        Ok(ExchangeSuccess::Log { observation, bytes })
    }
}

fn map_admission_error(error: crate::store::StoreError) -> HandlerError {
    match error {
        crate::store::StoreError::Conflict(_) => HandlerError::Rejected,
        _ => HandlerError::Unavailable,
    }
}

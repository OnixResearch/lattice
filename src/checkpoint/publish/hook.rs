#![allow(
    unknown_lints,
    ambiguous_params,
    ambient_clock,
    assertion_density,
    clone_then_borrow,
    fallible_int_fallback,
    non_trait_imports,
    numeric_units,
    path_segment_repetition,
    raw_arithmetic_overflow,
    sentinel_fallback,
    unbounded_collection_growth,
    reason = "the node-boundary hook keeps the C8 commit order and its effect fence explicit"
)]

//! The node-boundary hook: publication at completed-node boundaries and before effect reservations.

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::sync::Arc;

use lattice_workflow_contract::Identity;

use super::CheckpointRoot;
use super::local_runtime;
use super::snapshot_workspace;
use crate::checkpoint::CheckpointBinding;
use crate::checkpoint::CheckpointProfile;
use crate::checkpoint::CompletedNode;
use crate::checkpoint::Consumed;
use crate::checkpoint::LocalRuntime;
use crate::checkpoint::Refusal;

const DRAIN_POLL: std::time::Duration = std::time::Duration::from_millis(25);
const NODE_PREFIX: &str = "lattice-node/";

/// Carried state a hook starts from: zero for a first incarnation, the adoption for a restart.
#[derive(Debug, Clone, Default)]
pub struct HookSeed {
    pub incarnation: u32,
    pub consumed: Consumed,
    pub cancellation_requested: bool,
    pub completed: Vec<CompletedNode>,
}

#[derive(Debug, Default)]
struct HookState {
    completed: BTreeMap<u64, CompletedNode>,
    in_flight: BTreeSet<u64>,
    /// Finished nodes whose outputs carry sensitive fragments: never persisted, never replayed.
    sensitive: BTreeSet<u64>,
    last: Option<Identity>,
}

// r[impl lattice.application_checkpoint.publish]
/// Publishes a C8 manifest at every completed-node boundary and, synchronously, before every
/// effect reservation. Constructed only for routes whose C1 profile declares a checkpoint.
pub struct NodeBoundaryHook {
    root: CheckpointRoot,
    binding: CheckpointBinding,
    local: LocalRuntime,
    store: crate::store::Store,
    execution: Identity,
    seed: HookSeed,
    started: std::time::Instant,
    state: std::sync::Mutex<HookState>,
    changed: std::sync::Condvar,
}

impl std::fmt::Debug for NodeBoundaryHook {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("NodeBoundaryHook")
            .field("root", &self.root)
            .field("execution", &self.execution)
            .field("incarnation", &self.seed.incarnation)
            .finish_non_exhaustive()
    }
}

impl NodeBoundaryHook {
    /// `None` unless `binding` declares a checkpoint profile: the hook is then a no-op by absence.
    pub fn for_route(
        store: &crate::store::Store,
        binding: Option<&CheckpointBinding>,
        record: &crate::workflow_exchange::ExecutionRecord,
        seed: HookSeed,
    ) -> Result<Option<Arc<Self>>, Refusal> {
        let Some(binding) = binding.filter(|binding| binding.checkpoint != CheckpointProfile::None) else {
            return Ok(None);
        };
        crate::checkpoint::validate_binding(binding)?;
        let root = CheckpointRoot::new(binding.root.clone());
        let request_bytes = serde_json::to_vec(&record.request).map_err(|_| Refusal::Encoding)?;
        root.put_request(&record.request.identity, &request_bytes)?;
        let completed = seed.completed.iter().map(|node| (node.workflow_node_id, node.clone())).collect();
        Ok(Some(Arc::new(Self {
            root,
            binding: binding.clone(),
            local: local_runtime(binding.environment.clone())?,
            store: store.clone(),
            execution: record.accepted.execution_identity.clone(),
            seed,
            started: std::time::Instant::now(),
            state: std::sync::Mutex::new(HookState {
                completed,
                ..HookState::default()
            }),
            changed: std::sync::Condvar::new(),
        })))
    }

    #[must_use]
    pub const fn root(&self) -> &CheckpointRoot {
        &self.root
    }

    /// The last manifest this hook committed.
    #[must_use]
    pub fn last_published(&self) -> Option<Identity> {
        self.state.lock().ok().and_then(|state| state.last.clone())
    }

    /// Drain executor observations on a dedicated thread until the run drops its sender.
    pub fn observe_progress(
        self: &Arc<Self>,
        mut receiver: tokio::sync::mpsc::Receiver<crate::engine::executor::NodeProgress>,
    ) -> std::io::Result<std::thread::JoinHandle<()>> {
        let hook = Arc::clone(self);
        std::thread::Builder::new().name("lattice-checkpoint".to_string()).spawn(move || {
            while let Some(progress) = receiver.blocking_recv() {
                hook.observe(&progress);
            }
        })
    }

    /// Record one finished node and publish the completed-node boundary.
    pub fn observe(&self, progress: &crate::engine::executor::NodeProgress) {
        let crate::engine::executor::NodeProgress::Finished { node_id, result, .. } = progress else {
            return;
        };
        let Ok(mut state) = self.state.lock() else {
            return;
        };
        state.in_flight.remove(&node_id.0);
        match crate::workflow::gate::shell::checkpoint_from_results(std::slice::from_ref(result)) {
            Ok(outcomes) => {
                for outcome in outcomes {
                    match self.persist_outcome(&outcome) {
                        Ok(node) => {
                            state.completed.insert(node.workflow_node_id, node);
                        }
                        Err(_) => {
                            state.sensitive.insert(outcome.node_id.0);
                        }
                    }
                }
            }
            Err(_) if is_terminal(&result.status) => {
                state.sensitive.insert(node_id.0);
            }
            Err(_) => {}
        }
        if self.publish(&mut state).is_err() {
            tracing::warn!("application checkpoint boundary publication failed");
        }
        self.changed.notify_all();
    }

    fn persist_outcome(&self, outcome: &crate::workflow::gate::PersistedNodeOutcome) -> Result<CompletedNode, Refusal> {
        let record = self.current_record()?;
        let (bytes, identity) = crate::checkpoint::node_outputs_object(outcome.node_id.0, outcome)?;
        self.root.put_node_object(&identity, &bytes)?;
        crate::checkpoint::completed_node(&record, outcome, identity)
    }

    fn current_record(&self) -> Result<crate::workflow_exchange::ExecutionRecord, Refusal> {
        self.store
            .load_workflow_exchange(&self.execution)
            .map_err(|_| Refusal::Store)?
            .ok_or(Refusal::Store)
    }

    fn publish(&self, state: &mut HookState) -> Result<Identity, Refusal> {
        let record = self.current_record()?;
        let workspace = match &self.binding.workspace {
            Some(workspace) => snapshot_workspace(&self.root, workspace)?,
            None => Vec::new(),
        };
        let elapsed = u64::try_from(self.started.elapsed().as_millis()).unwrap_or(u64::MAX);
        let consumed = Consumed {
            elapsed_ms: self.seed.consumed.elapsed_ms.saturating_add(elapsed),
            ..self.seed.consumed
        };
        let uncertain = state.in_flight.union(&state.sensitive).copied().collect::<BTreeSet<_>>();
        let manifest = crate::checkpoint::build_manifest(&crate::checkpoint::ManifestInput {
            record: &record,
            binding: &self.binding,
            producing_incarnation: self.seed.incarnation,
            local: &self.local,
            completed: &state.completed,
            uncertain: &uncertain,
            workspace: &workspace,
            consumed,
            cancellation_requested: self.seed.cancellation_requested,
        })?;
        let (bytes, identity) = crate::checkpoint::canonical(crate::checkpoint::MANIFEST_SCHEMA, &manifest)?;
        if state.last.as_ref() == Some(&identity) {
            return Ok(identity);
        }
        self.root.put_manifest(&identity, &bytes)?;
        self.root.commit(&identity)?;
        state.last = Some(identity.clone());
        Ok(identity)
    }

    // r[impl lattice.application_checkpoint.fence]
    fn fence(&self, workflow_node_id: u64) -> Result<(), Refusal> {
        let mut state = self.state.lock().map_err(|_| Refusal::Io)?;
        if self.root.drain_requested() {
            while !state.in_flight.is_empty() {
                state = self.changed.wait_timeout(state, DRAIN_POLL).map_err(|_| Refusal::Io)?.0;
            }
            let quiescent = self.publish(&mut state)?;
            self.root.acknowledge_drain(&quiescent)?;
            while self.root.drain_requested() {
                state = self.changed.wait_timeout(state, DRAIN_POLL).map_err(|_| Refusal::Io)?.0;
            }
        }
        if state.in_flight.insert(workflow_node_id) && self.publish(&mut state).is_err() {
            state.in_flight.remove(&workflow_node_id);
            return Err(Refusal::Io);
        }
        Ok(())
    }
}

impl crate::workflow_exchange::ReservationFence for NodeBoundaryHook {
    fn admit(
        &self,
        binding: &crate::engine::effect::authority::IntentBinding,
    ) -> Result<(), crate::engine::effect::dispatch::Error> {
        let workflow_node_id = binding
            .node
            .as_str()
            .strip_prefix(NODE_PREFIX)
            .and_then(|id| id.parse::<u64>().ok())
            .ok_or(crate::engine::effect::dispatch::Error::AttemptPersistence)?;
        self.fence(workflow_node_id).map_err(|_| crate::engine::effect::dispatch::Error::AttemptPersistence)
    }
}

const fn is_terminal(status: &crate::engine::executor::NodeStatus) -> bool {
    !matches!(status, crate::engine::executor::NodeStatus::Pending | crate::engine::executor::NodeStatus::Running)
}

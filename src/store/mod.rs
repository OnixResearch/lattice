//! redb-backed persistent storage.
//!
//! Tables:
//!  - workflows: name → serialized Workflow
//!  - values: ContentHash → serialized Value (dedup blob store)
//!  - runs: RunId → execution log
//!  - run_details: RunId → per-node execution detail log
//!  - cache: input_hash → output values (skip re-execution)
//!
//! Public storage ports and durable table identities remain here. Private
//! implementation modules group transactions by the records they publish.

mod admission;
mod exchange;
mod occurrences;
mod quarantine;

mod implementation;

pub use implementation::model::RunDetailRecord;
pub use implementation::model::RunNodeDetail;
pub use implementation::model::RunNodeDetailStatus;
pub use implementation::model::RunOutputDetail;
pub use implementation::model::RunOwner;
pub use implementation::model::RunRecord;
pub use implementation::model::TriggerAdmissionWrite;
pub use implementation::model::WorkflowApprovalAdvance;
pub use implementation::ports::EffectAttemptRepository;
pub use implementation::ports::ExecutionCache;
pub use implementation::ports::ScheduleStore;
pub use implementation::ports::TriggerStore;
pub use implementation::ports::ValueStore;
pub use implementation::ports::WorkflowRepository;

/// An attempt identity and relative path for one retained candidate file.
#[derive(Debug)]
pub struct ArtifactKey<'a> {
    pub attempt_id: &'a str,
    pub relative_path: &'a str,
}

use std::collections::HashMap;

pub use quarantine::CommitFault;
use redb::Database;
use redb::ReadableDatabase;
use redb::ReadableTable;
use redb::TableDefinition;
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::replay::ReplayEventRecord;

const WORKFLOWS: TableDefinition<&str, &[u8]> = TableDefinition::new("workflows");
const VALUES: TableDefinition<&[u8], &[u8]> = TableDefinition::new("values");
const CACHE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("cache");
const RUNS: TableDefinition<u64, &[u8]> = TableDefinition::new("runs");
const RUN_DETAILS: TableDefinition<u64, &[u8]> = TableDefinition::new("run_details");
const RUN_REPLAY_EVENTS: TableDefinition<u64, &[u8]> = TableDefinition::new("run_replay_events");
const RUN_COUNTERS: TableDefinition<&str, u64> = TableDefinition::new("run_counters");
const SCHEDULES: TableDefinition<&str, &[u8]> = TableDefinition::new("schedules");
const TRIGGERS: TableDefinition<&str, &[u8]> = TableDefinition::new("triggers");
const TRIGGER_RUNS: TableDefinition<&str, &[u8]> = TableDefinition::new("trigger_runs");
const WEBHOOK_CLAIMS: TableDefinition<&str, &[u8]> = TableDefinition::new("webhook_claims_v1");
const ORCHESTRATION_DEFINITIONS: TableDefinition<&str, &[u8]> = TableDefinition::new("orchestration_definitions");
const ORCHESTRATION_INSTANCES: TableDefinition<&str, &[u8]> = TableDefinition::new("orchestration_instances");
const ORCHESTRATION_EVENTS: TableDefinition<&str, &[u8]> = TableDefinition::new("orchestration_events");
const ORCHESTRATION_TRIGGER_ADMISSIONS: TableDefinition<&str, &str> =
    TableDefinition::new("orchestration_trigger_admissions");
const ORCHESTRATION_COUNTERS: TableDefinition<&str, u64> = TableDefinition::new("orchestration_counters");
const EFFECT_ATTEMPTS: TableDefinition<&str, &[u8]> = TableDefinition::new("effect_attempts_v1");
const EFFECT_OUTCOMES: TableDefinition<&str, &[u8]> = TableDefinition::new("effect_outcomes_v1");
const PROTOCOL_SESSIONS: TableDefinition<&str, &[u8]> = TableDefinition::new("protocol_sessions_v1");
const PROTOCOL_EVENTS: TableDefinition<&str, &[u8]> = TableDefinition::new("protocol_events_v1");
const WORKFLOW_APPROVAL_REQUESTS: TableDefinition<&str, &[u8]> = TableDefinition::new("workflow_approval_requests_v1");
const WORKFLOW_APPROVAL_RUN_INDEX: TableDefinition<u64, &str> = TableDefinition::new("workflow_approval_run_index_v1");
const WORKFLOW_TEMPLATE_ACTIVATIONS: TableDefinition<&str, &[u8]> =
    TableDefinition::new("workflow_template_activations_v1");
const EVALUATION_TRIALS: TableDefinition<&str, &[u8]> = TableDefinition::new("agent_evaluation_trials_v1");
const EVALUATION_RECEIPTS: TableDefinition<&str, &[u8]> = TableDefinition::new("agent_evaluation_receipts_v1");
const OPERATION_RUNS: TableDefinition<&str, &[u8]> = TableDefinition::new("operation_runs_v1");
const WORKFLOW_EXCHANGE_EXECUTIONS: TableDefinition<&str, &[u8]> =
    TableDefinition::new("workflow_exchange_executions_v1");
const WORKFLOW_EXCHANGE_REQUEST_INDEX: TableDefinition<&str, &str> =
    TableDefinition::new("workflow_exchange_request_index_v1");
const CANDIDATE_RECEIPTS: TableDefinition<&str, &[u8]> = TableDefinition::new("candidate_receipts_v1");
const CANDIDATE_ARTIFACTS: TableDefinition<&str, &[u8]> = TableDefinition::new("candidate_artifacts_v1");
const COMMAND_WORKER_USAGE: TableDefinition<&str, &[u8]> = TableDefinition::new("command_worker_usage_v1");
const COMMAND_WORKER_USAGE_BY_RUN: TableDefinition<u64, &str> = TableDefinition::new("command_worker_usage_by_run_v1");
const CANDIDATE_ATTEMPT_RESULTS: TableDefinition<&str, &[u8]> = TableDefinition::new("candidate_attempt_results_v1");
const WORKFLOW_EXCHANGE_IDEMPOTENCY_INDEX: TableDefinition<&str, &str> =
    TableDefinition::new("workflow_exchange_idempotency_index_v1");
const ORCHESTRATION_INSTANCE_COUNTER_KEY: &str = "instance_sequence";
const RUN_ID_COUNTER_KEY: &str = "next_run_id";
const RUN_ID_EXHAUSTED_MESSAGE: &str = "run id space is exhausted";
const INITIAL_RUN_ID_VALUE: u64 = 0;
const NEXT_RUN_ID_INCREMENT: u64 = 1;
const INITIAL_ORCHESTRATION_SEQUENCE: u64 = 0;
const NEXT_ORCHESTRATION_SEQUENCE_INCREMENT: u64 = 1;
const ORCHESTRATION_VERSION_INCREMENT: u64 = 1;
const ORCHESTRATION_EVENT_SEQUENCE_INCREMENT: u64 = 1;
const WORKFLOW_APPROVAL_VERSION_INCREMENT: u64 = 1;

// r[impl n8nclone.workflow_execution.trait_boundaries.storage_ports]
pub trait RunHistoryStore {
    fn save_run(&self, record: &RunRecord) -> Result<(), StoreError>;
    fn load_run(&self, run_id: crate::engine::RunId) -> Result<Option<RunRecord>, StoreError>;
    fn save_run_details(&self, record: &RunDetailRecord) -> Result<(), StoreError>;
    fn load_run_details(&self, run_id: crate::engine::RunId) -> Result<Option<RunDetailRecord>, StoreError>;
    fn save_run_replay_events(
        &self,
        run_id: crate::engine::RunId,
        events: &[ReplayEventRecord],
    ) -> Result<(), StoreError>;
    fn load_run_replay_events(
        &self,
        run_id: crate::engine::RunId,
    ) -> Result<Option<Vec<ReplayEventRecord>>, StoreError>;
    fn list_runs(&self, limit: usize) -> Result<Vec<RunRecord>, StoreError>;
    fn next_run_id(&self) -> Result<crate::engine::RunId, StoreError>;
}

pub trait OrchestrationStore {
    fn save_orchestration_definition(&self, definition: &crate::orchestration::Definition) -> Result<(), StoreError>;
    fn load_orchestration_definition(
        &self,
        id: &crate::orchestration::DefinitionId,
    ) -> Result<Option<crate::orchestration::Definition>, StoreError>;
    fn next_orchestration_sequence(&self) -> Result<u64, StoreError>;
    fn create_orchestration(
        &self,
        instance: &crate::orchestration::Instance,
        events: &[crate::orchestration::Event],
    ) -> Result<(), StoreError>;
    fn create_triggered_orchestration(
        &self,
        scoped_key: &crate::orchestration::TriggerScopedKey,
        instance: &crate::orchestration::Instance,
    ) -> Result<TriggerAdmissionWrite, StoreError>;
    fn commit_orchestration_transition(
        &self,
        expected_version: u64,
        instance: &crate::orchestration::Instance,
        event: &crate::orchestration::Event,
    ) -> Result<(), StoreError>;
    fn load_orchestration(
        &self,
        id: &crate::orchestration::InstanceId,
    ) -> Result<Option<crate::orchestration::Instance>, StoreError>;
    fn load_orchestration_events(
        &self,
        id: &crate::orchestration::InstanceId,
    ) -> Result<Vec<crate::orchestration::Event>, StoreError>;
    fn list_orchestrations(&self) -> Result<Vec<crate::orchestration::Instance>, StoreError>;
    fn save_orchestration_trigger_admission(
        &self,
        scoped_key: &crate::orchestration::TriggerScopedKey,
        instance_id: &crate::orchestration::InstanceId,
    ) -> Result<TriggerAdmissionWrite, StoreError>;
    fn list_orchestration_trigger_admissions(
        &self,
    ) -> Result<HashMap<crate::orchestration::TriggerScopedKey, crate::orchestration::InstanceId>, StoreError>;
}

/// Persistent store backed by redb.
#[derive(Clone)]
pub struct Store {
    db: std::sync::Arc<Database>,
    quarantine: std::sync::Arc<std::sync::Mutex<Option<quarantine::State>>>,
    occurrence_admission: std::sync::Arc<std::sync::Mutex<()>>,
}

include!("gate/state.rs");
include!("gate/transition.rs");
include!("run/records.rs");
include!("agent/evaluation.rs");
include!("operation.rs");
include!("operation/data.rs");

fn serialize<T: Serialize + ?Sized>(value: &T) -> Result<Vec<u8>, StoreError> {
    serde_json::to_vec(value).map_err(|error| StoreError::Serialize(error.to_string()))
}

fn deserialize<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, StoreError> {
    serde_json::from_slice(bytes).map_err(|error| StoreError::Deserialize(error.to_string()))
}

fn deserialize_optional<T: DeserializeOwned>(
    data: Option<redb::AccessGuard<'_, &[u8]>>,
) -> Result<Option<T>, StoreError> {
    data.map(|guard| deserialize(guard.value())).transpose()
}

fn reserve_run_id(txn: &redb::WriteTransaction) -> Result<crate::engine::RunId, StoreError> {
    let stored_next = {
        let counters = txn.open_table(RUN_COUNTERS).map_err(|error| StoreError::Table(error.to_string()))?;
        counters
            .get(RUN_ID_COUNTER_KEY)
            .map_err(|error| StoreError::Db(error.to_string()))?
            .map(|value| value.value())
    };
    let next_id = match stored_next {
        Some(next_id) => next_id,
        None => {
            let table = txn.open_table(RUNS).map_err(|error| StoreError::Table(error.to_string()))?;
            let mut seeded_next = INITIAL_RUN_ID_VALUE;
            let iter = table.iter().map_err(|error| StoreError::Db(error.to_string()))?;
            for entry in iter {
                let (key, _data) = entry.map_err(|error| StoreError::Db(error.to_string()))?;
                let candidate = key
                    .value()
                    .checked_add(NEXT_RUN_ID_INCREMENT)
                    .ok_or_else(|| StoreError::Conflict(RUN_ID_EXHAUSTED_MESSAGE.to_string()))?;
                seeded_next = seeded_next.max(candidate);
            }
            seeded_next
        }
    };
    let following_id = next_id
        .checked_add(NEXT_RUN_ID_INCREMENT)
        .ok_or_else(|| StoreError::Conflict(RUN_ID_EXHAUSTED_MESSAGE.to_string()))?;
    debug_assert!(following_id > next_id);
    debug_assert_eq!(following_id.checked_sub(next_id), Some(NEXT_RUN_ID_INCREMENT));
    {
        let mut counters = txn.open_table(RUN_COUNTERS).map_err(|error| StoreError::Table(error.to_string()))?;
        counters
            .insert(RUN_ID_COUNTER_KEY, following_id)
            .map_err(|error| StoreError::Db(error.to_string()))?;
    }
    Ok(crate::engine::RunId(next_id))
}

/// Store errors.
#[derive(Debug)]
pub enum StoreError {
    Db(String),
    Table(String),
    Serialize(String),
    Deserialize(String),
    Conflict(String),
    NotFound(String),
    PublicationUnknown,
    Quarantined,
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Db(e) => write!(f, "database error: {e}"),
            Self::Table(e) => write!(f, "table error: {e}"),
            Self::Serialize(e) => write!(f, "serialization error: {e}"),
            Self::Deserialize(e) => write!(f, "deserialization error: {e}"),
            Self::Conflict(e) => write!(f, "store conflict: {e}"),
            Self::NotFound(e) => write!(f, "store item not found: {e}"),
            Self::PublicationUnknown => f.write_str("storage publication outcome is unknown; reconcile before retry"),
            Self::Quarantined => f.write_str("store is quarantined pending exact persistence reconciliation"),
        }
    }
}

impl std::error::Error for StoreError {}

impl Store {
    pub fn list_trigger_runs(
        &self,
        max_record_count: usize,
    ) -> Result<Vec<crate::triggers::TriggerRunRecord>, crate::store::StoreError> {
        let txn = self.db.begin_read().map_err(|e| crate::store::StoreError::Db(e.to_string()))?;
        let table = txn
            .open_table(crate::store::TRIGGER_RUNS)
            .map_err(|e| crate::store::StoreError::Table(e.to_string()))?;
        let retained_record_count = crate::store::implementation::entry_count(&table)?.min(max_record_count);
        let mut records = Vec::with_capacity(retained_record_count);
        let iter = table.iter().map_err(|e| crate::store::StoreError::Db(e.to_string()))?;
        for entry in iter {
            if records.len() >= max_record_count {
                break;
            }
            let (_key, data) = entry.map_err(|e| crate::store::StoreError::Db(e.to_string()))?;
            let bytes: &[u8] = data.value();
            let record =
                serde_json::from_slice(bytes).map_err(|e| crate::store::StoreError::Deserialize(e.to_string()))?;
            records.push(record);
        }
        Ok(records)
    }
}

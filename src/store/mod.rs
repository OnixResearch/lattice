//! redb-backed persistent storage.
//!
//! Tables:
//!  - workflows: name → serialized Workflow
//!  - values: ContentHash → serialized Value (dedup blob store)
//!  - runs: RunId → execution log
//!  - run_details: RunId → per-node execution detail log
//!  - cache: input_hash → output values (skip re-execution)

use std::collections::HashMap;
use std::path::Path;

use rat_nodegraph::model::NodeId;
use rat_nodegraph::model::PortId;
use redb::Database;
use redb::ReadableDatabase;
use redb::ReadableTable;
use redb::TableDefinition;
use serde::Deserialize;
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::engine::ContentHash;
use crate::engine::RunId;
use crate::engine::Value;
use crate::engine::Workflow;
use crate::orchestration::OrchestrationDefinition;
use crate::orchestration::OrchestrationEvent;
use crate::orchestration::OrchestrationInstance;
use crate::replay::ReplayEventRecord;
use crate::scheduler::ScheduleDefinition;
use crate::triggers::TriggerDefinition;
use crate::triggers::TriggerRunRecord;

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
const ORCHESTRATION_INSTANCE_COUNTER_KEY: &str = "instance_sequence";
const RUN_ID_COUNTER_KEY: &str = "next_run_id";
const RUN_ID_EXHAUSTED_MESSAGE: &str = "run id space is exhausted";
const INITIAL_RUN_ID_VALUE: u64 = 0;
const NEXT_RUN_ID_INCREMENT: u64 = 1;
const INITIAL_ORCHESTRATION_SEQUENCE: u64 = 0;
const NEXT_ORCHESTRATION_SEQUENCE_INCREMENT: u64 = 1;
const ORCHESTRATION_VERSION_INCREMENT: u64 = 1;
const ORCHESTRATION_EVENT_SEQUENCE_INCREMENT: u64 = 1;

/// Durable owner identity used to prevent one execution from satisfying another dispatch.
// r[impl lattice.agent_orchestration.run_identity]
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum RunOwner {
    #[default]
    Unscoped,
    Orchestration {
        instance_id: String,
        dispatch_id: String,
    },
}

/// Serializable run record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunRecord {
    pub run_id: RunId,
    #[serde(default)]
    pub owner: RunOwner,
    pub workflow_name: String,
    pub started_epoch_ms: i64,
    pub finished_epoch_ms: Option<i64>,
    pub node_count: usize,
    pub cache_hits: usize,
    pub failures: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunNodeDetailStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunOutputDetail {
    pub port_id: PortId,
    pub port_label: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunNodeDetail {
    pub node_id: NodeId,
    pub node_label: String,
    pub status: RunNodeDetailStatus,
    pub cached: bool,
    pub outputs: Vec<RunOutputDetail>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunDetailRecord {
    pub run_id: RunId,
    pub nodes: Vec<RunNodeDetail>,
}

// r[impl n8nclone.workflow_execution.trait_boundaries.storage_ports]
pub trait WorkflowRepository {
    fn save_workflow(&self, workflow: &Workflow) -> Result<(), StoreError>;
    fn load_workflow(&self, name: &str) -> Result<Option<Workflow>, StoreError>;
    fn list_workflows(&self) -> Result<Vec<String>, StoreError>;
    fn delete_workflow(&self, name: &str) -> Result<(), StoreError>;
}

// r[impl n8nclone.workflow_execution.trait_boundaries.storage_ports]
pub trait RunHistoryStore {
    fn save_run(&self, record: &RunRecord) -> Result<(), StoreError>;
    fn load_run(&self, run_id: RunId) -> Result<Option<RunRecord>, StoreError>;
    fn save_run_details(&self, record: &RunDetailRecord) -> Result<(), StoreError>;
    fn load_run_details(&self, run_id: RunId) -> Result<Option<RunDetailRecord>, StoreError>;
    fn save_run_replay_events(&self, run_id: RunId, events: &[ReplayEventRecord]) -> Result<(), StoreError>;
    fn load_run_replay_events(&self, run_id: RunId) -> Result<Option<Vec<ReplayEventRecord>>, StoreError>;
    fn list_runs(&self, limit: usize) -> Result<Vec<RunRecord>, StoreError>;
    fn next_run_id(&self) -> Result<RunId, StoreError>;
}

// r[impl n8nclone.workflow_execution.trait_boundaries.storage_ports]
pub trait ExecutionCache {
    fn put_cache(&self, input_hash: &ContentHash, outputs: &HashMap<PortId, Value>) -> Result<(), StoreError>;
    fn get_cache(&self, input_hash: &ContentHash) -> Result<Option<HashMap<PortId, Value>>, StoreError>;
}

// r[impl n8nclone.workflow_execution.trait_boundaries.storage_ports]
pub trait ValueStore {
    fn put_value(&self, value: &Value) -> Result<(), StoreError>;
    fn get_value(&self, hash: &ContentHash) -> Result<Option<Value>, StoreError>;
}

// r[impl n8nclone.workflow_execution.trait_boundaries.storage_ports]
pub trait ScheduleStore {
    fn save_schedule(&self, schedule: &ScheduleDefinition) -> Result<(), StoreError>;
    fn load_schedule(&self, schedule_id: &str) -> Result<Option<ScheduleDefinition>, StoreError>;
    fn list_schedules(&self) -> Result<Vec<ScheduleDefinition>, StoreError>;
    fn delete_schedule(&self, schedule_id: &str) -> Result<(), StoreError>;
}

pub trait TriggerStore {
    fn save_trigger(&self, trigger: &TriggerDefinition) -> Result<(), StoreError>;
    fn load_trigger(&self, trigger_id: &str) -> Result<Option<TriggerDefinition>, StoreError>;
    fn list_triggers(&self) -> Result<Vec<TriggerDefinition>, StoreError>;
    fn delete_trigger(&self, trigger_id: &str) -> Result<(), StoreError>;
    fn save_trigger_run(&self, record: &TriggerRunRecord) -> Result<(), StoreError>;
    fn list_trigger_runs(&self, limit: usize) -> Result<Vec<TriggerRunRecord>, StoreError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TriggerAdmissionWrite {
    Inserted,
    Duplicate { instance_id: String },
}

pub trait OrchestrationStore {
    fn save_orchestration_definition(&self, definition: &OrchestrationDefinition) -> Result<(), StoreError>;
    fn load_orchestration_definition(&self, id: &str) -> Result<Option<OrchestrationDefinition>, StoreError>;
    fn next_orchestration_sequence(&self) -> Result<u64, StoreError>;
    fn create_orchestration(
        &self,
        instance: &OrchestrationInstance,
        events: &[OrchestrationEvent],
    ) -> Result<(), StoreError>;
    fn create_triggered_orchestration(
        &self,
        scoped_key: &str,
        instance: &OrchestrationInstance,
    ) -> Result<TriggerAdmissionWrite, StoreError>;
    fn commit_orchestration_transition(
        &self,
        expected_version: u64,
        instance: &OrchestrationInstance,
        event: &OrchestrationEvent,
    ) -> Result<(), StoreError>;
    fn load_orchestration(&self, id: &str) -> Result<Option<OrchestrationInstance>, StoreError>;
    fn load_orchestration_events(&self, id: &str) -> Result<Vec<OrchestrationEvent>, StoreError>;
    fn list_orchestrations(&self) -> Result<Vec<OrchestrationInstance>, StoreError>;
    fn save_orchestration_trigger_admission(
        &self,
        scoped_key: &str,
        instance_id: &str,
    ) -> Result<TriggerAdmissionWrite, StoreError>;
    fn list_orchestration_trigger_admissions(&self) -> Result<HashMap<String, String>, StoreError>;
}

/// Persistent store backed by redb.
pub struct Store {
    db: Database,
}

impl Store {
    /// Open or create the database at the given path.
    pub fn open(path: impl AsRef<Path>) -> Result<(), StoreError> {
        let db = Database::create(path.as_ref()).map_err(|e| StoreError::Db(e.to_string()))?;

        // Ensure tables exist.
        let txn = db.begin_write().map_err(|e| StoreError::Db(e.to_string()))?;
        {
            let _ = txn.open_table(WORKFLOWS).map_err(|e| StoreError::Table(e.to_string()))?;
            let _ = txn.open_table(VALUES).map_err(|e| StoreError::Table(e.to_string()))?;
            let _ = txn.open_table(CACHE).map_err(|e| StoreError::Table(e.to_string()))?;
            let _ = txn.open_table(RUNS).map_err(|e| StoreError::Table(e.to_string()))?;
            let _ = txn.open_table(RUN_DETAILS).map_err(|e| StoreError::Table(e.to_string()))?;
            let _ = txn.open_table(RUN_REPLAY_EVENTS).map_err(|e| StoreError::Table(e.to_string()))?;
            let _ = txn.open_table(RUN_COUNTERS).map_err(|e| StoreError::Table(e.to_string()))?;
            let _ = txn.open_table(SCHEDULES).map_err(|e| StoreError::Table(e.to_string()))?;
            let _ = txn.open_table(TRIGGERS).map_err(|e| StoreError::Table(e.to_string()))?;
            let _ = txn.open_table(TRIGGER_RUNS).map_err(|e| StoreError::Table(e.to_string()))?;
            let _ = txn.open_table(WEBHOOK_CLAIMS).map_err(|e| StoreError::Table(e.to_string()))?;
            let _ = txn.open_table(ORCHESTRATION_DEFINITIONS).map_err(|e| StoreError::Table(e.to_string()))?;
            let _ = txn.open_table(ORCHESTRATION_INSTANCES).map_err(|e| StoreError::Table(e.to_string()))?;
            let _ = txn.open_table(ORCHESTRATION_EVENTS).map_err(|e| StoreError::Table(e.to_string()))?;
            let _ = txn.open_table(ORCHESTRATION_TRIGGER_ADMISSIONS).map_err(|e| StoreError::Table(e.to_string()))?;
            let _ = txn.open_table(ORCHESTRATION_COUNTERS).map_err(|e| StoreError::Table(e.to_string()))?;
        }
        txn.commit().map_err(|e| StoreError::Db(e.to_string()))?;

        Ok(())
    }

    /// Open and return a Store instance.
    pub fn new(path: impl AsRef<Path>) -> Result<Self, StoreError> {
        let p = path.as_ref();
        Self::open(p)?;
        let db = Database::create(p).map_err(|e| StoreError::Db(e.to_string()))?;
        Ok(Self { db })
    }

    // -- workflows ----------------------------------------------------------

    /// Save a workflow.
    pub fn save_workflow(&self, workflow: &Workflow) -> Result<(), StoreError> {
        let data = serde_json::to_vec(workflow).map_err(|e| StoreError::Serialize(e.to_string()))?;
        let txn = self.db.begin_write().map_err(|e| StoreError::Db(e.to_string()))?;
        {
            let mut table = txn.open_table(WORKFLOWS).map_err(|e| StoreError::Table(e.to_string()))?;
            table
                .insert(workflow.meta.name.as_str(), data.as_slice())
                .map_err(|e| StoreError::Db(e.to_string()))?;
        }
        txn.commit().map_err(|e| StoreError::Db(e.to_string()))?;
        Ok(())
    }

    /// Load a workflow by name.
    pub fn load_workflow(&self, name: &str) -> Result<Option<Workflow>, StoreError> {
        let txn = self.db.begin_read().map_err(|e| StoreError::Db(e.to_string()))?;
        let table = txn.open_table(WORKFLOWS).map_err(|e| StoreError::Table(e.to_string()))?;
        match table.get(name).map_err(|e| StoreError::Db(e.to_string()))? {
            Some(data) => {
                let bytes: &[u8] = data.value();
                let wf: Workflow = serde_json::from_slice(bytes).map_err(|e| StoreError::Deserialize(e.to_string()))?;
                Ok(Some(wf))
            }
            None => Ok(None),
        }
    }

    /// List all workflow names.
    pub fn list_workflows(&self) -> Result<Vec<String>, StoreError> {
        let txn = self.db.begin_read().map_err(|e| StoreError::Db(e.to_string()))?;
        let table = txn.open_table(WORKFLOWS).map_err(|e| StoreError::Table(e.to_string()))?;
        let mut names = Vec::new();
        let iter = table.iter().map_err(|e| StoreError::Db(e.to_string()))?;
        for entry in iter {
            let (key, _val) = entry.map_err(|e| StoreError::Db(e.to_string()))?;
            names.push(key.value().to_string());
        }
        Ok(names)
    }

    /// Delete a workflow by name.
    pub fn delete_workflow(&self, name: &str) -> Result<(), StoreError> {
        let txn = self.db.begin_write().map_err(|e| StoreError::Db(e.to_string()))?;
        {
            let mut table = txn.open_table(WORKFLOWS).map_err(|e| StoreError::Table(e.to_string()))?;
            table.remove(name).map_err(|e| StoreError::Db(e.to_string()))?;
        }
        txn.commit().map_err(|e| StoreError::Db(e.to_string()))?;
        Ok(())
    }

    // -- schedules -----------------------------------------------------------

    /// Save or replace a schedule definition.
    pub fn save_schedule(&self, schedule: &ScheduleDefinition) -> Result<(), StoreError> {
        let data = serde_json::to_vec(schedule).map_err(|e| StoreError::Serialize(e.to_string()))?;
        let txn = self.db.begin_write().map_err(|e| StoreError::Db(e.to_string()))?;
        {
            let mut table = txn.open_table(SCHEDULES).map_err(|e| StoreError::Table(e.to_string()))?;
            table.insert(schedule.id.as_str(), data.as_slice()).map_err(|e| StoreError::Db(e.to_string()))?;
        }
        txn.commit().map_err(|e| StoreError::Db(e.to_string()))?;
        Ok(())
    }

    /// Load a schedule definition by id.
    pub fn load_schedule(&self, schedule_id: &str) -> Result<Option<ScheduleDefinition>, StoreError> {
        let txn = self.db.begin_read().map_err(|e| StoreError::Db(e.to_string()))?;
        let table = txn.open_table(SCHEDULES).map_err(|e| StoreError::Table(e.to_string()))?;
        match table.get(schedule_id).map_err(|e| StoreError::Db(e.to_string()))? {
            Some(data) => {
                let bytes: &[u8] = data.value();
                let schedule: ScheduleDefinition =
                    serde_json::from_slice(bytes).map_err(|e| StoreError::Deserialize(e.to_string()))?;
                Ok(Some(schedule))
            }
            None => Ok(None),
        }
    }

    /// List all schedule definitions.
    pub fn list_schedules(&self) -> Result<Vec<ScheduleDefinition>, StoreError> {
        let txn = self.db.begin_read().map_err(|e| StoreError::Db(e.to_string()))?;
        let table = txn.open_table(SCHEDULES).map_err(|e| StoreError::Table(e.to_string()))?;
        let mut schedules = Vec::new();
        let iter = table.iter().map_err(|e| StoreError::Db(e.to_string()))?;
        for entry in iter {
            let (_key, data) = entry.map_err(|e| StoreError::Db(e.to_string()))?;
            let bytes: &[u8] = data.value();
            let schedule: ScheduleDefinition =
                serde_json::from_slice(bytes).map_err(|e| StoreError::Deserialize(e.to_string()))?;
            schedules.push(schedule);
        }
        Ok(schedules)
    }

    /// Delete a schedule definition by id.
    pub fn delete_schedule(&self, schedule_id: &str) -> Result<(), StoreError> {
        let txn = self.db.begin_write().map_err(|e| StoreError::Db(e.to_string()))?;
        {
            let mut table = txn.open_table(SCHEDULES).map_err(|e| StoreError::Table(e.to_string()))?;
            table.remove(schedule_id).map_err(|e| StoreError::Db(e.to_string()))?;
        }
        txn.commit().map_err(|e| StoreError::Db(e.to_string()))?;
        Ok(())
    }

    // -- triggers ------------------------------------------------------------

    // r[impl n8nclone.scheduling.triggers.model]
    pub fn save_trigger(&self, trigger: &TriggerDefinition) -> Result<(), StoreError> {
        let data = serde_json::to_vec(trigger).map_err(|e| StoreError::Serialize(e.to_string()))?;
        let txn = self.db.begin_write().map_err(|e| StoreError::Db(e.to_string()))?;
        {
            let mut table = txn.open_table(TRIGGERS).map_err(|e| StoreError::Table(e.to_string()))?;
            table.insert(trigger.id.as_str(), data.as_slice()).map_err(|e| StoreError::Db(e.to_string()))?;
        }
        txn.commit().map_err(|e| StoreError::Db(e.to_string()))?;
        Ok(())
    }

    pub fn load_trigger(&self, trigger_id: &str) -> Result<Option<TriggerDefinition>, StoreError> {
        let txn = self.db.begin_read().map_err(|e| StoreError::Db(e.to_string()))?;
        let table = txn.open_table(TRIGGERS).map_err(|e| StoreError::Table(e.to_string()))?;
        match table.get(trigger_id).map_err(|e| StoreError::Db(e.to_string()))? {
            Some(data) => {
                let bytes: &[u8] = data.value();
                let trigger = serde_json::from_slice(bytes).map_err(|e| StoreError::Deserialize(e.to_string()))?;
                Ok(Some(trigger))
            }
            None => Ok(None),
        }
    }

    pub fn list_triggers(&self) -> Result<Vec<TriggerDefinition>, StoreError> {
        let txn = self.db.begin_read().map_err(|e| StoreError::Db(e.to_string()))?;
        let table = txn.open_table(TRIGGERS).map_err(|e| StoreError::Table(e.to_string()))?;
        let mut triggers = Vec::new();
        let iter = table.iter().map_err(|e| StoreError::Db(e.to_string()))?;
        for entry in iter {
            let (_key, data) = entry.map_err(|e| StoreError::Db(e.to_string()))?;
            let bytes: &[u8] = data.value();
            let trigger = serde_json::from_slice(bytes).map_err(|e| StoreError::Deserialize(e.to_string()))?;
            triggers.push(trigger);
        }
        Ok(triggers)
    }

    pub fn delete_trigger(&self, trigger_id: &str) -> Result<(), StoreError> {
        let txn = self.db.begin_write().map_err(|e| StoreError::Db(e.to_string()))?;
        {
            let mut table = txn.open_table(TRIGGERS).map_err(|e| StoreError::Table(e.to_string()))?;
            table.remove(trigger_id).map_err(|e| StoreError::Db(e.to_string()))?;
        }
        txn.commit().map_err(|e| StoreError::Db(e.to_string()))?;
        Ok(())
    }

    // r[impl n8nclone.scheduling.triggers.history]
    pub fn save_trigger_run(&self, record: &TriggerRunRecord) -> Result<(), StoreError> {
        let data = serde_json::to_vec(record).map_err(|e| StoreError::Serialize(e.to_string()))?;
        let key = trigger_run_record_key(record);
        let txn = self.db.begin_write().map_err(|e| StoreError::Db(e.to_string()))?;
        {
            let mut table = txn.open_table(TRIGGER_RUNS).map_err(|e| StoreError::Table(e.to_string()))?;
            table.insert(key.as_str(), data.as_slice()).map_err(|e| StoreError::Db(e.to_string()))?;
        }
        txn.commit().map_err(|e| StoreError::Db(e.to_string()))?;
        Ok(())
    }

    pub fn list_trigger_runs(&self, limit: usize) -> Result<Vec<TriggerRunRecord>, StoreError> {
        let txn = self.db.begin_read().map_err(|e| StoreError::Db(e.to_string()))?;
        let table = txn.open_table(TRIGGER_RUNS).map_err(|e| StoreError::Table(e.to_string()))?;
        let mut records = Vec::new();
        let iter = table.iter().map_err(|e| StoreError::Db(e.to_string()))?;
        for entry in iter {
            if records.len() >= limit {
                break;
            }
            let (_key, data) = entry.map_err(|e| StoreError::Db(e.to_string()))?;
            let bytes: &[u8] = data.value();
            let record = serde_json::from_slice(bytes).map_err(|e| StoreError::Deserialize(e.to_string()))?;
            records.push(record);
        }
        Ok(records)
    }

    // r[impl lattice.scheduling.webhook_atomic_idempotency]
    pub fn claim_webhook(
        &self,
        request: &crate::triggers::WebhookClaimRequest,
    ) -> Result<crate::triggers::WebhookClaimResolution, StoreError> {
        let txn = self.db.begin_write().map_err(|error| StoreError::Db(error.to_string()))?;
        {
            let claims = txn.open_table(WEBHOOK_CLAIMS).map_err(|error| StoreError::Table(error.to_string()))?;
            if let Some(existing) =
                claims.get(request.identity.as_str()).map_err(|error| StoreError::Db(error.to_string()))?
            {
                let claim: crate::triggers::WebhookClaim = deserialize(existing.value())?;
                return Ok(crate::triggers::WebhookClaimResolution::Duplicate {
                    existing_run_id: claim.run_id,
                });
            }
        }

        let run_id = reserve_run_id(&txn)?;
        let claim = request.claim(run_id);
        let record = request.started_record(run_id);
        let claim_data = serialize(&claim)?;
        let record_data = serialize(&record)?;
        let record_key = trigger_run_record_key(&record);
        {
            let mut claims = txn.open_table(WEBHOOK_CLAIMS).map_err(|error| StoreError::Table(error.to_string()))?;
            claims
                .insert(request.identity.as_str(), claim_data.as_slice())
                .map_err(|error| StoreError::Db(error.to_string()))?;
        }
        {
            let mut history = txn.open_table(TRIGGER_RUNS).map_err(|error| StoreError::Table(error.to_string()))?;
            history
                .insert(record_key.as_str(), record_data.as_slice())
                .map_err(|error| StoreError::Db(error.to_string()))?;
        }
        txn.commit().map_err(|error| StoreError::Db(error.to_string()))?;
        Ok(crate::triggers::WebhookClaimResolution::Inserted { run_id })
    }

    pub fn list_webhook_claims(&self, max_claim_count: u64) -> Result<Vec<crate::triggers::WebhookClaim>, StoreError> {
        let max_claim_count = usize::try_from(max_claim_count)
            .map_err(|error| StoreError::Conflict(format!("webhook claim limit is unsupported: {error}")))?;
        let txn = self.db.begin_read().map_err(|error| StoreError::Db(error.to_string()))?;
        let table = txn.open_table(WEBHOOK_CLAIMS).map_err(|error| StoreError::Table(error.to_string()))?;
        let mut claims = Vec::with_capacity(max_claim_count);
        let iter = table.iter().map_err(|error| StoreError::Db(error.to_string()))?;
        for entry in iter {
            if claims.len() >= max_claim_count {
                break;
            }
            let (_identity, data) = entry.map_err(|error| StoreError::Db(error.to_string()))?;
            claims.push(deserialize(data.value())?);
        }
        assert!(claims.len() <= max_claim_count);
        Ok(claims)
    }

    // -- value store (content-addressed) ------------------------------------

    /// Store a value by its content hash.
    pub fn put_value(&self, value: &Value) -> Result<(), StoreError> {
        let data = serde_json::to_vec(value).map_err(|e| StoreError::Serialize(e.to_string()))?;
        let txn = self.db.begin_write().map_err(|e| StoreError::Db(e.to_string()))?;
        {
            let mut table = txn.open_table(VALUES).map_err(|e| StoreError::Table(e.to_string()))?;
            table.insert(value.hash.0.as_slice(), data.as_slice()).map_err(|e| StoreError::Db(e.to_string()))?;
        }
        txn.commit().map_err(|e| StoreError::Db(e.to_string()))?;
        Ok(())
    }

    /// Retrieve a value by content hash.
    pub fn get_value(&self, hash: &ContentHash) -> Result<Option<Value>, StoreError> {
        let txn = self.db.begin_read().map_err(|e| StoreError::Db(e.to_string()))?;
        let table = txn.open_table(VALUES).map_err(|e| StoreError::Table(e.to_string()))?;
        match table.get(hash.0.as_slice()).map_err(|e| StoreError::Db(e.to_string()))? {
            Some(data) => {
                let bytes: &[u8] = data.value();
                let val: Value = serde_json::from_slice(bytes).map_err(|e| StoreError::Deserialize(e.to_string()))?;
                Ok(Some(val))
            }
            None => Ok(None),
        }
    }

    // -- execution cache ----------------------------------------------------

    /// Store cached outputs for an input hash.
    pub fn put_cache(&self, input_hash: &ContentHash, outputs: &HashMap<PortId, Value>) -> Result<(), StoreError> {
        let data = serde_json::to_vec(outputs).map_err(|e| StoreError::Serialize(e.to_string()))?;
        let txn = self.db.begin_write().map_err(|e| StoreError::Db(e.to_string()))?;
        {
            let mut table = txn.open_table(CACHE).map_err(|e| StoreError::Table(e.to_string()))?;
            table.insert(input_hash.0.as_slice(), data.as_slice()).map_err(|e| StoreError::Db(e.to_string()))?;
        }
        txn.commit().map_err(|e| StoreError::Db(e.to_string()))?;
        Ok(())
    }

    /// Retrieve cached outputs for an input hash.
    pub fn get_cache(&self, input_hash: &ContentHash) -> Result<Option<HashMap<PortId, Value>>, StoreError> {
        let txn = self.db.begin_read().map_err(|e| StoreError::Db(e.to_string()))?;
        let table = txn.open_table(CACHE).map_err(|e| StoreError::Table(e.to_string()))?;
        match table.get(input_hash.0.as_slice()).map_err(|e| StoreError::Db(e.to_string()))? {
            Some(data) => {
                let bytes: &[u8] = data.value();
                let map: HashMap<PortId, Value> =
                    serde_json::from_slice(bytes).map_err(|e| StoreError::Deserialize(e.to_string()))?;
                Ok(Some(map))
            }
            None => Ok(None),
        }
    }

    // -- run records --------------------------------------------------------

    /// Store a run record without allowing a global run identity to be overwritten.
    // r[impl lattice.agent_orchestration.run_identity]
    pub fn save_run(&self, record: &RunRecord) -> Result<(), StoreError> {
        let data = serde_json::to_vec(record).map_err(|e| StoreError::Serialize(e.to_string()))?;
        let next_after_record = record
            .run_id
            .0
            .checked_add(NEXT_RUN_ID_INCREMENT)
            .ok_or_else(|| StoreError::Conflict(RUN_ID_EXHAUSTED_MESSAGE.to_string()))?;
        let txn = self.db.begin_write().map_err(|e| StoreError::Db(e.to_string()))?;
        {
            let mut table = txn.open_table(RUNS).map_err(|e| StoreError::Table(e.to_string()))?;
            if table.get(record.run_id.0).map_err(|e| StoreError::Db(e.to_string()))?.is_some() {
                return Err(StoreError::Conflict(format!("run identity {} is already persisted", record.run_id)));
            }
            table.insert(record.run_id.0, data.as_slice()).map_err(|e| StoreError::Db(e.to_string()))?;
        }
        {
            let mut counters = txn.open_table(RUN_COUNTERS).map_err(|e| StoreError::Table(e.to_string()))?;
            let current = counters
                .get(RUN_ID_COUNTER_KEY)
                .map_err(|e| StoreError::Db(e.to_string()))?
                .map(|value| value.value())
                .unwrap_or(INITIAL_RUN_ID_VALUE);
            if next_after_record > current {
                counters.insert(RUN_ID_COUNTER_KEY, next_after_record).map_err(|e| StoreError::Db(e.to_string()))?;
            }
        }
        txn.commit().map_err(|e| StoreError::Db(e.to_string()))?;
        Ok(())
    }

    /// Load one run record by identity.
    pub fn load_run(&self, run_id: RunId) -> Result<Option<RunRecord>, StoreError> {
        let txn = self.db.begin_read().map_err(|error| StoreError::Db(error.to_string()))?;
        let table = txn.open_table(RUNS).map_err(|error| StoreError::Table(error.to_string()))?;
        match table.get(run_id.0).map_err(|error| StoreError::Db(error.to_string()))? {
            Some(data) => deserialize(data.value()).map(Some),
            None => Ok(None),
        }
    }

    /// Store detailed per-node records for a run.
    pub fn save_run_details(&self, record: &RunDetailRecord) -> Result<(), StoreError> {
        let data = serde_json::to_vec(record).map_err(|e| StoreError::Serialize(e.to_string()))?;
        let txn = self.db.begin_write().map_err(|e| StoreError::Db(e.to_string()))?;
        {
            let mut table = txn.open_table(RUN_DETAILS).map_err(|e| StoreError::Table(e.to_string()))?;
            table.insert(record.run_id.0, data.as_slice()).map_err(|e| StoreError::Db(e.to_string()))?;
        }
        txn.commit().map_err(|e| StoreError::Db(e.to_string()))?;
        Ok(())
    }

    /// Load detailed per-node records for a run.
    pub fn load_run_details(&self, run_id: RunId) -> Result<Option<RunDetailRecord>, StoreError> {
        let txn = self.db.begin_read().map_err(|e| StoreError::Db(e.to_string()))?;
        let table = txn.open_table(RUN_DETAILS).map_err(|e| StoreError::Table(e.to_string()))?;
        match table.get(run_id.0).map_err(|e| StoreError::Db(e.to_string()))? {
            Some(data) => {
                let bytes: &[u8] = data.value();
                let record: RunDetailRecord =
                    serde_json::from_slice(bytes).map_err(|e| StoreError::Deserialize(e.to_string()))?;
                Ok(Some(record))
            }
            None => Ok(None),
        }
    }

    // r[impl n8nclone.workflow_execution.replay.event_store]
    pub fn save_run_replay_events(&self, run_id: RunId, events: &[ReplayEventRecord]) -> Result<(), StoreError> {
        let data = serde_json::to_vec(events).map_err(|e| StoreError::Serialize(e.to_string()))?;
        let txn = self.db.begin_write().map_err(|e| StoreError::Db(e.to_string()))?;
        {
            let mut table = txn.open_table(RUN_REPLAY_EVENTS).map_err(|e| StoreError::Table(e.to_string()))?;
            table.insert(run_id.0, data.as_slice()).map_err(|e| StoreError::Db(e.to_string()))?;
        }
        txn.commit().map_err(|e| StoreError::Db(e.to_string()))?;
        Ok(())
    }

    // r[impl n8nclone.workflow_execution.replay.event_store]
    pub fn load_run_replay_events(&self, run_id: RunId) -> Result<Option<Vec<ReplayEventRecord>>, StoreError> {
        let txn = self.db.begin_read().map_err(|e| StoreError::Db(e.to_string()))?;
        let table = txn.open_table(RUN_REPLAY_EVENTS).map_err(|e| StoreError::Table(e.to_string()))?;
        match table.get(run_id.0).map_err(|e| StoreError::Db(e.to_string()))? {
            Some(data) => {
                let bytes: &[u8] = data.value();
                let events = serde_json::from_slice(bytes).map_err(|e| StoreError::Deserialize(e.to_string()))?;
                Ok(Some(events))
            }
            None => Ok(None),
        }
    }

    /// List recent runs (newest first).
    pub fn list_runs(&self, limit: usize) -> Result<Vec<RunRecord>, StoreError> {
        let txn = self.db.begin_read().map_err(|e| StoreError::Db(e.to_string()))?;
        let table = txn.open_table(RUNS).map_err(|e| StoreError::Table(e.to_string()))?;
        let mut records = Vec::new();
        let iter = table.iter().map_err(|e| StoreError::Db(e.to_string()))?;
        for entry in iter {
            let (_key, data) = entry.map_err(|e| StoreError::Db(e.to_string()))?;
            let bytes: &[u8] = data.value();
            let rec: RunRecord = serde_json::from_slice(bytes).map_err(|e| StoreError::Deserialize(e.to_string()))?;
            records.push(rec);
        }
        records.reverse();
        records.truncate(limit);
        Ok(records)
    }

    /// Atomically reserve and return one globally unique run id.
    // r[impl lattice.agent_orchestration.run_identity]
    pub fn next_run_id(&self) -> Result<RunId, StoreError> {
        let txn = self.db.begin_write().map_err(|error| StoreError::Db(error.to_string()))?;
        let run_id = reserve_run_id(&txn)?;
        txn.commit().map_err(|error| StoreError::Db(error.to_string()))?;
        Ok(run_id)
    }

    // -- durable orchestration ---------------------------------------------

    // r[impl lattice.agent_orchestration.durability]
    pub fn save_orchestration_definition(&self, definition: &OrchestrationDefinition) -> Result<(), StoreError> {
        let data = serialize(definition)?;
        let txn = self.db.begin_write().map_err(|error| StoreError::Db(error.to_string()))?;
        {
            let mut table =
                txn.open_table(ORCHESTRATION_DEFINITIONS).map_err(|error| StoreError::Table(error.to_string()))?;
            table
                .insert(definition.id.as_str(), data.as_slice())
                .map_err(|error| StoreError::Db(error.to_string()))?;
        }
        txn.commit().map_err(|error| StoreError::Db(error.to_string()))?;
        Ok(())
    }

    pub fn load_orchestration_definition(&self, id: &str) -> Result<Option<OrchestrationDefinition>, StoreError> {
        let txn = self.db.begin_read().map_err(|error| StoreError::Db(error.to_string()))?;
        let table = txn.open_table(ORCHESTRATION_DEFINITIONS).map_err(|error| StoreError::Table(error.to_string()))?;
        deserialize_optional(table.get(id).map_err(|error| StoreError::Db(error.to_string()))?)
    }

    pub fn next_orchestration_sequence(&self) -> Result<u64, StoreError> {
        let txn = self.db.begin_write().map_err(|error| StoreError::Db(error.to_string()))?;
        let next = {
            let mut table =
                txn.open_table(ORCHESTRATION_COUNTERS).map_err(|error| StoreError::Table(error.to_string()))?;
            let current = table
                .get(ORCHESTRATION_INSTANCE_COUNTER_KEY)
                .map_err(|error| StoreError::Db(error.to_string()))?
                .map_or(INITIAL_ORCHESTRATION_SEQUENCE, |value| value.value());
            let next = current
                .checked_add(NEXT_ORCHESTRATION_SEQUENCE_INCREMENT)
                .ok_or_else(|| StoreError::Conflict("orchestration instance sequence exhausted".to_string()))?;
            table
                .insert(ORCHESTRATION_INSTANCE_COUNTER_KEY, next)
                .map_err(|error| StoreError::Db(error.to_string()))?;
            next
        };
        txn.commit().map_err(|error| StoreError::Db(error.to_string()))?;
        Ok(next)
    }

    // r[impl lattice.agent_orchestration.durability]
    pub fn create_orchestration(
        &self,
        instance: &OrchestrationInstance,
        events: &[OrchestrationEvent],
    ) -> Result<(), StoreError> {
        validate_event_batch(instance, events)?;
        let instance_data = serialize(instance)?;
        let event_data = serialize(events)?;
        let txn = self.db.begin_write().map_err(|error| StoreError::Db(error.to_string()))?;
        {
            let mut instances =
                txn.open_table(ORCHESTRATION_INSTANCES).map_err(|error| StoreError::Table(error.to_string()))?;
            if instances.get(instance.id.as_str()).map_err(|error| StoreError::Db(error.to_string()))?.is_some() {
                return Err(StoreError::Conflict(format!("orchestration instance '{}' already exists", instance.id)));
            }
            instances
                .insert(instance.id.as_str(), instance_data.as_slice())
                .map_err(|error| StoreError::Db(error.to_string()))?;
            let mut event_table =
                txn.open_table(ORCHESTRATION_EVENTS).map_err(|error| StoreError::Table(error.to_string()))?;
            event_table
                .insert(instance.id.as_str(), event_data.as_slice())
                .map_err(|error| StoreError::Db(error.to_string()))?;
        }
        txn.commit().map_err(|error| StoreError::Db(error.to_string()))?;
        Ok(())
    }

    // r[impl lattice.agent_orchestration.triggers]
    pub fn create_triggered_orchestration(
        &self,
        scoped_key: &str,
        instance: &OrchestrationInstance,
    ) -> Result<TriggerAdmissionWrite, StoreError> {
        validate_event_batch(instance, &[])?;
        let instance_data = serialize(instance)?;
        let empty_events = serialize(&Vec::<OrchestrationEvent>::new())?;
        let txn = self.db.begin_write().map_err(|error| StoreError::Db(error.to_string()))?;
        let outcome = {
            let mut admissions = txn
                .open_table(ORCHESTRATION_TRIGGER_ADMISSIONS)
                .map_err(|error| StoreError::Table(error.to_string()))?;
            let existing_instance_id = admissions
                .get(scoped_key)
                .map_err(|error| StoreError::Db(error.to_string()))?
                .map(|existing| existing.value().to_string());
            match existing_instance_id {
                Some(existing_instance_id) => TriggerAdmissionWrite::Duplicate {
                    instance_id: existing_instance_id,
                },
                None => {
                    let mut instances = txn
                        .open_table(ORCHESTRATION_INSTANCES)
                        .map_err(|error| StoreError::Table(error.to_string()))?;
                    if instances.get(instance.id.as_str()).map_err(|error| StoreError::Db(error.to_string()))?.is_some()
                    {
                        return Err(StoreError::Conflict(format!(
                            "orchestration instance '{}' already exists",
                            instance.id
                        )));
                    }
                    instances
                        .insert(instance.id.as_str(), instance_data.as_slice())
                        .map_err(|error| StoreError::Db(error.to_string()))?;
                    let mut events =
                        txn.open_table(ORCHESTRATION_EVENTS).map_err(|error| StoreError::Table(error.to_string()))?;
                    events
                        .insert(instance.id.as_str(), empty_events.as_slice())
                        .map_err(|error| StoreError::Db(error.to_string()))?;
                    admissions
                        .insert(scoped_key, instance.id.as_str())
                        .map_err(|error| StoreError::Db(error.to_string()))?;
                    TriggerAdmissionWrite::Inserted
                }
            }
        };
        txn.commit().map_err(|error| StoreError::Db(error.to_string()))?;
        Ok(outcome)
    }

    // r[impl lattice.agent_orchestration.durability]
    pub fn commit_orchestration_transition(
        &self,
        expected_version: u64,
        instance: &OrchestrationInstance,
        event: &OrchestrationEvent,
    ) -> Result<(), StoreError> {
        if event.instance_id != instance.id || instance.event_head.as_deref() != Some(event.event_hash.as_str()) {
            return Err(StoreError::Conflict(
                "orchestration transition event does not match the resulting instance head".to_string(),
            ));
        }
        let instance_data = serialize(instance)?;
        let txn = self.db.begin_write().map_err(|error| StoreError::Db(error.to_string()))?;
        {
            let mut instances =
                txn.open_table(ORCHESTRATION_INSTANCES).map_err(|error| StoreError::Table(error.to_string()))?;
            let current = instances
                .get(instance.id.as_str())
                .map_err(|error| StoreError::Db(error.to_string()))?
                .ok_or_else(|| StoreError::NotFound(format!("orchestration instance '{}'", instance.id)))?;
            let current_instance: OrchestrationInstance = deserialize(current.value())?;
            if current_instance.version != expected_version
                || instance.version != expected_version.saturating_add(ORCHESTRATION_VERSION_INCREMENT)
            {
                return Err(StoreError::Conflict(format!(
                    "orchestration instance '{}' changed concurrently",
                    instance.id
                )));
            }
            let mut event_table =
                txn.open_table(ORCHESTRATION_EVENTS).map_err(|error| StoreError::Table(error.to_string()))?;
            let mut events =
                match event_table.get(instance.id.as_str()).map_err(|error| StoreError::Db(error.to_string()))? {
                    Some(data) => deserialize::<Vec<OrchestrationEvent>>(data.value())?,
                    None => Vec::new(),
                };
            validate_next_event(&current_instance, &events, event)?;
            events.push(event.clone());
            let event_data = serialize(&events)?;
            drop(current);
            instances
                .insert(instance.id.as_str(), instance_data.as_slice())
                .map_err(|error| StoreError::Db(error.to_string()))?;
            event_table
                .insert(instance.id.as_str(), event_data.as_slice())
                .map_err(|error| StoreError::Db(error.to_string()))?;
        }
        txn.commit().map_err(|error| StoreError::Db(error.to_string()))?;
        Ok(())
    }

    pub fn load_orchestration(&self, id: &str) -> Result<Option<OrchestrationInstance>, StoreError> {
        let txn = self.db.begin_read().map_err(|error| StoreError::Db(error.to_string()))?;
        let table = txn.open_table(ORCHESTRATION_INSTANCES).map_err(|error| StoreError::Table(error.to_string()))?;
        deserialize_optional(table.get(id).map_err(|error| StoreError::Db(error.to_string()))?)
    }

    pub fn load_orchestration_events(&self, id: &str) -> Result<Vec<OrchestrationEvent>, StoreError> {
        let txn = self.db.begin_read().map_err(|error| StoreError::Db(error.to_string()))?;
        let table = txn.open_table(ORCHESTRATION_EVENTS).map_err(|error| StoreError::Table(error.to_string()))?;
        match table.get(id).map_err(|error| StoreError::Db(error.to_string()))? {
            Some(data) => deserialize(data.value()),
            None => Ok(Vec::new()),
        }
    }

    pub fn list_orchestrations(&self) -> Result<Vec<OrchestrationInstance>, StoreError> {
        let txn = self.db.begin_read().map_err(|error| StoreError::Db(error.to_string()))?;
        let table = txn.open_table(ORCHESTRATION_INSTANCES).map_err(|error| StoreError::Table(error.to_string()))?;
        let mut instances = Vec::new();
        let iter = table.iter().map_err(|error| StoreError::Db(error.to_string()))?;
        for entry in iter {
            let (_key, data) = entry.map_err(|error| StoreError::Db(error.to_string()))?;
            instances.push(deserialize(data.value())?);
        }
        instances.sort_by_key(|instance: &OrchestrationInstance| instance.sequence);
        Ok(instances)
    }

    // r[impl lattice.agent_orchestration.triggers]
    pub fn save_orchestration_trigger_admission(
        &self,
        scoped_key: &str,
        instance_id: &str,
    ) -> Result<TriggerAdmissionWrite, StoreError> {
        let txn = self.db.begin_write().map_err(|error| StoreError::Db(error.to_string()))?;
        let outcome = {
            let mut table = txn
                .open_table(ORCHESTRATION_TRIGGER_ADMISSIONS)
                .map_err(|error| StoreError::Table(error.to_string()))?;
            let existing_instance_id = table
                .get(scoped_key)
                .map_err(|error| StoreError::Db(error.to_string()))?
                .map(|existing| existing.value().to_string());
            match existing_instance_id {
                Some(existing_instance_id) => TriggerAdmissionWrite::Duplicate {
                    instance_id: existing_instance_id,
                },
                None => {
                    table.insert(scoped_key, instance_id).map_err(|error| StoreError::Db(error.to_string()))?;
                    TriggerAdmissionWrite::Inserted
                }
            }
        };
        txn.commit().map_err(|error| StoreError::Db(error.to_string()))?;
        Ok(outcome)
    }

    pub fn list_orchestration_trigger_admissions(&self) -> Result<HashMap<String, String>, StoreError> {
        let txn = self.db.begin_read().map_err(|error| StoreError::Db(error.to_string()))?;
        let table = txn
            .open_table(ORCHESTRATION_TRIGGER_ADMISSIONS)
            .map_err(|error| StoreError::Table(error.to_string()))?;
        let mut admissions = HashMap::new();
        let iter = table.iter().map_err(|error| StoreError::Db(error.to_string()))?;
        for entry in iter {
            let (key, value) = entry.map_err(|error| StoreError::Db(error.to_string()))?;
            admissions.insert(key.value().to_string(), value.value().to_string());
        }
        Ok(admissions)
    }
}

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

fn validate_event_batch(instance: &OrchestrationInstance, events: &[OrchestrationEvent]) -> Result<(), StoreError> {
    let mut prior_hash: Option<&str> = None;
    let mut prior_sequence: Option<u64> = None;
    for event in events {
        if event.instance_id != instance.id || event.prior_hash.as_deref() != prior_hash {
            return Err(StoreError::Conflict(
                "orchestration creation event chain does not match the instance".to_string(),
            ));
        }
        if let Some(sequence) = prior_sequence
            && event.sequence != sequence.saturating_add(ORCHESTRATION_EVENT_SEQUENCE_INCREMENT)
        {
            return Err(StoreError::Conflict("orchestration creation events are not contiguous".to_string()));
        }
        prior_hash = Some(event.event_hash.as_str());
        prior_sequence = Some(event.sequence);
    }
    if instance.event_head.as_deref() != prior_hash || instance.last_event_sequence != prior_sequence {
        return Err(StoreError::Conflict(
            "orchestration creation events do not match the persisted instance head".to_string(),
        ));
    }
    Ok(())
}

fn validate_next_event(
    current: &OrchestrationInstance,
    events: &[OrchestrationEvent],
    event: &OrchestrationEvent,
) -> Result<(), StoreError> {
    if event.instance_id != current.id || event.prior_hash != current.event_head {
        return Err(StoreError::Conflict("orchestration event does not extend the current event head".to_string()));
    }
    let expected_sequence = current
        .last_event_sequence
        .map(|sequence| sequence.saturating_add(ORCHESTRATION_EVENT_SEQUENCE_INCREMENT))
        .unwrap_or_default();
    if event.sequence != expected_sequence || events.len() != expected_sequence as usize {
        return Err(StoreError::Conflict("orchestration event sequence is not contiguous".to_string()));
    }
    Ok(())
}

fn reserve_run_id(txn: &redb::WriteTransaction) -> Result<RunId, StoreError> {
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
    Ok(RunId(next_id))
}

fn trigger_run_record_key(record: &TriggerRunRecord) -> String {
    let suffix = record
        .run_id
        .map(|run_id| run_id.0.to_string())
        .or_else(|| record.idempotency_key.clone())
        .unwrap_or_else(|| record.message.clone());
    format!("{}:{suffix}", record.trigger_id)
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
        }
    }
}

impl std::error::Error for StoreError {}

// r[impl n8nclone.workflow_execution.trait_boundaries.storage_ports]
impl WorkflowRepository for Store {
    fn save_workflow(&self, workflow: &Workflow) -> Result<(), StoreError> {
        Store::save_workflow(self, workflow)
    }

    fn load_workflow(&self, name: &str) -> Result<Option<Workflow>, StoreError> {
        Store::load_workflow(self, name)
    }

    fn list_workflows(&self) -> Result<Vec<String>, StoreError> {
        Store::list_workflows(self)
    }

    fn delete_workflow(&self, name: &str) -> Result<(), StoreError> {
        Store::delete_workflow(self, name)
    }
}

// r[impl n8nclone.workflow_execution.trait_boundaries.storage_ports]
impl RunHistoryStore for Store {
    fn save_run(&self, record: &RunRecord) -> Result<(), StoreError> {
        Store::save_run(self, record)
    }

    fn load_run(&self, run_id: RunId) -> Result<Option<RunRecord>, StoreError> {
        Store::load_run(self, run_id)
    }

    fn save_run_details(&self, record: &RunDetailRecord) -> Result<(), StoreError> {
        Store::save_run_details(self, record)
    }

    fn load_run_details(&self, run_id: RunId) -> Result<Option<RunDetailRecord>, StoreError> {
        Store::load_run_details(self, run_id)
    }

    fn save_run_replay_events(&self, run_id: RunId, events: &[ReplayEventRecord]) -> Result<(), StoreError> {
        Store::save_run_replay_events(self, run_id, events)
    }

    fn load_run_replay_events(&self, run_id: RunId) -> Result<Option<Vec<ReplayEventRecord>>, StoreError> {
        Store::load_run_replay_events(self, run_id)
    }

    fn list_runs(&self, limit: usize) -> Result<Vec<RunRecord>, StoreError> {
        Store::list_runs(self, limit)
    }

    fn next_run_id(&self) -> Result<RunId, StoreError> {
        Store::next_run_id(self)
    }
}

// r[impl n8nclone.workflow_execution.trait_boundaries.storage_ports]
impl ExecutionCache for Store {
    fn put_cache(&self, input_hash: &ContentHash, outputs: &HashMap<PortId, Value>) -> Result<(), StoreError> {
        Store::put_cache(self, input_hash, outputs)
    }

    fn get_cache(&self, input_hash: &ContentHash) -> Result<Option<HashMap<PortId, Value>>, StoreError> {
        Store::get_cache(self, input_hash)
    }
}

// r[impl n8nclone.workflow_execution.trait_boundaries.storage_ports]
impl ValueStore for Store {
    fn put_value(&self, value: &Value) -> Result<(), StoreError> {
        Store::put_value(self, value)
    }

    fn get_value(&self, hash: &ContentHash) -> Result<Option<Value>, StoreError> {
        Store::get_value(self, hash)
    }
}

// r[impl n8nclone.workflow_execution.trait_boundaries.storage_ports]
impl ScheduleStore for Store {
    fn save_schedule(&self, schedule: &ScheduleDefinition) -> Result<(), StoreError> {
        Store::save_schedule(self, schedule)
    }

    fn load_schedule(&self, schedule_id: &str) -> Result<Option<ScheduleDefinition>, StoreError> {
        Store::load_schedule(self, schedule_id)
    }

    fn list_schedules(&self) -> Result<Vec<ScheduleDefinition>, StoreError> {
        Store::list_schedules(self)
    }

    fn delete_schedule(&self, schedule_id: &str) -> Result<(), StoreError> {
        Store::delete_schedule(self, schedule_id)
    }
}

impl TriggerStore for Store {
    fn save_trigger(&self, trigger: &TriggerDefinition) -> Result<(), StoreError> {
        Store::save_trigger(self, trigger)
    }

    fn load_trigger(&self, trigger_id: &str) -> Result<Option<TriggerDefinition>, StoreError> {
        Store::load_trigger(self, trigger_id)
    }

    fn list_triggers(&self) -> Result<Vec<TriggerDefinition>, StoreError> {
        Store::list_triggers(self)
    }

    fn delete_trigger(&self, trigger_id: &str) -> Result<(), StoreError> {
        Store::delete_trigger(self, trigger_id)
    }

    fn save_trigger_run(&self, record: &TriggerRunRecord) -> Result<(), StoreError> {
        Store::save_trigger_run(self, record)
    }

    fn list_trigger_runs(&self, limit: usize) -> Result<Vec<TriggerRunRecord>, StoreError> {
        Store::list_trigger_runs(self, limit)
    }
}

impl OrchestrationStore for Store {
    fn save_orchestration_definition(&self, definition: &OrchestrationDefinition) -> Result<(), StoreError> {
        Store::save_orchestration_definition(self, definition)
    }

    fn load_orchestration_definition(&self, id: &str) -> Result<Option<OrchestrationDefinition>, StoreError> {
        Store::load_orchestration_definition(self, id)
    }

    fn next_orchestration_sequence(&self) -> Result<u64, StoreError> {
        Store::next_orchestration_sequence(self)
    }

    fn create_orchestration(
        &self,
        instance: &OrchestrationInstance,
        events: &[OrchestrationEvent],
    ) -> Result<(), StoreError> {
        Store::create_orchestration(self, instance, events)
    }

    fn create_triggered_orchestration(
        &self,
        scoped_key: &str,
        instance: &OrchestrationInstance,
    ) -> Result<TriggerAdmissionWrite, StoreError> {
        Store::create_triggered_orchestration(self, scoped_key, instance)
    }

    fn commit_orchestration_transition(
        &self,
        expected_version: u64,
        instance: &OrchestrationInstance,
        event: &OrchestrationEvent,
    ) -> Result<(), StoreError> {
        Store::commit_orchestration_transition(self, expected_version, instance, event)
    }

    fn load_orchestration(&self, id: &str) -> Result<Option<OrchestrationInstance>, StoreError> {
        Store::load_orchestration(self, id)
    }

    fn load_orchestration_events(&self, id: &str) -> Result<Vec<OrchestrationEvent>, StoreError> {
        Store::load_orchestration_events(self, id)
    }

    fn list_orchestrations(&self) -> Result<Vec<OrchestrationInstance>, StoreError> {
        Store::list_orchestrations(self)
    }

    fn save_orchestration_trigger_admission(
        &self,
        scoped_key: &str,
        instance_id: &str,
    ) -> Result<TriggerAdmissionWrite, StoreError> {
        Store::save_orchestration_trigger_admission(self, scoped_key, instance_id)
    }

    fn list_orchestration_trigger_admissions(&self) -> Result<HashMap<String, String>, StoreError> {
        Store::list_orchestration_trigger_admissions(self)
    }
}

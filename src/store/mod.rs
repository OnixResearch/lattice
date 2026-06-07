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

use crate::engine::ContentHash;
use crate::engine::RunId;
use crate::engine::Value;
use crate::engine::Workflow;
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
const SCHEDULES: TableDefinition<&str, &[u8]> = TableDefinition::new("schedules");
const TRIGGERS: TableDefinition<&str, &[u8]> = TableDefinition::new("triggers");
const TRIGGER_RUNS: TableDefinition<&str, &[u8]> = TableDefinition::new("trigger_runs");
const INITIAL_RUN_ID_VALUE: u64 = 0;
const NEXT_RUN_ID_INCREMENT: u64 = 1;

/// Serializable run record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunRecord {
    pub run_id: RunId,
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
            let _ = txn.open_table(SCHEDULES).map_err(|e| StoreError::Table(e.to_string()))?;
            let _ = txn.open_table(TRIGGERS).map_err(|e| StoreError::Table(e.to_string()))?;
            let _ = txn.open_table(TRIGGER_RUNS).map_err(|e| StoreError::Table(e.to_string()))?;
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

    /// Store a run record.
    pub fn save_run(&self, record: &RunRecord) -> Result<(), StoreError> {
        let data = serde_json::to_vec(record).map_err(|e| StoreError::Serialize(e.to_string()))?;
        let txn = self.db.begin_write().map_err(|e| StoreError::Db(e.to_string()))?;
        {
            let mut table = txn.open_table(RUNS).map_err(|e| StoreError::Table(e.to_string()))?;
            table.insert(record.run_id.0, data.as_slice()).map_err(|e| StoreError::Db(e.to_string()))?;
        }
        txn.commit().map_err(|e| StoreError::Db(e.to_string()))?;
        Ok(())
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

    /// Return the next run id after currently persisted run records.
    pub fn next_run_id(&self) -> Result<RunId, StoreError> {
        let txn = self.db.begin_read().map_err(|e| StoreError::Db(e.to_string()))?;
        let table = txn.open_table(RUNS).map_err(|e| StoreError::Table(e.to_string()))?;
        let mut next_id = INITIAL_RUN_ID_VALUE;
        let iter = table.iter().map_err(|e| StoreError::Db(e.to_string()))?;
        for entry in iter {
            let (key, _data) = entry.map_err(|e| StoreError::Db(e.to_string()))?;
            let candidate = key.value().saturating_add(NEXT_RUN_ID_INCREMENT);
            next_id = next_id.max(candidate);
        }
        Ok(RunId(next_id))
    }
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
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Db(e) => write!(f, "database error: {e}"),
            Self::Table(e) => write!(f, "table error: {e}"),
            Self::Serialize(e) => write!(f, "serialization error: {e}"),
            Self::Deserialize(e) => write!(f, "deserialization error: {e}"),
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

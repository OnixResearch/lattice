//! redb-backed persistent storage.
//!
//! Tables:
//!  - workflows: name → serialized Workflow
//!  - values: ContentHash → serialized Value (dedup blob store)
//!  - runs: RunId → execution log
//!  - cache: input_hash → output values (skip re-execution)

use crate::engine::{ContentHash, RunId, Value, Workflow};
use rat_nodegraph::model::PortId;
use redb::{Database, ReadableDatabase, ReadableTable, TableDefinition};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

const WORKFLOWS: TableDefinition<&str, &[u8]> = TableDefinition::new("workflows");
const VALUES: TableDefinition<&[u8], &[u8]> = TableDefinition::new("values");
const CACHE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("cache");
const RUNS: TableDefinition<u64, &[u8]> = TableDefinition::new("runs");

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
                let wf: Workflow = serde_json::from_slice(bytes)
                    .map_err(|e| StoreError::Deserialize(e.to_string()))?;
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

    // -- value store (content-addressed) ------------------------------------

    /// Store a value by its content hash.
    pub fn put_value(&self, value: &Value) -> Result<(), StoreError> {
        let data = serde_json::to_vec(value).map_err(|e| StoreError::Serialize(e.to_string()))?;
        let txn = self.db.begin_write().map_err(|e| StoreError::Db(e.to_string()))?;
        {
            let mut table = txn.open_table(VALUES).map_err(|e| StoreError::Table(e.to_string()))?;
            table
                .insert(value.hash.0.as_slice(), data.as_slice())
                .map_err(|e| StoreError::Db(e.to_string()))?;
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
                let val: Value = serde_json::from_slice(bytes)
                    .map_err(|e| StoreError::Deserialize(e.to_string()))?;
                Ok(Some(val))
            }
            None => Ok(None),
        }
    }

    // -- execution cache ----------------------------------------------------

    /// Store cached outputs for an input hash.
    pub fn put_cache(
        &self,
        input_hash: &ContentHash,
        outputs: &HashMap<PortId, Value>,
    ) -> Result<(), StoreError> {
        let data =
            serde_json::to_vec(outputs).map_err(|e| StoreError::Serialize(e.to_string()))?;
        let txn = self.db.begin_write().map_err(|e| StoreError::Db(e.to_string()))?;
        {
            let mut table = txn.open_table(CACHE).map_err(|e| StoreError::Table(e.to_string()))?;
            table
                .insert(input_hash.0.as_slice(), data.as_slice())
                .map_err(|e| StoreError::Db(e.to_string()))?;
        }
        txn.commit().map_err(|e| StoreError::Db(e.to_string()))?;
        Ok(())
    }

    /// Retrieve cached outputs for an input hash.
    pub fn get_cache(
        &self,
        input_hash: &ContentHash,
    ) -> Result<Option<HashMap<PortId, Value>>, StoreError> {
        let txn = self.db.begin_read().map_err(|e| StoreError::Db(e.to_string()))?;
        let table = txn.open_table(CACHE).map_err(|e| StoreError::Table(e.to_string()))?;
        match table.get(input_hash.0.as_slice()).map_err(|e| StoreError::Db(e.to_string()))? {
            Some(data) => {
                let bytes: &[u8] = data.value();
                let map: HashMap<PortId, Value> = serde_json::from_slice(bytes)
                    .map_err(|e| StoreError::Deserialize(e.to_string()))?;
                Ok(Some(map))
            }
            None => Ok(None),
        }
    }

    // -- run records --------------------------------------------------------

    /// Store a run record.
    pub fn save_run(&self, record: &RunRecord) -> Result<(), StoreError> {
        let data =
            serde_json::to_vec(record).map_err(|e| StoreError::Serialize(e.to_string()))?;
        let txn = self.db.begin_write().map_err(|e| StoreError::Db(e.to_string()))?;
        {
            let mut table = txn.open_table(RUNS).map_err(|e| StoreError::Table(e.to_string()))?;
            table
                .insert(record.run_id.0, data.as_slice())
                .map_err(|e| StoreError::Db(e.to_string()))?;
        }
        txn.commit().map_err(|e| StoreError::Db(e.to_string()))?;
        Ok(())
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
            let rec: RunRecord = serde_json::from_slice(bytes)
                .map_err(|e| StoreError::Deserialize(e.to_string()))?;
            records.push(rec);
        }
        records.reverse();
        records.truncate(limit);
        Ok(records)
    }
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

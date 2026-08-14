impl Store {
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

    // r[impl lattice.agent_orchestration.durability]
}

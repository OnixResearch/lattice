use redb::ReadableTable as _;

impl crate::store::Store {
    /// Store a run record without allowing a global run identity to be overwritten.
    // r[impl lattice.agent_orchestration.run_identity]
    pub fn save_run(&self, record: &crate::store::RunRecord) -> Result<(), crate::store::StoreError> {
        let data = serde_json::to_vec(record).map_err(|e| crate::store::StoreError::Serialize(e.to_string()))?;
        let next_after_record =
            record.run_id.0.checked_add(crate::store::NEXT_RUN_ID_INCREMENT).ok_or_else(|| {
                crate::store::StoreError::Conflict(crate::store::RUN_ID_EXHAUSTED_MESSAGE.to_string())
            })?;
        let txn = self.db.begin_write().map_err(|e| crate::store::StoreError::Db(e.to_string()))?;
        {
            let mut table =
                txn.open_table(crate::store::RUNS).map_err(|e| crate::store::StoreError::Table(e.to_string()))?;
            if table.get(record.run_id.0).map_err(|e| crate::store::StoreError::Db(e.to_string()))?.is_some() {
                return Err(crate::store::StoreError::Conflict(format!(
                    "run identity {} is already persisted",
                    record.run_id
                )));
            }
            table
                .insert(record.run_id.0, data.as_slice())
                .map_err(|e| crate::store::StoreError::Db(e.to_string()))?;
        }
        {
            let mut counters = txn
                .open_table(crate::store::RUN_COUNTERS)
                .map_err(|e| crate::store::StoreError::Table(e.to_string()))?;
            let current = counters
                .get(crate::store::RUN_ID_COUNTER_KEY)
                .map_err(|e| crate::store::StoreError::Db(e.to_string()))?
                .map(|value| value.value())
                .unwrap_or(crate::store::INITIAL_RUN_ID_VALUE);
            if next_after_record > current {
                counters
                    .insert(crate::store::RUN_ID_COUNTER_KEY, next_after_record)
                    .map_err(|e| crate::store::StoreError::Db(e.to_string()))?;
            }
        }
        txn.commit().map_err(|e| crate::store::StoreError::Db(e.to_string()))?;
        Ok(())
    }
}

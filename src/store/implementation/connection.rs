impl crate::store::Store {
    /// Open or create the database at the given path.
    pub fn open(path: impl AsRef<std::path::Path>) -> Result<(), crate::store::StoreError> {
        Self::new(path).map(|_| ())
    }

    fn open_database(path: &std::path::Path) -> Result<redb::Database, crate::store::StoreError> {
        let db = redb::Database::create(path).map_err(|e| crate::store::StoreError::Db(e.to_string()))?;

        // Ensure tables exist.
        let txn = db.begin_write().map_err(|e| crate::store::StoreError::Db(e.to_string()))?;
        {
            let _ = txn
                .open_table(crate::store::WORKFLOWS)
                .map_err(|e| crate::store::StoreError::Table(e.to_string()))?;
            let _ = txn.open_table(crate::store::VALUES).map_err(|e| crate::store::StoreError::Table(e.to_string()))?;
            let _ = txn.open_table(crate::store::CACHE).map_err(|e| crate::store::StoreError::Table(e.to_string()))?;
            let _ = txn.open_table(crate::store::RUNS).map_err(|e| crate::store::StoreError::Table(e.to_string()))?;
            let _ = txn
                .open_table(crate::store::RUN_DETAILS)
                .map_err(|e| crate::store::StoreError::Table(e.to_string()))?;
            let _ = txn
                .open_table(crate::store::RUN_REPLAY_EVENTS)
                .map_err(|e| crate::store::StoreError::Table(e.to_string()))?;
            let _ = txn
                .open_table(crate::store::RUN_COUNTERS)
                .map_err(|e| crate::store::StoreError::Table(e.to_string()))?;
            let _ = txn
                .open_table(crate::store::SCHEDULES)
                .map_err(|e| crate::store::StoreError::Table(e.to_string()))?;
            crate::store::admission::ensure_table(&txn)?;
            let _ =
                txn.open_table(crate::store::TRIGGERS).map_err(|e| crate::store::StoreError::Table(e.to_string()))?;
            let _ = txn
                .open_table(crate::store::TRIGGER_RUNS)
                .map_err(|e| crate::store::StoreError::Table(e.to_string()))?;
            let _ = txn
                .open_table(crate::store::WEBHOOK_CLAIMS)
                .map_err(|e| crate::store::StoreError::Table(e.to_string()))?;
            let _ = txn
                .open_table(crate::store::ORCHESTRATION_DEFINITIONS)
                .map_err(|e| crate::store::StoreError::Table(e.to_string()))?;
            let _ = txn
                .open_table(crate::store::ORCHESTRATION_INSTANCES)
                .map_err(|e| crate::store::StoreError::Table(e.to_string()))?;
            let _ = txn
                .open_table(crate::store::ORCHESTRATION_EVENTS)
                .map_err(|e| crate::store::StoreError::Table(e.to_string()))?;
            let _ = txn
                .open_table(crate::store::ORCHESTRATION_TRIGGER_ADMISSIONS)
                .map_err(|e| crate::store::StoreError::Table(e.to_string()))?;
            let _ = txn
                .open_table(crate::store::ORCHESTRATION_COUNTERS)
                .map_err(|e| crate::store::StoreError::Table(e.to_string()))?;
            let _ = txn
                .open_table(crate::store::EFFECT_ATTEMPTS)
                .map_err(|e| crate::store::StoreError::Table(e.to_string()))?;
            let _ = txn
                .open_table(crate::store::EFFECT_OUTCOMES)
                .map_err(|e| crate::store::StoreError::Table(e.to_string()))?;
            let _ = txn
                .open_table(crate::store::PROTOCOL_SESSIONS)
                .map_err(|e| crate::store::StoreError::Table(e.to_string()))?;
            let _ = txn
                .open_table(crate::store::PROTOCOL_EVENTS)
                .map_err(|e| crate::store::StoreError::Table(e.to_string()))?;
            let _ = txn
                .open_table(crate::store::WORKFLOW_TEMPLATE_ACTIVATIONS)
                .map_err(|error| crate::store::StoreError::Table(error.to_string()))?;
            let _ = txn
                .open_table(crate::store::EVALUATION_TRIALS)
                .map_err(|error| crate::store::StoreError::Table(error.to_string()))?;
            let _ = txn
                .open_table(crate::store::EVALUATION_RECEIPTS)
                .map_err(|error| crate::store::StoreError::Table(error.to_string()))?;
            let _ = txn
                .open_table(crate::store::OPERATION_RUNS)
                .map_err(|error| crate::store::StoreError::Table(error.to_string()))?;
            let _ = txn
                .open_table(crate::store::WORKFLOW_EXCHANGE_EXECUTIONS)
                .map_err(|error| crate::store::StoreError::Table(error.to_string()))?;
            let _ = txn
                .open_table(crate::store::WORKFLOW_EXCHANGE_REQUEST_INDEX)
                .map_err(|error| crate::store::StoreError::Table(error.to_string()))?;
            let _ = txn
                .open_table(crate::store::WORKFLOW_EXCHANGE_IDEMPOTENCY_INDEX)
                .map_err(|error| crate::store::StoreError::Table(error.to_string()))?;
            let _ = txn
                .open_table(crate::store::CANDIDATE_RECEIPTS)
                .map_err(|error| crate::store::StoreError::Table(error.to_string()))?;
            let _ = txn
                .open_table(crate::store::CANDIDATE_ARTIFACTS)
                .map_err(|error| crate::store::StoreError::Table(error.to_string()))?;
            let _ = txn
                .open_table(crate::store::COMMAND_WORKER_USAGE)
                .map_err(|error| crate::store::StoreError::Table(error.to_string()))?;
            let _ = txn
                .open_table(crate::store::COMMAND_WORKER_USAGE_BY_RUN)
                .map_err(|error| crate::store::StoreError::Table(error.to_string()))?;
            let _ = txn
                .open_table(crate::store::CANDIDATE_ATTEMPT_RESULTS)
                .map_err(|error| crate::store::StoreError::Table(error.to_string()))?;
            crate::store::ensure_operation_data_tables(&txn)?;
            crate::store::quarantine::fence_pending(&txn)?;
        }
        txn.commit().map_err(|e| crate::store::StoreError::Db(e.to_string()))?;
        Ok(db)
    }

    /// Open and return a Store instance.
    pub fn new(path: impl AsRef<std::path::Path>) -> Result<Self, crate::store::StoreError> {
        let db = Self::open_database(path.as_ref())?;
        Ok(Self {
            db: std::sync::Arc::new(db),
            quarantine: std::sync::Arc::new(std::sync::Mutex::new(None)),
            occurrence_admission: std::sync::Arc::new(std::sync::Mutex::new(())),
        })
    }
}

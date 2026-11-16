use redb::ReadableDatabase as _;
use redb::ReadableTable as _;

impl crate::store::Store {
    /// Persist immutable usage content, allowing only monotonic host outcome refinement.
    pub fn save_command_worker_usage(
        &self,
        observation: &crate::orchestration::CommandWorkerObservation,
    ) -> Result<(), crate::store::StoreError> {
        self.ensure_not_quarantined()?;
        let mut data = observation.to_json().map_err(|error| crate::store::StoreError::Serialize(error.to_string()))?;
        let txn = self.db.begin_write().map_err(|error| crate::store::StoreError::Db(error.to_string()))?;
        {
            let mut table = txn
                .open_table(crate::store::COMMAND_WORKER_USAGE)
                .map_err(|error| crate::store::StoreError::Table(error.to_string()))?;
            if let Some(existing) = table
                .get(observation.attempt_id.as_str())
                .map_err(|error| crate::store::StoreError::Db(error.to_string()))?
            {
                let text = std::str::from_utf8(existing.value())
                    .map_err(|error| crate::store::StoreError::Deserialize(error.to_string()))?;
                let mut existing = crate::orchestration::CommandWorkerObservation::from_json(text)
                    .map_err(|error| crate::store::StoreError::Deserialize(error.to_string()))?;
                let previous_outcome = existing.accounting.as_ref().and_then(|accounting| accounting.outcome);
                if let (Some(current), Some(incoming)) = (&mut existing.accounting, &observation.accounting) {
                    current.outcome = incoming.outcome;
                }
                if existing != *observation {
                    return Err(crate::store::StoreError::Conflict(
                        "command-worker usage already exists with different content".to_string(),
                    ));
                }
                if let Some(accounting) = &mut existing.accounting {
                    let incoming = accounting.outcome;
                    accounting.outcome = previous_outcome;
                    if let Some(incoming) = incoming {
                        accounting
                            .refine_outcome(incoming)
                            .map_err(|error| crate::store::StoreError::Conflict(error.to_string()))?;
                    }
                }
                data = existing.to_json().map_err(|error| crate::store::StoreError::Serialize(error.to_string()))?;
            }
            table
                .insert(observation.attempt_id.as_str(), data.as_bytes())
                .map_err(|error| crate::store::StoreError::Db(error.to_string()))?;
        }
        {
            let mut index = txn
                .open_table(crate::store::COMMAND_WORKER_USAGE_BY_RUN)
                .map_err(|error| crate::store::StoreError::Table(error.to_string()))?;
            if let Some(existing) =
                index.get(observation.run_id).map_err(|error| crate::store::StoreError::Db(error.to_string()))?
            {
                if existing.value() != observation.attempt_id.as_str() {
                    return Err(crate::store::StoreError::Conflict(
                        "run already has a different command-worker usage observation".to_string(),
                    ));
                }
            } else {
                index
                    .insert(observation.run_id, observation.attempt_id.as_str())
                    .map_err(|error| crate::store::StoreError::Db(error.to_string()))?;
            }
        }
        txn.commit().map_err(|error| crate::store::StoreError::Db(error.to_string()))?;
        Ok(())
    }

    /// Load and revalidate one command-worker usage observation.
    pub fn load_command_worker_usage(
        &self,
        attempt_id: &str,
    ) -> Result<Option<crate::orchestration::CommandWorkerObservation>, crate::store::StoreError> {
        let txn = self.db.begin_read().map_err(|error| crate::store::StoreError::Db(error.to_string()))?;
        let table = txn
            .open_table(crate::store::COMMAND_WORKER_USAGE)
            .map_err(|error| crate::store::StoreError::Table(error.to_string()))?;
        match table.get(attempt_id).map_err(|error| crate::store::StoreError::Db(error.to_string()))? {
            Some(data) => {
                let text = std::str::from_utf8(data.value())
                    .map_err(|error| crate::store::StoreError::Deserialize(error.to_string()))?;
                let observation = crate::orchestration::CommandWorkerObservation::from_json(text)
                    .map_err(|error| crate::store::StoreError::Deserialize(error.to_string()))?;
                if observation.attempt_id != attempt_id {
                    return Err(crate::store::StoreError::Conflict(
                        "usage observation attempt key mismatch".to_string(),
                    ));
                }
                Ok(Some(observation))
            }
            None => Ok(None),
        }
    }

    /// Load the command-worker usage observation indexed by run identity.
    pub fn load_command_worker_usage_for_run(
        &self,
        run_id: crate::engine::executor::RunId,
    ) -> Result<Option<crate::orchestration::CommandWorkerObservation>, crate::store::StoreError> {
        let txn = self.db.begin_read().map_err(|error| crate::store::StoreError::Db(error.to_string()))?;
        let index = txn
            .open_table(crate::store::COMMAND_WORKER_USAGE_BY_RUN)
            .map_err(|error| crate::store::StoreError::Table(error.to_string()))?;
        let Some(attempt_id) = index.get(run_id.0).map_err(|error| crate::store::StoreError::Db(error.to_string()))?
        else {
            return Ok(None);
        };
        let attempt_id = attempt_id.value().to_string();
        let table = txn
            .open_table(crate::store::COMMAND_WORKER_USAGE)
            .map_err(|error| crate::store::StoreError::Table(error.to_string()))?;
        match table.get(attempt_id.as_str()).map_err(|error| crate::store::StoreError::Db(error.to_string()))? {
            Some(data) => {
                let text = std::str::from_utf8(data.value())
                    .map_err(|error| crate::store::StoreError::Deserialize(error.to_string()))?;
                let observation = crate::orchestration::CommandWorkerObservation::from_json(text)
                    .map_err(|error| crate::store::StoreError::Deserialize(error.to_string()))?;
                if observation.attempt_id != attempt_id || observation.run_id != run_id.0 {
                    return Err(crate::store::StoreError::Conflict("usage observation run index mismatch".to_string()));
                }
                Ok(Some(observation))
            }
            None => Err(crate::store::StoreError::Conflict(
                "usage observation run index points to a missing receipt".to_string(),
            )),
        }
    }
}

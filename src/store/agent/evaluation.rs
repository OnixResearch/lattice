const MAX_EVALUATION_RECORD_LIMIT_COUNT: u32 = 100;

impl Store {
    pub fn record_evaluation_start(
        &self,
        trial: &crate::agent_evaluation::planning::TrialPlan,
        observed_epoch_ms: u64,
    ) -> Result<(), StoreError> {
        let record = crate::agent_evaluation::persistence::TrialRecord {
            plan: trial.clone(),
            status: crate::agent_evaluation::model::TrialStatus::Started,
            started_epoch_ms: observed_epoch_ms,
            updated_epoch_ms: observed_epoch_ms,
            cancel_requested: false,
            receipt_identity: None,
        };
        let data = serialize(&record)?;
        let transaction = self.db.begin_write().map_err(|error| StoreError::Db(error.to_string()))?;
        {
            let mut table =
                transaction.open_table(EVALUATION_TRIALS).map_err(|error| StoreError::Table(error.to_string()))?;
            if table
                .get(trial.trial_identity.as_str())
                .map_err(|error| StoreError::Db(error.to_string()))?
                .is_some()
            {
                return Err(StoreError::Conflict("evaluation trial already exists".to_string()));
            }
            table
                .insert(trial.trial_identity.as_str(), data.as_slice())
                .map_err(|error| StoreError::Db(error.to_string()))?;
        }
        transaction.commit().map_err(|error| StoreError::Db(error.to_string()))
    }

    pub fn load_evaluation_trial(
        &self,
        trial_identity: &str,
    ) -> Result<Option<crate::agent_evaluation::persistence::TrialRecord>, StoreError> {
        crate::agent_evaluation::identity::validate(trial_identity)
            .map_err(|_| StoreError::NotFound("invalid evaluation trial identity".to_string()))?;
        let transaction = self.db.begin_read().map_err(|error| StoreError::Db(error.to_string()))?;
        let table = transaction.open_table(EVALUATION_TRIALS).map_err(|error| StoreError::Table(error.to_string()))?;
        deserialize_optional(table.get(trial_identity).map_err(|error| StoreError::Db(error.to_string()))?)
    }

    pub fn load_evaluation_receipt(
        &self,
        receipt_identity: &str,
    ) -> Result<Option<crate::agent_evaluation::evidence::Receipt>, StoreError> {
        crate::agent_evaluation::identity::validate(receipt_identity)
            .map_err(|_| StoreError::NotFound("invalid evaluation receipt identity".to_string()))?;
        let transaction = self.db.begin_read().map_err(|error| StoreError::Db(error.to_string()))?;
        let table =
            transaction.open_table(EVALUATION_RECEIPTS).map_err(|error| StoreError::Table(error.to_string()))?;
        deserialize_optional(table.get(receipt_identity).map_err(|error| StoreError::Db(error.to_string()))?)
    }

    pub fn list_evaluation_trials(
        &self,
        maximum_records: u32,
    ) -> Result<Vec<crate::agent_evaluation::persistence::TrialRecord>, StoreError> {
        if maximum_records == 0 || maximum_records > MAX_EVALUATION_RECORD_LIMIT_COUNT {
            return Err(StoreError::Conflict("evaluation trial list limit is invalid".to_string()));
        }
        let maximum_records =
            usize::try_from(maximum_records).map_err(|error| StoreError::Conflict(error.to_string()))?;
        let transaction = self.db.begin_read().map_err(|error| StoreError::Db(error.to_string()))?;
        let table = transaction.open_table(EVALUATION_TRIALS).map_err(|error| StoreError::Table(error.to_string()))?;
        let mut records = Vec::with_capacity(maximum_records);
        for entry in table.iter().map_err(|error| StoreError::Db(error.to_string()))? {
            if records.len() >= maximum_records {
                break;
            }
            let (_, value) = entry.map_err(|error| StoreError::Db(error.to_string()))?;
            records.push(deserialize(value.value())?);
        }
        records.sort_by(|left: &crate::agent_evaluation::persistence::TrialRecord, right| {
            right
                .updated_epoch_ms
                .cmp(&left.updated_epoch_ms)
                .then(left.plan.trial_identity.cmp(&right.plan.trial_identity))
        });
        debug_assert!(records.len() <= maximum_records);
        Ok(records)
    }

    pub fn request_evaluation_cancel(&self, trial_identity: &str, observed_epoch_ms: u64) -> Result<(), StoreError> {
        self.update_evaluation_trial(trial_identity, |record| {
            if record.status != crate::agent_evaluation::model::TrialStatus::Started {
                return Err(StoreError::Conflict("evaluation trial is not running".to_string()));
            }
            record.cancel_requested = true;
            record.updated_epoch_ms = observed_epoch_ms;
            Ok(())
        })
    }

    fn evaluation_cancellation_requested(&self, trial_identity: &str) -> Result<bool, StoreError> {
        let record = self
            .load_evaluation_trial(trial_identity)?
            .ok_or_else(|| StoreError::NotFound("evaluation trial is missing".to_string()))?;
        Ok(record.cancel_requested)
    }

    fn record_evaluation_status(
        &self,
        trial_identity: &str,
        status: crate::agent_evaluation::model::TrialStatus,
        observed_epoch_ms: u64,
    ) -> Result<(), StoreError> {
        self.update_evaluation_trial(trial_identity, |record| {
            if record.status != crate::agent_evaluation::model::TrialStatus::Started {
                return Err(StoreError::Conflict("evaluation trial is not running".to_string()));
            }
            record.status = status;
            record.updated_epoch_ms = observed_epoch_ms;
            Ok(())
        })
    }

    fn record_evaluation_receipt(
        &self,
        receipt: &crate::agent_evaluation::evidence::Receipt,
        observed_epoch_ms: u64,
    ) -> Result<(), StoreError> {
        let receipt_data = serialize(receipt)?;
        let transaction = self.db.begin_write().map_err(|error| StoreError::Db(error.to_string()))?;
        {
            let mut trial_table =
                transaction.open_table(EVALUATION_TRIALS).map_err(|error| StoreError::Table(error.to_string()))?;
            let stored = trial_table
                .get(receipt.trial_identity.as_str())
                .map_err(|error| StoreError::Db(error.to_string()))?
                .ok_or_else(|| StoreError::NotFound("evaluation trial is missing".to_string()))?;
            let mut record: crate::agent_evaluation::persistence::TrialRecord = deserialize(stored.value())?;
            if record.status != crate::agent_evaluation::model::TrialStatus::Started
                || receipt.status == crate::agent_evaluation::model::TrialStatus::Started
                || receipt.status == crate::agent_evaluation::model::TrialStatus::Planned
            {
                return Err(StoreError::Conflict("evaluation trial transition is invalid".to_string()));
            }
            record.status = receipt.status;
            record.updated_epoch_ms = observed_epoch_ms;
            record.receipt_identity = Some(receipt.identity.clone());
            let record_data = serialize(&record)?;
            drop(stored);
            trial_table
                .insert(receipt.trial_identity.as_str(), record_data.as_slice())
                .map_err(|error| StoreError::Db(error.to_string()))?;
            let mut receipt_table =
                transaction.open_table(EVALUATION_RECEIPTS).map_err(|error| StoreError::Table(error.to_string()))?;
            if receipt_table
                .get(receipt.identity.as_str())
                .map_err(|error| StoreError::Db(error.to_string()))?
                .is_some()
            {
                return Err(StoreError::Conflict("evaluation receipt already exists".to_string()));
            }
            receipt_table
                .insert(receipt.identity.as_str(), receipt_data.as_slice())
                .map_err(|error| StoreError::Db(error.to_string()))?;
        }
        transaction.commit().map_err(|error| StoreError::Db(error.to_string()))
    }

    fn update_evaluation_trial<F>(&self, trial_identity: &str, update: F) -> Result<(), StoreError>
    where F: FnOnce(&mut crate::agent_evaluation::persistence::TrialRecord) -> Result<(), StoreError> {
        crate::agent_evaluation::identity::validate(trial_identity)
            .map_err(|_| StoreError::NotFound("invalid evaluation trial identity".to_string()))?;
        let transaction = self.db.begin_write().map_err(|error| StoreError::Db(error.to_string()))?;
        {
            let mut table =
                transaction.open_table(EVALUATION_TRIALS).map_err(|error| StoreError::Table(error.to_string()))?;
            let stored = table
                .get(trial_identity)
                .map_err(|error| StoreError::Db(error.to_string()))?
                .ok_or_else(|| StoreError::NotFound("evaluation trial is missing".to_string()))?;
            let mut record: crate::agent_evaluation::persistence::TrialRecord = deserialize(stored.value())?;
            update(&mut record)?;
            let data = serialize(&record)?;
            drop(stored);
            table.insert(trial_identity, data.as_slice()).map_err(|error| StoreError::Db(error.to_string()))?;
        }
        transaction.commit().map_err(|error| StoreError::Db(error.to_string()))
    }
}

impl crate::agent_evaluation::shell::TrialStorePort for Store {
    fn record_start(
        &self,
        trial: &crate::agent_evaluation::planning::TrialPlan,
        observed_epoch_ms: u64,
    ) -> Result<(), crate::agent_evaluation::Error> {
        self.record_evaluation_start(trial, observed_epoch_ms)
            .map_err(|_| crate::agent_evaluation::Error::ServiceFailure)
    }

    fn cancellation_requested(
        &self,
        trial: &crate::agent_evaluation::planning::TrialPlan,
    ) -> Result<bool, crate::agent_evaluation::Error> {
        self.evaluation_cancellation_requested(&trial.trial_identity)
            .map_err(|_| crate::agent_evaluation::Error::ServiceFailure)
    }

    fn record_cancelled(
        &self,
        trial: &crate::agent_evaluation::planning::TrialPlan,
        observed_epoch_ms: u64,
    ) -> Result<(), crate::agent_evaluation::Error> {
        self.record_evaluation_status(
            &trial.trial_identity,
            crate::agent_evaluation::model::TrialStatus::Cancelled,
            observed_epoch_ms,
        )
        .map_err(|_| crate::agent_evaluation::Error::ServiceFailure)
    }

    fn record_unknown(
        &self,
        trial: &crate::agent_evaluation::planning::TrialPlan,
        observed_epoch_ms: u64,
    ) -> Result<(), crate::agent_evaluation::Error> {
        self.record_evaluation_status(
            &trial.trial_identity,
            crate::agent_evaluation::model::TrialStatus::Unknown,
            observed_epoch_ms,
        )
        .map_err(|_| crate::agent_evaluation::Error::ServiceFailure)
    }

    fn record_receipt(
        &self,
        receipt: &crate::agent_evaluation::evidence::Receipt,
        observed_epoch_ms: u64,
    ) -> Result<(), crate::agent_evaluation::Error> {
        self.record_evaluation_receipt(receipt, observed_epoch_ms)
            .map_err(|_| crate::agent_evaluation::Error::ServiceFailure)
    }
}

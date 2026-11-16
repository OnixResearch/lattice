use redb::ReadableDatabase as _;
use redb::ReadableTable as _;

impl crate::store::Store {
    // r[impl n8nclone.scheduling.triggers.model]
    pub fn save_trigger(&self, trigger: &crate::triggers::TriggerDefinition) -> Result<(), crate::store::StoreError> {
        let data = serde_json::to_vec(trigger).map_err(|e| crate::store::StoreError::Serialize(e.to_string()))?;
        let txn = self.db.begin_write().map_err(|e| crate::store::StoreError::Db(e.to_string()))?;
        {
            let mut table =
                txn.open_table(crate::store::TRIGGERS).map_err(|e| crate::store::StoreError::Table(e.to_string()))?;
            table
                .insert(trigger.id.as_str(), data.as_slice())
                .map_err(|e| crate::store::StoreError::Db(e.to_string()))?;
        }
        txn.commit().map_err(|e| crate::store::StoreError::Db(e.to_string()))?;
        Ok(())
    }

    pub fn load_trigger(
        &self,
        trigger_id: &str,
    ) -> Result<Option<crate::triggers::TriggerDefinition>, crate::store::StoreError> {
        let txn = self.db.begin_read().map_err(|e| crate::store::StoreError::Db(e.to_string()))?;
        let table =
            txn.open_table(crate::store::TRIGGERS).map_err(|e| crate::store::StoreError::Table(e.to_string()))?;
        match table.get(trigger_id).map_err(|e| crate::store::StoreError::Db(e.to_string()))? {
            Some(data) => {
                let bytes: &[u8] = data.value();
                let trigger =
                    serde_json::from_slice(bytes).map_err(|e| crate::store::StoreError::Deserialize(e.to_string()))?;
                Ok(Some(trigger))
            }
            None => Ok(None),
        }
    }

    pub fn list_triggers(&self) -> Result<Vec<crate::triggers::TriggerDefinition>, crate::store::StoreError> {
        let txn = self.db.begin_read().map_err(|e| crate::store::StoreError::Db(e.to_string()))?;
        let table =
            txn.open_table(crate::store::TRIGGERS).map_err(|e| crate::store::StoreError::Table(e.to_string()))?;
        let mut triggers = Vec::with_capacity(crate::store::implementation::entry_count(&table)?);
        let iter = table.iter().map_err(|e| crate::store::StoreError::Db(e.to_string()))?;
        for entry in iter {
            let (_key, data) = entry.map_err(|e| crate::store::StoreError::Db(e.to_string()))?;
            let bytes: &[u8] = data.value();
            let trigger =
                serde_json::from_slice(bytes).map_err(|e| crate::store::StoreError::Deserialize(e.to_string()))?;
            triggers.push(trigger);
        }
        Ok(triggers)
    }

    pub fn delete_trigger(&self, trigger_id: &str) -> Result<(), crate::store::StoreError> {
        let txn = self.db.begin_write().map_err(|e| crate::store::StoreError::Db(e.to_string()))?;
        {
            let mut table =
                txn.open_table(crate::store::TRIGGERS).map_err(|e| crate::store::StoreError::Table(e.to_string()))?;
            table.remove(trigger_id).map_err(|e| crate::store::StoreError::Db(e.to_string()))?;
        }
        txn.commit().map_err(|e| crate::store::StoreError::Db(e.to_string()))?;
        Ok(())
    }

    // r[impl n8nclone.scheduling.triggers.history]
    pub fn save_trigger_run(&self, record: &crate::triggers::TriggerRunRecord) -> Result<(), crate::store::StoreError> {
        let data = serde_json::to_vec(record).map_err(|e| crate::store::StoreError::Serialize(e.to_string()))?;
        let key = trigger_run_record_key(record);
        let txn = self.db.begin_write().map_err(|e| crate::store::StoreError::Db(e.to_string()))?;
        {
            let mut table = txn
                .open_table(crate::store::TRIGGER_RUNS)
                .map_err(|e| crate::store::StoreError::Table(e.to_string()))?;
            table
                .insert(key.as_str(), data.as_slice())
                .map_err(|e| crate::store::StoreError::Db(e.to_string()))?;
        }
        txn.commit().map_err(|e| crate::store::StoreError::Db(e.to_string()))?;
        Ok(())
    }

    // r[impl lattice.scheduling.webhook_atomic_idempotency]
    pub fn claim_webhook(
        &self,
        request: &crate::triggers::WebhookClaimRequest,
    ) -> Result<crate::triggers::WebhookClaimResolution, crate::store::StoreError> {
        let txn = self.db.begin_write().map_err(|error| crate::store::StoreError::Db(error.to_string()))?;
        {
            let claims = txn
                .open_table(crate::store::WEBHOOK_CLAIMS)
                .map_err(|error| crate::store::StoreError::Table(error.to_string()))?;
            if let Some(existing) = claims
                .get(request.identity.as_str())
                .map_err(|error| crate::store::StoreError::Db(error.to_string()))?
            {
                let claim: crate::triggers::WebhookClaim = crate::store::deserialize(existing.value())?;
                return Ok(crate::triggers::WebhookClaimResolution::Duplicate {
                    existing_run_id: claim.run_id,
                });
            }
        }

        let run_id = crate::store::reserve_run_id(&txn)?;
        let claim = request.claim(run_id);
        let record = request.started_record(run_id);
        let claim_data = crate::store::serialize(&claim)?;
        let record_data = crate::store::serialize(&record)?;
        let record_key = trigger_run_record_key(&record);
        {
            let mut claims = txn
                .open_table(crate::store::WEBHOOK_CLAIMS)
                .map_err(|error| crate::store::StoreError::Table(error.to_string()))?;
            claims
                .insert(request.identity.as_str(), claim_data.as_slice())
                .map_err(|error| crate::store::StoreError::Db(error.to_string()))?;
        }
        {
            let mut history = txn
                .open_table(crate::store::TRIGGER_RUNS)
                .map_err(|error| crate::store::StoreError::Table(error.to_string()))?;
            history
                .insert(record_key.as_str(), record_data.as_slice())
                .map_err(|error| crate::store::StoreError::Db(error.to_string()))?;
        }
        txn.commit().map_err(|error| crate::store::StoreError::Db(error.to_string()))?;
        Ok(crate::triggers::WebhookClaimResolution::Inserted { run_id })
    }

    pub fn list_webhook_claims(
        &self,
        max_claim_count: u64,
    ) -> Result<Vec<crate::triggers::WebhookClaim>, crate::store::StoreError> {
        let max_claim_count = usize::try_from(max_claim_count).map_err(|error| {
            crate::store::StoreError::Conflict(format!("webhook claim limit is unsupported: {error}"))
        })?;
        let txn = self.db.begin_read().map_err(|error| crate::store::StoreError::Db(error.to_string()))?;
        let table = txn
            .open_table(crate::store::WEBHOOK_CLAIMS)
            .map_err(|error| crate::store::StoreError::Table(error.to_string()))?;
        let mut claims = Vec::with_capacity(max_claim_count);
        let iter = table.iter().map_err(|error| crate::store::StoreError::Db(error.to_string()))?;
        for entry in iter {
            if claims.len() >= max_claim_count {
                break;
            }
            let (_identity, data) = entry.map_err(|error| crate::store::StoreError::Db(error.to_string()))?;
            claims.push(crate::store::deserialize(data.value())?);
        }
        assert!(claims.len() <= max_claim_count);
        Ok(claims)
    }
}

fn trigger_run_record_key(record: &crate::triggers::TriggerRunRecord) -> String {
    let suffix = record
        .run_id
        .map(|run_id| run_id.0.to_string())
        .or_else(|| record.idempotency_key.clone())
        .unwrap_or_else(|| record.message.clone());
    format!("{}:{suffix}", record.trigger_id)
}

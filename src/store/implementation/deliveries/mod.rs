use redb::ReadableDatabase as _;
use redb::ReadableTable as _;

mod accounting;

impl crate::store::Store {
    /// Persist one admitted candidate receipt by attempt identity.
    pub fn save_candidate_receipt(
        &self,
        receipt: &crate::candidate::CandidateReceipt,
    ) -> Result<(), crate::store::StoreError> {
        self.ensure_not_quarantined()?;
        let data = receipt.to_json().map_err(|error| crate::store::StoreError::Serialize(error.to_string()))?;
        let txn = self.db.begin_write().map_err(|error| crate::store::StoreError::Db(error.to_string()))?;
        {
            let mut table = txn
                .open_table(crate::store::CANDIDATE_RECEIPTS)
                .map_err(|error| crate::store::StoreError::Table(error.to_string()))?;
            if let Some(existing) = table
                .get(receipt.attempt_id.as_str())
                .map_err(|error| crate::store::StoreError::Db(error.to_string()))?
            {
                let text = std::str::from_utf8(existing.value())
                    .map_err(|error| crate::store::StoreError::Deserialize(error.to_string()))?;
                let existing = crate::candidate::CandidateReceipt::from_json(text)
                    .map_err(|error| crate::store::StoreError::Deserialize(error.to_string()))?;
                if existing != *receipt {
                    return Err(crate::store::StoreError::Conflict(
                        "candidate receipt already exists with different content".to_string(),
                    ));
                }
                return Ok(());
            }
            table
                .insert(receipt.attempt_id.as_str(), data.as_bytes())
                .map_err(|error| crate::store::StoreError::Db(error.to_string()))?;
        }
        txn.commit().map_err(|error| crate::store::StoreError::Db(error.to_string()))?;
        Ok(())
    }

    /// Load and revalidate one persisted candidate receipt.
    pub fn load_candidate_receipt(
        &self,
        attempt_id: &str,
    ) -> Result<Option<crate::candidate::CandidateReceipt>, crate::store::StoreError> {
        let txn = self.db.begin_read().map_err(|error| crate::store::StoreError::Db(error.to_string()))?;
        let table = txn
            .open_table(crate::store::CANDIDATE_RECEIPTS)
            .map_err(|error| crate::store::StoreError::Table(error.to_string()))?;
        match table.get(attempt_id).map_err(|error| crate::store::StoreError::Db(error.to_string()))? {
            Some(data) => {
                let text = std::str::from_utf8(data.value())
                    .map_err(|error| crate::store::StoreError::Deserialize(error.to_string()))?;
                let receipt = crate::candidate::CandidateReceipt::from_json(text)
                    .map_err(|error| crate::store::StoreError::Deserialize(error.to_string()))?;
                Ok(Some(receipt))
            }
            None => Ok(None),
        }
    }

    /// Persist one candidate file's exact bytes by attempt and relative path.
    pub fn save_candidate_artifact(
        &self,
        location: crate::store::ArtifactKey<'_>,
        bytes: &[u8],
    ) -> Result<(), crate::store::StoreError> {
        self.ensure_not_quarantined()?;
        if location.attempt_id.is_empty() || location.attempt_id.chars().any(char::is_control) {
            return Err(crate::store::StoreError::Serialize("candidate attempt identity is invalid".to_string()));
        }
        crate::candidate::validate_candidate_path(location.relative_path)
            .map_err(|error| crate::store::StoreError::Serialize(error.to_string()))?;
        let key = candidate_artifact_key(location);
        let txn = self.db.begin_write().map_err(|error| crate::store::StoreError::Db(error.to_string()))?;
        {
            let mut table = txn
                .open_table(crate::store::CANDIDATE_ARTIFACTS)
                .map_err(|error| crate::store::StoreError::Table(error.to_string()))?;
            if let Some(existing) =
                table.get(key.as_str()).map_err(|error| crate::store::StoreError::Db(error.to_string()))?
            {
                if existing.value() != bytes {
                    return Err(crate::store::StoreError::Conflict(
                        "candidate artifact already exists with different content".to_string(),
                    ));
                }
                return Ok(());
            }
            table.insert(key.as_str(), bytes).map_err(|error| crate::store::StoreError::Db(error.to_string()))?;
        }
        txn.commit().map_err(|error| crate::store::StoreError::Db(error.to_string()))?;
        Ok(())
    }

    /// Load one candidate file's exact bytes.
    pub fn load_candidate_artifact(
        &self,
        location: crate::store::ArtifactKey<'_>,
    ) -> Result<Option<Vec<u8>>, crate::store::StoreError> {
        crate::candidate::validate_candidate_path(location.relative_path)
            .map_err(|error| crate::store::StoreError::Deserialize(error.to_string()))?;
        let key = candidate_artifact_key(location);
        let txn = self.db.begin_read().map_err(|error| crate::store::StoreError::Db(error.to_string()))?;
        let table = txn
            .open_table(crate::store::CANDIDATE_ARTIFACTS)
            .map_err(|error| crate::store::StoreError::Table(error.to_string()))?;
        Ok(table
            .get(key.as_str())
            .map_err(|error| crate::store::StoreError::Db(error.to_string()))?
            .map(|data| data.value().to_vec()))
    }

    /// Load all candidate file bytes retained for one attempt.
    pub fn list_candidate_artifacts(
        &self,
        attempt_id: &str,
    ) -> Result<std::collections::BTreeMap<String, Vec<u8>>, crate::store::StoreError> {
        let txn = self.db.begin_read().map_err(|error| crate::store::StoreError::Db(error.to_string()))?;
        let table = txn
            .open_table(crate::store::CANDIDATE_ARTIFACTS)
            .map_err(|error| crate::store::StoreError::Table(error.to_string()))?;
        let max_artifact_count = crate::store::implementation::entry_count(&table)?;
        table
            .iter()
            .map_err(|error| crate::store::StoreError::Db(error.to_string()))?
            .take(max_artifact_count)
            .map(|entry| {
                let (key, value) = entry.map_err(|error| crate::store::StoreError::Db(error.to_string()))?;
                Ok(key
                    .value()
                    .split_once('\0')
                    .filter(|(owner, _)| *owner == attempt_id)
                    .map(|(_, path)| (path.to_string(), value.value().to_vec())))
            })
            .filter_map(Result::transpose)
            .collect()
    }
    /// Persist one validated candidate attempt result by attempt identity.
    pub fn save_candidate_attempt_result(
        &self,
        attempt_id: &str,
        result: &crate::candidate_worker::AttemptResult,
    ) -> Result<(), crate::store::StoreError> {
        self.ensure_not_quarantined()?;
        if attempt_id.is_empty() || attempt_id.chars().any(char::is_control) {
            return Err(crate::store::StoreError::Serialize("candidate attempt identity is invalid".to_string()));
        }
        let data = result.to_json().map_err(|error| crate::store::StoreError::Serialize(error.to_string()))?;
        let txn = self.db.begin_write().map_err(|error| crate::store::StoreError::Db(error.to_string()))?;
        {
            let mut table = txn
                .open_table(crate::store::CANDIDATE_ATTEMPT_RESULTS)
                .map_err(|error| crate::store::StoreError::Table(error.to_string()))?;
            if let Some(existing) =
                table.get(attempt_id).map_err(|error| crate::store::StoreError::Db(error.to_string()))?
            {
                let text = std::str::from_utf8(existing.value())
                    .map_err(|error| crate::store::StoreError::Deserialize(error.to_string()))?;
                let existing = crate::candidate_worker::AttemptResult::from_json(text)
                    .map_err(|error| crate::store::StoreError::Deserialize(error.to_string()))?;
                if existing != *result {
                    return Err(crate::store::StoreError::Conflict(
                        "candidate attempt result already exists with different content".to_string(),
                    ));
                }
                return Ok(());
            }
            table
                .insert(attempt_id, data.as_bytes())
                .map_err(|error| crate::store::StoreError::Db(error.to_string()))?;
        }
        txn.commit().map_err(|error| crate::store::StoreError::Db(error.to_string()))?;
        Ok(())
    }

    /// Load and revalidate one candidate attempt result.
    pub fn load_candidate_attempt_result(
        &self,
        attempt_id: &str,
    ) -> Result<Option<crate::candidate_worker::AttemptResult>, crate::store::StoreError> {
        let txn = self.db.begin_read().map_err(|error| crate::store::StoreError::Db(error.to_string()))?;
        let table = txn
            .open_table(crate::store::CANDIDATE_ATTEMPT_RESULTS)
            .map_err(|error| crate::store::StoreError::Table(error.to_string()))?;
        match table.get(attempt_id).map_err(|error| crate::store::StoreError::Db(error.to_string()))? {
            Some(data) => {
                let text = std::str::from_utf8(data.value())
                    .map_err(|error| crate::store::StoreError::Deserialize(error.to_string()))?;
                let result = crate::candidate_worker::AttemptResult::from_json(text)
                    .map_err(|error| crate::store::StoreError::Deserialize(error.to_string()))?;
                Ok(Some(result))
            }
            None => Ok(None),
        }
    }
}

fn candidate_artifact_key(location: crate::store::ArtifactKey<'_>) -> String {
    format!("{}\0{}", location.attempt_id, location.relative_path)
}

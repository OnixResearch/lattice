use redb::ReadableDatabase as _;
use redb::ReadableTable as _;

use crate::store::EffectAttemptRepository as _;

mod conversation;
mod history;

// r[impl lattice.choregraph_effect_authority.durability]
impl crate::store::EffectAttemptRepository for crate::store::Store {
    fn reserve_effect_attempt(
        &self,
        binding: crate::engine::effect::authority::IntentBinding,
    ) -> Result<crate::engine::effect::authority::Attempt, crate::store::StoreError> {
        self.ensure_not_quarantined()?;
        let existing = std::collections::BTreeSet::new();
        let attempt = crate::engine::effect::authority::reserve_attempt(binding, &existing).map_err(|error| {
            crate::store::StoreError::Conflict(format!("effect attempt reservation failed: {error:?}"))
        })?;
        let data = crate::store::serialize(&attempt)?;
        let txn = self.db.begin_write().map_err(|error| crate::store::StoreError::Db(error.to_string()))?;
        {
            let mut table = txn
                .open_table(crate::store::EFFECT_ATTEMPTS)
                .map_err(|error| crate::store::StoreError::Table(error.to_string()))?;
            if table
                .get(attempt.identity.as_str())
                .map_err(|error| crate::store::StoreError::Db(error.to_string()))?
                .is_some()
            {
                return Err(crate::store::StoreError::Conflict("effect attempt already exists".to_owned()));
            }
            table
                .insert(attempt.identity.as_str(), data.as_slice())
                .map_err(|error| crate::store::StoreError::Db(error.to_string()))?;
        }
        txn.commit().map_err(|error| crate::store::StoreError::Db(error.to_string()))?;
        Ok(attempt)
    }

    fn save_effect_outcome(
        &self,
        event: &crate::engine::effect::authority::AuthorizationOutcomeEvent,
    ) -> Result<(), crate::store::StoreError> {
        let data = crate::store::serialize(event)?;
        let txn = self.db.begin_write().map_err(|error| crate::store::StoreError::Db(error.to_string()))?;
        {
            let attempts = txn
                .open_table(crate::store::EFFECT_ATTEMPTS)
                .map_err(|error| crate::store::StoreError::Table(error.to_string()))?;
            if attempts
                .get(event.attempt.as_str())
                .map_err(|error| crate::store::StoreError::Db(error.to_string()))?
                .is_none()
            {
                return Err(crate::store::StoreError::NotFound("effect attempt is not persisted".to_owned()));
            }
            let mut outcomes = txn
                .open_table(crate::store::EFFECT_OUTCOMES)
                .map_err(|error| crate::store::StoreError::Table(error.to_string()))?;
            outcomes
                .insert(event.identity.as_str(), data.as_slice())
                .map_err(|error| crate::store::StoreError::Db(error.to_string()))?;
        }
        txn.commit().map_err(|error| crate::store::StoreError::Db(error.to_string()))?;
        Ok(())
    }

    fn load_effect_attempt(
        &self,
        identity: &crate::engine::effect::authority::AttemptIdentity,
    ) -> Result<Option<crate::engine::effect::authority::Attempt>, crate::store::StoreError> {
        let txn = self.db.begin_read().map_err(|error| crate::store::StoreError::Db(error.to_string()))?;
        let table = txn
            .open_table(crate::store::EFFECT_ATTEMPTS)
            .map_err(|error| crate::store::StoreError::Table(error.to_string()))?;
        table
            .get(identity.as_str())
            .map_err(|error| crate::store::StoreError::Db(error.to_string()))?
            .map(|data| crate::store::deserialize(data.value()))
            .transpose()
    }

    fn load_effect_outcome(
        &self,
        identity: &str,
    ) -> Result<Option<crate::engine::effect::authority::AuthorizationOutcomeEvent>, crate::store::StoreError> {
        let txn = self.db.begin_read().map_err(|error| crate::store::StoreError::Db(error.to_string()))?;
        let table = txn
            .open_table(crate::store::EFFECT_OUTCOMES)
            .map_err(|error| crate::store::StoreError::Table(error.to_string()))?;
        table
            .get(identity)
            .map_err(|error| crate::store::StoreError::Db(error.to_string()))?
            .map(|data| crate::store::deserialize(data.value()))
            .transpose()
    }
}

impl crate::engine::effect::dispatch::EffectAttemptStore for crate::store::Store {
    fn reserve(
        &self,
        binding: crate::engine::effect::authority::IntentBinding,
    ) -> Result<crate::engine::effect::authority::Attempt, crate::engine::effect::dispatch::Error> {
        self.reserve_effect_attempt(binding).map_err(|error| match error {
            crate::store::StoreError::Conflict(_) => crate::engine::effect::dispatch::Error::DuplicateAttempt,
            _ => crate::engine::effect::dispatch::Error::AttemptPersistence,
        })
    }

    fn record_outcome(
        &self,
        event: &crate::engine::effect::authority::AuthorizationOutcomeEvent,
    ) -> Result<(), crate::engine::effect::dispatch::Error> {
        self.save_effect_outcome(event)
            .map_err(|_| crate::engine::effect::dispatch::Error::OutcomePersistence)
    }
}

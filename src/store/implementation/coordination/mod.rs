use redb::ReadableDatabase as _;
use redb::ReadableTable as _;

mod creation;
mod ingress;
mod updates;

impl crate::store::Store {
    // r[impl lattice.agent_orchestration.nominal_identities.wire_boundary]
    // r[impl lattice.agent_orchestration.nominal_identities.compatibility]
    pub fn save_orchestration_definition(
        &self,
        definition: &crate::orchestration::Definition,
    ) -> Result<(), crate::store::StoreError> {
        let data = crate::store::serialize(definition)?;
        let txn = self.db.begin_write().map_err(|error| crate::store::StoreError::Db(error.to_string()))?;
        {
            let mut table = txn
                .open_table(crate::store::ORCHESTRATION_DEFINITIONS)
                .map_err(|error| crate::store::StoreError::Table(error.to_string()))?;
            table
                .insert(definition.id.as_str(), data.as_slice())
                .map_err(|error| crate::store::StoreError::Db(error.to_string()))?;
        }
        txn.commit().map_err(|error| crate::store::StoreError::Db(error.to_string()))?;
        Ok(())
    }

    pub fn load_orchestration_definition(
        &self,
        id: &crate::orchestration::DefinitionId,
    ) -> Result<Option<crate::orchestration::Definition>, crate::store::StoreError> {
        let txn = self.db.begin_read().map_err(|error| crate::store::StoreError::Db(error.to_string()))?;
        let table = txn
            .open_table(crate::store::ORCHESTRATION_DEFINITIONS)
            .map_err(|error| crate::store::StoreError::Table(error.to_string()))?;
        crate::store::deserialize_optional(
            table.get(id.as_str()).map_err(|error| crate::store::StoreError::Db(error.to_string()))?,
        )
    }
    pub fn next_orchestration_sequence(&self) -> Result<u64, crate::store::StoreError> {
        let txn = self.db.begin_write().map_err(|error| crate::store::StoreError::Db(error.to_string()))?;
        let next = {
            let mut table = txn
                .open_table(crate::store::ORCHESTRATION_COUNTERS)
                .map_err(|error| crate::store::StoreError::Table(error.to_string()))?;
            let current = table
                .get(crate::store::ORCHESTRATION_INSTANCE_COUNTER_KEY)
                .map_err(|error| crate::store::StoreError::Db(error.to_string()))?
                .map_or(crate::store::INITIAL_ORCHESTRATION_SEQUENCE, |value| value.value());
            let next = current.checked_add(crate::store::NEXT_ORCHESTRATION_SEQUENCE_INCREMENT).ok_or_else(|| {
                crate::store::StoreError::Conflict("orchestration instance sequence exhausted".to_string())
            })?;
            table
                .insert(crate::store::ORCHESTRATION_INSTANCE_COUNTER_KEY, next)
                .map_err(|error| crate::store::StoreError::Db(error.to_string()))?;
            next
        };
        txn.commit().map_err(|error| crate::store::StoreError::Db(error.to_string()))?;
        Ok(next)
    }
    pub fn load_orchestration(
        &self,
        id: &crate::orchestration::InstanceId,
    ) -> Result<Option<crate::orchestration::Instance>, crate::store::StoreError> {
        let txn = self.db.begin_read().map_err(|error| crate::store::StoreError::Db(error.to_string()))?;
        let table = txn
            .open_table(crate::store::ORCHESTRATION_INSTANCES)
            .map_err(|error| crate::store::StoreError::Table(error.to_string()))?;
        crate::store::deserialize_optional(
            table.get(id.as_str()).map_err(|error| crate::store::StoreError::Db(error.to_string()))?,
        )
    }

    pub fn load_orchestration_events(
        &self,
        id: &crate::orchestration::InstanceId,
    ) -> Result<Vec<crate::orchestration::Event>, crate::store::StoreError> {
        let txn = self.db.begin_read().map_err(|error| crate::store::StoreError::Db(error.to_string()))?;
        let table = txn
            .open_table(crate::store::ORCHESTRATION_EVENTS)
            .map_err(|error| crate::store::StoreError::Table(error.to_string()))?;
        match table.get(id.as_str()).map_err(|error| crate::store::StoreError::Db(error.to_string()))? {
            Some(data) => crate::store::deserialize(data.value()),
            None => Ok(Vec::new()),
        }
    }

    pub fn list_orchestrations(&self) -> Result<Vec<crate::orchestration::Instance>, crate::store::StoreError> {
        let txn = self.db.begin_read().map_err(|error| crate::store::StoreError::Db(error.to_string()))?;
        let table = txn
            .open_table(crate::store::ORCHESTRATION_INSTANCES)
            .map_err(|error| crate::store::StoreError::Table(error.to_string()))?;
        let mut instances = Vec::with_capacity(crate::store::implementation::entry_count(&table)?);
        let iter = table.iter().map_err(|error| crate::store::StoreError::Db(error.to_string()))?;
        for entry in iter {
            let (_key, data) = entry.map_err(|error| crate::store::StoreError::Db(error.to_string()))?;
            instances.push(crate::store::deserialize(data.value())?);
        }
        instances.sort_by_key(|instance: &crate::orchestration::Instance| instance.sequence);
        Ok(instances)
    }
}

pub(super) fn identity_admission_error(error: crate::orchestration::AdmissionError) -> crate::store::StoreError {
    crate::store::StoreError::Deserialize(error.to_string())
}

use redb::ReadableDatabase as _;
use redb::ReadableTable as _;

impl crate::store::Store {
    // r[impl lattice.agent_orchestration.triggers]
    pub fn save_orchestration_trigger_admission(
        &self,
        scoped_key: &crate::orchestration::TriggerScopedKey,
        instance_id: &crate::orchestration::InstanceId,
    ) -> Result<crate::store::TriggerAdmissionWrite, crate::store::StoreError> {
        let txn = self.db.begin_write().map_err(|error| crate::store::StoreError::Db(error.to_string()))?;
        let outcome = {
            let mut table = txn
                .open_table(crate::store::ORCHESTRATION_TRIGGER_ADMISSIONS)
                .map_err(|error| crate::store::StoreError::Table(error.to_string()))?;
            let existing_instance_id = table
                .get(scoped_key.as_str())
                .map_err(|error| crate::store::StoreError::Db(error.to_string()))?
                .map(|existing| {
                    crate::orchestration::InstanceId::try_from_hex(existing.value())
                        .map_err(super::identity_admission_error)
                })
                .transpose()?;
            match existing_instance_id {
                Some(existing_instance_id) => crate::store::TriggerAdmissionWrite::Duplicate {
                    instance_id: existing_instance_id,
                },
                None => {
                    table
                        .insert(scoped_key.as_str(), instance_id.as_str())
                        .map_err(|error| crate::store::StoreError::Db(error.to_string()))?;
                    crate::store::TriggerAdmissionWrite::Inserted
                }
            }
        };
        txn.commit().map_err(|error| crate::store::StoreError::Db(error.to_string()))?;
        Ok(outcome)
    }

    pub fn list_orchestration_trigger_admissions(
        &self,
    ) -> Result<
        std::collections::HashMap<crate::orchestration::TriggerScopedKey, crate::orchestration::InstanceId>,
        crate::store::StoreError,
    > {
        let txn = self.db.begin_read().map_err(|error| crate::store::StoreError::Db(error.to_string()))?;
        let table = txn
            .open_table(crate::store::ORCHESTRATION_TRIGGER_ADMISSIONS)
            .map_err(|error| crate::store::StoreError::Table(error.to_string()))?;
        let mut admissions =
            std::collections::HashMap::with_capacity(crate::store::implementation::entry_count(&table)?);
        let iter = table.iter().map_err(|error| crate::store::StoreError::Db(error.to_string()))?;
        for entry in iter {
            let (key, value) = entry.map_err(|error| crate::store::StoreError::Db(error.to_string()))?;
            let scoped_key = crate::orchestration::TriggerScopedKey::try_from_hex(key.value())
                .map_err(super::identity_admission_error)?;
            let instance_id = crate::orchestration::InstanceId::try_from_hex(value.value())
                .map_err(super::identity_admission_error)?;
            admissions.insert(scoped_key, instance_id);
        }
        Ok(admissions)
    }
}

use redb::ReadableDatabase as _;
use redb::ReadableTable as _;

impl crate::store::Store {
    /// Save or replace a schedule definition.
    pub fn save_schedule(
        &self,
        schedule: &crate::scheduler::ScheduleDefinition,
    ) -> Result<(), crate::store::StoreError> {
        let data = serde_json::to_vec(schedule).map_err(|e| crate::store::StoreError::Serialize(e.to_string()))?;
        let txn = self.db.begin_write().map_err(|e| crate::store::StoreError::Db(e.to_string()))?;
        {
            let mut table = txn
                .open_table(crate::store::SCHEDULES)
                .map_err(|e| crate::store::StoreError::Table(e.to_string()))?;
            table
                .insert(schedule.id.as_str(), data.as_slice())
                .map_err(|e| crate::store::StoreError::Db(e.to_string()))?;
        }
        txn.commit().map_err(|e| crate::store::StoreError::Db(e.to_string()))?;
        Ok(())
    }

    /// Load a schedule definition by id.
    pub fn load_schedule(
        &self,
        schedule_id: &str,
    ) -> Result<Option<crate::scheduler::ScheduleDefinition>, crate::store::StoreError> {
        let txn = self.db.begin_read().map_err(|e| crate::store::StoreError::Db(e.to_string()))?;
        let table = txn
            .open_table(crate::store::SCHEDULES)
            .map_err(|e| crate::store::StoreError::Table(e.to_string()))?;
        match table.get(schedule_id).map_err(|e| crate::store::StoreError::Db(e.to_string()))? {
            Some(data) => {
                let bytes: &[u8] = data.value();
                let schedule: crate::scheduler::ScheduleDefinition =
                    serde_json::from_slice(bytes).map_err(|e| crate::store::StoreError::Deserialize(e.to_string()))?;
                Ok(Some(schedule))
            }
            None => Ok(None),
        }
    }

    /// List all schedule definitions.
    pub fn list_schedules(&self) -> Result<Vec<crate::scheduler::ScheduleDefinition>, crate::store::StoreError> {
        let txn = self.db.begin_read().map_err(|e| crate::store::StoreError::Db(e.to_string()))?;
        let table = txn
            .open_table(crate::store::SCHEDULES)
            .map_err(|e| crate::store::StoreError::Table(e.to_string()))?;
        let mut schedules = Vec::with_capacity(crate::store::implementation::entry_count(&table)?);
        let iter = table.iter().map_err(|e| crate::store::StoreError::Db(e.to_string()))?;
        for entry in iter {
            let (_key, data) = entry.map_err(|e| crate::store::StoreError::Db(e.to_string()))?;
            let bytes: &[u8] = data.value();
            let schedule: crate::scheduler::ScheduleDefinition =
                serde_json::from_slice(bytes).map_err(|e| crate::store::StoreError::Deserialize(e.to_string()))?;
            schedules.push(schedule);
        }
        Ok(schedules)
    }

    /// Delete a schedule definition by id.
    pub fn delete_schedule(&self, schedule_id: &str) -> Result<(), crate::store::StoreError> {
        let txn = self.db.begin_write().map_err(|e| crate::store::StoreError::Db(e.to_string()))?;
        {
            let mut table = txn
                .open_table(crate::store::SCHEDULES)
                .map_err(|e| crate::store::StoreError::Table(e.to_string()))?;
            table.remove(schedule_id).map_err(|e| crate::store::StoreError::Db(e.to_string()))?;
        }
        txn.commit().map_err(|e| crate::store::StoreError::Db(e.to_string()))?;
        Ok(())
    }
}

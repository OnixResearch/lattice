use redb::ReadableDatabase as _;
use redb::ReadableTable as _;

mod ingress;
mod timing;

impl crate::store::Store {
    /// Save a workflow.
    pub fn save_workflow(&self, workflow: &crate::engine::Workflow) -> Result<(), crate::store::StoreError> {
        self.ensure_not_quarantined()?;
        let data = serde_json::to_vec(workflow).map_err(|e| crate::store::StoreError::Serialize(e.to_string()))?;
        let txn = self.db.begin_write().map_err(|e| crate::store::StoreError::Db(e.to_string()))?;
        {
            let mut table = txn
                .open_table(crate::store::WORKFLOWS)
                .map_err(|e| crate::store::StoreError::Table(e.to_string()))?;
            table
                .insert(workflow.meta.name.as_str(), data.as_slice())
                .map_err(|e| crate::store::StoreError::Db(e.to_string()))?;
        }
        txn.commit().map_err(|e| crate::store::StoreError::Db(e.to_string()))?;
        Ok(())
    }
    /// Load a workflow by name.
    pub fn load_workflow(&self, name: &str) -> Result<Option<crate::engine::Workflow>, crate::store::StoreError> {
        let txn = self.db.begin_read().map_err(|e| crate::store::StoreError::Db(e.to_string()))?;
        let table = txn
            .open_table(crate::store::WORKFLOWS)
            .map_err(|e| crate::store::StoreError::Table(e.to_string()))?;
        match table.get(name).map_err(|e| crate::store::StoreError::Db(e.to_string()))? {
            Some(data) => {
                let bytes: &[u8] = data.value();
                let wf: crate::engine::Workflow =
                    serde_json::from_slice(bytes).map_err(|e| crate::store::StoreError::Deserialize(e.to_string()))?;
                Ok(Some(wf))
            }
            None => Ok(None),
        }
    }

    /// Load a saved workflow without creating a database, tables, or write transactions.
    pub fn load_workflow_read_only(
        path: &std::path::Path,
        name: &str,
    ) -> Result<Option<crate::engine::Workflow>, crate::store::StoreError> {
        let db = redb::ReadOnlyDatabase::open(path).map_err(|e| crate::store::StoreError::Db(e.to_string()))?;
        let txn = db.begin_read().map_err(|e| crate::store::StoreError::Db(e.to_string()))?;
        let table = txn
            .open_table(crate::store::WORKFLOWS)
            .map_err(|e| crate::store::StoreError::Table(e.to_string()))?;
        match table.get(name).map_err(|e| crate::store::StoreError::Db(e.to_string()))? {
            Some(data) => serde_json::from_slice(data.value())
                .map(Some)
                .map_err(|e| crate::store::StoreError::Deserialize(e.to_string())),
            None => Ok(None),
        }
    }

    /// List all workflow names.
    pub fn list_workflows(&self) -> Result<Vec<String>, crate::store::StoreError> {
        let txn = self.db.begin_read().map_err(|e| crate::store::StoreError::Db(e.to_string()))?;
        let table = txn
            .open_table(crate::store::WORKFLOWS)
            .map_err(|e| crate::store::StoreError::Table(e.to_string()))?;
        let mut names = Vec::with_capacity(crate::store::implementation::entry_count(&table)?);
        let iter = table.iter().map_err(|e| crate::store::StoreError::Db(e.to_string()))?;
        for entry in iter {
            let (key, _val) = entry.map_err(|e| crate::store::StoreError::Db(e.to_string()))?;
            names.push(key.value().to_string());
        }
        Ok(names)
    }

    /// Delete a workflow by name.
    pub fn delete_workflow(&self, name: &str) -> Result<(), crate::store::StoreError> {
        let txn = self.db.begin_write().map_err(|e| crate::store::StoreError::Db(e.to_string()))?;
        {
            let mut table = txn
                .open_table(crate::store::WORKFLOWS)
                .map_err(|e| crate::store::StoreError::Table(e.to_string()))?;
            table.remove(name).map_err(|e| crate::store::StoreError::Db(e.to_string()))?;
        }
        txn.commit().map_err(|e| crate::store::StoreError::Db(e.to_string()))?;
        Ok(())
    }
    /// Store a value by its content hash.
    pub fn put_value(&self, value: &crate::engine::Value) -> Result<(), crate::store::StoreError> {
        let data = serde_json::to_vec(value).map_err(|e| crate::store::StoreError::Serialize(e.to_string()))?;
        let txn = self.db.begin_write().map_err(|e| crate::store::StoreError::Db(e.to_string()))?;
        {
            let mut table =
                txn.open_table(crate::store::VALUES).map_err(|e| crate::store::StoreError::Table(e.to_string()))?;
            table
                .insert(value.hash.0.as_slice(), data.as_slice())
                .map_err(|e| crate::store::StoreError::Db(e.to_string()))?;
        }
        txn.commit().map_err(|e| crate::store::StoreError::Db(e.to_string()))?;
        Ok(())
    }

    /// Retrieve a value by content hash.
    pub fn get_value(
        &self,
        hash: &crate::engine::ContentHash,
    ) -> Result<Option<crate::engine::Value>, crate::store::StoreError> {
        let txn = self.db.begin_read().map_err(|e| crate::store::StoreError::Db(e.to_string()))?;
        let table = txn.open_table(crate::store::VALUES).map_err(|e| crate::store::StoreError::Table(e.to_string()))?;
        match table.get(hash.0.as_slice()).map_err(|e| crate::store::StoreError::Db(e.to_string()))? {
            Some(data) => {
                let bytes: &[u8] = data.value();
                let val: crate::engine::Value =
                    serde_json::from_slice(bytes).map_err(|e| crate::store::StoreError::Deserialize(e.to_string()))?;
                Ok(Some(val))
            }
            None => Ok(None),
        }
    }

    /// Store cached outputs for an input hash.
    pub fn put_cache(
        &self,
        input_hash: &crate::engine::ContentHash,
        outputs: &std::collections::HashMap<crate::engine::workflow::PortId, crate::engine::Value>,
    ) -> Result<(), crate::store::StoreError> {
        let data = serde_json::to_vec(outputs).map_err(|e| crate::store::StoreError::Serialize(e.to_string()))?;
        let txn = self.db.begin_write().map_err(|e| crate::store::StoreError::Db(e.to_string()))?;
        {
            let mut table =
                txn.open_table(crate::store::CACHE).map_err(|e| crate::store::StoreError::Table(e.to_string()))?;
            table
                .insert(input_hash.0.as_slice(), data.as_slice())
                .map_err(|e| crate::store::StoreError::Db(e.to_string()))?;
        }
        txn.commit().map_err(|e| crate::store::StoreError::Db(e.to_string()))?;
        Ok(())
    }

    /// Retrieve cached outputs for an input hash.
    pub fn get_cache(
        &self,
        input_hash: &crate::engine::ContentHash,
    ) -> Result<
        Option<std::collections::HashMap<crate::engine::workflow::PortId, crate::engine::Value>>,
        crate::store::StoreError,
    > {
        let txn = self.db.begin_read().map_err(|e| crate::store::StoreError::Db(e.to_string()))?;
        let table = txn.open_table(crate::store::CACHE).map_err(|e| crate::store::StoreError::Table(e.to_string()))?;
        match table.get(input_hash.0.as_slice()).map_err(|e| crate::store::StoreError::Db(e.to_string()))? {
            Some(data) => {
                let bytes: &[u8] = data.value();
                let map: std::collections::HashMap<crate::engine::workflow::PortId, crate::engine::Value> =
                    serde_json::from_slice(bytes).map_err(|e| crate::store::StoreError::Deserialize(e.to_string()))?;
                Ok(Some(map))
            }
            None => Ok(None),
        }
    }
    // r[impl lattice.workflow_templates.activation]
    pub fn save_workflow_template_activation(
        &self,
        receipt: &crate::workflow_template::ActivationReceipt,
    ) -> Result<(), crate::store::StoreError> {
        let data = crate::store::serialize(receipt)?;
        let transaction = self.db.begin_write().map_err(|error| crate::store::StoreError::Db(error.to_string()))?;
        {
            let mut table = transaction
                .open_table(crate::store::WORKFLOW_TEMPLATE_ACTIVATIONS)
                .map_err(|error| crate::store::StoreError::Table(error.to_string()))?;
            if let Some(existing) = table
                .get(receipt.identity.as_str())
                .map_err(|error| crate::store::StoreError::Db(error.to_string()))?
            {
                let stored: crate::workflow_template::ActivationReceipt = crate::store::deserialize(existing.value())?;
                if stored != *receipt {
                    return Err(crate::store::StoreError::Conflict(
                        "workflow-template activation receipt identity collision".to_string(),
                    ));
                }
            } else {
                table
                    .insert(receipt.identity.as_str(), data.as_slice())
                    .map_err(|error| crate::store::StoreError::Db(error.to_string()))?;
            }
        }
        transaction.commit().map_err(|error| crate::store::StoreError::Db(error.to_string()))
    }

    pub fn load_workflow_template_activation(
        &self,
        identity: &crate::workflow_template::ContentIdentity,
    ) -> Result<Option<crate::workflow_template::ActivationReceipt>, crate::store::StoreError> {
        let transaction = self.db.begin_read().map_err(|error| crate::store::StoreError::Db(error.to_string()))?;
        let table = transaction
            .open_table(crate::store::WORKFLOW_TEMPLATE_ACTIVATIONS)
            .map_err(|error| crate::store::StoreError::Table(error.to_string()))?;
        crate::store::deserialize_optional(
            table.get(identity.as_str()).map_err(|error| crate::store::StoreError::Db(error.to_string()))?,
        )
    }
}

#![allow(
    unknown_lints,
    compound_condition,
    path_segment_repetition,
    reason = "the adoption transaction keeps its occupancy check and four inserts in one explicit write"
)]

//! Adopted application checkpoints in the target store (Lattice #12).

use redb::ReadableDatabase as _;
use redb::ReadableTable as _;
use redb::ReadableTableMetadata as _;

/// Adopted application checkpoints, keyed by execution identity.
const WORKFLOW_EXCHANGE_ADOPTIONS: redb::TableDefinition<&str, &[u8]> =
    redb::TableDefinition::new("workflow_exchange_adoptions_v1");

impl crate::store::Store {
    // r[impl lattice.application_checkpoint.restore]
    /// Atomically adopt a checkpoint into a store that holds no trace of the execution.
    ///
    /// `build` runs inside the write transaction with the reserved run id; any error it returns
    /// aborts the transaction, so a refused adoption writes nothing. An existing adoption for the
    /// execution is returned unchanged for the caller to classify; any other existing execution,
    /// request or idempotency entry is a conflict.
    pub fn adopt_workflow_exchange_checkpoint(
        &self,
        accepted: &lattice_workflow_contract::WorkflowAccepted,
        build: impl FnOnce(
            u64,
        ) -> Result<
            (crate::workflow_exchange::ExecutionRecord, crate::checkpoint::AdoptionRecord),
            crate::store::StoreError,
        >,
    ) -> Result<crate::checkpoint::AdoptionOutcome, crate::store::StoreError> {
        let transaction = self.db.begin_write().map_err(super::store_db_error)?;
        let execution_identity = accepted.execution_identity.as_str();
        let prior = {
            let adoptions = transaction.open_table(WORKFLOW_EXCHANGE_ADOPTIONS).map_err(super::store_table_error)?;
            adoptions
                .get(execution_identity)
                .map_err(super::store_db_error)?
                .map(|value| crate::store::deserialize::<crate::checkpoint::AdoptionRecord>(value.value()))
                .transpose()?
        };
        if let Some(prior) = prior {
            transaction.abort().map_err(super::store_db_error)?;
            return Ok(crate::checkpoint::AdoptionOutcome::Existing(Box::new(prior)));
        }
        let is_occupied = {
            let executions = transaction
                .open_table(crate::store::WORKFLOW_EXCHANGE_EXECUTIONS)
                .map_err(super::store_table_error)?;
            let request_index = transaction
                .open_table(crate::store::WORKFLOW_EXCHANGE_REQUEST_INDEX)
                .map_err(super::store_table_error)?;
            let idempotency_index = transaction
                .open_table(crate::store::WORKFLOW_EXCHANGE_IDEMPOTENCY_INDEX)
                .map_err(super::store_table_error)?;
            executions.get(execution_identity).map_err(super::store_db_error)?.is_some()
                || request_index.get(accepted.request_identity.as_str()).map_err(super::store_db_error)?.is_some()
                || idempotency_index.get(accepted.idempotency_key.as_str()).map_err(super::store_db_error)?.is_some()
        };
        if is_occupied {
            return Err(crate::store::StoreError::Conflict(
                "checkpoint adoption target already holds this execution".to_string(),
            ));
        }
        let run_id = crate::store::reserve_run_id(&transaction)?;
        let (record, adoption) = build(run_id.0)?;
        if record.accepted != *accepted || adoption.execution != accepted.execution_identity {
            return Err(crate::store::StoreError::Conflict(
                "checkpoint adoption changed the adopted execution identity".to_string(),
            ));
        }
        let record_bytes = crate::store::serialize(&record)?;
        let adoption_bytes = crate::store::serialize(&adoption)?;
        {
            let mut executions = transaction
                .open_table(crate::store::WORKFLOW_EXCHANGE_EXECUTIONS)
                .map_err(super::store_table_error)?;
            executions.insert(execution_identity, record_bytes.as_slice()).map_err(super::store_db_error)?;
            let mut request_index = transaction
                .open_table(crate::store::WORKFLOW_EXCHANGE_REQUEST_INDEX)
                .map_err(super::store_table_error)?;
            request_index
                .insert(record.request.identity.as_str(), execution_identity)
                .map_err(super::store_db_error)?;
            let mut idempotency_index = transaction
                .open_table(crate::store::WORKFLOW_EXCHANGE_IDEMPOTENCY_INDEX)
                .map_err(super::store_table_error)?;
            idempotency_index
                .insert(record.request.idempotency_key.as_str(), execution_identity)
                .map_err(super::store_db_error)?;
            let mut adoptions =
                transaction.open_table(WORKFLOW_EXCHANGE_ADOPTIONS).map_err(super::store_table_error)?;
            adoptions.insert(execution_identity, adoption_bytes.as_slice()).map_err(super::store_db_error)?;
        }
        transaction.commit().map_err(super::store_db_error)?;
        Ok(crate::checkpoint::AdoptionOutcome::Adopted {
            record: Box::new(record),
            adoption: Box::new(adoption),
        })
    }

    /// The number of adoptions in this store, read from the table header without visiting any
    /// entry. Adoptions are never removed, so an equal count means no adoption was added.
    pub fn count_workflow_exchange_adoptions(&self) -> Result<u64, crate::store::StoreError> {
        let transaction = self.db.begin_read().map_err(super::store_db_error)?;
        match transaction.open_table(WORKFLOW_EXCHANGE_ADOPTIONS) {
            Ok(table) => table.len().map_err(super::store_db_error),
            Err(redb::TableError::TableDoesNotExist(_)) => Ok(0),
            Err(error) => Err(super::store_table_error(error)),
        }
    }

    pub fn load_workflow_exchange_adoption(
        &self,
        execution_identity: &lattice_workflow_contract::Identity,
    ) -> Result<Option<crate::checkpoint::AdoptionRecord>, crate::store::StoreError> {
        let transaction = self.db.begin_read().map_err(super::store_db_error)?;
        let adoptions = match transaction.open_table(WORKFLOW_EXCHANGE_ADOPTIONS) {
            Ok(table) => table,
            Err(redb::TableError::TableDoesNotExist(_)) => return Ok(None),
            Err(error) => return Err(super::store_table_error(error)),
        };
        adoptions
            .get(execution_identity.as_str())
            .map_err(super::store_db_error)?
            .map(|value| crate::store::deserialize(value.value()))
            .transpose()
    }

    /// Every adoption in this store, in execution-identity order.
    pub fn list_workflow_exchange_adoptions(
        &self,
    ) -> Result<Vec<crate::checkpoint::AdoptionRecord>, crate::store::StoreError> {
        let transaction = self.db.begin_read().map_err(super::store_db_error)?;
        let adoptions = match transaction.open_table(WORKFLOW_EXCHANGE_ADOPTIONS) {
            Ok(table) => table,
            Err(redb::TableError::TableDoesNotExist(_)) => return Ok(Vec::new()),
            Err(error) => return Err(super::store_table_error(error)),
        };
        adoptions
            .iter()
            .map_err(super::store_db_error)?
            .map(|entry| {
                let (_execution, value) = entry.map_err(super::store_db_error)?;
                crate::store::deserialize(value.value())
            })
            .collect()
    }
}

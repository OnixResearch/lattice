#![allow(
    unknown_lints,
    compound_condition,
    path_segment_repetition,
    reason = "the exchange repository keeps one atomic three-index transaction with explicit product types"
)]

use redb::ReadableDatabase as _;
use redb::ReadableTable as _;

impl super::Store {
    // r[impl lattice.workflow_interchange.application_admission]
    pub fn admit_workflow_exchange(
        &self,
        request: lattice_workflow_contract::WorkflowRequest,
        route: &crate::workflow_exchange::WorkflowRoute,
        execution_workflow_revision: lattice_workflow_contract::Identity,
        handler_profile_identity: lattice_workflow_contract::Identity,
    ) -> Result<crate::workflow_exchange::AdmissionResolution, super::StoreError> {
        let transaction = self.db.begin_write().map_err(store_db_error)?;
        let existing_by_request = {
            let request_index =
                transaction.open_table(super::WORKFLOW_EXCHANGE_REQUEST_INDEX).map_err(store_table_error)?;
            let execution_identity = request_index
                .get(request.identity.as_str())
                .map_err(store_db_error)?
                .map(|value| value.value().to_string());
            match execution_identity {
                Some(identity) => Some(load_execution(&transaction, &identity)?),
                None => None,
            }
        };
        if let Some(existing) = existing_by_request {
            if existing.request == request {
                transaction.commit().map_err(store_db_error)?;
                return Ok(crate::workflow_exchange::AdmissionResolution::Existing(existing));
            }
            return Err(super::StoreError::Conflict(
                "workflow exchange request identity conflicts with stored material".to_string(),
            ));
        }
        let existing_by_idempotency = {
            let idempotency_index =
                transaction.open_table(super::WORKFLOW_EXCHANGE_IDEMPOTENCY_INDEX).map_err(store_table_error)?;
            let execution_identity = idempotency_index
                .get(request.idempotency_key.as_str())
                .map_err(store_db_error)?
                .map(|value| value.value().to_string());
            match execution_identity {
                Some(identity) => Some(load_execution(&transaction, &identity)?),
                None => None,
            }
        };
        if let Some(existing) = existing_by_idempotency {
            if existing.request.identity != request.identity {
                return Err(super::StoreError::Conflict(
                    "workflow exchange idempotency key already names another request".to_string(),
                ));
            }
            transaction.commit().map_err(store_db_error)?;
            return Ok(crate::workflow_exchange::AdmissionResolution::Existing(existing));
        }
        let run_id = super::reserve_run_id(&transaction)?;
        let record = crate::workflow_exchange::initial_record(
            request,
            route,
            execution_workflow_revision,
            handler_profile_identity,
            run_id.0,
        )
        .map_err(transition_error)?;
        let bytes = super::serialize(&record)?;
        let execution_identity = record.accepted.execution_identity.as_str();
        {
            let mut executions =
                transaction.open_table(super::WORKFLOW_EXCHANGE_EXECUTIONS).map_err(store_table_error)?;
            executions.insert(execution_identity, bytes.as_slice()).map_err(store_db_error)?;
            let mut request_index =
                transaction.open_table(super::WORKFLOW_EXCHANGE_REQUEST_INDEX).map_err(store_table_error)?;
            request_index.insert(record.request.identity.as_str(), execution_identity).map_err(store_db_error)?;
            let mut idempotency_index =
                transaction.open_table(super::WORKFLOW_EXCHANGE_IDEMPOTENCY_INDEX).map_err(store_table_error)?;
            idempotency_index
                .insert(record.request.idempotency_key.as_str(), execution_identity)
                .map_err(store_db_error)?;
        }
        transaction.commit().map_err(store_db_error)?;
        Ok(crate::workflow_exchange::AdmissionResolution::Inserted(record))
    }

    pub fn load_workflow_exchange_by_request(
        &self,
        request_identity: &lattice_workflow_contract::Identity,
    ) -> Result<Option<crate::workflow_exchange::ExecutionRecord>, super::StoreError> {
        let transaction = self.db.begin_read().map_err(store_db_error)?;
        let request_index =
            transaction.open_table(super::WORKFLOW_EXCHANGE_REQUEST_INDEX).map_err(store_table_error)?;
        let execution_identity = request_index
            .get(request_identity.as_str())
            .map_err(store_db_error)?
            .map(|value| value.value().to_string());
        let Some(execution_identity) = execution_identity else {
            return Ok(None);
        };
        let executions = transaction.open_table(super::WORKFLOW_EXCHANGE_EXECUTIONS).map_err(store_table_error)?;
        executions
            .get(execution_identity.as_str())
            .map_err(store_db_error)?
            .map(|value| super::deserialize(value.value()))
            .transpose()
    }

    pub fn load_workflow_exchange(
        &self,
        execution_identity: &lattice_workflow_contract::Identity,
    ) -> Result<Option<crate::workflow_exchange::ExecutionRecord>, super::StoreError> {
        let transaction = self.db.begin_read().map_err(store_db_error)?;
        let executions = transaction.open_table(super::WORKFLOW_EXCHANGE_EXECUTIONS).map_err(store_table_error)?;
        executions
            .get(execution_identity.as_str())
            .map_err(store_db_error)?
            .map(|value| super::deserialize(value.value()))
            .transpose()
    }

    pub fn save_workflow_exchange_transition(
        &self,
        current: &crate::workflow_exchange::ExecutionRecord,
        next: &crate::workflow_exchange::ExecutionRecord,
    ) -> Result<(), super::StoreError> {
        if current == next {
            return Ok(());
        }
        if current.accepted.execution_identity != next.accepted.execution_identity
            || current.request.identity != next.request.identity
            || current.handler_profile_identity != next.handler_profile_identity
            || current.run_id != next.run_id
            || next.sequence <= current.sequence
        {
            return Err(super::StoreError::Conflict(
                "workflow exchange transition changed immutable identity or sequence".to_string(),
            ));
        }
        let transaction = self.db.begin_write().map_err(store_db_error)?;
        let bytes = super::serialize(next)?;
        {
            let mut executions =
                transaction.open_table(super::WORKFLOW_EXCHANGE_EXECUTIONS).map_err(store_table_error)?;
            let stored: crate::workflow_exchange::ExecutionRecord = {
                let value = executions
                    .get(current.accepted.execution_identity.as_str())
                    .map_err(store_db_error)?
                    .ok_or_else(|| super::StoreError::NotFound("workflow exchange execution".to_string()))?;
                super::deserialize(value.value())?
            };
            if stored != *current {
                return Err(super::StoreError::Conflict(
                    "workflow exchange execution changed concurrently".to_string(),
                ));
            }
            executions
                .insert(next.accepted.execution_identity.as_str(), bytes.as_slice())
                .map_err(store_db_error)?;
        }
        transaction.commit().map_err(store_db_error)
    }
}

fn load_execution(
    transaction: &redb::WriteTransaction,
    execution_identity: &str,
) -> Result<crate::workflow_exchange::ExecutionRecord, super::StoreError> {
    let executions = transaction.open_table(super::WORKFLOW_EXCHANGE_EXECUTIONS).map_err(store_table_error)?;
    let value = executions.get(execution_identity).map_err(store_db_error)?.ok_or_else(|| {
        super::StoreError::Conflict("workflow exchange index references a missing execution".to_string())
    })?;
    super::deserialize(value.value())
}

fn transition_error(error: crate::workflow_exchange::TransitionError) -> super::StoreError {
    super::StoreError::Conflict(format!("workflow exchange transition rejected: {error:?}"))
}

fn store_db_error(error: impl std::fmt::Display) -> super::StoreError {
    super::StoreError::Db(error.to_string())
}

fn store_table_error(error: impl std::fmt::Display) -> super::StoreError {
    super::StoreError::Table(error.to_string())
}

const OPERATION_RUN_NODE_ID: u64 = 0;

impl Store {
    pub fn begin_operation_run(
        &self,
        plan: &crate::operation::InvocationPlan,
        observed_epoch_millis: i64,
    ) -> Result<crate::operation::RunRecord, StoreError> {
        crate::operation::planning::validate_plan_identity(plan)
            .map_err(|error| StoreError::Conflict(error.to_string()))?;
        let transaction = self.db.begin_write().map_err(|error| StoreError::Db(error.to_string()))?;
        let run_id = reserve_run_id(&transaction)?;
        let record = crate::operation::RunRecord::started(plan, run_id, observed_epoch_millis);
        let bytes = serialize(&record)?;
        {
            let mut table =
                transaction.open_table(OPERATION_RUNS).map_err(|error| StoreError::Table(error.to_string()))?;
            if table.get(plan.identity.as_str()).map_err(|error| StoreError::Db(error.to_string()))?.is_some() {
                return Err(StoreError::Conflict("operation invocation already has durable run evidence".to_string()));
            }
            table
                .insert(plan.identity.as_str(), bytes.as_slice())
                .map_err(|error| StoreError::Db(error.to_string()))?;
        }
        transaction.commit().map_err(|error| StoreError::Db(error.to_string()))?;
        Ok(record)
    }

    pub fn finish_operation_run(&self, record: &crate::operation::RunRecord) -> Result<(), StoreError> {
        if record.state == crate::operation::RunState::Started || record.finished_epoch_millis.is_none() {
            return Err(StoreError::Conflict("operation terminal record is not finished".to_string()));
        }
        if record.finished_epoch_millis.is_some_and(|finished| finished < record.started_epoch_millis) {
            return Err(StoreError::Conflict("operation terminal observation predates its durable start".to_string()));
        }
        let transaction = self.db.begin_write().map_err(|error| StoreError::Db(error.to_string()))?;
        let bytes = serialize(record)?;
        {
            let mut table =
                transaction.open_table(OPERATION_RUNS).map_err(|error| StoreError::Table(error.to_string()))?;
            let current: crate::operation::RunRecord = {
                let current = table
                    .get(record.invocation_identity.as_str())
                    .map_err(|error| StoreError::Db(error.to_string()))?
                    .ok_or_else(|| StoreError::Conflict("operation durable start is missing".to_string()))?;
                deserialize(current.value())?
            };
            if !terminal_matches_start(&current, record) {
                return Err(StoreError::Conflict(
                    "operation durable start does not match the terminal backend attempt".to_string(),
                ));
            }
            table
                .insert(record.invocation_identity.as_str(), bytes.as_slice())
                .map_err(|error| StoreError::Db(error.to_string()))?;
        }
        insert_standard_operation_run(&transaction, record)?;
        transaction.commit().map_err(|error| StoreError::Db(error.to_string()))?;
        Ok(())
    }

    pub fn load_operation_run(
        &self,
        invocation_identity: &str,
    ) -> Result<Option<crate::operation::RunRecord>, StoreError> {
        let transaction = self.db.begin_read().map_err(|error| StoreError::Db(error.to_string()))?;
        let table = transaction.open_table(OPERATION_RUNS).map_err(|error| StoreError::Table(error.to_string()))?;
        deserialize_optional(table.get(invocation_identity).map_err(|error| StoreError::Db(error.to_string()))?)
    }
}

#[allow(
    unknown_lints,
    assertion_density,
    reason = "the transaction writes one exact compatibility record and one exact detail record with fail-closed errors"
)]
fn insert_standard_operation_run(
    transaction: &redb::WriteTransaction,
    record: &crate::operation::RunRecord,
) -> Result<(), StoreError> {
    let failure_count = usize::from(record.state != crate::operation::RunState::Completed);
    let run = RunRecord {
        run_id: record.run_id,
        owner: RunOwner::Unscoped,
        workflow_name: format!("operation:{}", record.invocation_identity),
        started_epoch_ms: record.started_epoch_millis,
        finished_epoch_ms: record.finished_epoch_millis,
        node_count: 1,
        cache_hits: 0,
        failures: failure_count,
    };
    let run_bytes = serialize(&run)?;
    {
        let mut runs = transaction.open_table(RUNS).map_err(|error| StoreError::Table(error.to_string()))?;
        if runs.get(record.run_id.0).map_err(|error| StoreError::Db(error.to_string()))?.is_some() {
            return Err(StoreError::Conflict("operation run identity is already persisted".to_string()));
        }
        runs.insert(record.run_id.0, run_bytes.as_slice())
            .map_err(|error| StoreError::Db(error.to_string()))?;
    }
    let details = operation_run_details(record);
    let detail_bytes = serialize(&details)?;
    let mut table = transaction.open_table(RUN_DETAILS).map_err(|error| StoreError::Table(error.to_string()))?;
    table
        .insert(record.run_id.0, detail_bytes.as_slice())
        .map_err(|error| StoreError::Db(error.to_string()))?;
    Ok(())
}

#[allow(
    unknown_lints,
    assertion_density,
    reason = "one metadata-only projection keeps terminal status and exact backend route visible"
)]
fn operation_run_details(record: &crate::operation::RunRecord) -> RunDetailRecord {
    let status = if record.state == crate::operation::RunState::Completed {
        RunNodeDetailStatus::Completed
    } else {
        RunNodeDetailStatus::Failed
    };
    let node_id = crate::engine::workflow::NodeId(OPERATION_RUN_NODE_ID);
    RunDetailRecord {
        run_id: record.run_id,
        nodes: vec![RunNodeDetail {
            node_id,
            node_label: format!(
                "operation:{}:{}:{}",
                record.backend_attempt.route.backend_class.label(),
                record.backend_attempt.route.backend_identity,
                record.backend_attempt.state.label()
            ),
            status,
            cached: false,
            outputs: Vec::new(),
            error: (record.state != crate::operation::RunState::Completed).then(|| record.diagnostic.clone()),
        }],
        observed_completion_order: vec![node_id],
    }
}

fn terminal_matches_start(current: &crate::operation::RunRecord, terminal: &crate::operation::RunRecord) -> bool {
    current.state == crate::operation::RunState::Started
        && current.run_id == terminal.run_id
        && current.backend_attempt.attempt_identity == terminal.backend_attempt.attempt_identity
        && current.backend_attempt.route == terminal.backend_attempt.route
        && current.backend_attempt.state == crate::operation::BackendAttemptState::Admitted
}

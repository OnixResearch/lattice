impl Store {
    pub fn create_workflow_approval_suspension(
        &self,
        request: &crate::workflow::gate::ApprovalRequest,
        run: &RunRecord,
        details: &RunDetailRecord,
        replay_events: &[ReplayEventRecord],
    ) -> Result<(), StoreError> {
        if request.run_id != run.run_id || request.run_id != details.run_id {
            return Err(StoreError::Conflict(
                "workflow approval suspension records do not share one run identity".to_string(),
            ));
        }
        let request_bytes = serialize(request)?;
        let run_bytes = serialize(run)?;
        let detail_bytes = serialize(details)?;
        let replay_bytes = serialize(replay_events)?;
        let next_after_record = run
            .run_id
            .0
            .checked_add(NEXT_RUN_ID_INCREMENT)
            .ok_or_else(|| StoreError::Conflict(RUN_ID_EXHAUSTED_MESSAGE.to_string()))?;
        let txn = self.db.begin_write().map_err(|error| StoreError::Db(error.to_string()))?;
        {
            let mut approvals =
                txn.open_table(WORKFLOW_APPROVAL_REQUESTS).map_err(|error| StoreError::Table(error.to_string()))?;
            if approvals.get(request.id.as_str()).map_err(|error| StoreError::Db(error.to_string()))?.is_some() {
                return Err(StoreError::Conflict("workflow approval request is already persisted".to_string()));
            }
            approvals
                .insert(request.id.as_str(), request_bytes.as_slice())
                .map_err(|error| StoreError::Db(error.to_string()))?;
        }
        {
            let mut index =
                txn.open_table(WORKFLOW_APPROVAL_RUN_INDEX).map_err(|error| StoreError::Table(error.to_string()))?;
            if index.get(run.run_id.0).map_err(|error| StoreError::Db(error.to_string()))?.is_some() {
                return Err(StoreError::Conflict("workflow approval run is already indexed".to_string()));
            }
            index.insert(run.run_id.0, request.id.as_str()).map_err(|error| StoreError::Db(error.to_string()))?;
        }
        {
            let mut runs = txn.open_table(RUNS).map_err(|error| StoreError::Table(error.to_string()))?;
            if runs.get(run.run_id.0).map_err(|error| StoreError::Db(error.to_string()))?.is_some() {
                return Err(StoreError::Conflict("workflow approval run identity is already persisted".to_string()));
            }
            runs.insert(run.run_id.0, run_bytes.as_slice()).map_err(|error| StoreError::Db(error.to_string()))?;
        }
        txn.open_table(RUN_DETAILS)
            .map_err(|error| StoreError::Table(error.to_string()))?
            .insert(run.run_id.0, detail_bytes.as_slice())
            .map_err(|error| StoreError::Db(error.to_string()))?;
        txn.open_table(RUN_REPLAY_EVENTS)
            .map_err(|error| StoreError::Table(error.to_string()))?
            .insert(run.run_id.0, replay_bytes.as_slice())
            .map_err(|error| StoreError::Db(error.to_string()))?;
        {
            let mut counters = txn.open_table(RUN_COUNTERS).map_err(|error| StoreError::Table(error.to_string()))?;
            let current = counters
                .get(RUN_ID_COUNTER_KEY)
                .map_err(|error| StoreError::Db(error.to_string()))?
                .map(|value| value.value())
                .unwrap_or(INITIAL_RUN_ID_VALUE);
            if next_after_record > current {
                counters
                    .insert(RUN_ID_COUNTER_KEY, next_after_record)
                    .map_err(|error| StoreError::Db(error.to_string()))?;
            }
        }
        txn.commit().map_err(|error| StoreError::Db(error.to_string()))?;
        Ok(())
    }

    pub fn load_workflow_approval(
        &self,
        request_id: &crate::workflow::gate::ApprovalRequestId,
    ) -> Result<Option<crate::workflow::gate::ApprovalRequest>, StoreError> {
        let txn = self.db.begin_read().map_err(|error| StoreError::Db(error.to_string()))?;
        let table = txn.open_table(WORKFLOW_APPROVAL_REQUESTS).map_err(|error| StoreError::Table(error.to_string()))?;
        match table.get(request_id.as_str()).map_err(|error| StoreError::Db(error.to_string()))? {
            Some(value) => deserialize(value.value()).map(Some),
            None => Ok(None),
        }
    }

    pub fn load_workflow_approval_for_run(
        &self,
        run_id: RunId,
    ) -> Result<Option<crate::workflow::gate::ApprovalRequest>, StoreError> {
        let txn = self.db.begin_read().map_err(|error| StoreError::Db(error.to_string()))?;
        let index =
            txn.open_table(WORKFLOW_APPROVAL_RUN_INDEX).map_err(|error| StoreError::Table(error.to_string()))?;
        let Some(request_id) = index
            .get(run_id.0)
            .map_err(|error| StoreError::Db(error.to_string()))?
            .map(|value| value.value().to_owned())
        else {
            return Ok(None);
        };
        let approvals =
            txn.open_table(WORKFLOW_APPROVAL_REQUESTS).map_err(|error| StoreError::Table(error.to_string()))?;
        match approvals.get(request_id.as_str()).map_err(|error| StoreError::Db(error.to_string()))? {
            Some(value) => deserialize(value.value()).map(Some),
            None => Err(StoreError::Conflict("workflow approval run index references a missing request".to_string())),
        }
    }

    pub fn list_workflow_approvals(
        &self,
        count: u32,
    ) -> Result<Vec<crate::workflow::gate::ApprovalRequest>, StoreError> {
        let maximum_items = usize::try_from(count)
            .map_err(|_| StoreError::Conflict("workflow approval list limit is unsupported".to_string()))?;
        let txn = self.db.begin_read().map_err(|error| StoreError::Db(error.to_string()))?;
        let table = txn.open_table(WORKFLOW_APPROVAL_REQUESTS).map_err(|error| StoreError::Table(error.to_string()))?;
        let mut requests = Vec::with_capacity(maximum_items);
        for entry in table.iter().map_err(|error| StoreError::Db(error.to_string()))? {
            let (_key, value) = entry.map_err(|error| StoreError::Db(error.to_string()))?;
            requests.push(deserialize(value.value())?);
            requests.sort_by(|left: &crate::workflow::gate::ApprovalRequest, right| {
                right.requested_epoch_millis.cmp(&left.requested_epoch_millis).then_with(|| left.id.cmp(&right.id))
            });
            requests.truncate(maximum_items);
        }
        debug_assert!(requests.len() <= maximum_items);
        Ok(requests)
    }
}

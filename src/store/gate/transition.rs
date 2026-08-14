impl Store {
    pub fn transition_workflow_approval(
        &self,
        expected_version: u64,
        request: &crate::workflow::gate::ApprovalRequest,
    ) -> Result<(), StoreError> {
        let request_bytes = serialize(request)?;
        let txn = self.db.begin_write().map_err(|error| StoreError::Db(error.to_string()))?;
        {
            let mut table =
                txn.open_table(WORKFLOW_APPROVAL_REQUESTS).map_err(|error| StoreError::Table(error.to_string()))?;
            let current: crate::workflow::gate::ApprovalRequest = {
                let value = table
                    .get(request.id.as_str())
                    .map_err(|error| StoreError::Db(error.to_string()))?
                    .ok_or_else(|| StoreError::Conflict("workflow approval request is missing".to_string()))?;
                deserialize(value.value())?
            };
            let expected_next = expected_version
                .checked_add(WORKFLOW_APPROVAL_VERSION_INCREMENT)
                .ok_or_else(|| StoreError::Conflict("workflow approval version is exhausted".to_string()))?;
            if current.version != expected_version || request.version != expected_next {
                return Err(StoreError::Conflict("workflow approval request changed concurrently".to_string()));
            }
            table
                .insert(request.id.as_str(), request_bytes.as_slice())
                .map_err(|error| StoreError::Db(error.to_string()))?;
        }
        txn.commit().map_err(|error| StoreError::Db(error.to_string()))?;
        Ok(())
    }

    pub fn finish_workflow_approval_run(
        &self,
        expected_version: u64,
        request: &crate::workflow::gate::ApprovalRequest,
        run: &RunRecord,
        details: &RunDetailRecord,
        replay_events: &[ReplayEventRecord],
    ) -> Result<(), StoreError> {
        if request.run_id != run.run_id || request.run_id != details.run_id {
            return Err(StoreError::Conflict(
                "workflow approval completion records do not share one run identity".to_string(),
            ));
        }
        let request_bytes = serialize(request)?;
        let run_bytes = serialize(run)?;
        let detail_bytes = serialize(details)?;
        let replay_bytes = serialize(replay_events)?;
        let txn = self.db.begin_write().map_err(|error| StoreError::Db(error.to_string()))?;
        {
            let mut approvals =
                txn.open_table(WORKFLOW_APPROVAL_REQUESTS).map_err(|error| StoreError::Table(error.to_string()))?;
            let current: crate::workflow::gate::ApprovalRequest = {
                let value = approvals
                    .get(request.id.as_str())
                    .map_err(|error| StoreError::Db(error.to_string()))?
                    .ok_or_else(|| StoreError::Conflict("workflow approval request is missing".to_string()))?;
                deserialize(value.value())?
            };
            let expected_next = expected_version
                .checked_add(WORKFLOW_APPROVAL_VERSION_INCREMENT)
                .ok_or_else(|| StoreError::Conflict("workflow approval version is exhausted".to_string()))?;
            if current.version != expected_version || request.version != expected_next {
                return Err(StoreError::Conflict("workflow approval request changed concurrently".to_string()));
            }
            approvals
                .insert(request.id.as_str(), request_bytes.as_slice())
                .map_err(|error| StoreError::Db(error.to_string()))?;
        }
        {
            let index =
                txn.open_table(WORKFLOW_APPROVAL_RUN_INDEX).map_err(|error| StoreError::Table(error.to_string()))?;
            let active = index
                .get(run.run_id.0)
                .map_err(|error| StoreError::Db(error.to_string()))?
                .map(|value| value.value().to_owned())
                .ok_or_else(|| StoreError::Conflict("workflow approval run index is missing".to_string()))?;
            if active != request.id.as_str() {
                return Err(StoreError::Conflict("workflow approval run owner changed concurrently".to_string()));
            }
        }
        txn.open_table(RUNS)
            .map_err(|error| StoreError::Table(error.to_string()))?
            .insert(run.run_id.0, run_bytes.as_slice())
            .map_err(|error| StoreError::Db(error.to_string()))?;
        txn.open_table(RUN_DETAILS)
            .map_err(|error| StoreError::Table(error.to_string()))?
            .insert(run.run_id.0, detail_bytes.as_slice())
            .map_err(|error| StoreError::Db(error.to_string()))?;
        txn.open_table(RUN_REPLAY_EVENTS)
            .map_err(|error| StoreError::Table(error.to_string()))?
            .insert(run.run_id.0, replay_bytes.as_slice())
            .map_err(|error| StoreError::Db(error.to_string()))?;
        txn.commit().map_err(|error| StoreError::Db(error.to_string()))?;
        Ok(())
    }

    pub fn advance_workflow_approval_gate(&self, input: WorkflowApprovalAdvance<'_>) -> Result<(), StoreError> {
        let WorkflowApprovalAdvance {
            expected_version,
            completed_request,
            next_request,
            run,
            details,
            replay_events,
        } = input;
        if completed_request.run_id != next_request.run_id
            || completed_request.run_id != run.run_id
            || completed_request.run_id != details.run_id
        {
            return Err(StoreError::Conflict(
                "workflow approval gate records do not share one run identity".to_string(),
            ));
        }
        let completed_bytes = serialize(completed_request)?;
        let next_bytes = serialize(next_request)?;
        let run_bytes = serialize(run)?;
        let detail_bytes = serialize(details)?;
        let replay_bytes = serialize(replay_events)?;
        let txn = self.db.begin_write().map_err(|error| StoreError::Db(error.to_string()))?;
        {
            let mut approvals =
                txn.open_table(WORKFLOW_APPROVAL_REQUESTS).map_err(|error| StoreError::Table(error.to_string()))?;
            let current: crate::workflow::gate::ApprovalRequest = {
                let value = approvals
                    .get(completed_request.id.as_str())
                    .map_err(|error| StoreError::Db(error.to_string()))?
                    .ok_or_else(|| StoreError::Conflict("workflow approval request is missing".to_string()))?;
                deserialize(value.value())?
            };
            let expected_next = expected_version
                .checked_add(WORKFLOW_APPROVAL_VERSION_INCREMENT)
                .ok_or_else(|| StoreError::Conflict("workflow approval version is exhausted".to_string()))?;
            if current.version != expected_version || completed_request.version != expected_next {
                return Err(StoreError::Conflict("workflow approval request changed concurrently".to_string()));
            }
            if approvals
                .get(next_request.id.as_str())
                .map_err(|error| StoreError::Db(error.to_string()))?
                .is_some()
            {
                return Err(StoreError::Conflict("next workflow approval request is already persisted".to_string()));
            }
            approvals
                .insert(completed_request.id.as_str(), completed_bytes.as_slice())
                .map_err(|error| StoreError::Db(error.to_string()))?;
            approvals
                .insert(next_request.id.as_str(), next_bytes.as_slice())
                .map_err(|error| StoreError::Db(error.to_string()))?;
        }
        {
            let mut index =
                txn.open_table(WORKFLOW_APPROVAL_RUN_INDEX).map_err(|error| StoreError::Table(error.to_string()))?;
            let active = index
                .get(run.run_id.0)
                .map_err(|error| StoreError::Db(error.to_string()))?
                .map(|value| value.value().to_owned())
                .ok_or_else(|| StoreError::Conflict("workflow approval run index is missing".to_string()))?;
            if active != completed_request.id.as_str() {
                return Err(StoreError::Conflict("workflow approval run owner changed concurrently".to_string()));
            }
            index
                .insert(run.run_id.0, next_request.id.as_str())
                .map_err(|error| StoreError::Db(error.to_string()))?;
        }
        txn.open_table(RUNS)
            .map_err(|error| StoreError::Table(error.to_string()))?
            .insert(run.run_id.0, run_bytes.as_slice())
            .map_err(|error| StoreError::Db(error.to_string()))?;
        txn.open_table(RUN_DETAILS)
            .map_err(|error| StoreError::Table(error.to_string()))?
            .insert(run.run_id.0, detail_bytes.as_slice())
            .map_err(|error| StoreError::Db(error.to_string()))?;
        txn.open_table(RUN_REPLAY_EVENTS)
            .map_err(|error| StoreError::Table(error.to_string()))?
            .insert(run.run_id.0, replay_bytes.as_slice())
            .map_err(|error| StoreError::Db(error.to_string()))?;
        txn.commit().map_err(|error| StoreError::Db(error.to_string()))?;
        Ok(())
    }
}

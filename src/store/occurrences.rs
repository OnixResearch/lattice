use redb::ReadableDatabase as _;
use redb::ReadableTable as _;

impl super::Store {
    /// Read exact recovery evidence even while ordinary mutations are fenced.
    pub fn load_schedule_occurrence_for_recovery(
        &self,
        identity: &str,
    ) -> Result<Option<crate::scheduler::ScheduleOccurrence>, super::StoreError> {
        let txn = self.db.begin_write().map_err(|error| super::StoreError::Db(error.to_string()))?;
        match super::admission::load_optional_for_update(&txn, identity)? {
            Some(row) => Ok(Some(row)),
            None => {
                let fences = txn
                    .open_table(super::quarantine::FENCES)
                    .map_err(|error| super::StoreError::Table(error.to_string()))?;
                fences
                    .get(identity)
                    .map_err(|error| super::StoreError::Db(error.to_string()))?
                    .map(|bytes| super::quarantine::decode_fence(identity, bytes.value()).map(|fence| fence.expected))
                    .transpose()
            }
        }
    }

    /// Admit dispatch only from the exact current durable active reservation.
    pub fn admit_schedule_occurrence_dispatch(
        &self,
        expected: &crate::scheduler::ScheduleOccurrence,
    ) -> Result<transactional_reconciliation_core::DispatchIntent, super::StoreError> {
        self.ensure_not_quarantined()?;
        let observed = self.load_schedule_occurrence(&expected.identity)?;
        crate::reconciliation::admit_occurrence_readback(expected, observed.as_ref())
            .map_err(|error| super::StoreError::Conflict(error.to_string()))
    }

    pub fn load_schedule_occurrence(
        &self,
        identity: &str,
    ) -> Result<Option<crate::scheduler::ScheduleOccurrence>, super::StoreError> {
        let txn = self.db.begin_read().map_err(|error| super::StoreError::Db(error.to_string()))?;
        let occurrences = txn
            .open_table(super::admission::OCCURRENCES)
            .map_err(|error| super::StoreError::Table(error.to_string()))?;
        let occurrence = occurrences
            .get(identity)
            .map_err(|error| super::StoreError::Db(error.to_string()))?
            .map(|data| super::deserialize::<crate::scheduler::ScheduleOccurrence>(data.value()))
            .transpose()?;
        if let Some(occurrence) = &occurrence {
            crate::scheduler::validate_schedule_occurrence(occurrence)
                .map_err(|error| super::StoreError::Conflict(error.to_string()))?;
        }
        Ok(occurrence)
    }

    pub fn list_schedule_occurrences(
        &self,
        max_occurrence_count: u64,
    ) -> Result<Vec<crate::scheduler::ScheduleOccurrence>, super::StoreError> {
        let max_occurrence_count = usize::try_from(max_occurrence_count)
            .map_err(|error| super::StoreError::Conflict(format!("occurrence list limit is unsupported: {error}")))?;
        let txn = self.db.begin_read().map_err(|error| super::StoreError::Db(error.to_string()))?;
        let occurrences = txn
            .open_table(super::admission::OCCURRENCES)
            .map_err(|error| super::StoreError::Table(error.to_string()))?;
        let mut listed = Vec::with_capacity(max_occurrence_count);
        let iter = occurrences.iter().map_err(|error| super::StoreError::Db(error.to_string()))?;
        for entry in iter {
            if listed.len() >= max_occurrence_count {
                break;
            }
            let (_identity, data) = entry.map_err(|error| super::StoreError::Db(error.to_string()))?;
            let occurrence = super::deserialize::<crate::scheduler::ScheduleOccurrence>(data.value())?;
            crate::scheduler::validate_schedule_occurrence(&occurrence)
                .map_err(|error| super::StoreError::Conflict(error.to_string()))?;
            listed.push(occurrence);
        }
        let fences = txn
            .open_table(super::quarantine::FENCES)
            .map_err(|error| super::StoreError::Table(error.to_string()))?;
        for entry in fences.iter().map_err(|error| super::StoreError::Db(error.to_string()))? {
            if listed.len() >= max_occurrence_count {
                break;
            }
            let (identity, bytes) = entry.map_err(|error| super::StoreError::Db(error.to_string()))?;
            if occurrences
                .get(identity.value())
                .map_err(|error| super::StoreError::Db(error.to_string()))?
                .is_none()
            {
                let fence = super::quarantine::decode_fence(identity.value(), bytes.value())?;
                listed.push(fence.expected);
            }
        }
        listed.sort_by(|left, right| left.identity.cmp(&right.identity));
        assert!(listed.len() <= max_occurrence_count);
        Ok(listed)
    }

    pub fn settle_schedule_occurrence(
        &self,
        identity: &str,
        run_id: crate::engine::RunId,
        status: crate::scheduler::ScheduleOccurrenceStatus,
    ) -> Result<crate::scheduler::ScheduleOccurrence, super::StoreError> {
        let _admission = self.occurrence_admission.lock().map_err(|_| super::StoreError::Quarantined)?;
        let txn = self.db.begin_write().map_err(|error| super::StoreError::Db(error.to_string()))?;
        let current = load_recovery_for_update(&txn, identity)?;
        let record = load_run_in_transaction(&txn, run_id)?.ok_or(super::StoreError::Quarantined)?;
        if record.workflow_name != current.workflow_name {
            return Err(super::StoreError::Quarantined);
        }
        let owner = match &record.owner {
            super::RunOwner::ScheduleOccurrence { identity: owner } if owner == identity => {
                crate::scheduler::ScheduleOccurrenceRunOwnerObservation::Exact
            }
            _ => crate::scheduler::ScheduleOccurrenceRunOwnerObservation::Foreign,
        };
        let decision = crate::scheduler::classify_schedule_occurrence_recovery(
            &current,
            Some(crate::scheduler::ScheduleOccurrenceRunObservation {
                run_id: record.run_id,
                owner,
                finished: record.finished_epoch_ms.is_some(),
                failures: record.failures,
            }),
        )
        .map_err(|error| super::StoreError::Conflict(error.to_string()))?;
        if !matches!(decision, crate::scheduler::ScheduleOccurrenceRecoveryDecision::Settle {
            run_id: exact_run, status: exact_status,
        } if exact_run == run_id && exact_status == status)
        {
            return Err(super::StoreError::Quarantined);
        }
        let settled = crate::scheduler::settle_schedule_occurrence(&current, run_id, status)
            .map_err(|error| super::StoreError::Conflict(error.to_string()))?;
        super::admission::save_in_transaction(&txn, &settled)?;
        super::quarantine::remove_fence(&txn, identity)?;
        self.finish_occurrence_commit(txn, &settled)?;
        Ok(settled)
    }

    pub fn abandon_schedule_occurrence(
        &self,
        identity: &str,
        is_unknown_outcome_acknowledged: bool,
    ) -> Result<crate::scheduler::ScheduleOccurrence, super::StoreError> {
        let _admission = self.occurrence_admission.lock().map_err(|_| super::StoreError::Quarantined)?;
        let txn = self.db.begin_write().map_err(|error| super::StoreError::Db(error.to_string()))?;
        let current = load_recovery_for_update(&txn, identity)?;
        if run_exists(&txn, current.active_run_id)? {
            return Err(super::StoreError::Conflict(
                "schedule occurrence has persisted active-run evidence; recover before abandonment".to_string(),
            ));
        }
        let abandoned = crate::scheduler::abandon_schedule_occurrence(&current, is_unknown_outcome_acknowledged)
            .map_err(|error| super::StoreError::Conflict(error.to_string()))?;
        advance_unpublished_slot(&txn, &current)?;
        super::admission::save_in_transaction(&txn, &abandoned)?;
        super::quarantine::remove_fence(&txn, identity)?;
        self.finish_occurrence_commit(txn, &abandoned)?;
        Ok(abandoned)
    }
}

pub(super) fn load_recovery_for_update(
    txn: &redb::WriteTransaction,
    identity: &str,
) -> Result<crate::scheduler::ScheduleOccurrence, super::StoreError> {
    let observed = super::admission::load_optional_for_update(txn, identity)?;
    let fences = txn
        .open_table(super::quarantine::FENCES)
        .map_err(|error| super::StoreError::Table(error.to_string()))?;
    let bytes = fences.get(identity).map_err(|error| super::StoreError::Db(error.to_string()))?;
    let Some(bytes) = bytes else {
        return observed.ok_or(super::StoreError::Quarantined);
    };
    let fence = super::quarantine::decode_fence(identity, bytes.value())?;
    if observed.as_ref().is_some_and(|row| row != &fence.expected) {
        return Err(super::StoreError::Quarantined);
    }
    Ok(observed.unwrap_or(fence.expected))
}

fn load_run_in_transaction(
    txn: &redb::WriteTransaction,
    run_id: crate::engine::RunId,
) -> Result<Option<super::RunRecord>, super::StoreError> {
    let runs = txn.open_table(super::RUNS).map_err(|error| super::StoreError::Table(error.to_string()))?;
    runs.get(run_id.0)
        .map_err(|error| super::StoreError::Db(error.to_string()))?
        .map(|bytes| super::deserialize(bytes.value()))
        .transpose()
}

pub(super) fn advance_unpublished_slot(
    txn: &redb::WriteTransaction,
    occurrence: &crate::scheduler::ScheduleOccurrence,
) -> Result<(), super::StoreError> {
    if super::admission::load_optional_for_update(txn, &occurrence.identity)?.is_some() {
        return Ok(());
    }
    let mut schedules =
        txn.open_table(super::SCHEDULES).map_err(|error| super::StoreError::Table(error.to_string()))?;
    let existing = schedules
        .get(occurrence.schedule_id.as_str())
        .map_err(|error| super::StoreError::Db(error.to_string()))?
        .map(|bytes| super::deserialize::<crate::scheduler::ScheduleDefinition>(bytes.value()))
        .transpose()?;
    let Some(mut schedule) = existing else {
        return Ok(());
    };
    let request = crate::scheduler::plan_schedule_occurrence(
        &crate::scheduler::DueRunPlan {
            schedule_id: occurrence.schedule_id.clone(),
            workflow_name: occurrence.workflow_name.clone(),
            due_epoch_ms: occurrence.due_epoch_ms,
            next_run_epoch_ms: occurrence.next_run_epoch_ms,
        },
        &occurrence.workflow_revision,
    )
    .map_err(|error| super::StoreError::Conflict(error.to_string()))?;
    if crate::scheduler::validate_schedule_occurrence_claim(&schedule, &request).is_ok() {
        debug_assert_eq!(schedule.id, occurrence.schedule_id);
        debug_assert_eq!(schedule.workflow_name, occurrence.workflow_name);
        schedule.next_run_epoch_ms = occurrence.next_run_epoch_ms;
        let bytes = super::serialize(&schedule)?;
        schedules
            .insert(schedule.id.as_str(), bytes.as_slice())
            .map_err(|error| super::StoreError::Db(error.to_string()))?;
    }
    Ok(())
}

pub(super) fn run_exists(
    txn: &redb::WriteTransaction,
    run_id: crate::engine::RunId,
) -> Result<bool, super::StoreError> {
    let runs = txn.open_table(super::RUNS).map_err(|error| super::StoreError::Table(error.to_string()))?;
    let is_present = runs.get(run_id.0).map_err(|error| super::StoreError::Db(error.to_string()))?.is_some();
    Ok(is_present)
}

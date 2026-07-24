//! Atomic durable schedule-occurrence storage.

use redb::ReadableDatabase;
use redb::ReadableTable;

const OCCURRENCES: redb::TableDefinition<&str, &[u8]> = redb::TableDefinition::new("schedule_occurrences_v1");

pub(super) fn ensure_table(txn: &redb::WriteTransaction) -> Result<(), super::StoreError> {
    let _ = txn.open_table(OCCURRENCES).map_err(|error| super::StoreError::Table(error.to_string()))?;
    Ok(())
}

impl super::Store {
    // r[impl lattice.scheduling.scheduler_occurrence_admission]
    pub fn claim_schedule_occurrence(
        &self,
        request: &crate::scheduler::ScheduleOccurrenceClaimRequest,
    ) -> Result<crate::scheduler::ScheduleOccurrenceClaimResolution, super::StoreError> {
        let txn = self.db.begin_write().map_err(|error| super::StoreError::Db(error.to_string()))?;
        let existing = {
            let occurrences =
                txn.open_table(OCCURRENCES).map_err(|error| super::StoreError::Table(error.to_string()))?;
            occurrences
                .get(request.identity.as_str())
                .map_err(|error| super::StoreError::Db(error.to_string()))?
                .map(|data| super::deserialize::<crate::scheduler::ScheduleOccurrence>(data.value()))
                .transpose()?
        };
        if let Some(existing) = existing {
            crate::scheduler::validate_schedule_occurrence(&existing)
                .map_err(|error| super::StoreError::Conflict(error.to_string()))?;
            return Ok(crate::scheduler::ScheduleOccurrenceClaimResolution::Existing(Box::new(existing)));
        }

        let mut schedule = {
            let schedules =
                txn.open_table(super::SCHEDULES).map_err(|error| super::StoreError::Table(error.to_string()))?;
            let Some(data) = schedules
                .get(request.schedule_id.as_str())
                .map_err(|error| super::StoreError::Db(error.to_string()))?
            else {
                return Ok(crate::scheduler::ScheduleOccurrenceClaimResolution::MissingSchedule);
            };
            super::deserialize::<crate::scheduler::ScheduleDefinition>(data.value())?
        };
        if crate::scheduler::validate_schedule_occurrence_claim(&schedule, request).is_err() {
            return Ok(crate::scheduler::ScheduleOccurrenceClaimResolution::StaleSchedule {
                current_next_run_epoch_ms: schedule.next_run_epoch_ms,
            });
        }

        let run_id = super::reserve_run_id(&txn)?;
        let occurrence = request.occurrence(run_id);
        crate::scheduler::validate_schedule_occurrence(&occurrence)
            .map_err(|error| super::StoreError::Conflict(error.to_string()))?;
        schedule.next_run_epoch_ms = request.next_run_epoch_ms;
        let occurrence_data = super::serialize(&occurrence)?;
        let schedule_data = super::serialize(&schedule)?;
        {
            let mut occurrences =
                txn.open_table(OCCURRENCES).map_err(|error| super::StoreError::Table(error.to_string()))?;
            occurrences
                .insert(occurrence.identity.as_str(), occurrence_data.as_slice())
                .map_err(|error| super::StoreError::Db(error.to_string()))?;
        }
        {
            let mut schedules =
                txn.open_table(super::SCHEDULES).map_err(|error| super::StoreError::Table(error.to_string()))?;
            schedules
                .insert(schedule.id.as_str(), schedule_data.as_slice())
                .map_err(|error| super::StoreError::Db(error.to_string()))?;
        }
        txn.commit().map_err(|error| super::StoreError::Db(error.to_string()))?;
        Ok(crate::scheduler::ScheduleOccurrenceClaimResolution::Inserted(Box::new(occurrence)))
    }

    pub fn load_schedule_occurrence(
        &self,
        identity: &str,
    ) -> Result<Option<crate::scheduler::ScheduleOccurrence>, super::StoreError> {
        let txn = self.db.begin_read().map_err(|error| super::StoreError::Db(error.to_string()))?;
        let occurrences = txn.open_table(OCCURRENCES).map_err(|error| super::StoreError::Table(error.to_string()))?;
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
        let occurrences = txn.open_table(OCCURRENCES).map_err(|error| super::StoreError::Table(error.to_string()))?;
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
        assert!(listed.len() <= max_occurrence_count);
        Ok(listed)
    }

    pub fn settle_schedule_occurrence(
        &self,
        identity: &str,
        run_id: crate::engine::RunId,
        status: crate::scheduler::ScheduleOccurrenceStatus,
    ) -> Result<crate::scheduler::ScheduleOccurrence, super::StoreError> {
        let txn = self.db.begin_write().map_err(|error| super::StoreError::Db(error.to_string()))?;
        let current = load_for_update(&txn, identity)?;
        let settled = crate::scheduler::settle_schedule_occurrence(&current, run_id, status)
            .map_err(|error| super::StoreError::Conflict(error.to_string()))?;
        save_in_transaction(&txn, &settled)?;
        txn.commit().map_err(|error| super::StoreError::Db(error.to_string()))?;
        Ok(settled)
    }

    pub fn retry_schedule_occurrence(
        &self,
        identity: &str,
        workflow_revision: &str,
        is_duplicate_risk_acknowledged: bool,
    ) -> Result<crate::scheduler::ScheduleOccurrence, super::StoreError> {
        let txn = self.db.begin_write().map_err(|error| super::StoreError::Db(error.to_string()))?;
        let current = load_for_update(&txn, identity)?;
        if run_exists(&txn, current.active_run_id)? {
            return Err(super::StoreError::Conflict(
                "schedule occurrence has persisted active-run evidence; recover before retry".to_string(),
            ));
        }
        let run_id = super::reserve_run_id(&txn)?;
        let retried = crate::scheduler::retry_schedule_occurrence(
            &current,
            workflow_revision,
            run_id,
            is_duplicate_risk_acknowledged,
        )
        .map_err(|error| super::StoreError::Conflict(error.to_string()))?;
        save_in_transaction(&txn, &retried)?;
        txn.commit().map_err(|error| super::StoreError::Db(error.to_string()))?;
        Ok(retried)
    }

    pub fn abandon_schedule_occurrence(
        &self,
        identity: &str,
        is_unknown_outcome_acknowledged: bool,
    ) -> Result<crate::scheduler::ScheduleOccurrence, super::StoreError> {
        let txn = self.db.begin_write().map_err(|error| super::StoreError::Db(error.to_string()))?;
        let current = load_for_update(&txn, identity)?;
        if run_exists(&txn, current.active_run_id)? {
            return Err(super::StoreError::Conflict(
                "schedule occurrence has persisted active-run evidence; recover before abandonment".to_string(),
            ));
        }
        let abandoned = crate::scheduler::abandon_schedule_occurrence(&current, is_unknown_outcome_acknowledged)
            .map_err(|error| super::StoreError::Conflict(error.to_string()))?;
        save_in_transaction(&txn, &abandoned)?;
        txn.commit().map_err(|error| super::StoreError::Db(error.to_string()))?;
        Ok(abandoned)
    }
}

fn load_for_update(
    txn: &redb::WriteTransaction,
    identity: &str,
) -> Result<crate::scheduler::ScheduleOccurrence, super::StoreError> {
    let occurrences = txn.open_table(OCCURRENCES).map_err(|error| super::StoreError::Table(error.to_string()))?;
    let Some(data) = occurrences.get(identity).map_err(|error| super::StoreError::Db(error.to_string()))? else {
        return Err(super::StoreError::Conflict("schedule occurrence does not exist".to_string()));
    };
    let occurrence = super::deserialize::<crate::scheduler::ScheduleOccurrence>(data.value())?;
    crate::scheduler::validate_schedule_occurrence(&occurrence)
        .map_err(|error| super::StoreError::Conflict(error.to_string()))?;
    Ok(occurrence)
}

fn save_in_transaction(
    txn: &redb::WriteTransaction,
    occurrence: &crate::scheduler::ScheduleOccurrence,
) -> Result<(), super::StoreError> {
    crate::scheduler::validate_schedule_occurrence(occurrence)
        .map_err(|error| super::StoreError::Conflict(error.to_string()))?;
    let data = super::serialize(occurrence)?;
    let mut occurrences = txn.open_table(OCCURRENCES).map_err(|error| super::StoreError::Table(error.to_string()))?;
    occurrences
        .insert(occurrence.identity.as_str(), data.as_slice())
        .map_err(|error| super::StoreError::Db(error.to_string()))?;
    Ok(())
}

fn run_exists(txn: &redb::WriteTransaction, run_id: crate::engine::RunId) -> Result<bool, super::StoreError> {
    let runs = txn.open_table(super::RUNS).map_err(|error| super::StoreError::Table(error.to_string()))?;
    let is_present = runs.get(run_id.0).map_err(|error| super::StoreError::Db(error.to_string()))?.is_some();
    Ok(is_present)
}

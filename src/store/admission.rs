//! Atomic durable schedule-occurrence storage.

use redb::ReadableDatabase;
use redb::ReadableTable;
use redb::ReadableTableMetadata as _;

pub(super) const OCCURRENCES: redb::TableDefinition<&str, &[u8]> =
    redb::TableDefinition::new("schedule_occurrences_v1");

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
        self.claim_schedule_occurrence_with_fault(request, super::CommitFault::None)
    }

    // r[impl lattice.persistence_uncertainty.commit_outcomes]
    // r[impl lattice.persistence_uncertainty.effect_gate]
    pub fn claim_schedule_occurrence_with_fault(
        &self,
        request: &crate::scheduler::ScheduleOccurrenceClaimRequest,
        fault: super::CommitFault,
    ) -> Result<crate::scheduler::ScheduleOccurrenceClaimResolution, super::StoreError> {
        let _admission = self.occurrence_admission.lock().map_err(|_| super::StoreError::Quarantined)?;
        self.ensure_not_quarantined()?;
        let txn = self.db.begin_write().map_err(|error| super::StoreError::Db(error.to_string()))?;
        if !txn
            .open_table(super::quarantine::FENCES)
            .map_err(|error| super::StoreError::Table(error.to_string()))?
            .is_empty()
            .map_err(|error| super::StoreError::Db(error.to_string()))?
        {
            return Err(super::StoreError::Quarantined);
        }
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

        let schedule = {
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
        drop(schedule);

        let run_id = super::reserve_run_id(&txn)?;
        let occurrence = request.occurrence(run_id);
        crate::scheduler::validate_schedule_occurrence(&occurrence)
            .map_err(|error| super::StoreError::Conflict(error.to_string()))?;
        // Reserve the run identity and uncertainty fence before the publication
        // transaction. A failed or interrupted publication cannot reuse this run.
        super::quarantine::write_fence(&txn, &occurrence, false)?;
        if txn.commit().is_err() {
            let decision = crate::reconciliation::classify_occurrence_commit(
                &occurrence,
                transactional_reconciliation_core::CommitObservation::OutcomeUnknown,
            )
            .map_err(|error| super::StoreError::Conflict(error.to_string()))?;
            self.quarantine(occurrence.identity.clone(), decision)?;
            return Err(super::StoreError::PublicationUnknown);
        }
        let txn = self.db.begin_write().map_err(|_| super::StoreError::PublicationUnknown)?;
        let latest = {
            let schedules =
                txn.open_table(super::SCHEDULES).map_err(|error| super::StoreError::Table(error.to_string()))?;
            let bytes = schedules
                .get(request.schedule_id.as_str())
                .map_err(|error| super::StoreError::Db(error.to_string()))?
                .ok_or(super::StoreError::PublicationUnknown)?;
            super::deserialize::<crate::scheduler::ScheduleDefinition>(bytes.value())?
        };
        crate::scheduler::validate_schedule_occurrence_claim(&latest, request)
            .map_err(|_| super::StoreError::PublicationUnknown)?;
        let mut schedule = latest;
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
        if fault == super::CommitFault::CommitOutcomeUnknown {
            let decision = crate::reconciliation::classify_occurrence_commit(
                &occurrence,
                transactional_reconciliation_core::CommitObservation::OutcomeUnknown,
            )
            .map_err(|error| super::StoreError::Conflict(error.to_string()))?;
            self.quarantine(occurrence.identity.clone(), decision)?;
            return Err(super::StoreError::PublicationUnknown);
        }
        #[cfg(test)]
        if fault == super::CommitFault::ArmCommitCrash {
            tests::arm_commit_crash();
        }
        let commit_result = txn.commit();
        if commit_result.is_err() || fault == super::CommitFault::ObservationLost {
            let decision = crate::reconciliation::classify_occurrence_commit(
                &occurrence,
                transactional_reconciliation_core::CommitObservation::OutcomeUnknown,
            )
            .map_err(|error| super::StoreError::Conflict(error.to_string()))?;
            self.quarantine(occurrence.identity.clone(), decision)?;
            return Err(super::StoreError::PublicationUnknown);
        }
        if self.reconcile_occurrence_publication(&occurrence.identity)?
            != transactional_reconciliation_core::PersistenceState::Published
        {
            return Err(super::StoreError::PublicationUnknown);
        }
        Ok(crate::scheduler::ScheduleOccurrenceClaimResolution::Inserted(Box::new(occurrence)))
    }

    // r[impl lattice.persistence_uncertainty.reconciliation]
    // r[impl lattice.persistence_uncertainty.operator_actions]
    pub fn reconcile_schedule_occurrence_publication(
        &self,
    ) -> Result<transactional_reconciliation_core::PersistenceState, super::StoreError> {
        let _admission = self.occurrence_admission.lock().map_err(|_| super::StoreError::Quarantined)?;
        let identity = match self.quarantine_state()? {
            Some(state) => state.operation_identity,
            None => {
                let txn = self.db.begin_read().map_err(|error| super::StoreError::Db(error.to_string()))?;
                let fences = txn
                    .open_table(super::quarantine::FENCES)
                    .map_err(|error| super::StoreError::Table(error.to_string()))?;
                let first = fences
                    .first()
                    .map_err(|error| super::StoreError::Db(error.to_string()))?
                    .ok_or_else(|| super::StoreError::Conflict("store is not quarantined".to_string()))?;
                first.0.value().to_string()
            }
        };
        self.reconcile_occurrence_publication(&identity)
    }

    fn reconcile_occurrence_publication(
        &self,
        identity: &str,
    ) -> Result<transactional_reconciliation_core::PersistenceState, super::StoreError> {
        let txn = self.db.begin_write().map_err(|error| super::StoreError::Db(error.to_string()))?;
        let fence = {
            let fences = txn
                .open_table(super::quarantine::FENCES)
                .map_err(|error| super::StoreError::Table(error.to_string()))?;
            let bytes = fences
                .get(identity)
                .map_err(|error| super::StoreError::Db(error.to_string()))?
                .ok_or(super::StoreError::Quarantined)?;
            super::quarantine::decode_fence(identity, bytes.value())?
        };
        if fence.repaired_history {
            return Ok(transactional_reconciliation_core::PersistenceState::PublicationUnknown);
        }
        let expected = crate::reconciliation::classify_occurrence_commit(
            &fence.expected,
            transactional_reconciliation_core::CommitObservation::OutcomeUnknown,
        )
        .map_err(|error| super::StoreError::Conflict(error.to_string()))?;
        let observed = load_optional_for_update(&txn, identity)?;
        if observed.as_ref().is_some_and(|row| row != &fence.expected) {
            return Ok(transactional_reconciliation_core::PersistenceState::Conflicting);
        }
        let decision = crate::reconciliation::reconcile_occurrence_commit(expected, observed.as_ref())
            .map_err(|error| super::StoreError::Conflict(error.to_string()))?;
        if decision.quarantine() == transactional_reconciliation_core::QuarantineStatus::Clear {
            super::quarantine::remove_fence(&txn, identity)?;
            self.finish_occurrence_commit(txn, &fence.expected)?;
        }
        Ok(decision.state())
    }

    pub fn retry_schedule_occurrence(
        &self,
        identity: &str,
        workflow_revision: &str,
        is_duplicate_risk_acknowledged: bool,
    ) -> Result<crate::scheduler::ScheduleOccurrence, super::StoreError> {
        let _admission = self.occurrence_admission.lock().map_err(|_| super::StoreError::Quarantined)?;
        let txn = self.db.begin_write().map_err(|error| super::StoreError::Db(error.to_string()))?;
        let current = super::occurrences::load_recovery_for_update(&txn, identity)?;
        if super::occurrences::run_exists(&txn, current.active_run_id)? {
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
        super::occurrences::advance_unpublished_slot(&txn, &current)?;
        save_in_transaction(&txn, &retried)?;
        super::quarantine::remove_fence(&txn, identity)?;
        self.finish_occurrence_commit(txn, &retried)?;
        Ok(retried)
    }

    pub(super) fn finish_occurrence_commit(
        &self,
        txn: redb::WriteTransaction,
        occurrence: &crate::scheduler::ScheduleOccurrence,
    ) -> Result<(), super::StoreError> {
        if txn.commit().is_err() {
            let decision = crate::reconciliation::classify_occurrence_commit(
                occurrence,
                transactional_reconciliation_core::CommitObservation::OutcomeUnknown,
            )
            .map_err(|error| super::StoreError::Conflict(error.to_string()))?;
            self.quarantine(occurrence.identity.clone(), decision)?;
            return Err(super::StoreError::PublicationUnknown);
        }
        self.clear_process_quarantine(&occurrence.identity)
    }
}

pub(super) fn load_optional_for_update(
    txn: &redb::WriteTransaction,
    identity: &str,
) -> Result<Option<crate::scheduler::ScheduleOccurrence>, super::StoreError> {
    let occurrences = txn.open_table(OCCURRENCES).map_err(|error| super::StoreError::Table(error.to_string()))?;
    let Some(data) = occurrences.get(identity).map_err(|error| super::StoreError::Db(error.to_string()))? else {
        return Ok(None);
    };
    let occurrence = super::deserialize::<crate::scheduler::ScheduleOccurrence>(data.value())?;
    crate::scheduler::validate_schedule_occurrence(&occurrence)
        .map_err(|error| super::StoreError::Conflict(error.to_string()))?;
    Ok(Some(occurrence))
}

pub(super) fn save_in_transaction(
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

#[cfg(test)]
mod tests;

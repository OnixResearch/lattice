//! Store-wide persistence quarantine state.

use redb::ReadableDatabase as _;
use redb::ReadableTable as _;
use redb::ReadableTableMetadata as _;

pub(super) const FENCES: redb::TableDefinition<&str, &[u8]> =
    redb::TableDefinition::new("schedule_occurrence_fences_v1");
const MAX_FENCE_BYTES: usize = 65_536;

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Fence {
    pub(super) expected: crate::scheduler::ScheduleOccurrence,
    pub(super) repaired_history: bool,
}

#[derive(serde::Serialize)]
struct FenceDraft<'a> {
    expected: &'a crate::scheduler::ScheduleOccurrence,
    repaired_history: bool,
}

pub(super) fn decode_fence(identity: &str, bytes: &[u8]) -> Result<Fence, super::StoreError> {
    if bytes.len() > MAX_FENCE_BYTES {
        return Err(super::StoreError::Quarantined);
    }
    let fence: Fence = super::deserialize(bytes)?;
    if fence.expected.identity != identity {
        return Err(super::StoreError::Quarantined);
    }
    crate::scheduler::validate_schedule_occurrence(&fence.expected)
        .map_err(|error| super::StoreError::Conflict(error.to_string()))?;
    Ok(fence)
}

pub(super) fn write_fence(
    txn: &redb::WriteTransaction,
    occurrence: &crate::scheduler::ScheduleOccurrence,
    repaired_history: bool,
) -> Result<(), super::StoreError> {
    debug_assert!(occurrence.next_run_epoch_ms > occurrence.due_epoch_ms);
    debug_assert_eq!(occurrence.attempt_run_ids.last().copied(), Some(occurrence.active_run_id));
    let mut fences = txn.open_table(FENCES).map_err(|error| super::StoreError::Table(error.to_string()))?;
    let prior = fences
        .get(occurrence.identity.as_str())
        .map_err(|error| super::StoreError::Db(error.to_string()))?
        .map(|bytes| decode_fence(&occurrence.identity, bytes.value()))
        .transpose()?;
    let encoded = match prior {
        Some(mut prior) if repaired_history => {
            prior.repaired_history = true;
            super::serialize(&prior)?
        }
        Some(_) => return Err(super::StoreError::Quarantined),
        None => super::serialize(&FenceDraft {
            expected: occurrence,
            repaired_history,
        })?,
    };
    if encoded.len() > MAX_FENCE_BYTES {
        return Err(super::StoreError::Conflict("occurrence uncertainty record exceeds its bound".to_string()));
    }
    fences
        .insert(occurrence.identity.as_str(), encoded.as_slice())
        .map_err(|error| super::StoreError::Db(error.to_string()))?;
    Ok(())
}

pub(super) fn remove_fence(txn: &redb::WriteTransaction, identity: &str) -> Result<(), super::StoreError> {
    txn.open_table(FENCES)
        .map_err(|error| super::StoreError::Table(error.to_string()))?
        .remove(identity)
        .map_err(|error| super::StoreError::Db(error.to_string()))?;
    Ok(())
}

// r[impl lattice.persistence_uncertainty.repair_boundary]
// r[impl lattice.persistence_uncertainty.repair_boundary.scenario.records_stay_blocked]
pub(super) fn fence_pending(txn: &redb::WriteTransaction) -> Result<(), super::StoreError> {
    let occurrences = txn
        .open_table(super::admission::OCCURRENCES)
        .map_err(|error| super::StoreError::Table(error.to_string()))?;
    let _ = txn.open_table(FENCES).map_err(|error| super::StoreError::Table(error.to_string()))?;
    for entry in occurrences.iter().map_err(|error| super::StoreError::Db(error.to_string()))? {
        let (_, bytes) = entry.map_err(|error| super::StoreError::Db(error.to_string()))?;
        let occurrence: crate::scheduler::ScheduleOccurrence = super::deserialize(bytes.value())?;
        crate::scheduler::validate_schedule_occurrence(&occurrence)
            .map_err(|error| super::StoreError::Conflict(error.to_string()))?;
        if occurrence.status == crate::scheduler::ScheduleOccurrenceStatus::PendingUnobserved {
            write_fence(txn, &occurrence, true)?;
        }
    }
    Ok(())
}
// r[impl lattice.persistence_uncertainty.quarantine]
// r[impl lattice.persistence_uncertainty.operator_actions]

/// Controlled scheduler commit fault for storage-boundary tests.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommitFault {
    /// Use the real commit result.
    None,
    /// Model a commit error whose publication outcome is unknown.
    CommitOutcomeUnknown,
    /// Commit, then lose the publication observation.
    ObservationLost,
    #[cfg(test)]
    ArmCommitCrash,
}

#[derive(Clone, Debug)]
pub(super) struct State {
    pub(super) operation_identity: String,
}

impl super::Store {
    pub(super) fn ensure_not_quarantined(&self) -> Result<(), super::StoreError> {
        if self.is_quarantined()? {
            return Err(super::StoreError::Quarantined);
        }
        Ok(())
    }

    pub(super) fn quarantine(
        &self,
        operation_identity: String,
        decision: transactional_reconciliation_core::PersistenceDecision,
    ) -> Result<(), super::StoreError> {
        debug_assert!(matches!(
            decision.quarantine(),
            transactional_reconciliation_core::QuarantineStatus::Quarantined(_)
        ));
        let mut guard = self.quarantine.lock().map_err(|_| super::StoreError::Quarantined)?;
        *guard = Some(State { operation_identity });
        Ok(())
    }

    pub(super) fn quarantine_state(&self) -> Result<Option<State>, super::StoreError> {
        self.quarantine.lock().map(|guard| guard.clone()).map_err(|_| super::StoreError::Quarantined)
    }

    pub(super) fn clear_process_quarantine(&self, identity: &str) -> Result<(), super::StoreError> {
        let mut guard = self.quarantine.lock().map_err(|_| super::StoreError::Quarantined)?;
        if guard.as_ref().is_some_and(|state| state.operation_identity == identity) {
            *guard = None;
        }
        Ok(())
    }

    /// Report process-local or durable occurrence uncertainty without hiding recovery evidence.
    pub fn is_quarantined(&self) -> Result<bool, super::StoreError> {
        if self.quarantine.lock().map_err(|_| super::StoreError::Quarantined)?.is_some() {
            return Ok(true);
        }
        let txn = self.db.begin_read().map_err(|error| super::StoreError::Db(error.to_string()))?;
        let fences = txn.open_table(FENCES).map_err(|error| super::StoreError::Table(error.to_string()))?;
        Ok(!fences.is_empty().map_err(|error| super::StoreError::Db(error.to_string()))?)
    }

    pub(crate) fn fence_reopened_occurrences(database: &redb::Database) -> Result<(), super::StoreError> {
        let txn = database.begin_write().map_err(|error| super::StoreError::Db(error.to_string()))?;
        fence_pending(&txn)?;
        txn.commit().map_err(|error| super::StoreError::Db(error.to_string()))
    }
}

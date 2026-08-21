//! Store-wide persistence quarantine state.
// r[impl lattice.persistence_uncertainty.quarantine]
// r[impl lattice.persistence_uncertainty.operator_actions]
// r[impl lattice.persistence_uncertainty.repair_boundary]

/// Controlled scheduler commit fault for storage-boundary tests.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommitFault {
    /// Use the real commit result.
    None,
    /// Model a commit error whose publication outcome is unknown.
    CommitOutcomeUnknown,
    /// Commit, then lose the publication observation.
    ObservationLost,
}

#[derive(Clone, Debug)]
pub(super) struct State {
    pub(super) operation_identity: String,
    pub(super) decision: transactional_reconciliation_core::PersistenceDecision,
}

impl super::Store {
    pub(super) fn ensure_not_quarantined(&self) -> Result<(), super::StoreError> {
        let guard = self.quarantine.lock().map_err(|_| super::StoreError::Quarantined)?;
        if guard.is_some() {
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
        *guard = Some(State {
            operation_identity,
            decision,
        });
        Ok(())
    }

    pub(super) fn quarantine_state(&self) -> Result<Option<State>, super::StoreError> {
        self.quarantine.lock().map(|guard| guard.clone()).map_err(|_| super::StoreError::Quarantined)
    }

    pub(super) fn clear_quarantine(
        &self,
        decision: transactional_reconciliation_core::PersistenceDecision,
    ) -> Result<(), super::StoreError> {
        if decision.quarantine() != transactional_reconciliation_core::QuarantineStatus::Clear {
            return Err(super::StoreError::Quarantined);
        }
        let mut guard = self.quarantine.lock().map_err(|_| super::StoreError::Quarantined)?;
        *guard = None;
        Ok(())
    }

    /// Report whether this process-local store handle is quarantined.
    pub fn is_quarantined(&self) -> Result<bool, super::StoreError> {
        self.quarantine.lock().map(|guard| guard.is_some()).map_err(|_| super::StoreError::Quarantined)
    }
}

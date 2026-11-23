//! The worker's durable execution record reads and compare-and-swap writes.

use lattice_workflow_contract::Identity;

use super::ExecutionRecord;

/// A seam over the store, so a test can land a Cancel between a reload and a compare-and-swap.
pub(super) trait Records {
    /// The stored record of `execution`; `None` when it is absent or the store fails.
    fn load(&self, execution: &Identity) -> Option<ExecutionRecord>;
    /// Replace `current` with `next`; `false` when `current` is no longer stored or the store
    /// fails.
    fn swap(&self, current: &ExecutionRecord, next: &ExecutionRecord) -> bool;
}

impl Records for crate::store::Store {
    fn load(&self, execution: &Identity) -> Option<ExecutionRecord> {
        self.load_workflow_exchange(execution).ok().flatten()
    }

    fn swap(&self, current: &ExecutionRecord, next: &ExecutionRecord) -> bool {
        self.save_workflow_exchange_transition(current, next).is_ok()
    }
}

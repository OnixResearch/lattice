// Public root reexports intentionally keep the `Worker*` prefix for embedders.
#![allow(unknown_lints)]
#![allow(path_segment_repetition)]

/// Host-injected worker service.
pub trait WorkerService {
    /// Enqueues a validated worker task.
    ///
    /// # Errors
    ///
    /// Returns [`DelegationFailure`](crate::worker_team::DelegationFailure) when
    /// the injected host service is unavailable, denies the request, times out,
    /// or fails safely.
    fn enqueue(
        &mut self,
        task: &super::WorkerTask,
    ) -> Result<WorkerHandle, crate::worker_team::DelegationFailure>;

    /// Returns worker status for a known handle.
    ///
    /// # Errors
    ///
    /// Returns [`DelegationFailure`](crate::worker_team::DelegationFailure) when
    /// status lookup fails safely or the handle is not usable.
    fn status(
        &mut self,
        handle: &WorkerHandle,
    ) -> Result<WorkerStatus, crate::worker_team::DelegationFailure>;

    /// Requests worker cancellation.
    ///
    /// # Errors
    ///
    /// Returns [`DelegationFailure`](crate::worker_team::DelegationFailure) when
    /// cancellation fails safely.
    fn cancel(
        &mut self,
        handle: &WorkerHandle,
    ) -> Result<WorkerCancelOutcome, crate::worker_team::DelegationFailure>;

    /// Collects a terminal worker report.
    ///
    /// # Errors
    ///
    /// Returns [`DelegationFailure`](crate::worker_team::DelegationFailure) when
    /// collection fails or produces a malformed report.
    fn collect(
        &mut self,
        handle: &WorkerHandle,
    ) -> Result<super::Report, crate::worker_team::DelegationFailure>;
}

/// Worker service handle returned by an injected service.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkerHandle {
    id: String,
}

impl WorkerHandle {
    /// Creates a worker handle after validating its safe identity.
    ///
    /// # Errors
    ///
    /// Returns [`DelegationFailure`](crate::worker_team::DelegationFailure) when
    /// the handle id is empty, too long, or unsafe.
    pub fn new(id: impl Into<String>) -> Result<Self, crate::worker_team::DelegationFailure> {
        Ok(Self {
            id: crate::worker_team::core::validate_label(
                crate::worker_team::WORKER_HANDLE_FIELD,
                id.into(),
            )?,
        })
    }

    /// Returns the safe handle id.
    #[must_use]
    pub fn id(&self) -> &str {
        self.id.as_str()
    }
}

/// Host-observed worker status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkerStatus {
    /// Worker is queued.
    Queued,
    /// Worker is running.
    Running,
    /// Worker completed and may have a report.
    Completed,
    /// Worker was cancelled.
    Cancelled,
    /// Worker failed safely.
    Failed,
}

/// Cancellation outcome from the host worker service.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkerCancelOutcome {
    /// Cancellation request was accepted.
    Accepted,
    /// Worker had already completed.
    AlreadyCompleted,
    /// Worker was already cancelled.
    AlreadyCancelled,
}

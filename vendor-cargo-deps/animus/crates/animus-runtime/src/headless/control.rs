use super::{
    AutomationError, FinalOutput, LEDGER_HASH_FIELD, RunRequest, THREAD_ID_FIELD, Text, TextField,
    ThreadResume, required_text,
};

/// Thread status visible to controllers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ThreadStatus {
    /// Thread exists and has not started execution.
    New,
    /// Thread is running or resumable.
    Running,
    /// Thread completed.
    Completed,
    /// Thread was cancelled.
    Cancelled,
}

/// Controller-visible thread snapshot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ThreadSnapshot {
    thread_id: Text,
    ledger_hash: Text,
    status: ThreadStatus,
}

impl ThreadSnapshot {
    /// Creates a controller-visible thread snapshot.
    ///
    /// # Errors
    ///
    /// Returns [`AutomationError`] when thread identity or ledger hash is empty.
    pub fn new(
        thread_id: &str,
        ledger_hash: &str,
        status: ThreadStatus,
    ) -> Result<Self, AutomationError> {
        Ok(Self {
            thread_id: required_text(TextField {
                field: THREAD_ID_FIELD,
                value: thread_id,
            })?,
            ledger_hash: required_text(TextField {
                field: LEDGER_HASH_FIELD,
                value: ledger_hash,
            })?,
            status,
        })
    }

    /// Returns the thread identity.
    #[must_use]
    pub fn thread_id(&self) -> &str {
        self.thread_id.as_str()
    }

    /// Returns the ledger hash.
    #[must_use]
    pub fn ledger_hash(&self) -> &str {
        self.ledger_hash.as_str()
    }

    /// Returns thread status.
    #[must_use]
    pub const fn status(&self) -> ThreadStatus {
        self.status
    }
}

/// Controller-facing thread operations over typed runtime identities.
pub trait RuntimeThreadController {
    /// Starts a new runtime thread.
    ///
    /// # Errors
    ///
    /// Returns [`AutomationError`] when the request cannot be started.
    fn start(&mut self, request: &RunRequest) -> Result<ThreadSnapshot, AutomationError>;

    /// Resumes an existing runtime thread.
    ///
    /// # Errors
    ///
    /// Returns [`AutomationError`] when thread identity or ledger hash fails
    /// validation before execution continues.
    fn resume(&mut self, resume: &ThreadResume) -> Result<ThreadSnapshot, AutomationError>;

    /// Runs a validated thread until terminal status or host cancellation.
    ///
    /// # Errors
    ///
    /// Returns [`AutomationError`] when runtime execution fails closed.
    fn run(&mut self, thread_id: &str) -> Result<FinalOutput, AutomationError>;

    /// Inspects a runtime thread without executing it.
    ///
    /// # Errors
    ///
    /// Returns [`AutomationError`] when the thread identity is unknown or invalid.
    fn inspect(&self, thread_id: &str) -> Result<ThreadSnapshot, AutomationError>;

    /// Cancels a runtime thread through the host boundary.
    ///
    /// # Errors
    ///
    /// Returns [`AutomationError`] when the thread cannot be cancelled safely.
    fn cancel(&mut self, thread_id: &str) -> Result<ThreadSnapshot, AutomationError>;
}

/// Validates that a resume request matches the current controller snapshot.
///
/// # Errors
///
/// Returns [`AutomationError`] before execution continues when thread identity
/// or ledger hash does not match.
pub fn validate_thread_resume(
    snapshot: &ThreadSnapshot,
    resume: &ThreadResume,
) -> Result<(), AutomationError> {
    if snapshot.thread_id() != resume.thread_id() {
        return Err(AutomationError::ThreadIdentityMismatch {
            expected_thread_id: Text::from(snapshot.thread_id()),
            actual_thread_id: Text::from(resume.thread_id()),
        });
    }
    if snapshot.ledger_hash() != resume.ledger_hash() {
        return Err(AutomationError::ResumeMismatch {
            thread_id: Text::from(snapshot.thread_id()),
            expected_ledger_hash: Text::from(snapshot.ledger_hash()),
            actual_ledger_hash: Text::from(resume.ledger_hash()),
        });
    }
    Ok(())
}

/// Validates that a controller cancellation targets a running thread.
///
/// # Errors
///
/// Returns [`AutomationError`] when the thread identity is empty, mismatched, or
/// no longer running.
pub fn validate_thread_cancellation(
    snapshot: &ThreadSnapshot,
    thread_id: &str,
) -> Result<(), AutomationError> {
    let requested_id = required_text(TextField {
        field: THREAD_ID_FIELD,
        value: thread_id,
    })?;
    if snapshot.thread_id() != requested_id {
        return Err(AutomationError::ThreadIdentityMismatch {
            expected_thread_id: Text::from(snapshot.thread_id()),
            actual_thread_id: requested_id,
        });
    }
    if snapshot.status() != ThreadStatus::Running {
        return Err(AutomationError::ThreadNotRunning {
            thread_id: Text::from(snapshot.thread_id()),
            status: snapshot.status(),
        });
    }
    Ok(())
}

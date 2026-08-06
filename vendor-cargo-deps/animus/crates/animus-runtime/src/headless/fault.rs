use super::Text;
use super::ThreadStatus;

/// Automation contract validation error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AutomationError {
    /// Required field contained no user-visible characters.
    EmptyField {
        /// Field name.
        field: &'static str,
    },
    /// Enum-like field contained an unsupported value.
    UnsupportedValue {
        /// Field name.
        field: &'static str,
        /// Unsupported value.
        value: Text,
    },
    /// A field value was repeated where identities must be unique.
    DuplicateValue {
        /// Field name.
        field: &'static str,
        /// Repeated value.
        value: Text,
    },
    /// Resume ledger identity did not match the expected hash.
    ResumeMismatch {
        /// Thread identity being resumed.
        thread_id: Text,
        /// Expected ledger hash.
        expected_ledger_hash: Text,
        /// Actual ledger hash supplied by the request.
        actual_ledger_hash: Text,
    },
    /// Event sequence was below the accepted range.
    InvalidSequence {
        /// Field name.
        field: &'static str,
    },
    /// Event sequence was not strictly increasing.
    OutOfOrderEvent {
        /// Previous event sequence.
        previous: u64,
        /// Next event sequence.
        next: u64,
    },
    /// Event body exceeded the configured body budget.
    EventBodyTooLarge {
        /// Maximum accepted body byte length.
        max_bytes: usize,
        /// Observed body byte length.
        actual_bytes: usize,
    },
    /// Event stream did not contain a terminal completion item.
    MissingTerminalResponse,
    /// Event stream ended with a failed run.
    RunFailed {
        /// Bounded failure reason.
        reason: Text,
    },
    /// Controller thread identity did not match the request.
    ThreadIdentityMismatch {
        /// Expected thread identity.
        expected_thread_id: Text,
        /// Actual thread identity.
        actual_thread_id: Text,
    },
    /// Controller operation requires a running thread.
    ThreadNotRunning {
        /// Thread identity.
        thread_id: Text,
        /// Current status.
        status: ThreadStatus,
    },
}

impl core::fmt::Display for AutomationError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::EmptyField { field } => write!(formatter, "automation field {field} is empty"),
            Self::UnsupportedValue { field, value } => {
                write!(formatter, "automation field {field} has unsupported value {value}")
            }
            Self::DuplicateValue { field, value } => {
                write!(formatter, "automation field {field} repeats value {value}")
            }
            Self::ResumeMismatch {
                thread_id,
                expected_ledger_hash,
                actual_ledger_hash,
            } => write!(
                formatter,
                "thread {thread_id} ledger hash mismatch: expected {expected_ledger_hash}, got {actual_ledger_hash}"
            ),
            Self::InvalidSequence { field } => {
                write!(formatter, "automation field {field} must be non-zero")
            }
            Self::OutOfOrderEvent { previous, next } => {
                write!(formatter, "automation event sequence {next} must be greater than {previous}")
            }
            Self::EventBodyTooLarge {
                max_bytes,
                actual_bytes,
            } => write!(formatter, "automation event body has {actual_bytes} bytes, exceeding {max_bytes}"),
            Self::MissingTerminalResponse => {
                write!(formatter, "automation stream has no terminal completion")
            }
            Self::RunFailed { reason } => write!(formatter, "automation run failed: {reason}"),
            Self::ThreadIdentityMismatch {
                expected_thread_id,
                actual_thread_id,
            } => write!(formatter, "thread identity mismatch: expected {expected_thread_id}, got {actual_thread_id}"),
            Self::ThreadNotRunning { thread_id, status } => {
                write!(formatter, "thread {thread_id} is not running: {status:?}")
            }
        }
    }
}

impl std::error::Error for AutomationError {}

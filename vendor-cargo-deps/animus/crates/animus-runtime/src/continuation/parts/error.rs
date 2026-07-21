/// Continuation validation or acknowledgement error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContinuationError {
    /// A required field was empty.
    EmptyField {
        /// Field name.
        field: &'static str,
    },
    /// A safe-label field contained unsupported characters or a leak marker.
    InvalidLabel {
        /// Field name.
        field: &'static str,
        /// Rejected value.
        value: Text,
    },
    /// A priority candidate appeared more than once.
    DuplicatePriority {
        /// Duplicated candidate.
        candidate: ContinuationCandidate,
    },
    /// An acknowledgement omitted its work ID.
    MissingWorkId {
        /// Field name.
        field: &'static str,
    },
    /// Dispatch acknowledgement arrived when no dispatch was pending.
    MissingPendingDispatch,
    /// Completion acknowledgement arrived when no work was busy.
    MissingBusyWork,
    /// Acknowledgement work ID did not match pending work.
    MismatchedWork {
        /// Hash of expected work ID.
        expected_work_id_hash: Text,
        /// Hash of acknowledged work ID.
        actual_work_id_hash: Text,
    },
    /// A decision or acknowledgement referenced stale work.
    StaleWork {
        /// Hash of stale work ID.
        work_id_hash: Text,
    },
    /// A completion acknowledgement duplicated an already completed work ID.
    DuplicateCompletion {
        /// Hash of duplicated work ID.
        work_id_hash: Text,
    },
    /// A field had an unsupported value.
    UnsupportedValue {
        /// Field name.
        field: &'static str,
        /// Unsupported value.
        value: Text,
    },
    /// Receipt metadata contained a leak marker.
    UnsafeMetadata {
        /// Field name.
        field: &'static str,
        /// Leak marker class.
        marker: &'static str,
    },
}

impl core::fmt::Display for ContinuationError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::EmptyField { field } => write!(formatter, "continuation field {field} is empty"),
            Self::InvalidLabel { field, value } => {
                write!(
                    formatter,
                    "continuation field {field} has invalid label {value}"
                )
            }
            Self::DuplicatePriority { candidate } => write!(
                formatter,
                "continuation priority duplicates {}",
                candidate.as_str()
            ),
            Self::MissingWorkId { field } => {
                write!(
                    formatter,
                    "continuation acknowledgement field {field} is missing"
                )
            }
            Self::MissingPendingDispatch => {
                write!(
                    formatter,
                    "continuation dispatch acknowledgement has no pending dispatch"
                )
            }
            Self::MissingBusyWork => {
                write!(
                    formatter,
                    "continuation completion acknowledgement has no busy work"
                )
            }
            Self::MismatchedWork { .. } => {
                write!(
                    formatter,
                    "continuation acknowledgement work ID does not match"
                )
            }
            Self::StaleWork { .. } => {
                write!(
                    formatter,
                    "continuation acknowledgement references stale work"
                )
            }
            Self::DuplicateCompletion { .. } => {
                write!(
                    formatter,
                    "continuation completion acknowledgement is duplicated"
                )
            }
            Self::UnsupportedValue { field, value } => {
                write!(
                    formatter,
                    "continuation field {field} has unsupported value {value}"
                )
            }
            Self::UnsafeMetadata { field, marker } => write!(
                formatter,
                "continuation field {field} contains unsafe metadata marker {marker}"
            ),
        }
    }
}

impl std::error::Error for ContinuationError {}


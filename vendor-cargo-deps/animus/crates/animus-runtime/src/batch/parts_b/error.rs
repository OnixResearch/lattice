/// Batch contract validation error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BatchError {
    /// Required field contained no visible text.
    EmptyField {
        /// Field name.
        field: &'static str,
    },
    /// Batch requests must include at least one job.
    EmptyJobs {
        /// Field name.
        field: &'static str,
    },
    /// Location field refers to unsupported remote input or output.
    UnsupportedLocation {
        /// Field name.
        field: &'static str,
        /// Unsupported location value.
        value: Text,
    },
    /// Enum-like field contained an unsupported value.
    UnsupportedValue {
        /// Field name.
        field: &'static str,
        /// Unsupported value.
        value: Text,
    },
    /// Safe label validation failed.
    InvalidLabel {
        /// Field name.
        field: &'static str,
        /// Rejected value.
        value: Text,
    },
    /// Field value was duplicated.
    DuplicateValue {
        /// Field name.
        field: &'static str,
        /// Repeated value.
        value: Text,
    },
    /// Concurrency limit was below the accepted minimum.
    InvalidConcurrency {
        /// Field name.
        field: &'static str,
        /// Supplied concurrency limit.
        limit: u16,
    },
    /// Sequence field was outside the accepted range.
    InvalidSequence {
        /// Field name.
        field: &'static str,
    },
    /// No injected execution service was supplied.
    ExecutionServiceUnavailable {
        /// Safe failure reason.
        reason: &'static str,
    },
    /// Outcome referenced a job outside the plan.
    UnknownJob {
        /// Unknown job identity.
        job_id: Text,
    },
    /// Outcome was missing for a planned job.
    MissingOutcome {
        /// Missing job identity.
        job_id: Text,
    },
    /// Receipt or outcome metadata contained unsafe material.
    UnsafeMetadata {
        /// Field name.
        field: &'static str,
        /// Leak marker class.
        marker: &'static str,
    },
}

impl fmt::Display for BatchError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyField { field } => write!(formatter, "batch field {field} is empty"),
            Self::EmptyJobs { field } => write!(formatter, "batch field {field} has no jobs"),
            Self::UnsupportedLocation { field, value } => {
                write!(
                    formatter,
                    "batch field {field} has unsupported remote location {value}"
                )
            }
            Self::UnsupportedValue { field, value } => {
                write!(
                    formatter,
                    "batch field {field} has unsupported value {value}"
                )
            }
            Self::InvalidLabel { field, value } => {
                write!(
                    formatter,
                    "batch field {field} has invalid safe label {value}"
                )
            }
            Self::DuplicateValue { field, value } => {
                write!(formatter, "batch field {field} repeats value {value}")
            }
            Self::InvalidConcurrency { field, limit } => {
                write!(
                    formatter,
                    "batch field {field} has invalid concurrency {limit}"
                )
            }
            Self::InvalidSequence { field } => {
                write!(formatter, "batch field {field} has invalid sequence")
            }
            Self::ExecutionServiceUnavailable { reason } => {
                write!(formatter, "batch execution service unavailable: {reason}")
            }
            Self::UnknownJob { job_id } => {
                write!(formatter, "batch outcome references unknown job {job_id}")
            }
            Self::MissingOutcome { job_id } => {
                write!(formatter, "batch outcome missing for job {job_id}")
            }
            Self::UnsafeMetadata { field, marker } => {
                write!(
                    formatter,
                    "batch field {field} contains unsafe metadata marker {marker}"
                )
            }
        }
    }
}

impl std::error::Error for BatchError {}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ResumeEntry {
    job_id: Text,
    plan_hash: Text,
    output_format: BatchOutputFormat,
    redaction_policy: BatchRedactionPolicy,
    output_evidence_hash: Text,
    status: ResumeJobStatus,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ResumeJobStatus {
    Completed,
    Failed,
    Partial,
}

impl ResumeJobStatus {
    fn parse(value: &str) -> Result<Self, BatchError> {
        match value {
            RESUME_COMPLETED => Ok(Self::Completed),
            RESUME_FAILED => Ok(Self::Failed),
            RESUME_PARTIAL => Ok(Self::Partial),
            _unsupported => Err(unsupported_value(STATUS_FIELD, value)),
        }
    }
}

#[derive(Clone, Copy)]
struct PlanHashParts<'a> {
    source_location: &'a str,
    output_location: &'a str,
    output_format: BatchOutputFormat,
    concurrency: BatchConcurrencyPolicy,
    execution_mode: BatchExecutionMode,
    redaction_policy: BatchRedactionPolicy,
    jobs: &'a [BatchJob],
}

#[derive(Clone, Copy)]
struct ReceiptHashParts<'a> {
    run_id: &'a str,
    plan_hash: &'a str,
    output_format: BatchOutputFormat,
    redaction_policy: BatchRedactionPolicy,
    jobs: &'a [BatchJobReceipt],
}

#[derive(Clone, Copy)]
struct LeakMarker {
    needle: &'static str,
    marker: &'static str,
}

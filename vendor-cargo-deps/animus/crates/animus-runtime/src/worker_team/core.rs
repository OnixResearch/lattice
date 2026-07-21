//! Pure worker-team validation and merge helpers.
#![allow(module_file_count)]

#[allow(module_file_count)]
mod capability;
mod conflict;
mod context;
mod contract;
mod reportcheck;
mod task;
mod text;

/// Validates a minimized worker context packet.
///
/// # Errors
///
/// Returns [`DelegationFailure`](super::DelegationFailure) when the packet is malformed.
pub fn validate_context_packet(
    packet: &super::WorkerContextPacket,
) -> Result<(), super::DelegationFailure> {
    context::validate(packet)
}

/// Validates a scoped worker task draft.
///
/// # Errors
///
/// Returns [`DelegationFailure`](super::DelegationFailure) when the draft is invalid.
pub fn validate_task(
    draft: super::WorkerTaskDraft,
) -> Result<super::WorkerTask, super::DelegationFailure> {
    task::validate(draft)
}

/// Merges validated worker reports without applying mutation intents.
///
/// # Errors
///
/// Returns [`DelegationFailure`](super::DelegationFailure) when reports are empty, invalid, or conflicting.
pub fn merge_reports(
    task: &super::WorkerTask,
    reports: &[super::Report],
) -> Result<super::ReportMergeOutcome, super::DelegationFailure> {
    debug_assert!(!task.output_schema().required_fields.is_empty());
    debug_assert!(
        reports
            .iter()
            .all(|report| report.fields.len() <= super::MAX_REPORT_FIELDS)
    );
    validate_non_empty_slice(super::WORKER_REPORT_FIELD, reports)?;
    for report in reports {
        validate_report(report, task.output_schema())?;
    }
    Ok(conflict::merge_validated_reports(task, reports))
}

/// Validates a worker report against an expected schema.
///
/// # Errors
///
/// Returns [`DelegationFailure`](super::DelegationFailure) when the report violates the schema.
pub fn validate_report(
    report: &super::Report,
    schema: &super::WorkerOutputSchema,
) -> Result<(), super::DelegationFailure> {
    reportcheck::validate_report_header(report, schema)?;
    reportcheck::validate_report_fields(report, schema)?;
    reportcheck::validate_report_evidence(report.evidence.as_slice())?;
    reportcheck::validate_report_claims(report.claims.as_slice())?;
    reportcheck::validate_report_mutations(report.mutation_intents.as_slice())?;
    Ok(())
}

/// Enqueues a worker task through an injected host worker service.
///
/// # Errors
///
/// Returns [`DelegationFailure`](super::DelegationFailure) when no service is injected or enqueueing fails.
pub fn enqueue_task(
    service: Option<&mut dyn super::WorkerService>,
    task: &super::WorkerTask,
) -> Result<super::WorkerHandle, super::DelegationFailure> {
    injected(service)?.enqueue(task)
}

/// Requests worker status through an injected host worker service.
///
/// # Errors
///
/// Returns [`DelegationFailure`](super::DelegationFailure) when no service is injected or status lookup fails.
pub fn request_status(
    service: Option<&mut dyn super::WorkerService>,
    handle: &super::WorkerHandle,
) -> Result<super::WorkerStatus, super::DelegationFailure> {
    injected(service)?.status(handle)
}

/// Requests worker cancellation through an injected host worker service.
///
/// # Errors
///
/// Returns [`DelegationFailure`](super::DelegationFailure) when no service is injected or cancellation fails.
pub fn request_cancel(
    service: Option<&mut dyn super::WorkerService>,
    handle: &super::WorkerHandle,
) -> Result<super::WorkerCancelOutcome, super::DelegationFailure> {
    injected(service)?.cancel(handle)
}

/// Collects a worker report through an injected host worker service.
///
/// # Errors
///
/// Returns [`DelegationFailure`](super::DelegationFailure) when no service is injected or collection fails.
pub fn collect_report(
    service: Option<&mut dyn super::WorkerService>,
    handle: &super::WorkerHandle,
) -> Result<super::Report, super::DelegationFailure> {
    injected(service)?.collect(handle)
}

fn injected(
    service: Option<&mut dyn super::WorkerService>,
) -> Result<&mut dyn super::WorkerService, super::DelegationFailure> {
    service.ok_or_else(|| {
        super::DelegationFailure::new(
            super::FailureKind::WorkerServiceUnavailable,
            super::WORKER_SERVICE_FIELD,
            super::MISSING_SERVICE_REASON,
        )
    })
}

pub(crate) fn validate_label(
    field: &'static str,
    value: impl AsRef<str>,
) -> Result<String, super::DelegationFailure> {
    text::validate_label(field, value)
}

pub(crate) fn validate_text(
    field: &'static str,
    value: impl AsRef<str>,
) -> Result<String, super::DelegationFailure> {
    text::validate_body(field, value)
}

pub(crate) fn validate_non_empty_slice<T>(
    field: &'static str,
    values: &[T],
) -> Result<(), super::DelegationFailure> {
    text::validate_non_empty_slice(field, values)
}

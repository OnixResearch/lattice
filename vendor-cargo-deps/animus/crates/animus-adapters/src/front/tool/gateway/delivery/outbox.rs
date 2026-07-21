/// Records an adapter-backed delivery attempt in safe outbox form.
///
/// # Errors
///
/// Returns [`DeliveryError`] when the attempt ID or safe adapter handle
/// fails adapter-boundary validation.
///
/// # Panics
///
/// Panics if a request previously normalized by this module no longer preserves
/// its non-empty safe artifact label or validated artifact hash invariants.
// r[impl agent_harness_runtime.artifact_delivery_gateway.adapters]
// r[impl agent_harness_runtime.artifact_delivery_gateway.outbox]
pub fn record_delivery_attempt(
    input: DeliveryExecutionInput<'_>,
) -> Result<DeliveryOutboxRecord, DeliveryError> {
    assert!(
        !input.request.safe_artifact_label().is_empty(),
        "delivery attempt requires a safe artifact label"
    );
    assert!(
        !input.request.artifact_hash().as_str().is_empty(),
        "delivery attempt requires a validated artifact hash"
    );
    let attempt_id = normalize(ATTEMPT_ID_FIELD, input.attempt_id)?;
    if input.policy.status() != DeliveryPolicyStatus::Approved {
        return Ok(failed_record(
            input.request,
            FailedRecordDraft {
                attempt_id,
                adapter_availability: input.adapter_availability,
                adapter_outcome: None,
                error_class: input
                    .policy
                    .failure_class()
                    .unwrap_or(DeliveryFailureClass::InvalidRequest),
                retryability: DeliveryRetryability::NonRetryable,
                platform_handle: None,
            },
        ));
    }
    record_after_policy_approval(input.request, attempt_id, input)
}

fn record_after_policy_approval(
    request: &ArtifactDeliveryRequest,
    attempt_id: alloc::string::String,
    input: DeliveryExecutionInput<'_>,
) -> Result<DeliveryOutboxRecord, DeliveryError> {
    match input.adapter_availability {
        DeliveryAdapterAvailability::Missing => Ok(failed_record(
            request,
            FailedRecordDraft {
                attempt_id,
                adapter_availability: input.adapter_availability,
                adapter_outcome: None,
                error_class: DeliveryFailureClass::MissingAdapter,
                retryability: DeliveryRetryability::NonRetryable,
                platform_handle: None,
            },
        )),
        DeliveryAdapterAvailability::Unavailable => Ok(failed_record(
            request,
            FailedRecordDraft {
                attempt_id,
                adapter_availability: input.adapter_availability,
                adapter_outcome: None,
                error_class: DeliveryFailureClass::AdapterUnavailable,
                retryability: DeliveryRetryability::Retryable,
                platform_handle: None,
            },
        )),
        DeliveryAdapterAvailability::Denied => Ok(failed_record(
            request,
            FailedRecordDraft {
                attempt_id,
                adapter_availability: input.adapter_availability,
                adapter_outcome: None,
                error_class: DeliveryFailureClass::AdapterDenied,
                retryability: DeliveryRetryability::NonRetryable,
                platform_handle: None,
            },
        )),
        DeliveryAdapterAvailability::Available => {
            record_available_adapter(request, attempt_id, input.adapter_outcome)
        }
    }
}

fn record_available_adapter(
    request: &ArtifactDeliveryRequest,
    attempt_id: alloc::string::String,
    outcome: Option<DeliveryAdapterOutcomeDraft<'_>>,
) -> Result<DeliveryOutboxRecord, DeliveryError> {
    assert!(
        !request.safe_artifact_label().is_empty(),
        "available adapter record requires a safe artifact label"
    );
    assert!(
        !attempt_id.is_empty(),
        "available adapter record requires a normalized attempt id"
    );
    let Some(outcome) = outcome else {
        return Ok(failed_record(
            request,
            FailedRecordDraft {
                attempt_id,
                adapter_availability: DeliveryAdapterAvailability::Available,
                adapter_outcome: None,
                error_class: DeliveryFailureClass::MalformedAdapterOutput,
                retryability: DeliveryRetryability::NonRetryable,
                platform_handle: None,
            },
        ));
    };
    let platform_handle = normalize_optional(PLATFORM_HANDLE_FIELD, outcome.platform_handle)?;
    Ok(match outcome.status {
        DeliveryAdapterOutcomeStatus::Succeeded => {
            successful_record(request, attempt_id, outcome, platform_handle)
        }
        DeliveryAdapterOutcomeStatus::RetryableFailure => failed_record(
            request,
            FailedRecordDraft {
                attempt_id,
                adapter_availability: DeliveryAdapterAvailability::Available,
                adapter_outcome: Some(outcome.status),
                error_class: outcome
                    .failure_class
                    .unwrap_or(DeliveryFailureClass::RetryableAdapterFailure),
                retryability: DeliveryRetryability::Retryable,
                platform_handle,
            },
        ),
        DeliveryAdapterOutcomeStatus::NonRetryableFailure => failed_record(
            request,
            FailedRecordDraft {
                attempt_id,
                adapter_availability: DeliveryAdapterAvailability::Available,
                adapter_outcome: Some(outcome.status),
                error_class: outcome
                    .failure_class
                    .unwrap_or(DeliveryFailureClass::NonRetryableAdapterFailure),
                retryability: DeliveryRetryability::NonRetryable,
                platform_handle,
            },
        ),
        DeliveryAdapterOutcomeStatus::MalformedOutput => failed_record(
            request,
            FailedRecordDraft {
                attempt_id,
                adapter_availability: DeliveryAdapterAvailability::Available,
                adapter_outcome: Some(outcome.status),
                error_class: DeliveryFailureClass::MalformedAdapterOutput,
                retryability: DeliveryRetryability::NonRetryable,
                platform_handle,
            },
        ),
    })
}

fn successful_record(
    request: &ArtifactDeliveryRequest,
    attempt_id: alloc::string::String,
    outcome: DeliveryAdapterOutcomeDraft<'_>,
    platform_handle: Option<alloc::string::String>,
) -> DeliveryOutboxRecord {
    DeliveryOutboxRecord {
        attempt_id,
        status: DeliveryAttemptStatus::Succeeded,
        retryability: DeliveryRetryability::NonRetryable,
        target_kind: request.target_kind(),
        artifact_kind: request.artifact_kind(),
        safe_artifact_label: alloc::string::String::from(request.safe_artifact_label()),
        artifact_hash: request.artifact_hash().clone(),
        adapter_availability: DeliveryAdapterAvailability::Available,
        adapter_outcome: Some(outcome.status),
        platform_handle,
        error_class: None,
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct FailedRecordDraft {
    attempt_id: alloc::string::String,
    adapter_availability: DeliveryAdapterAvailability,
    adapter_outcome: Option<DeliveryAdapterOutcomeStatus>,
    error_class: DeliveryFailureClass,
    retryability: DeliveryRetryability,
    platform_handle: Option<alloc::string::String>,
}

fn failed_record(
    request: &ArtifactDeliveryRequest,
    draft: FailedRecordDraft,
) -> DeliveryOutboxRecord {
    DeliveryOutboxRecord {
        attempt_id: draft.attempt_id,
        status: DeliveryAttemptStatus::Failed,
        retryability: draft.retryability,
        target_kind: request.target_kind(),
        artifact_kind: request.artifact_kind(),
        safe_artifact_label: alloc::string::String::from(request.safe_artifact_label()),
        artifact_hash: request.artifact_hash().clone(),
        adapter_availability: draft.adapter_availability,
        adapter_outcome: draft.adapter_outcome,
        platform_handle: draft.platform_handle,
        error_class: Some(draft.error_class),
    }
}


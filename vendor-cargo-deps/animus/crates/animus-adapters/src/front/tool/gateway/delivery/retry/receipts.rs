/// Retry plan status for persisted delivery attempts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeliveryRetryPlanStatus {
    /// Retry may execute through the adapter shell.
    Execute,
    /// Retry is refused before adapter execution.
    Refused,
}

/// Borrowed retry planning input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DeliveryRetryInput<'a> {
    /// Persisted safe attempt record.
    pub attempt: &'a DeliveryOutboxRecord,
    /// Current policy decision after revalidation.
    pub current_policy: &'a PolicyDecision,
    /// Current adapter availability.
    pub adapter_availability: DeliveryAdapterAvailability,
}

/// Safe retry decision for a persisted attempt ID.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeliveryRetryPlan {
    status: DeliveryRetryPlanStatus,
    attempt_id: alloc::string::String,
    failure_class: Option<DeliveryFailureClass>,
}

impl DeliveryRetryPlan {
    /// Returns the retry plan status.
    #[must_use]
    pub const fn status(&self) -> DeliveryRetryPlanStatus {
        self.status
    }

    /// Returns the referenced attempt ID.
    #[must_use]
    pub fn attempt_id(&self) -> &str {
        self.attempt_id.as_str()
    }

    /// Returns the refusal failure class when retry is refused.
    #[must_use]
    pub const fn failure_class(&self) -> Option<DeliveryFailureClass> {
        self.failure_class
    }
}

/// Plans retry for a persisted attempt ID after current policy revalidation.
///
/// # Panics
///
/// Panics if a persisted outbox record no longer preserves its non-empty attempt
/// ID or safe artifact label invariants.
// r[impl agent_harness_runtime.artifact_delivery_gateway.outbox]
#[must_use]
pub fn plan_delivery_retry(input: DeliveryRetryInput<'_>) -> DeliveryRetryPlan {
    assert!(
        !input.attempt.attempt_id().is_empty(),
        "retry planning requires a persisted attempt id"
    );
    assert!(
        !input.attempt.safe_artifact_label().is_empty(),
        "retry planning requires safe artifact metadata"
    );
    if input.attempt.retryability() == DeliveryRetryability::NonRetryable {
        return refused_retry(
            input.attempt,
            input
                .attempt
                .error_class()
                .unwrap_or(DeliveryFailureClass::NotRetryable),
        );
    }
    if input.current_policy.status() != DeliveryPolicyStatus::Approved {
        return refused_retry(
            input.attempt,
            input
                .current_policy
                .failure_class()
                .unwrap_or(DeliveryFailureClass::CurrentPolicyDenied),
        );
    }
    match input.adapter_availability {
        DeliveryAdapterAvailability::Available => DeliveryRetryPlan {
            status: DeliveryRetryPlanStatus::Execute,
            attempt_id: alloc::string::String::from(input.attempt.attempt_id()),
            failure_class: None,
        },
        DeliveryAdapterAvailability::Missing => {
            refused_retry(input.attempt, DeliveryFailureClass::MissingAdapter)
        }
        DeliveryAdapterAvailability::Unavailable => {
            refused_retry(input.attempt, DeliveryFailureClass::AdapterUnavailable)
        }
        DeliveryAdapterAvailability::Denied => {
            refused_retry(input.attempt, DeliveryFailureClass::AdapterDenied)
        }
    }
}

fn refused_retry(
    attempt: &DeliveryOutboxRecord,
    failure_class: DeliveryFailureClass,
) -> DeliveryRetryPlan {
    DeliveryRetryPlan {
        status: DeliveryRetryPlanStatus::Refused,
        attempt_id: alloc::string::String::from(attempt.attempt_id()),
        failure_class: Some(failure_class),
    }
}

/// Delivery receipt leak class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeliveryReceiptLeak {
    /// Raw destination URL was present.
    RawDestination,
    /// Raw token or credential was present.
    Token,
    /// Raw header was present.
    Header,
    /// Full host path was present.
    FullPath,
    /// Raw artifact bytes were present.
    ArtifactBytes,
    /// Raw platform payload was present.
    PlatformPayload,
}

/// Borrowed delivery receipt before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DeliveryReceiptDraft<'a> {
    /// Safe outbox attempt record.
    pub attempt: &'a DeliveryOutboxRecord,
    /// Forbidden raw destination URL.
    pub raw_destination_url: Option<&'a str>,
    /// Forbidden token or credential.
    pub token: Option<&'a str>,
    /// Forbidden header value.
    pub header: Option<&'a str>,
    /// Forbidden full host path.
    pub full_path: Option<&'a str>,
    /// Forbidden raw artifact bytes.
    pub artifact_bytes: Option<&'a [u8]>,
    /// Forbidden raw platform payload.
    pub platform_payload: Option<&'a str>,
}

/// Safe delivery receipt with deterministic BLAKE3 hash.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeliveryReceipt {
    attempt: DeliveryOutboxRecord,
    receipt_hash: crate::artifact::Hash,
}

impl DeliveryReceipt {
    /// Validates a delivery receipt and computes its deterministic BLAKE3 hash.
    ///
    /// # Errors
    ///
    /// Returns [`DeliveryError`] when forbidden receipt fields are present.
    // r[impl agent_harness_runtime.artifact_delivery_gateway.receipts]
    pub fn validate(draft: &DeliveryReceiptDraft<'_>) -> Result<Self, DeliveryError> {
        reject_delivery_receipt_leaks(draft)?;
        Ok(Self {
            attempt: draft.attempt.clone(),
            receipt_hash: delivery_receipt_hash(draft.attempt),
        })
    }

    /// Returns the safe outbox attempt record.
    #[must_use]
    pub const fn attempt(&self) -> &DeliveryOutboxRecord {
        &self.attempt
    }

    /// Returns the receipt hash.
    #[must_use]
    pub const fn receipt_hash(&self) -> &crate::artifact::Hash {
        &self.receipt_hash
    }
}

fn reject_delivery_receipt_leaks(
    draft: &DeliveryReceiptDraft<'_>,
) -> Result<(), DeliveryError> {
    assert!(
        !draft.attempt.attempt_id().is_empty(),
        "receipt validation requires a safe attempt id"
    );
    assert!(
        !draft.attempt.safe_artifact_label().is_empty(),
        "receipt validation requires a safe artifact label"
    );
    if draft.raw_destination_url.is_some() {
        return Err(receipt_leak(DeliveryReceiptLeak::RawDestination));
    }
    if draft.token.is_some() {
        return Err(receipt_leak(DeliveryReceiptLeak::Token));
    }
    if draft.header.is_some() {
        return Err(receipt_leak(DeliveryReceiptLeak::Header));
    }
    if draft.full_path.is_some() {
        return Err(receipt_leak(DeliveryReceiptLeak::FullPath));
    }
    if draft.artifact_bytes.is_some() {
        return Err(receipt_leak(DeliveryReceiptLeak::ArtifactBytes));
    }
    if draft.platform_payload.is_some() {
        return Err(receipt_leak(DeliveryReceiptLeak::PlatformPayload));
    }
    Ok(())
}

const fn receipt_leak(leak: DeliveryReceiptLeak) -> DeliveryError {
    DeliveryError::ReceiptLeak { leak }
}

fn delivery_receipt_hash(attempt: &DeliveryOutboxRecord) -> crate::artifact::Hash {
    assert!(
        !attempt.attempt_id().is_empty(),
        "receipt hash requires a safe attempt id"
    );
    assert!(
        !attempt.artifact_hash().as_str().is_empty(),
        "receipt hash requires a validated artifact hash"
    );
    let mut canonical = alloc::string::String::from(RECEIPT_CANONICAL_VERSION);
    canonical.push_str(CANONICAL_SEPARATOR);
    canonical.push_str(attempt.attempt_id());
    canonical.push_str(CANONICAL_SEPARATOR);
    canonical.push_str(attempt.status().as_str());
    canonical.push_str(CANONICAL_SEPARATOR);
    canonical.push_str(attempt.retryability().as_str());
    canonical.push_str(CANONICAL_SEPARATOR);
    canonical.push_str(attempt.target_kind().as_str());
    canonical.push_str(CANONICAL_SEPARATOR);
    canonical.push_str(attempt.artifact_kind().as_str());
    canonical.push_str(CANONICAL_SEPARATOR);
    canonical.push_str(attempt.safe_artifact_label());
    canonical.push_str(CANONICAL_SEPARATOR);
    canonical.push_str(attempt.artifact_hash().as_str());
    canonical.push_str(CANONICAL_SEPARATOR);
    canonical.push_str(attempt.adapter_availability().as_str());
    canonical.push_str(CANONICAL_SEPARATOR);
    if let Some(outcome) = attempt.adapter_outcome() {
        canonical.push_str(outcome.as_str());
    }
    canonical.push_str(CANONICAL_SEPARATOR);
    if let Some(platform_handle) = attempt.platform_handle() {
        canonical.push_str(platform_handle);
    }
    canonical.push_str(CANONICAL_SEPARATOR);
    if let Some(failure) = attempt.error_class() {
        canonical.push_str(failure.as_str());
    }
    let digest = blake3::hash(canonical.as_bytes());
    crate::artifact::Hash::from_bytes(digest.as_bytes())
}


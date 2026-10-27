use alloc::string::{String, ToString};

use blake3::Hasher;

use crate::{ChangeRequestBinding, Error, validate_binding};

const RECEIPT_HASH_DOMAIN: &str = "animus.change-request-adapter.receipt.v1";

// r[impl agent_harness_runtime.remora_adapters.evidence]

/// Receipt operation class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReceiptOperation {
    /// Change-request metadata fetch.
    Fetch,
    /// Advisory review analysis.
    Analyze,
    /// Change-request publication observation.
    PublishChangeRequest,
    /// Review publication observation.
    PublishReview,
}

/// Receipt status class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReceiptStatus {
    /// Operation completed with matching bounded evidence.
    Completed,
    /// Operation was blocked before effect dispatch.
    Blocked,
    /// Operation failed in the host shell.
    Failed,
}

/// Receipt content-disclosure posture.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContentDisclosure {
    /// Receipt contains metadata only.
    MetadataOnly,
    /// Receipt requests raw or private content.
    RawContent,
}

/// Receipt claim class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReceiptClaim {
    /// Bounded contract validation only.
    BoundedContract,
    /// Provider delivery guarantee.
    ProviderDelivery,
    /// Review-quality guarantee.
    ReviewQuality,
    /// Merge-safety guarantee.
    MergeSafety,
    /// Production-readiness claim.
    ProductionReadiness,
    /// Release-eligibility claim.
    ReleaseEligibility,
}

/// Draft for one metadata-only adapter receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdapterReceiptDraft {
    /// Exact request binding.
    pub request_binding: ChangeRequestBinding,
    /// Exact observed binding.
    pub observation_binding: ChangeRequestBinding,
    /// Operation class.
    pub operation: ReceiptOperation,
    /// Status class.
    pub status: ReceiptStatus,
    /// Bounded item count.
    pub item_count: u32,
    /// BLAKE3 identity of the request projection.
    pub request_identity: String,
    /// BLAKE3 identity of the observation projection.
    pub observation_identity: String,
    /// Content-disclosure posture.
    pub disclosure: ContentDisclosure,
    /// Requested claim class.
    pub claim: ReceiptClaim,
}

/// Validated metadata-only adapter receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdapterReceipt {
    /// Operation class.
    pub operation: ReceiptOperation,
    /// Status class.
    pub status: ReceiptStatus,
    /// Bounded item count.
    pub item_count: u32,
    /// BLAKE3 identity of the request projection.
    pub request_identity: String,
    /// BLAKE3 identity of the observation projection.
    pub observation_identity: String,
    /// BLAKE3 identity of the complete safe receipt.
    pub receipt_identity: String,
}

/// Builds one validated metadata-only receipt.
///
/// # Errors
///
/// Returns a typed error when bindings, identities, disclosure, or claims are invalid.
pub fn build_receipt(draft: &AdapterReceiptDraft) -> Result<AdapterReceipt, Error> {
    validate_binding(&draft.request_binding, &draft.observation_binding)?;
    crate::validate_blake3("receipt.request_identity", &draft.request_identity)?;
    crate::validate_blake3("receipt.observation_identity", &draft.observation_identity)?;
    if draft.disclosure != ContentDisclosure::MetadataOnly {
        return Err(Error::UnsafeReceipt);
    }
    if draft.claim != ReceiptClaim::BoundedContract {
        return Err(Error::UnsupportedClaim);
    }

    let mut hasher = Hasher::new();
    frame(&mut hasher, RECEIPT_HASH_DOMAIN);
    frame(&mut hasher, operation_label(draft.operation));
    frame(&mut hasher, status_label(draft.status));
    frame(&mut hasher, &draft.item_count.to_string());
    frame(&mut hasher, &draft.request_identity);
    frame(&mut hasher, &draft.observation_identity);
    frame(&mut hasher, draft.request_binding.repository_identity());
    frame(&mut hasher, draft.request_binding.policy_identity());
    frame(&mut hasher, draft.request_binding.plan_identity());
    frame(&mut hasher, &draft.request_binding.generation().to_string());
    frame(&mut hasher, draft.request_binding.implementation_identity());
    frame(&mut hasher, draft.request_binding.correlation_identity());
    let receipt_identity = alloc::format!("b3:{}", hasher.finalize().to_hex());

    Ok(AdapterReceipt {
        operation: draft.operation,
        status: draft.status,
        item_count: draft.item_count,
        request_identity: draft.request_identity.clone(),
        observation_identity: draft.observation_identity.clone(),
        receipt_identity,
    })
}

fn frame(hasher: &mut Hasher, value: &str) {
    let length = u64::try_from(value.len()).unwrap_or(u64::MAX);
    hasher.update(&length.to_le_bytes());
    hasher.update(value.as_bytes());
}

const fn operation_label(operation: ReceiptOperation) -> &'static str {
    match operation {
        ReceiptOperation::Fetch => "fetch",
        ReceiptOperation::Analyze => "analyze",
        ReceiptOperation::PublishChangeRequest => "publish-change-request",
        ReceiptOperation::PublishReview => "publish-review",
    }
}

const fn status_label(status: ReceiptStatus) -> &'static str {
    match status {
        ReceiptStatus::Completed => "completed",
        ReceiptStatus::Blocked => "blocked",
        ReceiptStatus::Failed => "failed",
    }
}

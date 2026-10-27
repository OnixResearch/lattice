use alloc::string::{String, ToString};

use blake3::Hasher;

use crate::{Error, MaintenanceBinding, validate_binding};

const RECEIPT_HASH_DOMAIN: &str = "animus.maintenance-adapter.receipt.v1";

// r[impl agent_harness_runtime.remora_adapters.evidence]

/// Maintenance receipt operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReceiptOperation {
    /// Dependency inventory observation.
    Inventory,
    /// Candidate discovery observation.
    CandidateDiscovery,
    /// Advisory observation.
    AdvisoryObservation,
    /// Pure update planning.
    Plan,
    /// Workspace update observation.
    WorkspaceUpdate,
    /// Validation observation.
    Validation,
}

/// Maintenance receipt status.
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

/// Maintenance receipt claim class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReceiptClaim {
    /// Bounded contract validation only.
    BoundedContract,
    /// Registry-freshness guarantee.
    RegistryFreshness,
    /// Advisory-completeness guarantee.
    AdvisoryCompleteness,
    /// Update-correctness guarantee.
    UpdateCorrectness,
    /// Validation-sufficiency guarantee.
    ValidationSufficiency,
    /// Production-readiness claim.
    ProductionReadiness,
    /// Release-eligibility claim.
    ReleaseEligibility,
}

/// Draft for one metadata-only maintenance receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MaintenanceReceiptDraft {
    /// Exact request binding.
    pub request_binding: MaintenanceBinding,
    /// Exact observed binding.
    pub observation_binding: MaintenanceBinding,
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

/// Validated metadata-only maintenance receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MaintenanceReceipt {
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

/// Builds one validated metadata-only maintenance receipt.
///
/// # Errors
///
/// Returns a typed error when bindings, identities, disclosure, or claims are invalid.
pub fn build_receipt(draft: &MaintenanceReceiptDraft) -> Result<MaintenanceReceipt, Error> {
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

    Ok(MaintenanceReceipt {
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
        ReceiptOperation::Inventory => "inventory",
        ReceiptOperation::CandidateDiscovery => "candidate-discovery",
        ReceiptOperation::AdvisoryObservation => "advisory-observation",
        ReceiptOperation::Plan => "plan",
        ReceiptOperation::WorkspaceUpdate => "workspace-update",
        ReceiptOperation::Validation => "validation",
    }
}

const fn status_label(status: ReceiptStatus) -> &'static str {
    match status {
        ReceiptStatus::Completed => "completed",
        ReceiptStatus::Blocked => "blocked",
        ReceiptStatus::Failed => "failed",
    }
}

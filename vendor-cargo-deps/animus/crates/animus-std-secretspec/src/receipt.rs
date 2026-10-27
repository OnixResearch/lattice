//! Metadata-only credential-resolution receipts.

use animus_adapters::secret_resolution::{
    BLAKE3_HEX_CHARS, SecretDestinationClass, SecretProviderScheme, SecretResolutionBounds,
    SecretResolutionRequest,
};

/// Stable host-resolution outcome category.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResolutionOutcome {
    /// One value crossed the ephemeral delivery boundary.
    Delivered,
    /// Pure policy rejected the request.
    PolicyRejected,
    /// Manifest access or identity validation failed.
    ManifestRejected,
    /// The worker executable was unavailable.
    WorkerUnavailable,
    /// The selected provider executable was unavailable.
    ProviderExecutableMissing,
    /// Provider execution returned a nonzero result.
    ProviderProcessFailed,
    /// Provider output was malformed.
    ProviderOutputMalformed,
    /// The owned worker exceeded its deadline.
    TimedOut,
    /// Explicit cancellation stopped the owned worker.
    Cancelled,
    /// A bounded output stream overflowed.
    OutputLimitExceeded,
    /// The private worker protocol was malformed.
    WorkerProtocolRejected,
    /// Process setup, capture, or teardown failed.
    WorkerInfrastructureFailed,
}

/// Admitted execution capability recorded by a receipt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResolutionCapability {
    /// Owned worker with explicit time, I/O, cancellation, and teardown bounds.
    BoundedWorker,
}

/// Mandatory sensitive-material omission class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvidenceOmission {
    /// Values, credentials, child output, diagnostics, and private paths are omitted.
    SensitiveMaterial,
}

/// Public metadata-only receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolutionReceipt {
    /// BLAKE3 identity over public receipt fields only.
    pub receipt_identity: String,
    /// BLAKE3 identity of the admitted request metadata.
    pub request_identity: String,
    /// Exact resolver implementation identity.
    pub resolver_identity: String,
    /// Exact public manifest identity.
    pub manifest_identity: String,
    /// Exact public policy identity.
    pub policy_identity: String,
    /// Exact public profile.
    pub profile: String,
    /// Exact public scope.
    pub scope: String,
    /// Exact public declaration.
    pub declaration: String,
    /// Selected provider family.
    pub provider: SecretProviderScheme,
    /// Safe public purpose.
    pub purpose: String,
    /// Admitted destination class.
    pub destination: SecretDestinationClass,
    /// Exact positive execution bounds.
    pub bounds: SecretResolutionBounds,
    /// Admitted execution capability.
    pub capability: ResolutionCapability,
    /// Number of values delivered. This is never greater than one.
    pub delivered_value_count: usize,
    /// Final bounded outcome.
    pub outcome: ResolutionOutcome,
    /// Mandatory sensitive-material omission class.
    pub evidence_omission: EvidenceOmission,
    /// Explicit non-claim labels.
    pub non_claims: [&'static str; 7],
}

impl ResolutionReceipt {
    pub(crate) fn new(
        request_identity: String,
        request: &SecretResolutionRequest,
        outcome: ResolutionOutcome,
    ) -> Self {
        let payload = format!(
            "{request_identity}|{}|{}|{}|{}|{outcome:?}|omitted",
            request.profile,
            request.scope,
            request.declaration,
            request.provider.as_str()
        );
        Self {
            receipt_identity: blake3::hash(payload.as_bytes()).to_hex().to_string(),
            request_identity,
            resolver_identity: request.resolver_identity.clone(),
            manifest_identity: request.manifest_identity.clone(),
            policy_identity: request.policy_identity.clone(),
            profile: request.profile.clone(),
            scope: request.scope.clone(),
            declaration: request.declaration.clone(),
            provider: request.provider,
            purpose: request.purpose.clone(),
            destination: request.destination,
            bounds: request.bounds,
            capability: ResolutionCapability::BoundedWorker,
            delivered_value_count: usize::from(outcome == ResolutionOutcome::Delivered),
            outcome,
            evidence_omission: EvidenceOmission::SensitiveMaterial,
            non_claims: [
                "caller-authority",
                "credential-correctness",
                "credential-freshness",
                "provider-integrity",
                "audit-completeness",
                "production-readiness",
                "release-eligibility",
            ],
        }
    }

    /// Validate mandatory omission and non-claim markers.
    #[must_use]
    pub fn is_safe(&self) -> bool {
        self.evidence_omission == EvidenceOmission::SensitiveMaterial
            && self.request_identity.len() == BLAKE3_HEX_CHARS
            && self.receipt_identity.len() == BLAKE3_HEX_CHARS
            && self.manifest_identity.len() == BLAKE3_HEX_CHARS
            && self.policy_identity.len() == BLAKE3_HEX_CHARS
            && self.capability == ResolutionCapability::BoundedWorker
            && self.delivered_value_count <= 1
            && self.non_claims.contains(&"caller-authority")
            && self.non_claims.contains(&"release-eligibility")
    }
}

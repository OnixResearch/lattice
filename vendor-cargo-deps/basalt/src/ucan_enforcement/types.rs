use serde::Deserialize;
use serde::Serialize;
use ucan::CaveatPolicySet;
use ucan::HolderInvocationLimits;
use ucan::HolderInvocationNonce;
use ucan::HolderInvocationReplayAdmission;
use ucan::KeyResolver;
use ucan::OperationProfileId;
use ucan::ProofStore;
use ucan::RevocationChecker;
use ucan::SignerSignature;
use ucan::VerificationLimits;
use ucan::VerificationTime;

use super::core::operation_profile_digest;
use crate::EnforcementRequest;

const DOCUMENT_READ_CONTRACT: &str = "document-read";
const DOCUMENT_READ_ABILITY: &str = "document/read";
const DOCUMENT_WRITE_CONTRACT: &str = "document-write";
const DOCUMENT_WRITE_ABILITY: &str = "document/write";

const TARGET_CANDIDATE_ACTIVATION_CONTRACT: &str = "onixos-target-candidate-activation";
const TARGET_CANDIDATE_ACTIVATION_ABILITY: &str = "onix/target-candidate-activation";
const TARGET_GENERATION_PERSISTENCE_CONTRACT: &str = "onixos-target-generation-persistence";
const TARGET_GENERATION_PERSISTENCE_ABILITY: &str = "onix/target-generation-persistence";
const TARGET_ROLLBACK_CONTRACT: &str = "onixos-target-rollback";
const TARGET_ROLLBACK_ABILITY: &str = "onix/target-rollback";
const STATE_SYNC_PUBLISH_CONTRACT: &str = "onixos-state-sync-publish";
const STATE_SYNC_PUBLISH_ABILITY: &str = "state_sync.publish";
const STATE_SYNC_DELETE_CONTRACT: &str = "onixos-state-sync-delete";
const STATE_SYNC_DELETE_ABILITY: &str = "state_sync.delete";
const STATE_SYNC_ADMIN_CONTRACT: &str = "onixos-state-sync-admin";
const STATE_SYNC_ADMIN_ABILITY: &str = "state_sync.admin";
const ONIXOS_TARGET_RESOURCE_PREFIX: &str = "did:key:z6Mkonix/onixos/";
const STATE_SYNC_RESOURCE_PREFIX: &str = "onix://state-sync/";

/// Closed downstream actions with reviewed Basalt contract and UCAN ability mappings.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum EnforcementAction {
    /// Read one document resource.
    DocumentRead,
    /// Write one document resource.
    DocumentWrite,
    /// Candidate activation for one `OnixOS` target.
    OnixosTargetCandidateActivation,
    /// Generation persistence for one `OnixOS` target.
    OnixosTargetGenerationPersistence,
    /// Rollback for one `OnixOS` target.
    OnixosTargetRollback,
    /// State-sync publish for one `OnixOS` target.
    OnixosStateSyncPublish,
    /// State-sync delete for one `OnixOS` target.
    OnixosStateSyncDelete,
    /// State-sync administration for one `OnixOS` target.
    OnixosStateSyncAdmin,
}

impl EnforcementAction {
    /// Return the stable action identifier.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DocumentRead => "document-read",
            Self::DocumentWrite => "document-write",
            Self::OnixosTargetCandidateActivation => "onixos-target-candidate-activation",
            Self::OnixosTargetGenerationPersistence => "onixos-target-generation-persistence",
            Self::OnixosTargetRollback => "onixos-target-rollback",
            Self::OnixosStateSyncPublish => "onixos-state-sync-publish",
            Self::OnixosStateSyncDelete => "onixos-state-sync-delete",
            Self::OnixosStateSyncAdmin => "onixos-state-sync-admin",
        }
    }

    /// Return the reviewed Basalt contract identifier.
    #[must_use]
    pub const fn contract_id(self) -> &'static str {
        match self {
            Self::DocumentRead => DOCUMENT_READ_CONTRACT,
            Self::DocumentWrite => DOCUMENT_WRITE_CONTRACT,
            Self::OnixosTargetCandidateActivation => TARGET_CANDIDATE_ACTIVATION_CONTRACT,
            Self::OnixosTargetGenerationPersistence => TARGET_GENERATION_PERSISTENCE_CONTRACT,
            Self::OnixosTargetRollback => TARGET_ROLLBACK_CONTRACT,
            Self::OnixosStateSyncPublish => STATE_SYNC_PUBLISH_CONTRACT,
            Self::OnixosStateSyncDelete => STATE_SYNC_DELETE_CONTRACT,
            Self::OnixosStateSyncAdmin => STATE_SYNC_ADMIN_CONTRACT,
        }
    }

    /// Return the reviewed UCAN ability.
    #[must_use]
    pub const fn ability(self) -> &'static str {
        match self {
            Self::DocumentRead => DOCUMENT_READ_ABILITY,
            Self::DocumentWrite => DOCUMENT_WRITE_ABILITY,
            Self::OnixosTargetCandidateActivation => TARGET_CANDIDATE_ACTIVATION_ABILITY,
            Self::OnixosTargetGenerationPersistence => TARGET_GENERATION_PERSISTENCE_ABILITY,
            Self::OnixosTargetRollback => TARGET_ROLLBACK_ABILITY,
            Self::OnixosStateSyncPublish => STATE_SYNC_PUBLISH_ABILITY,
            Self::OnixosStateSyncDelete => STATE_SYNC_DELETE_ABILITY,
            Self::OnixosStateSyncAdmin => STATE_SYNC_ADMIN_ABILITY,
        }
    }

    /// Return the reviewed resource prefix for this action.
    #[must_use]
    pub const fn resource_prefix(self) -> &'static str {
        match self {
            Self::DocumentRead | Self::DocumentWrite => "",
            Self::OnixosTargetCandidateActivation
            | Self::OnixosTargetGenerationPersistence
            | Self::OnixosTargetRollback => ONIXOS_TARGET_RESOURCE_PREFIX,
            Self::OnixosStateSyncPublish | Self::OnixosStateSyncDelete | Self::OnixosStateSyncAdmin => {
                STATE_SYNC_RESOURCE_PREFIX
            }
        }
    }

    /// Return the fixed BLAKE3 operation-profile identity for this mapping.
    #[must_use]
    pub fn operation_profile_id(self) -> OperationProfileId {
        OperationProfileId::from_bytes(operation_profile_digest(self))
    }
}

/// Borrowed raw holder invocation bound to one existing Basalt request.
pub struct UcanEnforcementRequest<'request, 'credential> {
    pub(super) enforcement: &'request EnforcementRequest,
    pub(super) compact_token: &'credential str,
    pub(super) action: EnforcementAction,
    pub(super) nonce: HolderInvocationNonce,
    pub(super) holder_signature: &'credential SignerSignature,
}

impl<'request, 'credential> UcanEnforcementRequest<'request, 'credential> {
    /// Construct one borrowed raw UCAN enforcement request.
    #[must_use]
    pub const fn new(
        enforcement: &'request EnforcementRequest,
        compact_token: &'credential str,
        action: EnforcementAction,
        nonce: HolderInvocationNonce,
        holder_signature: &'credential SignerSignature,
    ) -> Self {
        Self {
            enforcement,
            compact_token,
            action,
            nonce,
            holder_signature,
        }
    }

    /// Return the exact borrowed Basalt request.
    #[must_use]
    pub const fn enforcement(&self) -> &EnforcementRequest {
        self.enforcement
    }

    /// Return the selected typed downstream action.
    #[must_use]
    pub const fn action(&self) -> EnforcementAction {
        self.action
    }
}

/// Explicit caller-owned services and limits for UCAN verification.
pub struct UcanEnforcementBoundaries<'a> {
    /// Explicit verification time.
    pub verification_time: VerificationTime,
    /// Caller-owned issuer and holder key resolver.
    pub key_resolver: &'a dyn KeyResolver,
    /// Caller-owned proof store.
    pub proof_store: &'a dyn ProofStore,
    /// Caller-owned revocation checker.
    pub revocations: &'a dyn RevocationChecker,
    /// Non-consuming caveat policy observations.
    pub caveat_policies: &'a dyn CaveatPolicySet,
    /// Final state-consuming holder replay admission.
    pub replay: &'a dyn HolderInvocationReplayAdmission,
    /// Compact-token and proof limits.
    pub verification_limits: VerificationLimits,
    /// Holder canonical request limits.
    pub holder_limits: HolderInvocationLimits,
}

/// Stable typed adapter denial reason.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum UcanEnforcementReason {
    OversizedCredential,
    MalformedCredential,
    SignatureDenied,
    KeyResolutionDenied,
    ProofDenied,
    TimeDenied,
    CaveatDenied,
    ScopeDenied,
    Revoked,
    HolderDenied,
    ReplayDenied,
    LimitExceeded,
    OperationMismatch,
    UnverifiedAuthorityFields,
    PolicyDenied,
    BackendUnavailable,
    InternalAuthorityDenied,
    Allowed,
}

impl UcanEnforcementReason {
    /// Return the stable serialized reason identifier.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::OversizedCredential => "oversized-credential",
            Self::MalformedCredential => "malformed-credential",
            Self::SignatureDenied => "signature-denied",
            Self::KeyResolutionDenied => "key-resolution-denied",
            Self::ProofDenied => "proof-denied",
            Self::TimeDenied => "time-denied",
            Self::CaveatDenied => "caveat-denied",
            Self::ScopeDenied => "scope-denied",
            Self::Revoked => "revoked",
            Self::HolderDenied => "holder-denied",
            Self::ReplayDenied => "replay-denied",
            Self::LimitExceeded => "limit-exceeded",
            Self::OperationMismatch => "operation-mismatch",
            Self::UnverifiedAuthorityFields => "unverified-authority-fields",
            Self::PolicyDenied => "policy-denied",
            Self::BackendUnavailable => "backend-unavailable",
            Self::InternalAuthorityDenied => "internal-authority-denied",
            Self::Allowed => "allowed",
        }
    }
}

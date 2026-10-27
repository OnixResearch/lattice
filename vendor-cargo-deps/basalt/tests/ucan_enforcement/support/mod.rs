#![allow(clippy::redundant_pub_crate)]

pub(crate) mod evaluate;
pub(crate) mod fixtures;
pub(crate) mod services;

pub(crate) use basalt::authority_core::default_authority_policy_ref;
#[cfg(feature = "durable-authority")]
pub(crate) use basalt::durable_authority::{
    BasaltDurableAuthority, DurableAuthorityVerificationInputs,
};
pub(crate) use basalt::ucan::core::holder_invocation::HOLDER_INVOCATION_DIGEST_BYTES;
pub(crate) use basalt::ucan::token::DenyUnknownCaveats;
pub(crate) use basalt::ucan::{
    AudienceDid, CapabilityDocument, CapabilitySet, CaveatDecision, CaveatDocument,
    CaveatIdentifier, CaveatPolicy, CaveatPolicySet, CompactToken, ED25519_SECRET_KEY_BYTES,
    Ed25519InMemorySigner, Ed25519PublicKey, HolderInvocationLimits, HolderInvocationNonce,
    HolderInvocationReplayAdmission, HolderInvocationReplayCandidate, HolderInvocationReplayError,
    HolderInvocationSigningRequest, IssuerDid, KeyResolutionContext, KeyResolver, KeyResolverError,
    NoRevocations, OperationBinding, OperationProfileId, ProofCollection, ProofReferences,
    ProofStore, RevocationChecker, RevocationList, SignerSignature, TokenSigner, TokenTimeBounds,
    VerificationLimits, VerificationTime, issue_token_with_signer, proof_reference,
    sign_holder_invocation,
};
pub(crate) use basalt::ucan_enforcement::{
    EnforcementAction, LiveUcanEnforcement, UcanEnforcementBoundaries, UcanEnforcementFailure,
    UcanEnforcementReason, UcanEnforcementReceipt, UcanEnforcementRequest, bind_enforcement_action,
    verify_ucan_enforcement_request,
};
pub(crate) use basalt::ucan_profile::BasaltUcanProtocolProfile;
pub(crate) use basalt::{CapabilityGrant, EnforcementRequest, default_policy};
pub(crate) use evaluate::*;
pub(crate) use fixtures::*;
pub(crate) use services::*;

pub(crate) const ISSUER_SEED_BYTE: u8 = 11;
pub(crate) const DELEGATE_SEED_BYTE: u8 = 13;
pub(crate) const HOLDER_SEED_BYTE: u8 = 17;
pub(crate) const WRONG_ISSUER_SEED_BYTE: u8 = 31;
pub(crate) const NONCE_BYTE: u8 = 19;
pub(crate) const OTHER_NONCE_BYTE: u8 = 23;
pub(crate) const TOKEN_NOT_BEFORE_SECONDS: u64 = 1_000;
pub(crate) const TOKEN_EXPIRES_SECONDS: u64 = 2_000;
pub(crate) const VERIFY_AT_SECONDS: u64 = 1_500;
pub(crate) const EXPIRED_VERIFY_AT_SECONDS: u64 = 2_500;
pub(crate) const PARENT_RESOURCE: &str = "did:key:z6Mkresource";
pub(crate) const DOCUMENT_RESOURCE: &str = "did:key:z6Mkresource/docs/report";
pub(crate) const PRIVATE_RESOURCE: &str = "did:key:z6Mkresource/private/report";
pub(crate) const OTHER_RESOURCE: &str = "did:key:z6Mkresource/videos/report";
pub(crate) const SUBSTITUTED_REQUEST_RESOURCE: &str = "did:key:z6Mkresource/docs/other";
pub(crate) const DOCUMENT_WRITE_ABILITY: &str = "document/write";
pub(crate) const DOCUMENT_READ_ABILITY: &str = "document/read";
pub(crate) const CAVEAT_DOMAIN: &str = "example";
pub(crate) const CAVEAT_TYPE: &str = "approval";
pub(crate) const CAVEAT_KEY: &str = "required";
pub(crate) const CAVEAT_PAYLOAD: &[u8] = b"yes";
pub(crate) const BACKEND_ERROR_MESSAGE: &str = "injected resolver failure";
pub(crate) const EXPECTED_PREFLIGHT_RESOLVER_CALLS: u32 = 1;
pub(crate) const EXPECTED_CAVEAT_OBSERVATIONS: u32 = 2;
pub(crate) const FIRST_CALL: u32 = 1;
pub(crate) const OVERSIZED_TOKEN_LIMIT: usize = 64;
pub(crate) const OVERSIZED_TOKEN_BYTES: usize = OVERSIZED_TOKEN_LIMIT + 1;
pub(crate) const OVERSIZED_REQUEST_BYTES: usize = 4_097;
pub(crate) const SHORT_CANONICAL_RESOURCE_LIMIT: usize = 8;
pub(crate) const UNKNOWN_PROFILE_BYTE: u8 = 29;

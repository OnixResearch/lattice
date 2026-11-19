//! Pure serving-readiness, request-admission, and invocation-receipt contracts.
//!
//! Hosts supply bounded availability snapshots, serving-owner admission
//! replies, and current serving observations. This module screens named model
//! routes before any preference or rotation, gates one dispatch against the
//! serving owner's request-scoped grant and a current recheck, and projects a
//! metadata-only invocation receipt. It reads no clock, network, credential,
//! or health probe; every time value is a host-supplied logical tick.

mod basis;
mod eligibility;
mod facts;
mod ledger;
mod policy;
mod predispatch;
mod version;

#[cfg(test)]
mod tests;

pub use basis::{
    Attestation, CallerBinding, CallerDraft, ContentRef, ContractIssue, DeploymentBinding,
    ModelBinding, ReplicaFact, RouteBinding, Token,
};
pub use eligibility::{
    CapacityStanding, EligibleRoute, Exclusion, ExclusionReason, Freshness, RouteCandidate,
    RouteCandidateDraft, ScreenRequest, Screening, SnapshotStatus, rank_eligible, screen_routes,
};
pub use facts::{
    AdmissionOffer, AvailabilitySnapshot, CapabilitySet, CapacityDraft, CapacityKind,
    CapacityObservation, CapacityOrigin, CapacityScope, CapacityUnit, Counter, Envelope, Locality,
    ModelDraft, Readiness, ServiceDraft, ServiceObservation, SnapshotDraft, capability_label,
};
pub use ledger::{
    AdmissionRecord, DispatchOutcome, EvidenceLabel, InvocationReceipt, NotDispatched,
    PolicyGenerationProvenance, ReceiptInput, ResponseFacts, ResponseProvenance, ResponseRejection,
    UncertainCause, Usage, UsageOrigin, project_receipt, response_model_label, response_provenance,
};
pub use policy::{CapacityRule, IdentityRule, Profile, ProfileDraft, ReplicaRule};
pub use predispatch::{
    AdmissionQuery, CurrentReading, CurrentState, DispatchPermit, GateBlock, GateInput,
    GateRejection, GateVerdict, Grant, GrantDraft, OwnerRejection, OwnerReply, admission_query,
    gate_dispatch,
};
pub use version::{
    ProbabilitySemantics, RolloutDraft, RolloutFacts, RolloutField, RolloutOperation,
    RolloutRequirement, RolloutRequirementDraft,
};

/// Schema of a host-supplied availability snapshot.
pub const SNAPSHOT_SCHEMA: &str = "animus.serving-availability-snapshot.v1";
/// Schema of the capacity-aware selection profile.
pub const PROFILE_SCHEMA: &str = "animus.serving-capacity-profile.v1";
/// Schema of the metadata-only per-invocation receipt.
pub const RECEIPT_SCHEMA: &str = "animus.serving-invocation-receipt.v1";
/// Domain of the per-snapshot BLAKE3 identity.
pub const SNAPSHOT_DOMAIN: &str = "animus-serving-snapshot-v1";
/// Domain of the profile BLAKE3 identity.
pub const PROFILE_DOMAIN: &str = "animus-serving-profile-v1";
/// Domain of the receipt BLAKE3 identity.
pub const RECEIPT_DOMAIN: &str = "animus-serving-receipt-v1";
/// Maximum services in one snapshot.
pub const MAX_SERVICES: u32 = 64;
/// Maximum capacity observations for one service.
pub const MAX_CAPACITY_OBSERVATIONS: u32 = 16;
/// Maximum named route candidates in one screening request.
pub const MAX_ROUTE_CANDIDATES: u32 = 32;
/// Maximum rollout operations named by facts or a requirement.
pub const MAX_ROLLOUT_OPERATIONS: u32 = 8;
/// Maximum bytes in one opaque token.
pub const MAX_TOKEN_BYTES: u32 = 256;
/// Receipt non-claims, in fixed order.
pub const RECEIPT_NON_CLAIMS: [&str; 7] = [
    "not-gpu-scheduling",
    "not-production-availability",
    "not-physical-placement-truth",
    "not-model-correctness",
    "not-capacity-reservation",
    "not-provider-honesty",
    "not-cost-or-pricing",
];

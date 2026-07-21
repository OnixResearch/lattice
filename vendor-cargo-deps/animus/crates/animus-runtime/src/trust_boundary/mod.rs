//! Pure Ornith-inspired trust-boundary monitor contracts.
//!
//! The outer trust boundary is immutable input data. Hosts own filesystem,
//! sandbox, verifier, provider, approval, and judge execution. This module only
//! validates already-loaded descriptors and evaluates planned/observed action
//! descriptors deterministically.

// Cairn adaptation compatibility modules intentionally keep public DTO names and table-driven validation compact while drained changes are archived.
#![allow(
    unknown_lints,
    ambiguous_params,
    assertion_density,
    bool_naming,
    borrowed_argument_types
)]
#![allow(
    compound_condition,
    excessive_file_length,
    function_length,
    non_trait_imports
)]
#![allow(
    numeric_units,
    path_segment_repetition,
    raw_arithmetic_overflow,
    unbounded_collection_growth
)]
mod core;
mod data;

pub use core::{
    bind_trust_boundary_judge, blake3_trust_boundary_fingerprint, monitor_trust_boundary_action,
    validate_trust_boundary_descriptor, validate_trust_boundary_receipt,
};
pub use data::{
    TrustBoundaryAction, TrustBoundaryActionKind, TrustBoundaryDescriptor, TrustBoundaryDiagnostic,
    TrustBoundaryDiagnosticKind, TrustBoundaryEvidencePolicy, TrustBoundaryFingerprint,
    TrustBoundaryIssueCode, TrustBoundaryJudgeBinding, TrustBoundaryJudgeOutcome,
    TrustBoundaryJudgeVeto, TrustBoundaryMonitorDecision, TrustBoundaryReceiptLeak,
    TrustBoundaryRedactionStatus, TrustBoundaryValidationResult, TrustBoundaryVerifierArtifact,
    TrustBoundaryViolationReceipt,
};

pub(crate) const BLAKE3_ALGORITHM: &str = "BLAKE3";
pub(crate) const TRUST_BOUNDARY_HASH_DOMAIN: &str = "animus.trust-boundary.v1";
pub(crate) const HASH_PART_SEPARATOR: &str = "|";
pub(crate) const FIELD_ACTION_ID: &str = "action.action_id";
pub(crate) const FIELD_ACTION_KIND: &str = "action.kind";
pub(crate) const FIELD_APPROVAL: &str = "action.approval";
pub(crate) const FIELD_BOUNDARY_ID: &str = "boundary.boundary_id";
pub(crate) const FIELD_EVIDENCE: &str = "boundary.evidence_policy";
pub(crate) const FIELD_MUTABLE_REQUEST: &str = "boundary.mutable_boundary_request";
pub(crate) const FIELD_POLICY_HASH: &str = "boundary.policy_hash";
pub(crate) const FIELD_RECEIPT_LEAK: &str = "receipt.leaks";
pub(crate) const FIELD_RECEIPT_POLICY_HASH: &str = "receipt.policy_hash";
pub(crate) const FIELD_SANCTIONED_TOOLS: &str = "boundary.sanctioned_tools";
pub(crate) const FIELD_SANDBOX: &str = "boundary.sandbox_profile_id";
pub(crate) const FIELD_VERIFIER_HASH: &str = "boundary.verifier.hash";
pub(crate) const FIELD_VERIFIER_LABEL: &str = "boundary.verifier.label";
pub(crate) const FIELD_WITHHELD_LABELS: &str = "boundary.withheld_path_labels";

#[cfg(test)]
mod tests;

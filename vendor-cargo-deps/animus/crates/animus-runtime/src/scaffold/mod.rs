//! Pure Ornith-inspired scaffold profile contracts.
//!
//! Scaffold profiles are reviewed strategy data. Hosts own persistence, prompt
//! assembly, model dispatch, Steel inputs, admission UI, and any future training
//! or evaluation loops. This module validates already-loaded descriptors and
//! computes deterministic selection, admission, and receipt decisions without
//! ambient authority.

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
    apply_scaffold_admission, blake3_scaffold_fingerprint, select_scaffold_profile,
    validate_scaffold_influence_receipt, validate_scaffold_profile,
};
pub use data::{
    ScaffoldAdmissionDecision, ScaffoldAdmissionEffect, ScaffoldAdmissionOutcome,
    ScaffoldAdmissionRequest, ScaffoldDiagnostic, ScaffoldDiagnosticKind, ScaffoldFingerprint,
    ScaffoldInfluenceReceipt, ScaffoldInfluenceValidationOutcome, ScaffoldProfile,
    ScaffoldProfileProvenance, ScaffoldReceiptSensitiveEvidence, ScaffoldReviewStatus,
    ScaffoldRevision, ScaffoldRevisionProposal, ScaffoldSelectionOutcome, ScaffoldSelectionPlan,
    ScaffoldSelectionRequest, ScaffoldStrategyKnob, ScaffoldValidationResult,
};

pub(crate) const BLAKE3_ALGORITHM: &str = "BLAKE3";
pub(crate) const SCAFFOLD_HASH_DOMAIN: &str = "animus.scaffold-profile.v1";
pub(crate) const HASH_PART_SEPARATOR: &str = "|";
pub(crate) const DEFAULT_FALLBACK_TASK_CLASS: &str = "general";
pub(crate) const FIELD_ACTIVE_REVISION: &str = "profile.active_revision_id";
pub(crate) const FIELD_BASE_REVISION: &str = "admission.base_revision_id";
pub(crate) const FIELD_CONTENT_HASH: &str = "revision.content_hash";
pub(crate) const FIELD_PROFILE_ENABLED: &str = "profile.enabled";
pub(crate) const FIELD_PROFILE_ID: &str = "profile.profile_id";
pub(crate) const FIELD_PROFILE_PROVENANCE: &str = "profile.provenance.source";
pub(crate) const FIELD_RECEIPT_HASH: &str = "receipt.revision_hash";
pub(crate) const FIELD_RECEIPT_PROFILE: &str = "receipt.profile_id";
pub(crate) const FIELD_RECEIPT_ROUTE: &str = "receipt.route_labels";
pub(crate) const FIELD_RECEIPT_SENSITIVE: &str = "receipt.sensitive_evidence";
pub(crate) const FIELD_RECEIPT_TASK: &str = "receipt.task_class";
pub(crate) const FIELD_REVISION_HASH_MATCH: &str = "revision.hash_matches_content";
pub(crate) const FIELD_REVISION_ID: &str = "revision.revision_id";
pub(crate) const FIELD_REVISION_STATUS: &str = "revision.review_status";
pub(crate) const FIELD_ROUTE_CONSTRAINTS: &str = "profile.route_constraints";
pub(crate) const FIELD_SELECTION_PROFILE: &str = "selection.profile";
pub(crate) const FIELD_STRATEGY_KNOBS: &str = "revision.strategy_knobs";
pub(crate) const FIELD_TASK_CLASS: &str = "profile.task_classes";

#[cfg(test)]
mod tests;

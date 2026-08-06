//! Pure offline scaffold-evaluation feedback contracts.
//!
//! This module models local, reviewed evaluation feedback data. Hosts own batch
//! execution, providers, verifiers, filesystem/process/network access, review UI,
//! and scaffold admission. Runtime core only validates DTOs and classifies safe
//! evidence freshness.

// Cairn adaptation compatibility modules intentionally keep public DTO names and table-driven validation compact while
// drained changes are archived.
#![allow(
    unknown_lints,
    ambiguous_params,
    assertion_density,
    bool_naming,
    borrowed_argument_types
)]
#![allow(compound_condition, excessive_file_length, function_length, non_trait_imports)]
#![allow(
    numeric_units,
    path_segment_repetition,
    raw_arithmetic_overflow,
    unbounded_collection_growth
)]
mod core;
mod data;

pub use core::blake3_eval_feedback_fingerprint;
pub use core::classify_eval_feedback_freshness;
pub use core::evaluate_feedback_review;
pub use core::validate_eval_feedback_plan;
pub use core::validate_eval_feedback_receipt;

pub use data::EvalFeedbackCandidate;
pub use data::EvalFeedbackCandidateKind;
pub use data::EvalFeedbackDiagnostic;
pub use data::EvalFeedbackDiagnosticKind;
pub use data::EvalFeedbackFingerprint;
pub use data::EvalFeedbackFreshnessDecision;
pub use data::EvalFeedbackFreshnessPolicy;
pub use data::EvalFeedbackOutputFormat;
pub use data::EvalFeedbackPlan;
pub use data::EvalFeedbackReceipt;
pub use data::EvalFeedbackReceiptLeak;
pub use data::EvalFeedbackRedactionState;
pub use data::EvalFeedbackReviewDecision;
pub use data::EvalFeedbackReviewOutcome;
pub use data::EvalFeedbackRunStatus;
pub use data::EvalFeedbackScopeRequest;
pub use data::EvalFeedbackTaskSet;
pub use data::EvalFeedbackTaskSetKind;
pub use data::EvalFeedbackValidationResult;
pub use data::EvalFeedbackVerifierIdentity;

pub(crate) const BLAKE3_ALGORITHM: &str = "BLAKE3";
pub(crate) const EVAL_FEEDBACK_HASH_DOMAIN: &str = "animus.eval-feedback.v1";
pub(crate) const HASH_PART_SEPARATOR: &str = "|";
pub(crate) const FIELD_CANDIDATE: &str = "feedback.candidate";
pub(crate) const FIELD_OUTPUT_FORMAT: &str = "plan.output_format";
pub(crate) const FIELD_PLAN_SCOPE: &str = "plan.unsupported_scope_requests";
pub(crate) const FIELD_RECEIPT_LEAK: &str = "receipt.leaks";
pub(crate) const FIELD_RECEIPT_REDATION: &str = "receipt.redaction_state";
pub(crate) const FIELD_RESOURCE_LABELS: &str = "plan.resource_labels";
pub(crate) const FIELD_RUN_COUNT: &str = "plan.run_count";
pub(crate) const FIELD_SAMPLING: &str = "sampling_profile";
pub(crate) const FIELD_SCAFFOLD_HASH: &str = "scaffold.hash";
pub(crate) const FIELD_TASK_SET: &str = "plan.task_set";
pub(crate) const FIELD_TOOL_SURFACE_HASH: &str = "tool_surface.hash";
pub(crate) const FIELD_TRUST_BOUNDARY_HASH: &str = "trust_boundary.hash";
pub(crate) const FIELD_VERIFIER_HASH: &str = "verifier.hash";

#[cfg(test)]
mod tests;

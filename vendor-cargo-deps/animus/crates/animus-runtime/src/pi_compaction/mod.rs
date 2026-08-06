//! Pure Pi-style compaction and branch-summary contracts.
//!
//! Runtime core prepares deterministic plans and validates safe summary metadata.
//! Hosts own token estimation, persistence, providers, clocks, files, UI, and
//! concrete summarizer execution.

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

pub use core::blake3_pi_compaction_fingerprint;
pub use core::evaluate_pi_compaction_trigger;
pub use core::evaluate_required_pi_summarizer;
pub use core::prepare_pi_branch_handoff_summary;
pub use core::prepare_pi_compaction;
pub use core::validate_pi_summary_entry;

pub use data::PiBranchHandoffPlan;
pub use data::PiBranchHandoffRequest;
pub use data::PiBranchSummaryMode;
pub use data::PiCompactionDecision;
pub use data::PiCompactionDiagnostic;
pub use data::PiCompactionDiagnosticKind;
pub use data::PiCompactionFingerprint;
pub use data::PiCompactionPlan;
pub use data::PiCompactionPolicy;
pub use data::PiCompactionRedactionBehavior;
pub use data::PiCompactionReplayEntry;
pub use data::PiCompactionReplayEntryKind;
pub use data::PiCompactionSummarizerRequest;
pub use data::PiCompactionSummarizerSelection;
pub use data::PiCompactionSummarizerState;
pub use data::PiCompactionSummaryEntry;
pub use data::PiCompactionSummaryFormat;
pub use data::PiCompactionToolBinding;
pub use data::PiCompactionTrigger;
pub use data::PiCompactionValidationResult;

pub(crate) const BLAKE3_ALGORITHM: &str = "BLAKE3";
pub(crate) const EMPTY_COUNT: usize = 0;
pub(crate) const HASH_PART_SEPARATOR: &str = "|";
pub(crate) const PI_COMPACTION_HASH_DOMAIN: &str = "animus.pi-compaction.v1";
pub(crate) const REQUIRED_COUNT: u32 = 1;
pub(crate) const REQUIRED_USIZE: usize = 1;
pub(crate) const FIELD_BRANCH: &str = "branch";
pub(crate) const FIELD_CONTEXT_ESTIMATE: &str = "context_estimate";
pub(crate) const FIELD_CUT_POINT: &str = "cut_point";
pub(crate) const FIELD_POLICY: &str = "policy";
pub(crate) const FIELD_RESOURCE_ID: &str = "summary.resource_id";
pub(crate) const FIELD_SUMMARIZER: &str = "summarizer";
pub(crate) const FIELD_SUMMARY_TEXT: &str = "summary.text";
pub(crate) const FIELD_TOOL_BINDING: &str = "tool_binding";

#[cfg(test)]
mod tests;

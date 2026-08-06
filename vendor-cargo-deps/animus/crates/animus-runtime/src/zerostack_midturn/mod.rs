//! Pure Zerostack-style mid-turn compaction contracts.
//!
//! Runtime core classifies prompt pressure, prepares explicit stop/compact/resume
//! plans, validates summaries, and bounds tool-output artifacts. Hosts own
//! provider calls, token estimation, summarizer execution, ledger persistence,
//! files, clocks, tools, and continuation side effects.

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

pub use core::artifactize_zerostack_tool_output;
pub use core::blake3_zerostack_midturn_fingerprint;
pub use core::calculate_zerostack_prompt_pressure;
pub use core::plan_zerostack_midturn_compaction;
pub use core::validate_zerostack_midturn_summary;

pub use data::ZerostackCachedTokenRule;
pub use data::ZerostackInjectedServiceState;
pub use data::ZerostackMidturnDecision;
pub use data::ZerostackMidturnDiagnostic;
pub use data::ZerostackMidturnDiagnosticKind;
pub use data::ZerostackMidturnFingerprint;
pub use data::ZerostackMidturnPlan;
pub use data::ZerostackMidturnPolicy;
pub use data::ZerostackMidturnServices;
pub use data::ZerostackMidturnSummary;
pub use data::ZerostackMidturnSummaryKind;
pub use data::ZerostackMidturnValidationResult;
pub use data::ZerostackPromptPressure;
pub use data::ZerostackProviderUsage;
pub use data::ZerostackRouteMetadata;
pub use data::ZerostackToolArtifactDescriptor;
pub use data::ZerostackToolArtifactPolicy;

pub(crate) const BLAKE3_ALGORITHM: &str = "BLAKE3";
pub(crate) const EMPTY_COUNT: usize = 0;
pub(crate) const FIELD_LEDGER: &str = "ledger";
pub(crate) const FIELD_POLICY: &str = "policy";
pub(crate) const FIELD_PROMPT_PRESSURE: &str = "prompt_pressure";
pub(crate) const FIELD_SUMMARIZER: &str = "summarizer";
pub(crate) const FIELD_SUMMARY: &str = "summary";
pub(crate) const FIELD_TOOL_ARTIFACT: &str = "tool_artifact";
pub(crate) const HASH_PART_SEPARATOR: &str = "|";
pub(crate) const ZERO_BASIS_POINTS: u32 = 0;
pub(crate) const FULL_BASIS_POINTS: u32 = 10_000;
pub(crate) const ZEROSTACK_MIDTURN_HASH_DOMAIN: &str = "animus.zerostack-midturn.v1";

#[cfg(test)]
mod tests;

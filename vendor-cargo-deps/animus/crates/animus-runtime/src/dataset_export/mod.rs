//! Pure Animus-to-Hugging Face dataset export contracts.

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
mod jsonl;

pub use core::blake3_dataset_export_fingerprint;
pub use core::export_chat_messages_row;
pub use core::export_preference_row;
pub use core::export_rl_trajectory_row;
pub use core::inspect_dataset_exports;

pub use data::DatasetChatMessage;
pub use data::DatasetChatRow;
pub use data::DatasetDiagnostic;
pub use data::DatasetDiagnosticKind;
pub use data::DatasetExportPolicy;
pub use data::DatasetExportResult;
pub use data::DatasetFingerprint;
pub use data::DatasetInspectionReport;
pub use data::DatasetMetadataProjection;
pub use data::DatasetPreferenceExample;
pub use data::DatasetPreferenceRow;
pub use data::DatasetReplayEntry;
pub use data::DatasetReplayRole;
pub use data::DatasetRlStep;
pub use data::DatasetRlStepKind;
pub use data::DatasetRlTrajectory;
pub use data::DatasetRlTrajectoryRow;
pub use data::DatasetSessionSource;
pub use data::DatasetSkippedRow;
pub use data::DatasetTarget;
pub use jsonl::render_chat_row_jsonl;
pub use jsonl::render_chat_rows_jsonl;

pub(crate) const BLAKE3_ALGORITHM: &str = "BLAKE3";
pub(crate) const BLAKE3_HEX_LENGTH: usize = 64;
pub(crate) const DATASET_HASH_DOMAIN: &str = "animus.dataset-export.v1";
pub(crate) const FIELD_MESSAGE: &str = "message";
pub(crate) const FIELD_POLICY: &str = "policy";
pub(crate) const FIELD_REPLAY: &str = "replay";
pub(crate) const FIELD_ROW: &str = "row";
pub(crate) const FIELD_ROW_ID: &str = "row_id";
pub(crate) const FIELD_SOURCE_SPAN: &str = "source_span";
pub(crate) const HASH_PART_SEPARATOR: &str = "|";
pub(crate) const REQUIRED_COUNT: usize = 1;
pub(crate) const ROLE_ASSISTANT: &str = "assistant";
pub(crate) const ROLE_SYSTEM: &str = "system";
pub(crate) const ROLE_TOOL: &str = "tool";
pub(crate) const ROLE_USER: &str = "user";

#[cfg(test)]
mod tests;

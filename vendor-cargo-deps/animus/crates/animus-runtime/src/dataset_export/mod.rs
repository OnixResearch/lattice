//! Pure Animus-to-Hugging Face dataset export contracts.

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
mod jsonl;

pub use core::{
    blake3_dataset_export_fingerprint, export_chat_messages_row, export_preference_row,
    export_rl_trajectory_row, inspect_dataset_exports,
};
pub use data::{
    DatasetChatMessage, DatasetChatRow, DatasetDiagnostic, DatasetDiagnosticKind,
    DatasetExportPolicy, DatasetExportResult, DatasetFingerprint, DatasetInspectionReport,
    DatasetMetadataProjection, DatasetPreferenceExample, DatasetPreferenceRow, DatasetReplayEntry,
    DatasetReplayRole, DatasetRlStep, DatasetRlStepKind, DatasetRlTrajectory,
    DatasetRlTrajectoryRow, DatasetSessionSource, DatasetSkippedRow, DatasetTarget,
};
pub use jsonl::{render_chat_row_jsonl, render_chat_rows_jsonl};

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

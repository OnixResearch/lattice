//! BLAKE3-backed tagged-span edit contracts.

mod core;
mod fault;
mod hash;
mod record;
mod state;
mod types;

pub use core::{RequestDraft, SpanDraft, validate_plan};
pub use fault::Error;
pub use record::{Receipt, ReceiptDraft, SpanMetadata};
pub use state::{Snapshot, SnapshotDraft, TaggedLine, TaggedLineDraft};
pub use types::{NewlinePolicy, Plan, ResourceRedaction, TextNormalization, Validated};

const SNAPSHOT_SCHEMA: &str = "animus.hashed-edit.snapshot.v1";
const PLAN_SCHEMA: &str = "animus.hashed-edit.plan.v1";
const RECEIPT_SCHEMA: &str = "animus.hashed-edit.receipt.v1";
const NORMALIZATION_LF: &str = "lf";
const NORMALIZATION_CRLF: &str = "crlf";
const NORMALIZATION_MIXED_LF: &str = "mixed_normalized_lf";
const NEWLINE_POLICY_PRESERVE: &str = "preserve_snapshot";
const NEWLINE_POLICY_NORMALIZE_LF: &str = "normalize_lf";
const RESOURCE_REDACTION_OMITTED: &str = "omitted";
const RESOURCE_REDACTION_RELATIVE_PATH: &str = "relative_path";
const RESOURCE_REDACTION_RESOURCE_HASH: &str = "resource_hash";
const OMITTED_RESOURCE_IDENTITY: &str = "omitted";
const HASH_LENGTH_REASON: &str = "hash must use the configured BLAKE3 hex length";
const HASH_ALPHABET_REASON: &str = "hash must contain lowercase hex only";
const TAGGED_LINES_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("hashed_edit_tagged_lines");
const EDITS_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("hashed_edit_edits");
const DIAGNOSTICS_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("hashed_edit_diagnostics");
const RESOURCE_CLASS_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("hashed_edit_resource_class");
const RESOURCE_IDENTITY_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("hashed_edit_resource_identity");
const SNAPSHOT_HASH_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("hashed_edit_snapshot_hash");
const CURRENT_SNAPSHOT_HASH_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("hashed_edit_current_snapshot_hash");
const AFTER_SNAPSHOT_HASH_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("hashed_edit_after_snapshot_hash");
const NORMALIZATION_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("hashed_edit_normalization");
const NEWLINE_POLICY_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("hashed_edit_newline_policy");
const TAG_GENERATION_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("hashed_edit_tag_generation");
const LINE_TAG_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("hashed_edit_line_tag");
const LINE_HASH_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("hashed_edit_line_hash");
const REPLACEMENT_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("hashed_edit_replacement");
const RECEIPT_DIAGNOSTIC_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("hashed_edit_receipt_diagnostic");
const MAX_LINES: usize = 512;
const MAX_OPERATIONS: usize = super::MAX_TOOL_METADATA_ITEMS;
const MAX_RECEIPT_DIAGNOSTICS: usize = super::MAX_TOOL_METADATA_ITEMS;
const MAX_REPLACEMENT_CHARS: usize = crate::validation::MAX_ADAPTER_FIELD_CHARS;

const _: () = assert!(MAX_LINES >= MAX_OPERATIONS);
const _: () = assert!(MAX_REPLACEMENT_CHARS > 0);

type Text = super::Text;
type Collection<T> = super::Collection<T>;

#[derive(Clone, Copy)]
struct CountLimit {
    field: crate::validation::FieldName,
    actual_items: usize,
    max_items: usize,
}

fn required_label(field: crate::validation::FieldName, value: &str) -> Result<Text, Error> {
    Ok(crate::validation::normalized_adapter_field(field, value)?)
}

fn validate_hash(field: crate::validation::FieldName, value: &str) -> Result<Text, Error> {
    let normalized = required_label(field, value)?;
    let actual_chars = normalized.chars().count();
    if actual_chars != crate::BLAKE3_HASH_HEX_CHARS {
        return Err(Error::InvalidHash {
            field: field.as_str(),
            reason: HASH_LENGTH_REASON,
        });
    }
    if !has_lowercase_hex_only(normalized.as_str()) {
        return Err(Error::InvalidHash {
            field: field.as_str(),
            reason: HASH_ALPHABET_REASON,
        });
    }
    Ok(normalized)
}

fn has_lowercase_hex_only(value: &str) -> bool {
    value
        .chars()
        .all(|character| character.is_ascii_hexdigit() && !character.is_ascii_uppercase())
}

fn validate_collection_count(limit: CountLimit) -> Result<(), Error> {
    if limit.actual_items > limit.max_items {
        return Err(Error::TooManyItems {
            field: limit.field.as_str(),
            max_items: limit.max_items,
            actual_items: limit.actual_items,
        });
    }
    Ok(())
}

fn validate_non_empty_collection(
    field: crate::validation::FieldName,
    actual_items: usize,
) -> Result<(), Error> {
    if actual_items == 0 {
        return Err(crate::AdapterContractError::EmptyCollection {
            field: field.as_str(),
        }
        .into());
    }
    Ok(())
}

fn usize_to_u64(value: usize) -> u64 {
    super::usize_to_u64(value)
}

/// Host boundary that applies an already validated hashed edit plan.
pub trait Host {
    /// Host-specific apply error.
    type Error;

    /// Applies a validated hashed edit plan and returns a safe receipt.
    ///
    /// # Errors
    ///
    /// Returns a host-specific error when the host cannot read, lock, write,
    /// atomically replace, permission-check, or receipt the target resource.
    fn apply_validated(&mut self, plan: &Plan) -> Result<Receipt, Self::Error>;
}

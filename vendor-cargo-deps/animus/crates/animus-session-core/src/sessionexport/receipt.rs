//! Metadata-only redacted session-tree export receipts.

#[allow(unused_imports)]
use crate::prelude::*;

type SessionTreeExportError = super::data::SessionTreeExportError;
type SessionTreeExportIssue = super::data::SessionTreeExportIssue;
type SessionTreeExportReceipt = super::data::SessionTreeExportReceipt;
type SessionTreeExportReceiptInput<'a> = super::data::SessionTreeExportReceiptInput<'a>;
type SessionTreeExportSensitiveDraft<'a> = super::data::SessionTreeExportSensitiveDraft<'a>;
type WriteProof<'a> = super::data::SessionTreeExportWriteEvidence<'a>;

const NON_CLAIM: &str = super::data::NON_CLAIM;
const REDACTION_MARKER: &str = super::data::REDACTION_MARKER;
const PRIVATE_FILE_MODE: u32 = super::data::PRIVATE_FILE_MODE;
const REQUIRED_WRITE_STATUS: &str = "committed";
const SESSION_TREE_EXPORT_RECEIPT_HASH_DOMAIN: &str =
    super::data::SESSION_TREE_EXPORT_RECEIPT_HASH_DOMAIN;
const SESSION_TREE_EXPORT_RECEIPT_SCHEMA: &str = super::data::SESSION_TREE_EXPORT_RECEIPT_SCHEMA;
const FIELD_RECEIPT: &str = "session_tree_export_receipt";
const RECEIPT_FIELD: super::core::ExportField = super::core::ExportField(FIELD_RECEIPT);

struct ReceiptHashInput<'a> {
    receipt: &'a SessionTreeExportReceiptInput<'a>,
    entry_count_text: &'a str,
    active_path_count_text: &'a str,
    branch_point_count_text: &'a str,
    omission_count_text: &'a str,
    artifact_bytes_text: &'a str,
}

/// Projects a metadata-only export receipt.
///
/// # Errors
///
/// Returns [`SessionTreeExportError`] when artifact/projection/write binding
/// differs or any private input field is present.
// r[impl agent_harness_runtime.session_tree_export.receipts]
pub fn project_session_tree_export_receipt(
    input: &SessionTreeExportReceiptInput<'_>,
) -> Result<SessionTreeExportReceipt, SessionTreeExportError> {
    reject_sensitive(input.sensitive)?;
    validate_host_write(input.write, input)?;
    let entry_count = u32::try_from(input.projection.entries.len()).map_err(|_error| {
        SessionTreeExportError::new(SessionTreeExportIssue::BoundExceeded, FIELD_RECEIPT)
    })?;
    let entry_count_text = entry_count.to_string();
    let active_path_count_text = input.projection.active_path_count.to_string();
    let branch_point_count_text = input.projection.branch_point_count.to_string();
    let omission_count_text = input.projection.omission_count.to_string();
    let artifact_bytes = input.artifact.byte_count;
    let artifact_bytes_text = artifact_bytes.to_string();
    debug_assert_eq!(
        input.artifact.projection_hash,
        input.projection.projection_hash
    );
    debug_assert!(artifact_bytes > 0);
    let receipt_hash = receipt_hash(&ReceiptHashInput {
        receipt: input,
        entry_count_text: entry_count_text.as_str(),
        active_path_count_text: active_path_count_text.as_str(),
        branch_point_count_text: branch_point_count_text.as_str(),
        omission_count_text: omission_count_text.as_str(),
        artifact_bytes_text: artifact_bytes_text.as_str(),
    });
    Ok(SessionTreeExportReceipt {
        schema: String::from(SESSION_TREE_EXPORT_RECEIPT_SCHEMA),
        policy_id: input.policy.policy_id.clone(),
        policy_hash: input.policy.policy_hash.clone(),
        source_hash: input.projection.source_hash.clone(),
        projection_hash: input.projection.projection_hash.clone(),
        entry_count,
        active_path_count: input.projection.active_path_count,
        branch_point_count: input.projection.branch_point_count,
        omission_count: input.projection.omission_count,
        format: input.artifact.format,
        artifact_hash: input.artifact.artifact_hash.clone(),
        artifact_bytes,
        write_status: String::from(input.write.status),
        redaction_marker: String::from(REDACTION_MARKER),
        non_claim: String::from(NON_CLAIM),
        receipt_hash,
    })
}

fn validate_host_write(
    write: WriteProof<'_>,
    input: &SessionTreeExportReceiptInput<'_>,
) -> Result<(), SessionTreeExportError> {
    super::core::safe_label(write.destination_capability, RECEIPT_FIELD)?;
    super::core::safe_label(write.status, RECEIPT_FIELD)?;
    let has_exact_artifact = input.artifact.projection_hash == input.projection.projection_hash
        && write.artifact_hash == input.artifact.artifact_hash;
    let has_exact_byte_count =
        write.bytes_written == input.artifact.byte_count && write.max_bytes >= write.bytes_written;
    let has_completed_write =
        write.committed && write.synchronized && write.status == REQUIRED_WRITE_STATUS;
    if !has_exact_artifact || !has_exact_byte_count {
        return Err(SessionTreeExportError::new(
            SessionTreeExportIssue::InvalidWrite,
            FIELD_RECEIPT,
        ));
    }
    if write.file_mode != PRIVATE_FILE_MODE || !has_completed_write {
        return Err(SessionTreeExportError::new(
            SessionTreeExportIssue::InvalidWrite,
            FIELD_RECEIPT,
        ));
    }
    debug_assert_eq!(write.artifact_hash, input.artifact.artifact_hash);
    debug_assert!(write.bytes_written <= write.max_bytes);
    Ok(())
}

fn receipt_hash(input: &ReceiptHashInput<'_>) -> String {
    let overwrite_posture = input.receipt.write.allow_overwrite.to_string();
    let file_mode = input.receipt.write.file_mode.to_string();
    let write_budget_bytes = input.receipt.write.max_bytes.to_string();
    debug_assert!(!input.entry_count_text.is_empty());
    debug_assert!(!input.artifact_bytes_text.is_empty());
    super::core::hash_parts(
        SESSION_TREE_EXPORT_RECEIPT_HASH_DOMAIN,
        &[
            SESSION_TREE_EXPORT_RECEIPT_SCHEMA,
            input.receipt.policy.policy_id.as_str(),
            input.receipt.policy.policy_hash.as_str(),
            input.receipt.projection.source_hash.as_str(),
            input.receipt.projection.projection_hash.as_str(),
            input.entry_count_text,
            input.active_path_count_text,
            input.branch_point_count_text,
            input.omission_count_text,
            input.receipt.artifact.format.as_str(),
            input.receipt.artifact.artifact_hash.as_str(),
            input.artifact_bytes_text,
            input.receipt.write.destination_capability,
            overwrite_posture.as_str(),
            file_mode.as_str(),
            write_budget_bytes.as_str(),
            input.receipt.write.status,
            REDACTION_MARKER,
            NON_CLAIM,
        ],
    )
}

fn reject_sensitive(
    sensitive: SessionTreeExportSensitiveDraft<'_>,
) -> Result<(), SessionTreeExportError> {
    let has_content = sensitive.raw_content.is_some() || sensitive.tool_body.is_some();
    let has_provider_material =
        sensitive.reasoning.is_some() || sensitive.provider_payload.is_some();
    let has_host_material = sensitive.credential.is_some()
        || sensitive.private_path.is_some()
        || sensitive.environment.is_some();
    let has_unbounded_material =
        sensitive.raw_error.is_some() || sensitive.private_custom_entry.is_some();
    if has_content || has_provider_material {
        return Err(SessionTreeExportError::new(
            SessionTreeExportIssue::ReceiptLeak,
            FIELD_RECEIPT,
        ));
    }
    if has_host_material || has_unbounded_material {
        return Err(SessionTreeExportError::new(
            SessionTreeExportIssue::ReceiptLeak,
            FIELD_RECEIPT,
        ));
    }
    debug_assert!(!has_content && !has_provider_material);
    debug_assert!(!has_host_material && !has_unbounded_material);
    Ok(())
}

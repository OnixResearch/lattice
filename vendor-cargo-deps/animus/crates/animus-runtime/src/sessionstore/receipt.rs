//! Metadata-only durable session-store receipt projection.

type DurableSessionStoreError = super::data::DurableSessionStoreError;
type DurableSessionStoreIssue = super::data::DurableSessionStoreIssue;
type DurableSessionStoreReceipt = super::data::DurableSessionStoreReceipt;
type DurableSessionStoreReceiptInput<'a> = super::data::DurableSessionStoreReceiptInput<'a>;
type SensitiveDraft<'a> = super::data::DurableSessionStoreSensitiveDraft<'a>;

const FIELD_RECEIPT: &str = "durable_session_store_receipt";
const RECEIPT_FIELD: super::core::StoreField = super::core::StoreField(FIELD_RECEIPT);

/// Projects a metadata-only durable store receipt.
///
/// # Errors
///
/// Returns [`DurableSessionStoreError`] for malformed safe metadata, changed
/// identity bindings, or any supplied private payload, path, lock dump, key, or
/// unbounded error body.
// r[impl agent_harness_runtime.durable_session_store.receipts]
pub fn project_durable_session_store_receipt(
    input: &DurableSessionStoreReceiptInput<'_>,
) -> Result<DurableSessionStoreReceipt, DurableSessionStoreError> {
    reject_sensitive(input.sensitive)?;
    super::core::validate_durable_session_store_descriptor(input.policy, input.descriptor)?;
    if input.durability != input.descriptor.durability {
        return Err(DurableSessionStoreError::new(
            DurableSessionStoreIssue::InvalidDescriptor,
            FIELD_RECEIPT,
        ));
    }
    super::core::safe_label(input.status, RECEIPT_FIELD)?;
    super::core::safe_label(input.lock_status, RECEIPT_FIELD)?;
    super::core::lowercase_blake3(input.prior_head_hash, RECEIPT_FIELD)?;
    super::core::lowercase_blake3(input.new_head_hash, RECEIPT_FIELD)?;
    let frame_count = input.committed_frame_count.to_string();
    let receipt_hash = super::core::hash_parts(
        super::data::DURABLE_SESSION_RECEIPT_HASH_DOMAIN,
        &[
            super::data::DURABLE_SESSION_RECEIPT_SCHEMA.as_bytes(),
            input.descriptor.store_id.as_bytes(),
            input.descriptor.policy_hash.as_bytes(),
            input.descriptor.session_id.as_bytes(),
            input.descriptor.branch_id.as_bytes(),
            input.operation.as_str().as_bytes(),
            input.durability.as_str().as_bytes(),
            input.status.as_bytes(),
            input.prior_head_hash.as_bytes(),
            input.new_head_hash.as_bytes(),
            frame_count.as_bytes(),
            recovery_label(input.recovery_status).as_bytes(),
            input.lock_status.as_bytes(),
            super::data::REDACTION_MARKER.as_bytes(),
            super::data::NON_CLAIM.as_bytes(),
        ],
    )?;
    debug_assert!(!receipt_hash.is_empty());
    debug_assert_eq!(receipt_hash.len(), super::data::BLAKE3_HEX_CHARS);
    Ok(DurableSessionStoreReceipt {
        schema: String::from(super::data::DURABLE_SESSION_RECEIPT_SCHEMA),
        store_id: input.descriptor.store_id.clone(),
        policy_hash: input.descriptor.policy_hash.clone(),
        session_id: input.descriptor.session_id.clone(),
        branch_id: input.descriptor.branch_id.clone(),
        operation: input.operation,
        durability: input.durability,
        status: String::from(input.status),
        prior_head_hash: String::from(input.prior_head_hash),
        new_head_hash: String::from(input.new_head_hash),
        committed_frame_count: input.committed_frame_count,
        recovery_status: input.recovery_status,
        lock_status: String::from(input.lock_status),
        redaction_marker: String::from(super::data::REDACTION_MARKER),
        non_claim: String::from(super::data::NON_CLAIM),
        receipt_hash,
    })
}

fn reject_sensitive(sensitive: SensitiveDraft<'_>) -> Result<(), DurableSessionStoreError> {
    let has_private_bytes = sensitive.private_payload.is_some()
        || sensitive.tool_result.is_some()
        || sensitive.raw_frame.is_some();
    let has_host_private = sensitive.provider_payload.is_some()
        || sensitive.credential.is_some()
        || sensitive.private_path.is_some();
    let has_unbounded_diagnostic = sensitive.lock_dump.is_some() || sensitive.raw_error.is_some();
    if has_private_bytes || has_host_private || has_unbounded_diagnostic {
        return Err(DurableSessionStoreError::new(
            DurableSessionStoreIssue::ReceiptLeak,
            FIELD_RECEIPT,
        ));
    }
    debug_assert!(!has_private_bytes);
    debug_assert!(!has_host_private && !has_unbounded_diagnostic);
    Ok(())
}

const fn recovery_label(status: super::data::DurableSessionRecoveryStatus) -> &'static str {
    match status {
        super::data::DurableSessionRecoveryStatus::Empty => "empty",
        super::data::DurableSessionRecoveryStatus::Clean => "clean",
        super::data::DurableSessionRecoveryStatus::IncompleteTailFrame => "incomplete_tail_frame",
        super::data::DurableSessionRecoveryStatus::IncompleteTailTransaction => {
            "incomplete_tail_transaction"
        }
    }
}

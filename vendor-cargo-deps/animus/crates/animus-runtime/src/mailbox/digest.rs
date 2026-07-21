/// Returns a deterministic BLAKE3 hash over queue-control hash parts.
#[must_use]
pub fn blake3_message_queue_hash(parts: &[&str]) -> String {
    let mut hasher = blake3::Hasher::new();
    for part in parts {
        hasher.update(part.len().to_string().as_bytes());
        hasher.update(&[super::HASH_PART_SEPARATOR as u8]);
        hasher.update(part.as_bytes());
        hasher.update(&[super::HASH_PART_SEPARATOR as u8]);
    }
    hasher.finalize().to_hex().to_string()
}

pub(crate) fn receipt_hash(input: &super::record::ReceiptHashInput<'_>) -> String {
    blake3_message_queue_hash(&[
        super::MESSAGE_QUEUE_RECEIPT_SCHEMA,
        input.operation_kind,
        input.source,
        input.active_work_hash,
        input.queue_item_hash,
        input.decision_class,
        input.cancellation_handle_class,
        input.budget_class,
        input.policy_hash,
        input.denial_class.unwrap_or(super::EMPTY_HASH_MARKER),
        input.redaction_state,
    ])
}

//! Metadata-only coding-session receipt projection.

const FIELD_RECEIPT: &str = "receipt";
const RECEIPT_HASH_HEX_CHARS: usize = 64;

#[derive(Clone, Copy)]
struct ReceiptHashInput<'a> {
    state: &'a super::data::CodingSessionState,
    terminal_status: super::data::CodingSessionTerminalKind,
    committed_message_count: u32,
    pending_queue_count: u32,
    non_claims: &'a [String],
}

/// Projects bounded terminal session metadata while rejecting supplied private material.
///
/// # Errors
///
/// Returns [`super::data::CodingSessionError`] when the session is not terminal, counters overflow,
/// or any sensitive receipt field is supplied.
pub fn project_coding_session_receipt(
    state: &super::data::CodingSessionState,
    sensitive: super::data::CodingSessionSensitiveDraft<'_>,
) -> Result<super::data::CodingSessionReceipt, super::data::CodingSessionError> {
    reject_sensitive(sensitive)?;
    let terminal_status = state
        .terminal
        .ok_or_else(|| super::core::error(super::data::CodingSessionIssue::InvalidState, FIELD_RECEIPT))?;
    let committed_message_count = u32::try_from(state.messages.len())
        .map_err(|_overflow| super::core::error(super::data::CodingSessionIssue::BoundExceeded, FIELD_RECEIPT))?;
    let pending_queue_count = u32::try_from(state.queued_bodies.len())
        .map_err(|_overflow| super::core::error(super::data::CodingSessionIssue::BoundExceeded, FIELD_RECEIPT))?;
    let authority_profile_identity = state
        .committed_authority_profile_identity()
        .ok_or_else(|| super::core::error(super::data::CodingSessionIssue::AuthorityProfileUnbound, FIELD_RECEIPT))?;
    let authority_binding_identity = state
        .authority_binding_identity()
        .ok_or_else(|| super::core::error(super::data::CodingSessionIssue::AuthorityProfileUnbound, FIELD_RECEIPT))?;
    let non_claims = super::data::NON_CLAIMS.iter().map(|value| String::from(*value)).collect::<Vec<_>>();
    let receipt_hash = receipt_hash(ReceiptHashInput {
        state,
        terminal_status,
        committed_message_count,
        pending_queue_count,
        non_claims: non_claims.as_slice(),
    });
    let receipt = super::data::CodingSessionReceipt {
        schema: super::data::CODING_SESSION_RECEIPT_SCHEMA,
        session_id: String::from(state.session_id()),
        branch_id: String::from(state.branch_id()),
        terminal_status,
        committed_message_count,
        commit_count: state.commit_count,
        provider_dispatch_count: state.provider_dispatch_count,
        tool_dispatch_count: state.tool_dispatch_count,
        pending_queue_count,
        durable_cursor: state.durable_cursor,
        committed_head_hash: state.durable_head_hash.clone(),
        authority_profile_identity: String::from(authority_profile_identity),
        authority_binding_identity: String::from(authority_binding_identity),
        redaction_marker: super::data::REDACTION_MARKER,
        non_claims,
        receipt_hash,
    };
    debug_assert_eq!(receipt.session_id, state.session_id());
    debug_assert_eq!(receipt.terminal_status, terminal_status);
    Ok(receipt)
}

fn reject_sensitive(
    sensitive: super::data::CodingSessionSensitiveDraft<'_>,
) -> Result<(), super::data::CodingSessionError> {
    if has_sensitive_data(sensitive) {
        Err(super::core::error(super::data::CodingSessionIssue::UnsafeReceipt, FIELD_RECEIPT))
    } else {
        Ok(())
    }
}

fn has_sensitive_data(sensitive: super::data::CodingSessionSensitiveDraft<'_>) -> bool {
    [
        sensitive.raw_prompt,
        sensitive.provider_payload,
        sensitive.tool_output,
        sensitive.credential,
        sensitive.capability_path,
        sensitive.store_path,
        sensitive.raw_error,
    ]
    .iter()
    .any(Option::is_some)
}

fn receipt_hash(input: ReceiptHashInput<'_>) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(super::data::CODING_SESSION_RECEIPT_HASH_DOMAIN.as_bytes());
    debug_assert!(!super::data::CODING_SESSION_RECEIPT_HASH_DOMAIN.is_empty());
    let owned = [
        super::data::CODING_SESSION_RECEIPT_SCHEMA.to_string(),
        input.state.session_id().to_string(),
        input.state.branch_id().to_string(),
        input.terminal_status.as_str().to_string(),
        input.committed_message_count.to_string(),
        input.state.commit_count.to_string(),
        input.state.provider_dispatch_count.to_string(),
        input.state.tool_dispatch_count.to_string(),
        input.pending_queue_count.to_string(),
        input.state.durable_cursor.to_string(),
        input.state.durable_head_hash.clone(),
        input.state.committed_authority_profile_identity().unwrap_or_default().to_string(),
        input.state.authority_binding_identity().unwrap_or_default().to_string(),
        super::data::REDACTION_MARKER.to_string(),
    ];
    for part in owned.iter().chain(input.non_claims.iter()) {
        hasher.update(super::data::HASH_SEPARATOR);
        hasher.update(part.as_bytes());
    }
    let hash = hasher.finalize().to_hex().to_string();
    debug_assert_eq!(hash.len(), RECEIPT_HASH_HEX_CHARS);
    hash
}

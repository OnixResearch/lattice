//! Metadata-only coding-session receipt projection.

const FIELD_RECEIPT: &str = "receipt";
const RECEIPT_HASH_HEX_CHARS: usize = 64;
const QUEUE_RECEIPT_SCHEMA: &str = "animus.coding-session.queue-mutation-receipt.v1";
const QUEUE_RECEIPT_HASH_DOMAIN: &str = "animus.coding-session.queue-mutation-receipt.v1";
const QUEUE_REDACTION_MARKER: &str = "private_queue_text_omitted";
const QUEUE_STATUS_ACCEPTED: &str = "accepted";
const QUEUE_STATUS_CONFLICT: &str = "conflict";

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
    let terminal_status = state.terminal.ok_or_else(|| {
        super::core::error(super::data::CodingSessionIssue::InvalidState, FIELD_RECEIPT)
    })?;
    let committed_message_count = u32::try_from(state.messages.len()).map_err(|_overflow| {
        super::core::error(
            super::data::CodingSessionIssue::BoundExceeded,
            FIELD_RECEIPT,
        )
    })?;
    let pending_queue_count = u32::try_from(state.queued_bodies.len()).map_err(|_overflow| {
        super::core::error(
            super::data::CodingSessionIssue::BoundExceeded,
            FIELD_RECEIPT,
        )
    })?;
    let authority_profile_identity =
        state
            .committed_authority_profile_identity()
            .ok_or_else(|| {
                super::core::error(
                    super::data::CodingSessionIssue::AuthorityProfileUnbound,
                    FIELD_RECEIPT,
                )
            })?;
    let authority_binding_identity = state.authority_binding_identity().ok_or_else(|| {
        super::core::error(
            super::data::CodingSessionIssue::AuthorityProfileUnbound,
            FIELD_RECEIPT,
        )
    })?;
    let non_claims = super::data::NON_CLAIMS
        .iter()
        .map(|value| String::from(*value))
        .collect::<Vec<_>>();
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
        Err(super::core::error(
            super::data::CodingSessionIssue::UnsafeReceipt,
            FIELD_RECEIPT,
        ))
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

// r[impl agent_harness_runtime.coding_session_queue_cas.evidence]
/// Projects metadata-only evidence for an accepted queue mutation.
///
/// # Errors
///
/// Returns [`super::data::CodingSessionQueueReceiptError`] when private text is
/// supplied, identities are unsafe, or the generation did not advance once.
pub fn project_coding_session_queue_success_receipt(
    before: &super::data::CodingSessionState,
    after: &super::data::CodingSessionState,
    accepted: super::data::CodingSessionQueueAcceptedMutation<'_>,
    sensitive: super::data::CodingSessionQueueSensitiveDraft<'_>,
) -> Result<
    super::data::CodingSessionQueueMutationReceipt,
    super::data::CodingSessionQueueReceiptError,
> {
    reject_queue_sensitive(sensitive)?;
    validate_queue_receipt_identity(accepted.item_id)?;
    validate_queue_receipt_identity(accepted.precondition.entry_id())?;
    validate_accepted_queue_transition(before, after, accepted)?;
    let snapshot = super::core::project_coding_session_queue_safe_snapshot(after)
        .map_err(|_error| super::data::CodingSessionQueueReceiptError::InvalidIdentity)?;
    Ok(build_queue_receipt(QueueReceiptInput {
        operation: accepted.operation,
        status: QUEUE_STATUS_ACCEPTED,
        item_id: accepted.item_id,
        entry_id: accepted.precondition.entry_id(),
        position: Some(accepted.precondition.expected_position()),
        generation_before: before.queue_generation,
        generation_after: Some(after.queue_generation),
        queue_identity: snapshot.queue_identity.as_str(),
        conflict: None,
    }))
}

/// Projects metadata-only evidence for a rejected queue mutation.
///
/// # Errors
///
/// Returns [`super::data::CodingSessionQueueReceiptError`] when private text is
/// supplied or safe identities are malformed.
pub fn project_coding_session_queue_conflict_receipt(
    conflict: &super::data::CodingSessionQueueConflict,
    sensitive: super::data::CodingSessionQueueSensitiveDraft<'_>,
) -> Result<
    super::data::CodingSessionQueueMutationReceipt,
    super::data::CodingSessionQueueReceiptError,
> {
    reject_queue_sensitive(sensitive)?;
    validate_queue_receipt_identity(conflict.item_id.as_str())?;
    validate_queue_receipt_identity(conflict.entry_id.as_str())?;
    let position = conflict
        .snapshot
        .entries
        .iter()
        .find(|entry| entry.item_id == conflict.item_id)
        .map(|entry| entry.position);
    Ok(build_queue_receipt(QueueReceiptInput {
        operation: conflict.operation,
        status: QUEUE_STATUS_CONFLICT,
        item_id: conflict.item_id.as_str(),
        entry_id: conflict.entry_id.as_str(),
        position,
        generation_before: conflict.snapshot.generation,
        generation_after: None,
        queue_identity: conflict.snapshot.queue_identity.as_str(),
        conflict: Some(conflict.kind),
    }))
}

#[derive(Clone, Copy)]
struct QueueReceiptInput<'a> {
    operation: super::data::CodingSessionQueueMutationOperation,
    status: &'static str,
    item_id: &'a str,
    entry_id: &'a str,
    position: Option<u32>,
    generation_before: super::data::CodingSessionQueueGeneration,
    generation_after: Option<super::data::CodingSessionQueueGeneration>,
    queue_identity: &'a str,
    conflict: Option<super::data::CodingSessionQueueConflictKind>,
}

fn build_queue_receipt(
    input: QueueReceiptInput<'_>,
) -> super::data::CodingSessionQueueMutationReceipt {
    let receipt_hash = queue_receipt_hash(input);
    super::data::CodingSessionQueueMutationReceipt {
        schema: QUEUE_RECEIPT_SCHEMA,
        operation: input.operation,
        status: input.status,
        item_id: String::from(input.item_id),
        entry_id: String::from(input.entry_id),
        position: input.position,
        generation_before: input.generation_before,
        generation_after: input.generation_after,
        queue_identity: String::from(input.queue_identity),
        conflict: input.conflict,
        redaction_marker: QUEUE_REDACTION_MARKER,
        receipt_hash,
    }
}

fn validate_accepted_queue_transition(
    before: &super::data::CodingSessionState,
    after: &super::data::CodingSessionState,
    accepted: super::data::CodingSessionQueueAcceptedMutation<'_>,
) -> Result<(), super::data::CodingSessionQueueReceiptError> {
    let expected_after = before
        .queue_generation
        .checked_next()
        .ok_or(super::data::CodingSessionQueueReceiptError::InvalidIdentity)?;
    let stable_session = before.session_id() == after.session_id()
        && before.branch_id() == after.branch_id()
        && before.run == after.run
        && before.messages == after.messages
        && before.active_leaf_id == after.active_leaf_id
        && before.durable_head_hash == after.durable_head_hash
        && before.durable_cursor == after.durable_cursor;
    if !stable_session
        || before.queue_generation != accepted.precondition.expected_generation()
        || after.queue_generation != expected_after
    {
        return Err(super::data::CodingSessionQueueReceiptError::InvalidIdentity);
    }
    let expected_position = usize::try_from(accepted.precondition.expected_position())
        .map_err(|_overflow| super::data::CodingSessionQueueReceiptError::InvalidIdentity)?;
    let before_target = before
        .queued_bodies
        .get(expected_position)
        .ok_or(super::data::CodingSessionQueueReceiptError::InvalidIdentity)?;
    let exact_before = before_target.item_id == accepted.item_id
        && before_target.entry_id == accepted.precondition.entry_id()
        && super::core::coding_session_queue_text_identity(before_target.body.as_str())
            == accepted.precondition.expected_text_identity();
    if !exact_before {
        return Err(super::data::CodingSessionQueueReceiptError::InvalidIdentity);
    }
    match accepted.operation {
        super::data::CodingSessionQueueMutationOperation::Edit => {
            let after_target = after
                .queued_bodies
                .get(expected_position)
                .ok_or(super::data::CodingSessionQueueReceiptError::InvalidIdentity)?;
            if before.queued_bodies.len() != after.queued_bodies.len()
                || after_target.item_id != accepted.item_id
                || after_target.entry_id != accepted.precondition.entry_id()
            {
                return Err(super::data::CodingSessionQueueReceiptError::InvalidIdentity);
            }
            for (position, before_entry) in before.queued_bodies.iter().enumerate() {
                if position != expected_position
                    && after.queued_bodies.get(position) != Some(before_entry)
                {
                    return Err(super::data::CodingSessionQueueReceiptError::InvalidIdentity);
                }
            }
        }
        super::data::CodingSessionQueueMutationOperation::Remove => {
            let expected_count = before
                .queued_bodies
                .len()
                .checked_sub(1)
                .ok_or(super::data::CodingSessionQueueReceiptError::InvalidIdentity)?;
            let expected_entries = before
                .queued_bodies
                .iter()
                .enumerate()
                .filter_map(|(position, entry)| (position != expected_position).then_some(entry));
            if after.queued_bodies.len() != expected_count
                || expected_entries.ne(after.queued_bodies.iter())
            {
                return Err(super::data::CodingSessionQueueReceiptError::InvalidIdentity);
            }
        }
    }
    Ok(())
}

fn reject_queue_sensitive(
    sensitive: super::data::CodingSessionQueueSensitiveDraft<'_>,
) -> Result<(), super::data::CodingSessionQueueReceiptError> {
    if [sensitive.current, sensitive.expected, sensitive.replacement]
        .iter()
        .any(Option::is_some)
    {
        return Err(super::data::CodingSessionQueueReceiptError::UnsafePrivateText);
    }
    Ok(())
}

fn validate_queue_receipt_identity(
    value: &str,
) -> Result<(), super::data::CodingSessionQueueReceiptError> {
    let valid_length = !value.is_empty() && value.len() <= super::data::MAX_SAFE_ID_BYTES;
    let valid_characters = value
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-'));
    if !valid_length || !valid_characters {
        return Err(super::data::CodingSessionQueueReceiptError::InvalidIdentity);
    }
    Ok(())
}

fn queue_receipt_hash(input: QueueReceiptInput<'_>) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(QUEUE_RECEIPT_HASH_DOMAIN.as_bytes());
    let position = input
        .position
        .map_or_else(String::new, |value| value.to_string());
    let generation_after = input
        .generation_after
        .map_or_else(String::new, |value| value.value().to_string());
    let conflict = input.conflict.map_or("none", |value| value.as_str());
    let parts = [
        QUEUE_RECEIPT_SCHEMA,
        input.operation.as_str(),
        input.status,
        input.item_id,
        input.entry_id,
        position.as_str(),
        generation_after.as_str(),
        input.queue_identity,
        conflict,
        QUEUE_REDACTION_MARKER,
    ];
    hasher.update(super::data::HASH_SEPARATOR);
    hasher.update(&input.generation_before.value().to_le_bytes());
    for part in parts {
        hasher.update(super::data::HASH_SEPARATOR);
        hasher.update(part.as_bytes());
    }
    let hash = hasher.finalize().to_hex().to_string();
    debug_assert_eq!(hash.len(), RECEIPT_HASH_HEX_CHARS);
    hash
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
        input
            .state
            .committed_authority_profile_identity()
            .unwrap_or_default()
            .to_string(),
        input
            .state
            .authority_binding_identity()
            .unwrap_or_default()
            .to_string(),
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

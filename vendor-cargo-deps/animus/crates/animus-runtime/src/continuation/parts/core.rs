#[derive(Clone, Copy)]
struct ReceiptHashParts<'a> {
    prior_state_class: ContinuationStateClass,
    selected_action: ContinuationAction,
    pending_work_id_hash: &'a str,
    reason: &'a str,
    rejection_class: Option<&'a str>,
    policy_hash: &'a str,
}

// r[impl agent_harness_runtime.continuation_policy.shell_owned]
/// Selects the next continuation action without side effects.
#[must_use]
pub fn select_next_continuation(
    state: &ContinuationState,
    policy: &ContinuationPolicy,
) -> ContinuationDecision {
    debug_assert_eq!(policy.priority().order().len(), PRIORITY_CANDIDATE_COUNT);
    debug_assert!(matches!(
        state.state_class(),
        ContinuationStateClass::Idle
            | ContinuationStateClass::Busy
            | ContinuationStateClass::PendingDispatch
            | ContinuationStateClass::Cancelled
    ));
    if let Some(cancellation) = state.cancellation() {
        return ContinuationDecision::Cancel {
            work_id: Text::from(cancellation.work_id()),
            reason: ContinuationReason::CancellationRequested,
        };
    }
    for candidate in policy.priority().order() {
        if let Some(work) = state.candidate(*candidate) {
            return ContinuationDecision::Dispatch(ContinuationDispatch {
                work_id: Text::from(work.work_id()),
                action: candidate.action(),
                reason: candidate.reason(),
            });
        }
    }
    if policy.requires_authoritative_completion() {
        return ContinuationDecision::AwaitingCompletionEvidence {
            reason: ContinuationReason::CompletionEvidenceRequired,
        };
    }
    ContinuationDecision::Complete {
        reason: ContinuationReason::NoPendingWork,
    }
}

fn clear_if_matches(slot: &mut Option<PendingContinuationWork>, work_id: &str) {
    if slot.as_ref().is_some_and(|work| work.work_id() == work_id) {
        *slot = None;
    }
}

fn reject_duplicate_candidates(
    order: [ContinuationCandidate; PRIORITY_CANDIDATE_COUNT],
) -> Result<(), ContinuationError> {
    let [first, second, third] = order;
    if first == second || first == third {
        return Err(ContinuationError::DuplicatePriority { candidate: first });
    }
    if second == third {
        return Err(ContinuationError::DuplicatePriority { candidate: second });
    }
    Ok(())
}

fn validate_work_id(field: &'static str, value: &str) -> Result<Text, ContinuationError> {
    validate_safe_label(field, value)
}

fn validate_ack_work_id(value: &str) -> Result<Text, ContinuationError> {
    if value.trim().is_empty() {
        return Err(ContinuationError::MissingWorkId {
            field: FIELD_ACK_WORK_ID,
        });
    }
    validate_work_id(FIELD_ACK_WORK_ID, value)
}

fn validate_safe_label(field: &'static str, value: &str) -> Result<Text, ContinuationError> {
    let text = required_text(field, value)?;
    if leak_marker(text.as_str()).is_some()
        || !text.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.')
        })
    {
        return Err(ContinuationError::InvalidLabel { field, value: text });
    }
    Ok(text)
}

fn validate_safe_metadata<'a>(
    field: &'static str,
    value: &'a str,
) -> Result<&'a str, ContinuationError> {
    let text = value.trim();
    if text.is_empty() {
        return Err(ContinuationError::EmptyField { field });
    }
    if let Some(marker) = leak_marker(text) {
        return Err(ContinuationError::UnsafeMetadata { field, marker });
    }
    Ok(text)
}

fn validate_hash_metadata<'a>(
    field: &'static str,
    value: &'a str,
) -> Result<&'a str, ContinuationError> {
    let text = validate_safe_metadata(field, value)?;
    if !text.chars().all(|character| character.is_ascii_hexdigit()) {
        return Err(ContinuationError::InvalidLabel {
            field,
            value: Text::from(text),
        });
    }
    Ok(text)
}

fn required_text(field: &'static str, value: &str) -> Result<Text, ContinuationError> {
    let text = value.trim();
    if text.is_empty() {
        return Err(ContinuationError::EmptyField { field });
    }
    Ok(Text::from(text))
}

fn leak_marker(value: &str) -> Option<&'static str> {
    let lower = value.to_ascii_lowercase();
    LEAK_MARKERS
        .iter()
        .find(|marker| lower.contains(marker.needle))
        .map(|marker| marker.marker)
}

fn mismatched_work_error(expected_work_id: &str, actual_work_id: &str) -> ContinuationError {
    ContinuationError::MismatchedWork {
        expected_work_id_hash: blake3_hex(expected_work_id.as_bytes()),
        actual_work_id_hash: blake3_hex(actual_work_id.as_bytes()),
    }
}

fn continuation_receipt_hash(parts: ReceiptHashParts<'_>) -> Text {
    let mut normalized = Text::from(CONTINUATION_RECEIPT_SCHEMA);
    push_hash_part(&mut normalized, parts.prior_state_class.as_str());
    push_hash_part(&mut normalized, parts.selected_action.as_str());
    push_hash_part(&mut normalized, parts.pending_work_id_hash);
    push_hash_part(&mut normalized, parts.reason);
    push_hash_part(&mut normalized, parts.rejection_class.unwrap_or_default());
    push_hash_part(&mut normalized, parts.policy_hash);
    blake3_hex(normalized.as_bytes())
}

fn push_hash_part(output: &mut Text, part: &str) {
    output.push('|');
    output.push_str(part.len().to_string().as_str());
    output.push(':');
    output.push_str(part);
}

fn blake3_hex(bytes: &[u8]) -> Text {
    blake3::hash(bytes).to_hex().to_string()
}


// r[impl agent_harness_runtime.claude_code_agents.context_packet]
/// Validates a minimized worker context packet.
///
/// # Errors
///
/// Returns [`DelegationFailure`](super::super::DelegationFailure) when the packet
/// includes disallowed full transcripts, raw hidden memory, credentials,
/// provider payloads, unbounded tool output, or unsafe text markers.
pub fn validate(
    packet: &super::super::WorkerContextPacket,
) -> Result<(), super::super::DelegationFailure> {
    super::validate_label(
        super::super::CONTEXT_PACKET_FIELD,
        packet.packet_id.as_str(),
    )?;
    super::validate_non_empty_slice(super::super::CONTEXT_ITEM_FIELD, packet.items.as_slice())?;
    if packet.items.len() > super::super::MAX_CONTEXT_ITEMS {
        return Err(super::super::DelegationFailure::new(
            super::super::FailureKind::FieldTooLong,
            super::super::CONTEXT_ITEM_FIELD,
            super::super::TOO_LONG_REASON,
        ));
    }
    for item in &packet.items {
        validate_item(&packet.policy, item)?;
    }
    Ok(())
}

fn validate_item(
    policy: &super::super::ContextAdmissionPolicy,
    item: &super::super::WorkerContextItem,
) -> Result<(), super::super::DelegationFailure> {
    if !policy.admits(item.kind) {
        return Err(super::super::DelegationFailure::new(
            super::super::FailureKind::ContextLeak,
            super::super::CONTEXT_ITEM_FIELD,
            "context kind is not admitted for worker packets",
        ));
    }
    let label = super::validate_label(super::super::CONTEXT_ITEM_FIELD, item.label.as_str())?;
    let body = super::validate_text(super::super::CONTEXT_ITEM_FIELD, item.body.as_str())?;
    assert!(!label.is_empty(), "context label must be validated");
    assert!(!body.is_empty(), "context body must be validated");
    if matches!(item.redaction, super::super::WorkerContextRedaction::Raw)
        && !item.kind.is_default_safe()
    {
        return Err(super::super::DelegationFailure::new(
            super::super::FailureKind::ContextLeak,
            super::super::CONTEXT_ITEM_FIELD,
            "raw unsafe context kind is not allowed",
        ));
    }
    Ok(())
}

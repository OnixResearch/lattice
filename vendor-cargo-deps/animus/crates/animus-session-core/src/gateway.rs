//! Pure persist-before-process gateway admission, scheduling, recovery, and authority.

#![cfg_attr(
    animus_octet,
    allow(
        tigerstyle::assertion_density,
        tigerstyle::path_segment_repetition,
        tigerstyle::unbounded_collection_growth,
        reason = "this focused pure reducer returns typed denials, uses stable gateway domain names, and emits at most one recovery action per caller-bounded record"
    )
)]

use crate::prelude::*;

/// Durable gateway record class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GatewayRecordKind {
    /// Remote inbound update.
    InboundUpdate,
    /// Pending outbound reply intent.
    ReplyIntent,
}

/// Durable admission observation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GatewayDurability {
    /// No durable commit is observed.
    Uncommitted,
    /// The exact record commit is observed.
    Committed,
}

/// Processing state for an inbound update.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GatewayWorkState {
    /// Durable work awaits processing.
    Pending,
    /// One worker owns processing.
    Processing,
    /// Processing completed.
    Completed,
    /// Processing was explicitly parked.
    Parked,
    /// Processing was explicitly cancelled.
    Cancelled,
}

/// Delivery state for a reply intent.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GatewayDeliveryState {
    /// Durable reply intent has not started delivery.
    Pending,
    /// Delivery started without a terminal observation.
    Uncertain,
    /// Exact transport delivery was observed.
    Delivered,
}

/// Gateway record draft with caller-supplied stable identities.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GatewayRecordDraft<'a> {
    /// Stable gateway work identity.
    pub work_identity: &'a str,
    /// Stable conversation identity.
    pub conversation_identity: &'a str,
    /// Stable content identity.
    pub content_identity: &'a str,
    /// Monotonic sequence within one conversation.
    pub conversation_sequence: u64,
    /// Record class.
    pub kind: GatewayRecordKind,
}

/// Validated pure gateway record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GatewayRecord {
    /// Stable gateway work identity.
    pub work_identity: String,
    /// Stable conversation identity.
    pub conversation_identity: String,
    /// Stable content identity.
    pub content_identity: String,
    /// Monotonic sequence within one conversation.
    pub conversation_sequence: u64,
    /// Record class.
    pub kind: GatewayRecordKind,
    /// Current durable admission observation.
    pub durability: GatewayDurability,
    /// Current inbound processing state.
    pub work_state: GatewayWorkState,
    /// Current reply delivery state.
    pub delivery_state: GatewayDeliveryState,
}

/// Pure admission plan. Shell effects remain blocked until exact commit evidence exists.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GatewayAdmissionPlan {
    /// Record that the shell must commit.
    pub record: GatewayRecord,
    /// Processing or delivery is always blocked by the initial plan.
    pub effect_admitted: bool,
}

/// Bounded scheduling decision for one admitted update.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GatewayProcessingPlan {
    /// Exact work identity that can start.
    pub work_identity: String,
    /// Exact conversation identity that owns the slot.
    pub conversation_identity: String,
    /// Active cross-conversation count after admission.
    pub active_conversation_count: u32,
}

/// Pure recovery action for one durable nonterminal record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GatewayRecoveryAction {
    /// Stable work identity.
    pub work_identity: String,
    /// State that recovery must preserve or resume.
    pub disposition: GatewayRecoveryDisposition,
}

/// Recovery disposition without invented external certainty.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GatewayRecoveryDisposition {
    /// Return an admitted update to pending processing.
    ResumePendingUpdate,
    /// Return an unsent reply intent to pending delivery.
    ResumePendingReply,
    /// Keep an in-flight reply uncertain.
    PreserveUncertainReply,
}

/// Gateway policy rejection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GatewayError {
    /// A stable identity is empty.
    EmptyIdentity,
    /// An effect was requested before exact durable admission.
    NotDurable,
    /// Record kind does not support the requested transition.
    WrongRecordKind,
    /// Earlier same-conversation work blocks this update.
    ConversationOrder,
    /// The explicit cross-conversation capacity is exhausted.
    CapacityExceeded,
    /// A durable work identity appears more than once.
    DuplicateIdentity,
    /// Recovery attempted to invent delivered state.
    FalseDelivery,
    /// Exact remote mutation authority is absent.
    MutationDenied,
}

/// Exact remote gateway mutation request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GatewayToolRequest<'a> {
    /// Gateway session identity.
    pub session_identity: &'a str,
    /// Bound remote-user identity.
    pub remote_user_identity: &'a str,
    /// Exact tool name.
    pub tool_name: &'a str,
    /// Exact request identity.
    pub request_identity: &'a str,
    /// Whether the tool is read-only.
    pub read_only: bool,
    /// Transport authentication observation. This never grants mutation.
    pub transport_authenticated: bool,
}

/// Exact current remote gateway mutation authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GatewayMutationAuthority<'a> {
    /// Gateway session identity.
    pub session_identity: &'a str,
    /// Bound remote-user identity.
    pub remote_user_identity: &'a str,
    /// Exact tool name.
    pub tool_name: &'a str,
    /// Exact request identity.
    pub request_identity: &'a str,
    /// Current authority decision.
    pub admitted: bool,
}

/// Plans an inbound or reply record that must be committed before its effect.
///
/// # Errors
///
/// Returns [`GatewayError::EmptyIdentity`] when any stable identity is empty.
// r[impl animus.gateway.persist]
pub fn plan_gateway_admission(
    draft: GatewayRecordDraft<'_>,
) -> Result<GatewayAdmissionPlan, GatewayError> {
    if draft.work_identity.is_empty()
        || draft.conversation_identity.is_empty()
        || draft.content_identity.is_empty()
    {
        return Err(GatewayError::EmptyIdentity);
    }
    Ok(GatewayAdmissionPlan {
        record: GatewayRecord {
            work_identity: String::from(draft.work_identity),
            conversation_identity: String::from(draft.conversation_identity),
            content_identity: String::from(draft.content_identity),
            conversation_sequence: draft.conversation_sequence,
            kind: draft.kind,
            durability: GatewayDurability::Uncommitted,
            work_state: GatewayWorkState::Pending,
            delivery_state: GatewayDeliveryState::Pending,
        },
        effect_admitted: false,
    })
}

/// Applies an exact durable commit observation.
///
/// # Errors
///
/// Returns [`GatewayError::DuplicateIdentity`] when the observed identity differs.
pub fn observe_gateway_commit(
    record: &GatewayRecord,
    observed_work_identity: &str,
) -> Result<GatewayRecord, GatewayError> {
    if record.work_identity != observed_work_identity {
        return Err(GatewayError::DuplicateIdentity);
    }
    let mut committed = record.clone();
    committed.durability = GatewayDurability::Committed;
    Ok(committed)
}

/// Admits processing only after durable admission, in conversation order, and under capacity.
///
/// # Errors
///
/// Returns a bounded policy error when durability, ordering, kind, or capacity fails.
// r[impl animus.gateway.order]
pub fn plan_gateway_processing(
    record: &GatewayRecord,
    records: &[GatewayRecord],
    active_conversation_capacity: u32,
) -> Result<GatewayProcessingPlan, GatewayError> {
    if record.kind != GatewayRecordKind::InboundUpdate {
        return Err(GatewayError::WrongRecordKind);
    }
    if record.durability != GatewayDurability::Committed {
        return Err(GatewayError::NotDurable);
    }
    if records.iter().any(|candidate| {
        candidate.conversation_identity == record.conversation_identity
            && candidate.conversation_sequence < record.conversation_sequence
            && !matches!(
                candidate.work_state,
                GatewayWorkState::Completed
                    | GatewayWorkState::Parked
                    | GatewayWorkState::Cancelled
            )
    }) {
        return Err(GatewayError::ConversationOrder);
    }
    let mut active_conversation_count = 0_u32;
    let mut is_record_conversation_active = false;
    for (index, candidate) in records.iter().enumerate() {
        if candidate.work_state != GatewayWorkState::Processing {
            continue;
        }
        let is_already_counted = records[..index].iter().any(|prior| {
            prior.work_state == GatewayWorkState::Processing
                && prior.conversation_identity == candidate.conversation_identity
        });
        if !is_already_counted {
            active_conversation_count = active_conversation_count
                .checked_add(1)
                .ok_or(GatewayError::CapacityExceeded)?;
        }
        if candidate.conversation_identity == record.conversation_identity {
            is_record_conversation_active = true;
        }
    }
    let is_new_conversation = !is_record_conversation_active;
    if is_new_conversation && active_conversation_count >= active_conversation_capacity {
        return Err(GatewayError::CapacityExceeded);
    }
    let planned_count = active_conversation_count
        .checked_add(u32::from(is_new_conversation))
        .ok_or(GatewayError::CapacityExceeded)?;
    Ok(GatewayProcessingPlan {
        work_identity: record.work_identity.clone(),
        conversation_identity: record.conversation_identity.clone(),
        active_conversation_count: planned_count,
    })
}

/// Admits reply delivery only after the exact reply intent is durable.
///
/// # Errors
///
/// Returns a bounded policy error when kind or durability fails.
pub fn plan_gateway_delivery(record: &GatewayRecord) -> Result<&str, GatewayError> {
    if record.kind != GatewayRecordKind::ReplyIntent {
        return Err(GatewayError::WrongRecordKind);
    }
    if record.durability != GatewayDurability::Committed {
        return Err(GatewayError::NotDurable);
    }
    Ok(record.work_identity.as_str())
}

/// Reconstructs pending work while preserving uncertain delivery.
///
/// # Errors
///
/// Returns [`GatewayError::DuplicateIdentity`] for duplicate durable work identities.
// r[impl animus.gateway.resume]
pub fn recover_gateway_work(
    records: &[GatewayRecord],
) -> Result<Vec<GatewayRecoveryAction>, GatewayError> {
    let mut seen = Vec::<&str>::new();
    let mut actions = Vec::new();
    for record in records {
        if seen.contains(&record.work_identity.as_str()) {
            return Err(GatewayError::DuplicateIdentity);
        }
        seen.push(record.work_identity.as_str());
        if record.durability != GatewayDurability::Committed {
            continue;
        }
        let disposition = match (record.kind, record.work_state, record.delivery_state) {
            (
                GatewayRecordKind::InboundUpdate,
                GatewayWorkState::Pending | GatewayWorkState::Processing,
                _,
            ) => Some(GatewayRecoveryDisposition::ResumePendingUpdate),
            (GatewayRecordKind::ReplyIntent, _, GatewayDeliveryState::Pending) => {
                Some(GatewayRecoveryDisposition::ResumePendingReply)
            }
            (GatewayRecordKind::ReplyIntent, _, GatewayDeliveryState::Uncertain) => {
                Some(GatewayRecoveryDisposition::PreserveUncertainReply)
            }
            _ => None,
        };
        if let Some(disposition) = disposition {
            actions.push(GatewayRecoveryAction {
                work_identity: record.work_identity.clone(),
                disposition,
            });
        }
    }
    Ok(actions)
}

/// Rejects a recovery proposal that changes uncertain delivery into delivered.
///
/// # Errors
///
/// Returns [`GatewayError::FalseDelivery`] for invented delivery certainty.
pub const fn validate_gateway_delivery_recovery(
    durable: GatewayDeliveryState,
    proposed: GatewayDeliveryState,
) -> Result<(), GatewayError> {
    if matches!(
        durable,
        GatewayDeliveryState::Pending | GatewayDeliveryState::Uncertain
    ) && matches!(proposed, GatewayDeliveryState::Delivered)
    {
        return Err(GatewayError::FalseDelivery);
    }
    Ok(())
}

/// Admits read-only tools by default and mutation only under exact current authority.
///
/// # Errors
///
/// Returns [`GatewayError::MutationDenied`] when mutation authority is absent or inexact.
// r[impl animus.gateway.authority]
pub fn admit_gateway_tool(
    request: GatewayToolRequest<'_>,
    authorities: &[GatewayMutationAuthority<'_>],
) -> Result<(), GatewayError> {
    if request.read_only {
        return Ok(());
    }
    if authorities.iter().any(|authority| {
        authority.admitted
            && authority.session_identity == request.session_identity
            && authority.remote_user_identity == request.remote_user_identity
            && authority.tool_name == request.tool_name
            && authority.request_identity == request.request_identity
    }) {
        Ok(())
    } else {
        Err(GatewayError::MutationDenied)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONVERSATION: &str = "conversation-a";
    const CONTENT: &str = "b3:content";
    const UPDATE: &str = "update-1";
    const REPLY: &str = "reply-1";
    const CAPACITY: u32 = 2;
    const EXPECTED_RECOVERY_ACTIONS: usize = 2;

    fn draft(
        work: &'static str,
        sequence: u64,
        kind: GatewayRecordKind,
    ) -> GatewayRecordDraft<'static> {
        GatewayRecordDraft {
            work_identity: work,
            conversation_identity: CONVERSATION,
            content_identity: CONTENT,
            conversation_sequence: sequence,
            kind,
        }
    }

    fn committed(draft: GatewayRecordDraft<'_>) -> GatewayRecord {
        let plan = plan_gateway_admission(draft).expect("valid plan");
        observe_gateway_commit(&plan.record, plan.record.work_identity.as_str()).expect("commit")
    }

    #[test]
    fn inbound_and_reply_effects_require_durable_admission() {
        // r[verify animus.gateway.persist.incoming]
        // r[verify animus.gateway.persist.reply]
        let inbound = plan_gateway_admission(draft(UPDATE, 0, GatewayRecordKind::InboundUpdate))
            .expect("inbound plan");
        assert!(!inbound.effect_admitted);
        assert_eq!(
            plan_gateway_processing(&inbound.record, &[], CAPACITY),
            Err(GatewayError::NotDurable)
        );
        let reply = plan_gateway_admission(draft(REPLY, 0, GatewayRecordKind::ReplyIntent))
            .expect("reply plan");
        assert_eq!(
            plan_gateway_delivery(&reply.record),
            Err(GatewayError::NotDurable)
        );
        assert_eq!(
            plan_gateway_delivery(&committed(draft(REPLY, 0, GatewayRecordKind::ReplyIntent))),
            Ok(REPLY)
        );
    }

    #[test]
    fn same_conversation_order_fails_while_separate_conversations_fit_capacity() {
        // r[verify animus.gateway.order.same]
        // r[verify animus.gateway.order.different]
        let first = committed(draft(UPDATE, 0, GatewayRecordKind::InboundUpdate));
        let second = committed(draft("update-2", 1, GatewayRecordKind::InboundUpdate));
        assert_eq!(
            plan_gateway_processing(&second, core::slice::from_ref(&first), CAPACITY),
            Err(GatewayError::ConversationOrder)
        );
        let mut active = first;
        active.work_state = GatewayWorkState::Processing;
        let mut other = committed(draft("update-b", 0, GatewayRecordKind::InboundUpdate));
        other.conversation_identity = String::from("conversation-b");
        assert_eq!(
            plan_gateway_processing(&other, core::slice::from_ref(&active), 1),
            Err(GatewayError::CapacityExceeded)
        );
        assert!(plan_gateway_processing(&other, &[active], CAPACITY).is_ok());
        assert_eq!(
            plan_gateway_processing(&other, &[other.clone()], 1),
            Ok(GatewayProcessingPlan {
                work_identity: String::from("update-b"),
                conversation_identity: String::from("conversation-b"),
                active_conversation_count: 1,
            })
        );
    }

    #[test]
    fn restart_is_idempotent_and_preserves_delivery_uncertainty() {
        // r[verify animus.gateway.resume.restart]
        // r[verify animus.gateway.resume.duplicate]
        let update = committed(draft(UPDATE, 0, GatewayRecordKind::InboundUpdate));
        let mut reply = committed(draft(REPLY, 0, GatewayRecordKind::ReplyIntent));
        reply.delivery_state = GatewayDeliveryState::Uncertain;
        let actions = recover_gateway_work(&[update.clone(), reply.clone()]).expect("recover");
        assert_eq!(actions.len(), EXPECTED_RECOVERY_ACTIONS);
        assert_eq!(
            actions[1].disposition,
            GatewayRecoveryDisposition::PreserveUncertainReply
        );
        assert_eq!(
            recover_gateway_work(&[update.clone(), update]),
            Err(GatewayError::DuplicateIdentity)
        );
        assert_eq!(
            validate_gateway_delivery_recovery(
                reply.delivery_state,
                GatewayDeliveryState::Delivered
            ),
            Err(GatewayError::FalseDelivery)
        );
    }

    #[test]
    fn mutation_needs_exact_authority_and_transport_authentication_is_insufficient() {
        // r[verify animus.gateway.authority.mutation]
        let request = GatewayToolRequest {
            session_identity: "session",
            remote_user_identity: "user",
            tool_name: "write",
            request_identity: "request",
            read_only: false,
            transport_authenticated: true,
        };
        assert_eq!(
            admit_gateway_tool(request, &[]),
            Err(GatewayError::MutationDenied)
        );
        let authority = GatewayMutationAuthority {
            session_identity: request.session_identity,
            remote_user_identity: request.remote_user_identity,
            tool_name: request.tool_name,
            request_identity: request.request_identity,
            admitted: true,
        };
        assert_eq!(admit_gateway_tool(request, &[authority]), Ok(()));
        assert_eq!(
            admit_gateway_tool(
                GatewayToolRequest {
                    read_only: true,
                    ..request
                },
                &[]
            ),
            Ok(())
        );
    }
}

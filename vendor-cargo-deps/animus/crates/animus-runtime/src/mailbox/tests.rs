use super::*;

const WORK_ID: &str = "work-alpha";
const STALE_WORK_ID: &str = "work-stale";
const USER_SOURCE: &str = "user";
const CONTROLLER_SOURCE: &str = "controller";
const INTRUDER_SOURCE: &str = "intruder";
const ITEM_ONE: &str = "item-one";
const ITEM_TWO: &str = "item-two";
const ITEM_THREE: &str = "item-three";
const UNKNOWN_ITEM: &str = "item-missing";
const POLICY_HASH: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const OTHER_POLICY_HASH: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const BODY_HASH_ONE: &str = "1111111111111111111111111111111111111111111111111111111111111111";
const BODY_HASH_TWO: &str = "2222222222222222222222222222222222222222222222222222222222222222";
const BODY_HASH_THREE: &str = "3333333333333333333333333333333333333333333333333333333333333333";
const BODY_HASH_FOUR: &str = "4444444444444444444444444444444444444444444444444444444444444444";
const TARGET_HASH: &str = "5555555555555555555555555555555555555555555555555555555555555555";
const INSTRUCTION_HASH: &str = "6666666666666666666666666666666666666666666666666666666666666666";
const ACTIVE_WORK_HASH: &str = "7777777777777777777777777777777777777777777777777777777777777777";
const QUEUE_ITEM_HASH: &str = "8888888888888888888888888888888888888888888888888888888888888888";
const QUEUE_CAPACITY: u32 = 3;
const SMALL_QUEUE_CAPACITY: u32 = 1;
const FIRST_ITEM_INDEX: usize = 0;
const SECOND_ITEM_INDEX: usize = 1;
const POST_ACK_ITEM_COUNT: usize = 2;

#[test]
fn queue_reducer_appends_reorders_dispatches_and_acknowledges() {
    let policy = queue_policy(QUEUE_CAPACITY, true, true);
    let state = MessageQueueState::new(WORK_ID).expect("state should validate");
    let state = append(&state, ITEM_ONE, BODY_HASH_ONE, &policy).state;
    let state = append(&state, ITEM_TWO, BODY_HASH_TWO, &policy).state;
    let state = append(&state, ITEM_THREE, BODY_HASH_THREE, &policy).state;

    let transition = apply_message_queue_operation(
        &state,
        MessageQueueOperation::Reorder(MessageReorderDraft {
            item_id: ITEM_THREE,
            before_item_id: Some(ITEM_TWO),
            active_work_id: WORK_ID,
            source: USER_SOURCE,
        }),
        &policy,
    )
    .expect("reorder should succeed");
    assert_eq!(
        transition.state.items()[FIRST_ITEM_INDEX].item_id(),
        ITEM_ONE
    );
    assert_eq!(
        transition.state.items()[SECOND_ITEM_INDEX].item_id(),
        ITEM_THREE
    );
    assert!(matches!(
        transition.event,
        UserControlEvent::QueueReordered(_)
    ));

    let transition = apply_message_queue_operation(
        &transition.state,
        MessageQueueOperation::DispatchNext(scope(CONTROLLER_SOURCE)),
        &policy,
    )
    .expect("dispatch-next should succeed");
    let dispatch = transition
        .state
        .pending_dispatch()
        .expect("dispatch should be pending");
    assert_eq!(dispatch.item_id(), ITEM_ONE);
    assert!(matches!(
        transition.event,
        UserControlEvent::DispatchReady(_)
    ));

    let transition = apply_message_queue_operation(
        &transition.state,
        MessageQueueOperation::Acknowledge(DispatchAcknowledgementDraft {
            item_id: ITEM_ONE,
            active_work_id: WORK_ID,
            outcome: MessageDispatchAcknowledgementOutcome::Accepted,
        }),
        &policy,
    )
    .expect("acknowledgement should succeed");
    assert_eq!(transition.state.items().len(), POST_ACK_ITEM_COUNT);
    assert!(transition.state.pending_dispatch().is_none());
    assert_eq!(
        transition.state.items()[FIRST_ITEM_INDEX].item_id(),
        ITEM_THREE
    );
}

#[test]
fn queue_reducer_edits_deletes_clears_and_cancels() {
    let policy = queue_policy(QUEUE_CAPACITY, true, true);
    let state = MessageQueueState::new(WORK_ID).expect("state should validate");
    let state = append(&state, ITEM_ONE, BODY_HASH_ONE, &policy).state;
    let transition = apply_message_queue_operation(
        &state,
        MessageQueueOperation::Edit(MessageEditDraft {
            item_id: ITEM_ONE,
            active_work_id: WORK_ID,
            source: USER_SOURCE,
            replacement_body_hash: BODY_HASH_TWO,
            safety: MessageSafetyClass::Safe,
            budget: MessageBudgetClass::WithinBudget,
        }),
        &policy,
    )
    .expect("edit should succeed");
    assert_eq!(
        transition.state.items()[FIRST_ITEM_INDEX].body_hash(),
        BODY_HASH_TWO
    );
    assert!(matches!(transition.event, UserControlEvent::QueueEdited(_)));

    let transition = apply_message_queue_operation(
        &transition.state,
        MessageQueueOperation::CancelPending(item_op(ITEM_ONE, USER_SOURCE)),
        &policy,
    )
    .expect("cancel-pending should remove item");
    assert!(transition.state.items().is_empty());
    assert!(matches!(
        transition.event,
        UserControlEvent::PendingCancelled(_)
    ));

    let state = append(&transition.state, ITEM_ONE, BODY_HASH_ONE, &policy).state;
    let state = append(&state, ITEM_TWO, BODY_HASH_TWO, &policy).state;
    let transition = apply_message_queue_operation(
        &state,
        MessageQueueOperation::Delete(item_op(ITEM_ONE, USER_SOURCE)),
        &policy,
    )
    .expect("delete should remove item");
    assert_eq!(transition.state.items().len(), SECOND_ITEM_INDEX);
    assert_eq!(
        transition.state.items()[FIRST_ITEM_INDEX].item_id(),
        ITEM_TWO
    );

    let transition = apply_message_queue_operation(
        &transition.state,
        MessageQueueOperation::Clear(scope(USER_SOURCE)),
        &policy,
    )
    .expect("clear should remove owned items");
    assert!(transition.state.items().is_empty());
    assert_eq!(transition.event.decision_class(), "queue_cleared");
}

#[test]
fn interrupt_classification_returns_explicit_control_events() {
    let policy = queue_policy(QUEUE_CAPACITY, true, true);
    let active = active(
        CancellationHandleClass::Available,
        ActiveWorkStatus::Running,
    );
    let cancellation = classify_interrupt_signal(
        signal(
            InterruptSendMode::Heuristic,
            Some(InterruptKeyword::Stop),
            None,
        ),
        &active,
        &policy,
    )
    .expect("stop keyword should classify as cancellation");
    assert!(matches!(cancellation, UserControlEvent::Cancellation(_)));

    let redirect = classify_interrupt_signal(
        signal(
            InterruptSendMode::Redirect,
            None,
            Some(RedirectionPayloadDraft {
                active_work_id: WORK_ID,
                source: USER_SOURCE,
                target_hash: TARGET_HASH,
                instruction_hash: INSTRUCTION_HASH,
            }),
        ),
        &active,
        &policy,
    )
    .expect("redirect payload should classify");
    assert!(matches!(redirect, UserControlEvent::Redirection(_)));

    let queued = classify_interrupt_signal(
        signal(InterruptSendMode::Queue, None, None),
        &active,
        &policy,
    )
    .expect("queue mode should classify");
    assert!(matches!(queued, UserControlEvent::QueueInterrupt(_)));

    let immediate = classify_interrupt_signal(
        signal(InterruptSendMode::ImmediateDispatch, None, None),
        &active,
        &policy,
    )
    .expect("immediate mode should classify");
    assert!(matches!(immediate, UserControlEvent::ImmediateDispatch(_)));
}

#[test]
fn receipt_validation_uses_safe_hashes_and_redaction_classes() {
    let receipt = MessageQueueReceipt::validate(MessageQueueReceiptDraft {
        operation_kind: MessageQueueOperationKind::Append.as_str(),
        source: USER_SOURCE,
        active_work_hash: ACTIVE_WORK_HASH,
        queue_item_hash: QUEUE_ITEM_HASH,
        decision_class: "queue_admitted",
        cancellation_handle_class: CancellationHandleClass::Available.as_str(),
        budget_class: MessageBudgetClass::WithinBudget.as_str(),
        policy_hash: POLICY_HASH,
        denial_class: None,
        redaction_state: MessageRedactionState::Redacted.as_str(),
    })
    .expect("safe receipt should validate");

    assert_eq!(receipt.operation_kind(), MessageQueueOperationKind::Append);
    assert_eq!(receipt.source(), USER_SOURCE);
    assert_eq!(receipt.active_work_hash(), ACTIVE_WORK_HASH);
    assert_eq!(receipt.queue_item_hash(), QUEUE_ITEM_HASH);
    assert!(!receipt.receipt_hash().is_empty());
}

#[test]
fn queue_reducer_rejects_stale_duplicate_unauthorized_and_overflow_operations() {
    let policy = queue_policy(SMALL_QUEUE_CAPACITY, true, true);
    let state = MessageQueueState::new(WORK_ID).expect("state should validate");
    let state = append(&state, ITEM_ONE, BODY_HASH_ONE, &policy).state;

    let stale_work_error = apply_message_queue_operation(
        &state,
        MessageQueueOperation::Delete(MessageItemOperationDraft {
            active_work_id: STALE_WORK_ID,
            ..item_op(ITEM_ONE, USER_SOURCE)
        }),
        &policy,
    )
    .expect_err("stale work should fail");
    assert_eq!(
        stale_work_error.kind(),
        MessageQueueErrorKind::StaleActiveWork
    );

    let duplicate = apply_message_queue_operation(
        &state,
        MessageQueueOperation::Append(message(ITEM_ONE, BODY_HASH_TWO)),
        &policy,
    )
    .expect_err("duplicate should fail before overflow when identity exists");
    assert_eq!(duplicate.kind(), MessageQueueErrorKind::DuplicateQueueItem);

    let unauthorized = apply_message_queue_operation(
        &state,
        MessageQueueOperation::Delete(item_op(ITEM_ONE, INTRUDER_SOURCE)),
        &policy,
    )
    .expect_err("unauthorized source should fail");
    assert_eq!(
        unauthorized.kind(),
        MessageQueueErrorKind::UnauthorizedSource
    );

    let overflow = apply_message_queue_operation(
        &state,
        MessageQueueOperation::Append(message(ITEM_TWO, BODY_HASH_TWO)),
        &policy,
    )
    .expect_err("overflow should fail closed");
    assert_eq!(overflow.kind(), MessageQueueErrorKind::MaxQueueExceeded);
}

#[test]
fn queue_reducer_rejects_unsafe_budget_policy_and_malformed_acknowledgement() {
    let policy = queue_policy(QUEUE_CAPACITY, true, true);
    let disabled_policy = queue_policy(QUEUE_CAPACITY, false, true);
    let state = MessageQueueState::new(WORK_ID).expect("state should validate");

    let unsafe_body = apply_message_queue_operation(
        &state,
        MessageQueueOperation::Append(QueuedMessageDraft {
            safety: MessageSafetyClass::Unsafe,
            ..message(ITEM_ONE, BODY_HASH_ONE)
        }),
        &policy,
    )
    .expect_err("unsafe body should fail closed");
    assert_eq!(unsafe_body.kind(), MessageQueueErrorKind::UnsafeQueuedBody);

    let over_budget = apply_message_queue_operation(
        &state,
        MessageQueueOperation::Append(QueuedMessageDraft {
            budget: MessageBudgetClass::OverBudget,
            ..message(ITEM_ONE, BODY_HASH_ONE)
        }),
        &policy,
    )
    .expect_err("over-budget body should fail closed");
    assert_eq!(over_budget.kind(), MessageQueueErrorKind::BudgetExceeded);

    let denied = apply_message_queue_operation(
        &state,
        MessageQueueOperation::Append(message(ITEM_ONE, BODY_HASH_ONE)),
        &disabled_policy,
    )
    .expect_err("disabled queue policy should fail");
    assert_eq!(denied.kind(), MessageQueueErrorKind::PolicyDenied);

    let state = append(&state, ITEM_ONE, BODY_HASH_ONE, &policy).state;
    let state = apply_message_queue_operation(
        &state,
        MessageQueueOperation::DispatchNext(scope(CONTROLLER_SOURCE)),
        &policy,
    )
    .expect("dispatch should start")
    .state;
    let malformed = apply_message_queue_operation(
        &state,
        MessageQueueOperation::Acknowledge(DispatchAcknowledgementDraft {
            item_id: UNKNOWN_ITEM,
            active_work_id: WORK_ID,
            outcome: MessageDispatchAcknowledgementOutcome::Accepted,
        }),
        &policy,
    )
    .expect_err("mismatched ack should fail");
    assert_eq!(
        malformed.kind(),
        MessageQueueErrorKind::MalformedAcknowledgement
    );
}

#[test]
fn interrupt_classification_rejects_missing_handles_stale_work_and_bad_redirects() {
    let policy = queue_policy(QUEUE_CAPACITY, true, true);
    let missing_handle = active(CancellationHandleClass::Missing, ActiveWorkStatus::Running);
    let missing = classify_interrupt_signal(
        signal(InterruptSendMode::Cancel, None, None),
        &missing_handle,
        &policy,
    )
    .expect_err("missing handle should fail closed");
    assert_eq!(
        missing.kind(),
        MessageQueueErrorKind::MissingCancellationHandle
    );

    let completed = active(
        CancellationHandleClass::Available,
        ActiveWorkStatus::Completed,
    );
    let completed_work_error = classify_interrupt_signal(
        signal(InterruptSendMode::Pause, None, None),
        &completed,
        &policy,
    )
    .expect_err("completed active work should reject controls");
    assert_eq!(
        completed_work_error.kind(),
        MessageQueueErrorKind::UnsupportedActiveWorkState
    );

    let active = active(
        CancellationHandleClass::Available,
        ActiveWorkStatus::Running,
    );
    let stale = classify_interrupt_signal(
        InterruptSignalDraft {
            active_work_id: STALE_WORK_ID,
            ..signal(InterruptSendMode::Queue, None, None)
        },
        &active,
        &policy,
    )
    .expect_err("stale signal should fail");
    assert_eq!(stale.kind(), MessageQueueErrorKind::StaleActiveWork);

    let malformed = classify_interrupt_signal(
        signal(InterruptSendMode::Redirect, None, None),
        &active,
        &policy,
    )
    .expect_err("redirect mode requires payload");
    assert_eq!(malformed.kind(), MessageQueueErrorKind::MalformedInterrupt);
}

#[test]
fn receipts_reject_raw_prompt_and_path_leaks() {
    let prompt_leak = MessageQueueReceipt::validate(MessageQueueReceiptDraft {
        decision_class: "raw_prompt: do the thing",
        operation_kind: MessageQueueOperationKind::Append.as_str(),
        source: USER_SOURCE,
        active_work_hash: ACTIVE_WORK_HASH,
        queue_item_hash: QUEUE_ITEM_HASH,
        cancellation_handle_class: CancellationHandleClass::Available.as_str(),
        budget_class: MessageBudgetClass::WithinBudget.as_str(),
        policy_hash: POLICY_HASH,
        denial_class: None,
        redaction_state: MessageRedactionState::Redacted.as_str(),
    })
    .expect_err("raw prompt should be rejected");
    assert_eq!(prompt_leak.kind(), MessageQueueErrorKind::ReceiptLeak);

    let path_leak = validate_message_queue_receipt_text("artifact at /home/user/secret")
        .expect_err("full paths should be rejected");
    assert_eq!(path_leak.kind(), MessageQueueErrorKind::ReceiptLeak);
}

#[test]
fn queue_admission_rejects_policy_hash_mismatch_and_unknown_item() {
    let policy = queue_policy(QUEUE_CAPACITY, true, true);
    let state = MessageQueueState::new(WORK_ID).expect("state should validate");
    let mismatch = apply_message_queue_operation(
        &state,
        MessageQueueOperation::Append(QueuedMessageDraft {
            policy_hash: OTHER_POLICY_HASH,
            ..message(ITEM_ONE, BODY_HASH_ONE)
        }),
        &policy,
    )
    .expect_err("policy hash mismatch should fail");
    assert_eq!(mismatch.kind(), MessageQueueErrorKind::PolicyDenied);

    let unknown = apply_message_queue_operation(
        &state,
        MessageQueueOperation::Delete(item_op(UNKNOWN_ITEM, USER_SOURCE)),
        &policy,
    )
    .expect_err("unknown item should fail");
    assert_eq!(unknown.kind(), MessageQueueErrorKind::UnknownQueueItem);
}

fn queue_policy(
    max_items: u32,
    allow_queue_mutations: bool,
    allow_interrupts: bool,
) -> MessageQueuePolicy {
    MessageQueuePolicy::new(MessageQueuePolicyDraft {
        max_items,
        allowed_sources: &[USER_SOURCE, CONTROLLER_SOURCE],
        policy_label: DEFAULT_QUEUE_POLICY_LABEL,
        policy_hash: POLICY_HASH,
        allow_queue_mutations,
        allow_interrupts,
    })
    .expect("policy should validate")
}

fn active(
    cancellation_handle: CancellationHandleClass,
    status: ActiveWorkStatus,
) -> ActiveWorkReference {
    ActiveWorkReference::new(WORK_ID, status, cancellation_handle)
        .expect("active work should validate")
}

fn append(
    state: &MessageQueueState,
    item_id: &'static str,
    body_hash: &'static str,
    policy: &MessageQueuePolicy,
) -> MessageQueueTransition {
    apply_message_queue_operation(
        state,
        MessageQueueOperation::Append(message(item_id, body_hash)),
        policy,
    )
    .expect("append should succeed")
}

fn message(item_id: &'static str, body_hash: &'static str) -> QueuedMessageDraft<'static> {
    QueuedMessageDraft {
        item_id,
        active_work_id: WORK_ID,
        source: USER_SOURCE,
        body_hash,
        safety: MessageSafetyClass::Safe,
        budget: MessageBudgetClass::WithinBudget,
        policy_hash: POLICY_HASH,
    }
}

fn item_op(item_id: &'static str, source: &'static str) -> MessageItemOperationDraft<'static> {
    MessageItemOperationDraft {
        item_id,
        active_work_id: WORK_ID,
        source,
    }
}

fn scope(source: &'static str) -> MessageScopeOperationDraft<'static> {
    MessageScopeOperationDraft {
        active_work_id: WORK_ID,
        source,
    }
}

fn signal(
    send_mode: InterruptSendMode,
    keyword: Option<InterruptKeyword>,
    redirection: Option<RedirectionPayloadDraft<'static>>,
) -> InterruptSignalDraft<'static> {
    InterruptSignalDraft {
        active_work_id: WORK_ID,
        source: USER_SOURCE,
        message_hash: BODY_HASH_FOUR,
        send_mode,
        keyword,
        redirection,
    }
}

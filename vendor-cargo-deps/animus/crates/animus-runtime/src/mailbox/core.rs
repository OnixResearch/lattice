#[derive(Clone, Copy)]
enum InterruptIntent {
    Queue,
    ImmediateDispatch,
    Pause,
    Cancel,
    Redirect,
}

// r[impl agent_harness_runtime.goose_message_queue.reducer]
/// Applies one pure queue operation to in-memory queue state.
///
/// No model, tool, ledger, host cancellation, storage, filesystem, network,
/// process, or clock authority is used by this reducer.
///
/// # Errors
///
/// Returns [`super::MessageQueueError`] when active-work identity, policy,
/// authorization, budget, safety, queue bounds, item identity, or acknowledgement
/// validation fails.
pub fn apply_message_queue_operation(
    state: &super::MessageQueueState,
    operation: super::MessageQueueOperation<'_>,
    policy: &super::MessageQueuePolicy,
) -> Result<super::MessageQueueTransition, super::MessageQueueError> {
    ensure_queue_policy_allows(policy)?;
    match operation {
        super::MessageQueueOperation::Append(draft) => append_message(state, draft, policy),
        super::MessageQueueOperation::Edit(draft) => edit_message(state, draft, policy),
        super::MessageQueueOperation::Reorder(draft) => reorder_message(state, draft, policy),
        super::MessageQueueOperation::Delete(draft) => delete_message(state, draft, policy),
        super::MessageQueueOperation::Clear(draft) => clear_source_messages(state, draft, policy),
        super::MessageQueueOperation::DispatchNext(draft) => dispatch_next(state, draft, policy),
        super::MessageQueueOperation::Acknowledge(draft) => acknowledge_dispatch(state, draft),
        super::MessageQueueOperation::CancelPending(draft) => cancel_pending(state, draft, policy),
    }
}

// r[impl agent_harness_runtime.goose_message_queue.interrupts]
/// Classifies an interrupt signal into an explicit user-control event.
///
/// No text is made model-visible by this function. Queue and immediate-dispatch
/// outcomes carry only message hashes; host shells own eventual dispatch.
///
/// # Errors
///
/// Returns [`super::MessageQueueError`] when policy denies interrupts, source or
/// active-work identity is stale, the active-work state cannot accept control,
/// cancellation is unavailable, or redirect metadata is missing/unsafe.
pub fn classify_interrupt_signal(
    signal: super::InterruptSignalDraft<'_>,
    active_work: &super::ActiveWorkReference,
    policy: &super::MessageQueuePolicy,
) -> Result<super::UserControlEvent, super::MessageQueueError> {
    debug_assert!(!active_work.work_id().is_empty());
    debug_assert!(!policy.policy_hash().is_empty());
    ensure_interrupt_policy_allows(policy)?;
    ensure_active_work_matches(active_work.work_id(), signal.active_work_id)?;
    ensure_source_allowed(policy, signal.source)?;
    super::data::validate_hash_metadata(super::FIELD_BODY_HASH, signal.message_hash)?;
    let intent = interrupt_intent(signal);
    ensure_live_control_state(active_work.status())?;
    match intent {
        InterruptIntent::Queue => Ok(super::UserControlEvent::QueueInterrupt(
            super::InterruptQueueAdmission::new(
                signal.active_work_id,
                signal.source,
                signal.message_hash,
            )?,
        )),
        InterruptIntent::ImmediateDispatch => Ok(super::UserControlEvent::ImmediateDispatch(
            super::ImmediateDispatchRequest::new(
                signal.active_work_id,
                signal.source,
                signal.message_hash,
            )?,
        )),
        InterruptIntent::Pause => Ok(super::UserControlEvent::Pause(super::PauseRequest::new(
            signal.active_work_id,
            signal.source,
        )?)),
        InterruptIntent::Cancel => cancel_event(signal, active_work),
        InterruptIntent::Redirect => redirect_event(signal),
    }
}

fn append_message(
    state: &super::MessageQueueState,
    draft: super::QueuedMessageDraft<'_>,
    policy: &super::MessageQueuePolicy,
) -> Result<super::MessageQueueTransition, super::MessageQueueError> {
    ensure_active_work_matches(state.active_work_id(), draft.active_work_id)?;
    ensure_source_allowed(policy, draft.source)?;
    let message = super::QueuedMessage::new(draft)?;
    ensure_policy_hash_matches(message.policy_hash(), policy.policy_hash())?;
    ensure_unique_item(state, message.item_id())?;
    ensure_queue_has_capacity(state, policy)?;
    let mut next = state.clone();
    let mut items = next.items().to_vec();
    items.push(message.clone());
    next.replace_items(items);
    Ok(super::MessageQueueTransition {
        state: next,
        event: super::UserControlEvent::QueueAdmitted(super::QueueItemEvent::from_message(
            &message,
        )),
    })
}

fn edit_message(
    state: &super::MessageQueueState,
    draft: super::MessageEditDraft<'_>,
    policy: &super::MessageQueuePolicy,
) -> Result<super::MessageQueueTransition, super::MessageQueueError> {
    debug_assert!(!state.active_work_id().is_empty());
    debug_assert!(!policy.policy_hash().is_empty());
    ensure_active_work_matches(state.active_work_id(), draft.active_work_id)?;
    ensure_source_allowed(policy, draft.source)?;
    let index = find_item_index(state, draft.item_id)?;
    let Some(current) = state.items().get(index) else {
        return Err(unknown_item_error(draft.item_id));
    };
    ensure_item_source_matches(current, draft.source)?;
    let replacement = current.with_replacement(draft)?;
    let mut items = state.items().to_vec();
    items[index] = replacement.clone();
    let mut next = state.clone();
    next.replace_items(items);
    Ok(super::MessageQueueTransition {
        state: next,
        event: super::UserControlEvent::QueueEdited(super::QueueItemEvent::from_message(
            &replacement,
        )),
    })
}

fn reorder_message(
    state: &super::MessageQueueState,
    draft: super::MessageReorderDraft<'_>,
    policy: &super::MessageQueuePolicy,
) -> Result<super::MessageQueueTransition, super::MessageQueueError> {
    debug_assert!(!state.active_work_id().is_empty());
    debug_assert!(!policy.policy_hash().is_empty());
    ensure_active_work_matches(state.active_work_id(), draft.active_work_id)?;
    ensure_source_allowed(policy, draft.source)?;
    let index = find_item_index(state, draft.item_id)?;
    let Some(current) = state.items().get(index) else {
        return Err(unknown_item_error(draft.item_id));
    };
    ensure_item_source_matches(current, draft.source)?;
    if draft.before_item_id == Some(draft.item_id) {
        return Ok(unchanged_reorder(state, current));
    }
    let mut items = state.items().to_vec();
    let item = items.remove(index);
    let insertion_index = insertion_index(items.as_slice(), draft.before_item_id)?;
    items.insert(insertion_index, item.clone());
    let mut next = state.clone();
    next.replace_items(items);
    Ok(super::MessageQueueTransition {
        state: next,
        event: super::UserControlEvent::QueueReordered(super::QueueItemEvent::from_message(&item)),
    })
}

fn delete_message(
    state: &super::MessageQueueState,
    draft: super::MessageItemOperationDraft<'_>,
    policy: &super::MessageQueuePolicy,
) -> Result<super::MessageQueueTransition, super::MessageQueueError> {
    debug_assert!(!state.active_work_id().is_empty());
    debug_assert!(!policy.policy_hash().is_empty());
    ensure_active_work_matches(state.active_work_id(), draft.active_work_id)?;
    ensure_source_allowed(policy, draft.source)?;
    let index = find_item_index(state, draft.item_id)?;
    let Some(current) = state.items().get(index) else {
        return Err(unknown_item_error(draft.item_id));
    };
    ensure_item_source_matches(current, draft.source)?;
    let mut items = state.items().to_vec();
    let removed = items.remove(index);
    let mut next = state.clone();
    next.replace_items(items);
    clear_pending_if_matches(&mut next, removed.item_id());
    Ok(super::MessageQueueTransition {
        state: next,
        event: super::UserControlEvent::QueueDeleted(super::QueueItemEvent::from_message(&removed)),
    })
}

fn clear_source_messages(
    state: &super::MessageQueueState,
    draft: super::MessageScopeOperationDraft<'_>,
    policy: &super::MessageQueuePolicy,
) -> Result<super::MessageQueueTransition, super::MessageQueueError> {
    debug_assert!(!state.active_work_id().is_empty());
    debug_assert!(!policy.policy_hash().is_empty());
    ensure_active_work_matches(state.active_work_id(), draft.active_work_id)?;
    ensure_source_allowed(policy, draft.source)?;
    let source = super::data::normalize_safe_metadata(super::FIELD_SOURCE, draft.source)?;
    let mut removed_count = 0_u32;
    let mut retained = Vec::with_capacity(state.items().len());
    for item in state.items() {
        let should_retain = item.source() != source;
        if should_retain {
            retained.push(item.clone());
        } else {
            removed_count = increment_removed_count(removed_count)?;
        }
    }
    let mut next = state.clone();
    next.replace_items(retained);
    if next
        .pending_dispatch()
        .is_some_and(|pending| pending.source() == source)
    {
        next.set_pending_dispatch(None);
    }
    Ok(super::MessageQueueTransition {
        state: next,
        event: super::UserControlEvent::QueueCleared {
            source,
            removed_count,
        },
    })
}

fn dispatch_next(
    state: &super::MessageQueueState,
    draft: super::MessageScopeOperationDraft<'_>,
    policy: &super::MessageQueuePolicy,
) -> Result<super::MessageQueueTransition, super::MessageQueueError> {
    debug_assert!(!state.active_work_id().is_empty());
    debug_assert!(!policy.policy_hash().is_empty());
    ensure_active_work_matches(state.active_work_id(), draft.active_work_id)?;
    ensure_source_allowed(policy, draft.source)?;
    if state.pending_dispatch().is_some() {
        return Err(super::MessageQueueError::new(
            super::MessageQueueErrorKind::PendingDispatchAlreadyExists,
            super::FIELD_ITEM_ID,
            "dispatch acknowledgement is still pending",
        ));
    }
    let Some(message) = state.items().first() else {
        return Err(super::MessageQueueError::new(
            super::MessageQueueErrorKind::UnknownQueueItem,
            super::FIELD_ITEM_ID,
            "queue has no dispatchable messages",
        ));
    };
    let dispatch = super::MessageDispatch::from_message(message);
    let mut next = state.clone();
    next.set_pending_dispatch(Some(dispatch.clone()));
    Ok(super::MessageQueueTransition {
        state: next,
        event: super::UserControlEvent::DispatchReady(dispatch),
    })
}

fn acknowledge_dispatch(
    state: &super::MessageQueueState,
    draft: super::DispatchAcknowledgementDraft<'_>,
) -> Result<super::MessageQueueTransition, super::MessageQueueError> {
    debug_assert!(!state.active_work_id().is_empty());
    debug_assert!(
        state
            .items()
            .iter()
            .all(|item| item.active_work_id() == state.active_work_id())
    );
    ensure_active_work_matches(state.active_work_id(), draft.active_work_id)?;
    let pending = state
        .pending_dispatch()
        .ok_or_else(|| {
            super::MessageQueueError::new(
                super::MessageQueueErrorKind::MissingPendingDispatch,
                super::FIELD_ITEM_ID,
                "acknowledgement arrived without pending dispatch",
            )
        })?
        .clone();
    let item_id = super::data::normalize_safe_metadata(super::FIELD_ITEM_ID, draft.item_id)?;
    if pending.item_id() != item_id {
        return Err(super::MessageQueueError::new(
            super::MessageQueueErrorKind::MalformedAcknowledgement,
            super::FIELD_ITEM_ID,
            "acknowledgement item did not match pending dispatch",
        ));
    }
    let mut next = state.clone();
    next.set_pending_dispatch(None);
    if draft.outcome == super::MessageDispatchAcknowledgementOutcome::Accepted {
        next.replace_items(remove_item(state.items(), pending.item_id())?);
    }
    Ok(super::MessageQueueTransition {
        state: next,
        event: super::UserControlEvent::DispatchAcknowledged {
            item_id,
            outcome: draft.outcome,
        },
    })
}

fn cancel_pending(
    state: &super::MessageQueueState,
    draft: super::MessageItemOperationDraft<'_>,
    policy: &super::MessageQueuePolicy,
) -> Result<super::MessageQueueTransition, super::MessageQueueError> {
    debug_assert!(!state.active_work_id().is_empty());
    debug_assert!(!policy.policy_hash().is_empty());
    ensure_active_work_matches(state.active_work_id(), draft.active_work_id)?;
    ensure_source_allowed(policy, draft.source)?;
    let index = find_item_index(state, draft.item_id)?;
    let Some(current) = state.items().get(index) else {
        return Err(unknown_item_error(draft.item_id));
    };
    ensure_item_source_matches(current, draft.source)?;
    let mut items = state.items().to_vec();
    let removed = items.remove(index);
    let mut next = state.clone();
    next.replace_items(items);
    clear_pending_if_matches(&mut next, removed.item_id());
    Ok(super::MessageQueueTransition {
        state: next,
        event: super::UserControlEvent::PendingCancelled(super::QueueItemEvent::from_message(
            &removed,
        )),
    })
}

fn unchanged_reorder(
    state: &super::MessageQueueState,
    item: &super::QueuedMessage,
) -> super::MessageQueueTransition {
    super::MessageQueueTransition {
        state: state.clone(),
        event: super::UserControlEvent::QueueReordered(super::QueueItemEvent::from_message(item)),
    }
}

fn remove_item(
    items: &[super::QueuedMessage],
    item_id: &str,
) -> Result<Vec<super::QueuedMessage>, super::MessageQueueError> {
    let index = items
        .iter()
        .position(|item| item.item_id() == item_id)
        .ok_or_else(|| unknown_item_error(item_id))?;
    let mut retained = items.to_vec();
    retained.remove(index);
    Ok(retained)
}

fn clear_pending_if_matches(state: &mut super::MessageQueueState, item_id: &str) {
    if state
        .pending_dispatch()
        .is_some_and(|pending| pending.item_id() == item_id)
    {
        state.set_pending_dispatch(None);
    }
}

fn insertion_index(
    items: &[super::QueuedMessage],
    before_item_id: Option<&str>,
) -> Result<usize, super::MessageQueueError> {
    match before_item_id {
        Some(target) => items
            .iter()
            .position(|item| item.item_id() == target)
            .ok_or_else(|| unknown_item_error(target)),
        None => Ok(items.len()),
    }
}

fn find_item_index(
    state: &super::MessageQueueState,
    item_id: &str,
) -> Result<usize, super::MessageQueueError> {
    let item_id = super::data::normalize_safe_metadata(super::FIELD_ITEM_ID, item_id)?;
    state
        .items()
        .iter()
        .position(|item| item.item_id() == item_id)
        .ok_or_else(|| unknown_item_error(item_id.as_str()))
}

fn ensure_queue_policy_allows(
    policy: &super::MessageQueuePolicy,
) -> Result<(), super::MessageQueueError> {
    if policy.allow_queue_mutations() {
        Ok(())
    } else {
        Err(super::MessageQueueError::new(
            super::MessageQueueErrorKind::PolicyDenied,
            super::FIELD_POLICY_LABEL,
            "queue mutations are disabled by policy",
        ))
    }
}

fn ensure_interrupt_policy_allows(
    policy: &super::MessageQueuePolicy,
) -> Result<(), super::MessageQueueError> {
    if policy.allow_interrupts() {
        Ok(())
    } else {
        Err(super::MessageQueueError::new(
            super::MessageQueueErrorKind::PolicyDenied,
            super::FIELD_POLICY_LABEL,
            "interrupt classification is disabled by policy",
        ))
    }
}

fn ensure_active_work_matches(
    expected: &str,
    provided: &str,
) -> Result<(), super::MessageQueueError> {
    let provided = super::data::normalize_safe_metadata(super::FIELD_ACTIVE_WORK_ID, provided)?;
    if expected == provided {
        Ok(())
    } else {
        Err(super::MessageQueueError::new(
            super::MessageQueueErrorKind::StaleActiveWork,
            super::FIELD_ACTIVE_WORK_ID,
            "operation targeted a stale active-work identity",
        ))
    }
}

fn ensure_source_allowed(
    policy: &super::MessageQueuePolicy,
    source: &str,
) -> Result<(), super::MessageQueueError> {
    let source = super::data::normalize_safe_metadata(super::FIELD_SOURCE, source)?;
    if policy.admits_source(source.as_str()) {
        Ok(())
    } else {
        Err(super::MessageQueueError::new(
            super::MessageQueueErrorKind::UnauthorizedSource,
            super::FIELD_SOURCE,
            "source is not admitted by queue policy",
        ))
    }
}

fn ensure_queue_has_capacity(
    state: &super::MessageQueueState,
    policy: &super::MessageQueuePolicy,
) -> Result<(), super::MessageQueueError> {
    let item_count = u32::try_from(state.items().len()).map_err(|_| {
        super::MessageQueueError::new(
            super::MessageQueueErrorKind::MaxQueueExceeded,
            super::FIELD_ITEM_ID,
            "queue item count exceeded portable bound",
        )
    })?;
    if item_count < policy.max_items() {
        Ok(())
    } else {
        Err(super::MessageQueueError::new(
            super::MessageQueueErrorKind::MaxQueueExceeded,
            super::FIELD_ITEM_ID,
            "queue is at configured capacity",
        ))
    }
}

fn increment_removed_count(current: u32) -> Result<u32, super::MessageQueueError> {
    current.checked_add(1).ok_or_else(|| {
        super::MessageQueueError::new(
            super::MessageQueueErrorKind::MaxQueueExceeded,
            super::FIELD_ITEM_ID,
            "removed queue item count exceeded portable bound",
        )
    })
}

fn ensure_unique_item(
    state: &super::MessageQueueState,
    item_id: &str,
) -> Result<(), super::MessageQueueError> {
    if state.items().iter().any(|item| item.item_id() == item_id) {
        Err(super::MessageQueueError::new(
            super::MessageQueueErrorKind::DuplicateQueueItem,
            super::FIELD_ITEM_ID,
            "queue item identity is already present",
        ))
    } else {
        Ok(())
    }
}

fn ensure_item_source_matches(
    item: &super::QueuedMessage,
    source: &str,
) -> Result<(), super::MessageQueueError> {
    let source = super::data::normalize_safe_metadata(super::FIELD_SOURCE, source)?;
    if item.source() == source {
        Ok(())
    } else {
        Err(super::MessageQueueError::new(
            super::MessageQueueErrorKind::UnauthorizedSource,
            super::FIELD_SOURCE,
            "source cannot mutate an item owned by another source",
        ))
    }
}

fn ensure_policy_hash_matches(
    provided: &str,
    expected: &str,
) -> Result<(), super::MessageQueueError> {
    if provided == expected {
        Ok(())
    } else {
        Err(super::MessageQueueError::new(
            super::MessageQueueErrorKind::PolicyDenied,
            super::FIELD_POLICY_HASH,
            "queued item was admitted under a different policy hash",
        ))
    }
}

fn ensure_live_control_state(
    status: super::ActiveWorkStatus,
) -> Result<(), super::MessageQueueError> {
    if status.accepts_live_controls() {
        Ok(())
    } else {
        Err(super::MessageQueueError::new(
            super::MessageQueueErrorKind::UnsupportedActiveWorkState,
            super::FIELD_ACTIVE_WORK_ID,
            "active work cannot accept live control events",
        ))
    }
}

fn interrupt_intent(signal: super::InterruptSignalDraft<'_>) -> InterruptIntent {
    match signal.send_mode {
        super::InterruptSendMode::Queue => InterruptIntent::Queue,
        super::InterruptSendMode::ImmediateDispatch => InterruptIntent::ImmediateDispatch,
        super::InterruptSendMode::Pause => InterruptIntent::Pause,
        super::InterruptSendMode::Cancel => InterruptIntent::Cancel,
        super::InterruptSendMode::Redirect => InterruptIntent::Redirect,
        super::InterruptSendMode::Heuristic => heuristic_intent(signal.keyword),
    }
}

fn heuristic_intent(keyword: Option<super::InterruptKeyword>) -> InterruptIntent {
    match keyword {
        Some(super::InterruptKeyword::Stop) => InterruptIntent::Cancel,
        Some(super::InterruptKeyword::Wait | super::InterruptKeyword::Pause) => {
            InterruptIntent::Pause
        }
        Some(super::InterruptKeyword::Redirect) => InterruptIntent::Redirect,
        Some(super::InterruptKeyword::Continue) => InterruptIntent::ImmediateDispatch,
        None => InterruptIntent::Queue,
    }
}

fn cancel_event(
    signal: super::InterruptSignalDraft<'_>,
    active_work: &super::ActiveWorkReference,
) -> Result<super::UserControlEvent, super::MessageQueueError> {
    if active_work.cancellation_handle() != super::CancellationHandleClass::Available {
        return Err(super::MessageQueueError::new(
            super::MessageQueueErrorKind::MissingCancellationHandle,
            super::FIELD_CANCELLATION_HANDLE_CLASS,
            "cancellation requested without an available host handle",
        ));
    }
    Ok(super::UserControlEvent::Cancellation(
        super::CancellationRequest::new(signal.active_work_id, signal.source)?,
    ))
}

fn redirect_event(
    signal: super::InterruptSignalDraft<'_>,
) -> Result<super::UserControlEvent, super::MessageQueueError> {
    debug_assert!(!signal.active_work_id.is_empty());
    debug_assert!(!signal.source.is_empty());
    let draft = signal.redirection.ok_or_else(|| {
        super::MessageQueueError::new(
            super::MessageQueueErrorKind::MalformedInterrupt,
            super::FIELD_REDIRECTION_HASH,
            "redirect mode requires redirection payload hashes",
        )
    })?;
    ensure_active_work_matches(signal.active_work_id, draft.active_work_id)?;
    let signal_source = super::data::normalize_safe_metadata(super::FIELD_SOURCE, signal.source)?;
    let redirect_source = super::data::normalize_safe_metadata(super::FIELD_SOURCE, draft.source)?;
    if signal_source != redirect_source {
        return Err(super::MessageQueueError::new(
            super::MessageQueueErrorKind::UnauthorizedSource,
            super::FIELD_SOURCE,
            "redirect payload source did not match signal source",
        ));
    }
    Ok(super::UserControlEvent::Redirection(
        super::RedirectionPayload::new(draft)?,
    ))
}

fn unknown_item_error(item_id: &str) -> super::MessageQueueError {
    super::MessageQueueError::new(
        super::MessageQueueErrorKind::UnknownQueueItem,
        super::FIELD_ITEM_ID,
        format!("queue item was not found: {item_id}"),
    )
}

#[test]
// r[verify agent_harness_runtime.authoritative_completion.continuation]
// r[verify agent_harness_runtime.authoritative_completion.continuation.empty]
fn empty_queue_waits_when_authoritative_completion_is_required() {
    let policy = ContinuationPolicy::new_with_authoritative_completion(
        "receipt-authoritative",
        ContinuationPriority::default(),
    )
    .expect("authoritative policy should validate");
    let decision = select_next_continuation(&ContinuationState::new(), &policy);
    let receipt = ContinuationReceipt::from_decision(
        ContinuationStateClass::Idle,
        &decision,
        &policy,
        None,
    )
    .expect("awaiting-evidence receipt should validate");

    assert!(matches!(
        decision,
        ContinuationDecision::AwaitingCompletionEvidence {
            reason: ContinuationReason::CompletionEvidenceRequired
        }
    ));
    assert_eq!(
        receipt.selected_action(),
        ContinuationAction::AwaitCompletionEvidence
    );
    assert_ne!(policy.policy_hash(), default_policy().policy_hash());
}

#[test]
fn complete_decision_uses_stable_no_work_hash() {
    let policy = default_policy();
    let decision = select_next_continuation(&ContinuationState::new(), &policy);
    let receipt =
        ContinuationReceipt::from_decision(ContinuationStateClass::Idle, &decision, &policy, None)
            .expect("receipt should validate");

    assert_eq!(decision.action(), ContinuationAction::Complete);
    assert_eq!(receipt.selected_action(), ContinuationAction::Complete);
    assert_eq!(
        receipt.pending_work_id_hash(),
        decision.pending_work_id_hash()
    );
}

#[test]
fn duplicate_priority_candidates_are_rejected() {
    let error = ContinuationPriority::new([
        ContinuationCandidate::QueuedPrompt,
        ContinuationCandidate::QueuedPrompt,
        ContinuationCandidate::ValidationFollowUp,
    ])
    .expect_err("duplicate priority should fail");

    assert_eq!(
        error,
        ContinuationError::DuplicatePriority {
            candidate: ContinuationCandidate::QueuedPrompt,
        }
    );
}

#[test]
// r[verify agent_harness_runtime.authoritative_completion.continuation.work_id]
fn missing_dispatch_acknowledgement_work_id_preserves_state() {
    let state = state_with_all_candidates();
    let decision = select_next_continuation(&state, &default_policy());
    let staged = state
        .begin_dispatch(&decision)
        .expect("dispatch should stage");
    let original = staged.clone();
    let error = staged
        .acknowledge_dispatch(&DispatchAcknowledgement {
            work_id: "",
            outcome: DispatchAcknowledgementOutcome::Accepted,
        })
        .expect_err("missing acknowledgement ID should fail");

    assert_eq!(
        error,
        ContinuationError::MissingWorkId {
            field: FIELD_ACK_WORK_ID,
        }
    );
    assert_eq!(
        original
            .pending_dispatch()
            .expect("dispatch should remain")
            .work_id(),
        QUEUED_WORK_ID
    );
}

#[test]
fn mismatched_dispatch_acknowledgement_preserves_pending_state() {
    let state = state_with_all_candidates();
    let decision = select_next_continuation(&state, &default_policy());
    let staged = state
        .begin_dispatch(&decision)
        .expect("dispatch should stage");
    let original = staged.clone();
    let error = staged
        .acknowledge_dispatch(&DispatchAcknowledgement {
            work_id: UNKNOWN_WORK_ID,
            outcome: DispatchAcknowledgementOutcome::Accepted,
        })
        .expect_err("mismatched acknowledgement ID should fail");

    assert!(matches!(error, ContinuationError::MismatchedWork { .. }));
    assert_eq!(
        original
            .pending_dispatch()
            .expect("dispatch should remain")
            .work_id(),
        QUEUED_WORK_ID
    );
}

#[test]
fn stale_dispatch_decision_is_rejected_before_state_changes() {
    // r[verify agent_harness_runtime.continuation_policy.acknowledgements.stale]
    let decision = ContinuationDecision::Dispatch(ContinuationDispatch {
        work_id: Text::from(UNKNOWN_WORK_ID),
        action: ContinuationAction::ReplayQueuedPrompt,
        reason: ContinuationReason::QueuedUserInput,
    });
    let state = state_with_all_candidates();
    let original = state.clone();
    let error = state
        .begin_dispatch(&decision)
        .expect_err("stale decision should fail");

    assert!(matches!(error, ContinuationError::StaleWork { .. }));
    assert_eq!(
        original.queued_prompt().expect("queued work").work_id(),
        QUEUED_WORK_ID
    );
}

#[test]
fn duplicate_completion_acknowledgement_is_rejected() {
    let state = ContinuationState::new()
        .with_busy_prompt(DUPLICATE_WORK_ID)
        .expect("busy work should validate")
        .acknowledge_completion(&CompletionAcknowledgement {
            work_id: DUPLICATE_WORK_ID,
            action: ContinuationAction::ContinueLoop,
        })
        .expect("first completion should acknowledge");
    let completed_hash_count = state.completed_work_hashes().len();
    let error = state
        .acknowledge_completion(&CompletionAcknowledgement {
            work_id: DUPLICATE_WORK_ID,
            action: ContinuationAction::ContinueLoop,
        })
        .expect_err("duplicate completion should fail");

    assert!(matches!(
        error,
        ContinuationError::DuplicateCompletion { .. }
    ));
    assert_eq!(completed_hash_count, EXPECTED_COMPLETED_HASH_COUNT);
}

#[test]
fn mismatched_completion_acknowledgement_preserves_busy_state() {
    let state = ContinuationState::new()
        .with_busy_prompt(BUSY_WORK_ID)
        .expect("busy work should validate");
    let original = state.clone();
    let error = state
        .acknowledge_completion(&CompletionAcknowledgement {
            work_id: UNKNOWN_WORK_ID,
            action: ContinuationAction::ContinueLoop,
        })
        .expect_err("mismatched completion should fail");

    assert!(matches!(error, ContinuationError::MismatchedWork { .. }));
    assert_eq!(
        original
            .busy_prompt()
            .expect("busy work should remain")
            .work_id(),
        BUSY_WORK_ID
    );
}

#[test]
fn rejected_dispatch_acknowledgement_keeps_candidate_pending() {
    let state = state_with_all_candidates();
    let decision = select_next_continuation(&state, &default_policy());
    let state = state
        .begin_dispatch(&decision)
        .expect("dispatch should stage")
        .acknowledge_dispatch(&DispatchAcknowledgement {
            work_id: QUEUED_WORK_ID,
            outcome: DispatchAcknowledgementOutcome::Rejected,
        })
        .expect("rejection should acknowledge without clearing candidate");

    assert!(state.pending_dispatch().is_none());
    assert_eq!(
        state
            .queued_prompt()
            .expect("queued work should remain")
            .work_id(),
        QUEUED_WORK_ID
    );
}

#[test]
fn receipt_metadata_rejects_leak_fixtures() {
    // r[verify agent_harness_runtime.continuation_policy.receipts.rejects_leaks]
    let policy_hash = default_policy().policy_hash();
    let work_hash = blake3_hex(QUEUED_WORK_ID.as_bytes());
    let fixtures = [
        RAW_PROMPT_LEAK,
        MODEL_OUTPUT_LEAK,
        TOOL_OUTPUT_LEAK,
        PROVIDER_JSON_LEAK,
        TRANSCRIPT_LEAK,
        UNBOUNDED_LOG_LEAK,
    ];

    for fixture in fixtures {
        let error = ContinuationReceipt::validate(&ContinuationReceiptDraft {
            reason: fixture,
            ..valid_receipt_draft(work_hash.as_str(), policy_hash.as_str())
        })
        .expect_err("leak fixture should fail");

        assert!(matches!(error, ContinuationError::UnsafeMetadata { .. }));
        assert!(!fixture.is_empty());
    }
}

#[test]
fn receipt_rejects_raw_work_id_in_hash_field() {
    let policy_hash = default_policy().policy_hash();
    let error = ContinuationReceipt::validate(&ContinuationReceiptDraft {
        pending_work_id_hash: RAW_PROMPT_LEAK,
        ..valid_receipt_draft(QUEUED_WORK_ID, policy_hash.as_str())
    })
    .expect_err("raw work ID field should fail");

    assert!(matches!(error, ContinuationError::UnsafeMetadata { .. }));
}

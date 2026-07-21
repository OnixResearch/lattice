const QUEUED_WORK_ID: &str = "queued-work";
const LOOP_WORK_ID: &str = "loop-work";
const VALIDATION_WORK_ID: &str = "validation-work";
const BUSY_WORK_ID: &str = "busy-work";
const CANCELLATION_WORK_ID: &str = "cancel-work";
const UNKNOWN_WORK_ID: &str = "unknown-work";
const DUPLICATE_WORK_ID: &str = "duplicate-work";
const SAFE_REJECTION_CLASS: &str = "dispatch_rejected";
const RAW_PROMPT_LEAK: &str = "RAW_PROMPT: summarize private text";
const MODEL_OUTPUT_LEAK: &str = "MODEL_OUTPUT: private answer";
const TOOL_OUTPUT_LEAK: &str = "TOOL_OUTPUT: command output";
const PROVIDER_JSON_LEAK: &str = "provider_json { secret: true }";
const TRANSCRIPT_LEAK: &str = "TRANSCRIPT: full conversation";
const UNBOUNDED_LOG_LEAK: &str = "UNBOUNDED_LOG: lots of output";
const EXPECTED_COMPLETED_HASH_COUNT: usize = 1;

#[derive(Default)]
struct ShellProbe {
    provider_calls: usize,
    command_calls: usize,
    storage_writes: usize,
    ui_events: usize,
}

impl ShellProbe {
    fn assert_no_effects(&self) {
        assert_eq!(self.provider_calls, usize::default());
        assert_eq!(self.command_calls, usize::default());
        assert_eq!(self.storage_writes, usize::default());
        assert_eq!(self.ui_events, usize::default());
    }
}

fn default_policy() -> ContinuationPolicy {
    ContinuationPolicy::default()
}

fn state_with_all_candidates() -> ContinuationState {
    ContinuationState::new()
        .with_queued_prompt(QUEUED_WORK_ID)
        .expect("queued work should validate")
        .with_active_loop(LOOP_WORK_ID)
        .expect("loop work should validate")
        .with_validation_follow_up(VALIDATION_WORK_ID)
        .expect("validation work should validate")
}

fn valid_receipt_draft<'a>(
    pending_work_id_hash: &'a str,
    policy_hash: &'a str,
) -> ContinuationReceiptDraft<'a> {
    ContinuationReceiptDraft {
        prior_state_class: IDLE_LABEL,
        selected_action: REPLAY_QUEUED_PROMPT_LABEL,
        pending_work_id_hash,
        reason: QUEUED_USER_INPUT_REASON,
        rejection_class: None,
        policy_hash,
    }
}

#[test]
fn queued_prompt_state_is_explicit_and_needs_no_hidden_flag() {
    // r[verify agent_harness_runtime.continuation_policy.state.queued_prompt]
    let state = ContinuationState::new()
        .with_busy_prompt(BUSY_WORK_ID)
        .expect("busy work should validate")
        .with_queued_prompt(QUEUED_WORK_ID)
        .expect("queued work should validate");
    let queued = state
        .queued_prompt()
        .expect("queued prompt should be typed state");

    assert_eq!(queued.kind(), ContinuationWorkKind::QueuedPrompt);
    assert_eq!(queued.work_id(), QUEUED_WORK_ID);
    assert_ne!(queued.work_id_hash(), QUEUED_WORK_ID);

    let decision = select_next_continuation(&state, &default_policy());
    assert_eq!(decision.action(), ContinuationAction::ReplayQueuedPrompt);
    assert_eq!(decision.work_id(), Some(QUEUED_WORK_ID));
}

#[test]
fn queued_prompt_priority_wins_and_leaves_automation_pending() {
    // r[verify agent_harness_runtime.continuation_policy.priority.queued_wins]
    let state = state_with_all_candidates();
    let policy = default_policy();
    let decision = select_next_continuation(&state, &policy);

    assert_eq!(decision.action(), ContinuationAction::ReplayQueuedPrompt);
    assert_eq!(decision.reason(), ContinuationReason::QueuedUserInput);

    let state = state
        .begin_dispatch(&decision)
        .expect("dispatch should stage")
        .acknowledge_dispatch(&DispatchAcknowledgement {
            work_id: QUEUED_WORK_ID,
            outcome: DispatchAcknowledgementOutcome::Accepted,
        })
        .expect("dispatch should acknowledge");

    assert!(state.queued_prompt().is_none());
    assert!(state.busy_prompt().is_some());
    assert_eq!(
        state
            .active_loop()
            .expect("loop work should remain pending")
            .work_id(),
        LOOP_WORK_ID
    );
    assert_eq!(
        state
            .validation_follow_up()
            .expect("validation work should remain pending")
            .work_id(),
        VALIDATION_WORK_ID
    );
}

#[test]
fn custom_priority_can_make_loop_work_win_explicitly() {
    let priority = ContinuationPriority::new([
        ContinuationCandidate::LoopContinuation,
        ContinuationCandidate::QueuedPrompt,
        ContinuationCandidate::ValidationFollowUp,
    ])
    .expect("priority should validate");
    let policy = ContinuationPolicy::new("loop-first", priority).expect("policy should validate");
    let decision = select_next_continuation(&state_with_all_candidates(), &policy);

    assert_eq!(decision.action(), ContinuationAction::ContinueLoop);
    assert_eq!(decision.work_id(), Some(LOOP_WORK_ID));
}

#[test]
fn exact_completion_acknowledgement_records_safe_hash_only() {
    let state = ContinuationState::new()
        .with_busy_prompt(BUSY_WORK_ID)
        .expect("busy work should validate")
        .acknowledge_completion(&CompletionAcknowledgement {
            work_id: BUSY_WORK_ID,
            action: ContinuationAction::ReplayQueuedPrompt,
        })
        .expect("completion should acknowledge");
    let record = state
        .completion_acknowledgement()
        .expect("completion record should exist");

    assert_eq!(record.action(), ContinuationAction::ReplayQueuedPrompt);
    assert_ne!(record.work_id_hash(), BUSY_WORK_ID);
    assert_eq!(
        state.completed_work_hashes().len(),
        EXPECTED_COMPLETED_HASH_COUNT
    );
}

#[test]
fn decision_core_does_not_call_shell_owned_authority() {
    // r[verify agent_harness_runtime.continuation_policy.shell_owned.no_provider]
    let probe = ShellProbe::default();
    let decision = select_next_continuation(&state_with_all_candidates(), &default_policy());

    assert_eq!(decision.action(), ContinuationAction::ReplayQueuedPrompt);
    probe.assert_no_effects();
}

#[test]
fn cancellation_preempts_normal_continuation() {
    let state = state_with_all_candidates()
        .with_cancellation(CANCELLATION_WORK_ID)
        .expect("cancellation should validate");
    let decision = select_next_continuation(&state, &default_policy());

    assert_eq!(decision.action(), ContinuationAction::Cancel);
    assert_eq!(decision.reason(), ContinuationReason::CancellationRequested);
    assert_eq!(decision.work_id(), Some(CANCELLATION_WORK_ID));
}

#[test]
fn safe_receipt_contains_metadata_hashes_only() {
    // r[verify agent_harness_runtime.continuation_policy.receipts]
    // r[verify agent_harness_runtime.continuation_policy.verification]
    // r[verify agent_harness_runtime.continuation_policy.verification.closeout]
    let policy = default_policy();
    let decision = select_next_continuation(&state_with_all_candidates(), &policy);
    let receipt = ContinuationReceipt::from_decision(
        ContinuationStateClass::Idle,
        &decision,
        &policy,
        Some(SAFE_REJECTION_CLASS),
    )
    .expect("receipt should validate");

    assert_eq!(receipt.prior_state_class(), ContinuationStateClass::Idle);
    assert_eq!(
        receipt.selected_action(),
        ContinuationAction::ReplayQueuedPrompt
    );
    assert_eq!(receipt.reason(), QUEUED_USER_INPUT_REASON);
    assert_eq!(receipt.rejection_class(), Some(SAFE_REJECTION_CLASS));
    assert_ne!(receipt.pending_work_id_hash(), QUEUED_WORK_ID);
    assert!(!receipt.receipt_hash().contains(QUEUED_WORK_ID));
}


use super::*;

const POLICY_HASH: &str = "c4c5f883019c739cc4dd3c4edd790dfa038d095109f6bbac2e5f40f7483c15a6";
const RUN_ID: &str = "run.retry";
const PROVIDER_ID: &str = "mock";
const MODEL_ID: &str = "mock-text";
const FIRST_ATTEMPT_COUNT: u32 = 1;
const BLAKE3_HEX_CHARS: usize = 64;
const SECOND_ATTEMPT_COUNT: u32 = 2;
const THIRD_ATTEMPT_COUNT: u32 = 3;
const FINAL_ATTEMPT_COUNT: u32 = 4;
const ZERO_DELAY_MS: u64 = 0;
const FIRST_EVENT_INDEX: usize = 0;
const FIRST_BACKOFF_MS: u64 = 250;
const SECOND_BACKOFF_MS: u64 = 1_000;
const POLICY_MAX_DELAY_MS: u64 = 30_000;
const INVOCATION_VARIANT_CAPACITY: usize = 10;
const SENSITIVE_CLASS_COUNT: usize = 10;
const EXCESSIVE_ATTEMPT_COUNT: u32 = 9;
const EXCESSIVE_DELAY_MS: u64 = 300_001;
const EXCESSIVE_REASON_COUNT: usize = 65;
const RECEIPT_BOUND_CASE_COUNT: usize = 3;
const PRIVATE_REQUEST: &str = "RAW_REQUEST private";
const PRIVATE_BODY: &str = "response body private";
const PRIVATE_HEADERS: &str = "authorization: private";
const PRIVATE_ENDPOINT: &str = "https://private.example";
const PRIVATE_CREDENTIAL: &str = "API_KEY=private";
const PRIVATE_EXCEPTION: &str = "sdk exception private";
const PRIVATE_PROMPT: &str = "system prompt private";
const PRIVATE_REASONING: &str = "hidden reasoning private";
const PRIVATE_TOOL: &str = "tool body private";
const PRIVATE_PATH: &str = "/home/private/retry";

#[test]
fn generated_policy_revalidates_with_canonical_identity() {
    // r[verify agent_harness_runtime.tau_provider_retry.policy]
    // r[verify agent_harness_runtime.tau_provider_retry.verification]
    let policy = policy();

    assert_eq!(policy.policy_hash(), POLICY_HASH);
    assert_eq!(policy.max_attempts(), FINAL_ATTEMPT_COUNT);
    assert_eq!(
        policy.delay_schedule_ms(),
        &[ZERO_DELAY_MS, FIRST_BACKOFF_MS, 1_000]
    );
    assert_eq!(provider_retry_policy_content_hash(&policy), POLICY_HASH);
}

#[test]
fn stale_policy_duplicate_mapping_and_invalid_schedule_fail_closed() {
    let mut stale = policy_input();
    stale.policy_hash = hash("stale-policy");
    let mut duplicate = policy_input();
    duplicate.mappings.push(duplicate.mappings[0].clone());
    let mut schedule = policy_input();
    schedule.delay_schedule_ms = vec![ZERO_DELAY_MS];
    let mut decreasing_schedule = policy_input();
    decreasing_schedule.delay_schedule_ms =
        vec![SECOND_BACKOFF_MS, FIRST_BACKOFF_MS, ZERO_DELAY_MS];

    assert_eq!(
        validate_provider_retry_policy(&stale)
            .expect_err("stale policy")
            .issue(),
        ProviderRetryIssue::StalePolicy
    );
    assert_eq!(
        validate_provider_retry_policy(&duplicate)
            .expect_err("duplicate mapping")
            .issue(),
        ProviderRetryIssue::InvalidMapping
    );
    assert_eq!(
        validate_provider_retry_policy(&schedule)
            .expect_err("invalid schedule")
            .issue(),
        ProviderRetryIssue::InvalidSchedule
    );
    assert_eq!(
        validate_provider_retry_policy(&decreasing_schedule)
            .expect_err("decreasing schedule")
            .issue(),
        ProviderRetryIssue::InvalidSchedule
    );
}

#[test]
fn attempt_delay_and_invocation_bounds_fail_closed() {
    let mut zero_attempts = policy_input();
    zero_attempts.max_attempts = u32::default();
    let mut excessive_attempts = policy_input();
    excessive_attempts.max_attempts = EXCESSIVE_ATTEMPT_COUNT;
    let mut excessive_policy = policy_input();
    excessive_policy.max_delay_ms = EXCESSIVE_DELAY_MS;

    for invalid in [zero_attempts, excessive_attempts, excessive_policy] {
        assert_eq!(
            validate_provider_retry_policy(&invalid)
                .expect_err("invalid policy bounds")
                .issue(),
            ProviderRetryIssue::InvalidSchedule
        );
    }

    let policy = policy();
    let mut stale_invocation = invocation_input(&policy);
    stale_invocation.exact_plan_hash = hash("stale invocation");
    let mut malformed_invocation = invocation_input(&policy);
    malformed_invocation.request_hash = "BAD".to_string();
    assert_eq!(
        validate_provider_retry_invocation(&stale_invocation, &policy)
            .expect_err("stale invocation")
            .issue(),
        ProviderRetryIssue::StaleInvocation
    );
    assert_eq!(
        validate_provider_retry_invocation(&malformed_invocation, &policy)
            .expect_err("malformed invocation")
            .issue(),
        ProviderRetryIssue::InvalidInput
    );
}

#[test]
fn native_failure_mapping_is_versioned_and_unknown_is_terminal() {
    // r[verify agent_harness_runtime.tau_provider_retry.classification]
    let policy = policy();
    let throttle_observation = observation("rate-limit");
    let unknown = observation("new-provider-status");
    let mut wrong_version = observation("rate-limit");
    wrong_version.mapping_version = "v2".to_string();

    assert_eq!(
        classify_provider_retry_failure(&policy, &throttle_observation).expect("rate limit"),
        ProviderRetryFailureClass::RateLimited
    );
    assert_eq!(
        classify_provider_retry_failure(&policy, &unknown).expect("unknown mapping"),
        ProviderRetryFailureClass::Unknown
    );
    assert_eq!(
        classify_provider_retry_failure(&policy, &wrong_version).expect("wrong version"),
        ProviderRetryFailureClass::Unknown
    );
}

#[test]
fn transient_retry_uses_zero_delay_then_exact_dispatch() {
    // r[verify agent_harness_runtime.tau_provider_retry.backoff]
    // r[verify agent_harness_runtime.tau_provider_retry.events]
    let policy = policy();
    let invocation = invocation(&policy);
    let decision = decision(
        &policy,
        &invocation,
        &invocation,
        DecisionState {
            completed_attempt_count: FIRST_ATTEMPT_COUNT,
            stream_state: ProviderRetryStreamState::TransportStarted,
            cancellation: ProviderRetryCancellation::None,
            native_class: "network-reset",
        },
    );
    let wait = plan_provider_retry_wait(&decision).expect("timer wait plan");
    let timer = timer_observation(&decision, ProviderRetryTimerOutcome::Completed);
    let evaluation =
        evaluate_provider_retry_timer(&decision, wait.state(), &timer).expect("timer evaluation");
    let gate = evaluation.gate();
    let scheduled = project_provider_retry_decision_event(&decision);
    let ready = project_provider_retry_timer_event(&decision, gate);
    let threaded_state = attempt_state(
        &invocation,
        FIRST_ATTEMPT_COUNT,
        ProviderRetryStreamState::TransportStarted,
        ProviderRetryCancellation::None,
    );
    let dispatch = gate.request().expect("dispatch request");
    let advanced =
        advance_provider_retry_attempt(&threaded_state, dispatch).expect("attempt advances once");
    let duplicate_advance = advance_provider_retry_attempt(&advanced, dispatch)
        .expect_err("same dispatch cannot advance twice");

    assert_eq!(decision.status(), ProviderRetryDecisionStatus::Retry);
    assert_eq!(decision.delay_ms(), Some(ZERO_DELAY_MS));
    assert_eq!(wait.request().next_attempt_count, SECOND_ATTEMPT_COUNT);
    assert!(wait.request().cancellation_required);
    assert_eq!(gate.status(), ProviderRetryDispatchGateStatus::Ready);
    assert_eq!(
        evaluation.state().status(),
        ProviderRetryWaitStatus::Completed
    );
    assert_eq!(advanced.completed_attempt_count, SECOND_ATTEMPT_COUNT);
    assert_eq!(duplicate_advance.issue(), ProviderRetryIssue::InvalidInput);
    assert_eq!(
        gate.request().expect("dispatch").exact_plan_hash(),
        invocation.exact_plan_hash()
    );
    assert_eq!(
        [scheduled.status, ready.status],
        [
            ProviderRetryProgressStatus::Scheduled,
            ProviderRetryProgressStatus::DispatchReady,
        ]
    );
}

#[test]
fn retry_schedule_advances_with_named_attempt_bounds() {
    let policy = policy();
    let invocation = invocation(&policy);
    let decision = decision(
        &policy,
        &invocation,
        &invocation,
        DecisionState {
            completed_attempt_count: SECOND_ATTEMPT_COUNT,
            stream_state: ProviderRetryStreamState::TransportStarted,
            cancellation: ProviderRetryCancellation::None,
            native_class: "service-down",
        },
    );

    assert_eq!(decision.status(), ProviderRetryDecisionStatus::Retry);
    assert_eq!(decision.next_attempt_count(), Some(THIRD_ATTEMPT_COUNT));
    assert_eq!(decision.delay_ms(), Some(FIRST_BACKOFF_MS));
}

#[test]
fn permanent_and_unknown_failures_never_retry() {
    let policy = policy();
    let invocation = invocation(&policy);
    let native_classes = [
        "auth",
        "authorization",
        "entitlement",
        "policy-denied",
        "capability-denied",
        "invalid-request",
        "unsupported",
        "malformed",
        "unsafe",
        "missing-service",
        "missing-credential",
        "unmapped",
    ];

    for native_class in native_classes {
        let decision = decision(
            &policy,
            &invocation,
            &invocation,
            DecisionState {
                completed_attempt_count: FIRST_ATTEMPT_COUNT,
                stream_state: ProviderRetryStreamState::TransportStarted,
                cancellation: ProviderRetryCancellation::None,
                native_class,
            },
        );
        assert_eq!(decision.status(), ProviderRetryDecisionStatus::Terminal);
        assert!(plan_provider_retry_wait(&decision).is_err());
    }
}

#[test]
fn every_exact_identity_change_requires_recomposition() {
    // r[verify agent_harness_runtime.tau_provider_retry.same_target]
    let policy = policy();
    let current = invocation(&policy);
    let mut inputs = Vec::with_capacity(INVOCATION_VARIANT_CAPACITY);
    let mut provider = invocation_input(&policy);
    provider.provider_id = "other-provider".to_string();
    inputs.push(provider);
    let mut model = invocation_input(&policy);
    model.model_id = "other-model".to_string();
    inputs.push(model);
    for field in [
        "request",
        "profile",
        "prompt",
        "tool",
        "format",
        "reasoning",
        "stream",
        "credential",
    ] {
        let mut input = invocation_input(&policy);
        replace_invocation_hash(&mut input, field, hash(format!("changed-{field}").as_str()));
        inputs.push(input);
    }

    for input in inputs {
        let next = validate_invocation_with_fresh_hash(input, &policy);
        let decision = decision(
            &policy,
            &current,
            &next,
            DecisionState {
                completed_attempt_count: FIRST_ATTEMPT_COUNT,
                stream_state: ProviderRetryStreamState::TransportStarted,
                cancellation: ProviderRetryCancellation::None,
                native_class: "network-reset",
            },
        );
        assert_eq!(
            decision.status(),
            ProviderRetryDecisionStatus::RecompositionRequired
        );
        assert!(plan_provider_retry_wait(&decision).is_err());
    }
}

#[test]
fn semantic_stream_frames_make_transient_failure_terminal() {
    // r[verify agent_harness_runtime.tau_provider_retry.stream_safety]
    let policy = policy();
    let invocation = invocation(&policy);
    let frames = [
        ProviderRetrySemanticFrame::Content,
        ProviderRetrySemanticFrame::Reasoning,
        ProviderRetrySemanticFrame::ToolCall,
        ProviderRetrySemanticFrame::StructuredOutput,
        ProviderRetrySemanticFrame::Other,
    ];

    for frame in frames {
        let stream_state = observe_provider_retry_stream(
            ProviderRetryStreamState::TransportStarted,
            ProviderRetryStreamObservation::Semantic(frame),
        );
        let decision = decision(
            &policy,
            &invocation,
            &invocation,
            DecisionState {
                completed_attempt_count: FIRST_ATTEMPT_COUNT,
                stream_state,
                cancellation: ProviderRetryCancellation::None,
                native_class: "network-reset",
            },
        );
        assert_eq!(
            decision.status(),
            ProviderRetryDecisionStatus::PostCommitTerminal
        );
        assert!(plan_provider_retry_wait(&decision).is_err());
    }
}

#[test]
fn fragmented_transport_before_semantic_frame_remains_retryable() {
    let started = observe_provider_retry_stream(
        ProviderRetryStreamState::NotStarted,
        ProviderRetryStreamObservation::TransportStarted,
    );
    let unchanged =
        observe_provider_retry_model_frame(started, animus_adapters::ModelStreamFrameKind::Failure);
    let committed = observe_provider_retry_model_frame(
        unchanged,
        animus_adapters::ModelStreamFrameKind::ToolCallDelta,
    );

    assert_eq!(started, ProviderRetryStreamState::TransportStarted);
    assert_eq!(unchanged, ProviderRetryStreamState::TransportStarted);
    assert_eq!(
        committed,
        ProviderRetryStreamState::Committed(ProviderRetrySemanticFrame::ToolCall)
    );
}

#[test]
fn attempt_exhaustion_and_cancellation_stop_dispatch() {
    let policy = policy();
    let invocation = invocation(&policy);
    let exhausted = decision(
        &policy,
        &invocation,
        &invocation,
        DecisionState {
            completed_attempt_count: FINAL_ATTEMPT_COUNT,
            stream_state: ProviderRetryStreamState::TransportStarted,
            cancellation: ProviderRetryCancellation::None,
            native_class: "timeout",
        },
    );
    let cancelled = decision(
        &policy,
        &invocation,
        &invocation,
        DecisionState {
            completed_attempt_count: FIRST_ATTEMPT_COUNT,
            stream_state: ProviderRetryStreamState::TransportStarted,
            cancellation: ProviderRetryCancellation::Requested,
            native_class: "timeout",
        },
    );

    assert_eq!(exhausted.status(), ProviderRetryDecisionStatus::Exhausted);
    assert_eq!(cancelled.status(), ProviderRetryDecisionStatus::Cancelled);
    assert!(plan_provider_retry_wait(&exhausted).is_err());
    assert!(plan_provider_retry_wait(&cancelled).is_err());
}

#[test]
fn timer_cancellation_failure_and_stale_correlation_block_dispatch() {
    let policy = policy();
    let invocation = invocation(&policy);
    let decision = decision(
        &policy,
        &invocation,
        &invocation,
        DecisionState {
            completed_attempt_count: FIRST_ATTEMPT_COUNT,
            stream_state: ProviderRetryStreamState::TransportStarted,
            cancellation: ProviderRetryCancellation::None,
            native_class: "rate-limit",
        },
    );
    let outcomes = [
        (
            ProviderRetryTimerOutcome::Cancelled,
            ProviderRetryDispatchGateStatus::Cancelled,
        ),
        (
            ProviderRetryTimerOutcome::TimedOut,
            ProviderRetryDispatchGateStatus::BlockedTimer,
        ),
        (
            ProviderRetryTimerOutcome::Unavailable,
            ProviderRetryDispatchGateStatus::BlockedTimer,
        ),
        (
            ProviderRetryTimerOutcome::Failed,
            ProviderRetryDispatchGateStatus::BlockedTimer,
        ),
    ];
    for (outcome, expected) in outcomes {
        let wait = plan_provider_retry_wait(&decision).expect("wait plan");
        let timer = timer_observation(&decision, outcome);
        let evaluation =
            evaluate_provider_retry_timer(&decision, wait.state(), &timer).expect("timer outcome");
        assert_eq!(evaluation.gate().status(), expected);
        assert!(evaluation.gate().request().is_none());
    }
    let wait = plan_provider_retry_wait(&decision).expect("wait plan");
    let mut stale = timer_observation(&decision, ProviderRetryTimerOutcome::Completed);
    stale.next_attempt_count = FINAL_ATTEMPT_COUNT;
    let evaluation =
        evaluate_provider_retry_timer(&decision, wait.state(), &stale).expect("stale timer");
    assert_eq!(
        evaluation.gate().status(),
        ProviderRetryDispatchGateStatus::BlockedStale
    );
    assert!(evaluation.gate().request().is_none());
    let mut malformed = timer_observation(&decision, ProviderRetryTimerOutcome::Completed);
    malformed.decision_hash = "BAD".to_string();
    let error = evaluate_provider_retry_timer(&decision, wait.state(), &malformed)
        .expect_err("malformed timer identity");
    assert_eq!(error.issue(), ProviderRetryIssue::InvalidInput);
}

#[test]
fn cancellation_and_completion_races_close_threaded_wait_state() {
    let policy = policy();
    let invocation = invocation(&policy);
    let decision = decision(
        &policy,
        &invocation,
        &invocation,
        DecisionState {
            completed_attempt_count: FIRST_ATTEMPT_COUNT,
            stream_state: ProviderRetryStreamState::TransportStarted,
            cancellation: ProviderRetryCancellation::None,
            native_class: "rate-limit",
        },
    );
    let wait = plan_provider_retry_wait(&decision).expect("wait plan");
    let cancelled = timer_observation(&decision, ProviderRetryTimerOutcome::Cancelled);
    let cancelled_evaluation = evaluate_provider_retry_timer(&decision, wait.state(), &cancelled)
        .expect("cancelled timer");
    let completed = timer_observation(&decision, ProviderRetryTimerOutcome::Completed);
    let after_cancel =
        evaluate_provider_retry_timer(&decision, cancelled_evaluation.state(), &completed)
            .expect("closed cancellation state");
    let completed_evaluation = evaluate_provider_retry_timer(&decision, wait.state(), &completed)
        .expect("completed timer");
    let duplicate_completion =
        evaluate_provider_retry_timer(&decision, completed_evaluation.state(), &completed)
            .expect("closed completion state");

    assert_eq!(
        after_cancel.gate().status(),
        ProviderRetryDispatchGateStatus::Cancelled
    );
    assert!(after_cancel.gate().request().is_none());
    assert_eq!(
        duplicate_completion.gate().status(),
        ProviderRetryDispatchGateStatus::BlockedStale
    );
    assert!(duplicate_completion.gate().request().is_none());
}

#[test]
fn receipt_is_deterministic_bounded_and_explicit_about_non_claims() {
    // r[verify agent_harness_runtime.tau_provider_retry.receipts]
    let policy = policy();
    let invocation = invocation(&policy);
    let reasons = [
        ProviderRetryFailureClass::TransientTransport,
        ProviderRetryFailureClass::RateLimited,
    ];
    let input = ProviderRetryReceiptInput {
        policy_hash: policy.policy_hash(),
        exact_plan_hash: invocation.exact_plan_hash(),
        completed_attempt_count: SECOND_ATTEMPT_COUNT,
        reason_classes: &reasons,
        timer_outcome: Some(ProviderRetryTimerOutcome::Completed),
        status: ProviderRetryReceiptStatus::Succeeded,
        sensitive: ProviderRetrySensitiveDraft::default(),
    };
    let first = project_provider_retry_receipt(input).expect("first receipt");
    let second = project_provider_retry_receipt(input).expect("second receipt");

    assert_eq!(first, second);
    assert_eq!(first.schema(), PROVIDER_RETRY_RECEIPT_SCHEMA);
    assert_eq!(first.completed_attempt_count(), SECOND_ATTEMPT_COUNT);
    assert_eq!(first.reason_classes(), reasons);
    assert!(first.redaction_marker().contains("omitted"));
    assert!(first.non_claim().contains("does not prove"));
    assert_eq!(first.receipt_hash().len(), BLAKE3_HEX_CHARS);
}

#[test]
fn receipt_rejects_every_private_material_class_without_echoing_it() {
    let policy = policy();
    let invocation = invocation(&policy);
    let reasons = [ProviderRetryFailureClass::Unknown];
    for leak in sensitive_leak_drafts() {
        let error = project_provider_retry_receipt(ProviderRetryReceiptInput {
            policy_hash: policy.policy_hash(),
            exact_plan_hash: invocation.exact_plan_hash(),
            completed_attempt_count: FIRST_ATTEMPT_COUNT,
            reason_classes: &reasons,
            timer_outcome: None,
            status: ProviderRetryReceiptStatus::Terminal,
            sensitive: leak,
        })
        .expect_err("private material");
        assert_eq!(error.issue(), ProviderRetryIssue::ReceiptLeak);
        let rendered = format!("{error:?}");
        for private in private_material() {
            assert!(!rendered.contains(private));
        }
    }
}

#[test]
fn receipt_attempt_and_reason_bounds_fail_closed() {
    let policy = policy();
    let invocation = invocation(&policy);
    let one_reason = [ProviderRetryFailureClass::Unknown];
    let excessive_reasons = vec![ProviderRetryFailureClass::Unknown; EXCESSIVE_REASON_COUNT];
    let cases = [
        (u32::default(), one_reason.as_slice()),
        (FIRST_ATTEMPT_COUNT, &[]),
        (FIRST_ATTEMPT_COUNT, excessive_reasons.as_slice()),
    ];

    assert_eq!(cases.len(), RECEIPT_BOUND_CASE_COUNT);
    for (completed_attempt_count, reason_classes) in cases {
        let error = project_provider_retry_receipt(ProviderRetryReceiptInput {
            policy_hash: policy.policy_hash(),
            exact_plan_hash: invocation.exact_plan_hash(),
            completed_attempt_count,
            reason_classes,
            timer_outcome: None,
            status: ProviderRetryReceiptStatus::Terminal,
            sensitive: ProviderRetrySensitiveDraft::default(),
        })
        .expect_err("receipt bound");
        assert_eq!(error.issue(), ProviderRetryIssue::InvalidInput);
    }
}

#[test]
fn progress_events_are_metadata_only_and_never_assistant_messages() {
    let policy = policy();
    let invocation = invocation(&policy);
    let decision = decision(
        &policy,
        &invocation,
        &invocation,
        DecisionState {
            completed_attempt_count: FIRST_ATTEMPT_COUNT,
            stream_state: ProviderRetryStreamState::TransportStarted,
            cancellation: ProviderRetryCancellation::None,
            native_class: "rate-limit",
        },
    );
    let event = project_provider_retry_decision_event(&decision);
    let rendered = format!("{event:?}");

    assert_eq!(event.run_id, RUN_ID);
    assert_eq!(event.reason, ProviderRetryFailureClass::RateLimited);
    assert_eq!(
        validate_provider_retry_progress_event(event.clone()).expect("valid progress event"),
        event
    );
    let mut malformed_event = event.clone();
    malformed_event.run_id = String::from(PRIVATE_BODY);
    assert_eq!(
        validate_provider_retry_progress_event(malformed_event)
            .expect_err("private progress metadata")
            .issue(),
        ProviderRetryIssue::InvalidInput
    );
    let runtime_event = crate::RuntimeEvent::ProviderRetryProgress {
        progress: event.clone(),
    };
    let projected = crate::map_runtime_events_to_headless(std::slice::from_ref(&runtime_event))
        .expect("headless retry event");

    assert_eq!(event.delay_ms, Some(ZERO_DELAY_MS));
    assert_eq!(
        runtime_event.kind(),
        crate::RuntimeEventKind::ProviderRetryProgress
    );
    assert_eq!(
        projected[FIRST_EVENT_INDEX].kind(),
        crate::RuntimeEventKind::ProviderRetryProgress
    );
    let summary = projected[FIRST_EVENT_INDEX].summary();
    assert!(summary.contains("scheduled"));
    assert!(summary.contains("attempt=2/4"));
    assert!(summary.contains("delay_ms=0"));
    assert!(summary.contains("reason=rate_limited"));
    assert!(!rendered.contains(PRIVATE_BODY));
    assert!(!rendered.contains(PRIVATE_HEADERS));
}

fn sensitive_leak_drafts() -> [ProviderRetrySensitiveDraft<'static>; SENSITIVE_CLASS_COUNT] {
    [
        ProviderRetrySensitiveDraft {
            raw_request: Some(PRIVATE_REQUEST),
            ..ProviderRetrySensitiveDraft::default()
        },
        ProviderRetrySensitiveDraft {
            response_body: Some(PRIVATE_BODY),
            ..ProviderRetrySensitiveDraft::default()
        },
        ProviderRetrySensitiveDraft {
            response_headers: Some(PRIVATE_HEADERS),
            ..ProviderRetrySensitiveDraft::default()
        },
        ProviderRetrySensitiveDraft {
            endpoint: Some(PRIVATE_ENDPOINT),
            ..ProviderRetrySensitiveDraft::default()
        },
        ProviderRetrySensitiveDraft {
            credential: Some(PRIVATE_CREDENTIAL),
            ..ProviderRetrySensitiveDraft::default()
        },
        ProviderRetrySensitiveDraft {
            exception_body: Some(PRIVATE_EXCEPTION),
            ..ProviderRetrySensitiveDraft::default()
        },
        ProviderRetrySensitiveDraft {
            prompt: Some(PRIVATE_PROMPT),
            ..ProviderRetrySensitiveDraft::default()
        },
        ProviderRetrySensitiveDraft {
            reasoning: Some(PRIVATE_REASONING),
            ..ProviderRetrySensitiveDraft::default()
        },
        ProviderRetrySensitiveDraft {
            tool_body: Some(PRIVATE_TOOL),
            ..ProviderRetrySensitiveDraft::default()
        },
        ProviderRetrySensitiveDraft {
            private_path: Some(PRIVATE_PATH),
            ..ProviderRetrySensitiveDraft::default()
        },
    ]
}

const fn private_material() -> [&'static str; SENSITIVE_CLASS_COUNT] {
    [
        PRIVATE_REQUEST,
        PRIVATE_BODY,
        PRIVATE_HEADERS,
        PRIVATE_ENDPOINT,
        PRIVATE_CREDENTIAL,
        PRIVATE_EXCEPTION,
        PRIVATE_PROMPT,
        PRIVATE_REASONING,
        PRIVATE_TOOL,
        PRIVATE_PATH,
    ]
}

fn policy() -> ProviderRetryPolicy {
    validate_provider_retry_policy(&policy_input()).expect("valid retry policy")
}

fn policy_input() -> ProviderRetryPolicyInput {
    ProviderRetryPolicyInput {
        schema: PROVIDER_RETRY_POLICY_SCHEMA.to_string(),
        policy_id: "bounded-provider-retry".to_string(),
        max_attempts: FINAL_ATTEMPT_COUNT,
        max_delay_ms: POLICY_MAX_DELAY_MS,
        delay_schedule_ms: vec![ZERO_DELAY_MS, FIRST_BACKOFF_MS, SECOND_BACKOFF_MS],
        cancellation_posture: "required".to_string(),
        stream_commit_policy: "deny_after_semantic".to_string(),
        retryable_classes: [
            "conflict",
            "rate_limited",
            "request_timeout",
            "service_unavailable",
            "transient_transport",
        ]
        .into_iter()
        .map(str::to_string)
        .collect(),
        mappings: [
            ("auth", "authentication"),
            ("conflict", "conflict"),
            ("network-reset", "transient_transport"),
            ("rate-limit", "rate_limited"),
            ("service-down", "service_unavailable"),
            ("timeout", "request_timeout"),
            ("unsupported", "unsupported_feature"),
        ]
        .into_iter()
        .map(
            |(native_class, canonical_class)| ProviderRetryMappingInput {
                provider_family: PROVIDER_ID.to_string(),
                mapping_version: "v1".to_string(),
                native_class: native_class.to_string(),
                canonical_class: canonical_class.to_string(),
            },
        )
        .collect(),
        policy_hash: POLICY_HASH.to_string(),
    }
}

fn invocation(policy: &ProviderRetryPolicy) -> ProviderRetryInvocation {
    validate_provider_retry_invocation(&invocation_input(policy), policy).expect("valid invocation")
}

fn invocation_input(policy: &ProviderRetryPolicy) -> ProviderRetryInvocationInput {
    let mut input = ProviderRetryInvocationInput {
        provider_id: PROVIDER_ID.to_string(),
        model_id: MODEL_ID.to_string(),
        request_hash: hash("request"),
        profile_plan_hash: hash("profile"),
        prompt_hash: hash("prompt"),
        tool_catalog_hash: hash("tool-catalog"),
        response_format_hash: hash("response-format"),
        reasoning_intent_hash: hash("reasoning-intent"),
        stream_intent_hash: hash("stream-intent"),
        credential_handle_hash: hash("credential-handle"),
        policy_hash: policy.policy_hash().to_string(),
        exact_plan_hash: hash("placeholder"),
    };
    input.exact_plan_hash = invocation_hash_from_input(&input);
    input
}

fn validate_invocation_with_fresh_hash(
    mut input: ProviderRetryInvocationInput,
    policy: &ProviderRetryPolicy,
) -> ProviderRetryInvocation {
    input.exact_plan_hash = invocation_hash_from_input(&input);
    validate_provider_retry_invocation(&input, policy).expect("fresh invocation identity")
}

fn invocation_hash_from_input(input: &ProviderRetryInvocationInput) -> String {
    provider_retry_invocation_input_content_hash(input)
}

fn replace_invocation_hash(input: &mut ProviderRetryInvocationInput, field: &str, hash: String) {
    match field {
        "request" => input.request_hash = hash,
        "profile" => input.profile_plan_hash = hash,
        "prompt" => input.prompt_hash = hash,
        "tool" => input.tool_catalog_hash = hash,
        "format" => input.response_format_hash = hash,
        "reasoning" => input.reasoning_intent_hash = hash,
        "stream" => input.stream_intent_hash = hash,
        "credential" => input.credential_handle_hash = hash,
        _other => panic!("unexpected invocation field"),
    }
}

fn observation(native_class: &str) -> ProviderRetryFailureObservation {
    ProviderRetryFailureObservation {
        provider_family: PROVIDER_ID.to_string(),
        mapping_version: "v1".to_string(),
        native_class: native_class.to_string(),
    }
}

#[derive(Clone, Copy)]
struct DecisionState<'a> {
    completed_attempt_count: u32,
    stream_state: ProviderRetryStreamState,
    cancellation: ProviderRetryCancellation,
    native_class: &'a str,
}

fn decision(
    policy: &ProviderRetryPolicy,
    current: &ProviderRetryInvocation,
    next: &ProviderRetryInvocation,
    input: DecisionState<'_>,
) -> ProviderRetryDecision {
    let state = attempt_state(
        current,
        input.completed_attempt_count,
        input.stream_state,
        input.cancellation,
    );
    let failure = observation(input.native_class);
    plan_provider_retry(ProviderRetryPlanInput {
        policy,
        current,
        next,
        state: &state,
        observation: &failure,
    })
    .expect("retry decision")
}

fn attempt_state(
    invocation: &ProviderRetryInvocation,
    completed_attempt_count: u32,
    stream_state: ProviderRetryStreamState,
    cancellation: ProviderRetryCancellation,
) -> ProviderRetryAttemptState {
    ProviderRetryAttemptState {
        run_id: RUN_ID.to_string(),
        completed_attempt_count,
        exact_plan_hash: invocation.exact_plan_hash().to_string(),
        stream_state,
        cancellation,
    }
}

fn timer_observation(
    decision: &ProviderRetryDecision,
    outcome: ProviderRetryTimerOutcome,
) -> ProviderRetryTimerObservation {
    ProviderRetryTimerObservation {
        run_id: RUN_ID.to_string(),
        next_attempt_count: decision.next_attempt_count().expect("next attempt"),
        exact_plan_hash: decision.exact_plan_hash().to_string(),
        decision_hash: decision.decision_hash().to_string(),
        outcome,
    }
}

fn hash(value: &str) -> String {
    blake3::hash(value.as_bytes()).to_hex().to_string()
}

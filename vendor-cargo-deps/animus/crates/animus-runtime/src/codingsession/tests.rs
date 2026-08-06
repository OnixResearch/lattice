use super::*;

const SESSION_ID: &str = "session.fixture";
const BRANCH_ID: &str = "branch.main";
const RUN_ID: &str = "run.one";
const OTHER_RUN_ID: &str = "run.other";
const AUTHORITY_ENTRY: &str = "authority.fixture";
const USER_ENTRY: &str = "entry.user";
const QUEUED_ENTRY: &str = "entry.queued";
const QUEUED_ENTRY_TWO: &str = "entry.queued.two";
const SUMMARY_ENTRY: &str = "entry.summary";
const QUEUE_ITEM: &str = "queue.one";
const QUEUE_ITEM_TWO: &str = "queue.two";
const PROMPT: &str = "review the bounded change";
const STEERING_PROMPT: &str = "steer toward tests";
const FOLLOW_UP_PROMPT: &str = "then summarize evidence";
const RESPONSE: &str = "bounded answer";
const TOOL_ASSISTANT: &str = "I will inspect the fixture";
const TOOL_OUTPUT: &str = "fixture result";
const SUMMARY: &str = "bounded durable summary";
const TOOL_NAME: &str = "fixture_tool";
const TOOL_SCHEMA: &str = "schema.fixture.v1";
const TOOL_ARGUMENT: &str = "query";
const TOOL_ARGUMENT_VALUE: &str = "bounded";
const TOOL_CALL_ID: &str = "call.fixture";
const TOOL_CAPABILITY: &str = "tool.read";
const TOOL_TARGET: &str = "resource.fixture";
const POLICY_HASH: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const SAFE_HEAD: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const QUEUE_MAX_ITEMS: u32 = 8;
const TOOL_VISIBLE_CHARS: u32 = 128;
const FIRST_CURSOR: u64 = 1;
const SECOND_CURSOR: u64 = 2;
const THIRD_CURSOR: u64 = 3;
const EXPECTED_INITIAL_COMMIT_MESSAGE_COUNT: usize = 2;
const EXPECTED_SINGLE_TURN_MESSAGE_COUNT: usize = 3;
const EXPECTED_SINGLE_TURN_RECEIPT_MESSAGE_COUNT: u32 = 3;
const EXPECTED_QUEUE_INJECTED_MESSAGE_COUNT: usize = 4;
const EXPECTED_TOOL_PRE_FINAL_MESSAGE_COUNT: usize = 4;
const EXPECTED_TOOL_MESSAGE_COUNT: usize = 5;
const EXPECTED_ALL_AT_ONCE_MESSAGE_COUNT: usize = 5;
const EXPECTED_SECOND_TURN_USER_MESSAGES: usize = 4;
const EXPECTED_SECOND_TURN_MESSAGES: usize = 5;
const WIRE_VERSION_INDEX: usize = 0;
const UNSUPPORTED_WIRE_VERSION: u8 = 2;

#[test]
fn identical_state_and_command_produce_identical_transition() {
    // r[verify agent_harness_runtime.coding_session_host.functional_core]
    let initial = state(CodingSessionQueueProfile::Sequential, false);
    let command = CodingSessionCommand::Prompt {
        run_id: String::from(RUN_ID),
        entry_id: String::from(USER_ENTRY),
        input: input(PROMPT),
    };
    let first = reduce_coding_session_command(&initial, &command).expect("first reduction");
    let second = reduce_coding_session_command(&initial, &command).expect("second reduction");
    assert_eq!(first, second);
    assert_eq!(initial.messages().len(), 0);
    assert!(initial.pending_effect().is_none());
}

#[test]
fn prompt_commits_before_provider_and_final_response_commits_before_completion() {
    // r[verify agent_harness_runtime.coding_session_host.functional_core]
    // r[verify agent_harness_runtime.coding_session_host.durable_boundaries]
    let state = state(CodingSessionQueueProfile::Sequential, false);
    let prompt = reduce_coding_session_command(&state, &CodingSessionCommand::Prompt {
        run_id: String::from(RUN_ID),
        entry_id: String::from(USER_ENTRY),
        input: input(PROMPT),
    })
    .expect("prompt reduction");
    assert!(matches!(prompt.effect, Some(CodingSessionEffectRequest::Commit(_))));
    assert!(
        !prompt
            .events
            .iter()
            .any(|event| matches!(event, crate::telemetry::RuntimeEvent::ModelRequested { .. }))
    );

    let user_commit = commit_pending(&prompt.state);
    assert!(matches!(user_commit.effect, Some(CodingSessionEffectRequest::Provider(_))));
    assert_provider_projection_excludes_authority(&user_commit);

    let provider = provider_final(&user_commit.state, RESPONSE);
    assert!(matches!(provider.effect, Some(CodingSessionEffectRequest::Commit(_))));
    assert!(
        !provider
            .events
            .iter()
            .any(|event| matches!(event, crate::telemetry::RuntimeEvent::FinalResponse { .. }))
    );

    let assistant_commit = commit_pending(&provider.state);
    assert_eq!(assistant_commit.state.messages().len(), EXPECTED_SINGLE_TURN_MESSAGE_COUNT);
    assert_eq!(assistant_commit.state.terminal(), Some(CodingSessionTerminalKind::Completed));
    assert!(
        assistant_commit
            .events
            .iter()
            .any(|event| matches!(event, crate::telemetry::RuntimeEvent::FinalResponse { .. }))
    );
    assert!(
        assistant_commit
            .events
            .iter()
            .any(|event| matches!(event, crate::telemetry::RuntimeEvent::Shutdown { .. }))
    );
}

fn assert_provider_projection_excludes_authority(transition: &CodingSessionTransition) {
    assert_eq!(transition.state.messages().len(), EXPECTED_INITIAL_COMMIT_MESSAGE_COUNT);
    assert_eq!(transition.state.messages()[0].kind(), CodingSessionMessageKind::AuthorityProfileBinding);
    let provider_request = match transition.effect.as_ref().expect("provider effect") {
        CodingSessionEffectRequest::Provider(request) => request,
        _other => panic!("expected provider request"),
    };
    assert!(provider_request.request().prompt().contains(PROMPT));
    assert!(
        !provider_request
            .request()
            .prompt()
            .contains(transition.state.authority_profile().profile_identity())
    );
}

#[test]
fn terminal_receipt_is_bounded_metadata() {
    let completed = completed_session();
    let receipt =
        project_coding_session_receipt(&completed, CodingSessionSensitiveDraft::default()).expect("safe receipt");
    assert_eq!(receipt.committed_message_count, EXPECTED_SINGLE_TURN_RECEIPT_MESSAGE_COUNT);
    assert_eq!(receipt.provider_dispatch_count, 1);
    assert_eq!(receipt.authority_profile_identity, completed.authority_profile().profile_identity());
    assert_eq!(
        receipt.authority_binding_identity,
        completed.authority_binding_identity().expect("authority binding")
    );
    assert!(!receipt.non_claims.is_empty());
}

#[test]
fn completed_session_admits_a_second_durable_turn_without_losing_history() {
    // r[verify agent_harness_runtime.coding_session_host.conformance]
    let completed = completed_session();
    let second_prompt = reduce_coding_session_command(&completed, &CodingSessionCommand::Prompt {
        run_id: String::from(OTHER_RUN_ID),
        entry_id: String::from("entry.user.second"),
        input: input("second bounded turn"),
    })
    .expect("second prompt");
    let second_user_commit = commit_pending(&second_prompt.state);
    assert_eq!(second_user_commit.state.messages().len(), EXPECTED_SECOND_TURN_USER_MESSAGES);
    assert!(matches!(second_user_commit.effect, Some(CodingSessionEffectRequest::Provider(_))));
    let second_final = provider_final(&second_user_commit.state, "second bounded response");
    let completed_second_turn = commit_pending(&second_final.state);
    assert_eq!(completed_second_turn.state.messages().len(), EXPECTED_SECOND_TURN_MESSAGES);
    assert_eq!(completed_second_turn.state.terminal(), Some(CodingSessionTerminalKind::Completed));
}

#[test]
fn tool_call_and_result_each_commit_before_dependent_effect() {
    let user_commit = committed_prompt();
    let provider = provider_tool_call(&user_commit.state);
    assert!(matches!(provider.effect, Some(CodingSessionEffectRequest::Commit(_))));
    assert!(
        !provider
            .events
            .iter()
            .any(|event| matches!(event, crate::telemetry::RuntimeEvent::ToolRequested { .. }))
    );

    let assistant_commit = commit_pending(&provider.state);
    assert!(matches!(assistant_commit.effect, Some(CodingSessionEffectRequest::Tool(_))));
    assert!(
        assistant_commit
            .events
            .iter()
            .any(|event| matches!(event, crate::telemetry::RuntimeEvent::ToolRequested { .. }))
    );

    let tool = tool_success(&assistant_commit.state);
    assert!(matches!(tool.effect, Some(CodingSessionEffectRequest::Commit(_))));
    assert!(
        !tool
            .events
            .iter()
            .any(|event| matches!(event, crate::telemetry::RuntimeEvent::ToolResponded { .. }))
    );

    let result_commit = commit_pending(&tool.state);
    assert!(matches!(result_commit.effect, Some(CodingSessionEffectRequest::Provider(_))));
    assert_eq!(result_commit.state.messages().len(), EXPECTED_TOOL_PRE_FINAL_MESSAGE_COUNT);
    assert!(
        result_commit
            .events
            .iter()
            .any(|event| matches!(event, crate::telemetry::RuntimeEvent::ToolResponded { .. }))
    );

    let final_provider = provider_final(&result_commit.state, RESPONSE);
    let final_commit = commit_pending(&final_provider.state);
    assert_eq!(final_commit.state.messages().len(), EXPECTED_TOOL_MESSAGE_COUNT);
}

#[test]
fn initial_commit_and_provider_failures_block_dependent_effects() {
    // r[verify agent_harness_runtime.coding_session_host.durable_boundaries]
    // r[verify agent_harness_runtime.coding_session_host.conformance]
    let initial = state(CodingSessionQueueProfile::Sequential, false);
    let prompt = reduce_coding_session_command(&initial, &CodingSessionCommand::Prompt {
        run_id: String::from(RUN_ID),
        entry_id: String::from(USER_ENTRY),
        input: input(PROMPT),
    })
    .expect("prompt transition");
    let prompt_commit_failed =
        failed_effect(&prompt.state, CodingSessionEffectService::Persistence, CodingSessionServiceIssue::Failed);
    assert_blocked(&prompt_commit_failed, CodingSessionIssue::PersistenceFailure);
    assert_eq!(prompt_commit_failed.state.provider_dispatch_count, 0);

    let committed = commit_pending(&prompt.state);
    let provider_failed =
        failed_effect(&committed.state, CodingSessionEffectService::Provider, CodingSessionServiceIssue::Unavailable);
    assert_blocked(&provider_failed, CodingSessionIssue::ProviderFailure);
    assert_eq!(provider_failed.state.last_issue(), Some(CodingSessionIssue::ProviderFailure));
}

#[test]
fn tool_and_later_commit_failures_block_dependent_effects() {
    let committed = committed_prompt();
    let assistant_commit = provider_tool_call(&committed.state);
    let assistant_commit_failed = failed_effect(
        &assistant_commit.state,
        CodingSessionEffectService::Persistence,
        CodingSessionServiceIssue::Failed,
    );
    assert_blocked(&assistant_commit_failed, CodingSessionIssue::PersistenceFailure);
    assert_eq!(assistant_commit_failed.state.tool_dispatch_count, 0);

    let tool_dispatch = commit_pending(&assistant_commit.state);
    let tool_failed =
        failed_effect(&tool_dispatch.state, CodingSessionEffectService::Tool, CodingSessionServiceIssue::Denied);
    assert_blocked(&tool_failed, CodingSessionIssue::ToolFailure);

    let tool_result_commit = tool_success(&tool_dispatch.state);
    let provider_dispatch_count = tool_result_commit.state.provider_dispatch_count;
    let tool_result_commit_failed = failed_effect(
        &tool_result_commit.state,
        CodingSessionEffectService::Persistence,
        CodingSessionServiceIssue::Failed,
    );
    assert_blocked(&tool_result_commit_failed, CodingSessionIssue::PersistenceFailure);
    assert_eq!(tool_result_commit_failed.state.provider_dispatch_count, provider_dispatch_count);

    let final_commit = provider_final(&committed.state, RESPONSE);
    let final_commit_failed =
        failed_effect(&final_commit.state, CodingSessionEffectService::Persistence, CodingSessionServiceIssue::Failed);
    assert_blocked(&final_commit_failed, CodingSessionIssue::PersistenceFailure);
    assert_ne!(final_commit_failed.state.terminal(), Some(CodingSessionTerminalKind::Completed));
}

#[test]
fn queue_items_remain_non_transcript_until_durable_injection() {
    // r[verify agent_harness_runtime.coding_session_host.queues]
    let user_commit = committed_prompt();
    let queued = reduce_coding_session_command(&user_commit.state, &CodingSessionCommand::Steer {
        item_id: String::from(QUEUE_ITEM),
        entry_id: String::from(QUEUED_ENTRY),
        input: input(STEERING_PROMPT),
    })
    .expect("queue steering");
    assert_eq!(queued.state.messages().len(), EXPECTED_INITIAL_COMMIT_MESSAGE_COUNT);
    assert_eq!(queued.state.queued_count(), Some(1));
    let removed = edit_and_remove_follow_up(&queued);

    let provider = provider_final(&removed.state, RESPONSE);
    let assistant_commit = commit_pending(&provider.state);
    assert!(matches!(assistant_commit.effect, Some(CodingSessionEffectRequest::Commit(_))));
    assert_eq!(assistant_commit.state.messages().len(), EXPECTED_SINGLE_TURN_MESSAGE_COUNT);

    let queue_commit = commit_pending(&assistant_commit.state);
    assert_eq!(queue_commit.state.messages().len(), EXPECTED_QUEUE_INJECTED_MESSAGE_COUNT);
    assert_eq!(queue_commit.state.queued_count(), Some(0));
    assert!(matches!(queue_commit.effect, Some(CodingSessionEffectRequest::Provider(_))));
}

fn edit_and_remove_follow_up(queued: &CodingSessionTransition) -> CodingSessionTransition {
    let with_follow_up = reduce_coding_session_command(&queued.state, &CodingSessionCommand::FollowUp {
        item_id: String::from(QUEUE_ITEM_TWO),
        entry_id: String::from(QUEUED_ENTRY_TWO),
        input: input(FOLLOW_UP_PROMPT),
    })
    .expect("queue follow-up");
    let edited = reduce_coding_session_command(&with_follow_up.state, &CodingSessionCommand::EditQueued {
        item_id: String::from(QUEUE_ITEM_TWO),
        entry_id: String::from("entry.queued.two.edited"),
        input: input("edited bounded follow-up"),
    })
    .expect("edit queued follow-up");
    assert_eq!(edited.state.messages().len(), EXPECTED_INITIAL_COMMIT_MESSAGE_COUNT);
    assert_eq!(edited.state.active_leaf_id(), queued.state.active_leaf_id());
    let removed = reduce_coding_session_command(&edited.state, &CodingSessionCommand::RemoveQueued {
        item_id: String::from(QUEUE_ITEM_TWO),
    })
    .expect("remove queued follow-up");
    assert_eq!(removed.state.messages().len(), EXPECTED_INITIAL_COMMIT_MESSAGE_COUNT);
    assert_eq!(removed.state.queued_count(), Some(1));
    assert_eq!(removed.state.active_leaf_id(), queued.state.active_leaf_id());
    removed
}

#[test]
fn all_at_once_profile_commits_every_selected_queue_item_before_provider() {
    let user_commit = committed_prompt_with_profile(CodingSessionQueueProfile::AllAtOnce);
    let first = queue_command(&user_commit.state, QueueCommandDraft {
        item_id: QUEUE_ITEM,
        entry_id: QUEUED_ENTRY,
        body: STEERING_PROMPT,
        is_steering: true,
    });
    let second = queue_command(&first.state, QueueCommandDraft {
        item_id: QUEUE_ITEM_TWO,
        entry_id: QUEUED_ENTRY_TWO,
        body: FOLLOW_UP_PROMPT,
        is_steering: false,
    });
    let provider = provider_final(&second.state, RESPONSE);
    let assistant_commit = commit_pending(&provider.state);
    let first_queue_commit = commit_pending(&assistant_commit.state);
    assert!(matches!(first_queue_commit.effect, Some(CodingSessionEffectRequest::Commit(_))));
    let second_queue_commit = commit_pending(&first_queue_commit.state);
    assert!(matches!(second_queue_commit.effect, Some(CodingSessionEffectRequest::Provider(_))));
    assert_eq!(second_queue_commit.state.messages().len(), EXPECTED_ALL_AT_ONCE_MESSAGE_COUNT);
    assert_eq!(second_queue_commit.state.queued_count(), Some(0));
}

#[test]
fn overlapping_prompt_and_stale_cancel_preserve_active_state() {
    // r[verify agent_harness_runtime.coding_session_host.single_run]
    let initial = state(CodingSessionQueueProfile::Sequential, false);
    let started = reduce_coding_session_command(&initial, &CodingSessionCommand::Prompt {
        run_id: String::from(RUN_ID),
        entry_id: String::from(USER_ENTRY),
        input: input(PROMPT),
    })
    .expect("start");
    let overlap_error = reduce_coding_session_command(&started.state, &CodingSessionCommand::Prompt {
        run_id: String::from(OTHER_RUN_ID),
        entry_id: String::from("entry.overlap"),
        input: input("overlap"),
    })
    .expect_err("overlapping prompt must fail");
    assert_eq!(overlap_error.issue(), CodingSessionIssue::AlreadyRunning);
    assert_eq!(started.state.messages().len(), initial.messages().len());

    let stale_cancel_error = reduce_coding_session_command(&started.state, &CodingSessionCommand::Cancel {
        run_id: String::from(OTHER_RUN_ID),
    })
    .expect_err("stale cancellation must fail");
    assert_eq!(stale_cancel_error.issue(), CodingSessionIssue::StaleRun);
    assert_eq!(started.state.run().expect("active run").run_id(), RUN_ID);
}

#[test]
fn oversized_and_unexpanded_inputs_preserve_state() {
    let started = committed_prompt();
    let invalid_input_body = "x".repeat(super::data::MAX_MESSAGE_BYTES.saturating_add(1));
    let input_error = reduce_coding_session_command(&started.state, &CodingSessionCommand::Steer {
        item_id: String::from(QUEUE_ITEM),
        entry_id: String::from(QUEUED_ENTRY),
        input: input(invalid_input_body.as_str()),
    })
    .expect_err("oversized steering input must fail");
    assert_eq!(input_error.issue(), CodingSessionIssue::InvalidInput);
    assert_eq!(started.state.queued_count(), Some(0));

    let denied = state(CodingSessionQueueProfile::Sequential, false);
    let unexpanded_error = reduce_coding_session_command(&denied, &CodingSessionCommand::Prompt {
        run_id: String::from(RUN_ID),
        entry_id: String::from(USER_ENTRY),
        input: CodingSessionInput {
            body: String::from("/unexpanded-command"),
            posture: CodingSessionInputPosture::UnexpandedWorkflowCommand,
        },
    })
    .expect_err("unexpanded command must fail");
    assert_eq!(unexpanded_error.issue(), CodingSessionIssue::InputNotAdmitted);
    assert!(denied.messages().is_empty());
}

#[test]
fn correlated_cancellation_is_advisory_and_late_outcomes_are_stale() {
    let user_commit = committed_prompt();
    let cancelled = reduce_coding_session_command(&user_commit.state, &CodingSessionCommand::Cancel {
        run_id: String::from(RUN_ID),
    })
    .expect("cancel request");
    let effect_id = cancelled.effect.as_ref().expect("cancel effect").effect_id();
    assert_issue(
        reduce_coding_session_outcome(&cancelled.state, &CodingSessionEffectOutcome::CancellationObserved {
            effect_id: String::from(effect_id),
            run_id: String::from(OTHER_RUN_ID),
            observed: true,
        }),
        CodingSessionIssue::StaleRun,
    );
    let observed = reduce_coding_session_outcome(&cancelled.state, &CodingSessionEffectOutcome::CancellationObserved {
        effect_id: String::from(effect_id),
        run_id: String::from(RUN_ID),
        observed: true,
    })
    .expect("cancel observed");
    assert_eq!(observed.state.terminal(), Some(CodingSessionTerminalKind::Cancelled));
    assert!(
        observed
            .events
            .iter()
            .any(|event| matches!(event, crate::telemetry::RuntimeEvent::Cancelled { .. }))
    );
    assert_issue(
        reduce_coding_session_outcome(&observed.state, &CodingSessionEffectOutcome::ProviderSucceeded {
            effect_id: String::from("stale-effect"),
            reply: CodingSessionProviderReply::Final(model_response(RESPONSE)),
        }),
        CodingSessionIssue::StaleEffect,
    );
}

#[test]
fn compaction_summary_commits_before_compaction_event() {
    let completed = completed_session();
    let requested = reduce_coding_session_command(&completed, &CodingSessionCommand::Compact {
        entry_id: String::from(SUMMARY_ENTRY),
    })
    .expect("compaction request");
    assert!(matches!(requested.effect, Some(CodingSessionEffectRequest::Summarizer(_))));
    let effect_id = requested.effect.as_ref().expect("effect").effect_id();
    let summarized =
        reduce_coding_session_outcome(&requested.state, &CodingSessionEffectOutcome::SummarizerSucceeded {
            effect_id: String::from(effect_id),
            entry_id: String::from(SUMMARY_ENTRY),
            summary: String::from(SUMMARY),
        })
        .expect("summary outcome");
    assert!(matches!(summarized.effect, Some(CodingSessionEffectRequest::Commit(_))));
    assert!(
        !summarized
            .events
            .iter()
            .any(|event| matches!(event, crate::telemetry::RuntimeEvent::CompactionRequested { .. }))
    );
    let committed = commit_pending(&summarized.state);
    assert!(
        committed
            .events
            .iter()
            .any(|event| matches!(event, crate::telemetry::RuntimeEvent::CompactionRequested { .. }))
    );
}

#[test]
fn clean_resume_replays_messages_and_repairs_unmatched_tool_without_reexecution() {
    // r[verify agent_harness_runtime.coding_session_host.recovery]
    let authority = recovered_authority_message();
    let user = super::core::test_message(super::core::CodingSessionMessageDraft {
        entry_id: USER_ENTRY,
        parent_id: Some(AUTHORITY_ENTRY),
        run_id: RUN_ID,
        kind: CodingSessionMessageKind::User,
        body: PROMPT,
        tool_correlation_id: None,
        tool_name: None,
    })
    .expect("user message");
    let call = super::core::test_message(super::core::CodingSessionMessageDraft {
        entry_id: "entry.tool.call",
        parent_id: Some(USER_ENTRY),
        run_id: RUN_ID,
        kind: CodingSessionMessageKind::AssistantToolCall,
        body: TOOL_ASSISTANT,
        tool_correlation_id: Some(TOOL_CALL_ID),
        tool_name: Some(TOOL_NAME),
    })
    .expect("tool call");
    let resume_state = state(CodingSessionQueueProfile::Sequential, true);
    let requested =
        reduce_coding_session_command(&resume_state, &CodingSessionCommand::Resume).expect("resume request");
    let effect_id = requested.effect.as_ref().expect("recovery effect").effect_id();
    let recovered = reduce_coding_session_outcome(&requested.state, &CodingSessionEffectOutcome::RecoverySucceeded {
        effect_id: String::from(effect_id),
        snapshot: CodingSessionRecoverySnapshot {
            status: CodingSessionRecoveryStatus::Clean,
            committed_head_hash: String::from(SAFE_HEAD),
            active_leaf_id: Some(String::from("entry.tool.call")),
            durable_cursor: THIRD_CURSOR,
            messages: vec![authority, user, call],
        },
    })
    .expect("recovery");
    let repair = match recovered.effect.as_ref().expect("repair commit") {
        CodingSessionEffectRequest::Commit(request) => request,
        other => panic!("expected repair commit, got {other:?}"),
    };
    assert_eq!(repair.messages().len(), 1);
    assert_eq!(repair.messages()[0].kind(), CodingSessionMessageKind::Repair);
    assert!(
        !recovered
            .events
            .iter()
            .any(|event| matches!(event, crate::telemetry::RuntimeEvent::ToolRequested { .. }))
    );
}

#[test]
fn explicit_authority_rebind_commits_successor_without_provider_dispatch() {
    // r[verify agent_harness_runtime.session_authority_profile.durable_binding]
    let completed = completed_session();
    let prior_binding_identity = completed.authority_binding_identity().expect("initial binding").to_string();
    let changed_profile = authority_profile(POLICY_HASH);
    let requested = reduce_coding_session_command(&completed, &CodingSessionCommand::RebindAuthority {
        entry_id: String::from("authority.rebind"),
        profile: changed_profile.clone(),
        authorization_identity: String::from(SAFE_HEAD),
    })
    .expect("rebind request");
    assert!(matches!(requested.effect, Some(CodingSessionEffectRequest::Commit(_))));
    let rebound = commit_pending(&requested.state);
    assert!(rebound.effect.is_none());
    assert_eq!(rebound.state.authority_profile(), &changed_profile);
    assert_ne!(rebound.state.authority_binding_identity(), Some(prior_binding_identity.as_str()));
    assert_eq!(rebound.state.provider_dispatch_count, completed.provider_dispatch_count);
}

#[test]
fn authority_transition_planning_exposes_abandon_and_fork_without_mutation() {
    let completed = completed_session();
    let original = completed.clone();
    let requested = authority_profile(POLICY_HASH);
    let fork = plan_coding_session_authority_transition(
        &completed,
        &requested,
        crate::session_authority_profile::SessionAuthorityTransitionAction::Fork {
            branch_id: "branch.authority-fork",
            authorization_identity: SAFE_HEAD,
        },
    )
    .expect("fork plan");
    assert_eq!(
        fork.disposition(),
        crate::session_authority_profile::SessionAuthorityTransitionDisposition::ForkCommitRequired
    );
    assert_eq!(fork.binding().expect("fork binding").branch_id(), "branch.authority-fork");
    let abandon = plan_coding_session_authority_transition(
        &completed,
        completed.authority_profile(),
        crate::session_authority_profile::SessionAuthorityTransitionAction::Abandon,
    )
    .expect("abandon plan");
    assert_eq!(
        abandon.disposition(),
        crate::session_authority_profile::SessionAuthorityTransitionDisposition::Abandoned
    );
    assert_eq!(completed, original);
}

#[test]
fn resume_blocks_profile_drift_and_legacy_unbound_history_before_effects() {
    // r[verify agent_harness_runtime.session_authority_profile.resume_admission]
    let mut drift_config = config(CodingSessionQueueProfile::Sequential, true);
    drift_config.authority_profile = authority_profile(POLICY_HASH);
    let drift_state = new_coding_session_state(drift_config, queue_policy()).expect("drift state");
    let drift_request =
        reduce_coding_session_command(&drift_state, &CodingSessionCommand::Resume).expect("drift resume request");
    let drift_effect_id = drift_request.effect.as_ref().expect("recovery").effect_id();
    let drifted = reduce_coding_session_outcome(&drift_request.state, &CodingSessionEffectOutcome::RecoverySucceeded {
        effect_id: String::from(drift_effect_id),
        snapshot: CodingSessionRecoverySnapshot {
            status: CodingSessionRecoveryStatus::Clean,
            committed_head_hash: String::from(SAFE_HEAD),
            active_leaf_id: Some(String::from(USER_ENTRY)),
            durable_cursor: SECOND_CURSOR,
            messages: vec![recovered_authority_message(), recovered_user_message()],
        },
    })
    .expect("drift blocked transition");
    assert_eq!(drifted.state.last_issue(), Some(CodingSessionIssue::AuthorityProfileMismatch));
    assert!(drifted.effect.is_none());
    assert_eq!(drifted.state.provider_dispatch_count, 0);

    let legacy_state = state(CodingSessionQueueProfile::Sequential, true);
    let legacy_request =
        reduce_coding_session_command(&legacy_state, &CodingSessionCommand::Resume).expect("legacy resume request");
    let legacy_effect_id = legacy_request.effect.as_ref().expect("recovery").effect_id();
    let legacy = reduce_coding_session_outcome(&legacy_request.state, &CodingSessionEffectOutcome::RecoverySucceeded {
        effect_id: String::from(legacy_effect_id),
        snapshot: CodingSessionRecoverySnapshot {
            status: CodingSessionRecoveryStatus::Empty,
            committed_head_hash: String::from(CODING_SESSION_EMPTY_HEAD_HASH),
            active_leaf_id: None,
            durable_cursor: 0,
            messages: Vec::new(),
        },
    })
    .expect("legacy blocked transition");
    assert_eq!(legacy.state.last_issue(), Some(CodingSessionIssue::AuthorityProfileUnbound));
    assert!(legacy.effect.is_none());
}

#[test]
fn incomplete_and_stale_leaf_recovery_fail_closed() {
    let resume_state = state(CodingSessionQueueProfile::Sequential, true);
    let requested =
        reduce_coding_session_command(&resume_state, &CodingSessionCommand::Resume).expect("resume request");
    let effect_id = requested.effect.as_ref().expect("effect").effect_id();
    let blocked = reduce_coding_session_outcome(&requested.state, &CodingSessionEffectOutcome::RecoverySucceeded {
        effect_id: String::from(effect_id),
        snapshot: CodingSessionRecoverySnapshot {
            status: CodingSessionRecoveryStatus::IncompleteTail,
            committed_head_hash: String::from(CODING_SESSION_EMPTY_HEAD_HASH),
            active_leaf_id: None,
            durable_cursor: 0,
            messages: Vec::new(),
        },
    })
    .expect("blocked recovery transition");
    assert_eq!(blocked.state.terminal(), Some(CodingSessionTerminalKind::Blocked));
    assert!(blocked.effect.is_none());

    let stale_leaf_state = state(CodingSessionQueueProfile::Sequential, true);
    let stale_leaf_request = reduce_coding_session_command(&stale_leaf_state, &CodingSessionCommand::Resume)
        .expect("stale leaf resume request");
    let stale_leaf_effect = stale_leaf_request.effect.as_ref().expect("recovery effect");
    let recovered_user = recovered_user_message();
    let stale_leaf_error =
        reduce_coding_session_outcome(&stale_leaf_request.state, &CodingSessionEffectOutcome::RecoverySucceeded {
            effect_id: String::from(stale_leaf_effect.effect_id()),
            snapshot: CodingSessionRecoverySnapshot {
                status: CodingSessionRecoveryStatus::Clean,
                committed_head_hash: String::from(SAFE_HEAD),
                active_leaf_id: Some(String::from("entry.stale-leaf")),
                durable_cursor: SECOND_CURSOR,
                messages: vec![recovered_authority_message(), recovered_user],
            },
        })
        .expect_err("stale active leaf must fail recovery");
    assert_eq!(stale_leaf_error.issue(), CodingSessionIssue::RecoveryBlocked);
    assert_eq!(stale_leaf_request.state.recovery_posture(), CodingSessionRecoveryPosture::ResumeRequired);
}

#[test]
fn missing_recovery_timer_and_receipt_leaks_fail_closed() {
    let unavailable_state = state(CodingSessionQueueProfile::Sequential, true);
    let unavailable_request = reduce_coding_session_command(&unavailable_state, &CodingSessionCommand::Resume)
        .expect("unavailable recovery request");
    let unavailable = failed_effect(
        &unavailable_request.state,
        CodingSessionEffectService::Recovery,
        CodingSessionServiceIssue::Unavailable,
    );
    assert_eq!(unavailable.state.terminal(), Some(CodingSessionTerminalKind::Blocked));
    assert_eq!(unavailable.state.last_issue(), Some(CodingSessionIssue::PersistenceFailure));

    let mut config = config(CodingSessionQueueProfile::Sequential, false);
    config.services.timer = CodingSessionServicePosture::Missing;
    let timer_error =
        new_coding_session_state(config, queue_policy()).expect_err("missing timer must reject configuration");
    assert_eq!(timer_error.issue(), CodingSessionIssue::UnavailableService);

    let completed = completed_session();
    let receipt_error = project_coding_session_receipt(&completed, CodingSessionSensitiveDraft {
        raw_prompt: Some(PROMPT),
        ..CodingSessionSensitiveDraft::default()
    })
    .expect_err("private receipt material must be rejected");
    assert_eq!(receipt_error.issue(), CodingSessionIssue::UnsafeReceipt);
}

#[test]
fn message_wire_round_trips_and_rejects_malformed_version_and_tool_binding() {
    let message = super::core::test_message(super::core::CodingSessionMessageDraft {
        entry_id: "entry.tool",
        parent_id: Some(USER_ENTRY),
        run_id: RUN_ID,
        kind: CodingSessionMessageKind::AssistantToolCall,
        body: TOOL_ASSISTANT,
        tool_correlation_id: Some(TOOL_CALL_ID),
        tool_name: Some(TOOL_NAME),
    })
    .expect("message");
    let bytes = encode_coding_session_message(&message).expect("encode");
    let decoded =
        decode_coding_session_message(message.entry_id(), message.parent_id(), bytes.as_slice()).expect("decode");
    assert_eq!(decoded, message);

    let mut wrong_version = bytes;
    wrong_version[WIRE_VERSION_INDEX] = UNSUPPORTED_WIRE_VERSION;
    assert_issue(
        decode_coding_session_message(message.entry_id(), message.parent_id(), wrong_version.as_slice()),
        CodingSessionIssue::InvalidInput,
    );
    assert!(
        super::core::test_message(super::core::CodingSessionMessageDraft {
            entry_id: "entry.invalid",
            parent_id: Some(USER_ENTRY),
            run_id: RUN_ID,
            kind: CodingSessionMessageKind::ToolResult,
            body: TOOL_OUTPUT,
            tool_correlation_id: None,
            tool_name: None,
        })
        .is_err()
    );
    let incomplete_repair = super::core::test_message(super::core::CodingSessionMessageDraft {
        entry_id: "repair.incomplete",
        parent_id: Some(USER_ENTRY),
        run_id: RUN_ID,
        kind: CodingSessionMessageKind::Repair,
        body: "repair without a complete tool binding",
        tool_correlation_id: Some(TOOL_CALL_ID),
        tool_name: None,
    });
    assert!(incomplete_repair.is_err());
}

#[test]
fn frontend_projection_tracks_shared_commit_queue_tool_cancel_and_terminal_events() {
    // r[verify agent_harness_runtime.coding_session_host.events]
    let events = [
        crate::telemetry::RuntimeEvent::SessionStateChanged {
            session_id: String::from(SESSION_ID),
            run_id: Some(String::from(RUN_ID)),
            status: String::from("steering"),
            queue_count: 1,
        },
        crate::telemetry::RuntimeEvent::SessionCommitObserved {
            session_id: String::from(SESSION_ID),
            run_id: String::from(RUN_ID),
            entry_id: String::from(USER_ENTRY),
            message_kind: String::from("user"),
            head_hash: String::from(SAFE_HEAD),
            durable_cursor: FIRST_CURSOR,
        },
        crate::telemetry::RuntimeEvent::ToolRequested {
            request: tool_request(),
        },
        crate::telemetry::RuntimeEvent::Cancelled {
            reason: String::from("correlated cancellation observed"),
        },
        crate::telemetry::RuntimeEvent::Shutdown {
            status: crate::RuntimeTerminalKind::Stopped,
        },
    ];
    let first = events.iter().fold(CodingSessionFrontendState::default(), |state, event| {
        reduce_coding_session_frontend_event(&state, event)
    });
    let second = events.iter().fold(CodingSessionFrontendState::default(), |state, event| {
        reduce_coding_session_frontend_event(&state, event)
    });
    assert_eq!(first, second);
    assert_eq!(first.queue_count, 1);
    assert_eq!(first.committed_message_count, 1);
    assert_eq!(first.active_tool_correlation.as_deref(), Some(TOOL_CALL_ID));
    assert!(first.cancellation_observed);
    assert_eq!(first.terminal_status.as_deref(), Some("stopped"));
}

fn state(profile: CodingSessionQueueProfile, resume_required: bool) -> CodingSessionState {
    new_coding_session_state(config(profile, resume_required), queue_policy()).expect("state")
}

fn config(profile: CodingSessionQueueProfile, resume_required: bool) -> CodingSessionConfig {
    CodingSessionConfig {
        session_id: String::from(SESSION_ID),
        branch_id: String::from(BRANCH_ID),
        queue_profile: profile,
        services: CodingSessionServiceSet::all_available(),
        authority_profile: authority_profile(SAFE_HEAD),
        authority_authorization_identity: String::from(POLICY_HASH),
        resume_required,
    }
}

fn authority_profile(permission_identity: &str) -> crate::session_authority_profile::SessionRuntimeAuthorityProfile {
    let classes = crate::session_authority_profile::SessionAuthorityComponentClass::ALL;
    let identities = [
        POLICY_HASH,
        permission_identity,
        POLICY_HASH,
        POLICY_HASH,
        POLICY_HASH,
        POLICY_HASH,
        POLICY_HASH,
        POLICY_HASH,
        POLICY_HASH,
        POLICY_HASH,
    ];
    let components = classes
        .iter()
        .zip(identities.iter())
        .map(|(class, identity)| crate::session_authority_profile::SessionAuthorityComponentDraft {
            class: *class,
            identity,
        })
        .collect::<Vec<_>>();
    let profile = crate::session_authority_profile::build_session_authority_profile(
        crate::session_authority_profile::SessionAuthorityProfileDraft {
            components: components.as_slice(),
        },
    )
    .expect("authority profile");
    assert_eq!(profile.components().len(), classes.len());
    assert_eq!(
        profile
            .component(crate::session_authority_profile::SessionAuthorityComponentClass::Permission)
            .expect("permission component")
            .identity(),
        permission_identity
    );
    profile
}

fn queue_policy() -> crate::mailbox::MessageQueuePolicy {
    crate::mailbox::MessageQueuePolicy::new(crate::mailbox::MessageQueuePolicyDraft {
        max_items: QUEUE_MAX_ITEMS,
        allowed_sources: &["coding-session-host"],
        policy_label: "coding-session-queue",
        policy_hash: POLICY_HASH,
        allow_queue_mutations: true,
        allow_interrupts: true,
    })
    .expect("queue policy")
}

fn input(body: &str) -> CodingSessionInput {
    CodingSessionInput {
        body: String::from(body),
        posture: CodingSessionInputPosture::ExpandedAndAdmitted,
    }
}

fn committed_prompt() -> CodingSessionTransition {
    committed_prompt_with_profile(CodingSessionQueueProfile::Sequential)
}

fn committed_prompt_with_profile(profile: CodingSessionQueueProfile) -> CodingSessionTransition {
    let initial = state(profile, false);
    let prompt = reduce_coding_session_command(&initial, &CodingSessionCommand::Prompt {
        run_id: String::from(RUN_ID),
        entry_id: String::from(USER_ENTRY),
        input: input(PROMPT),
    })
    .expect("prompt");
    commit_pending(&prompt.state)
}

fn completed_session() -> CodingSessionState {
    let prompt = committed_prompt();
    let provider = provider_final(&prompt.state, RESPONSE);
    commit_pending(&provider.state).state
}

fn commit_pending(state: &CodingSessionState) -> CodingSessionTransition {
    let request = match state.pending_effect().expect("pending commit") {
        CodingSessionEffectRequest::Commit(request) => request,
        other => panic!("expected commit, got {other:?}"),
    };
    assert!(!request.messages().is_empty());
    let message_count = u64::try_from(request.messages().len()).expect("message count");
    let cursor = state.durable_cursor().checked_add(message_count).expect("cursor");
    assert!(cursor > state.durable_cursor());
    let head = format!("{cursor:064x}");
    reduce_coding_session_outcome(state, &CodingSessionEffectOutcome::CommitSucceeded {
        effect_id: String::from(request.effect_id()),
        new_head_hash: head,
        active_leaf_id: String::from(request.proposed_active_leaf_id()),
        durable_cursor: cursor,
        synchronized: true,
    })
    .expect("commit outcome")
}

fn provider_final(state: &CodingSessionState, response: &str) -> CodingSessionTransition {
    let effect_id = match state.pending_effect().expect("provider") {
        CodingSessionEffectRequest::Provider(request) => request.effect_id(),
        other => panic!("expected provider, got {other:?}"),
    };
    reduce_coding_session_outcome(state, &CodingSessionEffectOutcome::ProviderSucceeded {
        effect_id: String::from(effect_id),
        reply: CodingSessionProviderReply::Final(model_response(response)),
    })
    .expect("provider outcome")
}

fn provider_tool_call(state: &CodingSessionState) -> CodingSessionTransition {
    let effect_id = match state.pending_effect().expect("provider") {
        CodingSessionEffectRequest::Provider(request) => request.effect_id(),
        other => panic!("expected provider, got {other:?}"),
    };
    reduce_coding_session_outcome(state, &CodingSessionEffectOutcome::ProviderSucceeded {
        effect_id: String::from(effect_id),
        reply: CodingSessionProviderReply::ToolCall {
            assistant_response: model_response(TOOL_ASSISTANT),
            tool_request: tool_request(),
        },
    })
    .expect("provider tool outcome")
}

fn tool_success(state: &CodingSessionState) -> CodingSessionTransition {
    let effect_id = match state.pending_effect().expect("tool") {
        CodingSessionEffectRequest::Tool(request) => request.effect_id(),
        other => panic!("expected tool, got {other:?}"),
    };
    reduce_coding_session_outcome(state, &CodingSessionEffectOutcome::ToolSucceeded {
        effect_id: String::from(effect_id),
        response: tool_response(),
    })
    .expect("tool outcome")
}

#[derive(Clone, Copy)]
struct QueueCommandDraft<'a> {
    item_id: &'a str,
    entry_id: &'a str,
    body: &'a str,
    is_steering: bool,
}

fn failed_effect(
    state: &CodingSessionState,
    service: CodingSessionEffectService,
    issue: CodingSessionServiceIssue,
) -> CodingSessionTransition {
    let effect_id = String::from(state.pending_effect().expect("pending effect").effect_id());
    reduce_coding_session_outcome(state, &CodingSessionEffectOutcome::Failed {
        effect_id,
        service,
        issue,
    })
    .expect("failed effect transition")
}

fn assert_blocked(transition: &CodingSessionTransition, issue: CodingSessionIssue) {
    assert_eq!(transition.state.terminal(), Some(CodingSessionTerminalKind::Blocked));
    assert_eq!(transition.state.last_issue(), Some(issue));
    assert!(transition.effect.is_none());
}

fn queue_command(state: &CodingSessionState, draft: QueueCommandDraft<'_>) -> CodingSessionTransition {
    let command = if draft.is_steering {
        CodingSessionCommand::Steer {
            item_id: String::from(draft.item_id),
            entry_id: String::from(draft.entry_id),
            input: input(draft.body),
        }
    } else {
        CodingSessionCommand::FollowUp {
            item_id: String::from(draft.item_id),
            entry_id: String::from(draft.entry_id),
            input: input(draft.body),
        }
    };
    reduce_coding_session_command(state, &command).expect("queue command")
}

fn model_response(value: &str) -> animus_adapters::ModelResponse {
    animus_adapters::ModelResponse::new(value).expect("model response")
}

fn tool_request() -> animus_adapters::ToolRequest {
    let arguments =
        animus_adapters::tool::PreparedArguments::prepare(&animus_adapters::tool::ArgumentPreparationRequest {
            current_schema_identity: TOOL_SCHEMA,
            allowed_field_names: &[TOOL_ARGUMENT],
            compatibility_rules: &[],
            fields: &[animus_adapters::tool::ArgumentFieldDraft {
                name: Some(TOOL_ARGUMENT),
                value: Some(TOOL_ARGUMENT_VALUE),
            }],
        })
        .expect("arguments");
    assert_eq!(arguments.schema_identity(), TOOL_SCHEMA);
    assert_eq!(arguments.fields().len(), 1);
    animus_adapters::ToolRequest::from_draft(&animus_adapters::tool::RequestDraft {
        name: Some(TOOL_NAME),
        arguments: Some(&arguments),
        correlation_id: Some(TOOL_CALL_ID),
        output_policy: animus_adapters::tool::OutputPolicy {
            visible_char_budget: TOOL_VISIBLE_CHARS,
        },
        mutation_targets: &[TOOL_TARGET],
        required_capability_labels: &[TOOL_CAPABILITY],
        approval_posture: "pre_approved",
    })
    .expect("tool request")
}

fn recovered_authority_message() -> CodingSessionMessage {
    let profile = authority_profile(SAFE_HEAD);
    let binding = crate::session_authority_profile::build_session_authority_binding(
        crate::session_authority_profile::SessionAuthorityBindingDraft {
            profile: &profile,
            authorization_identity: POLICY_HASH,
            branch_id: BRANCH_ID,
            prior_head_hash: CODING_SESSION_EMPTY_HEAD_HASH,
            prior_profile_identity: None,
        },
    )
    .expect("authority binding");
    let payload =
        crate::session_authority_profile::encode_session_authority_binding(&binding).expect("authority payload");
    let message = super::core::test_message(super::core::CodingSessionMessageDraft {
        entry_id: AUTHORITY_ENTRY,
        parent_id: None,
        run_id: RUN_ID,
        kind: CodingSessionMessageKind::AuthorityProfileBinding,
        body: payload.as_str(),
        tool_correlation_id: None,
        tool_name: None,
    })
    .expect("recovered authority");
    assert_eq!(message.kind(), CodingSessionMessageKind::AuthorityProfileBinding);
    assert_eq!(message.parent_id(), None);
    message
}

fn recovered_user_message() -> CodingSessionMessage {
    super::core::test_message(super::core::CodingSessionMessageDraft {
        entry_id: USER_ENTRY,
        parent_id: Some(AUTHORITY_ENTRY),
        run_id: RUN_ID,
        kind: CodingSessionMessageKind::User,
        body: PROMPT,
        tool_correlation_id: None,
        tool_name: None,
    })
    .expect("recovered user")
}

fn tool_response() -> animus_adapters::ToolResponse {
    let bounded = animus_adapters::tool::BoundedOutput::new(
        &animus_adapters::tool::OutputDraft {
            body: TOOL_OUTPUT,
            full_output_reference: None,
            hash_requirement: animus_adapters::tool::HashRequirement::Required,
        },
        animus_adapters::tool::OutputPolicy {
            visible_char_budget: TOOL_VISIBLE_CHARS,
        },
    )
    .expect("bounded output");
    animus_adapters::ToolResponse::from_bounded(bounded, animus_adapters::tool::OutputRedactionClass::Safe)
}

fn assert_issue<T>(result: Result<T, CodingSessionError>, expected: CodingSessionIssue) {
    match result {
        Ok(_value) => panic!("expected failure"),
        Err(error) => assert_eq!(error.issue(), expected),
    }
}

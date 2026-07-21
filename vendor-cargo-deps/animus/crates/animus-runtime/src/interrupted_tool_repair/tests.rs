use super::*;

const SESSION_ID: &str = "session.one";
const BRANCH_ID: &str = "branch.active";
const RUN_ID: &str = "run.one";
const OTHER_RUN_ID: &str = "run.other";
const CALL_ONE: &str = "call.one";
const CALL_TWO: &str = "call.two";
const ENTRY_USER: &str = "entry.user";
const ENTRY_CALL: &str = "entry.call";
const ENTRY_RESULT: &str = "entry.result";
const ENTRY_LATER: &str = "entry.later";
const PRIVATE_PROMPT: &str = "RAW_PROMPT private";
const PRIVATE_ARGUMENTS: &str = "tool arguments private";
const PRIVATE_OUTPUT: &str = "tool output private";
const PRIVATE_PROVIDER: &str = "provider payload private";
const PRIVATE_CREDENTIAL: &str = "API_KEY=private";
const PRIVATE_PATH: &str = "/home/private/session";
const PRIVATE_EXCEPTION: &str = "exception body private";
const PRIVATE_TRANSCRIPT: &str = "full transcript private";
const ONE_ACTION: usize = 1;
const TWO_ACTIONS: usize = 2;
const ONE_ACTION_COUNT: u32 = 1;
const ONE_MATCHED_CALL: u32 = 1;
const ONE_EXCESS_ITEM: usize = 1;
const BLAKE3_HEX_CHARS: usize = 64;

#[test]
fn clean_branch_needs_no_repair_and_provider_gate_is_ready() {
    // r[verify agent_harness_runtime.tau_tool_repair.invariant]
    let snapshot = clean_snapshot();
    let diagnosis = diagnose_interrupted_tool_branch(&snapshot).expect("clean branch");
    let gate = evaluate_interrupted_tool_provider_gate(&diagnosis, None, None);

    assert_eq!(diagnosis.status(), InterruptedToolDiagnosisStatus::Clean);
    assert_eq!(diagnosis.matched_call_count(), ONE_MATCHED_CALL);
    assert!(diagnosis.unmatched().is_empty());
    assert_eq!(gate.status(), InterruptedToolProviderGateStatus::ReadyClean);
    assert!(gate.provider_dispatch_allowed());
}

#[test]
fn tail_gap_plans_applies_and_projects_canonical_failure() {
    // r[verify agent_harness_runtime.tau_tool_repair.plan]
    // r[verify agent_harness_runtime.tau_tool_repair.provider_gate]
    let snapshot = tail_gap_snapshot();
    let diagnosis = diagnose_interrupted_tool_branch(&snapshot).expect("repairable branch");
    let plan = plan_interrupted_tool_repair(&snapshot, &diagnosis, &[]).expect("repair plan");
    let request = interrupted_tool_repair_application_request(&plan);
    let outcome = passing_application(&plan);
    let gate = evaluate_interrupted_tool_provider_gate(&diagnosis, Some(&plan), Some(&outcome));
    let messages =
        project_interrupted_tool_repair_results(&plan, &outcome).expect("provider projection");

    assert_eq!(
        diagnosis.status(),
        InterruptedToolDiagnosisStatus::Repairable
    );
    assert_eq!(plan.mode(), InterruptedToolRepairMode::TailAppend);
    assert_eq!(plan.actions().len(), ONE_ACTION);
    assert_eq!(
        plan.actions()[0].reason(),
        InterruptedToolReason::UnknownInterruption
    );
    assert_eq!(
        request.effect,
        InterruptedToolRepairEffect::AppendSyntheticFailures
    );
    assert_eq!(request.session_id, SESSION_ID);
    assert_eq!(request.branch_id, BRANCH_ID);
    assert_eq!(request.repair_anchor_entry_id, ENTRY_CALL);
    assert_eq!(request.actions.len(), ONE_ACTION);
    assert_eq!(request.actions[0].call_id(), CALL_ONE);
    assert_eq!(request.action_hashes.len(), ONE_ACTION);
    assert_eq!(
        outcome.status(),
        InterruptedToolRepairApplicationStatus::Applied
    );
    assert_eq!(
        gate.status(),
        InterruptedToolProviderGateStatus::ReadyRepaired
    );
    assert!(gate.provider_dispatch_allowed());
    assert_eq!(messages.len(), ONE_ACTION);
    assert_eq!(
        messages[0].role(),
        animus_adapters::CanonicalModelRole::Tool
    );
    assert_eq!(messages[0].tool_call_id(), Some(CALL_ONE));
}

#[test]
fn partial_batch_preserves_match_and_repairs_each_missing_call() {
    let snapshot = partial_batch_snapshot();
    let diagnosis = diagnose_interrupted_tool_branch(&snapshot).expect("partial batch");
    let plan = plan_interrupted_tool_repair(&snapshot, &diagnosis, &[]).expect("partial plan");

    assert_eq!(diagnosis.matched_call_count(), ONE_MATCHED_CALL);
    assert_eq!(diagnosis.unmatched().len(), ONE_ACTION);
    assert_eq!(diagnosis.unmatched()[0].call_id(), CALL_TWO);
    assert_eq!(plan.actions().len(), ONE_ACTION);
    assert_eq!(plan.actions()[0].call_id(), CALL_TWO);
    assert_eq!(plan.mode(), InterruptedToolRepairMode::TailAppend);
}

#[test]
fn multiple_missing_calls_have_transcript_ordered_stable_actions() {
    let snapshot = multi_gap_snapshot();
    let diagnosis = diagnose_interrupted_tool_branch(&snapshot).expect("multi gap");
    let first = plan_interrupted_tool_repair(&snapshot, &diagnosis, &[]).expect("first plan");
    let second = plan_interrupted_tool_repair(&snapshot, &diagnosis, &[]).expect("second plan");

    assert_eq!(first, second);
    assert_eq!(first.actions().len(), TWO_ACTIONS);
    assert_eq!(first.actions()[0].call_id(), CALL_ONE);
    assert_eq!(first.actions()[1].call_id(), CALL_TWO);
    assert_ne!(
        first.actions()[0].action_hash(),
        first.actions()[1].action_hash()
    );
}

#[test]
fn interruption_reasons_require_exact_correlated_evidence() {
    // r[verify agent_harness_runtime.tau_tool_repair.reason]
    let snapshot = tail_gap_snapshot();
    let diagnosis = diagnose_interrupted_tool_branch(&snapshot).expect("gap");
    let gap = &diagnosis.unmatched()[0];
    let cases = [
        (
            InterruptedToolObservationKind::UserCancellation,
            InterruptedToolReason::UserCancelled,
        ),
        (
            InterruptedToolObservationKind::HostCancellation,
            InterruptedToolReason::HostCancelled,
        ),
        (
            InterruptedToolObservationKind::UncleanShutdown,
            InterruptedToolReason::UncleanShutdown,
        ),
    ];
    for (kind, expected) in cases {
        let observation = observation(kind, (RUN_ID, CALL_ONE));
        let reason = reduce_interrupted_tool_reason(gap, &[observation]).expect("reason");
        assert_eq!(reason, expected);
    }
    assert_eq!(
        reduce_interrupted_tool_reason(gap, &[]).expect("unknown reason"),
        InterruptedToolReason::UnknownInterruption
    );
}

#[test]
fn historical_gap_preserves_suffix_and_uses_new_branch_mode() {
    // r[verify agent_harness_runtime.tau_tool_repair.append_only]
    let snapshot = historical_gap_snapshot();
    let diagnosis = diagnose_interrupted_tool_branch(&snapshot).expect("historical gap");
    let plan = plan_interrupted_tool_repair(&snapshot, &diagnosis, &[]).expect("historical plan");

    assert_eq!(plan.mode(), InterruptedToolRepairMode::HistoricalBranch);
    assert_eq!(plan.repair_anchor_entry_id(), ENTRY_CALL);
    assert_eq!(plan.affected_suffix_entry_hashes(), &[hash(ENTRY_LATER)]);
    assert_eq!(plan.old_leaf_hash(), hash(ENTRY_LATER));
    assert_ne!(plan.proposed_new_leaf_hash(), plan.old_leaf_hash());
}

#[test]
fn repeated_application_is_idempotent() {
    let snapshot = tail_gap_snapshot();
    let diagnosis = diagnose_interrupted_tool_branch(&snapshot).expect("gap");
    let plan = plan_interrupted_tool_repair(&snapshot, &diagnosis, &[]).expect("plan");
    let action_hashes = action_hash_refs(&plan);
    let previous = [plan.plan_hash()];
    let draft = InterruptedToolRepairApplicationDraft {
        plan_hash: plan.plan_hash(),
        observed_old_leaf_hash: plan.old_leaf_hash(),
        observed_new_leaf_hash: plan.proposed_new_leaf_hash(),
        committed_action_hashes: action_hashes.as_slice(),
        previously_applied_plan_hashes: &previous,
        durability: InterruptedToolDurability::Durable,
        replay_state: InterruptedToolReplayState::Clean,
        executor_observation: InterruptedToolExecutorObservation::NotInvoked,
    };
    let outcome = evaluate_interrupted_tool_repair_application(&plan, &draft)
        .expect("idempotent application");

    assert_eq!(
        outcome.status(),
        InterruptedToolRepairApplicationStatus::AlreadyApplied
    );
    assert!(outcome.status().permits_provider_continuation());
}

#[test]
fn application_evidence_collections_are_bounded() {
    let snapshot = tail_gap_snapshot();
    let diagnosis = diagnose_interrupted_tool_branch(&snapshot).expect("gap");
    let plan = plan_interrupted_tool_repair(&snapshot, &diagnosis, &[]).expect("plan");
    let excessive_action_count = super::data::MAX_TOTAL_TOOL_CALLS
        .checked_add(ONE_EXCESS_ITEM)
        .expect("bounded test action count");
    let excessive_plan_count = super::data::MAX_PREVIOUSLY_APPLIED_PLANS
        .checked_add(ONE_EXCESS_ITEM)
        .expect("bounded test plan count");
    let excessive_action_hashes = vec![plan.plan_hash(); excessive_action_count];
    let excessive_plan_hashes = vec![plan.plan_hash(); excessive_plan_count];
    let drafts = [
        InterruptedToolRepairApplicationDraft {
            plan_hash: plan.plan_hash(),
            observed_old_leaf_hash: plan.old_leaf_hash(),
            observed_new_leaf_hash: plan.proposed_new_leaf_hash(),
            committed_action_hashes: excessive_action_hashes.as_slice(),
            previously_applied_plan_hashes: &[],
            durability: InterruptedToolDurability::Durable,
            replay_state: InterruptedToolReplayState::Clean,
            executor_observation: InterruptedToolExecutorObservation::NotInvoked,
        },
        InterruptedToolRepairApplicationDraft {
            plan_hash: plan.plan_hash(),
            observed_old_leaf_hash: plan.old_leaf_hash(),
            observed_new_leaf_hash: plan.proposed_new_leaf_hash(),
            committed_action_hashes: &[],
            previously_applied_plan_hashes: excessive_plan_hashes.as_slice(),
            durability: InterruptedToolDurability::Durable,
            replay_state: InterruptedToolReplayState::Clean,
            executor_observation: InterruptedToolExecutorObservation::NotInvoked,
        },
    ];

    assert_eq!(drafts.len(), TWO_ACTIONS);
    for draft in drafts {
        assert_eq!(
            evaluate_interrupted_tool_repair_application(&plan, &draft)
                .expect_err("evidence collection bound")
                .issue(),
            InterruptedToolRepairIssue::InvalidInput
        );
    }
}

#[test]
fn plan_and_receipt_hashes_are_stable_and_metadata_only() {
    // r[verify agent_harness_runtime.tau_tool_repair.receipts]
    let snapshot = tail_gap_snapshot();
    let diagnosis = diagnose_interrupted_tool_branch(&snapshot).expect("gap");
    let plan = plan_interrupted_tool_repair(&snapshot, &diagnosis, &[]).expect("plan");
    let outcome = passing_application(&plan);
    let first = project_interrupted_tool_repair_receipt(
        &plan,
        &outcome,
        InterruptedToolRepairSensitiveDraft::default(),
    )
    .expect("first receipt");
    let second = project_interrupted_tool_repair_receipt(
        &plan,
        &outcome,
        InterruptedToolRepairSensitiveDraft::default(),
    )
    .expect("second receipt");

    assert_eq!(first, second);
    assert_eq!(first.schema(), INTERRUPTED_TOOL_REPAIR_SCHEMA);
    assert_eq!(first.action_count(), ONE_ACTION_COUNT);
    assert_eq!(
        first.reason_classes(),
        &[InterruptedToolReason::UnknownInterruption]
    );
    assert!(first.redaction_marker().contains("omitted"));
    assert!(first.non_claim().contains("does not prove"));
    assert_eq!(first.receipt_hash().len(), BLAKE3_HEX_CHARS);
}

#[test]
fn duplicate_call_and_result_bindings_fail_closed() {
    // r[verify agent_harness_runtime.tau_tool_repair.verification]
    let duplicate_call = snapshot(vec![
        simple_entry(ENTRY_USER, InterruptedToolEntryKind::UserMessage),
        call_entry(ENTRY_CALL, &[CALL_ONE]),
        call_entry("entry.call.duplicate", &[CALL_ONE]),
    ]);
    let duplicate_result = snapshot(vec![
        simple_entry(ENTRY_USER, InterruptedToolEntryKind::UserMessage),
        call_entry(ENTRY_CALL, &[CALL_ONE]),
        result_entry(ENTRY_RESULT, (CALL_ONE, tool_hash(CALL_ONE))),
        result_entry("entry.result.duplicate", (CALL_ONE, tool_hash(CALL_ONE))),
    ]);

    assert_eq!(
        diagnose_interrupted_tool_branch(&duplicate_call)
            .expect_err("duplicate call")
            .issue(),
        InterruptedToolRepairIssue::DuplicateCall
    );
    assert_eq!(
        diagnose_interrupted_tool_branch(&duplicate_result)
            .expect_err("duplicate result")
            .issue(),
        InterruptedToolRepairIssue::DuplicateResult
    );
}

#[test]
fn reversed_and_mismatched_results_fail_closed() {
    let reversed = snapshot(vec![
        simple_entry(ENTRY_USER, InterruptedToolEntryKind::UserMessage),
        result_entry(ENTRY_RESULT, (CALL_ONE, tool_hash(CALL_ONE))),
    ]);
    let mismatched = snapshot(vec![
        call_entry(ENTRY_CALL, &[CALL_ONE]),
        result_entry(ENTRY_RESULT, (CALL_ONE, tool_hash(CALL_TWO))),
    ]);
    let mut wrong_run_result = result_entry(ENTRY_RESULT, (CALL_ONE, tool_hash(CALL_ONE)));
    wrong_run_result.run_id = OTHER_RUN_ID.to_string();
    let wrong_run = snapshot(vec![call_entry(ENTRY_CALL, &[CALL_ONE]), wrong_run_result]);

    assert_eq!(
        diagnose_interrupted_tool_branch(&reversed)
            .expect_err("result before call")
            .issue(),
        InterruptedToolRepairIssue::ResultBeforeCall
    );
    assert_eq!(
        diagnose_interrupted_tool_branch(&mismatched)
            .expect_err("mismatched tool")
            .issue(),
        InterruptedToolRepairIssue::ToolMismatch
    );
    assert_eq!(
        diagnose_interrupted_tool_branch(&wrong_run)
            .expect_err("mismatched run")
            .issue(),
        InterruptedToolRepairIssue::RunMismatch
    );
}

#[test]
fn stale_leaf_unsupported_entry_and_missing_anchor_fail_closed() {
    let mut stale = tail_gap_snapshot();
    stale.snapshot_leaf_hash = hash("stale.leaf");
    assert_eq!(
        diagnose_interrupted_tool_branch(&stale)
            .expect_err("stale leaf")
            .issue(),
        InterruptedToolRepairIssue::StaleLeaf
    );

    let invalid_entry_case = snapshot(vec![InterruptedToolBranchEntry {
        kind: InterruptedToolEntryKind::Unsupported,
        ..simple_entry(ENTRY_USER, InterruptedToolEntryKind::UserMessage)
    }]);
    assert_eq!(
        diagnose_interrupted_tool_branch(&invalid_entry_case)
            .expect_err("unsupported entry")
            .issue(),
        InterruptedToolRepairIssue::UnsupportedEntry
    );

    let original = tail_gap_snapshot();
    let diagnosis = diagnose_interrupted_tool_branch(&original).expect("diagnosis");
    let changed = InterruptedToolBranchSnapshot {
        session_id: "session.changed".to_string(),
        ..original
    };
    assert_eq!(
        plan_interrupted_tool_repair(&changed, &diagnosis, &[])
            .expect_err("missing original anchor")
            .issue(),
        InterruptedToolRepairIssue::MissingRepairAnchor
    );
}

#[test]
fn conflicting_and_false_user_cancellation_do_not_invent_cause() {
    let snapshot = tail_gap_snapshot();
    let diagnosis = diagnose_interrupted_tool_branch(&snapshot).expect("gap");
    let gap = &diagnosis.unmatched()[0];
    let unrelated = observation(
        InterruptedToolObservationKind::UserCancellation,
        (OTHER_RUN_ID, CALL_ONE),
    );
    assert_eq!(
        reduce_interrupted_tool_reason(gap, &[unrelated]).expect("unrelated evidence"),
        InterruptedToolReason::UnknownInterruption
    );

    let conflict = [
        observation(
            InterruptedToolObservationKind::UserCancellation,
            (RUN_ID, CALL_ONE),
        ),
        observation(
            InterruptedToolObservationKind::HostCancellation,
            (RUN_ID, CALL_ONE),
        ),
    ];
    assert_eq!(
        reduce_interrupted_tool_reason(gap, &conflict)
            .expect_err("conflicting cause")
            .issue(),
        InterruptedToolRepairIssue::ConflictingEvidence
    );
}

#[test]
fn partial_store_replay_and_tool_execution_block_provider_dispatch() {
    // r[verify agent_harness_runtime.tau_tool_repair.provider_gate]
    let snapshot = tail_gap_snapshot();
    let diagnosis = diagnose_interrupted_tool_branch(&snapshot).expect("gap");
    let plan = plan_interrupted_tool_repair(&snapshot, &diagnosis, &[]).expect("plan");
    let action_hashes = action_hash_refs(&plan);
    let statuses = [
        application_with(
            &plan,
            ApplicationObservation {
                committed_action_hashes: &[],
                durability: InterruptedToolDurability::Partial,
                replay_state: InterruptedToolReplayState::NotValidated,
                executor_observation: InterruptedToolExecutorObservation::NotInvoked,
            },
        ),
        application_with(
            &plan,
            ApplicationObservation {
                committed_action_hashes: &[],
                durability: InterruptedToolDurability::StoreFailure,
                replay_state: InterruptedToolReplayState::NotValidated,
                executor_observation: InterruptedToolExecutorObservation::NotInvoked,
            },
        ),
        application_with(
            &plan,
            ApplicationObservation {
                committed_action_hashes: action_hashes.as_slice(),
                durability: InterruptedToolDurability::Durable,
                replay_state: InterruptedToolReplayState::Invalid,
                executor_observation: InterruptedToolExecutorObservation::NotInvoked,
            },
        ),
        application_with(
            &plan,
            ApplicationObservation {
                committed_action_hashes: action_hashes.as_slice(),
                durability: InterruptedToolDurability::Durable,
                replay_state: InterruptedToolReplayState::Clean,
                executor_observation: InterruptedToolExecutorObservation::Invoked,
            },
        ),
    ];
    let expected = [
        InterruptedToolRepairApplicationStatus::BlockedPartial,
        InterruptedToolRepairApplicationStatus::BlockedStore,
        InterruptedToolRepairApplicationStatus::BlockedReplay,
        InterruptedToolRepairApplicationStatus::BlockedToolExecution,
    ];
    for (outcome, expected_status) in statuses.into_iter().zip(expected) {
        assert_eq!(outcome.status(), expected_status);
        assert!(!outcome.status().permits_provider_continuation());
        let gate = evaluate_interrupted_tool_provider_gate(&diagnosis, Some(&plan), Some(&outcome));
        assert!(!gate.provider_dispatch_allowed());
        assert!(project_interrupted_tool_repair_results(&plan, &outcome).is_err());
    }
}

#[test]
fn unrepaired_branch_is_rejected_before_provider_projection() {
    let snapshot = tail_gap_snapshot();
    let diagnosis = diagnose_interrupted_tool_branch(&snapshot).expect("gap");
    let gate = evaluate_interrupted_tool_provider_gate(&diagnosis, None, None);

    assert_eq!(
        gate.status(),
        InterruptedToolProviderGateStatus::BlockedRepairRequired
    );
    assert!(!gate.provider_dispatch_allowed());
}

#[test]
fn receipt_rejects_every_private_material_class() {
    let snapshot = tail_gap_snapshot();
    let diagnosis = diagnose_interrupted_tool_branch(&snapshot).expect("gap");
    let plan = plan_interrupted_tool_repair(&snapshot, &diagnosis, &[]).expect("plan");
    let outcome = passing_application(&plan);
    let leaks = [
        InterruptedToolRepairSensitiveDraft {
            raw_prompt: Some(PRIVATE_PROMPT),
            ..InterruptedToolRepairSensitiveDraft::default()
        },
        InterruptedToolRepairSensitiveDraft {
            raw_tool_arguments: Some(PRIVATE_ARGUMENTS),
            ..InterruptedToolRepairSensitiveDraft::default()
        },
        InterruptedToolRepairSensitiveDraft {
            raw_tool_output: Some(PRIVATE_OUTPUT),
            ..InterruptedToolRepairSensitiveDraft::default()
        },
        InterruptedToolRepairSensitiveDraft {
            provider_payload: Some(PRIVATE_PROVIDER),
            ..InterruptedToolRepairSensitiveDraft::default()
        },
        InterruptedToolRepairSensitiveDraft {
            credential: Some(PRIVATE_CREDENTIAL),
            ..InterruptedToolRepairSensitiveDraft::default()
        },
        InterruptedToolRepairSensitiveDraft {
            private_path: Some(PRIVATE_PATH),
            ..InterruptedToolRepairSensitiveDraft::default()
        },
        InterruptedToolRepairSensitiveDraft {
            exception_body: Some(PRIVATE_EXCEPTION),
            ..InterruptedToolRepairSensitiveDraft::default()
        },
        InterruptedToolRepairSensitiveDraft {
            transcript: Some(PRIVATE_TRANSCRIPT),
            ..InterruptedToolRepairSensitiveDraft::default()
        },
    ];
    for leak in leaks {
        let error = project_interrupted_tool_repair_receipt(&plan, &outcome, leak)
            .expect_err("private receipt material");
        assert_eq!(error.issue(), InterruptedToolRepairIssue::ReceiptLeak);
        let rendered = format!("{error:?}");
        for private in [
            PRIVATE_PROMPT,
            PRIVATE_ARGUMENTS,
            PRIVATE_OUTPUT,
            PRIVATE_PROVIDER,
            PRIVATE_CREDENTIAL,
            PRIVATE_PATH,
            PRIVATE_EXCEPTION,
            PRIVATE_TRANSCRIPT,
        ] {
            assert!(!rendered.contains(private));
        }
    }
}

fn clean_snapshot() -> InterruptedToolBranchSnapshot {
    snapshot(vec![
        simple_entry(ENTRY_USER, InterruptedToolEntryKind::UserMessage),
        call_entry(ENTRY_CALL, &[CALL_ONE]),
        result_entry(ENTRY_RESULT, (CALL_ONE, tool_hash(CALL_ONE))),
    ])
}

fn tail_gap_snapshot() -> InterruptedToolBranchSnapshot {
    snapshot(vec![
        simple_entry(ENTRY_USER, InterruptedToolEntryKind::UserMessage),
        call_entry(ENTRY_CALL, &[CALL_ONE]),
    ])
}

fn partial_batch_snapshot() -> InterruptedToolBranchSnapshot {
    snapshot(vec![
        call_entry(ENTRY_CALL, &[CALL_ONE, CALL_TWO]),
        result_entry(ENTRY_RESULT, (CALL_ONE, tool_hash(CALL_ONE))),
    ])
}

fn multi_gap_snapshot() -> InterruptedToolBranchSnapshot {
    snapshot(vec![call_entry(ENTRY_CALL, &[CALL_ONE, CALL_TWO])])
}

fn historical_gap_snapshot() -> InterruptedToolBranchSnapshot {
    snapshot(vec![
        call_entry(ENTRY_CALL, &[CALL_ONE]),
        simple_entry(ENTRY_LATER, InterruptedToolEntryKind::AssistantMessage),
    ])
}

fn snapshot(entries: Vec<InterruptedToolBranchEntry>) -> InterruptedToolBranchSnapshot {
    let active_leaf_hash = entries
        .last()
        .map_or_else(|| hash("missing"), |entry| entry.entry_hash.clone());
    InterruptedToolBranchSnapshot {
        session_id: SESSION_ID.to_string(),
        branch_id: BRANCH_ID.to_string(),
        active_leaf_hash: active_leaf_hash.clone(),
        snapshot_leaf_hash: active_leaf_hash,
        entries,
    }
}

fn simple_entry(entry_id: &str, kind: InterruptedToolEntryKind) -> InterruptedToolBranchEntry {
    InterruptedToolBranchEntry {
        entry_id: entry_id.to_string(),
        entry_hash: hash(entry_id),
        run_id: RUN_ID.to_string(),
        kind,
        tool_calls: Vec::new(),
        tool_result: None,
    }
}

fn call_entry(entry_id: &str, call_ids: &[&str]) -> InterruptedToolBranchEntry {
    InterruptedToolBranchEntry {
        kind: InterruptedToolEntryKind::AssistantToolCalls,
        tool_calls: call_ids
            .iter()
            .map(|call_id| InterruptedToolCall {
                call_id: (*call_id).to_string(),
                tool_identity_hash: tool_hash(call_id),
            })
            .collect(),
        ..simple_entry(entry_id, InterruptedToolEntryKind::AssistantToolCalls)
    }
}

fn result_entry(entry_id: &str, result: (&str, String)) -> InterruptedToolBranchEntry {
    let (call_id, tool_identity_hash) = result;
    InterruptedToolBranchEntry {
        kind: InterruptedToolEntryKind::ToolResult,
        tool_result: Some(InterruptedToolResult {
            call_id: call_id.to_string(),
            tool_identity_hash,
        }),
        ..simple_entry(entry_id, InterruptedToolEntryKind::ToolResult)
    }
}

fn observation(
    kind: InterruptedToolObservationKind,
    target: (&str, &str),
) -> InterruptedToolObservation {
    let (run_id, call_id) = target;
    InterruptedToolObservation {
        kind,
        run_id: run_id.to_string(),
        affected_call_ids: vec![call_id.to_string()],
    }
}

fn passing_application(
    plan: &InterruptedToolRepairPlan,
) -> InterruptedToolRepairApplicationOutcome {
    let action_hashes = action_hash_refs(plan);
    application_with(
        plan,
        ApplicationObservation {
            committed_action_hashes: action_hashes.as_slice(),
            durability: InterruptedToolDurability::Durable,
            replay_state: InterruptedToolReplayState::Clean,
            executor_observation: InterruptedToolExecutorObservation::NotInvoked,
        },
    )
}

#[derive(Clone, Copy)]
struct ApplicationObservation<'a> {
    committed_action_hashes: &'a [&'a str],
    durability: InterruptedToolDurability,
    replay_state: InterruptedToolReplayState,
    executor_observation: InterruptedToolExecutorObservation,
}

fn application_with(
    plan: &InterruptedToolRepairPlan,
    observation: ApplicationObservation<'_>,
) -> InterruptedToolRepairApplicationOutcome {
    evaluate_interrupted_tool_repair_application(
        plan,
        &InterruptedToolRepairApplicationDraft {
            plan_hash: plan.plan_hash(),
            observed_old_leaf_hash: plan.old_leaf_hash(),
            observed_new_leaf_hash: plan.proposed_new_leaf_hash(),
            committed_action_hashes: observation.committed_action_hashes,
            previously_applied_plan_hashes: &[],
            durability: observation.durability,
            replay_state: observation.replay_state,
            executor_observation: observation.executor_observation,
        },
    )
    .expect("application evidence should validate")
}

fn action_hash_refs(plan: &InterruptedToolRepairPlan) -> Vec<&str> {
    plan.actions()
        .iter()
        .map(InterruptedToolRepairAction::action_hash)
        .collect()
}

fn tool_hash(call_id: &str) -> String {
    hash(format!("tool:{call_id}").as_str())
}

fn hash(value: &str) -> String {
    blake3::hash(value.as_bytes()).to_hex().to_string()
}

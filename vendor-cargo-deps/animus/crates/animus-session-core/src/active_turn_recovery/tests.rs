// Table-driven negative fixtures keep source DTO names and explicit arithmetic cases visible.
#![allow(
    unknown_lints,
    ambiguous_params,
    assertion_density,
    numeric_units,
    path_segment_repetition,
    raw_arithmetic_overflow
)]

#[allow(unused_imports)]
use crate::prelude::*;

#[allow(unused_imports)]
use super::*;

type ControlDirective = animus_adapters::AgentControlCommand;

const SESSION_ID: &str = "session.one";
const BRANCH_ID: &str = "branch.active";
const POLICY_ID: &str = "durable-active-turn";
const RETENTION_CLASS: &str = "host_managed";
const CALL_ID: &str = "call.one";
const CONTROL_RUN_ID: &str = "run.control";
const PAUSE_COMMAND_ID: &str = "command.pause";
const RESUME_COMMAND_ID: &str = "command.resume";
const CANCEL_COMMAND_ID: &str = "command.cancel";
const TURN_SEQUENCE: u64 = 7;
const PRIOR_GENERATION: u64 = 4;
const NEXT_GENERATION: u64 = 5;
const NEXT_STEERING_SEQUENCE: u32 = 1;
const MAX_ENTRIES: u32 = 64;
const MAX_ENTRY_BYTES: u32 = 4_096;
const MAX_AGGREGATE_BYTES: u64 = 65_536;
const MAX_STEERING_ENTRIES: u32 = 16;
const PREPARED_ENTRY_BYTES: u32 = 32;
const ACCEPTED_SEQUENCE: u32 = 0;
const TOOL_CALL_SEQUENCE: u32 = 1;
const TOOL_RESULT_SEQUENCE: u32 = 2;
const TERMINAL_SEQUENCE: u32 = 3;
const ONE_ENTRY: usize = 1;
const THREE_ENTRIES: usize = 3;
const PRIVATE_VALUE: &str = "/home/private/raw prompt API_KEY";

#[test]
fn policy_acceptance_and_provider_gate_are_deterministic() {
    // r[verify agent_harness_runtime.active_turn_recovery.policy]
    // r[verify agent_harness_runtime.active_turn_recovery.journal]
    // r[verify agent_harness_runtime.active_turn_recovery.identity]
    let policy = policy();
    let draft = acceptance_draft();
    let first = plan_active_turn_acceptance(&policy, &draft).expect("acceptance plan");
    let second = plan_active_turn_acceptance(&policy, &draft).expect("stable acceptance plan");
    let observation = committed_observation(
        first.transaction_hash(),
        first.expected_generation(),
        first.next_generation(),
    );
    let outcome =
        evaluate_active_turn_acceptance(&first, &observation).expect("durable acceptance");

    assert_eq!(first, second);
    assert_eq!(first.next_generation(), NEXT_GENERATION);
    assert_eq!(outcome.status(), ActiveTurnTransactionStatus::Committed);
    assert!(outcome.provider_eligible());
    assert!(active_turn_provider_dispatch_allowed(
        &first,
        Some(&outcome)
    ));
}

#[test]
fn provider_dispatch_fails_before_durable_acceptance() {
    // r[verify agent_harness_runtime.active_turn_recovery.verification]
    let plan = plan_active_turn_acceptance(&policy(), &acceptance_draft()).expect("plan");
    assert!(!active_turn_provider_dispatch_allowed(&plan, None));

    let partial = ActiveTurnTransactionObservation {
        complete_manifest: false,
        ..committed_observation(
            plan.transaction_hash(),
            plan.expected_generation(),
            plan.next_generation(),
        )
    };
    let outcome = evaluate_active_turn_acceptance(&plan, &partial).expect("partial evidence");
    assert_eq!(outcome.status(), ActiveTurnTransactionStatus::Partial);
    assert!(!outcome.provider_eligible());
}

#[test]
fn complete_checkpoint_replacement_is_atomic_and_bounded() {
    let policy = policy();
    let acceptance = plan_active_turn_acceptance(&policy, &acceptance_draft()).expect("acceptance");
    let entries = completed_tool_entries();
    let draft = ActiveTurnReplacementDraft {
        turn_hash: acceptance.turn_hash().to_string(),
        expected_generation: NEXT_GENERATION,
        entries,
    };
    let plan = prepare_active_turn_checkpoint_replacement(&policy, &draft).expect("replacement");
    let observation = committed_observation(
        plan.transaction_hash(),
        NEXT_GENERATION,
        NEXT_GENERATION + 1,
    );
    let status = evaluate_active_turn_checkpoint_replacement(&plan, &observation)
        .expect("replacement evidence");

    assert_eq!(plan.entry_hashes().len(), THREE_ENTRIES);
    assert_eq!(status, ActiveTurnTransactionStatus::Committed);
}

#[test]
fn partial_and_overbound_replacements_fail_closed() {
    let policy = policy();
    let acceptance = plan_active_turn_acceptance(&policy, &acceptance_draft()).expect("acceptance");
    let mut missing_first = completed_tool_entries();
    missing_first.remove(0);
    let invalid_order = ActiveTurnReplacementDraft {
        turn_hash: acceptance.turn_hash().to_string(),
        expected_generation: NEXT_GENERATION,
        entries: missing_first,
    };
    assert_eq!(
        prepare_active_turn_checkpoint_replacement(&policy, &invalid_order)
            .expect_err("accepted entry is required")
            .issue(),
        ActiveTurnRecoveryIssue::CorruptJournal
    );

    let oversized = ActiveTurnReplacementDraft {
        turn_hash: acceptance.turn_hash().to_string(),
        expected_generation: NEXT_GENERATION,
        entries: vec![ActiveTurnPreparedEntry {
            prepared_bytes: MAX_ENTRY_BYTES + 1,
            ..accepted_entry()
        }],
    };
    assert_eq!(
        prepare_active_turn_checkpoint_replacement(&policy, &oversized)
            .expect_err("entry bound")
            .issue(),
        ActiveTurnRecoveryIssue::CorruptJournal
    );
}

#[test]
fn durable_steering_evidence_gates_delivery() {
    let policy = policy();
    let acceptance = plan_active_turn_acceptance(&policy, &acceptance_draft()).expect("acceptance");
    let plan = plan_active_turn_steering_append(
        &policy,
        &ActiveTurnSteeringDraft {
            turn_hash: acceptance.turn_hash().to_string(),
            expected_generation: NEXT_GENERATION,
            pending_count: 0,
            steering_sequence: NEXT_STEERING_SEQUENCE,
            steering_body_hash: hash("steering").to_string(),
        },
    )
    .expect("steering plan");
    let observation = committed_observation(plan.append_hash(), NEXT_GENERATION, NEXT_GENERATION);
    assert!(active_turn_steering_delivery_allowed(&plan, &observation).expect("delivery gate"));

    let stale = ActiveTurnTransactionObservation {
        observed_prior_generation: PRIOR_GENERATION,
        ..observation
    };
    assert!(!active_turn_steering_delivery_allowed(&plan, &stale).expect("stale gate"));
}

#[test]
fn pre_provider_recovery_preserves_turn_identity() {
    // r[verify agent_harness_runtime.active_turn_recovery.recovery]
    let (policy, snapshot) = snapshot(
        ActiveTurnRepairState::Clean,
        ActiveTurnTerminalClass::Active,
    );
    let first = plan_active_turn_recovery(&policy, &snapshot).expect("recovery plan");
    let second = plan_active_turn_recovery(&policy, &snapshot).expect("stable recovery plan");

    assert_eq!(first, second);
    assert_eq!(first.action(), ActiveTurnRecoveryAction::ContinueProvider);
    assert!(first.provider_continuation_allowed());
}

#[test]
fn durable_parked_control_blocks_recovery_until_exact_resume() {
    // r[verify agent_harness_runtime.agent_control.runtime.gate]
    // r[verify agent_harness_runtime.agent_control.runtime.recovery]
    // r[verify agent_harness_runtime.agent_control.verification.positive]
    let (policy, mut snapshot) = snapshot(
        ActiveTurnRepairState::Clean,
        ActiveTurnTerminalClass::Active,
    );
    let run_id = animus_adapters::AgentControlRunId::new(CONTROL_RUN_ID).expect("run id");
    let initial = crate::AgentControlState::new(run_id.clone());
    let pause = ControlDirective::new(
        run_id.clone(),
        animus_adapters::AgentControlCommandId::new(PAUSE_COMMAND_ID).expect("pause command id"),
        initial.revision(),
        animus_adapters::AgentControlAction::Pause,
    );
    let requested = crate::reduce_agent_control_command(&initial, &pause)
        .expect("pause transition")
        .into_state();
    let checkpoint = animus_adapters::AgentControlCheckpoint::new(
        run_id.clone(),
        animus_adapters::AgentControlCheckpointKind::BeforeModel,
    );
    let paused = crate::reduce_agent_control_checkpoint(&requested, &checkpoint)
        .expect("paused checkpoint")
        .into_state();
    let parked_kind = append_control_entry(&policy, &mut snapshot, &paused);
    assert_eq!(parked_kind, ActiveTurnJournalEntryKind::ControlParked);

    let parked_plan = plan_active_turn_recovery(&policy, &snapshot).expect("parked plan");
    assert_eq!(parked_plan.action(), ActiveTurnRecoveryAction::Parked);
    assert!(!parked_plan.provider_continuation_allowed());

    let resume = ControlDirective::new(
        run_id,
        animus_adapters::AgentControlCommandId::new(RESUME_COMMAND_ID).expect("resume command id"),
        paused.revision(),
        animus_adapters::AgentControlAction::ResumeRun,
    );
    let running = crate::reduce_agent_control_command(&paused, &resume)
        .expect("resume transition")
        .into_state();
    let running_kind = append_control_entry(&policy, &mut snapshot, &running);
    assert_eq!(running_kind, ActiveTurnJournalEntryKind::ControlRunning);

    let running_plan = plan_active_turn_recovery(&policy, &snapshot).expect("running plan");
    assert_eq!(
        running_plan.action(),
        ActiveTurnRecoveryAction::ContinueProvider
    );
    assert!(running_plan.provider_continuation_allowed());
}

#[test]
fn cancelling_control_cannot_be_persisted_as_parked_or_runnable() {
    // r[verify agent_harness_runtime.agent_control.verification.negative]
    let (policy, snapshot) = snapshot(
        ActiveTurnRepairState::Clean,
        ActiveTurnTerminalClass::Active,
    );
    let run_id = animus_adapters::AgentControlRunId::new(CONTROL_RUN_ID).expect("run id");
    let initial = crate::AgentControlState::new(run_id.clone());
    let cancel = ControlDirective::new(
        run_id,
        animus_adapters::AgentControlCommandId::new(CANCEL_COMMAND_ID).expect("cancel command id"),
        initial.revision(),
        animus_adapters::AgentControlAction::Cancel,
    );
    let cancelling = crate::reduce_agent_control_command(&initial, &cancel)
        .expect("cancel transition")
        .into_state();
    let sequence = u32::try_from(snapshot.entries.len()).expect("bounded entry count");

    let failure = prepare_active_turn_control_entry(
        &policy,
        ActiveTurnControlEntryDraft {
            turn_hash: snapshot.turn_hash.as_str(),
            sequence,
            prepared_bytes: PREPARED_ENTRY_BYTES,
            control: &cancelling,
        },
    )
    .expect_err("cancellation uses the dedicated recovery field");
    assert_eq!(failure.issue(), ActiveTurnRecoveryIssue::InvalidInput);
}

#[test]
fn interrupted_tool_repair_is_delegated_without_tool_request() {
    let (policy, mut snapshot) = snapshot(
        ActiveTurnRepairState::RepairRequired,
        ActiveTurnTerminalClass::Active,
    );
    snapshot.entries = vec![accepted_entry(), tool_call_entry()];
    snapshot.checkpoint_hash = active_turn_checkpoint_hash(
        &policy,
        snapshot.turn_hash.as_str(),
        snapshot.generation,
        snapshot.entries.as_slice(),
    )
    .expect("checkpoint");
    let plan = plan_active_turn_recovery(&policy, &snapshot).expect("repair delegation");

    assert_eq!(
        plan.action(),
        ActiveTurnRecoveryAction::PlanInterruptedToolRepair
    );
    assert!(!plan.provider_continuation_allowed());
}

#[test]
fn real_tool_result_is_preserved_without_rerun() {
    let (policy, snapshot) = snapshot(
        ActiveTurnRepairState::Clean,
        ActiveTurnTerminalClass::Active,
    );
    assert_eq!(
        snapshot.entries[TOOL_RESULT_SEQUENCE as usize].kind,
        ActiveTurnJournalEntryKind::ToolResult
    );
    let plan = plan_active_turn_recovery(&policy, &snapshot).expect("real result recovery");
    assert_eq!(plan.action(), ActiveTurnRecoveryAction::ContinueProvider);
}

#[test]
fn authority_cancellation_and_blocked_repair_stop_continuation() {
    let (policy, base) = snapshot(
        ActiveTurnRepairState::Clean,
        ActiveTurnTerminalClass::Active,
    );
    let mut denied = base.clone();
    denied.authority = ActiveTurnContinuationAuthority::Denied;
    let mut cancelled = base.clone();
    cancelled.cancellation = ActiveTurnCancellationState::Cancelled;
    let mut repair_blocked = base;
    repair_blocked.repair_state = ActiveTurnRepairState::Blocked;

    for candidate in [denied, cancelled, repair_blocked] {
        let plan = plan_active_turn_recovery(&policy, &candidate).expect("blocked plan");
        assert_eq!(plan.action(), ActiveTurnRecoveryAction::Blocked);
        assert!(!plan.provider_continuation_allowed());
    }
}

#[test]
fn terminal_commit_precedes_cleanup() {
    let (policy, mut snapshot) = snapshot(
        ActiveTurnRepairState::Clean,
        ActiveTurnTerminalClass::Completed,
    );
    snapshot.entries.push(terminal_entry());
    snapshot.checkpoint_hash = active_turn_checkpoint_hash(
        &policy,
        snapshot.turn_hash.as_str(),
        snapshot.generation,
        snapshot.entries.as_slice(),
    )
    .expect("terminal checkpoint");
    let retained = plan_active_turn_recovery(&policy, &snapshot).expect("retained terminal");
    assert_eq!(retained.action(), ActiveTurnRecoveryAction::CleanupRequired);

    snapshot.cleanup = ActiveTurnCleanupState::Complete;
    let cleaned = plan_active_turn_recovery(&policy, &snapshot).expect("clean terminal");
    assert_eq!(cleaned.action(), ActiveTurnRecoveryAction::ObserveTerminal);
}

#[test]
fn corrupt_sequences_duplicates_and_mixed_results_fail_closed() {
    let policy = policy();
    let acceptance = plan_active_turn_acceptance(&policy, &acceptance_draft()).expect("acceptance");
    let cases = [
        vec![ActiveTurnPreparedEntry {
            sequence: TOOL_CALL_SEQUENCE,
            ..accepted_entry()
        }],
        vec![accepted_entry(), accepted_entry()],
        vec![
            accepted_entry(),
            ActiveTurnPreparedEntry {
                sequence: TOOL_CALL_SEQUENCE,
                kind: ActiveTurnJournalEntryKind::ToolResult,
                entry_hash: hash("orphan-result").to_string(),
                prepared_bytes: PREPARED_ENTRY_BYTES,
                tool_call_id: Some(CALL_ID.to_string()),
            },
        ],
    ];
    for entries in cases {
        let draft = ActiveTurnReplacementDraft {
            turn_hash: acceptance.turn_hash().to_string(),
            expected_generation: NEXT_GENERATION,
            entries,
        };
        assert_eq!(
            prepare_active_turn_checkpoint_replacement(&policy, &draft)
                .expect_err("corrupt journal")
                .issue(),
            ActiveTurnRecoveryIssue::CorruptJournal
        );
    }
}

#[test]
fn stale_checkpoint_identity_blocks_recovery() {
    let (policy, mut snapshot) = snapshot(
        ActiveTurnRepairState::Clean,
        ActiveTurnTerminalClass::Active,
    );
    snapshot.checkpoint_hash = hash("stale").to_string();
    assert_eq!(
        plan_active_turn_recovery(&policy, &snapshot)
            .expect_err("stale checkpoint")
            .issue(),
        ActiveTurnRecoveryIssue::StaleGeneration
    );
}

#[test]
fn invalid_policy_posture_fails_closed() {
    let mut input = policy_input();
    input.corruption_posture = "skip_rows".to_string();
    assert_eq!(
        validate_active_turn_recovery_policy(&input)
            .expect_err("unsafe corruption posture")
            .issue(),
        ActiveTurnRecoveryIssue::InvalidPolicy
    );

    let mut missing_non_claim = policy_input();
    missing_non_claim.non_claims.pop();
    assert_eq!(
        validate_active_turn_recovery_policy(&missing_non_claim)
            .expect_err("missing non-claim")
            .issue(),
        ActiveTurnRecoveryIssue::InvalidPolicy
    );
}

#[test]
fn receipts_are_stable_metadata_only() {
    // r[verify agent_harness_runtime.active_turn_recovery.evidence]
    let (policy, snapshot) = snapshot(
        ActiveTurnRepairState::Clean,
        ActiveTurnTerminalClass::Active,
    );
    let plan = plan_active_turn_recovery(&policy, &snapshot).expect("plan");
    let first = project_active_turn_recovery_receipt(
        &policy,
        &plan,
        snapshot.entries.len(),
        ActiveTurnRecoverySensitiveDraft::default(),
    )
    .expect("receipt");
    let second = project_active_turn_recovery_receipt(
        &policy,
        &plan,
        snapshot.entries.len(),
        ActiveTurnRecoverySensitiveDraft::default(),
    )
    .expect("stable receipt");

    assert_eq!(first, second);
    assert_eq!(first.schema(), ACTIVE_TURN_RECOVERY_SCHEMA);
    assert_eq!(first.action(), ActiveTurnRecoveryAction::ContinueProvider);
    assert!(first.non_claim().contains("does not prove"));
    assert_eq!(first.receipt_hash().len(), super::data::BLAKE3_HEX_CHARS);
}

#[test]
fn receipt_rejects_every_private_and_overclaim_class() {
    let (policy, snapshot) = snapshot(
        ActiveTurnRepairState::Clean,
        ActiveTurnTerminalClass::Active,
    );
    let plan = plan_active_turn_recovery(&policy, &snapshot).expect("plan");
    let leaks = [
        ActiveTurnRecoverySensitiveDraft {
            message: Some(PRIVATE_VALUE),
            ..ActiveTurnRecoverySensitiveDraft::default()
        },
        ActiveTurnRecoverySensitiveDraft {
            reasoning: Some(PRIVATE_VALUE),
            ..ActiveTurnRecoverySensitiveDraft::default()
        },
        ActiveTurnRecoverySensitiveDraft {
            tool_body: Some(PRIVATE_VALUE),
            ..ActiveTurnRecoverySensitiveDraft::default()
        },
        ActiveTurnRecoverySensitiveDraft {
            provider_payload: Some(PRIVATE_VALUE),
            ..ActiveTurnRecoverySensitiveDraft::default()
        },
        ActiveTurnRecoverySensitiveDraft {
            credential: Some(PRIVATE_VALUE),
            ..ActiveTurnRecoverySensitiveDraft::default()
        },
        ActiveTurnRecoverySensitiveDraft {
            environment_value: Some(PRIVATE_VALUE),
            ..ActiveTurnRecoverySensitiveDraft::default()
        },
        ActiveTurnRecoverySensitiveDraft {
            physical_path: Some(PRIVATE_VALUE),
            ..ActiveTurnRecoverySensitiveDraft::default()
        },
        ActiveTurnRecoverySensitiveDraft {
            store_error: Some(PRIVATE_VALUE),
            ..ActiveTurnRecoverySensitiveDraft::default()
        },
        ActiveTurnRecoverySensitiveDraft {
            unsupported_claim: Some(PRIVATE_VALUE),
            ..ActiveTurnRecoverySensitiveDraft::default()
        },
    ];
    for leak in leaks {
        let failure = project_active_turn_recovery_receipt(&policy, &plan, ONE_ENTRY, leak)
            .expect_err("receipt leak");
        assert_eq!(failure.issue(), ActiveTurnRecoveryIssue::ReceiptLeak);
        assert!(!format!("{failure:?}").contains(PRIVATE_VALUE));
    }
}

fn append_control_entry(
    policy: &ActiveTurnRecoveryPolicy,
    snapshot: &mut ActiveTurnRecoverySnapshot,
    control: &crate::AgentControlState,
) -> ActiveTurnJournalEntryKind {
    let sequence = u32::try_from(snapshot.entries.len()).expect("bounded entry count");
    let entry = prepare_active_turn_control_entry(
        policy,
        ActiveTurnControlEntryDraft {
            turn_hash: snapshot.turn_hash.as_str(),
            sequence,
            prepared_bytes: PREPARED_ENTRY_BYTES,
            control,
        },
    )
    .expect("control entry");
    let kind = entry.kind;
    snapshot.entries.push(entry);
    snapshot.checkpoint_hash = active_turn_checkpoint_hash(
        policy,
        snapshot.turn_hash.as_str(),
        snapshot.generation,
        snapshot.entries.as_slice(),
    )
    .expect("control checkpoint hash");
    kind
}

fn policy() -> ActiveTurnRecoveryPolicy {
    validate_active_turn_recovery_policy(&policy_input()).expect("valid policy")
}

fn policy_input() -> ActiveTurnRecoveryPolicyInput {
    ActiveTurnRecoveryPolicyInput {
        schema: ACTIVE_TURN_RECOVERY_SCHEMA.to_string(),
        policy_id: POLICY_ID.to_string(),
        max_entries: MAX_ENTRIES,
        max_entry_bytes: MAX_ENTRY_BYTES,
        max_aggregate_bytes: MAX_AGGREGATE_BYTES,
        max_steering_entries: MAX_STEERING_ENTRIES,
        corruption_posture: "fail_closed".to_string(),
        replacement_posture: "compare_and_swap_atomic".to_string(),
        transaction_evidence_required: true,
        retention_class: RETENTION_CLASS.to_string(),
        receipt_fields: [
            "policy_hash",
            "turn_hash",
            "checkpoint_hash",
            "plan_hash",
            "generation",
            "action",
            "terminal",
        ]
        .into_iter()
        .map(str::to_string)
        .collect(),
        non_claims: [
            "store_correctness",
            "power_loss_durability",
            "provider_acceptance",
            "exactly_once_effects",
            "task_completion",
            "production_readiness",
        ]
        .into_iter()
        .map(str::to_string)
        .collect(),
        policy_hash: hash("policy-source").to_string(),
    }
}

fn acceptance_draft() -> ActiveTurnAcceptanceDraft {
    ActiveTurnAcceptanceDraft {
        session_id: SESSION_ID.to_string(),
        branch_id: BRANCH_ID.to_string(),
        turn_sequence: TURN_SEQUENCE,
        accepted_message_hash: hash("accepted-message").to_string(),
        expected_generation: PRIOR_GENERATION,
    }
}

fn snapshot(
    repair_state: ActiveTurnRepairState,
    terminal: ActiveTurnTerminalClass,
) -> (ActiveTurnRecoveryPolicy, ActiveTurnRecoverySnapshot) {
    let policy = policy();
    let acceptance = plan_active_turn_acceptance(&policy, &acceptance_draft()).expect("acceptance");
    let entries = completed_tool_entries();
    let checkpoint_hash = active_turn_checkpoint_hash(
        &policy,
        acceptance.turn_hash(),
        NEXT_GENERATION,
        entries.as_slice(),
    )
    .expect("checkpoint");
    let snapshot = ActiveTurnRecoverySnapshot {
        turn_hash: acceptance.turn_hash().to_string(),
        checkpoint_hash,
        generation: NEXT_GENERATION,
        entries,
        acceptance: ActiveTurnAcceptanceState::Durable,
        authority: ActiveTurnContinuationAuthority::Allowed,
        cancellation: ActiveTurnCancellationState::Active,
        repair_state,
        terminal,
        cleanup: ActiveTurnCleanupState::Retained,
    };
    (policy, snapshot)
}

fn completed_tool_entries() -> Vec<ActiveTurnPreparedEntry> {
    vec![accepted_entry(), tool_call_entry(), tool_result_entry()]
}

fn accepted_entry() -> ActiveTurnPreparedEntry {
    ActiveTurnPreparedEntry {
        sequence: ACCEPTED_SEQUENCE,
        kind: ActiveTurnJournalEntryKind::AcceptedMessage,
        entry_hash: hash("accepted-entry").to_string(),
        prepared_bytes: PREPARED_ENTRY_BYTES,
        tool_call_id: None,
    }
}

fn tool_call_entry() -> ActiveTurnPreparedEntry {
    ActiveTurnPreparedEntry {
        sequence: TOOL_CALL_SEQUENCE,
        kind: ActiveTurnJournalEntryKind::ToolCall,
        entry_hash: hash("tool-call-entry").to_string(),
        prepared_bytes: PREPARED_ENTRY_BYTES,
        tool_call_id: Some(CALL_ID.to_string()),
    }
}

fn tool_result_entry() -> ActiveTurnPreparedEntry {
    ActiveTurnPreparedEntry {
        sequence: TOOL_RESULT_SEQUENCE,
        kind: ActiveTurnJournalEntryKind::ToolResult,
        entry_hash: hash("tool-result-entry").to_string(),
        prepared_bytes: PREPARED_ENTRY_BYTES,
        tool_call_id: Some(CALL_ID.to_string()),
    }
}

fn terminal_entry() -> ActiveTurnPreparedEntry {
    ActiveTurnPreparedEntry {
        sequence: TERMINAL_SEQUENCE,
        kind: ActiveTurnJournalEntryKind::Terminal,
        entry_hash: hash("terminal-entry").to_string(),
        prepared_bytes: PREPARED_ENTRY_BYTES,
        tool_call_id: None,
    }
}

fn committed_observation(
    transaction_hash: &str,
    prior_generation: u64,
    generation: u64,
) -> ActiveTurnTransactionObservation {
    ActiveTurnTransactionObservation {
        transaction_hash: transaction_hash.to_string(),
        observed_prior_generation: prior_generation,
        observed_generation: generation,
        status: ActiveTurnTransactionStatus::Committed,
        complete_manifest: true,
    }
}

fn hash(value: &str) -> &'static str {
    Box::leak(
        blake3::hash(value.as_bytes())
            .to_hex()
            .to_string()
            .into_boxed_str(),
    )
}

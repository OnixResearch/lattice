use alloc::format;
use alloc::string::{String, ToString};

use super::*;

const HASH_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const HASH_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const HASH_C: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
const HASH_D: &str = "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";
const HASH_E: &str = "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";
const SESSION_ID: &str = "session.snapshot";
const BRANCH_ID: &str = "branch.main";
const ACTIVE_LEAF_ID: &str = "entry.user";
const PROVIDER_ID: &str = "openai";
const PROVIDER_TRANSPORT_ID: &str = "openai-responses-v1";
const MODEL_ID: &str = "gpt-snapshot";
const MODEL_DIALECT_ID: &str = "codex-responses-v1";
const SYSTEM_TEXT: &str = "Follow the admitted system contract.";
const USER_TEXT: &str = "Inspect the durable request.";
const TOOL_SCHEMA: &str = "schema.steel.v1";
const TURN_GENERATION: u64 = 7;
const REQUEST_SEQUENCE: u64 = 3;
const DURABLE_CURSOR: u64 = 11;
const RETRY_ATTEMPT: u32 = 2;

struct DraftIdentities {
    route: String,
    tools: String,
    output: String,
    budget: String,
}

#[test]
fn exact_snapshot_reconstructs_ordered_model_visible_request() {
    // r[verify agent_harness_runtime.model_request_snapshot.projection]
    // r[verify animus.dialect.binding]
    // r[verify agent_harness_runtime.model_request_snapshot.verification]
    let request = canonical_request(false);
    let first_identities = identities(&request);
    let first_snapshot = snapshot(&request, &first_identities);
    assert_eq!(first_snapshot.canonical_request(), &request);
    assert_eq!(
        first_snapshot.provider_transport_id(),
        PROVIDER_TRANSPORT_ID
    );
    assert_eq!(first_snapshot.model_dialect_id(), MODEL_DIALECT_ID);
    assert_eq!(first_snapshot.message_count(), 2);
    assert_eq!(first_snapshot.system_section_count(), 1);
    assert_eq!(first_snapshot.tool_count(), 1);
    assert!(!first_snapshot.private_payload().is_empty());
    let decoded = decode_model_request_snapshot_private_payload(first_snapshot.private_payload())
        .expect("private snapshot round trip");
    assert_eq!(decoded, first_snapshot);

    let reordered = canonical_request(true);
    let reordered_identities = identities(&reordered);
    let reordered_snapshot = snapshot(&reordered, &reordered_identities);
    assert_ne!(
        first_snapshot.snapshot_identity(),
        reordered_snapshot.snapshot_identity()
    );
    assert_ne!(
        first_snapshot.canonical_request_identity(),
        reordered_snapshot.canonical_request_identity()
    );
}

#[test]
fn missing_sections_tool_drift_duplicates_and_unsupported_fields_fail_closed() {
    // r[verify agent_harness_runtime.model_request_snapshot.projection]
    // r[verify agent_harness_runtime.model_request_snapshot.verification]
    let request = canonical_request(false);
    let identities = identities(&request);

    let mut missing = draft(&request, &identities);
    missing.schema = "wrong.schema";
    assert_eq!(
        validate_model_request_snapshot(&missing)
            .expect_err("schema")
            .issue(),
        ModelRequestSnapshotIssue::InvalidSchema
    );

    let mut truncated = snapshot(&request, &identities).private_payload().to_vec();
    let _removed = truncated.pop().expect("non-empty private payload");
    assert_eq!(
        decode_model_request_snapshot_private_payload(truncated.as_slice())
            .expect_err("truncated private payload")
            .issue(),
        ModelRequestSnapshotIssue::InvalidSchema
    );

    let mut tool_drift = draft(&request, &identities);
    tool_drift.tool_schema_set_identity = HASH_E;
    assert_eq!(
        validate_model_request_snapshot(&tool_drift)
            .expect_err("tool drift")
            .issue(),
        ModelRequestSnapshotIssue::IdentityMismatch
    );

    // r[verify animus.dialect.binding.leak]
    let mut dialect_drift = draft(&request, &identities);
    dialect_drift.route.model_dialect_id = "generic-responses-v1";
    assert_eq!(
        validate_model_request_snapshot(&dialect_drift)
            .expect_err("dialect drift")
            .issue(),
        ModelRequestSnapshotIssue::IdentityMismatch
    );

    let duplicate_context = [HASH_C, HASH_C];
    let mut duplicate = draft(&request, &identities);
    duplicate.selected_context_contribution_identities = &duplicate_context;
    assert_eq!(
        validate_model_request_snapshot(&duplicate)
            .expect_err("duplicate context")
            .issue(),
        ModelRequestSnapshotIssue::DuplicateIdentity
    );

    let mut unsupported = draft(&request, &identities);
    unsupported.unsupported.credential = Some("private credential");
    assert_eq!(
        validate_model_request_snapshot(&unsupported)
            .expect_err("credential")
            .issue(),
        ModelRequestSnapshotIssue::UnsupportedField
    );
}

#[test]
fn exact_private_commit_is_required_before_initial_dispatch() {
    // r[verify agent_harness_runtime.model_request_snapshot.durability]
    // r[verify agent_harness_runtime.model_request_snapshot.dispatch]
    let request = canonical_request(false);
    let identities = identities(&request);
    let snapshot = snapshot(&request, &identities);
    let plan = plan_model_request_snapshot_commit(snapshot, HASH_A).expect("commit plan");
    let mut observation = commit_observation(&plan);
    observation.complete = false;
    assert_eq!(
        observe_model_request_snapshot_commit(plan.clone(), &observation)
            .expect_err("partial commit")
            .issue(),
        ModelRequestSnapshotIssue::CommitMismatch
    );

    let mut observation = commit_observation(&plan);
    observation.active_leaf_id = String::from("entry.substituted");
    assert_eq!(
        observe_model_request_snapshot_commit(plan.clone(), &observation)
            .expect_err("substitution")
            .issue(),
        ModelRequestSnapshotIssue::CommitMismatch
    );

    let committed = observe_model_request_snapshot_commit(plan.clone(), &commit_observation(&plan))
        .expect("exact commit");
    let dispatch = plan_initial_model_request_dispatch(&committed, current_facts(&committed))
        .expect("initial dispatch");
    assert_eq!(
        dispatch.snapshot_identity(),
        committed.snapshot().snapshot_identity()
    );
    assert_eq!(dispatch.attempt_kind(), ModelRequestAttemptKind::Initial);
}

#[test]
fn dispatch_and_same_request_retry_reject_every_composition_drift() {
    // r[verify agent_harness_runtime.model_request_snapshot.dispatch]
    // r[verify agent_harness_runtime.model_request_snapshot.verification]
    let committed = committed_snapshot();
    let exact = current_facts(&committed);
    let retry = plan_same_model_request_retry(
        &committed,
        exact,
        ModelRequestRetryOwnerDecision::Admitted,
        RETRY_ATTEMPT,
    )
    .expect("same request retry");
    assert_eq!(
        retry.attempt_kind(),
        ModelRequestAttemptKind::SameRequestRetry
    );
    assert_eq!(
        retry.snapshot_identity(),
        committed.snapshot().snapshot_identity()
    );

    assert_eq!(
        plan_same_model_request_retry(
            &committed,
            exact,
            ModelRequestRetryOwnerDecision::Denied,
            RETRY_ATTEMPT,
        )
        .expect_err("owner denied")
        .issue(),
        ModelRequestSnapshotIssue::RetryDenied
    );

    let drifts = [
        ModelRequestDispatchFacts {
            model_invocation_identity: HASH_E,
            ..exact
        },
        ModelRequestDispatchFacts {
            request_context_plan_identity: HASH_E,
            ..exact
        },
        ModelRequestDispatchFacts {
            tool_schema_set_identity: HASH_E,
            ..exact
        },
        ModelRequestDispatchFacts {
            output_control_identity: HASH_E,
            ..exact
        },
        ModelRequestDispatchFacts {
            authority_seal_identity: HASH_E,
            ..exact
        },
        ModelRequestDispatchFacts {
            adapter_cohort_identity: HASH_E,
            ..exact
        },
    ];
    for drift in drifts {
        assert_eq!(
            plan_initial_model_request_dispatch(&committed, drift)
                .expect_err("drift")
                .issue(),
            ModelRequestSnapshotIssue::DispatchDrift
        );
    }
}

#[test]
fn restart_keeps_committed_unobserved_attempt_uncertain_without_replay() {
    // r[verify agent_harness_runtime.model_request_snapshot.recovery]
    let committed = committed_snapshot();
    let uncertain = classify_model_request_snapshot_recovery(&committed, None)
        .expect("uncertain classification");
    assert_eq!(
        uncertain.status,
        ModelRequestRecoveryStatus::CommittedUnobserved
    );
    assert!(uncertain.retry_owner_decision_required);
    assert!(!uncertain.automatic_replay);

    let dispatch = plan_initial_model_request_dispatch(&committed, current_facts(&committed))
        .expect("dispatch");
    let terminal = observe_model_request_provider_terminal(
        &dispatch,
        ModelRequestProviderTerminalDraft {
            snapshot_identity: dispatch.snapshot_identity(),
            adapter_cohort_identity: dispatch.adapter_cohort_identity(),
            attempt_number: dispatch.attempt_number(),
            status: ModelRequestProviderTerminalStatus::Succeeded,
        },
    )
    .expect("terminal");
    let terminal_payload = model_request_provider_terminal_private_payload(&terminal)
        .expect("terminal private payload");
    let decoded_terminal =
        decode_model_request_provider_terminal_private_payload(terminal_payload.as_slice())
            .expect("terminal payload round trip");
    assert_eq!(decoded_terminal, terminal);
    let mut stale_terminal = terminal.clone();
    stale_terminal.observation_identity = String::from(HASH_E);
    assert_eq!(
        classify_model_request_snapshot_recovery(&committed, Some(&stale_terminal))
            .expect_err("stale terminal identity")
            .issue(),
        ModelRequestSnapshotIssue::TerminalMismatch
    );
    let observed = classify_model_request_snapshot_recovery(&committed, Some(&terminal))
        .expect("terminal recovery");
    assert_eq!(
        observed.status,
        ModelRequestRecoveryStatus::TerminalObserved
    );
    assert!(!observed.retry_owner_decision_required);
    assert!(!observed.automatic_replay);
}

#[test]
fn public_receipts_are_metadata_only_and_reject_private_bodies() {
    // r[verify agent_harness_runtime.model_request_snapshot.evidence]
    let committed = committed_snapshot();
    let receipt = project_model_request_snapshot_receipt(
        committed.snapshot(),
        ModelRequestSnapshotReceiptStatus::Committed,
        ModelRequestSnapshotReceiptSensitiveDraft::default(),
    )
    .expect("safe receipt");
    assert_eq!(receipt.message_count, 2);
    assert_eq!(receipt.tool_count, 1);
    assert_eq!(receipt.redaction_marker, MODEL_REQUEST_SNAPSHOT_REDACTION);
    assert!(!format!("{receipt:?}").contains(USER_TEXT));
    validate_model_request_snapshot_receipt(committed.snapshot(), &receipt)
        .expect("receipt validation");

    let leak = ModelRequestSnapshotReceiptSensitiveDraft {
        raw_prompt: Some(USER_TEXT),
        ..ModelRequestSnapshotReceiptSensitiveDraft::default()
    };
    assert_eq!(
        project_model_request_snapshot_receipt(
            committed.snapshot(),
            ModelRequestSnapshotReceiptStatus::Committed,
            leak,
        )
        .expect_err("body leak")
        .issue(),
        ModelRequestSnapshotIssue::UnsafeEvidence
    );

    let mut tampered = receipt;
    tampered.adapter_cohort_identity = HASH_E.to_string();
    assert_eq!(
        validate_model_request_snapshot_receipt(committed.snapshot(), &tampered)
            .expect_err("tampered")
            .issue(),
        ModelRequestSnapshotIssue::UnsafeEvidence
    );
}

fn canonical_request(reverse_messages: bool) -> animus_adapters::CanonicalModelRequest {
    let system_parts =
        [animus_adapters::CanonicalMessageContentPartDraft::Text { text: SYSTEM_TEXT }];
    let user_parts = [animus_adapters::CanonicalMessageContentPartDraft::Text { text: USER_TEXT }];
    let system = animus_adapters::CanonicalModelMessageDraft {
        role: "system",
        content: &system_parts,
        tool_call_id: None,
    };
    let user = animus_adapters::CanonicalModelMessageDraft {
        role: "user",
        content: &user_parts,
        tool_call_id: None,
    };
    let messages = if reverse_messages {
        [user, system]
    } else {
        [system, user]
    };
    let tools = [animus_adapters::CanonicalModelToolDeclarationDraft {
        name: "steel_repl",
        description: "Evaluate one admitted Steel cell.",
        input_schema_label: TOOL_SCHEMA,
    }];
    animus_adapters::CanonicalModelRequest::from_draft(
        animus_adapters::CanonicalModelRequestDraft {
            contract_identity: animus_adapters::CANONICAL_MODEL_REQUEST_CONTRACT_IDENTITY,
            messages: &messages,
            tools: &tools,
            output: animus_adapters::CanonicalModelOutputControlsDraft {
                max_output_tokens: Some(256),
                temperature_millis: None,
                stop_sequences: &[],
                stream: true,
            },
            response_format: animus_adapters::CanonicalModelResponseFormatIntentDraft::Text,
            reasoning: animus_adapters::CanonicalModelReasoningRequestDraft {
                enabled: true,
                effort: animus_adapters::CanonicalModelReasoningEffort::Low,
                summary_label: None,
            },
            provenance: &[],
        },
    )
    .expect("canonical request")
}

fn identities(request: &animus_adapters::CanonicalModelRequest) -> DraftIdentities {
    DraftIdentities {
        route: model_request_route_identity(
            PROVIDER_ID,
            PROVIDER_TRANSPORT_ID,
            MODEL_ID,
            MODEL_DIALECT_ID,
        ),
        tools: model_request_tool_schema_set_identity(request),
        output: model_request_output_control_identity(request),
        budget: model_request_budget_identity(request),
    }
}

fn draft<'input>(
    request: &'input animus_adapters::CanonicalModelRequest,
    identities: &'input DraftIdentities,
) -> ModelRequestSnapshotDraft<'input> {
    ModelRequestSnapshotDraft {
        schema: MODEL_REQUEST_SNAPSHOT_SCHEMA,
        session_id: SESSION_ID,
        branch_id: BRANCH_ID,
        active_leaf_id: ACTIVE_LEAF_ID,
        turn_generation: TURN_GENERATION,
        request_sequence: REQUEST_SEQUENCE,
        canonical_request: request,
        route: ModelRequestRouteDraft {
            provider_id: PROVIDER_ID,
            provider_transport_id: PROVIDER_TRANSPORT_ID,
            model_id: MODEL_ID,
            model_dialect_id: MODEL_DIALECT_ID,
            route_identity: identities.route.as_str(),
        },
        model_invocation_identity: HASH_A,
        request_context_plan_identity: HASH_B,
        selected_context_contribution_identities: &[HASH_C, HASH_D],
        tool_schema_set_identity: identities.tools.as_str(),
        output_control_identity: identities.output.as_str(),
        budget_identity: identities.budget.as_str(),
        authority_seal_identity: HASH_C,
        adapter_cohort_identity: HASH_D,
        unsupported: ModelRequestSnapshotUnsupportedDraft::default(),
    }
}

fn snapshot(
    request: &animus_adapters::CanonicalModelRequest,
    identities: &DraftIdentities,
) -> ModelRequestSnapshot {
    validate_model_request_snapshot(&draft(request, identities)).expect("snapshot")
}

fn commit_observation(
    plan: &ModelRequestSnapshotCommitPlan,
) -> ModelRequestSnapshotCommitObservation {
    ModelRequestSnapshotCommitObservation {
        plan_identity: String::from(plan.plan_identity()),
        snapshot_identity: String::from(plan.snapshot().snapshot_identity()),
        session_id: String::from(plan.snapshot().session_id()),
        branch_id: String::from(plan.snapshot().branch_id()),
        active_leaf_id: String::from(plan.snapshot().active_leaf_id()),
        turn_generation: plan.snapshot().turn_generation(),
        request_sequence: plan.snapshot().request_sequence(),
        new_head_hash: String::from(HASH_B),
        durable_cursor: DURABLE_CURSOR,
        complete: true,
        synchronized: true,
    }
}

fn committed_snapshot() -> CommittedModelRequestSnapshot {
    let request = canonical_request(false);
    let identities = identities(&request);
    let snapshot = snapshot(&request, &identities);
    let plan = plan_model_request_snapshot_commit(snapshot, HASH_A).expect("plan");
    let observation = commit_observation(&plan);
    observe_model_request_snapshot_commit(plan, &observation).expect("commit")
}

fn current_facts(committed: &CommittedModelRequestSnapshot) -> ModelRequestDispatchFacts<'_> {
    let snapshot = committed.snapshot();
    ModelRequestDispatchFacts {
        active_leaf_id: snapshot.active_leaf_id(),
        turn_generation: snapshot.turn_generation(),
        request_sequence: snapshot.request_sequence(),
        model_invocation_identity: snapshot.model_invocation_identity(),
        request_context_plan_identity: snapshot.request_context_plan_identity(),
        tool_schema_set_identity: snapshot.tool_schema_set_identity(),
        output_control_identity: snapshot.output_control_identity(),
        budget_identity: snapshot.budget_identity(),
        authority_seal_identity: snapshot.authority_seal_identity(),
        adapter_cohort_identity: snapshot.adapter_cohort_identity(),
    }
}

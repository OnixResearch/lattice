use std::collections::BTreeMap;
use std::collections::BTreeSet;

use lattice::engine::RunId;
use lattice::engine::Workflow;
use lattice::orchestration::ABANDONED_ACKNOWLEDGED_SUMMARY;
use lattice::orchestration::ABANDONED_PENDING_SUMMARY;
use lattice::orchestration::ApprovalDecision;
use lattice::orchestration::AuthorityPolicy;
use lattice::orchestration::BudgetCharge;
use lattice::orchestration::BudgetChargePlan;
use lattice::orchestration::BudgetDimensions;
use lattice::orchestration::BudgetLedger;
use lattice::orchestration::BudgetLimits;
use lattice::orchestration::BudgetPolicy;
use lattice::orchestration::CompletionPolicy;
use lattice::orchestration::DispatchIntent;
use lattice::orchestration::DispatchStatus;
use lattice::orchestration::OrchestrationAction;
use lattice::orchestration::OrchestrationDefinition;
use lattice::orchestration::OrchestrationDiagnosticCode;
use lattice::orchestration::OrchestrationEventKind;
use lattice::orchestration::OrchestrationFact;
use lattice::orchestration::OrchestrationStatus;
use lattice::orchestration::OrchestrationTriggerAdmission;
use lattice::orchestration::OrchestrationTriggerBinding;
use lattice::orchestration::OrchestrationTriggerContext;
use lattice::orchestration::OrchestrationTriggerKind;
use lattice::orchestration::RedactionPolicy;
use lattice::orchestration::RevisionProposal;
use lattice::orchestration::TriggerStartRequest;
use lattice::orchestration::admit_trigger_start;
use lattice::orchestration::authority_policy_hash;
use lattice::orchestration::current_schema_version;
use lattice::orchestration::definition_revision;
use lattice::orchestration::dispatch_identity;
use lattice::orchestration::fact_identity;
use lattice::orchestration::make_approval;
use lattice::orchestration::make_proposal;
use lattice::orchestration::new_instance;
use lattice::orchestration::replay_events;
use lattice::orchestration::reserve_budget;
use lattice::orchestration::transition;
use lattice::orchestration::validate_definition;
use lattice::orchestration::workflow_revision;

const DEFINITION_ID: &str = "review-loop";
const OBJECTIVE: &str = "Produce an approved report";
const WORKFLOW_NAME: &str = "review-round";
const SUCCESSOR_WORKFLOW_NAME: &str = "review-round-v2";
const ACTOR: &str = "operator:alice";
const OTHER_ACTOR: &str = "agent:reviewer";
const TRIGGER_ID: &str = "review-webhook";
const IDEMPOTENCY_KEY: &str = "event-key";
const DISPATCH_ID: &str = "dispatch-one";
const FACT_ID_START: &str = "fact-start";
const FACT_ID_FINISH: &str = "fact-finish";
const FIRST_INSTANCE_SEQUENCE: u64 = 1;
const FIRST_RUN_ID: u64 = 7;
const TOKEN_LIMIT: u64 = 1_000;
const TOKEN_CHARGE: u64 = 100;
const MONEY_LIMIT: u64 = 5_000;
const MONEY_CHARGE: u64 = 500;
const DURATION_LIMIT: u64 = 60_000;
const DURATION_CHARGE: u64 = 10_000;
const HEX_CHARS_PER_BYTE: usize = 2;

fn workflow(name: &str) -> Workflow {
    Workflow::new(name)
}

fn definition(completion: CompletionPolicy) -> OrchestrationDefinition {
    let workflow = workflow(WORKFLOW_NAME);
    let revision = workflow_revision(&workflow).expect("fixture workflow must serialize");
    OrchestrationDefinition {
        schema_version: current_schema_version(),
        id: DEFINITION_ID.to_string(),
        objective: OBJECTIVE.to_string(),
        workflow: revision,
        completion,
        budgets: BudgetPolicy {
            enforced: BudgetDimensions {
                rounds: true,
                agent_invocations: true,
                tokens: true,
                monetary_microunits: true,
                duration_millis: true,
            },
            limits: BudgetLimits {
                rounds: FIRST_INSTANCE_SEQUENCE,
                agent_invocations: FIRST_INSTANCE_SEQUENCE,
                tokens: TOKEN_LIMIT,
                monetary_microunits: MONEY_LIMIT,
                duration_millis: DURATION_LIMIT,
            },
            per_round_upper_bound: charge(),
        },
        authority: AuthorityPolicy {
            approvers: BTreeSet::from([ACTOR.to_string()]),
        },
        triggers: vec![OrchestrationTriggerBinding {
            trigger_id: TRIGGER_ID.to_string(),
            kind: OrchestrationTriggerKind::Webhook,
        }],
        redaction: RedactionPolicy {
            retain_safe_summaries: true,
        },
    }
}

fn charge() -> BudgetCharge {
    BudgetCharge {
        rounds: FIRST_INSTANCE_SEQUENCE,
        agent_invocations: FIRST_INSTANCE_SEQUENCE,
        tokens: TOKEN_CHARGE,
        monetary_microunits: MONEY_CHARGE,
        duration_millis: DURATION_CHARGE,
    }
}

fn charge_plan() -> BudgetChargePlan {
    BudgetChargePlan {
        rounds: Some(FIRST_INSTANCE_SEQUENCE),
        agent_invocations: Some(FIRST_INSTANCE_SEQUENCE),
        tokens: Some(TOKEN_CHARGE),
        monetary_microunits: Some(MONEY_CHARGE),
        duration_millis: Some(DURATION_CHARGE),
    }
}

fn base_instance(completion: CompletionPolicy) -> lattice::orchestration::OrchestrationInstance {
    new_instance(definition(completion), FIRST_INSTANCE_SEQUENCE, None).expect("fixture instance must hash")
}

fn fact(instance: &lattice::orchestration::OrchestrationInstance, action: OrchestrationAction) -> OrchestrationFact {
    let id = fact_identity(&instance.id, instance.version, &action).expect("fixture fact must hash");
    OrchestrationFact { id, action }
}

fn start_transition(
    instance: &lattice::orchestration::OrchestrationInstance,
) -> lattice::orchestration::OrchestrationTransition {
    let run_id = RunId(FIRST_RUN_ID);
    let intent = DispatchIntent {
        id: dispatch_identity(instance, run_id).expect("dispatch must hash"),
        run_id,
        workflow_revision: instance.workflow_revision.clone(),
        status: DispatchStatus::Pending,
    };
    let action = OrchestrationAction::StartRound {
        intent,
        planned_charge: charge_plan(),
    };
    transition(instance, fact(instance, action)).expect("round must start")
}

fn finish_transition(
    instance: &lattice::orchestration::OrchestrationInstance,
    succeeded: bool,
) -> lattice::orchestration::OrchestrationTransition {
    let dispatch = instance.pending_dispatch.as_ref().expect("started instance has dispatch");
    let action = OrchestrationAction::RunFinished {
        dispatch_id: dispatch.id.clone(),
        run_id: dispatch.run_id,
        succeeded,
        observed_usage: charge(),
        safe_summary: "round finished with bounded output".to_string(),
    };
    transition(instance, fact(instance, action)).expect("round must finish")
}

// r[verify lattice.agent_orchestration.model]
#[test]
fn valid_definition_binds_exact_workflow_revision() {
    let definition = definition(CompletionPolicy::RunSucceeded);
    let actual = definition.workflow.clone();

    assert!(validate_definition(&definition, Some(&actual)).is_empty());
    assert_eq!(
        definition_revision(&definition).expect("definition must hash").len(),
        blake3::OUT_LEN * HEX_CHARS_PER_BYTE
    );
}

// r[verify lattice.agent_orchestration.model]
#[test]
fn workflow_revision_drift_is_rejected() {
    let definition = definition(CompletionPolicy::RunSucceeded);
    let drifted = workflow_revision(&workflow(SUCCESSOR_WORKFLOW_NAME)).expect("workflow must hash");

    let diagnostics = validate_definition(&definition, Some(&drifted));

    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == OrchestrationDiagnosticCode::WorkflowRevisionMismatch)
    );
}

// r[verify lattice.agent_orchestration.core]
#[test]
fn successful_terminal_round_completes_and_cannot_reopen() {
    let base = base_instance(CompletionPolicy::RunSucceeded);
    let started = start_transition(&base).instance;
    let completed = finish_transition(&started, true).instance;

    assert_eq!(completed.status, OrchestrationStatus::Completed);
    assert_eq!(completed.round, FIRST_INSTANCE_SEQUENCE);
    assert_eq!(completed.budgets.consumed, charge());

    let pause = fact(&completed, OrchestrationAction::PauseRequested {
        actor: ACTOR.to_string(),
    });
    let error = transition(&completed, pause).expect_err("terminal state must reject mutation");
    assert_eq!(error.code, OrchestrationDiagnosticCode::TerminalStateMutation);
}

// r[verify lattice.agent_orchestration.authority]
#[test]
fn proposal_requires_current_authority_and_successor_validation() {
    let base = base_instance(CompletionPolicy::AwaitProposal);
    let started = start_transition(&base).instance;
    let waiting = finish_transition(&started, true).instance;
    assert_eq!(waiting.status, OrchestrationStatus::WaitingForApproval);

    let successor = workflow_revision(&workflow(SUCCESSOR_WORKFLOW_NAME)).expect("successor must hash");
    let proposal = make_proposal(&waiting, successor, "revise the report workflow").expect("proposal must hash");
    let submit = transition(
        &waiting,
        fact(&waiting, OrchestrationAction::SubmitProposal {
            proposal: proposal.clone(),
        }),
    )
    .expect("proposal must be accepted")
    .instance;

    let unauthorized = make_approval(&submit, &proposal, OTHER_ACTOR, true).expect("approval identity must hash");
    let unauthorized_error = transition(
        &submit,
        fact(&submit, OrchestrationAction::DecideProposal {
            approval: unauthorized,
            successor_validated: true,
        }),
    )
    .expect_err("unauthorized actor must fail");
    assert_eq!(unauthorized_error.code, OrchestrationDiagnosticCode::UnauthorizedActor);

    let approval = make_approval(&submit, &proposal, ACTOR, true).expect("approval must hash");
    let invalid_successor_error = transition(
        &submit,
        fact(&submit, OrchestrationAction::DecideProposal {
            approval: approval.clone(),
            successor_validated: false,
        }),
    )
    .expect_err("approval must not bypass validation");
    assert_eq!(invalid_successor_error.code, OrchestrationDiagnosticCode::SuccessorValidationFailed);

    let approved = transition(
        &submit,
        fact(&submit, OrchestrationAction::DecideProposal {
            approval,
            successor_validated: true,
        }),
    )
    .expect("validated approval must advance")
    .instance;
    assert_eq!(approved.status, OrchestrationStatus::Ready);
    assert_eq!(approved.workflow_revision, proposal.successor);
}

// r[verify lattice.agent_orchestration.authority]
#[test]
fn stale_approval_is_rejected() {
    let mut instance = base_instance(CompletionPolicy::AwaitProposal);
    instance.status = OrchestrationStatus::WaitingForApproval;
    let successor = workflow_revision(&workflow(SUCCESSOR_WORKFLOW_NAME)).expect("successor must hash");
    let proposal = make_proposal(&instance, successor, "safe proposal").expect("proposal must hash");
    instance.pending_proposal = Some(proposal.clone());
    let mut stale = make_approval(&instance, &proposal, ACTOR, true).expect("approval must hash");
    stale.epoch = stale.epoch.saturating_add(FIRST_INSTANCE_SEQUENCE);
    stale.id = lattice::orchestration::approval_identity(&stale).expect("stale approval must hash");

    let error = transition(
        &instance,
        fact(&instance, OrchestrationAction::DecideProposal {
            approval: stale,
            successor_validated: true,
        }),
    )
    .expect_err("stale approval must fail");

    assert_eq!(error.code, OrchestrationDiagnosticCode::StaleApproval);
}

// r[verify lattice.agent_orchestration.budgets]
#[test]
fn budget_reservation_rejects_missing_bounds_and_oversubscription() {
    let policy = definition(CompletionPolicy::RunSucceeded).budgets;
    let mut missing_bound_ledger = BudgetLedger {
        policy,
        ..BudgetLedger::default()
    };
    let mut missing = charge_plan();
    missing.tokens = None;
    let missing_error = reserve_budget(&mut missing_bound_ledger, "missing", missing).expect_err("bound is required");
    assert_eq!(missing_error.code, OrchestrationDiagnosticCode::MissingBudgetBound);

    let mut ledger = BudgetLedger {
        policy,
        ..BudgetLedger::default()
    };
    reserve_budget(&mut ledger, "first", charge_plan()).expect("first reservation fits");
    let oversubscribed =
        reserve_budget(&mut ledger, "second", charge_plan()).expect_err("second reservation exceeds limit");
    assert_eq!(oversubscribed.code, OrchestrationDiagnosticCode::BudgetExceeded);
    assert_eq!(ledger.reservations.len(), FIRST_INSTANCE_SEQUENCE as usize);
}

// r[verify lattice.agent_orchestration.durability]
#[test]
fn duplicate_fact_is_idempotent() {
    let base = base_instance(CompletionPolicy::RunSucceeded);
    let action = OrchestrationAction::StartRound {
        intent: DispatchIntent {
            id: DISPATCH_ID.to_string(),
            run_id: RunId(FIRST_RUN_ID),
            workflow_revision: base.workflow_revision.clone(),
            status: DispatchStatus::Pending,
        },
        planned_charge: charge_plan(),
    };
    let start = OrchestrationFact {
        id: FACT_ID_START.to_string(),
        action,
    };
    let applied = transition(&base, start.clone()).expect("first fact applies");
    let duplicate = transition(&applied.instance, start).expect("duplicate fact is admitted idempotently");

    assert!(duplicate.duplicate);
    assert!(duplicate.event.is_none());
    assert_eq!(duplicate.instance, applied.instance);
}

// r[verify lattice.agent_orchestration.events]
#[test]
fn replay_reconstructs_state_and_rejects_tampering() {
    let base = base_instance(CompletionPolicy::RunSucceeded);
    let started = start_transition(&base);
    let finished = finish_transition(&started.instance, true);
    let events = vec![
        started.event.expect("start event exists"),
        finished.event.clone().expect("finish event exists"),
    ];
    let head = finished.instance.event_head.clone();

    let replay = replay_events(&base, &events, head.as_deref());
    assert!(replay.valid);
    assert_eq!(replay.instance, finished.instance);

    let mut tampered = events;
    tampered[0].state_hash.push('0');
    let rejected = replay_events(&base, &tampered, head.as_deref());
    assert!(!rejected.valid);
    assert_eq!(rejected.diagnostics[0].code, OrchestrationDiagnosticCode::ReplayMismatch);
}

// r[verify lattice.agent_orchestration.events]
#[test]
fn replay_rejects_wrong_local_head_without_claiming_external_anchor() {
    let base = base_instance(CompletionPolicy::RunSucceeded);
    let started = start_transition(&base);
    let events = vec![started.event.expect("event exists")];

    let report = replay_events(&base, &events, Some("not-the-local-head"));

    assert!(!report.valid);
    assert_eq!(report.diagnostics[0].code, OrchestrationDiagnosticCode::ReplayHeadMismatch);
}

// r[verify lattice.agent_orchestration.triggers]
#[test]
fn trigger_admission_is_scoped_and_duplicate_safe() {
    let definition = definition(CompletionPolicy::RunSucceeded);
    let revision = definition_revision(&definition).expect("definition must hash");
    let empty = OrchestrationTriggerContext {
        accepted: BTreeMap::new(),
    };
    let admitted = admit_trigger_start(TriggerStartRequest {
        definition: &definition,
        definition_revision: &revision,
        trigger_id: TRIGGER_ID,
        trigger_kind: OrchestrationTriggerKind::Webhook,
        idempotency_key: IDEMPOTENCY_KEY,
        next_sequence: FIRST_INSTANCE_SEQUENCE,
        context: &empty,
    })
    .expect("bound trigger must admit");
    let OrchestrationTriggerAdmission::Accepted(plan) = admitted else {
        panic!("first trigger must be accepted");
    };
    let duplicate_context = OrchestrationTriggerContext {
        accepted: BTreeMap::from([(plan.scoped_key.clone(), plan.instance_id.clone())]),
    };
    let duplicate = admit_trigger_start(TriggerStartRequest {
        definition: &definition,
        definition_revision: &revision,
        trigger_id: TRIGGER_ID,
        trigger_kind: OrchestrationTriggerKind::Webhook,
        idempotency_key: IDEMPOTENCY_KEY,
        next_sequence: FIRST_INSTANCE_SEQUENCE,
        context: &duplicate_context,
    })
    .expect("duplicate trigger returns prior result");
    assert_eq!(duplicate, OrchestrationTriggerAdmission::Duplicate {
        instance_id: plan.instance_id
    });

    let error = admit_trigger_start(TriggerStartRequest {
        definition: &definition,
        definition_revision: &revision,
        trigger_id: "missing-trigger",
        trigger_kind: OrchestrationTriggerKind::Webhook,
        idempotency_key: IDEMPOTENCY_KEY,
        next_sequence: FIRST_INSTANCE_SEQUENCE,
        context: &empty,
    })
    .expect_err("unbound trigger must fail");
    assert_eq!(error.code, OrchestrationDiagnosticCode::TriggerNotBound);
}

// r[verify lattice.agent_orchestration.controls]
#[test]
fn pause_and_cancel_preserve_observed_runner_boundary() {
    let base = base_instance(CompletionPolicy::AwaitProposal);
    let paused = transition(
        &base,
        fact(&base, OrchestrationAction::PauseRequested {
            actor: ACTOR.to_string(),
        }),
    )
    .expect("authorized pause must apply")
    .instance;
    assert_eq!(paused.status, OrchestrationStatus::Paused);

    let resumed = transition(
        &paused,
        fact(&paused, OrchestrationAction::ResumeRequested {
            actor: ACTOR.to_string(),
        }),
    )
    .expect("authorized resume must apply")
    .instance;
    assert_eq!(resumed.status, OrchestrationStatus::Ready);

    let running = start_transition(&resumed).instance;
    let cancel_requested = transition(
        &running,
        fact(&running, OrchestrationAction::CancelRequested {
            actor: ACTOR.to_string(),
        }),
    )
    .expect("cancel request must be recorded")
    .instance;
    assert_eq!(cancel_requested.status, OrchestrationStatus::Running);
    assert!(cancel_requested.cancellation_requested);
}

// r[verify lattice.agent_orchestration.unobserved_abandon]
#[test]
fn acknowledged_abandonment_consumes_reservation_and_replays_without_remote_claim() {
    let base = base_instance(CompletionPolicy::RunSucceeded);
    let started = start_transition(&base);
    let dispatch = started.instance.pending_dispatch.as_ref().expect("dispatch must be pending");
    let acknowledged_action = OrchestrationAction::DispatchAcknowledged {
        dispatch_id: dispatch.id.clone(),
    };
    let acknowledged = transition(&started.instance, fact(&started.instance, acknowledged_action))
        .expect("dispatch acknowledgement must apply");
    let cancel_action = OrchestrationAction::CancelRequested {
        actor: ACTOR.to_string(),
    };
    let cancelled = transition(&acknowledged.instance, fact(&acknowledged.instance, cancel_action))
        .expect("cancellation request must apply");
    let active = cancelled.instance.pending_dispatch.as_ref().expect("dispatch remains pending");
    let abandon_action = OrchestrationAction::AbandonPendingDispatch {
        actor: ACTOR.to_string(),
        dispatch_id: active.id.clone(),
        run_id: active.run_id,
    };
    let abandon_fact = fact(&cancelled.instance, abandon_action.clone());
    let abandoned = transition(&cancelled.instance, abandon_fact.clone()).expect("abandonment must apply");

    assert_eq!(abandoned.instance.status, OrchestrationStatus::Failed);
    assert_eq!(abandoned.instance.budgets.consumed, charge());
    assert!(abandoned.instance.budgets.reservations.is_empty());
    assert!(abandoned.instance.active_run_id.is_none());
    assert!(abandoned.instance.pending_dispatch.is_none());
    assert!(!abandoned.instance.cancellation_requested);
    assert_eq!(abandoned.instance.terminal_summary.as_deref(), Some(ABANDONED_ACKNOWLEDGED_SUMMARY));
    assert_eq!(abandoned.event.as_ref().expect("event must exist").kind, OrchestrationEventKind::DispatchAbandoned);

    let events = vec![
        started.event.expect("start event exists"),
        acknowledged.event.expect("acknowledgement event exists"),
        cancelled.event.expect("cancellation event exists"),
        abandoned.event.clone().expect("abandonment event exists"),
    ];
    let replay = replay_events(&base, &events, abandoned.instance.event_head.as_deref());
    assert!(replay.valid);
    assert_eq!(replay.instance, abandoned.instance);

    let duplicate = transition(&abandoned.instance, abandon_fact).expect("same fact must be idempotent");
    assert!(duplicate.duplicate);
    assert!(duplicate.event.is_none());
    let terminal_error = transition(&abandoned.instance, fact(&abandoned.instance, abandon_action))
        .expect_err("new abandonment fact cannot mutate terminal state");
    assert_eq!(terminal_error.code, OrchestrationDiagnosticCode::TerminalStateMutation);
}

// r[verify lattice.agent_orchestration.unobserved_abandon]
#[test]
fn planned_abandonment_releases_reservation_without_consumption() {
    let base = base_instance(CompletionPolicy::RunSucceeded);
    let started = start_transition(&base);
    let cancel_action = OrchestrationAction::CancelRequested {
        actor: ACTOR.to_string(),
    };
    let cancelled = transition(&started.instance, fact(&started.instance, cancel_action))
        .expect("cancellation request must apply")
        .instance;
    let dispatch = cancelled.pending_dispatch.as_ref().expect("dispatch remains pending");
    let abandon_action = OrchestrationAction::AbandonPendingDispatch {
        actor: ACTOR.to_string(),
        dispatch_id: dispatch.id.clone(),
        run_id: dispatch.run_id,
    };

    let abandoned = transition(&cancelled, fact(&cancelled, abandon_action))
        .expect("planned abandonment must apply")
        .instance;

    assert_eq!(abandoned.status, OrchestrationStatus::Failed);
    assert_eq!(abandoned.budgets.consumed, BudgetCharge::default());
    assert!(abandoned.budgets.reservations.is_empty());
    assert_eq!(abandoned.terminal_summary.as_deref(), Some(ABANDONED_PENDING_SUMMARY));
}

// r[verify lattice.agent_orchestration.unobserved_abandon]
#[test]
fn abandonment_rejects_missing_cancellation_authority_and_identity_drift() {
    let base = base_instance(CompletionPolicy::RunSucceeded);
    let running = start_transition(&base).instance;
    let dispatch = running.pending_dispatch.as_ref().expect("dispatch must be pending");
    let action = OrchestrationAction::AbandonPendingDispatch {
        actor: ACTOR.to_string(),
        dispatch_id: dispatch.id.clone(),
        run_id: dispatch.run_id,
    };
    let missing_cancellation =
        transition(&running, fact(&running, action)).expect_err("abandonment requires cancellation");
    assert_eq!(missing_cancellation.code, OrchestrationDiagnosticCode::CancellationRequired);

    let cancel_action = OrchestrationAction::CancelRequested {
        actor: ACTOR.to_string(),
    };
    let cancelled = transition(&running, fact(&running, cancel_action))
        .expect("cancellation request must apply")
        .instance;
    let dispatch = cancelled.pending_dispatch.as_ref().expect("dispatch remains pending");
    let unauthorized_action = OrchestrationAction::AbandonPendingDispatch {
        actor: OTHER_ACTOR.to_string(),
        dispatch_id: dispatch.id.clone(),
        run_id: dispatch.run_id,
    };
    let unauthorized =
        transition(&cancelled, fact(&cancelled, unauthorized_action)).expect_err("unauthorized abandonment must fail");
    assert_eq!(unauthorized.code, OrchestrationDiagnosticCode::UnauthorizedActor);

    let mismatched_action = OrchestrationAction::AbandonPendingDispatch {
        actor: ACTOR.to_string(),
        dispatch_id: DISPATCH_ID.to_string(),
        run_id: dispatch.run_id,
    };
    let mismatched =
        transition(&cancelled, fact(&cancelled, mismatched_action)).expect_err("mismatched abandonment must fail");
    assert_eq!(mismatched.code, OrchestrationDiagnosticCode::DispatchMismatch);
    assert_eq!(cancelled.budgets.consumed, BudgetCharge::default());
    assert_eq!(cancelled.budgets.reservations.len(), FIRST_INSTANCE_SEQUENCE as usize);
}

// r[verify lattice.agent_orchestration.authority]
#[test]
fn approval_identity_binds_authority_policy() {
    let mut instance = base_instance(CompletionPolicy::AwaitProposal);
    instance.status = OrchestrationStatus::WaitingForApproval;
    let successor = workflow_revision(&workflow(SUCCESSOR_WORKFLOW_NAME)).expect("successor must hash");
    let proposal = make_proposal(&instance, successor, "safe proposal").expect("proposal must hash");
    let approval = make_approval(&instance, &proposal, ACTOR, true).expect("approval must hash");

    assert_eq!(
        approval.authority_policy_hash,
        authority_policy_hash(&instance.definition.authority).expect("authority must hash")
    );
    let _: ApprovalDecision = approval;
    let _: RevisionProposal = proposal;
}

#[test]
fn explicitly_supplied_fact_ids_remain_distinct() {
    assert_ne!(FACT_ID_START, FACT_ID_FINISH);
}

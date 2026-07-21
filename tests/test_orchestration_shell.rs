use std::collections::BTreeSet;

use lattice::agent_node::AgentRuntimeConfig;
use lattice::engine::RunId;
use lattice::engine::Workflow;
use lattice::orchestration::AbandonPendingRequest;
use lattice::orchestration::AuthorityPolicy;
use lattice::orchestration::BudgetCharge;
use lattice::orchestration::BudgetChargePlan;
use lattice::orchestration::BudgetDimensions;
use lattice::orchestration::BudgetLimits;
use lattice::orchestration::BudgetPolicy;
use lattice::orchestration::CompletionPolicy;
use lattice::orchestration::DispatchIntent;
use lattice::orchestration::DispatchStatus;
use lattice::orchestration::OrchestrationAction;
use lattice::orchestration::OrchestrationDefinition;
use lattice::orchestration::OrchestrationEvent;
use lattice::orchestration::OrchestrationFact;
use lattice::orchestration::OrchestrationInstance;
use lattice::orchestration::OrchestrationStatus;
use lattice::orchestration::OrchestrationTriggerBinding;
use lattice::orchestration::OrchestrationTriggerKind;
use lattice::orchestration::RecoveryOutcome;
use lattice::orchestration::RedactionPolicy;
use lattice::orchestration::abandon_pending;
use lattice::orchestration::approve;
use lattice::orchestration::current_schema_version;
use lattice::orchestration::dispatch_identity;
use lattice::orchestration::fact_identity;
use lattice::orchestration::make_proposal;
use lattice::orchestration::new_instance;
use lattice::orchestration::recover_pending;
use lattice::orchestration::replay;
use lattice::orchestration::start;
use lattice::orchestration::start_from_trigger;
use lattice::orchestration::transition;
use lattice::orchestration::validate_definition_in_store;
use lattice::orchestration::workflow_revision;
use lattice::store::RunOwner;
use lattice::store::RunRecord;
use lattice::store::Store;
use tempfile::tempdir;

const DEFINITION_ID: &str = "shell-orchestration";
const OBJECTIVE: &str = "Execute bounded workflow rounds";
const FIRST_WORKFLOW_NAME: &str = "shell-round-one";
const SECOND_WORKFLOW_NAME: &str = "shell-round-two";
const SHELL_COMMAND: &str = "printf bounded";
const ACTOR: &str = "operator:shell";
const UNAUTHORIZED_ACTOR: &str = "agent:shell";
const TRIGGER_ID: &str = "shell-webhook";
const IDEMPOTENCY_KEY: &str = "request-one";
const FIRST_SEQUENCE: u64 = 1;
const ROUND_LIMIT: u64 = 2;
const RECOVERY_RUN_ID: u64 = 77;

fn workflow(name: &str) -> Workflow {
    let mut workflow = Workflow::new(name);
    workflow.add_shell_node("bounded", SHELL_COMMAND);
    workflow
}

fn definition(workflow: &Workflow, completion: CompletionPolicy) -> OrchestrationDefinition {
    OrchestrationDefinition {
        schema_version: current_schema_version(),
        id: DEFINITION_ID.to_string(),
        objective: OBJECTIVE.to_string(),
        workflow: workflow_revision(workflow).expect("workflow must hash"),
        completion,
        budgets: BudgetPolicy {
            enforced: BudgetDimensions {
                rounds: true,
                ..BudgetDimensions::default()
            },
            limits: BudgetLimits {
                rounds: ROUND_LIMIT,
                ..BudgetLimits::default()
            },
            per_round_upper_bound: BudgetCharge {
                rounds: FIRST_SEQUENCE,
                ..BudgetCharge::default()
            },
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

fn store() -> (tempfile::TempDir, Store) {
    let directory = tempdir().expect("temporary directory must exist");
    let store = Store::new(directory.path().join("shell.redb")).expect("store must open");
    (directory, store)
}

fn cancelled_acknowledged_instance(workflow: &Workflow) -> (OrchestrationInstance, Vec<OrchestrationEvent>) {
    let base = new_instance(definition(workflow, CompletionPolicy::RunSucceeded), FIRST_SEQUENCE, None)
        .expect("instance must hash");
    let run_id = RunId(RECOVERY_RUN_ID);
    let intent = DispatchIntent {
        id: dispatch_identity(&base, run_id).expect("dispatch must hash"),
        run_id,
        workflow_revision: base.workflow_revision.clone(),
        status: DispatchStatus::Pending,
    };
    let start_action = OrchestrationAction::StartRound {
        intent,
        planned_charge: BudgetChargePlan {
            rounds: Some(FIRST_SEQUENCE),
            agent_invocations: Some(0),
            tokens: Some(0),
            monetary_microunits: Some(0),
            duration_millis: Some(0),
        },
    };
    let started = transition(&base, OrchestrationFact {
        id: fact_identity(&base.id, base.version, &start_action).expect("start fact must hash"),
        action: start_action,
    })
    .expect("start must apply");
    let dispatch_id = started.instance.pending_dispatch.as_ref().expect("dispatch must be pending").id.clone();
    let acknowledge_action = OrchestrationAction::DispatchAcknowledged { dispatch_id };
    let acknowledged = transition(&started.instance, OrchestrationFact {
        id: fact_identity(&started.instance.id, started.instance.version, &acknowledge_action)
            .expect("acknowledgement fact must hash"),
        action: acknowledge_action,
    })
    .expect("acknowledgement must apply");
    let cancel_action = OrchestrationAction::CancelRequested {
        actor: ACTOR.to_string(),
    };
    let cancelled = transition(&acknowledged.instance, OrchestrationFact {
        id: fact_identity(&acknowledged.instance.id, acknowledged.instance.version, &cancel_action)
            .expect("cancellation fact must hash"),
        action: cancel_action,
    })
    .expect("cancellation must apply");
    let events = vec![
        started.event.expect("start event must exist"),
        acknowledged.event.expect("acknowledgement event must exist"),
        cancelled.event.expect("cancellation event must exist"),
    ];
    (cancelled.instance, events)
}

// r[verify lattice.agent_orchestration.rounds]
// r[verify lattice.agent_orchestration.cli]
// r[verify lattice.workflow_execution.headless_run_details.scenario.orchestration]
#[tokio::test]
async fn start_executes_one_seaglass_admitted_round_and_persists_replay() {
    let (_directory, store) = store();
    let workflow = workflow(FIRST_WORKFLOW_NAME);
    store.save_workflow(&workflow).expect("workflow must persist");

    let instance =
        start(&store, definition(&workflow, CompletionPolicy::RunSucceeded), ACTOR, AgentRuntimeConfig::default())
            .await
            .expect("orchestration must run");

    assert_eq!(instance.status, OrchestrationStatus::Completed);
    assert_eq!(instance.round, FIRST_SEQUENCE);
    assert!(instance.active_run_id.is_none());
    let runs = store.list_runs(ROUND_LIMIT as usize).expect("runs must list");
    assert_eq!(runs.len(), FIRST_SEQUENCE as usize);
    let RunOwner::Orchestration {
        instance_id,
        dispatch_id,
    } = &runs[0].owner
    else {
        panic!("orchestration run must retain its owner identity");
    };
    assert_eq!(instance_id, &instance.id);
    assert!(!dispatch_id.is_empty());
    let details = store
        .load_run_details(runs[0].run_id)
        .expect("run details must load")
        .expect("run details must persist");
    assert_eq!(details.nodes.len(), FIRST_SEQUENCE as usize);
    let report = replay(&store, &instance.id).expect("side-effect-free replay must validate");
    assert!(report.valid);
    assert_eq!(report.instance, instance);
}

// r[verify lattice.agent_orchestration.model]
#[test]
fn definition_validation_rejects_stale_workflow_revision() {
    let (_directory, store) = store();
    let mut workflow = workflow(FIRST_WORKFLOW_NAME);
    let definition = definition(&workflow, CompletionPolicy::RunSucceeded);
    workflow.add_shell_node("drift", SHELL_COMMAND);
    store.save_workflow(&workflow).expect("drifted workflow must persist");

    let report = validate_definition_in_store(&store, &definition).expect("validation must run");

    assert!(!report.valid());
    assert!(!report.orchestration_diagnostics.is_empty());
}

// r[verify lattice.agent_orchestration.authority]
// r[verify lattice.agent_orchestration.rounds]
#[tokio::test]
async fn approved_successor_runs_as_a_new_immutable_round() {
    let (_directory, store) = store();
    let first = workflow(FIRST_WORKFLOW_NAME);
    let second = workflow(SECOND_WORKFLOW_NAME);
    store.save_workflow(&first).expect("first workflow must persist");
    store.save_workflow(&second).expect("second workflow must persist");
    let waiting =
        start(&store, definition(&first, CompletionPolicy::AwaitProposal), ACTOR, AgentRuntimeConfig::default())
            .await
            .expect("first round must run");
    assert_eq!(waiting.status, OrchestrationStatus::WaitingForApproval);

    let proposal = make_proposal(
        &waiting,
        workflow_revision(&second).expect("successor must hash"),
        "operator-reviewed successor",
    )
    .expect("proposal must hash");
    let advanced = approve(&store, &waiting.id, proposal, ACTOR, AgentRuntimeConfig::default())
        .await
        .expect("approved successor must run");

    assert_eq!(advanced.status, OrchestrationStatus::WaitingForApproval);
    assert_eq!(advanced.round, ROUND_LIMIT);
    assert_eq!(advanced.workflow_revision.workflow_name, SECOND_WORKFLOW_NAME);
    assert_eq!(store.list_runs(ROUND_LIMIT as usize).expect("runs must list").len(), ROUND_LIMIT as usize);
}

// r[verify lattice.agent_orchestration.triggers]
#[tokio::test]
async fn duplicate_trigger_returns_the_original_instance_without_another_run() {
    let (_directory, store) = store();
    let workflow = workflow(FIRST_WORKFLOW_NAME);
    let definition = definition(&workflow, CompletionPolicy::RunSucceeded);
    store.save_workflow(&workflow).expect("workflow must persist");
    store.save_orchestration_definition(&definition).expect("definition must persist");

    let first = start_from_trigger(
        &store,
        DEFINITION_ID,
        TRIGGER_ID,
        OrchestrationTriggerKind::Webhook,
        IDEMPOTENCY_KEY,
        AgentRuntimeConfig::default(),
    )
    .await
    .expect("first trigger must run");
    let duplicate = start_from_trigger(
        &store,
        DEFINITION_ID,
        TRIGGER_ID,
        OrchestrationTriggerKind::Webhook,
        IDEMPOTENCY_KEY,
        AgentRuntimeConfig::default(),
    )
    .await
    .expect("duplicate trigger must return prior result");

    assert_eq!(duplicate.id, first.id);
    assert_eq!(store.list_orchestrations().expect("instances must list").len(), FIRST_SEQUENCE as usize);
    assert_eq!(store.list_runs(ROUND_LIMIT as usize).expect("runs must list").len(), FIRST_SEQUENCE as usize);
}

// r[verify lattice.agent_orchestration.durability]
#[test]
fn recovery_reconciles_persisted_run_without_redispatch() {
    let (_directory, store) = store();
    let workflow = workflow(FIRST_WORKFLOW_NAME);
    let definition = definition(&workflow, CompletionPolicy::RunSucceeded);
    store.save_workflow(&workflow).expect("workflow must persist");
    let base = new_instance(definition, FIRST_SEQUENCE, None).expect("instance must hash");
    let run_id = RunId(RECOVERY_RUN_ID);
    let intent = DispatchIntent {
        id: dispatch_identity(&base, run_id).expect("dispatch must hash"),
        run_id,
        workflow_revision: base.workflow_revision.clone(),
        status: DispatchStatus::Pending,
    };
    let action = OrchestrationAction::StartRound {
        intent,
        planned_charge: BudgetChargePlan {
            rounds: Some(FIRST_SEQUENCE),
            agent_invocations: Some(0),
            tokens: Some(0),
            monetary_microunits: Some(0),
            duration_millis: Some(0),
        },
    };
    let started = transition(&base, OrchestrationFact {
        id: fact_identity(&base.id, base.version, &action).expect("fact must hash"),
        action,
    })
    .expect("start must apply");
    store
        .create_orchestration(
            &started.instance,
            std::slice::from_ref(started.event.as_ref().expect("event must exist")),
        )
        .expect("started instance must persist");
    store
        .save_run(&RunRecord {
            run_id,
            owner: RunOwner::Orchestration {
                instance_id: started.instance.id.clone(),
                dispatch_id: started
                    .instance
                    .pending_dispatch
                    .as_ref()
                    .expect("dispatch must remain pending")
                    .id
                    .clone(),
            },
            workflow_name: FIRST_WORKFLOW_NAME.to_string(),
            started_epoch_ms: 0,
            finished_epoch_ms: Some(FIRST_SEQUENCE as i64),
            node_count: FIRST_SEQUENCE as usize,
            cache_hits: 0,
            failures: 0,
        })
        .expect("observed run must persist");

    let outcome = recover_pending(&store, &started.instance.id).expect("recovery must run");
    let RecoveryOutcome::Reconciled(reconciled) = outcome else {
        panic!("existing run must reconcile");
    };
    assert_eq!(reconciled.status, OrchestrationStatus::Completed);
    assert!(reconciled.pending_dispatch.is_none());
}

// r[verify lattice.agent_orchestration.unobserved_abandon]
#[test]
fn unobserved_cancelled_dispatch_requires_authority_then_abandons_without_retry() {
    let (_directory, store) = store();
    let workflow = workflow(FIRST_WORKFLOW_NAME);
    store.save_workflow(&workflow).expect("workflow must persist");
    let (instance, events) = cancelled_acknowledged_instance(&workflow);
    store.create_orchestration(&instance, &events).expect("pending instance must persist");

    let outcome = recover_pending(&store, &instance.id).expect("recovery must inspect pending run");
    assert_eq!(outcome, RecoveryOutcome::PendingUnobserved {
        run_id: RunId(RECOVERY_RUN_ID),
    });
    let unauthorized = abandon_pending(&store, AbandonPendingRequest {
        instance_id: &instance.id,
        actor: UNAUTHORIZED_ACTOR,
    })
    .expect_err("unauthorized abandonment must fail");
    assert!(unauthorized.to_string().contains("not authorized"));
    assert_eq!(store.load_orchestration(&instance.id).expect("instance must load"), Some(instance.clone()));

    let abandoned = abandon_pending(&store, AbandonPendingRequest {
        instance_id: &instance.id,
        actor: ACTOR,
    })
    .expect("authorized abandonment must apply");
    assert_eq!(abandoned.status, OrchestrationStatus::Failed);
    assert!(abandoned.pending_dispatch.is_none());
    assert!(abandoned.active_run_id.is_none());
    assert!(abandoned.budgets.reservations.is_empty());
    assert_eq!(abandoned.budgets.consumed.rounds, FIRST_SEQUENCE);
    let report = replay(&store, &instance.id).expect("abandonment replay must validate");
    assert!(report.valid);
    assert_eq!(report.instance, abandoned);

    let repeated = abandon_pending(&store, AbandonPendingRequest {
        instance_id: &instance.id,
        actor: ACTOR,
    })
    .expect_err("terminal abandonment cannot repeat");
    assert!(repeated.to_string().contains("no pending dispatch"));
    assert!(store.load_run(RunId(RECOVERY_RUN_ID)).expect("run lookup must succeed").is_none());
}

// r[verify lattice.agent_orchestration.unobserved_abandon]
#[test]
fn observed_pending_run_rejects_abandonment_and_uses_recovery() {
    let (_directory, store) = store();
    let workflow = workflow(FIRST_WORKFLOW_NAME);
    store.save_workflow(&workflow).expect("workflow must persist");
    let (instance, events) = cancelled_acknowledged_instance(&workflow);
    store.create_orchestration(&instance, &events).expect("pending instance must persist");
    store
        .save_run(&RunRecord {
            run_id: RunId(RECOVERY_RUN_ID),
            owner: RunOwner::Orchestration {
                instance_id: instance.id.clone(),
                dispatch_id: instance.pending_dispatch.as_ref().expect("dispatch must remain pending").id.clone(),
            },
            workflow_name: FIRST_WORKFLOW_NAME.to_string(),
            started_epoch_ms: 0,
            finished_epoch_ms: Some(FIRST_SEQUENCE as i64),
            node_count: FIRST_SEQUENCE as usize,
            cache_hits: 0,
            failures: 0,
        })
        .expect("observed run must persist");

    let rejected = abandon_pending(&store, AbandonPendingRequest {
        instance_id: &instance.id,
        actor: ACTOR,
    })
    .expect_err("observed run cannot be abandoned");
    assert!(rejected.to_string().contains("use orchestration recover"));
    assert_eq!(store.load_orchestration(&instance.id).expect("instance must load"), Some(instance.clone()));

    let outcome = recover_pending(&store, &instance.id).expect("ordinary recovery must reconcile");
    let RecoveryOutcome::Reconciled(reconciled) = outcome else {
        panic!("observed run must reconcile");
    };
    assert_eq!(reconciled.status, OrchestrationStatus::Cancelled);
    assert!(reconciled.pending_dispatch.is_none());
}

// r[verify lattice.agent_orchestration.run_identity]
#[test]
fn foreign_run_record_cannot_satisfy_or_abandon_a_pending_dispatch() {
    let (_directory, store) = store();
    let workflow = workflow(FIRST_WORKFLOW_NAME);
    store.save_workflow(&workflow).expect("workflow must persist");
    let (instance, events) = cancelled_acknowledged_instance(&workflow);
    store.create_orchestration(&instance, &events).expect("pending instance must persist");
    store
        .save_run(&RunRecord {
            run_id: RunId(RECOVERY_RUN_ID),
            owner: RunOwner::Unscoped,
            workflow_name: FIRST_WORKFLOW_NAME.to_string(),
            started_epoch_ms: 0,
            finished_epoch_ms: Some(FIRST_SEQUENCE as i64),
            node_count: FIRST_SEQUENCE as usize,
            cache_hits: 0,
            failures: 0,
        })
        .expect("foreign run must persist");

    let recovery_error = recover_pending(&store, &instance.id).expect_err("foreign run cannot satisfy recovery");
    let abandon_error = abandon_pending(&store, AbandonPendingRequest {
        instance_id: &instance.id,
        actor: ACTOR,
    })
    .expect_err("foreign run blocks abandonment");
    let persisted = store
        .load_orchestration(&instance.id)
        .expect("instance lookup must succeed")
        .expect("instance remains persisted");

    assert!(recovery_error.to_string().contains("not owned by the pending orchestration dispatch"));
    assert!(abandon_error.to_string().contains("not owned by the pending orchestration dispatch"));
    assert_eq!(persisted, instance);
    assert!(persisted.pending_dispatch.is_some());
    assert_eq!(persisted.budgets.reservations.len(), FIRST_SEQUENCE as usize);
}

// r[verify lattice.agent_orchestration.triggers]
#[tokio::test]
async fn unbound_trigger_fails_without_instance_or_run() {
    let (_directory, store) = store();
    let workflow = workflow(FIRST_WORKFLOW_NAME);
    let definition = definition(&workflow, CompletionPolicy::RunSucceeded);
    store.save_workflow(&workflow).expect("workflow must persist");
    store.save_orchestration_definition(&definition).expect("definition must persist");

    let error = start_from_trigger(
        &store,
        DEFINITION_ID,
        "missing-trigger",
        OrchestrationTriggerKind::Schedule,
        IDEMPOTENCY_KEY,
        AgentRuntimeConfig::default(),
    )
    .await
    .expect_err("unbound trigger must fail");

    assert!(error.to_string().contains("not bound"));
    assert!(store.list_orchestrations().expect("instances must list").is_empty());
    assert!(store.list_runs(ROUND_LIMIT as usize).expect("runs must list").is_empty());
}

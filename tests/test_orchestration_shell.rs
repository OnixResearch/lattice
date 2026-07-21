use std::collections::BTreeSet;

use lattice::agent_node::AgentRuntimeConfig;
use lattice::engine::RunId;
use lattice::engine::Workflow;
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
use lattice::orchestration::OrchestrationFact;
use lattice::orchestration::OrchestrationStatus;
use lattice::orchestration::OrchestrationTriggerBinding;
use lattice::orchestration::OrchestrationTriggerKind;
use lattice::orchestration::RecoveryOutcome;
use lattice::orchestration::RedactionPolicy;
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
use lattice::store::RunRecord;
use lattice::store::Store;
use tempfile::tempdir;

const DEFINITION_ID: &str = "shell-orchestration";
const OBJECTIVE: &str = "Execute bounded workflow rounds";
const FIRST_WORKFLOW_NAME: &str = "shell-round-one";
const SECOND_WORKFLOW_NAME: &str = "shell-round-two";
const SHELL_COMMAND: &str = "printf bounded";
const ACTOR: &str = "operator:shell";
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

// r[verify lattice.agent_orchestration.rounds]
// r[verify lattice.agent_orchestration.cli]
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
    assert_eq!(store.list_runs(ROUND_LIMIT as usize).expect("runs must list").len(), FIRST_SEQUENCE as usize);
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

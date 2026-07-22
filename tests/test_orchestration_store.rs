use std::collections::BTreeSet;

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
use lattice::orchestration::RedactionPolicy;
use lattice::orchestration::RetryPolicy;
use lattice::orchestration::current_schema_version;
use lattice::orchestration::dispatch_identity;
use lattice::orchestration::fact_identity;
use lattice::orchestration::new_instance;
use lattice::orchestration::transition;
use lattice::orchestration::workflow_revision;
use lattice::store::Store;
use lattice::store::StoreError;
use lattice::store::TriggerAdmissionWrite;
use tempfile::tempdir;

const DEFINITION_ID: &str = "store-orchestration";
const OBJECTIVE: &str = "Prove atomic storage";
const WORKFLOW_NAME: &str = "store-workflow";
const ACTOR: &str = "operator:store";
const FIRST_SEQUENCE: u64 = 1;
const SECOND_SEQUENCE: u64 = 2;
const RUN_ID: u64 = 41;
const FIRST_RETRY_RUN_ID: u64 = 42;
const SECOND_RETRY_RUN_ID: u64 = 43;
const RETRY_LIMIT: u64 = 2;
const BUDGET_LIMIT: u64 = 10;

fn definition() -> OrchestrationDefinition {
    let workflow = Workflow::new(WORKFLOW_NAME);
    OrchestrationDefinition {
        schema_version: current_schema_version(),
        id: DEFINITION_ID.to_string(),
        objective: OBJECTIVE.to_string(),
        workflow: workflow_revision(&workflow).expect("workflow must hash"),
        completion: CompletionPolicy::RunSucceeded,
        retry: None,
        budgets: BudgetPolicy {
            enforced: BudgetDimensions {
                rounds: true,
                ..BudgetDimensions::default()
            },
            limits: BudgetLimits {
                rounds: BUDGET_LIMIT,
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
        triggers: Vec::new(),
        redaction: RedactionPolicy::default(),
    }
}

fn store() -> (tempfile::TempDir, Store) {
    let directory = tempdir().expect("temporary directory must exist");
    let store = Store::new(directory.path().join("store.redb")).expect("store must open");
    (directory, store)
}

fn start(base: &lattice::orchestration::OrchestrationInstance) -> lattice::orchestration::OrchestrationTransition {
    let run_id = RunId(RUN_ID);
    let intent = DispatchIntent {
        id: dispatch_identity(base, run_id).expect("dispatch must hash"),
        run_id,
        workflow_revision: base.workflow_revision.clone(),
        status: DispatchStatus::Pending,
    };
    let action = OrchestrationAction::StartRound {
        intent,
        planned_charge: BudgetChargePlan {
            rounds: Some(FIRST_SEQUENCE),
            ..BudgetChargePlan::default()
        },
    };
    let fact = OrchestrationFact {
        id: fact_identity(&base.id, base.version, &action).expect("fact must hash"),
        action,
    };
    transition(base, fact).expect("start transition must apply")
}

// r[verify lattice.agent_orchestration.durability]
#[test]
fn store_persists_instance_and_event_atomically() {
    let (_directory, store) = store();
    let definition = definition();
    let sequence = store.next_orchestration_sequence().expect("sequence must allocate");
    let base = new_instance(definition.clone(), sequence, None).expect("instance must hash");
    let started = start(&base);
    let event = started.event.clone().expect("start event must exist");

    store.save_orchestration_definition(&definition).expect("definition must persist");
    store
        .create_orchestration(&started.instance, std::slice::from_ref(&event))
        .expect("instance and event must persist");

    assert_eq!(
        store.load_orchestration(&started.instance.id).expect("load must work"),
        Some(started.instance.clone())
    );
    assert_eq!(store.load_orchestration_events(&started.instance.id).expect("event load must work"), vec![event]);
    assert_eq!(
        store.load_orchestration_definition(DEFINITION_ID).expect("definition load must work"),
        Some(definition)
    );
}

// r[verify lattice.agent_orchestration.budgets]
// r[verify lattice.agent_orchestration.budgets.scenario.concurrent]
// r[verify lattice.agent_orchestration.durability]
#[test]
fn stale_concurrent_transition_cannot_overwrite_newer_state() {
    let (_directory, store) = store();
    let base = new_instance(definition(), FIRST_SEQUENCE, None).expect("instance must hash");
    let started = start(&base);
    let start_event = started.event.clone().expect("start event must exist");
    store
        .create_orchestration(&started.instance, std::slice::from_ref(&start_event))
        .expect("started instance must persist");

    let acknowledge_action = OrchestrationAction::DispatchAcknowledged {
        dispatch_id: started.instance.pending_dispatch.as_ref().expect("dispatch exists").id.clone(),
    };
    let acknowledge = transition(&started.instance, OrchestrationFact {
        id: fact_identity(&started.instance.id, started.instance.version, &acknowledge_action).expect("fact must hash"),
        action: acknowledge_action,
    })
    .expect("acknowledgement must apply");
    store
        .commit_orchestration_transition(
            started.instance.version,
            &acknowledge.instance,
            acknowledge.event.as_ref().expect("acknowledgement event exists"),
        )
        .expect("first writer must commit");

    let cancel_action = OrchestrationAction::CancelRequested {
        actor: ACTOR.to_string(),
    };
    let stale = transition(&started.instance, OrchestrationFact {
        id: fact_identity(&started.instance.id, started.instance.version, &cancel_action).expect("fact must hash"),
        action: cancel_action,
    })
    .expect("stale core decision is locally valid");
    let error = store
        .commit_orchestration_transition(
            started.instance.version,
            &stale.instance,
            stale.event.as_ref().expect("stale event exists"),
        )
        .expect_err("stale writer must fail");
    assert!(matches!(error, StoreError::Conflict(_)));
}

// r[verify lattice.agent_orchestration.retry_round.scenario.success]
// r[verify lattice.agent_orchestration.retry_round.scenario.refusal]
#[test]
fn concurrent_retry_transitions_admit_exactly_one_dispatch() {
    let (_directory, store) = store();
    let mut retry_definition = definition();
    retry_definition.retry = Some(RetryPolicy {
        max_observed_failure_retries: RETRY_LIMIT,
    });
    let base = new_instance(retry_definition, FIRST_SEQUENCE, None).expect("instance must hash");
    let started = start(&base);
    let dispatch = started.instance.pending_dispatch.as_ref().expect("dispatch exists");
    let failed_action = OrchestrationAction::RunFinished {
        dispatch_id: dispatch.id.clone(),
        run_id: dispatch.run_id,
        succeeded: false,
        retryable_observed_failure: true,
        observed_usage: BudgetCharge {
            rounds: FIRST_SEQUENCE,
            ..BudgetCharge::default()
        },
        safe_summary: "observed terminal failure".to_string(),
    };
    let failed = transition(&started.instance, OrchestrationFact {
        id: fact_identity(&started.instance.id, started.instance.version, &failed_action)
            .expect("failure fact must hash"),
        action: failed_action,
    })
    .expect("failure must apply");
    store
        .create_orchestration(&failed.instance, &[
            started.event.expect("start event exists"),
            failed.event.clone().expect("failure event exists"),
        ])
        .expect("failed instance must persist");

    let make_retry = |run_id| {
        let intent = DispatchIntent {
            id: dispatch_identity(&failed.instance, RunId(run_id)).expect("retry dispatch must hash"),
            run_id: RunId(run_id),
            workflow_revision: failed.instance.workflow_revision.clone(),
            status: DispatchStatus::Pending,
        };
        let action = OrchestrationAction::RetryRound {
            actor: ACTOR.to_string(),
            intent,
            planned_charge: BudgetChargePlan {
                rounds: Some(FIRST_SEQUENCE),
                ..BudgetChargePlan::default()
            },
        };
        transition(&failed.instance, OrchestrationFact {
            id: fact_identity(&failed.instance.id, failed.instance.version, &action).expect("retry fact must hash"),
            action,
        })
        .expect("retry decision must apply")
    };
    let first = make_retry(FIRST_RETRY_RUN_ID);
    let second = make_retry(SECOND_RETRY_RUN_ID);
    store
        .commit_orchestration_transition(
            failed.instance.version,
            &first.instance,
            first.event.as_ref().expect("first retry event exists"),
        )
        .expect("first retry writer must commit");
    let conflict = store
        .commit_orchestration_transition(
            failed.instance.version,
            &second.instance,
            second.event.as_ref().expect("second retry event exists"),
        )
        .expect_err("second retry writer must lose optimistic commit");

    assert!(matches!(conflict, StoreError::Conflict(_)));
    let current = store
        .load_orchestration(&failed.instance.id)
        .expect("current instance must load")
        .expect("current instance must exist");
    assert_eq!(current, first.instance);
    assert_eq!(current.budgets.reservations.len(), FIRST_SEQUENCE as usize);
    assert_eq!(current.retry.admitted, FIRST_SEQUENCE);
}

// r[verify lattice.agent_orchestration.triggers]
#[test]
fn trigger_admission_insert_is_atomic_and_duplicate_safe() {
    let (_directory, store) = store();
    let first = store
        .save_orchestration_trigger_admission("scoped-key", "instance-one")
        .expect("first trigger write must work");
    let duplicate = store
        .save_orchestration_trigger_admission("scoped-key", "instance-two")
        .expect("duplicate trigger write must return prior result");

    assert_eq!(first, TriggerAdmissionWrite::Inserted);
    assert_eq!(duplicate, TriggerAdmissionWrite::Duplicate {
        instance_id: "instance-one".to_string()
    });
    assert_eq!(
        store
            .list_orchestration_trigger_admissions()
            .expect("admissions must list")
            .get("scoped-key")
            .map(String::as_str),
        Some("instance-one")
    );
}

#[test]
fn orchestration_sequence_is_monotonic() {
    let (_directory, store) = store();

    assert_eq!(store.next_orchestration_sequence().expect("first sequence must allocate"), FIRST_SEQUENCE);
    assert_eq!(store.next_orchestration_sequence().expect("second sequence must allocate"), SECOND_SEQUENCE);
}

// r[verify lattice.agent_orchestration.events]
#[test]
fn mismatched_creation_event_batch_is_rejected_without_partial_state() {
    let (_directory, store) = store();
    let base = new_instance(definition(), FIRST_SEQUENCE, None).expect("instance must hash");
    let started = start(&base);
    let mut event = started.event.expect("event must exist");
    event.instance_id = "different-instance".to_string();

    let error = store
        .create_orchestration(&started.instance, &[event])
        .expect_err("mismatched event batch must fail");

    assert!(matches!(error, StoreError::Conflict(_)));
    assert_eq!(store.load_orchestration(&started.instance.id).expect("load must work"), None);
}

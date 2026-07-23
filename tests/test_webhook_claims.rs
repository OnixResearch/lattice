//! Pure and durable atomic webhook-claim tests.

mod common;

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Barrier;
use std::thread;

use lattice::engine::RunId;
use lattice::scheduler::OverlapPolicy;
use lattice::triggers::IdempotencyPolicy;
use lattice::triggers::TriggerDefinition;
use lattice::triggers::TriggerKind;
use lattice::triggers::TriggerRunPlan;
use lattice::triggers::TriggerRunStatus;
use lattice::triggers::WebhookAdmissionDecision;
use lattice::triggers::WebhookAdmissionDecisionError;
use lattice::triggers::WebhookClaimPlan;
use lattice::triggers::WebhookClaimPlanError;
use lattice::triggers::WebhookClaimResolution;
use lattice::triggers::decide_webhook_admission;
use lattice::triggers::plan_webhook_claim;
use pretty_assertions::assert_eq;

const TRIGGER_ID: &str = "atomic-webhook";
const OTHER_TRIGGER_ID: &str = "other-atomic-webhook";
const WORKFLOW_NAME: &str = "atomic-workflow";
const EVENT_KEY: &str = "raw-event-key-must-not-persist";
const PAYLOAD_SUMMARY: &str = "[payload redacted]";
const ACCEPTED_EPOCH_MS: i64 = 1_000;
const INSERTED_RUN_ID: RunId = RunId(7);
const EXISTING_RUN_ID: RunId = RunId(9);
const CLAIM_CONTENDER_COUNT: usize = 8;
const EXPECTED_INSERTED_COUNT: usize = 1;
const EXPECTED_DUPLICATE_COUNT: usize = CLAIM_CONTENDER_COUNT - EXPECTED_INSERTED_COUNT;
const EXPECTED_CLAIM_COUNT: usize = 1;
const EXPECTED_HISTORY_COUNT: usize = 1;
const FOLLOWING_RUN_ID_INCREMENT: u64 = 1;

fn trigger(id: &str, idempotency_policy: IdempotencyPolicy) -> TriggerDefinition {
    TriggerDefinition {
        id: id.to_string(),
        workflow_name: WORKFLOW_NAME.to_string(),
        enabled: true,
        kind: TriggerKind::Manual,
        input_bindings: Vec::new(),
        overlap_policy: OverlapPolicy::Allow,
        idempotency_policy,
    }
}

fn run_plan(trigger_id: &str, key: Option<&str>) -> TriggerRunPlan {
    TriggerRunPlan {
        trigger_id: trigger_id.to_string(),
        workflow_name: WORKFLOW_NAME.to_string(),
        bound_inputs: HashMap::new(),
        idempotency_key: key.map(str::to_string),
        payload_summary: PAYLOAD_SUMMARY.to_string(),
    }
}

fn required_request(plan: &WebhookClaimPlan) -> lattice::triggers::WebhookClaimRequest {
    let WebhookClaimPlan::Required(request) = plan else {
        panic!("idempotent trigger must require a durable claim");
    };
    request.clone()
}

// r[verify lattice.scheduling.webhook_atomic_idempotency]
// r[verify lattice.scheduling.webhook_atomic_idempotency.scenario.ordering]
#[test]
fn claim_planning_and_post_transaction_decision_are_deterministic_and_safe() {
    let trigger = trigger(TRIGGER_ID, IdempotencyPolicy::EventKey);
    let first = plan_webhook_claim(&trigger, &run_plan(TRIGGER_ID, Some(EVENT_KEY)), ACCEPTED_EPOCH_MS)
        .expect("valid idempotent plan produces a claim");
    let second = plan_webhook_claim(&trigger, &run_plan(TRIGGER_ID, Some(EVENT_KEY)), ACCEPTED_EPOCH_MS)
        .expect("same plan produces a claim");
    let request = required_request(&first);

    assert_eq!(first, second);
    assert!(request.identity.starts_with("b3:"));
    assert!(!request.identity.contains(EVENT_KEY));
    assert_eq!(
        decide_webhook_admission(
            &first,
            Some(WebhookClaimResolution::Inserted {
                run_id: INSERTED_RUN_ID
            })
        ),
        Ok(WebhookAdmissionDecision::ExecuteClaimed {
            run_id: INSERTED_RUN_ID,
        }),
    );
    assert_eq!(
        decide_webhook_admission(
            &first,
            Some(WebhookClaimResolution::Duplicate {
                existing_run_id: EXISTING_RUN_ID,
            }),
        ),
        Ok(WebhookAdmissionDecision::RejectDuplicate {
            existing_run_id: EXISTING_RUN_ID,
        }),
    );
    let claim_json = serde_json::to_string(&request.claim(INSERTED_RUN_ID)).expect("claim serializes");
    let history_json = serde_json::to_string(&request.started_record(INSERTED_RUN_ID)).expect("history serializes");
    assert!(!claim_json.contains(EVENT_KEY));
    assert!(!history_json.contains(EVENT_KEY));
}

// r[verify lattice.scheduling.webhook_atomic_idempotency]
#[test]
fn claim_planning_rejects_malformed_missing_and_cross_trigger_aliases() {
    let primary = trigger(TRIGGER_ID, IdempotencyPolicy::EventKey);
    let other = trigger(OTHER_TRIGGER_ID, IdempotencyPolicy::EventKey);
    let primary_plan = run_plan(TRIGGER_ID, Some(EVENT_KEY));
    let other_plan = run_plan(OTHER_TRIGGER_ID, Some(EVENT_KEY));
    let primary_claim =
        required_request(&plan_webhook_claim(&primary, &primary_plan, ACCEPTED_EPOCH_MS).expect("primary claim plans"));
    let other_claim =
        required_request(&plan_webhook_claim(&other, &other_plan, ACCEPTED_EPOCH_MS).expect("other claim plans"));
    let mut empty_workflow = primary_plan.clone();
    empty_workflow.workflow_name.clear();

    assert_ne!(primary_claim.identity, other_claim.identity);
    assert_eq!(
        plan_webhook_claim(&primary, &run_plan(TRIGGER_ID, None), ACCEPTED_EPOCH_MS),
        Err(WebhookClaimPlanError::MissingIdempotencyKey),
    );
    assert_eq!(
        plan_webhook_claim(&primary, &run_plan(TRIGGER_ID, Some("")), ACCEPTED_EPOCH_MS),
        Err(WebhookClaimPlanError::EmptyIdempotencyKey),
    );
    assert_eq!(
        plan_webhook_claim(&primary, &other_plan, ACCEPTED_EPOCH_MS),
        Err(WebhookClaimPlanError::TriggerMismatch),
    );
    assert_eq!(
        plan_webhook_claim(
            &trigger("", IdempotencyPolicy::EventKey),
            &run_plan("", Some(EVENT_KEY)),
            ACCEPTED_EPOCH_MS
        ),
        Err(WebhookClaimPlanError::EmptyTriggerId),
    );
    assert_eq!(
        plan_webhook_claim(&primary, &empty_workflow, ACCEPTED_EPOCH_MS),
        Err(WebhookClaimPlanError::EmptyWorkflowName),
    );
}

// r[verify lattice.scheduling.webhook_atomic_idempotency.scenario.disabled]
#[test]
fn idempotency_disabled_plan_creates_no_hidden_claim_and_rejects_claim_facts() {
    let no_claim = plan_webhook_claim(
        &trigger(TRIGGER_ID, IdempotencyPolicy::None),
        &run_plan(TRIGGER_ID, None),
        ACCEPTED_EPOCH_MS,
    )
    .expect("disabled idempotency remains explicit");

    assert_eq!(no_claim, WebhookClaimPlan::NotRequired);
    assert_eq!(decide_webhook_admission(&no_claim, None), Ok(WebhookAdmissionDecision::ExecuteUnclaimed));
    assert_eq!(
        decide_webhook_admission(
            &no_claim,
            Some(WebhookClaimResolution::Inserted {
                run_id: INSERTED_RUN_ID
            })
        ),
        Err(WebhookAdmissionDecisionError::UnexpectedClaimResolution),
    );
    let required = plan_webhook_claim(
        &trigger(TRIGGER_ID, IdempotencyPolicy::EventKey),
        &run_plan(TRIGGER_ID, Some(EVENT_KEY)),
        ACCEPTED_EPOCH_MS,
    )
    .expect("required claim plans");
    assert_eq!(
        decide_webhook_admission(&required, None),
        Err(WebhookAdmissionDecisionError::MissingClaimResolution),
    );
}

// r[verify lattice.scheduling.webhook_atomic_idempotency]
// r[verify lattice.scheduling.webhook_atomic_idempotency.scenario.concurrent]
// r[verify lattice.scheduling.webhook_atomic_idempotency.scenario.disabled]
#[test]
fn redb_claim_race_selects_one_immutable_owner_and_reserves_one_run() {
    let (store, _dir) = common::temp_store();
    let store = Arc::new(store);
    let plan = plan_webhook_claim(
        &trigger(TRIGGER_ID, IdempotencyPolicy::EventKey),
        &run_plan(TRIGGER_ID, Some(EVENT_KEY)),
        ACCEPTED_EPOCH_MS,
    )
    .expect("race claim plans");
    let request = Arc::new(required_request(&plan));
    let barrier = Arc::new(Barrier::new(CLAIM_CONTENDER_COUNT));
    let mut contenders = Vec::with_capacity(CLAIM_CONTENDER_COUNT);
    for _ in 0..CLAIM_CONTENDER_COUNT {
        let store = Arc::clone(&store);
        let request = Arc::clone(&request);
        let barrier = Arc::clone(&barrier);
        contenders.push(thread::spawn(move || {
            barrier.wait();
            store.claim_webhook(&request).expect("claim contender completes")
        }));
    }
    let resolutions = contenders
        .into_iter()
        .map(|contender| contender.join().expect("claim contender joins"))
        .collect::<Vec<_>>();
    let inserted = resolutions
        .iter()
        .filter_map(|resolution| match resolution {
            WebhookClaimResolution::Inserted { run_id } => Some(*run_id),
            WebhookClaimResolution::Duplicate { .. } => None,
        })
        .collect::<Vec<_>>();
    let duplicates = resolutions
        .iter()
        .filter_map(|resolution| match resolution {
            WebhookClaimResolution::Duplicate { existing_run_id } => Some(*existing_run_id),
            WebhookClaimResolution::Inserted { .. } => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(inserted.len(), EXPECTED_INSERTED_COUNT);
    assert_eq!(duplicates.len(), EXPECTED_DUPLICATE_COUNT);
    assert!(duplicates.iter().all(|run_id| *run_id == inserted[0]));
    let claims = store
        .list_webhook_claims(u64::try_from(EXPECTED_CLAIM_COUNT).expect("claim count fits u64"))
        .expect("claims list");
    let history = store.list_trigger_runs(EXPECTED_HISTORY_COUNT).expect("claim history lists");
    assert_eq!(claims.len(), EXPECTED_CLAIM_COUNT);
    assert_eq!(history.len(), EXPECTED_HISTORY_COUNT);
    assert_eq!(claims[0], request.claim(inserted[0]));
    assert_eq!(history[0].status, TriggerRunStatus::Started);
    assert_eq!(history[0].claim_identity.as_deref(), Some(request.identity.as_str()));
    assert_eq!(history[0].idempotency_key, None);
    assert!(store.list_runs(EXPECTED_CLAIM_COUNT).expect("terminal runs list").is_empty());
    assert_eq!(store.next_run_id().expect("following run reserves").0, inserted[0].0 + FOLLOWING_RUN_ID_INCREMENT);

    let disabled = plan_webhook_claim(
        &trigger(TRIGGER_ID, IdempotencyPolicy::None),
        &run_plan(TRIGGER_ID, None),
        ACCEPTED_EPOCH_MS,
    )
    .expect("disabled idempotency plans");
    assert_eq!(disabled, WebhookClaimPlan::NotRequired);
    assert_eq!(
        store
            .list_webhook_claims(u64::try_from(EXPECTED_CLAIM_COUNT).expect("claim count fits u64"))
            .expect("disabled path adds no claims")
            .len(),
        EXPECTED_CLAIM_COUNT
    );
}

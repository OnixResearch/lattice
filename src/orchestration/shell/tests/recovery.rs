use super::known_status;
use super::pending_usage_fixture;

// r[verify lattice.agent_orchestration.usage.scenario.recovery]
// r[verify lattice.agent_orchestration.usage.scenario.cancellation]
#[test]
fn recovery_before_producer_outcome_refines_evidence_and_settles_once() {
    use crate::orchestration::*;
    let (directory, store, started) = pending_usage_fixture(Some(52));
    let dispatch = started.pending_dispatch.as_ref().expect("pending dispatch");
    assert_eq!(
        super::super::recover_pending(&store, started.id.as_str()).expect("early recovery"),
        RecoveryOutcome::PendingUnobserved {
            run_id: dispatch.run_id
        }
    );
    let frozen = store.load_command_worker_usage_for_run(dispatch.run_id).expect("receipt loads").expect("receipt");
    assert_eq!(frozen.accounting.as_ref().expect("accounting").outcome.expect("partial").cancelled, None);
    let mut crossed = frozen.clone();
    crossed.instance_id = "crossed-instance".to_string();
    crossed.accounting.as_mut().expect("accounting").outcome = Some(known_status(true));
    assert!(matches!(store.save_command_worker_usage(&crossed), Err(crate::store::StoreError::Conflict(_))));
    let mut changed_charge = frozen.clone();
    let accounting = changed_charge.accounting.as_mut().expect("accounting");
    accounting.charged.tokens += 1;
    accounting.outcome = Some(known_status(true));
    assert!(matches!(
        store.save_command_worker_usage(&changed_charge),
        Err(crate::store::StoreError::Conflict(_))
    ));

    // This is the same completion boundary used after the live producer returns.
    // Do not commit its action: exercise a restart after durable refinement.
    super::super::finish_action(&store, &started, super::super::FinishRequest {
        dispatch_id: dispatch.id.clone(),
        run_id: dispatch.run_id,
        status: known_status(true),
        is_retryable_observed_failure: false,
        safe_summary: "producer returned".to_string(),
    })
    .expect("later typed cancellation evidence refines the early receipt");
    let refined = store.load_command_worker_usage_for_run(dispatch.run_id).expect("receipt loads").expect("receipt");
    let mut expected = frozen.clone();
    expected.accounting.as_mut().expect("accounting").outcome = Some(known_status(true));
    assert_eq!(refined, expected, "only the outcome changes; reservation and charge remain frozen");
    store
        .save_command_worker_usage(&frozen)
        .expect("stale partial evidence cannot downgrade known evidence");
    let mut conflict = refined.clone();
    conflict.accounting.as_mut().expect("accounting").outcome = Some(known_status(false));
    assert!(matches!(store.save_command_worker_usage(&conflict), Err(crate::store::StoreError::Conflict(_))));
    assert_eq!(
        store.load_command_worker_usage_for_run(dispatch.run_id).expect("receipt loads"),
        Some(refined.clone())
    );
    drop(store);

    let store = crate::store::Store::new(directory.path().join("usage.redb")).expect("reopen");
    let RecoveryOutcome::Reconciled(finished) =
        super::super::recover_pending(&store, started.id.as_str()).expect("settles")
    else {
        panic!("known evidence must settle");
    };
    assert_eq!(finished.status, crate::orchestration::Status::Completed);
    assert_eq!(finished.budgets.consumed, refined.accounting.as_ref().expect("accounting").charged);
    assert!(finished.budgets.reservations.is_empty());
    assert_eq!(
        super::super::recover_pending(&store, started.id.as_str()).expect("repeat"),
        RecoveryOutcome::NothingPending
    );
    assert_eq!(store.load_command_worker_usage_for_run(dispatch.run_id).expect("receipt loads"), Some(refined));
    assert!(super::super::replay(&store, started.id.as_str()).expect("durable replay").valid);
}

// r[verify lattice.agent_orchestration.usage.scenario.recovery]
#[test]
fn receipt_survives_restart_before_settlement_and_is_applied_once() {
    use crate::orchestration::*;
    let (directory, store, started) = pending_usage_fixture(Some(52));
    let dispatch = started.pending_dispatch.as_ref().expect("pending");
    let action = super::super::finish_action(&store, &started, super::super::FinishRequest {
        dispatch_id: dispatch.id.clone(),
        run_id: dispatch.run_id,
        status: known_status(true),
        is_retryable_observed_failure: false,
        safe_summary: "done".to_string(),
    })
    .expect("receipt persists before settlement");
    let receipt = store
        .load_command_worker_usage_for_run(dispatch.run_id)
        .expect("receipt loads")
        .expect("receipt exists");
    let accounting = receipt.accounting.as_ref().expect("accounting");
    assert_eq!(accounting.reserved.duration_millis, 500);
    assert_eq!(accounting.charged.duration_millis, 42);
    assert_eq!(accounting.charged.tokens, 100);
    assert!(accounting.unknown_tokens && accounting.unknown_model_requests && accounting.unknown_cost);
    assert!(receipt.worker_profile.is_empty());
    assert_eq!(store.load_orchestration(&started.id).expect("pending loads"), Some(started.clone()));
    drop(store);
    let store = crate::store::Store::new(directory.path().join("usage.redb")).expect("reopen");
    assert_eq!(
        super::super::finish_action(&store, &started, super::super::FinishRequest {
            dispatch_id: dispatch.id.clone(),
            run_id: dispatch.run_id,
            status: known_status(true),
            is_retryable_observed_failure: false,
            safe_summary: "done".to_string(),
        },)
        .expect("same frozen receipt"),
        action
    );
    let RecoveryOutcome::Reconciled(finished) =
        super::super::recover_pending(&store, started.id.as_str()).expect("recovery settles")
    else {
        panic!("reconciled")
    };
    assert_eq!(finished.budgets.consumed, accounting.charged);
    assert_eq!(finished.last_usage_basis, BudgetUsageBasis::ConservativeObservation);
    assert_eq!(
        super::super::recover_pending(&store, started.id.as_str()).expect("repeat recovery"),
        RecoveryOutcome::NothingPending
    );
    assert_eq!(store.load_command_worker_usage_for_run(dispatch.run_id).expect("receipt unchanged"), Some(receipt));
}

#[test]
fn crossed_receipt_cannot_settle_or_refund_reservation() {
    let (_directory, store, started) = pending_usage_fixture(Some(52));
    let dispatch = started.pending_dispatch.as_ref().expect("pending");
    super::super::finish_action(&store, &started, super::super::FinishRequest {
        dispatch_id: dispatch.id.clone(),
        run_id: dispatch.run_id,
        status: known_status(true),
        is_retryable_observed_failure: false,
        safe_summary: "done".to_string(),
    })
    .expect("host receipt persists");
    let mut crossed = started.clone();
    crossed.workflow_revision.content_hash =
        crate::orchestration::WorkflowRevision::try_from_hex("f".repeat(64)).expect("other revision");
    assert!(
        super::super::finish_action(&store, &crossed, super::super::FinishRequest {
            dispatch_id: dispatch.id.clone(),
            run_id: dispatch.run_id,
            status: known_status(true),
            is_retryable_observed_failure: false,
            safe_summary: "done".to_string(),
        },)
        .is_err()
    );
    assert_eq!(store.load_orchestration(&started.id).expect("pending remains"), Some(started));
}

// r[verify lattice.agent_orchestration.usage.scenario.shell]
#[test]
fn opaque_shell_wrappers_cannot_bypass_accounting_admission() {
    let (_directory, _store, started) = pending_usage_fixture(Some(52));
    let mut workflow = crate::engine::Workflow::new("opaque", crate::engine::workflow::WorkflowTimestamp::epoch());
    workflow.add_shell_node("wrapper", "opaque-wrapper --unknown-operation");
    assert!(super::super::lifecycle::admission::planned_charge(&started.definition, &workflow).is_err());
    let mut duration_only = started.definition;
    duration_only.budgets.enforced.tokens = false;
    duration_only.budgets.enforced.monetary_microunits = false;
    assert!(super::super::lifecycle::admission::planned_charge(&duration_only, &workflow).is_ok());
}

// r[verify lattice.agent_orchestration.usage.scenario.uncertainty]
#[test]
fn terminal_record_without_owned_cancellation_evidence_remains_pending() {
    let (_directory, store, started) = pending_usage_fixture(Some(52));
    let dispatch = started.pending_dispatch.as_ref().expect("pending");
    assert!(store.load_run(dispatch.run_id).expect("run").expect("terminal run").finished_epoch_ms.is_some());
    assert_eq!(
        super::super::recover_pending(&store, started.id.as_str()).expect("unknown outcome remains pending"),
        super::super::RecoveryOutcome::PendingUnobserved {
            run_id: dispatch.run_id
        }
    );
    assert_eq!(store.load_orchestration(&started.id).expect("pending unchanged"), Some(started.clone()));
    let accounting = store
        .load_command_worker_usage_for_run(dispatch.run_id)
        .expect("receipt")
        .expect("conservative receipt")
        .accounting
        .expect("accounting");
    assert_eq!(accounting.outcome.expect("partial outcome").cancelled, None);
    assert_eq!(accounting.charged.tokens, accounting.reserved.tokens);
    assert!(accounting.unknown_tokens && accounting.unknown_cost);
}

// r[verify lattice.agent_orchestration.usage.scenario.uncertainty]
#[test]
fn interrupted_run_keeps_reservation_and_unknown_remainder() {
    let (_directory, store, started) = pending_usage_fixture(None);
    let dispatch = started.pending_dispatch.as_ref().expect("pending");
    assert!(
        super::super::finish_action(&store, &started, super::super::FinishRequest {
            dispatch_id: dispatch.id.clone(),
            run_id: dispatch.run_id,
            status: crate::orchestration::usage::RunOutcomeObservation {
                succeeded: false,
                cancelled: None,
            },
            is_retryable_observed_failure: false,
            safe_summary: "interrupted".to_string(),
        },)
        .is_err()
    );
    let accounting = store
        .load_command_worker_usage_for_run(dispatch.run_id)
        .expect("receipt loads")
        .expect("receipt exists")
        .accounting
        .expect("accounting");
    assert_eq!(accounting.charged, accounting.reserved);
    assert!(accounting.host_interval.is_none());
    assert!(
        accounting.unknown_tokens
            && accounting.unknown_cost
            && accounting.unknown_model_requests
            && accounting.unknown_subagent_calls
    );
    assert_eq!(store.load_orchestration(&started.id).expect("pending loads"), Some(started.clone()));
    assert_eq!(
        super::super::recover_pending(&store, started.id.as_str()).expect("unknown recovery remains pending"),
        super::super::RecoveryOutcome::PendingUnobserved {
            run_id: dispatch.run_id
        }
    );
    assert_eq!(store.load_orchestration(&started.id).expect("pending unchanged"), Some(started));
}

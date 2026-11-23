#[test]
fn real_completion_wins_late_durable_and_atomic_cancel_intent() {
    for (command, expected) in [
        ("printf completed", crate::workflow_exchange::DurableExecutionState::Completed),
        ("exit 23", crate::workflow_exchange::DurableExecutionState::Failed),
    ] {
        for late_cancel in [false, true] {
            let (_directory, store, workflow, _contract, request, profile) =
                crate::workflow_exchange::tests::fixture(command);
            let route = &profile.routes[0];
            let handler_identity = crate::workflow_exchange::handler_profile_identity(&profile).unwrap();
            let admission = store
                .admit_workflow_exchange(request, route, route.workflow_revision.clone(), handler_identity)
                .unwrap();
            let crate::workflow_exchange::AdmissionResolution::Inserted(admitted) = admission else {
                panic!("fresh fixture must admit once");
            };
            let running = crate::workflow_exchange::mark_running(&admitted).unwrap();
            store.save_workflow_exchange_transition(&admitted, &running).unwrap();
            let authority = crate::workflow_exchange::RouteExactAuthorityProvider::new(
                &running.handler_profile_identity,
                &running.request.identity,
                &running.workflow_revision,
                crate::engine::RunId(running.run_id),
            );
            let cancellation = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
            let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
            let outcome = runtime
                .block_on(crate::headless::run_workflow_snapshot_with_authority_provider_and_cancellation(
                    &store,
                    crate::headless::SnapshotRunRequest {
                        workflow,
                        run_id: crate::engine::RunId(running.run_id),
                        owner: crate::store::RunOwner::Unscoped,
                        agent_runtime: crate::agent_node::RuntimeOptions::default(),
                        cancellation: std::sync::Arc::clone(&cancellation),
                    },
                    &authority,
                ))
                .expect("real process completion is retained");
            assert!(!outcome.cancelled, "a completed process is not cancelled");
            let current = if late_cancel {
                cancellation.store(true, std::sync::atomic::Ordering::Release);
                let requested = crate::workflow_exchange::request_cancellation(&running, running.sequence).unwrap();
                store.save_workflow_exchange_transition(&running, &requested).unwrap();
                requested
            } else {
                running
            };
            let facts = super::completion_facts(&store, &current, Ok(outcome));
            let completed = crate::workflow_exchange::complete(&current, &facts).unwrap();
            store.save_workflow_exchange_transition(&current, &completed).unwrap();
            let durable = store.load_workflow_exchange(&completed.accepted.execution_identity).unwrap().unwrap();
            assert_eq!(durable.state, expected, "late intent must not replace the observed process outcome");
            let expected_node = if expected == crate::workflow_exchange::DurableExecutionState::Completed {
                lattice_workflow_contract::NodeState::Completed
            } else {
                lattice_workflow_contract::NodeState::Failed
            };
            assert_eq!(durable.nodes[0].state, expected_node);
        }
    }
}

// r[verify lattice.application_checkpoint.cancellation]
/// A Cancel that lands while a fresh dispatch is still `Queued` bumps its sequence before the
/// worker's `Queued` to `Running` compare-and-swap, which then fails. The worker settles the
/// reloaded record `cancelled` and runs nothing.
#[test]
fn cancel_before_a_fresh_worker_starts_settles_cancelled_unrun() {
    let effects = tempfile::tempdir().expect("effects directory");
    let marker = effects.path().join("ran");
    let (_directory, store, workflow, _contract, request, profile) =
        crate::workflow_exchange::tests::fixture(&format!("touch {}", marker.display()));
    let route = &profile.routes[0];
    let handler_identity = crate::workflow_exchange::handler_profile_identity(&profile).unwrap();
    let admission = store
        .admit_workflow_exchange(request, route, route.workflow_revision.clone(), handler_identity)
        .unwrap();
    let crate::workflow_exchange::AdmissionResolution::Inserted(admitted) = admission else {
        panic!("fresh fixture must admit once");
    };
    let requested = crate::workflow_exchange::request_cancellation(&admitted, admitted.sequence).unwrap();
    store.save_workflow_exchange_transition(&admitted, &requested).unwrap();
    super::run(super::WorkerInput {
        store: store.clone(),
        admitted: admitted.clone(),
        authority_mode: route.effect_authority,
        workflow,
        agent_runtime: crate::agent_node::RuntimeOptions::default(),
        cancellation: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true)),
        checkpoint: None,
        seed: crate::checkpoint::HookSeed::default(),
        restored: Vec::new(),
    });
    let durable = store.load_workflow_exchange(&admitted.accepted.execution_identity).unwrap().unwrap();
    assert_eq!(
        durable.state,
        crate::workflow_exchange::DurableExecutionState::Cancelled,
        "a cancel that wins the start race settles the execution cancelled"
    );
    assert!(durable.nodes.iter().all(|node| node.state == lattice_workflow_contract::NodeState::Cancelled));
    assert!(store.load_run_details(crate::engine::RunId(admitted.run_id)).unwrap().is_none(), "no run started");
    assert!(!marker.exists(), "no effect ran");
}

// r[verify lattice.application_checkpoint.cancellation]
/// A Cancel after the worker's pre-start check but before the first plan: the ready frontier
/// blocks every node on the cancellation and dispatches nothing, without observing a cancel. The
/// stall alone, with no other node result, settles the execution `cancelled`.
#[test]
fn cancel_before_the_first_plan_settles_cancelled_unrun() {
    let effects = tempfile::tempdir().expect("effects directory");
    let marker = effects.path().join("ran");
    let (_directory, store, workflow, _contract, request, profile) =
        crate::workflow_exchange::tests::fixture(&format!("touch {}", marker.display()));
    let route = &profile.routes[0];
    let handler_identity = crate::workflow_exchange::handler_profile_identity(&profile).unwrap();
    let admission = store
        .admit_workflow_exchange(request, route, route.workflow_revision.clone(), handler_identity)
        .unwrap();
    let crate::workflow_exchange::AdmissionResolution::Inserted(admitted) = admission else {
        panic!("fresh fixture must admit once");
    };
    let running = crate::workflow_exchange::mark_running(&admitted).unwrap();
    store.save_workflow_exchange_transition(&admitted, &running).unwrap();
    let authority = crate::workflow_exchange::RouteExactAuthorityProvider::new(
        &running.handler_profile_identity,
        &running.request.identity,
        &running.workflow_revision,
        crate::engine::RunId(running.run_id),
    );
    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    let outcome = runtime
        .block_on(crate::headless::run_workflow_snapshot_with_authority_provider_and_cancellation(
            &store,
            crate::headless::SnapshotRunRequest {
                workflow,
                run_id: crate::engine::RunId(running.run_id),
                owner: crate::store::RunOwner::Unscoped,
                agent_runtime: crate::agent_node::RuntimeOptions::default(),
                cancellation: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true)),
            },
            &authority,
        ))
        .expect("the stalled run is retained");
    assert!(!outcome.cancelled, "the frontier stall observes no cancel");
    let facts = super::completion_facts(&store, &running, Ok(outcome));
    let settled = crate::workflow_exchange::complete(&running, &facts).unwrap();
    assert_eq!(
        settled.state,
        crate::workflow_exchange::DurableExecutionState::Cancelled,
        "a cancel before the first plan settles the execution cancelled"
    );
    assert!(settled.nodes.iter().all(|node| node.state == lattice_workflow_contract::NodeState::Cancelled));
    assert!(!marker.exists(), "no effect ran");
}

/// The store, with a client's Cancel landing right after each of the next `cancels` loads: the
/// worker's compare-and-swap against what it just read then loses again.
struct CancelAfterLoad {
    store: crate::store::Store,
    cancels: std::cell::Cell<usize>,
}

impl super::Records for CancelAfterLoad {
    fn load(&self, execution: &lattice_workflow_contract::Identity) -> Option<super::ExecutionRecord> {
        let loaded = super::Records::load(&self.store, execution)?;
        if self.cancels.get() > 0 {
            self.cancels.set(self.cancels.get() - 1);
            let cancelled = crate::workflow_exchange::request_cancellation(&loaded, loaded.sequence).unwrap();
            assert!(super::Records::swap(&self.store, &loaded, &cancelled), "the racing Cancel lands");
        }
        Some(loaded)
    }

    fn swap(&self, current: &super::ExecutionRecord, next: &super::ExecutionRecord) -> bool {
        super::Records::swap(&self.store, current, next)
    }
}

/// A freshly admitted execution of a fixture, with its store.
fn admitted() -> (tempfile::TempDir, crate::store::Store, super::ExecutionRecord) {
    let (directory, store, _workflow, _contract, request, profile) =
        crate::workflow_exchange::tests::fixture("printf unrun");
    let route = &profile.routes[0];
    let handler_identity = crate::workflow_exchange::handler_profile_identity(&profile).unwrap();
    let admission = store
        .admit_workflow_exchange(request, route, route.workflow_revision.clone(), handler_identity)
        .unwrap();
    let crate::workflow_exchange::AdmissionResolution::Inserted(admitted) = admission else {
        panic!("fresh fixture must admit once");
    };
    (directory, store, admitted)
}

// r[verify lattice.application_checkpoint.cancellation]
/// A first Cancel fails the worker's `Queued` to `Running` compare-and-swap, and a second lands
/// between its reload and its retry. The worker keeps reloading and still marks it running, so it
/// settles `cancelled` instead of leaving the execution for `Unknown`.
#[test]
fn cancels_racing_the_start_retry_still_settle_cancelled() {
    let (_directory, store, admitted) = admitted();
    let requested = crate::workflow_exchange::request_cancellation(&admitted, admitted.sequence).unwrap();
    store.save_workflow_exchange_transition(&admitted, &requested).unwrap();
    let racing = CancelAfterLoad {
        store: store.clone(),
        cancels: std::cell::Cell::new(1),
    };
    let running = super::start_running(&racing, &admitted);
    assert_eq!(
        running.as_ref().map(|running| running.state),
        Some(crate::workflow_exchange::DurableExecutionState::Running),
        "a start retry that loses to a second Cancel reloads again"
    );
    let running = running.unwrap();
    assert!(running.cancellation_requested);
    super::settle(&store, &running, &super::CompletionFacts {
        cancelled: true,
        ..super::CompletionFacts::default()
    });
    let durable = store.load_workflow_exchange(&admitted.accepted.execution_identity).unwrap().unwrap();
    assert_eq!(durable.state, crate::workflow_exchange::DurableExecutionState::Cancelled);
}

// r[verify lattice.application_checkpoint.cancellation]
/// A Cancel lands between `settle`'s load and its compare-and-swap. The reloaded record still
/// asks for cancellation, so the cancelled facts are recorded against it instead of `Unknown`.
#[test]
fn a_cancel_racing_settle_still_settles_cancelled() {
    let (_directory, store, admitted) = admitted();
    let running = crate::workflow_exchange::mark_running(&admitted).unwrap();
    store.save_workflow_exchange_transition(&admitted, &running).unwrap();
    let racing = CancelAfterLoad {
        store: store.clone(),
        cancels: std::cell::Cell::new(1),
    };
    super::settle(&racing, &running, &super::CompletionFacts {
        cancelled: true,
        ..super::CompletionFacts::default()
    });
    let durable = store.load_workflow_exchange(&admitted.accepted.execution_identity).unwrap().unwrap();
    assert_eq!(
        durable.state,
        crate::workflow_exchange::DurableExecutionState::Cancelled,
        "a settle that loses to a Cancel retries against the reloaded record"
    );
    assert!(durable.cancellation_requested);
}

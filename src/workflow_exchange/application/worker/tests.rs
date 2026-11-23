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

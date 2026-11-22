//! The standalone CLI's operator repair ports over the real run, recover,
//! inspect and export shell: a scripted fake OMP executable through the OMP
//! launcher's argv, process, capture and JSONL observation path, and catalog
//! checks of a pinned Nix store shell through the candidate worker's
//! generic route. Neither runs under bubblewrap here, and no model runs.

// r[verify lattice.implementation_attempt.operator_repair]
// r[verify lattice.implementation_attempt.operator_repair.scenario.repaired]
// r[verify lattice.implementation_attempt.operator_repair.scenario.authority]
// r[verify lattice.implementation_attempt.operator_repair.scenario.outcomes]
// r[verify lattice.implementation_attempt.operator_repair.scenario.checks]
// r[verify lattice.implementation_attempt.operator_repair.scenario.recovery]

mod harness;

use harness::Behaviour;
use harness::Operated;
use harness::Options;
use harness::Profile;

type Reason = super::super::super::refusal::Reason;

#[tokio::test]
async fn a_granted_operator_repairs_after_one_failing_check_and_exports_it() {
    let operated = Operated::new("attempt-operator-repaired", 2, 60_000);
    operated.script(&[Behaviour::Write("still broken"), Behaviour::Write("fixed")]);
    operated.run().await.expect("the loop reaches a disposition");
    let observation = operated.disposition();
    assert_eq!(observation["disposition"]["kind"], "repaired_candidate", "{observation}");
    assert_eq!(observation["disposition"]["repairs"], 1);
    assert_eq!((operated.launches(), operated.check_runs()), (2, 2));
    let records = operated.directory().join("repair");
    for iteration in [1, 2] {
        let step = harness::json(&records.join(format!("omp-step-{iteration}.json")));
        assert_eq!(step["worker_outcome"], serde_json::json!({"kind": "exited", "code": 0}), "{step}");
        assert_eq!(step["launcher_outcome"], "produced");
        assert_eq!(step["observation"]["terminal"], "completed");
        assert!(records.join(format!("omp-step-{iteration}.jsonl")).is_file());
    }
    // The repair runs from the parent revision with the admitted classes.
    let second = harness::json(&records.join("omp-step-2.json"));
    let first = harness::json(&records.join("omp-step-1.json"));
    assert_ne!(second["base_identity"], first["base_identity"], "the repair starts from its parent revision");
    assert_ne!(second["prompt"], first["prompt"]);
    // Recovery and a duplicate run report the retained disposition only.
    super::recover(&operated.path, &operated.fixture.state).await.expect("recovery");
    operated.run().await.expect("a duplicate run reconciles");
    assert_eq!((operated.launches(), operated.check_runs()), (2, 2), "nothing runs again");
    super::export::Seen::Terminal.assert(super::export::inventory(&operated.path, &operated.fixture.state), "repaired");
    let parent = super::export::output_parent(&operated.fixture.root, "exported");
    super::export::export(&operated.path, &operated.fixture.state, &parent).expect("the repaired candidate exports");
    let exported = parent.join(format!("{}-7", operated.request.envelope.attempt_id));
    assert_eq!(std::fs::read(exported.join("src/lib.txt")).expect("exported source"), b"fixed");
    assert!(exported.join("host-receipt.json").is_file());
    // A workspace changed after the run no longer re-derives the receipt.
    std::fs::write(operated.fixture.workspace.join("src/lib.txt"), "drifted").expect("drift");
    let parent = super::export::output_parent(&operated.fixture.root, "drifted");
    assert!(super::export::export(&operated.path, &operated.fixture.state, &parent).is_err());
}

/// Every repair step's broker forwards from the step's own index range, so
/// one host gateway serves both steps of a VM-bound repair attempt; a
/// reused index is refused before any upstream call.
// r[verify lattice.vm_profile.forwarded_upstream.scenario.request_index]
#[cfg(unix)]
#[test]
fn forwarded_request_indices_continue_across_repair_steps_through_one_gateway() {
    use crate::model_gateway::tests as gateway;
    let operated = Operated::new("attempt-operator-forwarded", 2, 60_000);
    operated.script(&[Behaviour::Write("still broken"), Behaviour::Write("fixed")]);
    let directory = tempfile::tempdir().expect("gateway scratch");
    let authority = gateway::Authority::start(directory.path(), "authority.sock", gateway::admitting("host-a", 0, 4));
    let upstream = gateway::Upstream::start(gateway::response_body());
    let socket = directory.path().join("vsock.sock_10243");
    let listener = crate::model_gateway::shell::listen(&socket).expect("gateway listener");
    let stopped = std::sync::atomic::AtomicBool::new(false);
    // The guest broker of each step, with the request's own per-step ceiling.
    let max_requests = operated.request.omp.model_broker.max_requests;
    let profile = crate::candidate_worker::broker::tests::forwarding_profile(
        gateway::MODEL,
        max_requests,
        gateway::binding_identity(),
        crate::vm_profile::VsockPort(10243),
    );
    let responses = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let (seen, forwarding, gateway_socket) = (responses.clone(), profile.clone(), socket.clone());
    operated.on_launch(Box::new(move |first_request_index| {
        let response =
            crate::candidate_worker::broker::tests::forward_one(&forwarding, &gateway_socket, first_request_index);
        seen.lock().expect("responses").push(response);
    }));
    let mut server = gateway::gateway(&authority, &upstream, &stopped, &gateway::fixed_clock, std::process::id());
    let reused = std::thread::scope(|scope| {
        let serving = scope.spawn(|| server.serve(&listener));
        let ran = harness::block_on(operated.run());
        // A reused index is refused by the gateway before any upstream call.
        let reused = crate::candidate_worker::broker::tests::forward_one(&profile, &socket, 0);
        stopped.store(true, std::sync::atomic::Ordering::Release);
        serving.join().expect("gateway join").expect("gateway serve");
        ran.expect("the loop reaches a disposition");
        reused
    });
    assert_eq!(operated.disposition()["disposition"]["kind"], "repaired_candidate");
    assert_eq!(operated.indices(), [0, max_requests], "each step starts after the previous step's range");
    let responses = responses.lock().expect("responses").clone();
    assert_eq!(responses.len(), 2);
    for response in &responses {
        assert!(response.starts_with("HTTP/1.1 200"), "{response}");
    }
    assert!(reused.starts_with("HTTP/1.1 403"), "{reused}");
    assert_eq!(upstream.calls(), 2, "both steps reached the upstream through one gateway; the replay did not");
    let admitted: Vec<u32> = authority
        .seen()
        .iter()
        .filter_map(|request| match request {
            crate::model_gateway::core::AuthorityRequest::Admit { request_index, .. } => Some(*request_index),
            crate::model_gateway::core::AuthorityRequest::Settle { .. } => None,
        })
        .collect();
    assert_eq!(admitted, [0, max_requests]);
}

#[tokio::test]
async fn repairs_stop_at_the_iteration_bound() {
    let operated = Operated::new("attempt-operator-exhausted", 1, 60_000);
    operated.script(&[
        Behaviour::Write("broken 1"),
        Behaviour::Write("broken 2"),
        Behaviour::Write("fixed"),
    ]);
    operated.run().await.expect("the loop reaches a disposition");
    assert_eq!(
        operated.disposition()["disposition"],
        serde_json::json!({"kind": "budget_exhausted", "bound": {"kind": "iterations"}})
    );
    assert_eq!(operated.launches(), 2, "the first step plus one repair");
}

#[tokio::test]
async fn a_crash_and_a_signal_death_in_the_sandbox_are_exits_and_neither_is_retried_blindly() {
    let crashed = Operated::new("attempt-operator-crashed", 2, 60_000);
    crashed.script(&[Behaviour::Exit(3)]);
    crashed.run().await.expect("the loop reaches a disposition");
    let observation = crashed.disposition();
    // The policy does not admit `no_candidate`, so nothing is repaired.
    assert_eq!(
        observation["disposition"],
        serde_json::json!({"kind": "not_repairable", "classes": ["no_candidate"]})
    );
    let step = harness::json(&crashed.directory().join("repair/omp-step-1.json"));
    assert_eq!(step["worker_outcome"], serde_json::json!({"kind": "exited", "code": 3}));
    assert_eq!((crashed.launches(), crashed.check_runs()), (1, 0));
    super::export::Seen::Terminal.assert(super::export::inventory(&crashed.path, &crashed.fixture.state), "exit");
    // The fake reports a killed child as 128 + 9, as bubblewrap reports a
    // signal death inside the sandbox: an observed exit.
    let killed = Operated::new("attempt-operator-killed", 2, 60_000);
    killed.script(&[Behaviour::Killed]);
    killed.run().await.expect("the loop reaches a disposition");
    let observation = killed.disposition();
    assert_eq!(
        observation["disposition"],
        serde_json::json!({"kind": "not_repairable", "classes": ["no_candidate"]})
    );
    let step = harness::json(&killed.directory().join("repair/omp-step-1.json"));
    assert_eq!(step["worker_outcome"], serde_json::json!({"kind": "exited", "code": 137}), "{step}");
    assert_eq!((killed.launches(), killed.check_runs()), (1, 0), "nothing is retried or checked");
    super::export::Seen::Terminal.assert(super::export::inventory(&killed.path, &killed.fixture.state), "signal");
}

#[tokio::test]
async fn a_worker_that_runs_to_its_deadline_exhausts_the_elapsed_bound() {
    let operated = Operated::new("attempt-operator-timeout", 2, 2_000);
    operated.script(&[Behaviour::Spin]);
    operated.run().await.expect("the loop reaches a disposition");
    assert_eq!(
        operated.disposition()["disposition"],
        serde_json::json!({"kind": "budget_exhausted", "bound": {"kind": "elapsed"}})
    );
    let step = harness::json(&operated.directory().join("repair/omp-step-1.json"));
    assert_eq!(step["launcher_outcome"], "failed");
    assert_eq!(step["worker_outcome"], serde_json::json!({"kind": "exited", "code": 137}), "{step}");
    assert_eq!((operated.launches(), operated.check_runs()), (1, 0), "an overrunning step is never checked");
}

#[tokio::test]
async fn a_persisted_cancellation_stops_the_running_worker() {
    let operated = Operated::new("attempt-operator-cancelled", 2, 60_000);
    operated.script(&[Behaviour::Spin]);
    let canceller = operated.cancel_after_intent();
    operated.run().await.expect("the loop reaches a disposition");
    canceller.join().expect("canceller");
    assert_eq!(operated.disposition()["disposition"]["kind"], "cancelled");
    let step = harness::json(&operated.directory().join("repair/omp-step-1.json"));
    assert_eq!(step["worker_outcome"], serde_json::json!({"kind": "cancelled"}));
    super::export::Seen::Terminal.assert(super::export::inventory(&operated.path, &operated.fixture.state), "cancel");
}

#[tokio::test]
async fn without_an_exact_grant_or_catalog_nothing_is_launched_and_only_the_closure_marker_is_written() {
    let cases = [
        (Profile::Unconfigured, Reason::RepairAuthorityDenied),
        (Profile::WorkerOnly, Reason::RepairAuthorityDenied),
        (Profile::OtherRevision, Reason::RepairAuthorityDenied),
        (Profile::Expired, Reason::RepairAuthorityDenied),
        (Profile::NoCatalog, Reason::RepairPortsUnavailable),
    ];
    for (index, (profile, reason)) in cases.into_iter().enumerate() {
        let operated = Operated::with_profile(&format!("attempt-operator-refused-{index}"), profile);
        operated.script(&[Behaviour::Write("fixed")]);
        assert_eq!(operated.run().await, Err(reason.message().to_owned()), "{profile:?}");
        assert_eq!((operated.launches(), operated.check_runs()), (0, 0), "{profile:?}");
        super::assert_closed(&operated.directory());
        assert!(!operated.directory().join("repair").exists(), "no step intent");
        // A re-run of the same request only reports its closed attempt-fence.
        operated.run().await.expect("the closure is reconciled");
        assert_eq!(operated.launches(), 0, "{profile:?}");
        super::assert_closed(&operated.directory());
    }
}

#[tokio::test]
async fn a_newer_fence_stops_the_operator_loop_before_its_next_step_intent() {
    let operated = Operated::new("attempt-operator-superseded", 2, 60_000);
    operated.script(&[Behaviour::Write("still broken"), Behaviour::Write("fixed")]);
    let state = operated.fixture.state.clone();
    let mut successor = operated.request.envelope.clone();
    successor.attempt_id = "attempt-operator-successor".into();
    successor.fence += 1;
    operated.after_launch(Box::new(move || {
        let directory = state.join("successor");
        drop(super::super::super::session::fencing::admit(&state, &successor, &directory).expect("fence 8"));
    }));
    operated.run().await.expect("the loop reaches a disposition");
    assert_eq!(operated.disposition()["disposition"], serde_json::json!({"kind": "fence_superseded"}));
    assert_eq!(operated.launches(), 1, "the repair never launched");
    assert!(!operated.directory().join("repair/step-2.intent").exists());
}

#[tokio::test]
async fn a_profile_withdrawn_during_the_run_ends_it_before_the_next_intent() {
    let operated = Operated::new("attempt-operator-withdrawn", 2, 60_000);
    operated.script(&[Behaviour::Write("still broken"), Behaviour::Write("fixed")]);
    let rewrite = operated.profile_writer(Profile::WorkerOnly);
    // Withdrawn after step 1's check and before step 2's intent.
    operated.after_check(Box::new(rewrite));
    operated.run().await.expect("the loop reaches a disposition");
    let observation = operated.disposition();
    assert_eq!(
        observation["disposition"],
        serde_json::json!({"kind": "authority_withdrawn", "cause": "effect_authority"})
    );
    assert_eq!(observation["authority_withdrawn"]["iteration"], 2);
    assert_eq!(operated.launches(), 1);
    super::export::Seen::Terminal
        .assert(super::export::inventory(&operated.path, &operated.fixture.state), "withdrawn");
}

/// A launcher that panics after its OMP step stands in for a controller
/// lost after the step intent.
#[test]
fn a_controller_lost_after_a_step_intent_recovers_without_a_second_launch() {
    let runtime = || tokio::runtime::Builder::new_current_thread().enable_all().build().expect("runtime");
    let operated = Operated::new("attempt-operator-lost", 2, 60_000);
    operated.script(&[Behaviour::Write("fixed")]);
    operated.lose_controller();
    let lost = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| runtime().block_on(operated.run())));
    assert!(lost.is_err(), "the controller was lost during the step");
    let directory = operated.directory();
    assert!(directory.join("repair/step-1.intent").exists());
    assert!(!directory.join("repair/step-1.json").exists() && !directory.join("repair/disposition.json").exists());
    let error = runtime().block_on(super::recover(&operated.path, &operated.fixture.state)).expect_err("unknown");
    assert!(error.contains("outcome unknown"), "{error}");
    assert!(runtime().block_on(operated.run()).is_err(), "a duplicate run reports the unknown outcome");
    assert_eq!(operated.launches(), 1, "never launched twice");
    super::export::Seen::Unknown.assert(super::export::inventory(&operated.path, &operated.fixture.state), "lost");
}

#[test]
fn the_catalog_is_admitted_only_pinned_bounded_and_outside_the_workspace() {
    let operated = Operated::new("attempt-operator-catalog", 2, 60_000);
    let policy = operated.request.repair.clone().expect("policy");
    let workspace = &operated.fixture.workspace;
    let load = |value: &serde_json::Value| {
        let path = operated.fixture.root.join("candidate-catalog.json");
        std::fs::write(&path, serde_json::to_vec(value).expect("bytes")).expect("catalog");
        super::super::super::repair::operator::checks::load(&path, workspace, &policy)
    };
    type Error = crate::implementation_attempt::repair::checks::catalog::Error;
    let admitted = load(&operated.catalog).expect("the pinned catalog");
    assert_eq!(admitted.executor(), policy.check_executor);
    let mut unpinned = operated.catalog.clone();
    unpinned["checks"][0]["program"]["path"] = "/usr/bin/true".into();
    assert_eq!(load(&unpinned).map(|_| ()), Err(Error::Unpinned));
    let mut mismatched = operated.catalog.clone();
    mismatched["checks"][0]["program"]["blake3"] = "0".repeat(64).into();
    // The cheap policy match runs before any executable is hashed.
    assert_eq!(load(&mismatched).map(|_| ()), Err(Error::Unmatched));
    let path = operated.fixture.root.join("mismatched-catalog.json");
    std::fs::write(&path, serde_json::to_vec(&mismatched).expect("bytes")).expect("catalog");
    let named = harness::policy(&mismatched);
    let loaded = super::super::super::repair::operator::checks::load(&path, workspace, &named);
    assert_eq!(loaded.map(|_| ()), Err(Error::Mismatched), "pinned identities, other bytes");
    let mut oversized = operated.catalog.clone();
    oversized["checks"][0]["args"] =
        serde_json::json!(vec!["x".repeat(crate::implementation_attempt::repair::checks::catalog::MAX_ARG_BYTES); 20]);
    assert_eq!(load(&oversized).map(|_| ()), Err(Error::Oversized));
    let inside = workspace.join("catalog.json");
    std::fs::write(&inside, serde_json::to_vec(&operated.catalog).expect("bytes")).expect("catalog in workspace");
    let loaded = super::super::super::repair::operator::checks::load(&inside, workspace, &policy);
    assert_eq!(loaded.map(|_| ()), Err(Error::Workspace), "never read from the worker's workspace");
    // Through `run`, a catalog with a wrong pin is refused before any launch.
    let refused = Operated::with_catalog("attempt-operator-mismatched", mismatched);
    assert_eq!(harness::block_on(refused.run()), Err(Reason::RepairPortsUnavailable.message().to_owned()));
    assert_eq!(refused.launches(), 0);
    super::assert_closed(&refused.directory());
}

#[tokio::test]
// r[verify lattice.implementation_attempt.operator_repair.scenario.absent]
async fn without_a_repair_policy_the_operator_ports_change_nothing() {
    let plain = harness::single_shot("attempt-operator-plain");
    let without = plain.run_without_operator().await;
    let with_operator = harness::single_shot("attempt-operator-plain");
    let with = with_operator.run().await;
    assert_eq!(with, without, "the same single-shot result");
    assert_eq!(harness::tree(&plain.directory()), harness::tree(&with_operator.directory()));
    assert!(!with_operator.directory().join("repair").exists());
    assert_eq!(with_operator.launches(), 0, "the single-shot route never uses the operator launcher");
}

/// The run is checked, unchecked or uncertain; nothing is exported.
fn assert_unexported(operated: &Operated, name: &str) {
    let parent = super::export::output_parent(&operated.fixture.root, name);
    assert!(super::export::export(&operated.path, &operated.fixture.state, &parent).is_err(), "{name}");
    assert!(super::export::is_empty(&parent), "{name}");
}

#[tokio::test]
async fn a_grant_withdrawn_during_a_worker_runs_none_of_its_checks() {
    let operated = Operated::new("attempt-operator-unchecked", 2, 60_000);
    operated.script(&[Behaviour::Write("fixed")]);
    let rewrite = operated.profile_writer(Profile::WorkerOnly);
    operated.after_launch(Box::new(rewrite));
    operated.run().await.expect("the loop reaches a disposition");
    let observation = operated.disposition();
    // A lapse before a revision's checks is a withdrawal, like one before a step.
    assert_eq!(
        observation["disposition"],
        serde_json::json!({"kind": "authority_withdrawn", "cause": "effect_authority"})
    );
    assert_eq!(observation["authority_withdrawn"]["iteration"], 1);
    assert_eq!(observation["authority_withdrawn"]["evidence"]["authority_refusal"], "not_granted");
    assert_eq!((operated.launches(), operated.check_runs()), (1, 0), "no check runs without the grant");
    super::export::Seen::Terminal
        .assert(super::export::inventory(&operated.path, &operated.fixture.state), "unchecked");
    assert_unexported(&operated, "unchecked");
}

#[tokio::test]
async fn an_admission_that_expires_before_the_checks_runs_none_of_them() {
    let operated = Operated::with("attempt-operator-expired", Options {
        expires_in_ms: Some(4_000),
        ..Options::default()
    });
    operated.script(&[Behaviour::Write("fixed")]);
    // The worker returns before the admission expires; the expiry passes
    // before its candidate's checks.
    operated.after_launch(Box::new(|| std::thread::sleep(std::time::Duration::from_millis(4_500))));
    operated.run().await.expect("the loop reaches a disposition");
    let observation = operated.disposition();
    assert_eq!(
        observation["disposition"],
        serde_json::json!({"kind": "authority_withdrawn", "cause": "effect_authority"})
    );
    assert_eq!(observation["authority_withdrawn"]["evidence"]["admission_expired"], true);
    assert_eq!(operated.check_runs(), 0, "no check runs after the admission expired");
    assert_unexported(&operated, "expired");
}

#[tokio::test]
async fn a_grant_withdrawn_between_checks_leaves_the_later_checks_unrun() {
    let operated = Operated::with("attempt-operator-between", Options {
        second_check: Some(harness::CHECK),
        ..Options::default()
    });
    operated.script(&[Behaviour::Write("fixed")]);
    let rewrite = operated.profile_writer(Profile::WorkerOnly);
    operated.after_check(Box::new(rewrite));
    operated.run().await.expect("the loop reaches a disposition");
    let observation = operated.disposition();
    assert_eq!(observation["disposition"], serde_json::json!({"kind": "uncertain", "cause": "check_outcome"}));
    assert_eq!(operated.check_runs(), 1, "the second check never runs");
    assert_unexported(&operated, "between");
}

#[tokio::test]
async fn a_cancellation_during_the_checks_is_cancelled() {
    let operated = Operated::new("attempt-operator-cancelled-checks", 2, 60_000);
    operated.script(&[Behaviour::Write("fixed")]);
    let (path, state) = (operated.path.clone(), operated.fixture.state.clone());
    operated.after_check(Box::new(move || {
        crate::implementation_attempt::cli::cancellation::request(&path, &state).expect("cancel");
    }));
    operated.run().await.expect("the loop reaches a disposition");
    assert_eq!(operated.disposition()["disposition"], serde_json::json!({"kind": "cancelled"}));
    super::export::Seen::Terminal
        .assert(super::export::inventory(&operated.path, &operated.fixture.state), "cancelled checks");
    let parent = super::export::output_parent(&operated.fixture.root, "cancelled-checks");
    let exported = super::export::export(&operated.path, &operated.fixture.state, &parent);
    assert_eq!(exported, super::export::refused(Reason::RepairNotCandidate), "a valid, non-passing disposition");
    assert!(super::export::is_empty(&parent));
}

#[tokio::test]
async fn a_worker_stopped_at_a_deadline_the_admission_capped_is_uncertain() {
    // A late start: the admission expires long before the elapsed bound.
    let operated = Operated::with("attempt-operator-late", Options {
        expires_in_ms: Some(3_000),
        ..Options::default()
    });
    operated.script(&[Behaviour::Spin]);
    operated.run().await.expect("the loop reaches a disposition");
    let observation = operated.disposition();
    assert_eq!(observation["disposition"], serde_json::json!({"kind": "uncertain", "cause": "worker_outcome"}));
    let step = harness::json(&operated.directory().join("repair/omp-step-1.json"));
    assert_eq!(step["deadline_capped"], true, "{step}");
    assert_eq!(step["worker_outcome"], serde_json::json!({"kind": "unknown"}), "{step}");
    assert_eq!(operated.check_runs(), 0, "the partial tree is never checked");
}

#[tokio::test]
async fn a_workspace_changed_between_steps_blocks_the_next_launch() {
    let operated = Operated::new("attempt-operator-drift", 2, 60_000);
    operated.script(&[Behaviour::Write("still broken"), Behaviour::Write("fixed")]);
    let workspace = operated.fixture.workspace.clone();
    operated.before_repair(Box::new(move || {
        std::fs::write(workspace.join("src/lib.txt"), "edited outside the attempt").expect("outside edit");
    }));
    operated.run().await.expect("the loop reaches a disposition");
    let observation = operated.disposition();
    assert_eq!(observation["disposition"], serde_json::json!({"kind": "uncertain", "cause": "worker_outcome"}));
    let step = harness::json(&operated.directory().join("repair/omp-step-2.json"));
    assert_eq!(step["launcher_outcome"], "blocked", "{step}");
    let count = std::fs::read_to_string(operated.fixture.root.join("omp-count")).expect("counter");
    assert_eq!(count.trim(), "1", "the fake OMP never ran on the changed workspace");
    assert_unexported(&operated, "drift");
}

#[tokio::test]
async fn checks_that_cannot_run_or_die_by_a_signal_are_unknown_not_failed() {
    for (index, check) in ["/bin/sh -c 'kill -9 $$'; exit $?", "exit 127"].into_iter().enumerate() {
        let operated = Operated::with(&format!("attempt-operator-infrastructure-{index}"), Options {
            check,
            ..Options::default()
        });
        operated.script(&[Behaviour::Write("fixed")]);
        operated.run().await.expect("the loop reaches a disposition");
        let observation = operated.disposition();
        assert_eq!(
            observation["disposition"],
            serde_json::json!({"kind": "uncertain", "cause": "check_outcome"}),
            "{check}"
        );
        assert_eq!((operated.launches(), operated.check_runs()), (1, 1), "no repair is spent on it: {check}");
    }
}

#[tokio::test]
async fn nested_paths_missing_from_the_copy_do_not_stop_the_checks() {
    let operated = Operated::with("attempt-operator-nested", Options {
        paths: Some((vec!["out", "src"], vec!["out/reports", "src"])),
        ..Options::default()
    });
    operated.script(&[Behaviour::Write("fixed")]);
    operated.run().await.expect("the loop reaches a disposition");
    assert_eq!(operated.disposition()["disposition"]["kind"], "correct_candidate");
    assert_eq!(operated.check_runs(), 1);
}

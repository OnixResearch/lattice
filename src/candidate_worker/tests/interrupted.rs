use super::*;
use crate::agent_node::ProcessFailurePhase as Phase;
use crate::agent_node::ProcessTermination as Termination;

// r[verify lattice.candidate_worker.process_observation.scenario.interrupted]
#[test]
fn actual_child_candidate_survives_injected_capture_and_teardown_failures() {
    for (phase, termination) in [
        (Phase::Capture, Termination::Observed),
        (Phase::Teardown, Termination::Observed),
        (Phase::Teardown, Termination::Unknown),
    ] {
        let directory = tempfile::tempdir().expect("workspace");
        let root = directory.path();
        std::fs::create_dir(root.join("src")).expect("source directory");
        let request = Request {
            work_id: "work".into(),
            attempt_id: "attempt".into(),
            fence: "1".into(),
            workspace_id: "workspace".into(),
            program: "sh".into(),
            args: vec![
                "-c".into(),
                "printf partial-event; printf candidate > src/marker.rs".into(),
            ],
            boundary: crate::candidate_capture::admit_workspace_boundary(
                root,
                &["src".into()],
                &["src".into()],
                false,
                false,
            )
            .expect("boundary"),
            require_isolation: false,
            sandbox_program: None,
            broker: None,
            timeout_secs: 5,
            output_limit_bytes: 4096,
        };
        let rejected = super::super::execution::run(
            &request,
            &crate::process::core::RuntimeConfig::default(),
            &std::sync::atomic::AtomicBool::new(false),
            Some("wrong-base"),
            |_, _, _| panic!("crossed base must not dispatch"),
        )
        .expect("pre-spawn base admission");
        assert!(matches!(rejected, Outcome::Blocked { .. }));
        assert!(!root.join("src/marker.rs").exists());
        let outcome = super::super::execution::run(
            &request,
            &crate::process::core::RuntimeConfig::default(),
            &std::sync::atomic::AtomicBool::new(false),
            None,
            |process, config, cancellation| {
                let observed = crate::process::shell::run_bounded_clear_environment_observed_with_cancellation(
                    process,
                    config,
                    Some(cancellation),
                )
                .expect("controlled child completes");
                assert_eq!(std::fs::read(root.join("src/marker.rs")).expect("independent marker"), b"candidate");
                let injected = match (phase, termination) {
                    (Phase::Capture, _) => bounded_exec::RunError::WorkerPanicked(bounded_exec::Worker::Stdout),
                    (Phase::Teardown, Termination::Observed) => {
                        bounded_exec::RunError::WorkerTeardownTimedOut(bounded_exec::Worker::Stdout)
                    }
                    _ => bounded_exec::RunError::ChildTeardownTimedOut,
                };
                Ok(crate::process::shell::Observation {
                    output: observed.output,
                    stop: Some(crate::process::shell::map_execution_error(injected)),
                })
            },
        )
        .expect("worker observation");
        let Outcome::Interrupted {
            observed_files,
            failure,
            stdout,
            ..
        } = &outcome
        else {
            panic!("post-start error must not claim no-start: {outcome:?}");
        };
        assert_eq!(failure.termination, termination);
        assert_eq!(observed_files.as_ref().expect("partial capture")["src/marker.rs"], b"candidate");
        assert_eq!(stdout, b"partial-event");
        let result = AttemptResult::from_outcome(&outcome);
        assert_eq!(result.disposition, crate::candidate::CandidateDisposition::Unknown);
        assert!(result.receipt.is_none());
        assert_eq!(result.process_failure, Some(*failure));
        assert_eq!(result.stdout_blake3, crate::pure_inet_profile::blake3_identity(b"partial-event"));
        assert_eq!(AttemptResult::from_json(&result.to_json().expect("serialize")).expect("parse"), result);
        let store_dir = tempfile::tempdir().expect("store directory");
        let store = crate::store::Store::new(store_dir.path().join("state.redb")).expect("store");
        let published = publish_outcome(&request, &store, outcome, None);
        assert!(matches!(published, Outcome::Interrupted { .. }));
        assert_eq!(store.load_candidate_attempt_result("attempt").expect("durable observation"), Some(result));
        assert_eq!(
            store
                .load_candidate_artifact(crate::store::ArtifactKey {
                    attempt_id: "attempt",
                    relative_path: "src/marker.rs"
                })
                .expect("durable artifact"),
            Some(b"candidate".to_vec())
        );
        assert!(matches!(reconcile_attempt(&store, "attempt").expect("reconcile"), Recovery::Incomplete { .. }));
        assert_eq!(std::fs::read(root.join("src/marker.rs")).expect("retained candidate"), b"candidate");
    }
}

// r[verify lattice.implementation_attempt.operator_repair.scenario.outcomes]

use crate::candidate_worker::Outcome;
use crate::implementation_attempt::repair::step::WorkerOutcome;

/// A failed launch with one of the launcher's own blocker texts.
fn failed(blocker: &str) -> Outcome {
    Outcome::Failed {
        blocker: blocker.into(),
        stdout: Vec::new(),
        stderr: Vec::new(),
    }
}

#[test]
fn only_observed_stops_are_exits_or_cancellation_and_everything_else_is_unknown() {
    let host_stop = WorkerOutcome::Exited {
        code: super::HOST_STOP_STATUS,
    };
    let observed = [
        (failed("worker exited with status Some(3)"), WorkerOutcome::Exited { code: 3 }),
        (failed("worker exceeded its admitted deadline; termination observed"), host_stop),
        (failed("worker exceeded its admitted output limit; termination observed"), host_stop),
        (
            Outcome::Cancelled {
                stdout: Vec::new(),
                stderr: Vec::new(),
            },
            WorkerOutcome::Cancelled,
        ),
    ];
    for (outcome, expected) in observed {
        assert_eq!(super::outcome(&outcome), expected, "{outcome:?}");
    }
    let unknown = [
        failed("worker exited with status None"),
        failed("worker changed a path outside its admitted write boundary"),
        failed("an unrecognized failure"),
        Outcome::Blocked {
            blocker: "pin mismatch".into(),
        },
        Outcome::Unknown {
            blocker: "incomplete OMP terminal".into(),
            stdout: Vec::new(),
            stderr: Vec::new(),
        },
        Outcome::Interrupted {
            failure: crate::candidate_worker::ProcessFailure {
                phase: crate::agent_node::ProcessFailurePhase::Teardown,
                termination: crate::agent_node::ProcessTermination::Unknown,
            },
            observed_files: None,
            stdout: Vec::new(),
            stderr: Vec::new(),
        },
    ];
    for outcome in unknown {
        assert_eq!(super::outcome(&outcome), WorkerOutcome::Unknown, "{outcome:?}");
        assert_ne!(super::describe(&outcome).0, "produced");
    }
}

#[test]
fn deadlines_round_up_within_the_admission_bound() {
    let seconds = |millis| super::timeout_secs(std::time::Duration::from_millis(millis), 60);
    assert_eq!((seconds(1), seconds(1_000), seconds(1_001)), (1, 1, 2));
    assert_eq!(seconds(600_000), 60, "never past the admission's own bound");
    assert_eq!(super::timeout_secs(std::time::Duration::from_millis(5_000), 0), 0, "an expired admission");
}

// r[verify lattice.implementation_attempt.operator_repair.scenario.repaired]
#[test]
fn a_repair_prompt_carries_only_its_parent_and_admitted_classes() {
    let binding = crate::implementation_attempt::repair::StepBinding {
        envelope: "e".repeat(64),
        work_id: "work".into(),
        work_version: 1,
        task_spec_digest: "c".repeat(64),
        attempt_id: "attempt".into(),
        fence: 7,
    };
    let mut step = crate::implementation_attempt::repair::step::Request {
        binding,
        iteration: 1,
        phase: crate::implementation_attempt::repair::step::Phase::Edit,
        parent: None,
        failures: Vec::new(),
    };
    assert_eq!(super::prompt("make it", &step), "make it", "the first step keeps the admitted prompt");
    step.iteration = 2;
    step.phase = crate::implementation_attempt::repair::step::Phase::Repair;
    step.parent = Some("a".repeat(64));
    step.failures = vec!["lint".into(), "test".into()];
    let repair = super::prompt("make it", &step);
    assert!(repair.starts_with("make it\n\nRepair step 2"), "{repair}");
    assert!(repair.contains(&"a".repeat(64)) && repair.contains("lint, test"), "{repair}");
}

// r[verify lattice.implementation_attempt.operator_repair.scenario.authority]
#[test]
fn the_requirements_name_the_worker_profile_and_both_exact_tuples() {
    let profile = "f".repeat(64);
    let required = super::requirements(&profile);
    assert_eq!(required.workflow_revision, format!("blake3:{profile}"));
    assert_eq!(required.machine_identity, super::machine_identity());
    assert_ne!(super::requirements(&"e".repeat(64)).workflow_revision, required.workflow_revision);
    let contracts: Vec<_> =
        required.grants.iter().map(crate::engine::effect::standalone::core::Grant::contract_id).collect();
    assert_eq!(contracts, [Some("lattice-agent"), Some("lattice-process")]);
    assert_eq!(required.grants[0].node_id, super::WORKER_NODE);
    assert_eq!(required.grants[1].node_id, super::CHECK_NODE);
}

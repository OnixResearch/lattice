// r[verify lattice.implementation_attempt.operator_repair.scenario.outcomes]

use crate::candidate_worker::Outcome;
use crate::implementation_attempt::repair::step::WorkerOutcome;

const DEADLINE: &str = "worker exceeded its admitted deadline; termination observed";

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
        // Bubblewrap reports a signal death inside the sandbox as 128 + N.
        (failed("worker exited with status Some(137)"), WorkerOutcome::Exited { code: 137 }),
        (failed(DEADLINE), host_stop),
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
        assert_eq!(super::outcome(&outcome, false), expected, "{outcome:?}");
    }
    let unknown = [
        failed("worker exited with status None"),
        failed("worker changed a path outside its admitted write boundary"),
        failed("an unrecognized failure"),
        Outcome::Blocked {
            blocker: "workspace base differs from the admitted source snapshot".into(),
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
        assert_eq!(super::outcome(&outcome, false), WorkerOutcome::Unknown, "{outcome:?}");
        assert_ne!(super::describe(&outcome).0, "produced");
    }
}

#[test]
fn a_deadline_the_admission_capped_is_never_checkable() {
    assert_eq!(super::outcome(&failed(DEADLINE), true), WorkerOutcome::Unknown);
    let exited = failed("worker exited with status Some(1)");
    assert_eq!(super::outcome(&exited, true), WorkerOutcome::Exited { code: 1 }, "only the deadline stop");
    let second = std::time::Duration::from_millis(1_500);
    assert!(super::is_capped(second, 1) && !super::is_capped(second, 2) && !super::is_capped(second, 60));
}

#[test]
fn deadlines_round_up_within_the_admission_bound() {
    let seconds = |millis| super::timeout_secs(std::time::Duration::from_millis(millis), 60);
    assert_eq!((seconds(1), seconds(1_000), seconds(1_001)), (1, 1, 2));
    assert_eq!(seconds(600_000), 60, "never past the admission's own bound");
    assert_eq!(super::timeout_secs(std::time::Duration::from_millis(5_000), 0), 0, "an expired admission");
}

fn repair(parent: Option<String>, failures: &[&str]) -> crate::implementation_attempt::repair::step::Request {
    crate::implementation_attempt::repair::step::Request {
        binding: crate::implementation_attempt::repair::StepBinding {
            envelope: "e".repeat(64),
            work_id: "work".into(),
            work_version: 1,
            task_spec_digest: "c".repeat(64),
            attempt_id: "attempt".into(),
            fence: 7,
        },
        iteration: 2,
        phase: crate::implementation_attempt::repair::step::Phase::Repair,
        parent,
        failures: failures.iter().map(|class| (*class).to_owned()).collect(),
    }
}

// r[verify lattice.implementation_attempt.operator_repair.scenario.repaired]
#[test]
fn every_repair_prompt_carries_only_its_parent_and_what_the_previous_step_did() {
    let mut first = repair(None, &[]);
    first.iteration = 1;
    first.phase = crate::implementation_attempt::repair::step::Phase::Edit;
    assert_eq!(super::prompt("make it", &first), "make it", "the first step keeps the admitted prompt");
    let failed = super::prompt("make it", &repair(Some("a".repeat(64)), &["lint", "test"]));
    assert!(failed.starts_with("make it\n\nRepair step 2"), "{failed}");
    assert!(failed.contains(&"a".repeat(64)) && failed.contains("check classes lint, test"), "{failed}");
    assert!(!failed.contains("no new candidate"), "{failed}");
    // A repair after a step without a candidate still gets repair text,
    // and `no_candidate` is never called a check class.
    let parentless = super::prompt("make it", &repair(None, &["no_candidate"]));
    assert!(
        parentless.contains("Repair step 2") && parentless.contains("produced no new candidate"),
        "{parentless}"
    );
    assert!(!parentless.contains("check classes") && !parentless.contains("revision"), "{parentless}");
    let unchanged = super::prompt("make it", &repair(Some("b".repeat(64)), &["no_candidate"]));
    assert!(
        unchanged.contains(&"b".repeat(64)) && unchanged.contains("produced no new candidate"),
        "{unchanged}"
    );
    assert!(!unchanged.contains("check classes"), "{unchanged}");
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

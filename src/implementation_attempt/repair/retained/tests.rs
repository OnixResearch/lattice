// r[verify lattice.implementation_attempt.repair_export.scenario.inventory]

use super::super::state::Bound;
use super::super::state::Disposition;
use super::super::state::Uncertainty;
use super::super::step::WorkerOutcome;

fn step(outcome: WorkerOutcome) -> super::StepRecord {
    super::StepRecord {
        schema: super::STEP_SCHEMA.into(),
        step: super::super::step::Request {
            binding: super::super::StepBinding {
                envelope: "e".repeat(64),
                work_id: "work-stopped".into(),
                work_version: 1,
                task_spec_digest: "c".repeat(64),
                attempt_id: "attempt-stopped".into(),
                fence: 7,
            },
            iteration: 1,
            phase: super::super::step::Phase::Edit,
            parent: None,
            failures: Vec::new(),
        },
        report: super::super::step::Report {
            outcome,
            model_requests: None,
            tool_invocations: None,
            proposal: None,
        },
        candidate: None,
        checks: Vec::new(),
        copies: Vec::new(),
        verdict: None,
    }
}

const EXITED: WorkerOutcome = WorkerOutcome::Exited { code: 0 };

#[test]
fn only_observed_worker_stops_settle_a_disposition() {
    let exhausted = Disposition::BudgetExhausted {
        bound: Bound::Iterations,
    };
    let elapsed = Disposition::BudgetExhausted { bound: Bound::Elapsed };
    let clock = Disposition::Uncertain {
        cause: Uncertainty::Clock,
    };
    let checks = Disposition::Uncertain {
        cause: Uncertainty::CheckOutcome,
    };
    let worker = Disposition::Uncertain {
        cause: Uncertainty::WorkerOutcome,
    };
    let kiln = Disposition::KilnOwned {
        continuation: super::super::KilnOwned::NewSemanticAttempt,
    };
    let exited = [step(EXITED)];
    // Settled: every started worker exited or was cancelled.
    assert!(super::is_stopped(&exhausted, &exited, false));
    assert!(super::is_stopped(&checks, &exited, false), "the worker exited; only a check is unknown");
    assert!(super::is_stopped(&Disposition::Cancelled, &[step(WorkerOutcome::Cancelled)], false));
    assert!(super::is_stopped(&Disposition::FenceSuperseded, &[], false), "no worker started");
    assert!(super::is_stopped(&kiln, &exited, false));
    // An intent without a record whose worker the loop never called.
    assert!(super::is_stopped(&elapsed, &exited, true));
    assert!(super::is_stopped(&clock, &exited, true));
    // Unknown: an unknown worker outcome or a worker that may have run.
    assert!(!super::is_stopped(&worker, &[step(WorkerOutcome::Unknown)], false));
    assert!(!super::is_stopped(&worker, &exited, false), "the kind alone stays unknown");
    assert!(!super::is_stopped(&exhausted, &[step(EXITED), step(WorkerOutcome::Unknown)], false), "a record");
    assert!(!super::is_stopped(&kiln, &exited, true), "a refused step's worker ran unrecorded");
    assert!(!super::is_stopped(&exhausted, &exited, true));
}

#[test]
fn records_refuse_unknown_and_missing_fields() {
    let record = serde_json::to_value(step(EXITED)).expect("record");
    let parsed: super::StepRecord = serde_json::from_value(record.clone()).expect("a complete record");
    assert_eq!(parsed, step(EXITED));
    let mut unknown = record.clone();
    unknown["accepted"] = true.into();
    assert!(serde_json::from_value::<super::StepRecord>(unknown).is_err());
    for field in ["candidate", "verdict", "copies"] {
        let mut missing = record.clone();
        missing.as_object_mut().expect("object").remove(field);
        assert!(serde_json::from_value::<super::StepRecord>(missing).is_err(), "{field}");
    }
}

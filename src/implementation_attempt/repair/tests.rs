// r[verify lattice.implementation_attempt.repair]
// r[verify lattice.implementation_attempt.repair.scenario.policy]
// r[verify lattice.implementation_attempt.repair.scenario.dispositions]
// r[verify lattice.implementation_attempt.repair.scenario.bypass]
// r[verify lattice.implementation_attempt.repair.scenario.kiln_owned]
// r[verify lattice.implementation_attempt.lineage.scenario.usage]

use super::checks::Defect;
use super::checks::Observation;
use super::checks::Outcome;
use super::checks::Verdict;
use super::state::Bound;
use super::state::Disposition;
use super::state::Loop;
use super::state::Next;
use super::state::Uncertainty;
use super::step::Phase;
use super::step::Report as StepReport;
use super::step::WorkerOutcome;

const STEP_MODEL_REQUESTS: u64 = 2;
const WORKER_PROFILE: &str = "9999999999999999999999999999999999999999999999999999999999999999";

fn hex(digit: char) -> String {
    digit.to_string().repeat(64)
}

pub(crate) fn policy() -> super::Policy {
    super::Policy {
        schema: super::POLICY_SCHEMA.into(),
        failure_classes: vec!["format".into(), "test".into()],
        max_iterations: 2,
        max_elapsed_ms: 60_000,
        max_model_requests: 6,
        max_tool_invocations: 12,
        step_tool_invocations: 2,
        check_executor: hex('7'),
        checks: vec![
            super::Check {
                name: "fmt".into(),
                configuration: hex('a'),
            },
            super::Check {
                name: "unit".into(),
                configuration: hex('b'),
            },
        ],
    }
}

fn budget() -> crate::implementation_attempt::AttemptBudget {
    crate::implementation_attempt::AttemptBudget {
        max_turns: 3,
        max_seconds: 60,
        max_cost_micros: 0,
    }
}

fn frame(budget: &crate::implementation_attempt::AttemptBudget) -> super::Frame<'_> {
    super::Frame {
        budget,
        step_model_requests: STEP_MODEL_REQUESTS,
        is_unchanged_allowed: false,
    }
}

fn binding() -> super::StepBinding {
    super::StepBinding {
        envelope: hex('e'),
        work_id: "work-6".into(),
        work_version: 1,
        task_spec_digest: hex('c'),
        attempt_id: "attempt-6".into(),
        fence: 7,
    }
}

fn observation(check: &str, revision: &str, outcome: Outcome) -> Observation {
    let configuration = if check == "fmt" { hex('a') } else { hex('b') };
    Observation {
        check: check.into(),
        configuration,
        revision: revision.into(),
        executor: hex('7'),
        outcome,
    }
}

fn subject<'a>(policy: &'a super::Policy, revision: &'a str) -> super::checks::Subject<'a> {
    super::checks::Subject {
        policy,
        revision,
        worker_profile: WORKER_PROFILE,
    }
}

const fn exited() -> StepReport {
    StepReport {
        outcome: WorkerOutcome::Exited { code: 0 },
        model_requests: None,
        tool_invocations: None,
        proposal: None,
    }
}

fn spawn(state: &Loop, elapsed_ms: u64) -> super::step::Request {
    match state.next(Some(elapsed_ms), false) {
        Next::Spawn(request) => request,
        other => panic!("expected a spawn, got {other:?}"),
    }
}

/// One step that produces `revision` and is checked with `outcomes`.
fn step(state: &mut Loop, elapsed_ms: u64, revision: &str, outcomes: [Outcome; 2]) -> Next {
    let request = spawn(state, elapsed_ms);
    state.start(&request, &exited()).expect("same attempt");
    assert_eq!(state.after_step(&exited(), Some(revision.into()), Some(elapsed_ms)), Next::Check {
        revision: revision.into()
    });
    let [fmt, unit] = outcomes;
    let policy = policy();
    let verdict = super::checks::classify(
        subject(&policy, revision),
        &[observation("fmt", revision, fmt), observation("unit", revision, unit)],
        None,
    );
    let _ = state.charge_checks(request.iteration, 2);
    state.after_checks(verdict, Some(elapsed_ms))
}

fn fresh() -> Loop {
    Loop::new(policy(), binding(), None, STEP_MODEL_REQUESTS).expect("admitted policy")
}

#[test]
fn an_admitted_policy_validates_and_frames_a_distinct_revision() {
    let budget = budget();
    policy().validate(frame(&budget)).expect("admitted");
    let identity = policy().identity();
    assert_eq!(identity, policy().identity());
    let mut changed = policy();
    changed.max_iterations = 1;
    assert_ne!(changed.identity(), identity);
    let single_shot = hex('5');
    assert_ne!(super::profile_revision(&single_shot, &policy()), single_shot);
    assert_ne!(super::profile_revision(&single_shot, &policy()), super::profile_revision(&single_shot, &changed));
}

#[test]
fn malformed_or_unbounded_policies_are_refused() {
    let budget = budget();
    type Case = (fn(&mut super::Policy), super::PolicyError);
    let cases: [Case; 12] = [
        (|value| value.schema = "v0".into(), super::PolicyError::Schema),
        (|value| value.failure_classes.clear(), super::PolicyError::FailureClasses),
        (|value| value.failure_classes.reverse(), super::PolicyError::FailureClasses),
        (|value| value.failure_classes.push("Bad Class".into()), super::PolicyError::FailureClasses),
        (|value| value.checks.clear(), super::PolicyError::Checks),
        (|value| value.checks[0].configuration = "a".into(), super::PolicyError::Checks),
        (|value| value.check_executor = "x".into(), super::PolicyError::Executor),
        (|value| value.max_iterations = 0, super::PolicyError::Iterations),
        (|value| value.max_iterations = 3, super::PolicyError::Iterations),
        (|value| value.max_elapsed_ms = 60_001, super::PolicyError::Elapsed),
        (|value| value.max_model_requests = 1, super::PolicyError::Budget),
        (|value| value.step_tool_invocations = 0, super::PolicyError::Budget),
    ];
    for (change, expected) in cases {
        let mut changed = policy();
        change(&mut changed);
        assert_eq!(changed.validate(frame(&budget)), Err(expected), "{changed:?}");
    }
    let mut unchanged = frame(&budget);
    unchanged.is_unchanged_allowed = true;
    assert_eq!(policy().validate(unchanged), Err(super::PolicyError::Unchanged));
}

#[test]
fn checks_pass_only_on_complete_admitted_evidence_for_the_exact_revision() {
    let policy = policy();
    let revision = hex('d');
    let passed = [
        observation("fmt", &revision, Outcome::Passed),
        observation("unit", &revision, Outcome::Passed),
    ];
    assert_eq!(super::checks::classify(subject(&policy, &revision), &passed, None), Verdict::Passed);
    let failed = [
        observation("unit", &revision, Outcome::Failed { class: "test".into() }),
        observation("fmt", &revision, Outcome::Failed { class: "format".into() }),
    ];
    assert_eq!(super::checks::classify(subject(&policy, &revision), &failed, None), Verdict::Failed {
        classes: vec!["format".into(), "test".into()]
    });
    let unknown = [observation("fmt", &revision, Outcome::Unknown), failed[0].clone()];
    assert_eq!(super::checks::classify(subject(&policy, &revision), &unknown, None), Verdict::Unknown);
    let invocations = super::checks::invocations(&policy, &revision, 3);
    assert_eq!(invocations.len(), 2);
    assert!(invocations.iter().all(|invocation| invocation.revision == revision && invocation.iteration == 3));
}

#[test]
// r[verify lattice.implementation_attempt.repair.scenario.bypass]
fn skipped_missing_reconfigured_crossed_and_self_approved_checks_cannot_pass() {
    let policy = policy();
    let revision = hex('d');
    let pass = |check: &str| observation(check, &revision, Outcome::Passed);
    let mut reconfigured = pass("unit");
    reconfigured.configuration = hex('f');
    let mut crossed = pass("unit");
    crossed.revision = hex('0');
    let mut self_approved = pass("unit");
    self_approved.executor = WORKER_PROFILE.into();
    let mut foreign = pass("unit");
    foreign.executor = hex('8');
    let unit = || "unit".to_owned();
    let cases = [
        (vec![pass("fmt")], Defect::MissingCheck { check: unit() }),
        (vec![pass("fmt"), observation("unit", &revision, Outcome::Skipped)], Defect::SkippedCheck {
            check: unit(),
        }),
        (vec![pass("fmt"), pass("unit"), pass("lint")], Defect::UndeclaredCheck { check: "lint".into() }),
        (vec![pass("fmt"), pass("unit"), pass("unit")], Defect::DuplicateCheck { check: unit() }),
        (vec![pass("fmt"), reconfigured], Defect::ConfigurationChanged { check: unit() }),
        (vec![pass("fmt"), crossed], Defect::CrossedRevision { check: unit() }),
        (vec![pass("fmt"), self_approved], Defect::SelfApproval { check: unit() }),
        (vec![pass("fmt"), foreign], Defect::UnadmittedExecutor { check: unit() }),
        (Vec::new(), Defect::MissingCheck { check: "fmt".into() }),
    ];
    for (observations, defect) in cases {
        let verdict = super::checks::classify(subject(&policy, &revision), &observations, None);
        assert_eq!(verdict, Verdict::Rejected { defect });
    }
}

#[test]
// r[verify lattice.implementation_attempt.repair.scenario.bypass]
fn a_check_whose_candidate_copy_drifted_cannot_pass() {
    let policy = policy();
    let revision = hex('d');
    let passed = [
        observation("fmt", &revision, Outcome::Passed),
        observation("unit", &revision, Outcome::Passed),
    ];
    let drift = |check: &str| Verdict::Rejected {
        defect: Defect::CandidateDrift { check: check.into() },
    };
    // A formatter that rewrote its copy, followed by a passing unit check.
    assert_eq!(super::checks::classify(subject(&policy, &revision), &passed, Some("fmt")), drift("fmt"));
    // Drift outranks every other defect, including the checks it cut short.
    let cut_short = [observation("fmt", &revision, Outcome::Skipped)];
    assert_eq!(super::checks::classify(subject(&policy, &revision), &cut_short, Some("fmt")), drift("fmt"));
    assert_eq!(super::checks::classify(subject(&policy, &revision), &passed, None), Verdict::Passed);
    let mut state = fresh();
    let request = spawn(&state, 0);
    state.start(&request, &exited()).expect("same attempt");
    let _ = state.after_step(&exited(), Some(revision.clone()), Some(0));
    assert_eq!(
        state.after_checks(drift("fmt"), Some(1)),
        Next::Stop(Disposition::ChecksRejected {
            defect: Defect::CandidateDrift { check: "fmt".into() }
        })
    );
}

#[test]
// r[verify lattice.implementation_attempt.repair.scenario.dispositions]
fn a_first_candidate_that_passes_is_correct_and_a_repaired_one_is_distinct() {
    let mut correct = fresh();
    let first = hex('1');
    assert_eq!(
        step(&mut correct, 0, &first, [Outcome::Passed, Outcome::Passed]),
        Next::Stop(Disposition::CorrectCandidate { revision: first })
    );
    let mut repaired = fresh();
    let failing = [Outcome::Passed, Outcome::Failed { class: "test".into() }];
    assert_eq!(step(&mut repaired, 0, &hex('1'), failing), Next::Repair);
    let request = spawn(&repaired, 10);
    assert_eq!((request.phase, request.iteration), (Phase::Repair, 2));
    assert_eq!(request.parent.as_deref(), Some(hex('1').as_str()));
    assert_eq!(request.failures, vec!["test".to_owned()]);
    assert_eq!(
        step(&mut repaired, 10, &hex('2'), [Outcome::Passed, Outcome::Passed]),
        Next::Stop(Disposition::RepairedCandidate {
            revision: hex('2'),
            repairs: 1
        })
    );
}

#[test]
fn every_bound_exhausts_before_spawn_with_its_own_disposition() {
    let failing = || [Outcome::Passed, Outcome::Failed { class: "test".into() }];
    let mut iterations = fresh();
    for (elapsed_ms, digit) in [(0, '1'), (1, '2'), (2, '3')] {
        assert_eq!(step(&mut iterations, elapsed_ms, &hex(digit), failing()), Next::Repair);
    }
    let exhausted = |bound| Next::Stop(Disposition::BudgetExhausted { bound });
    assert_eq!(iterations.next(Some(3), false), exhausted(Bound::Iterations));
    let state = fresh();
    assert_eq!(state.next(Some(60_000), false), exhausted(Bound::Elapsed));
    let mut models = policy();
    models.max_model_requests = 3;
    let mut model_state = Loop::new(models, binding(), None, STEP_MODEL_REQUESTS).expect("policy");
    assert_eq!(step(&mut model_state, 0, &hex('1'), failing()), Next::Repair);
    assert_eq!(model_state.next(Some(1), false), exhausted(Bound::ModelRequests));
    let mut tools = policy();
    tools.max_tool_invocations = 5;
    let mut tool_state = Loop::new(tools, binding(), None, STEP_MODEL_REQUESTS).expect("policy");
    assert_eq!(step(&mut tool_state, 0, &hex('1'), failing()), Next::Repair);
    assert_eq!(tool_state.next(Some(1), false), exhausted(Bound::ToolInvocations));
}

#[test]
// r[verify lattice.implementation_attempt.lineage.scenario.exhausted]
fn the_lineage_bounds_every_iteration_and_its_blockers_stop_the_loop() {
    let failing = [Outcome::Passed, Outcome::Failed { class: "test".into() }];
    let lineage = crate::implementation_attempt::lineage::tests::binding(3, 100);
    let mut state = Loop::new(policy(), binding(), Some(lineage), STEP_MODEL_REQUESTS).expect("policy");
    assert_eq!(step(&mut state, 0, &hex('1'), failing), Next::Repair);
    assert_eq!(
        state.next(Some(1), false),
        Next::Stop(Disposition::BudgetExhausted {
            bound: Bound::Lineage {
                dimension: crate::implementation_attempt::lineage::MODEL_REQUESTS.into()
            }
        })
    );
    let mut blocked = crate::implementation_attempt::lineage::tests::binding(100, 100);
    blocked.backpressure = crate::implementation_attempt::lineage::Backpressure::Overloaded;
    let state = Loop::new(policy(), binding(), Some(blocked), STEP_MODEL_REQUESTS).expect("policy");
    assert_eq!(
        state.next(Some(0), false),
        Next::Stop(Disposition::LineageBlocked {
            refusal: crate::implementation_attempt::lineage::Refusal::Backpressure {
                gate: crate::implementation_attempt::lineage::Backpressure::Overloaded
            }
        })
    );
}

#[test]
fn cancellation_uncertainty_and_unadmitted_failures_are_distinct_stops() {
    assert_eq!(fresh().next(Some(0), true), Next::Stop(Disposition::Cancelled));
    assert_eq!(
        fresh().next(None, false),
        Next::Stop(Disposition::Uncertain {
            cause: Uncertainty::Clock
        })
    );
    for (outcome, expected) in [
        (WorkerOutcome::Cancelled, Disposition::Cancelled),
        (WorkerOutcome::Unknown, Disposition::Uncertain {
            cause: Uncertainty::WorkerOutcome,
        }),
    ] {
        let mut state = fresh();
        let request = spawn(&state, 0);
        let report = StepReport { outcome, ..exited() };
        state.start(&request, &report).expect("same attempt");
        assert_eq!(state.after_step(&report, Some(hex('1')), Some(0)), Next::Stop(expected));
    }
    let mut unknown = fresh();
    assert_eq!(
        step(&mut unknown, 0, &hex('1'), [Outcome::Unknown, Outcome::Passed]),
        Next::Stop(Disposition::Uncertain {
            cause: Uncertainty::CheckOutcome
        })
    );
    let mut unadmitted = fresh();
    assert_eq!(
        step(&mut unadmitted, 0, &hex('1'), [Outcome::Passed, Outcome::Failed {
            class: "security".into()
        }]),
        Next::Stop(Disposition::NotRepairable {
            classes: vec!["security".into()]
        })
    );
    let mut skipped = fresh();
    assert!(matches!(
        step(&mut skipped, 0, &hex('1'), [Outcome::Passed, Outcome::Skipped]),
        Next::Stop(Disposition::ChecksRejected { .. })
    ));
}

#[test]
// r[verify lattice.implementation_attempt.repair.scenario.bypass]
fn an_exit_zero_step_without_a_candidate_is_a_failure_never_a_pass() {
    let mut state = fresh();
    let request = spawn(&state, 0);
    state.start(&request, &exited()).expect("same attempt");
    assert_eq!(
        state.after_step(&exited(), None, Some(0)),
        Next::Stop(Disposition::NotRepairable {
            classes: vec![super::NO_CANDIDATE.into()]
        })
    );
    let mut admitted = policy();
    admitted.failure_classes = vec![super::NO_CANDIDATE.into(), "test".into()];
    let mut state = Loop::new(admitted, binding(), None, STEP_MODEL_REQUESTS).expect("policy");
    let request = spawn(&state, 0);
    state.start(&request, &exited()).expect("same attempt");
    assert_eq!(state.after_step(&exited(), None, Some(0)), Next::Repair);
    assert_eq!(spawn(&state, 1).failures, vec![super::NO_CANDIDATE.to_owned()]);
}

#[test]
// r[verify lattice.implementation_attempt.repair.scenario.kiln_owned]
fn changed_work_and_new_semantic_attempts_stay_with_kiln() {
    let frozen = binding();
    type Change = (fn(&mut super::StepBinding), super::KilnOwned);
    let changes: [Change; 6] = [
        (|value| value.work_id = "work-7".into(), super::KilnOwned::ChangedWork),
        (|value| value.work_version = 2, super::KilnOwned::ChangedWork),
        (|value| value.task_spec_digest = hex('0'), super::KilnOwned::ChangedWork),
        (|value| value.attempt_id = "attempt-7".into(), super::KilnOwned::NewSemanticAttempt),
        (|value| value.fence = 8, super::KilnOwned::NewSemanticAttempt),
        (|value| value.envelope = hex('0'), super::KilnOwned::NewSemanticAttempt),
    ];
    for (change, expected) in changes {
        let mut requested = frozen.clone();
        change(&mut requested);
        assert_eq!(frozen.admit_step(&requested), Err(expected));
        let mut state = fresh();
        let mut request = spawn(&state, 0);
        request.binding = requested;
        assert_eq!(state.start(&request, &exited()), Err(expected));
        assert_eq!(state.steps(), 0, "a refused step is never recorded");
    }
    let mut state = fresh();
    let request = spawn(&state, 0);
    let proposing = StepReport {
        proposal: Some(super::KilnOwned::NewSemanticAttempt),
        ..exited()
    };
    state.start(&request, &proposing).expect("same attempt");
    assert_eq!(
        state.after_step(&proposing, Some(hex('1')), Some(0)),
        Next::Stop(Disposition::KilnOwned {
            continuation: super::KilnOwned::NewSemanticAttempt
        })
    );
}

#[test]
// r[verify lattice.implementation_attempt.repair.scenario.kiln_owned]
fn an_unknown_or_cancelled_outcome_outranks_a_proposal() {
    let cases = [
        (WorkerOutcome::Unknown, Disposition::Uncertain {
            cause: Uncertainty::WorkerOutcome,
        }),
        (WorkerOutcome::Cancelled, Disposition::Cancelled),
        (WorkerOutcome::Exited { code: 0 }, Disposition::KilnOwned {
            continuation: super::KilnOwned::ChangedWork,
        }),
    ];
    for (outcome, expected) in cases {
        let mut state = fresh();
        let request = spawn(&state, 0);
        let report = StepReport {
            outcome,
            proposal: Some(super::KilnOwned::ChangedWork),
            ..exited()
        };
        state.start(&request, &report).expect("same attempt");
        assert_eq!(state.after_step(&report, Some(hex('1')), Some(0)), Next::Stop(expected), "{outcome:?}");
    }
}

/// A policy with an aggregate tool budget of six: one step of two tool
/// invocations and its two checks fit it.
fn six_tools() -> super::Policy {
    let mut six = policy();
    six.max_tool_invocations = 6;
    six
}

/// Start one step reporting `tools` tool invocations and decide after it.
fn reported(state: &mut Loop, tools: u64, elapsed_ms: Option<u64>) -> Next {
    let request = spawn(state, 0);
    let report = StepReport {
        model_requests: Some(STEP_MODEL_REQUESTS),
        tool_invocations: Some(tools),
        ..exited()
    };
    state.start(&request, &report).expect("same attempt");
    state.after_step(&report, Some(hex('1')), elapsed_ms)
}

#[test]
// r[verify lattice.implementation_attempt.repair.scenario.dispositions]
fn a_candidate_whose_charges_overran_a_budget_is_never_passed() {
    let exhausted = |bound| Next::Stop(Disposition::BudgetExhausted { bound });
    let checked = Next::Check { revision: hex('1') };
    // The first step reports nine tool invocations against a budget of six:
    // it is never checked, so its checks cannot pass it.
    let mut over = Loop::new(six_tools(), binding(), None, STEP_MODEL_REQUESTS).expect("policy");
    assert_eq!(reported(&mut over, 9, Some(0)), exhausted(super::state::Bound::ToolInvocations));
    // Five fit before the checks; the two check executions overrun six, so a
    // pass is still exhausted.
    let mut checks_over = Loop::new(six_tools(), binding(), None, STEP_MODEL_REQUESTS).expect("policy");
    assert_eq!(reported(&mut checks_over, 5, Some(0)), checked);
    let _ = checks_over.charge_checks(1, 2);
    assert_eq!(checks_over.after_checks(Verdict::Passed, Some(1)), exhausted(Bound::ToolInvocations));
    // Exactly six fits.
    let mut exact = Loop::new(six_tools(), binding(), None, STEP_MODEL_REQUESTS).expect("policy");
    assert_eq!(reported(&mut exact, 4, Some(0)), checked);
    let _ = exact.charge_checks(1, 2);
    assert_eq!(
        exact.after_checks(Verdict::Passed, Some(1)),
        Next::Stop(Disposition::CorrectCandidate { revision: hex('1') })
    );
    // A reported model overrun is exhausted the same way.
    let mut models = fresh();
    let request = spawn(&models, 0);
    let report = StepReport {
        model_requests: Some(7),
        ..exited()
    };
    models.start(&request, &report).expect("same attempt");
    assert_eq!(models.after_step(&report, Some(hex('1')), Some(0)), exhausted(Bound::ModelRequests));
}

#[test]
// r[verify lattice.implementation_attempt.lineage.scenario.exhausted]
fn a_candidate_whose_charges_overran_the_lineage_is_never_passed() {
    let tools = crate::implementation_attempt::lineage::TOOL_INVOCATIONS;
    let lineage = crate::implementation_attempt::lineage::tests::binding(100, 5);
    let mut over = Loop::new(policy(), binding(), Some(lineage.clone()), STEP_MODEL_REQUESTS).expect("policy");
    assert_eq!(reported(&mut over, 4, Some(0)), Next::Check { revision: hex('1') });
    let _ = over.charge_checks(1, 2);
    assert_eq!(
        over.after_checks(Verdict::Passed, Some(1)),
        Next::Stop(Disposition::BudgetExhausted {
            bound: Bound::Lineage {
                dimension: tools.into()
            }
        })
    );
    let mut fits = Loop::new(policy(), binding(), Some(lineage), STEP_MODEL_REQUESTS).expect("policy");
    assert_eq!(reported(&mut fits, 3, Some(0)), Next::Check { revision: hex('1') });
    let _ = fits.charge_checks(1, 2);
    assert_eq!(
        fits.after_checks(Verdict::Passed, Some(1)),
        Next::Stop(Disposition::CorrectCandidate { revision: hex('1') })
    );
}

#[test]
// r[verify lattice.implementation_attempt.repair.scenario.dispositions]
fn the_elapsed_bound_and_the_clock_hold_through_the_terminal_decision() {
    let state = fresh();
    assert_eq!(state.remaining_ms(Some(59_000)), Ok(1_000));
    assert_eq!(state.remaining_ms(Some(60_000)), Err(Disposition::BudgetExhausted { bound: Bound::Elapsed }));
    let clock = Disposition::Uncertain {
        cause: Uncertainty::Clock,
    };
    assert_eq!(state.remaining_ms(None), Err(clock.clone()));
    // A worker that returns past the bound, or without a readable clock, is
    // never checked.
    for (elapsed_ms, expected) in [
        (Some(60_000), Disposition::BudgetExhausted { bound: Bound::Elapsed }),
        (None, clock.clone()),
    ] {
        let mut state = fresh();
        assert_eq!(reported(&mut state, 2, elapsed_ms), Next::Stop(expected));
    }
    // Checks that pass past the bound, or without a readable clock, do not
    // make a candidate; just inside the bound they do.
    for (elapsed_ms, expected) in [
        (Some(60_001), Disposition::BudgetExhausted { bound: Bound::Elapsed }),
        (None, clock.clone()),
        (Some(59_999), Disposition::CorrectCandidate { revision: hex('1') }),
    ] {
        let mut state = fresh();
        assert_eq!(reported(&mut state, 2, Some(0)), Next::Check { revision: hex('1') });
        let _ = state.charge_checks(1, 2);
        assert_eq!(state.after_checks(Verdict::Passed, elapsed_ms), Next::Stop(expected));
    }
}

#[test]
// r[verify lattice.implementation_attempt.lineage.scenario.usage]
fn charges_are_conservative_and_unknown_usage_stays_unknown() {
    let mut state = fresh();
    let request = spawn(&state, 0);
    let unknown = state.start(&request, &exited()).expect("same attempt");
    assert_eq!((unknown.model_requests, unknown.tool_invocations), (None, None));
    assert_eq!(unknown.provenance, crate::orchestration::UsageProvenance::Unknown);
    assert_eq!((unknown.charged.model_requests, unknown.charged.tool_invocations), (2, 2));
    assert_eq!(unknown.phase, Phase::Edit);
    let _ = state.after_step(&exited(), Some(hex('1')), Some(0));
    let checks = state.charge_checks(1, 2);
    assert_eq!((checks.phase, checks.charged.tool_invocations, checks.charged.model_requests), (Phase::Check, 2, 0));
    let _ = state.after_checks(
        Verdict::Failed {
            classes: vec!["test".into()],
        },
        Some(1),
    );
    let repair = spawn(&state, 1);
    let over = StepReport {
        model_requests: Some(1),
        tool_invocations: Some(9),
        ..exited()
    };
    let reported = state.start(&repair, &over).expect("same attempt");
    assert_eq!(reported.phase, Phase::Repair);
    assert_eq!(reported.provenance, crate::orchestration::UsageProvenance::WorkerReported);
    assert_eq!(
        (reported.charged.model_requests, reported.charged.tool_invocations),
        (2, 9),
        "an under-report cannot lower the reservation; an overrun is charged"
    );
    assert_eq!((state.drawn().model_requests, state.drawn().tool_invocations), (4, 13));
}

// r[verify lattice.implementation_attempt.repair_export.scenario.passing]
// r[verify lattice.implementation_attempt.repair_export.scenario.refused]

use super::super::checks::Outcome;
use super::super::checks::Verdict;
use super::super::retained::StepRecord;
use super::super::revision::Record;
use super::super::state::Disposition;
use super::Unexportable;

mod contradicted;

const WORKER_PROFILE: &str = "9999999999999999999999999999999999999999999999999999999999999999";
const WORKSPACE: &str = "workspace-export";
const BASE: &[u8] = b"broken";

fn binding(attempt_id: &str, fence: u64) -> super::super::StepBinding {
    super::super::StepBinding {
        envelope: "e".repeat(64),
        work_id: "work-export".into(),
        work_version: 1,
        task_spec_digest: "c".repeat(64),
        attempt_id: attempt_id.into(),
        fence,
    }
}

fn tree(bytes: &[u8]) -> std::collections::BTreeMap<String, Vec<u8>> {
    std::collections::BTreeMap::from([("src/lib.txt".to_owned(), bytes.to_vec())])
}

/// The admitted facts of `bound`, with `policy`.
struct Admitted {
    binding: super::super::StepBinding,
    fence: String,
    base_identity: String,
    policy: super::super::Policy,
}

impl Admitted {
    fn new(bound: super::super::StepBinding, policy: super::super::Policy) -> Self {
        Self {
            fence: bound.fence.to_string(),
            base_identity: crate::candidate::snapshot_identity(&tree(BASE)).expect("base"),
            binding: bound,
            policy,
        }
    }

    fn bindings(&self) -> crate::candidate::Bindings<'_> {
        crate::candidate::Bindings {
            work_id: &self.binding.work_id,
            attempt_id: &self.binding.attempt_id,
            fence: &self.fence,
            workspace_id: WORKSPACE,
        }
    }

    fn frame(&self) -> super::Frame<'_> {
        super::Frame {
            binding: &self.binding,
            receipt: self.bindings(),
            base_identity: &self.base_identity,
            policy: &self.policy,
            worker_profile: WORKER_PROFILE,
        }
    }

    fn identity(&self, record: &Record) -> String {
        super::super::revision::identity(super::super::revision::Input {
            binding: &self.binding,
            iteration: record.iteration,
            parent: record.parent.as_deref(),
            candidate_identity: record.receipt.candidate_identity.as_deref().expect("candidate"),
            executables: &record.executables,
            diff_identity: record.receipt.diff_identity.as_deref().expect("diff"),
        })
    }

    /// The records of a run whose step `n` checks both declared checks with
    /// `outcomes[n]`, every copy intact, the run continuing on admitted
    /// failures as the loop would.
    fn run(&self, outcomes: &[Outcome]) -> (Vec<Record>, Vec<StepRecord>) {
        let mut revisions: Vec<Record> = Vec::new();
        let mut steps = Vec::new();
        let mut failures = Vec::new();
        for (iteration, outcome) in (1..).zip(outcomes) {
            let receipt = crate::candidate::produced_receipt_from_snapshots(
                self.bindings(),
                &tree(BASE),
                &tree(format!("candidate {iteration}").as_bytes()),
            )
            .expect("receipt");
            let parent = revisions.last().map(|parent| parent.revision.clone());
            let mut record = Record {
                iteration,
                revision: String::new(),
                parent: parent.clone(),
                receipt,
                executables: std::collections::BTreeMap::new(),
                base_files: std::collections::BTreeMap::from([(
                    "src/lib.txt".to_owned(),
                    Some(crate::candidate::digests::FileDigest::of(BASE)),
                )]),
            };
            record.revision = self.identity(&record);
            let checks: Vec<_> = self
                .policy
                .checks
                .iter()
                .map(|declared| super::super::checks::Observation {
                    check: declared.name.clone(),
                    configuration: declared.configuration.clone(),
                    revision: record.revision.clone(),
                    executor: self.policy.check_executor.clone(),
                    outcome: outcome.clone(),
                })
                .collect();
            let copies = checks
                .iter()
                .map(|observation| super::super::retained::CopyIntegrity {
                    check: observation.check.clone(),
                    is_intact: true,
                })
                .collect();
            let subject = super::super::checks::Subject {
                policy: &self.policy,
                revision: &record.revision,
                worker_profile: WORKER_PROFILE,
            };
            let verdict = super::super::checks::classify(subject, &checks, None);
            steps.push(StepRecord {
                schema: super::super::retained::STEP_SCHEMA.into(),
                step: super::super::step::Request {
                    binding: self.binding.clone(),
                    iteration,
                    phase: if iteration == 1 {
                        super::super::step::Phase::Edit
                    } else {
                        super::super::step::Phase::Repair
                    },
                    parent,
                    failures: std::mem::take(&mut failures),
                },
                report: super::super::step::Report {
                    outcome: super::super::step::WorkerOutcome::Exited { code: 0 },
                    model_requests: None,
                    tool_invocations: None,
                    proposal: None,
                },
                candidate: Some(record.clone()),
                checks,
                copies,
                verdict: Some(verdict.clone()),
            });
            if let Verdict::Failed { classes } = verdict {
                failures = classes;
            }
            revisions.push(record);
        }
        (revisions, steps)
    }
}

fn failed() -> Outcome {
    Outcome::Failed { class: "test".into() }
}

fn repaired(revisions: &[Record]) -> Disposition {
    Disposition::RepairedCandidate {
        revision: revisions.last().expect("revision").revision.clone(),
        repairs: u32::try_from(revisions.len()).expect("count") - 1,
    }
}

fn candidate<'a>(
    admitted: &Admitted,
    disposition: &'a Disposition,
    revisions: &'a [Record],
    steps: &'a [StepRecord],
) -> Result<&'a Record, Unexportable> {
    super::candidate(&admitted.frame(), super::Retained {
        disposition,
        revisions,
        steps,
    })
}

#[test]
fn a_passing_run_names_its_exact_last_revision() {
    let admitted = Admitted::new(binding("attempt-export", 7), super::super::tests::policy());
    let (revisions, steps) = admitted.run(&[Outcome::Passed]);
    let correct = Disposition::CorrectCandidate {
        revision: revisions[0].revision.clone(),
    };
    assert_eq!(candidate(&admitted, &correct, &revisions, &steps), Ok(&revisions[0]));
    let (revisions, steps) = admitted.run(&[failed(), failed(), Outcome::Passed]);
    let disposition = repaired(&revisions);
    assert_eq!(
        candidate(&admitted, &disposition, &revisions, &steps),
        Ok(&revisions[2]),
        "the policy's two repairs"
    );
}

#[test]
fn non_passing_dispositions_name_no_candidate() {
    let admitted = Admitted::new(binding("attempt-export", 7), super::super::tests::policy());
    let (revisions, steps) = admitted.run(&[failed()]);
    let stops = [
        Disposition::BudgetExhausted {
            bound: super::super::state::Bound::Iterations,
        },
        Disposition::Cancelled,
        Disposition::Uncertain {
            cause: super::super::state::Uncertainty::CheckOutcome,
        },
        Disposition::NotRepairable {
            classes: vec!["lint".into()],
        },
        Disposition::ChecksRejected {
            defect: super::super::checks::Defect::MissingCheck { check: "unit".into() },
        },
        Disposition::KilnOwned {
            continuation: super::super::KilnOwned::ChangedWork,
        },
        Disposition::FenceSuperseded,
        Disposition::AuthorityWithdrawn {
            cause: super::super::state::Authority::ManagedRevision,
        },
    ];
    for stop in &stops {
        assert_eq!(candidate(&admitted, stop, &revisions, &steps), Err(Unexportable::NotCandidate), "{stop:?}");
    }
}

#[test]
fn a_failed_or_drifted_check_cannot_be_relabelled_as_a_pass() {
    let admitted = Admitted::new(binding("attempt-export", 7), super::super::tests::policy());
    // A failure outside the admitted classes ended the run as not_repairable.
    let (revisions, mut steps) = admitted.run(&[Outcome::Failed { class: "lint".into() }]);
    let forged = Disposition::CorrectCandidate {
        revision: revisions[0].revision.clone(),
    };
    assert_eq!(candidate(&admitted, &forged, &revisions, &steps), Err(Unexportable::Contradicted));
    steps[0].verdict = Some(Verdict::Passed);
    assert_eq!(candidate(&admitted, &forged, &revisions, &steps), Err(Unexportable::Contradicted), "verdict field");
    let (revisions, clean) = admitted.run(&[Outcome::Passed]);
    let correct = Disposition::CorrectCandidate {
        revision: revisions[0].revision.clone(),
    };
    let mut drifted = clean.clone();
    drifted[0].copies[1].is_intact = false;
    assert_eq!(candidate(&admitted, &correct, &revisions, &drifted), Err(Unexportable::Contradicted), "drift");
    let mut uncopied = clean.clone();
    uncopied[0].copies.pop();
    assert_eq!(candidate(&admitted, &correct, &revisions, &uncopied), Err(Unexportable::Contradicted), "no copy");
    let mut missing = clean.clone();
    missing[0].checks.pop();
    missing[0].copies.pop();
    assert_eq!(candidate(&admitted, &correct, &revisions, &missing), Err(Unexportable::Contradicted), "a check");
    let mut unknown = clean.clone();
    unknown[0].report.outcome = super::super::step::WorkerOutcome::Unknown;
    assert_eq!(candidate(&admitted, &correct, &revisions, &unknown), Err(Unexportable::Contradicted), "worker");
    assert_eq!(candidate(&admitted, &correct, &revisions, &clean), Ok(&revisions[0]));
}

#[test]
fn iterations_beyond_the_admitted_policy_name_no_candidate() {
    let mut narrow = super::super::tests::policy();
    narrow.max_iterations = 1;
    let admitted = Admitted::new(binding("attempt-export", 7), narrow);
    let (revisions, steps) = admitted.run(&[failed(), Outcome::Passed]);
    assert_eq!(candidate(&admitted, &repaired(&revisions), &revisions, &steps), Ok(&revisions[1]));
    let (revisions, steps) = admitted.run(&[failed(), failed(), Outcome::Passed]);
    let over = repaired(&revisions);
    assert_eq!(candidate(&admitted, &over, &revisions, &steps), Err(Unexportable::Contradicted), "policy bound");
    let mut wide = super::super::tests::policy();
    wide.max_iterations = 20;
    let admitted = Admitted::new(binding("attempt-export", 7), wide);
    let outcomes: Vec<_> = (0..9).map(|_| failed()).chain([Outcome::Passed]).collect();
    let (revisions, steps) = admitted.run(&outcomes);
    let over = repaired(&revisions);
    assert_eq!(candidate(&admitted, &over, &revisions, &steps), Err(Unexportable::Contradicted), "global bound");
}

//! Classification of one candidate revision's check observations.
//!
//! Only observations of every declared check, run by the admitted executor
//! with the admitted configuration against this exact revision, can pass a
//! candidate. A skipped, missing, duplicated, undeclared, reconfigured,
//! crossed or self-approved observation rejects the evidence instead.

// r[impl lattice.implementation_attempt.repair.scenario.bypass]

/// What the host check port asks for: one declared check of one revision.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Invocation {
    pub check: String,
    /// The admitted configuration identity the check must run with.
    pub configuration: String,
    pub revision: String,
    pub iteration: u32,
}

/// The outcome the check executor observed.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum Outcome {
    Passed,
    Failed {
        class: String,
    },
    Skipped,
    /// The check's own outcome is unknown, for example after a lost executor.
    Unknown,
}

/// One check observation as the host check port reports it.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Observation {
    pub check: String,
    /// The configuration identity the check actually ran with.
    pub configuration: String,
    /// The candidate revision the check actually ran against.
    pub revision: String,
    /// The identity of the executor that ran the check.
    pub executor: String,
    pub outcome: Outcome,
}

/// Why check evidence cannot decide a candidate.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case", tag = "code")]
pub enum Defect {
    MissingCheck {
        check: String,
    },
    SkippedCheck {
        check: String,
    },
    UndeclaredCheck {
        check: String,
    },
    DuplicateCheck {
        check: String,
    },
    ConfigurationChanged {
        check: String,
    },
    CrossedRevision {
        check: String,
    },
    /// Run by the implementing worker profile itself.
    SelfApproval {
        check: String,
    },
    /// Run by an executor other than the admitted one.
    UnadmittedExecutor {
        check: String,
    },
}

/// The classification of one revision's checks.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum Verdict {
    /// Every declared check passed.
    Passed,
    /// Valid evidence with at least one failure, sorted unique classes.
    Failed { classes: Vec<String> },
    /// Valid evidence with at least one unknown outcome.
    Unknown,
    /// Evidence that cannot decide the candidate.
    Rejected { defect: Defect },
}

/// The facts one revision's evidence is classified against.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Subject<'a> {
    pub policy: &'a super::Policy,
    pub revision: &'a str,
    /// The implementing worker's profile revision.
    pub worker_profile: &'a str,
}

/// The invocations of every declared check of `revision`.
#[must_use]
pub fn invocations(policy: &super::Policy, revision: &str, iteration: u32) -> Vec<Invocation> {
    policy
        .checks
        .iter()
        .map(|check| Invocation {
            check: check.name.clone(),
            configuration: check.configuration.clone(),
            revision: revision.to_owned(),
            iteration,
        })
        .collect()
}

/// Classify `observations` of `subject`. Defects take precedence over
/// unknown outcomes, and unknown outcomes over failures.
#[must_use]
pub fn classify(subject: Subject<'_>, observations: &[Observation]) -> Verdict {
    if let Err(defect) = admit_evidence(subject, observations) {
        return Verdict::Rejected { defect };
    }
    if observations.iter().any(|observation| observation.outcome == Outcome::Unknown) {
        return Verdict::Unknown;
    }
    debug_assert_eq!(observations.len(), subject.policy.checks.len(), "admitted evidence covers every check once");
    let mut classes: Vec<String> = observations
        .iter()
        .filter_map(|observation| match &observation.outcome {
            Outcome::Failed { class } => Some(class.clone()),
            Outcome::Passed | Outcome::Skipped | Outcome::Unknown => None,
        })
        .collect();
    if classes.is_empty() {
        return Verdict::Passed;
    }
    classes.sort();
    classes.dedup();
    debug_assert!(!classes.is_empty(), "a failed verdict names its classes");
    Verdict::Failed { classes }
}

fn admit_evidence(subject: Subject<'_>, observations: &[Observation]) -> Result<(), Defect> {
    debug_assert!(!subject.revision.is_empty(), "evidence is classified for one revision");
    debug_assert!(subject.policy.checks.len() <= super::MAX_CHECKS, "an admitted policy is bounded");
    for (index, observation) in observations.iter().enumerate() {
        let check = observation.check.clone();
        let declared = subject.policy.checks.iter().find(|declared| declared.name == observation.check);
        let Some(declared) = declared else {
            return Err(Defect::UndeclaredCheck { check });
        };
        if observations[..index].iter().any(|earlier| earlier.check == observation.check) {
            return Err(Defect::DuplicateCheck { check });
        }
        if observation.executor == subject.worker_profile {
            return Err(Defect::SelfApproval { check });
        }
        if observation.executor != subject.policy.check_executor {
            return Err(Defect::UnadmittedExecutor { check });
        }
        if observation.revision != subject.revision {
            return Err(Defect::CrossedRevision { check });
        }
        if observation.configuration != declared.configuration {
            return Err(Defect::ConfigurationChanged { check });
        }
        if observation.outcome == Outcome::Skipped {
            return Err(Defect::SkippedCheck { check });
        }
    }
    let missing = subject
        .policy
        .checks
        .iter()
        .find(|declared| !observations.iter().any(|observation| observation.check == declared.name));
    match missing {
        Some(declared) => Err(Defect::MissingCheck {
            check: declared.name.clone(),
        }),
        None => Ok(()),
    }
}

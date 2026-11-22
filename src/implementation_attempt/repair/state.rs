//! The bounded repair loop of one attempt as a pure state machine.
//!
//! The shell asks [`Loop::next`] before every worker spawn, reports each
//! step with [`Loop::after_step`] and each revision's checks with
//! [`Loop::after_checks`]. Every step and check execution is charged
//! conservatively against the policy's aggregate budgets and, when present,
//! the one Kiln lineage account: never a fresh budget per iteration.

// r[impl lattice.implementation_attempt.repair.scenario.dispositions]
// r[impl lattice.implementation_attempt.repair.scenario.kiln_owned]
// r[impl lattice.implementation_attempt.lineage.scenario.usage]

mod bounds;
mod charges;

/// The bound a stop exhausted.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum Bound {
    Iterations,
    Elapsed,
    ModelRequests,
    ToolInvocations,
    Lineage { dimension: String },
}

/// What made an outcome uncertain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Uncertainty {
    WorkerOutcome,
    CheckOutcome,
    Clock,
}

/// The revocable authority, besides the fence, a later repair step needs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Authority {
    /// The execution binding failed its revalidation.
    ExecutionBinding,
    /// The managed agent revision no longer permits an effect, or its
    /// lifecycle could not be read.
    ManagedRevision,
    /// The standalone operator's effect-authority profile no longer grants
    /// both the OMP worker and the check tuples with its admitted identity,
    /// or the admission expired.
    EffectAuthority,
}

/// The terminal disposition of a repair loop. None of them is acceptance.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum Disposition {
    /// The first candidate passed every declared check.
    CorrectCandidate {
        revision: String,
    },
    /// A repaired candidate passed every declared check.
    RepairedCandidate {
        revision: String,
        repairs: u32,
    },
    BudgetExhausted {
        bound: Bound,
    },
    Cancelled,
    Uncertain {
        cause: Uncertainty,
    },
    /// A failure outside the admitted classes.
    NotRepairable {
        classes: Vec<String>,
    },
    /// The check evidence itself could not decide the candidate.
    ChecksRejected {
        defect: super::checks::Defect,
    },
    /// Kiln reports a lineage blocker or serving backpressure.
    LineageBlocked {
        refusal: super::super::lineage::Refusal,
    },
    /// The worker proposed a continuation only Kiln may admit.
    KilnOwned {
        continuation: super::KilnOwned,
    },
    /// The shell found a higher fence of the work, or another attempt at
    /// this fence, recorded in its state root before the next step's intent;
    /// that step never spawned. A post-dispatch outcome: the attempt's intent
    /// exists and earlier steps may have had effects.
    FenceSuperseded,
    /// The shell found the execution binding or the managed revision no
    /// longer current before the next step's intent; that step never
    /// spawned. A post-dispatch outcome like [`Self::FenceSuperseded`].
    AuthorityWithdrawn {
        cause: Authority,
    },
}

/// The loop's next action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Next {
    Spawn(super::step::Request),
    Check {
        revision: String,
    },
    /// Admitted failures are recorded; ask [`Loop::next`] for the repair
    /// step, which applies every bound first.
    Repair,
    Stop(Disposition),
}

/// The bounded loop state of one attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Loop {
    policy: super::Policy,
    binding: super::StepBinding,
    lineage: Option<super::super::lineage::Binding>,
    reservation: super::super::lineage::Draws,
    steps: u32,
    drawn: super::super::lineage::Draws,
    last_revision: Option<String>,
    failures: Vec<String>,
}

impl Loop {
    /// A loop over an admitted policy; `step_model_requests` is the broker
    /// ceiling of one worker step.
    pub fn new(
        policy: super::Policy,
        binding: super::StepBinding,
        lineage: Option<super::super::lineage::Binding>,
        step_model_requests: u64,
    ) -> Result<Self, super::PolicyError> {
        let reservation = policy.reservation(step_model_requests)?;
        Ok(Self {
            policy,
            binding,
            lineage,
            reservation,
            steps: 0,
            drawn: super::super::lineage::Draws::default(),
            last_revision: None,
            failures: Vec::new(),
        })
    }

    /// Whether another step may spawn: cancellation, the clock, the
    /// iteration bound, the aggregate budgets and the lineage, in that order.
    #[must_use]
    pub fn next(&self, elapsed_ms: Option<u64>, is_cancel_requested: bool) -> Next {
        if is_cancel_requested {
            return Next::Stop(Disposition::Cancelled);
        }
        if let Err(disposition) = self.remaining_ms(elapsed_ms) {
            return Next::Stop(disposition);
        }
        if let Err(bound) = self.within_policy() {
            return Next::Stop(Disposition::BudgetExhausted { bound });
        }
        if let Some(lineage) = &self.lineage
            && let Err(refusal) = lineage.gate(&self.drawn, &self.reservation)
        {
            return Next::Stop(match refusal {
                super::super::lineage::Refusal::Exhausted { dimension } => Disposition::BudgetExhausted {
                    bound: Bound::Lineage { dimension },
                },
                other => Disposition::LineageBlocked { refusal: other },
            });
        }
        let iteration = self.steps.saturating_add(1);
        Next::Spawn(super::step::Request {
            binding: self.binding.clone(),
            iteration,
            phase: if self.steps == 0 {
                super::step::Phase::Edit
            } else {
                super::step::Phase::Repair
            },
            parent: self.last_revision.clone(),
            failures: self.failures.clone(),
        })
    }

    /// Decide after a recorded step. `revision` is the new host-derived
    /// candidate revision, `None` when the step produced none; `elapsed_ms`
    /// is re-observed after the worker returned. A cancelled or unknown
    /// outcome takes precedence over any proposal, and a step that overran
    /// a bound is never checked.
    #[must_use]
    pub fn after_step(
        &mut self,
        report: &super::step::Report,
        revision: Option<String>,
        elapsed_ms: Option<u64>,
    ) -> Next {
        match report.outcome {
            super::step::WorkerOutcome::Cancelled => return Next::Stop(Disposition::Cancelled),
            super::step::WorkerOutcome::Unknown => {
                return Next::Stop(Disposition::Uncertain {
                    cause: Uncertainty::WorkerOutcome,
                });
            }
            super::step::WorkerOutcome::Exited { .. } => {}
        }
        if let Some(continuation) = report.proposal {
            return Next::Stop(Disposition::KilnOwned { continuation });
        }
        if let Err(disposition) = self.settled(elapsed_ms) {
            return Next::Stop(disposition);
        }
        match revision {
            Some(revision) => {
                self.last_revision = Some(revision.clone());
                Next::Check { revision }
            }
            None => self.failed(vec![super::NO_CANDIDATE.to_owned()]),
        }
    }

    /// Decide after the checks of the last revision. `elapsed_ms` is
    /// re-observed after the last check: a pass becomes a candidate only
    /// while the clock is read, the elapsed bound is not reached and the
    /// charged totals fit every policy budget and the lineage remainder.
    #[must_use]
    pub fn after_checks(&mut self, verdict: super::checks::Verdict, elapsed_ms: Option<u64>) -> Next {
        let revision = self.last_revision.clone().unwrap_or_default();
        debug_assert!(!revision.is_empty(), "checks follow a recorded revision");
        match verdict {
            super::checks::Verdict::Passed => {
                if let Err(disposition) = self.settled(elapsed_ms) {
                    return Next::Stop(disposition);
                }
                if self.steps <= 1 {
                    return Next::Stop(Disposition::CorrectCandidate { revision });
                }
                Next::Stop(Disposition::RepairedCandidate {
                    revision,
                    repairs: self.steps.saturating_sub(1),
                })
            }
            super::checks::Verdict::Unknown => Next::Stop(Disposition::Uncertain {
                cause: Uncertainty::CheckOutcome,
            }),
            super::checks::Verdict::Rejected { defect } => Next::Stop(Disposition::ChecksRejected { defect }),
            super::checks::Verdict::Failed { classes } => self.failed(classes),
        }
    }

    fn failed(&mut self, classes: Vec<String>) -> Next {
        if !self.policy.admits(&classes) {
            return Next::Stop(Disposition::NotRepairable { classes });
        }
        self.failures = classes;
        Next::Repair
    }

    /// Everything charged so far.
    #[must_use]
    pub const fn drawn(&self) -> &super::super::lineage::Draws {
        &self.drawn
    }

    /// Worker steps started so far.
    #[must_use]
    pub const fn steps(&self) -> u32 {
        self.steps
    }

    /// The last recorded candidate revision.
    #[must_use]
    pub fn last_revision(&self) -> Option<&str> {
        self.last_revision.as_deref()
    }
}

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
        let Some(elapsed_ms) = elapsed_ms else {
            return Next::Stop(Disposition::Uncertain {
                cause: Uncertainty::Clock,
            });
        };
        if let Err(bound) = self.within_policy(elapsed_ms) {
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

    fn within_policy(&self, elapsed_ms: u64) -> Result<(), Bound> {
        if elapsed_ms >= self.policy.max_elapsed_ms {
            return Err(Bound::Elapsed);
        }
        if self.steps > self.policy.max_iterations {
            return Err(Bound::Iterations);
        }
        let after = self.drawn.saturating_add(&self.reservation);
        if after.model_requests > self.policy.max_model_requests {
            return Err(Bound::ModelRequests);
        }
        if after.tool_invocations > self.policy.max_tool_invocations {
            return Err(Bound::ToolInvocations);
        }
        Ok(())
    }

    /// Record a spawned step and charge it: the larger of the reservation
    /// and the reported count, or the reservation when the count is unknown.
    pub fn start(
        &mut self,
        request: &super::step::Request,
        report: &super::step::Report,
    ) -> Result<super::step::Usage, super::KilnOwned> {
        self.binding.admit_step(&request.binding)?;
        self.steps = self.steps.saturating_add(1);
        debug_assert_eq!(request.iteration, self.steps, "steps are recorded in order");
        let step_tools = self.policy.step_tool_invocations;
        let charged = super::super::lineage::Draws {
            model_requests: report
                .model_requests
                .map_or(self.reservation.model_requests, |reported| reported.max(self.reservation.model_requests)),
            tool_invocations: report.tool_invocations.map_or(step_tools, |reported| reported.max(step_tools)),
        };
        self.drawn = self.drawn.saturating_add(&charged);
        let is_reported = report.model_requests.is_some() && report.tool_invocations.is_some();
        Ok(super::step::Usage {
            phase: request.phase,
            iteration: request.iteration,
            model_requests: report.model_requests,
            tool_invocations: report.tool_invocations,
            provenance: if is_reported {
                crate::orchestration::UsageProvenance::WorkerReported
            } else {
                crate::orchestration::UsageProvenance::Unknown
            },
            charged,
        })
    }

    /// Decide after a recorded step. `revision` is the new host-derived
    /// candidate revision, `None` when the step produced none.
    #[must_use]
    pub fn after_step(&mut self, report: &super::step::Report, revision: Option<String>) -> Next {
        if let Some(continuation) = report.proposal {
            return Next::Stop(Disposition::KilnOwned { continuation });
        }
        match report.outcome {
            super::step::WorkerOutcome::Cancelled => return Next::Stop(Disposition::Cancelled),
            super::step::WorkerOutcome::Unknown => {
                return Next::Stop(Disposition::Uncertain {
                    cause: Uncertainty::WorkerOutcome,
                });
            }
            super::step::WorkerOutcome::Exited { .. } => {}
        }
        match revision {
            Some(revision) => {
                self.last_revision = Some(revision.clone());
                Next::Check { revision }
            }
            None => self.failed(vec![super::NO_CANDIDATE.to_owned()]),
        }
    }

    /// Charge the host check executions of one revision.
    pub fn charge_checks(&mut self, iteration: u32, executions: u64) -> super::step::Usage {
        let charged = super::super::lineage::Draws {
            model_requests: 0,
            tool_invocations: executions,
        };
        self.drawn = self.drawn.saturating_add(&charged);
        super::step::Usage {
            phase: super::step::Phase::Check,
            iteration,
            model_requests: Some(0),
            tool_invocations: Some(executions),
            provenance: crate::orchestration::UsageProvenance::HostMeasured,
            charged,
        }
    }

    /// Decide after the checks of the last revision.
    #[must_use]
    pub fn after_checks(&mut self, verdict: super::checks::Verdict) -> Next {
        let revision = self.last_revision.clone().unwrap_or_default();
        debug_assert!(!revision.is_empty(), "checks follow a recorded revision");
        match verdict {
            super::checks::Verdict::Passed if self.steps <= 1 => Next::Stop(Disposition::CorrectCandidate { revision }),
            super::checks::Verdict::Passed => Next::Stop(Disposition::RepairedCandidate {
                revision,
                repairs: self.steps.saturating_sub(1),
            }),
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

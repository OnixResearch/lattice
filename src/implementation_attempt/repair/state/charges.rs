//! Conservative charges of one repair loop's steps and checks.

// r[impl lattice.implementation_attempt.lineage.scenario.usage]

impl super::Loop {
    /// Record a spawned step and charge it: the larger of the reservation
    /// and the reported count, or the reservation when the count is unknown.
    pub fn start(
        &mut self,
        request: &super::super::step::Request,
        report: &super::super::step::Report,
    ) -> Result<super::super::step::Usage, super::super::KilnOwned> {
        self.binding.admit_step(&request.binding)?;
        self.steps = self.steps.saturating_add(1);
        debug_assert_eq!(request.iteration, self.steps, "steps are recorded in order");
        let step_tools = self.policy.step_tool_invocations;
        let charged = super::super::super::lineage::Draws {
            model_requests: report
                .model_requests
                .map_or(self.reservation.model_requests, |reported| reported.max(self.reservation.model_requests)),
            tool_invocations: report.tool_invocations.map_or(step_tools, |reported| reported.max(step_tools)),
        };
        self.drawn = self.drawn.saturating_add(&charged);
        let is_reported = report.model_requests.is_some() && report.tool_invocations.is_some();
        Ok(super::super::step::Usage {
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

    /// Charge the host check executions of one revision.
    pub fn charge_checks(&mut self, iteration: u32, executions: u64) -> super::super::step::Usage {
        let charged = super::super::super::lineage::Draws {
            model_requests: 0,
            tool_invocations: executions,
        };
        self.drawn = self.drawn.saturating_add(&charged);
        super::super::step::Usage {
            phase: super::super::step::Phase::Check,
            iteration,
            model_requests: Some(0),
            tool_invocations: Some(executions),
            provenance: crate::orchestration::UsageProvenance::HostMeasured,
            charged,
        }
    }
}

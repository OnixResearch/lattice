//! Charges of one repair loop's steps and checks: exact for host-observed
//! model requests, conservative for everything else.

// r[impl lattice.implementation_attempt.lineage.scenario.usage]
// r[impl lattice.implementation_attempt.observed_charge]
// r[impl lattice.implementation_attempt.observed_charge.scenario.step_range]

impl super::Loop {
    /// Record a spawned step and charge it. Model requests the host observed
    /// in complete broker evidence are charged exactly with host-measured
    /// provenance. Otherwise each dimension is charged the larger of the
    /// reservation and the worker-reported count, or the reservation when the
    /// count is unknown.
    pub fn start(
        &mut self,
        request: &super::super::step::Request,
        report: &super::super::step::Report,
    ) -> Result<super::super::step::Usage, super::super::KilnOwned> {
        self.binding.admit_step(&request.binding)?;
        self.steps = self.steps.saturating_add(1);
        debug_assert_eq!(request.iteration, self.steps, "steps are recorded in order");
        let step_tools = self.policy.step_tool_invocations;
        let tool_invocations = report.tool_invocations.map_or(step_tools, |reported| reported.max(step_tools));
        let (model_requests, charged_requests, provenance) = match report.observed_model_requests {
            Some(observed) => (Some(observed), observed, crate::orchestration::UsageProvenance::HostMeasured),
            None => {
                let reservation = self.reservation.model_requests;
                let charged = report.model_requests.map_or(reservation, |reported| reported.max(reservation));
                let is_reported = report.model_requests.is_some() && report.tool_invocations.is_some();
                let provenance = if is_reported {
                    crate::orchestration::UsageProvenance::WorkerReported
                } else {
                    crate::orchestration::UsageProvenance::Unknown
                };
                (report.model_requests, charged, provenance)
            }
        };
        let charged = super::super::super::lineage::Draws {
            model_requests: charged_requests,
            tool_invocations,
        };
        self.drawn = self.drawn.saturating_add(&charged);
        Ok(super::super::step::Usage {
            phase: request.phase,
            iteration: request.iteration,
            model_requests,
            tool_invocations: report.tool_invocations,
            provenance,
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

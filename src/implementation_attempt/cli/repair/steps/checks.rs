//! The declared checks of one candidate revision, each against its own copy.

// r[impl lattice.implementation_attempt.repair.scenario.bypass]

impl super::Driver<'_, '_> {
    /// Run every declared check, each against its own read-only copy of the
    /// candidate and within the time left, then classify the evidence.
    pub(super) fn check(
        &mut self,
        ports: &mut super::super::Ports<'_>,
        iteration: u32,
        revision: &str,
        tree: &super::super::materialized::Tree,
    ) -> Result<(Vec<super::Observation>, Option<super::Verdict>, super::Next), String> {
        let invocations = crate::implementation_attempt::repair::checks::invocations(&self.policy, revision, iteration);
        let mut observations = Vec::with_capacity(invocations.len());
        let mut drifted = None;
        let mut stopped = None;
        for invocation in &invocations {
            let path = self.frame.records.check_copy(iteration, &invocation.check);
            let copy = super::super::materialized::Copy::create(&path, tree)?;
            // The check is bounded by the time left now, after its copy is
            // materialized; a bound reached or a clock lost by then stops
            // before the check is called.
            let remaining_ms = match self.state.remaining_ms(self.elapsed_ms(ports.clock)) {
                Ok(remaining_ms) => remaining_ms,
                Err(disposition) => {
                    copy.remove()?;
                    stopped = Some(disposition);
                    break;
                }
            };
            let deadline = std::time::Duration::from_millis(remaining_ms);
            let observation = ports.checks.run(invocation, copy.path(), deadline);
            let is_intact = copy.is_intact(&tree.identity);
            copy.remove()?;
            observations.push(observation);
            if !is_intact {
                drifted = Some(invocation.check.clone());
                break;
            }
        }
        let executions = u64::try_from(observations.len()).map_err(|error| error.to_string())?;
        self.usage.push(self.state.charge_checks(iteration, executions));
        if let Some(disposition) = stopped {
            return Ok((observations, None, super::Next::Stop(disposition)));
        }
        let subject = crate::implementation_attempt::repair::checks::Subject {
            policy: &self.policy,
            revision,
            worker_profile: &self.frame.request.envelope.worker_profile,
        };
        let verdict =
            crate::implementation_attempt::repair::checks::classify(subject, &observations, drifted.as_deref());
        let next = self.state.after_checks(verdict.clone(), self.elapsed_ms(ports.clock));
        Ok((observations, Some(verdict), next))
    }
}

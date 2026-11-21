//! The elapsed, iteration, budget and lineage bounds of one repair loop.

// r[impl lattice.implementation_attempt.repair.scenario.dispositions]

type Bound = super::Bound;
type Disposition = super::Disposition;

impl super::Loop {
    /// The time left before the elapsed bound, which bounds the next worker
    /// step or check. An unreadable clock is uncertain; a reached bound is
    /// exhausted.
    pub fn remaining_ms(&self, elapsed_ms: Option<u64>) -> Result<u64, Disposition> {
        let Some(elapsed_ms) = elapsed_ms else {
            return Err(Disposition::Uncertain {
                cause: super::Uncertainty::Clock,
            });
        };
        self.policy
            .max_elapsed_ms
            .checked_sub(elapsed_ms)
            .filter(|remaining_ms| *remaining_ms > 0)
            .ok_or(Disposition::BudgetExhausted { bound: Bound::Elapsed })
    }

    /// Whether one more step's reservation fits the iteration bound and the
    /// aggregate budgets.
    pub(super) fn within_policy(&self) -> Result<(), Bound> {
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

    /// The bound the charges so far already exceed: a policy budget, or the
    /// lineage remainder of a dimension Lattice charges.
    fn overrun(&self) -> Option<Bound> {
        if self.drawn.model_requests > self.policy.max_model_requests {
            return Some(Bound::ModelRequests);
        }
        if self.drawn.tool_invocations > self.policy.max_tool_invocations {
            return Some(Bound::ToolInvocations);
        }
        let dimension = self.lineage.as_ref()?.overdrawn(&self.drawn)?;
        Some(Bound::Lineage { dimension })
    }

    /// Whether the run is still inside every bound after the charges and
    /// time of its latest step: the clock is read, the elapsed bound is not
    /// reached and no charge overran a budget.
    pub(super) fn settled(&self, elapsed_ms: Option<u64>) -> Result<(), Disposition> {
        self.remaining_ms(elapsed_ms)?;
        match self.overrun() {
            Some(bound) => Err(Disposition::BudgetExhausted { bound }),
            None => Ok(()),
        }
    }
}

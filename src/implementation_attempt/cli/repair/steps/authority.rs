//! The revocable authority repair steps recheck before their intents.

// r[impl lattice.implementation_attempt.repair.scenario.dispositions]

impl super::Driver<'_, '_> {
    /// Under the fence lock before step `iteration`'s intent: the standalone
    /// operator's grant for every step, then, after the first, the pre-spawn
    /// binding revalidation and revision-seam effect check, without their
    /// pre-dispatch records. `Some` when one withdrew the authority. A seam
    /// that cannot answer withdraws it too, since no step spawns without a
    /// current permit.
    // r[impl lattice.implementation_attempt.operator_repair.scenario.authority]
    pub(super) fn recheck_authority(&mut self, iteration: u32) -> Option<super::Withdrawal> {
        if let Some(withdrawal) = self.recheck_operator(iteration) {
            return Some(withdrawal);
        }
        let authority = &mut self.frame.authority;
        if iteration == 1 {
            return None;
        }
        let binding = super::super::super::binding::recheck::current(&authority.admitted, &mut *authority.compute);
        if let Err(record) = binding {
            return Some(super::Withdrawal {
                iteration,
                cause: super::Authority::ExecutionBinding,
                evidence: record,
            });
        }
        let revisions = authority.revisions;
        let checked =
            super::super::super::session::revision::recheck(revisions, self.frame.request, &authority.recorded);
        let evidence = match checked {
            Ok(None) => return None,
            Ok(Some(refusal)) => serde_json::json!({ "refusal": refusal }),
            Err(error) => serde_json::json!({ "error": error }),
        };
        Some(super::Withdrawal {
            iteration,
            cause: super::Authority::ManagedRevision,
            evidence,
        })
    }

    /// The standalone operator's grant, before step `iteration`'s intent or
    /// before its candidate's checks: `Some` when it no longer grants both
    /// tuples with its admitted identity or the admission expired; always
    /// `None` for embedder-supplied ports.
    // r[impl lattice.implementation_attempt.operator_repair.scenario.authority]
    pub(super) fn recheck_operator(&self, iteration: u32) -> Option<super::Withdrawal> {
        let evidence = self.frame.authority.operator.as_ref()?.recheck(self.frame.request)?;
        Some(super::Withdrawal {
            iteration,
            cause: super::Authority::EffectAuthority,
            evidence,
        })
    }
}

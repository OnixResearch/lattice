//! The revocable authority every repair step after the first rechecks.

// r[impl lattice.implementation_attempt.repair.scenario.dispositions]

impl super::Driver<'_, '_> {
    /// The pre-spawn binding revalidation and revision-seam effect check,
    /// without their pre-dispatch records: `Some` when either withdrew the
    /// authority. A seam that cannot answer withdraws it too, since no step
    /// spawns without a current permit.
    pub(super) fn recheck_authority(&mut self, iteration: u32) -> Option<super::Withdrawal> {
        let authority = &mut self.frame.authority;
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
}

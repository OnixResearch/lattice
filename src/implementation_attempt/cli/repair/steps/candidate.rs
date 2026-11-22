//! The host-derived candidate revision of one exited step.

// r[impl lattice.implementation_attempt.repair.scenario.bypass]

impl super::Driver<'_, '_> {
    /// The host-derived candidate of an exited step: `None` when the
    /// workspace equals its parent or the base. A cancelled or uncertain
    /// step's workspace is retained, never captured as a candidate.
    pub(super) fn candidate(
        &mut self,
        step: &super::StepRequest,
        report: &crate::implementation_attempt::repair::step::Report,
    ) -> Result<Option<super::Candidate>, String> {
        if !matches!(report.outcome, crate::implementation_attempt::repair::step::WorkerOutcome::Exited { .. }) {
            return Ok(None);
        }
        let workspace = &self.frame.request.workspace_root;
        let snapshot = crate::candidate_capture::workspace(workspace).map_err(|error| error.to_string())?;
        let identity = crate::candidate::snapshot_identity(&snapshot).map_err(|error| error.to_string())?;
        if identity == self.parent_identity {
            return Ok(None);
        }
        let modes =
            crate::candidate_capture::snapshot_modes(workspace, &snapshot).map_err(|error| error.to_string())?;
        let envelope = &self.frame.request.envelope;
        let fence = envelope.fence.to_string();
        let bindings = crate::candidate::Bindings {
            work_id: &envelope.work_id,
            attempt_id: &envelope.attempt_id,
            fence: &fence,
            workspace_id: &envelope.workspace_id,
        };
        let receipt = match crate::candidate::produced_receipt_from_snapshots(bindings, &self.base, &snapshot) {
            Ok(receipt) => receipt,
            Err(crate::candidate::CandidateReceiptError::NoCandidateChanges) => return Ok(None),
            Err(error) => return Err(error.to_string()),
        };
        self.parent_identity.clone_from(&identity);
        let parent = self.state.last_revision().map(str::to_owned);
        // The execute bits every check's copy is sealed with, and the base
        // digest of every changed path, retained with the revision.
        let executables = crate::implementation_attempt::repair::revision::executables(&modes);
        let base_files = receipt
            .changed_paths
            .iter()
            .map(|path| {
                (path.clone(), self.base.get(path).map(|bytes| crate::candidate::digests::FileDigest::of(bytes)))
            })
            .collect();
        let revision = crate::implementation_attempt::repair::revision::identity(
            crate::implementation_attempt::repair::revision::Input {
                binding: &step.binding,
                iteration: step.iteration,
                parent: parent.as_deref(),
                candidate_identity: receipt.candidate_identity.as_deref().unwrap_or_default(),
                executables: &executables,
                diff_identity: receipt.diff_identity.as_deref().unwrap_or_default(),
            },
        );
        Ok(Some(super::Candidate {
            revision: super::Revision {
                iteration: step.iteration,
                revision,
                parent,
                receipt,
                executables,
                base_files,
            },
            tree: super::super::materialized::Tree {
                files: snapshot,
                modes,
                identity,
            },
        }))
    }
}

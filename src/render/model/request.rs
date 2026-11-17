use anyhow::Context;

impl super::Request {
    pub fn id(&self) -> anyhow::Result<molten_batch_core::Id> {
        Ok(molten_orchestrator_codec::identity(super::super::SCHEMA, self)?)
    }

    pub fn prepare(&self) -> anyhow::Result<molten_render_pilot::Prepared> {
        anyhow::ensure!(self.schema == super::super::SCHEMA, "unsupported binding schema");
        for value in [&self.execution, &self.attempt, &self.fence, &self.budget.identity] {
            anyhow::ensure!(!value.is_empty() && value.len() <= 256, "missing or oversized owning identity");
        }
        anyhow::ensure!(self.budget.deadline_epoch_millis > 0, "missing original deadline");
        for path in [
            &self.route.executable,
            &self.route.client_root,
            &self.component_file,
            &self.authority_file,
            &self.journal_root,
        ] {
            let path = std::path::Path::new(path);
            anyhow::ensure!(
                path.is_absolute() && !path.components().any(|p| matches!(p, std::path::Component::ParentDir)),
                "absolute admitted path required"
            );
        }
        anyhow::ensure!(
            crate::operation::identity::validate(&self.route.executable_identity),
            "invalid executor identity"
        );
        let (peer, address) = self.route.controller.split_once('@').context("controller key/address required")?;
        anyhow::ensure!(
            molten_batch_core::Id::parse(peer).is_ok() && address.parse::<std::net::SocketAddr>().is_ok(),
            "numeric controller required"
        );
        anyhow::ensure!(
            self.route.cluster != molten_batch_core::Id([0; 32])
                && self.route.policy_ref != molten_batch_core::Id([0; 32])
                && self.route.generation > 0,
            "missing producer cohort"
        );
        let prepared = molten_render_pilot::prepare(self.manifest.clone())?;
        let attempts = u64::from(self.manifest.tile_count())
            .checked_add(1)
            .context("task reservation overflow")?
            .checked_mul(u64::from(self.manifest.runtime.max_attempts))
            .context("attempt reservation overflow")?;
        anyhow::ensure!(attempts <= self.budget.task_attempts, "task/assembly retry reservation exceeds budget");
        anyhow::ensure!(
            attempts.checked_mul(self.manifest.runtime.fuel).is_some_and(|n| n <= self.budget.fuel),
            "fuel reservation exceeds budget"
        );
        anyhow::ensure!(
            attempts
                .checked_mul(self.manifest.runtime.timeout_ms)
                .is_some_and(|n| n <= self.budget.task_wall_millis),
            "wall reservation exceeds budget"
        );
        Ok(prepared)
    }

    pub fn check_authority(&self, authority: &super::Authority) -> anyhow::Result<()> {
        anyhow::ensure!(
            authority.current
                && authority.execution == self.execution
                && authority.attempt == self.attempt
                && authority.fence == self.fence
                && authority.budget_identity == self.budget.identity,
            "owning attempt is not current"
        );
        Ok(())
    }

    pub fn submission_key(&self) -> anyhow::Result<String> {
        Ok(self.id()?.to_string())
    }

    /// Producer's existing deterministic run identity permits read-only lost-ACK
    /// reconciliation. It does NOT authorize repeating Submit.
    pub fn run_id(&self) -> anyhow::Result<molten_batch_core::Id> {
        Ok(molten_orchestrator_codec::identity("run-id.v1", &(self.route.cluster, self.submission_key()?))?)
    }
}

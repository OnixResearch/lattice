#![allow(
    unknown_lints,
    assertion_density,
    path_segment_repetition,
    reason = "the checkpoint continuation shares the snapshot persistence sequence of its parent shell"
)]

//! Checkpoint continuation of one snapshot run: restored terminal steps and node-boundary progress.

impl super::SnapshotRunRequest {
    /// [`super::run_workflow_snapshot_with_authority_provider_and_cancellation`] continued from
    /// terminal checkpoint steps, which restore through `Session::new_with_checkpoint` and never
    /// re-run. `progress` receives `NodeProgress::Finished` at each completed-node boundary.
    pub async fn run_from_checkpoint(
        self,
        store: &crate::store::Store,
        authority_provider: &dyn crate::engine::effect::execution::RunContextProvider,
        completed: Vec<crate::engine::executor::StepResult>,
        progress: Option<crate::engine::executor::progress::Sender>,
    ) -> Result<super::SnapshotRunOutcome, crate::headless::HeadlessRunError> {
        let Self {
            workflow,
            run_id,
            owner,
            agent_runtime,
            cancellation,
        } = self;
        let diagnostics = crate::workflow_validation::validate_workflow(&workflow);
        if crate::workflow_validation::has_fatal_diagnostics(&diagnostics) {
            return Err(crate::headless::HeadlessRunError::Validation(crate::workflow_validation::fatal_diagnostics(
                &diagnostics,
            )));
        }
        let started_epoch_ms = crate::headless::run_started_epoch_ms();
        let workflow_catalog = std::sync::Arc::new(crate::engine::WorkflowCatalogRepository::from_repository(store)?);
        let subworkflow_shell = crate::engine::RepositorySubworkflowRunnerShell::product(workflow_catalog);
        let executor = crate::engine::executor::Executor::new_with_agent_runtime_config(agent_runtime)
            .with_required_shell_success()
            .with_cancellation(cancellation)
            .with_subworkflow_runner_shell(Box::new(subworkflow_shell));
        let mut executor = match progress {
            Some(sender) => executor.with_progress_sender(sender),
            None => executor,
        };
        let authority = authority_provider.load(run_id, Some(store.clone())).map_err(|error| {
            crate::headless::HeadlessRunError::Validation(vec![super::authority_context_diagnostic(error.to_string())])
        })?;
        let cache = std::collections::HashMap::new();
        let outcome = executor.run_authorized_from_checkpoint(&workflow, &cache, &authority, &completed).await;
        let facts = super::record::SnapshotRunFacts {
            run_id,
            owner,
            started_epoch_ms,
        };
        super::record::persist_snapshot_outcome(store, &workflow, &mut executor, facts, outcome)
    }
}

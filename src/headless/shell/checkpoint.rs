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
        validate_checkpoint_workflow(&workflow)?;
        let executor = crate::engine::executor::Executor::new_with_agent_runtime_config(agent_runtime)
            .with_required_shell_success()
            .with_cancellation(cancellation);
        run_executor_from_checkpoint(store, authority_provider, executor, CheckpointRunContext {
            workflow,
            run_id,
            owner,
            completed,
            progress,
        })
        .await
    }
}

impl super::AppConfigSnapshotRun {
    /// Continue a snapshot run from checkpoint steps using [`crate::config::AppConfig`] credential
    /// providers and workflow-level credential registry validation.
    pub async fn run_from_checkpoint(
        self,
        store: &crate::store::Store,
        app_config: &crate::config::AppConfig,
        authority_provider: &dyn crate::engine::effect::execution::RunContextProvider,
        completed: Vec<crate::engine::executor::StepResult>,
        progress: Option<crate::engine::executor::progress::Sender>,
    ) -> Result<super::SnapshotRunOutcome, crate::headless::HeadlessRunError> {
        let Self {
            workflow,
            run_id,
            owner,
            cancellation,
        } = self;
        validate_checkpoint_workflow(&workflow)?;
        let executor = super::profile::configured_executor_for_workflow(app_config, &workflow, cancellation)?
            .with_required_shell_success();
        run_executor_from_checkpoint(store, authority_provider, executor, CheckpointRunContext {
            workflow,
            run_id,
            owner,
            completed,
            progress,
        })
        .await
    }
}

struct CheckpointRunContext {
    workflow: crate::engine::workflow::Workflow,
    run_id: crate::engine::executor::RunId,
    owner: crate::store::RunOwner,
    completed: Vec<crate::engine::executor::StepResult>,
    progress: Option<crate::engine::executor::progress::Sender>,
}

fn validate_checkpoint_workflow(
    workflow: &crate::engine::workflow::Workflow,
) -> Result<(), crate::headless::HeadlessRunError> {
    let diagnostics = crate::workflow_validation::validate_workflow(workflow);
    if crate::workflow_validation::has_fatal_diagnostics(&diagnostics) {
        return Err(crate::headless::HeadlessRunError::Validation(crate::workflow_validation::fatal_diagnostics(
            &diagnostics,
        )));
    }
    Ok(())
}

async fn run_executor_from_checkpoint(
    store: &crate::store::Store,
    authority_provider: &dyn crate::engine::effect::execution::RunContextProvider,
    executor: crate::engine::executor::Executor,
    context: CheckpointRunContext,
) -> Result<super::SnapshotRunOutcome, crate::headless::HeadlessRunError> {
    let started_epoch_ms = crate::headless::run_started_epoch_ms();
    let workflow_catalog = std::sync::Arc::new(crate::engine::WorkflowCatalogRepository::from_repository(store)?);
    let subworkflow_shell = crate::engine::RepositorySubworkflowRunnerShell::product(workflow_catalog);
    let executor = executor.with_subworkflow_runner_shell(Box::new(subworkflow_shell));
    let mut executor = match context.progress {
        Some(sender) => executor.with_progress_sender(sender),
        None => executor,
    };
    let authority = authority_provider.load(context.run_id, Some(store.clone())).map_err(|error| {
        crate::headless::HeadlessRunError::Validation(vec![super::authority_context_diagnostic(error.to_string())])
    })?;
    let cache = std::collections::HashMap::new();
    let outcome = executor
        .run_authorized_from_checkpoint(&context.workflow, &cache, &authority, &context.completed)
        .await;
    let facts = super::record::SnapshotRunFacts {
        run_id: context.run_id,
        owner: context.owner,
        started_epoch_ms,
    };
    super::record::persist_snapshot_outcome(store, &context.workflow, &mut executor, facts, outcome)
}

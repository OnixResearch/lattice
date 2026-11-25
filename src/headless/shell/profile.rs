pub struct AppConfigSnapshotRun {
    pub workflow: crate::engine::workflow::Workflow,
    pub run_id: crate::engine::executor::RunId,
    pub owner: crate::store::RunOwner,
    pub cancellation: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

pub async fn run_saved_workflow_with_app_config(
    store: &crate::store::Store,
    workflow_name: &str,
    run_id: crate::engine::executor::RunId,
    app_config: &crate::config::AppConfig,
) -> Result<super::super::HeadlessRunSummary, super::super::HeadlessRunError> {
    run_owned_workflow_with_app_config(store, workflow_name, run_id, crate::store::RunOwner::Unscoped, app_config).await
}

pub(in crate::headless) fn configured_executor_for_workflow(
    app_config: &crate::config::AppConfig,
    workflow: &crate::engine::workflow::Workflow,
    cancellation: std::sync::Arc<std::sync::atomic::AtomicBool>,
) -> Result<crate::engine::executor::Executor, super::super::HeadlessRunError> {
    let agent_runtime = app_config.effective_agent_runtime();
    let frontier_policy = app_config.frontier_capacity_policy().unwrap_or_else(|error| {
        tracing::error!(?error, "invalid frontier policy, preserving serial execution");
        crate::engine::frontier::serial_capacity_policy()
    });
    debug_assert!(frontier_policy.total_slots > 0);
    debug_assert!(!frontier_policy.lane_slots.is_empty());
    let registry = app_config
        .workflow_credential_registry(Some(workflow), std::sync::Arc::clone(&cancellation))
        .map_err(|error| {
            super::super::HeadlessRunError::Validation(vec![super::authority_context_diagnostic(
                error.code().to_string(),
            )])
        })?;
    if !app_config.credential_providers.is_empty() {
        let diagnostics = crate::workflow_validation::validate_workflow_with_credential_registry(workflow, &registry);
        if crate::workflow_validation::has_fatal_diagnostics(&diagnostics) {
            return Err(super::super::HeadlessRunError::Validation(crate::workflow_validation::fatal_diagnostics(
                &diagnostics,
            )));
        }
    }
    Ok(configured_executor(agent_runtime, frontier_policy)
        .with_credential_provider(Box::new(registry))
        .with_cancellation(cancellation))
}

fn configured_executor(
    agent_runtime: crate::agent_node::RuntimeOptions,
    frontier_policy: lattice_core::scheduling::CapacityPolicy,
) -> crate::engine::executor::Executor {
    let fallback_runtime = agent_runtime.clone();
    match crate::engine::executor::Executor::new_with_agent_runtime_config(agent_runtime)
        .with_frontier_policy(frontier_policy)
    {
        Ok(executor) => executor,
        Err(error) => {
            tracing::error!(?error, "validated frontier policy was rejected, preserving serial execution");
            crate::engine::executor::Executor::new_with_agent_runtime_config(fallback_runtime)
        }
    }
}

pub async fn run_owned_workflow_with_app_config(
    store: &crate::store::Store,
    workflow_name: &str,
    run_id: crate::engine::executor::RunId,
    owner: crate::store::RunOwner,
    app_config: &crate::config::AppConfig,
) -> Result<super::super::HeadlessRunSummary, super::super::HeadlessRunError> {
    debug_assert!(!workflow_name.is_empty());
    let workflow = store
        .load_workflow(workflow_name)?
        .ok_or_else(|| super::super::HeadlessRunError::WorkflowNotFound(workflow_name.to_string()))?;
    let cancellation = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let executor = configured_executor_for_workflow(app_config, &workflow, cancellation)?;
    let authority_provider = app_config.effect_authority_provider();
    debug_assert_eq!(workflow.meta.name, workflow_name);
    super::run_workflow_snapshot_with_executor_factory(
        super::ProductRunRequest {
            store,
            workflow_name,
            run_id,
            owner,
            authority_provider: authority_provider.as_ref(),
        },
        workflow,
        || executor,
    )
    .await
    .map(|outcome| outcome.summary)
}

pub async fn run_workflow_snapshot_with_app_config(
    store: &crate::store::Store,
    request: AppConfigSnapshotRun,
    app_config: &crate::config::AppConfig,
) -> Result<super::SnapshotRunOutcome, super::super::HeadlessRunError> {
    let workflow_name = request.workflow.meta.name.clone();
    let executor = configured_executor_for_workflow(app_config, &request.workflow, request.cancellation)?
        .with_required_shell_success();
    let authority_provider = app_config.effect_authority_provider();
    super::run_workflow_snapshot_with_executor_factory(
        super::ProductRunRequest {
            store,
            workflow_name: &workflow_name,
            run_id: request.run_id,
            owner: request.owner,
            authority_provider: authority_provider.as_ref(),
        },
        request.workflow,
        || executor,
    )
    .await
}

//! Imperative shell for one persisted product workflow run.

#[derive(Debug, Clone)]
pub struct OwnedRunRequest {
    pub workflow_name: String,
    pub run_id: crate::engine::executor::RunId,
    pub owner: crate::store::RunOwner,
    pub agent_runtime: crate::agent_node::AgentRuntimeConfig,
}

pub(super) struct ProductRunRequest<'a> {
    pub store: &'a crate::store::Store,
    pub workflow_name: &'a str,
    pub run_id: crate::engine::executor::RunId,
    pub owner: crate::store::RunOwner,
    pub authority_provider: &'a dyn crate::engine::effect::execution::RunContextProvider,
}

pub async fn run_saved_workflow_with_app_config(
    store: &crate::store::Store,
    workflow_name: &str,
    run_id: crate::engine::executor::RunId,
    app_config: &crate::config::AppConfig,
) -> Result<super::HeadlessRunSummary, super::HeadlessRunError> {
    let agent_runtime = app_config.effective_agent_runtime();
    let frontier_policy = app_config.frontier_capacity_policy().unwrap_or_else(|error| {
        tracing::error!(?error, "invalid frontier policy, preserving serial execution");
        crate::engine::frontier::serial_capacity_policy()
    });
    debug_assert!(frontier_policy.total_slots > 0);
    debug_assert!(!frontier_policy.lane_slots.is_empty());
    let authority_provider = app_config.effect_authority_provider();
    run_saved_workflow_with_executor_factory(
        ProductRunRequest {
            store,
            workflow_name,
            run_id,
            owner: crate::store::RunOwner::Unscoped,
            authority_provider: authority_provider.as_ref(),
        },
        || configured_executor(agent_runtime, frontier_policy),
    )
    .await
}

fn configured_executor(
    agent_runtime: crate::agent_node::AgentRuntimeConfig,
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

// r[impl lattice.agent_orchestration.run_identity]
pub async fn run_owned_workflow(
    store: &crate::store::Store,
    request: OwnedRunRequest,
) -> Result<super::HeadlessRunSummary, super::HeadlessRunError> {
    let authority_provider = crate::engine::effect::execution::UnavailableRunContextSource;
    run_owned_workflow_with_authority_provider(store, request, &authority_provider).await
}

pub async fn run_owned_workflow_with_authority_provider(
    store: &crate::store::Store,
    request: OwnedRunRequest,
    authority_provider: &dyn crate::engine::effect::execution::RunContextProvider,
) -> Result<super::HeadlessRunSummary, super::HeadlessRunError> {
    run_owned_workflow_with_authority_provider_and_cancellation(
        store,
        request,
        authority_provider,
        std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
    )
    .await
}

pub async fn run_owned_workflow_with_authority_provider_and_cancellation(
    store: &crate::store::Store,
    request: OwnedRunRequest,
    authority_provider: &dyn crate::engine::effect::execution::RunContextProvider,
    cancellation: std::sync::Arc<std::sync::atomic::AtomicBool>,
) -> Result<super::HeadlessRunSummary, super::HeadlessRunError> {
    let workflow_name = request.workflow_name;
    let agent_runtime = request.agent_runtime;
    run_saved_workflow_with_executor_factory(
        ProductRunRequest {
            store,
            workflow_name: &workflow_name,
            run_id: request.run_id,
            owner: request.owner,
            authority_provider,
        },
        || {
            crate::engine::executor::Executor::new_with_agent_runtime_config(agent_runtime)
                .with_cancellation(cancellation)
        },
    )
    .await
}

// r[impl n8nclone.workflow_execution.trait_boundaries.storage_ports]
pub(super) async fn run_saved_workflow_with_executor_factory(
    request: ProductRunRequest<'_>,
    executor_factory: impl FnOnce() -> crate::engine::executor::Executor,
) -> Result<super::HeadlessRunSummary, super::HeadlessRunError> {
    let workflow = request
        .store
        .load_workflow(request.workflow_name)?
        .ok_or_else(|| super::HeadlessRunError::WorkflowNotFound(request.workflow_name.to_string()))?;
    run_workflow_snapshot_with_executor_factory(request, workflow, executor_factory)
        .await
        .map(|outcome| outcome.summary)
}

pub(super) async fn run_workflow_snapshot_with_executor_factory(
    request: ProductRunRequest<'_>,
    workflow: crate::engine::workflow::Workflow,
    executor_factory: impl FnOnce() -> crate::engine::executor::Executor,
) -> Result<SnapshotRunOutcome, super::HeadlessRunError> {
    let ProductRunRequest {
        store,
        workflow_name,
        run_id,
        owner,
        authority_provider,
    } = request;
    if workflow.meta.name != workflow_name {
        return Err(super::HeadlessRunError::WorkflowNotFound(workflow_name.to_string()));
    }
    let diagnostics = crate::workflow_validation::validate_workflow(&workflow);
    if crate::workflow_validation::has_fatal_diagnostics(&diagnostics) {
        return Err(super::HeadlessRunError::Validation(crate::workflow_validation::fatal_diagnostics(&diagnostics)));
    }

    let started_epoch_ms = super::run_started_epoch_ms();
    let workflow_catalog = std::sync::Arc::new(crate::engine::WorkflowCatalogRepository::from_repository(store)?);
    let subworkflow_shell = crate::engine::RepositorySubworkflowRunnerShell::product(workflow_catalog);
    let mut executor = executor_factory().with_subworkflow_runner_shell(Box::new(subworkflow_shell));
    let authority = authority_provider
        .load(run_id, Some(store.clone()))
        .map_err(|error| super::HeadlessRunError::Validation(vec![authority_context_diagnostic(error.to_string())]))?;
    let cache = std::collections::HashMap::new();
    let outcome = executor.run_authorized_until_approval(&workflow, &cache, &authority).await;
    let observed_completion_order = executor.observed_completion_order().to_vec();
    super::persist_completed_outputs_to_cache(store, &outcome.results);
    if let Some(suspension) = outcome.suspension {
        let (summary, _request) =
            crate::workflow::gate::shell::persist_suspension(crate::workflow::gate::shell::SuspensionCommitInput {
                store,
                workflow: &workflow,
                run_id,
                owner,
                started_epoch_millis: started_epoch_ms,
                observed_epoch_millis: super::run_finished_epoch_ms(),
                results: &outcome.results,
                suspension,
            })
            .map_err(|error| super::HeadlessRunError::Store(error.to_string()))?;
        return Ok(SnapshotRunOutcome {
            summary,
            cancelled: executor.cancellation_observed(),
        });
    }
    let results = outcome.results;
    let summary = super::summarize_run(run_id, &workflow, &results);
    let finished_epoch_ms = super::run_finished_epoch_ms();
    let record = run_record_from_summary_with_owner(&summary, started_epoch_ms, Some(finished_epoch_ms), owner);
    let details = crate::run::history::detail_record_from_results_with_completion_order(
        run_id,
        &workflow,
        &results,
        observed_completion_order,
    );
    debug_assert_eq!(details.run_id, run_id);
    debug_assert_eq!(details.nodes.len(), results.len());
    store.save_run(&record)?;
    // r[impl lattice.workflow_execution.headless_run_details]
    store.save_run_details(&details)?;
    let replay_events = crate::replay::replay_events_from_results(
        run_id,
        &workflow,
        &results,
        started_epoch_ms,
        Some(finished_epoch_ms),
    );
    store.save_run_replay_events(run_id, &replay_events)?;

    Ok(SnapshotRunOutcome {
        summary,
        cancelled: executor.cancellation_observed(),
    })
}

#[derive(Debug, Clone)]
pub struct SnapshotRunOutcome {
    pub summary: super::HeadlessRunSummary,
    /// Observed owned process termination, never the cancellation request flag.
    pub cancelled: bool,
}

#[derive(Debug, Clone)]
pub struct SnapshotRunRequest {
    pub workflow: crate::engine::workflow::Workflow,
    pub run_id: crate::engine::executor::RunId,
    pub owner: crate::store::RunOwner,
    pub agent_runtime: crate::agent_node::AgentRuntimeConfig,
    pub cancellation: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

pub async fn run_workflow_snapshot_with_authority_provider_and_cancellation(
    store: &crate::store::Store,
    request: SnapshotRunRequest,
    authority_provider: &dyn crate::engine::effect::execution::RunContextProvider,
) -> Result<SnapshotRunOutcome, super::HeadlessRunError> {
    let workflow_name = request.workflow.meta.name.clone();
    run_workflow_snapshot_with_executor_factory(
        ProductRunRequest {
            store,
            workflow_name: &workflow_name,
            run_id: request.run_id,
            owner: request.owner,
            authority_provider,
        },
        request.workflow,
        || {
            crate::engine::executor::Executor::new_with_agent_runtime_config(request.agent_runtime)
                .with_required_shell_success()
                .with_cancellation(request.cancellation)
        },
    )
    .await
}

fn run_record_from_summary_with_owner(
    summary: &super::HeadlessRunSummary,
    started_epoch_ms: i64,
    finished_epoch_ms: Option<i64>,
    owner: crate::store::RunOwner,
) -> super::RunRecord {
    super::RunRecord {
        run_id: summary.run_id,
        owner,
        workflow_name: summary.workflow_name.clone(),
        started_epoch_ms,
        finished_epoch_ms,
        node_count: summary.node_count,
        cache_hits: summary.cache_hits,
        failures: summary.failures,
    }
}

fn authority_context_diagnostic(message: String) -> crate::workflow_validation::WorkflowDiagnostic {
    crate::workflow_validation::WorkflowDiagnostic {
        code: crate::workflow_validation::WorkflowDiagnosticCode::RunnerRuntimeFailure,
        severity: crate::workflow_validation::DiagnosticSeverity::Fatal,
        node_id: None,
        port_id: None,
        message,
    }
}

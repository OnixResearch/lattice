//! Imperative shell for one persisted product workflow run.

mod bound;
mod checkpoint;
mod profile;
mod record;

pub use profile::AppConfigSnapshotRun;
pub use profile::run_owned_workflow_with_app_config;
pub use profile::run_saved_workflow_with_app_config;
pub use profile::run_workflow_snapshot_with_app_config;

#[derive(Debug, Clone)]
pub struct OwnedRunRequest {
    pub workflow_name: String,
    pub run_id: crate::engine::executor::RunId,
    pub owner: crate::store::RunOwner,
    pub agent_runtime: crate::agent_node::RuntimeOptions,
}

pub(super) struct ProductRunRequest<'a> {
    pub store: &'a crate::store::Store,
    pub workflow_name: &'a str,
    pub run_id: crate::engine::executor::RunId,
    pub owner: crate::store::RunOwner,
    pub authority_provider: &'a dyn crate::engine::effect::execution::RunContextProvider,
}

/// A saved-workflow run whose bound Condition nodes use typed policy profiles.
pub struct TypedPolicyRunRequest<'a> {
    pub store: &'a crate::store::Store,
    pub workflow_name: &'a str,
    pub run_id: crate::engine::executor::RunId,
    pub authority_provider: &'a dyn crate::engine::effect::execution::RunContextProvider,
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
// r[impl lattice.graph_construction.snapshot]
// r[impl lattice.graph_construction.snapshot.readmit]
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
    let facts = record::SnapshotRunFacts {
        run_id,
        owner,
        started_epoch_ms,
    };
    let persisted = record::persist_snapshot_outcome(store, &workflow, &mut executor, facts, outcome)?;
    debug_assert_eq!(persisted.summary.run_id, run_id);
    debug_assert_eq!(persisted.summary.workflow_name, workflow_name);
    Ok(persisted)
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
    pub agent_runtime: crate::agent_node::RuntimeOptions,
    pub cancellation: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

// r[impl lattice.graph_construction.runtime_controls]
// r[impl lattice.graph_construction.runtime_controls.result]
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

use serde::Deserialize;
use serde::Serialize;

use crate::agent_node::AgentRuntimeConfig;
use crate::engine::executor::Executor;
use crate::engine::executor::NodeStatus;
use crate::engine::executor::RunId;
use crate::engine::executor::StepResult;
use crate::engine::workflow::Workflow;
use crate::store::RunRecord;
use crate::store::Store;
use crate::store::StoreError;
use crate::triggers;
use crate::triggers::TriggerRunPlan;
use crate::workflow_validation::WorkflowDiagnostic;

mod shell;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HeadlessRunStatus {
    Succeeded,
    Suspended,
    Failed,
}

impl HeadlessRunStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Succeeded => "succeeded",
            Self::Suspended => "suspended",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeadlessProcessResult {
    Success,
    Failure,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeadlessRunSummary {
    pub run_id: RunId,
    pub workflow_name: String,
    pub status: HeadlessRunStatus,
    pub node_count: usize,
    pub steps_run: usize,
    pub completed: usize,
    pub failures: usize,
    pub cache_hits: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeadlessRunError {
    Store(String),
    WorkflowNotFound(String),
    Validation(Vec<WorkflowDiagnostic>),
}

impl std::fmt::Display for HeadlessRunError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Store(message) => write!(f, "store error: {message}"),
            Self::WorkflowNotFound(name) => write!(f, "workflow not found: {name}"),
            Self::Validation(diagnostics) => {
                write!(f, "workflow validation failed with {} fatal diagnostics", diagnostics.len())
            }
        }
    }
}

impl std::error::Error for HeadlessRunError {}

impl From<StoreError> for HeadlessRunError {
    fn from(error: StoreError) -> Self {
        Self::Store(error.to_string())
    }
}

// r[impl n8nclone.workflow_execution.headless.summary]
pub fn summarize_run(run_id: RunId, workflow: &Workflow, results: &[StepResult]) -> HeadlessRunSummary {
    let node_count = workflow.graph.node_count();
    let steps_run = results.len();
    let completed = results.iter().filter(|step| matches!(step.status, NodeStatus::Completed { .. })).count();
    let explicit_failures = results.iter().filter(|step| matches!(step.status, NodeStatus::Failed { .. })).count();
    let incomplete_nodes = node_count.saturating_sub(steps_run);
    let failures = explicit_failures + incomplete_nodes;
    let cache_hits = results
        .iter()
        .filter(|step| matches!(step.status, NodeStatus::Completed { cached: true, .. }))
        .count();
    let status = if failures == 0 {
        HeadlessRunStatus::Succeeded
    } else {
        HeadlessRunStatus::Failed
    };

    HeadlessRunSummary {
        run_id,
        workflow_name: workflow.meta.name.clone(),
        status,
        node_count,
        steps_run,
        completed,
        failures,
        cache_hits,
    }
}

// r[impl n8nclone.workflow_execution.headless.summary]
pub fn render_run_summary_text(summary: &HeadlessRunSummary) -> String {
    format!(
        "{} | {} | {} nodes | {} steps | {} failures | {} cache hits",
        summary.run_id,
        summary.status.as_str(),
        summary.node_count,
        summary.steps_run,
        summary.failures,
        summary.cache_hits,
    )
}

// r[impl n8nclone.workflow_execution.headless.exit_status]
pub fn process_result_for_summary(summary: &HeadlessRunSummary) -> HeadlessProcessResult {
    match summary.status {
        HeadlessRunStatus::Succeeded | HeadlessRunStatus::Suspended => HeadlessProcessResult::Success,
        HeadlessRunStatus::Failed => HeadlessProcessResult::Failure,
    }
}

// r[impl n8nclone.workflow_execution.headless.history]
pub fn run_record_from_summary(
    summary: &HeadlessRunSummary,
    started_epoch_ms: i64,
    finished_epoch_ms: Option<i64>,
) -> RunRecord {
    RunRecord {
        run_id: summary.run_id,
        owner: crate::store::RunOwner::Unscoped,
        workflow_name: summary.workflow_name.clone(),
        started_epoch_ms,
        finished_epoch_ms,
        node_count: summary.node_count,
        cache_hits: summary.cache_hits,
        failures: summary.failures,
    }
}

fn run_started_epoch_ms() -> i64 {
    let started: chrono::DateTime<chrono::Utc> = chrono::Utc::now();
    started.timestamp_millis()
}

fn run_finished_epoch_ms() -> i64 {
    let finished: chrono::DateTime<chrono::Utc> = chrono::Utc::now();
    finished.timestamp_millis()
}

pub(crate) fn persist_completed_outputs_to_cache(store: &impl crate::store::ExecutionCache, results: &[StepResult]) {
    for step in results {
        if let NodeStatus::Completed { outputs, .. } = &step.status {
            let _ = store.put_cache(&step.input_hash, outputs);
        }
    }
}

// r[impl n8nclone.workflow_execution.headless.run]
// r[impl n8nclone.workflow_execution.headless.history]
// r[impl n8nclone.workflow_execution.headless.errors]
pub async fn run_saved_workflow(
    store: &Store,
    workflow_name: &str,
    run_id: RunId,
) -> Result<HeadlessRunSummary, HeadlessRunError> {
    let authority_provider = crate::engine::effect::execution::UnavailableRunContextSource;
    shell::run_saved_workflow_with_executor_factory(
        shell::ProductRunRequest {
            store,
            workflow_name,
            run_id,
            owner: crate::store::RunOwner::Unscoped,
            authority_provider: &authority_provider,
        },
        Executor::new,
    )
    .await
}

pub async fn run_saved_workflow_with_authority_provider(
    store: &Store,
    workflow_name: &str,
    run_id: RunId,
    authority_provider: &dyn crate::engine::effect::execution::RunContextProvider,
) -> Result<HeadlessRunSummary, HeadlessRunError> {
    shell::run_saved_workflow_with_executor_factory(
        shell::ProductRunRequest {
            store,
            workflow_name,
            run_id,
            owner: crate::store::RunOwner::Unscoped,
            authority_provider,
        },
        Executor::new,
    )
    .await
}

// r[impl n8nclone.scheduling.webhook_listener.admission.valid]
// r[impl n8nclone.scheduling.triggers.history]
pub async fn run_admitted_trigger_plan(
    store: &Store,
    plan: &TriggerRunPlan,
) -> Result<HeadlessRunSummary, HeadlessRunError> {
    let authority_provider = crate::engine::effect::execution::UnavailableRunContextSource;
    run_admitted_trigger_plan_with_authority_provider(store, plan, &authority_provider).await
}

pub async fn run_admitted_trigger_plan_with_authority_provider(
    store: &Store,
    plan: &TriggerRunPlan,
    authority_provider: &dyn crate::engine::effect::execution::RunContextProvider,
) -> Result<HeadlessRunSummary, HeadlessRunError> {
    let run_id = store.next_run_id()?;
    let summary =
        run_saved_workflow_with_authority_provider(store, &plan.workflow_name, run_id, authority_provider).await?;
    let record = triggers::trigger_run_record_started(plan, summary.run_id);
    store.save_trigger_run(&record)?;
    Ok(summary)
}

// r[impl lattice.scheduling.webhook_atomic_idempotency]
pub async fn run_claimed_trigger_plan(
    store: &Store,
    plan: &TriggerRunPlan,
    run_id: RunId,
) -> Result<HeadlessRunSummary, HeadlessRunError> {
    let authority_provider = crate::engine::effect::execution::UnavailableRunContextSource;
    run_claimed_trigger_plan_with_authority_provider(store, plan, run_id, &authority_provider).await
}

pub async fn run_claimed_trigger_plan_with_authority_provider(
    store: &Store,
    plan: &TriggerRunPlan,
    run_id: RunId,
    authority_provider: &dyn crate::engine::effect::execution::RunContextProvider,
) -> Result<HeadlessRunSummary, HeadlessRunError> {
    run_saved_workflow_with_authority_provider(store, &plan.workflow_name, run_id, authority_provider).await
}

// r[impl n8nclone.workflow_execution.agent_nodes.production_adapters.default_executor]
pub async fn run_saved_workflow_with_agent_runtime_config(
    store: &Store,
    workflow_name: &str,
    run_id: RunId,
    agent_runtime: AgentRuntimeConfig,
) -> Result<HeadlessRunSummary, HeadlessRunError> {
    let authority_provider = crate::engine::effect::execution::UnavailableRunContextSource;
    shell::run_saved_workflow_with_executor_factory(
        shell::ProductRunRequest {
            store,
            workflow_name,
            run_id,
            owner: crate::store::RunOwner::Unscoped,
            authority_provider: &authority_provider,
        },
        || Executor::new_with_agent_runtime_config(agent_runtime.clone()),
    )
    .await
}

pub use shell::OwnedRunRequest;
pub use shell::SnapshotRunRequest;
pub use shell::run_owned_workflow;
pub use shell::run_owned_workflow_with_authority_provider;
pub use shell::run_owned_workflow_with_authority_provider_and_cancellation;
pub use shell::run_saved_workflow_with_app_config;
pub use shell::run_workflow_snapshot_with_authority_provider_and_cancellation;

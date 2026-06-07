//! Headless workflow execution support.
//!
//! The pure core in this module summarizes executor results and renders stable
//! output. The shell function loads workflows, invokes the executor, and persists
//! run records.

use std::collections::HashMap;

use serde::Deserialize;
use serde::Serialize;

use crate::agent_node::AgentRuntimeConfig;
use crate::engine::executor::Executor;
use crate::engine::executor::NodeStatus;
use crate::engine::executor::RunId;
use crate::engine::executor::StepResult;
use crate::engine::workflow::Workflow;
use crate::replay;
use crate::store::ExecutionCache;
use crate::store::RunHistoryStore;
use crate::store::RunRecord;
use crate::store::Store;
use crate::store::StoreError;
use crate::store::WorkflowRepository;
use crate::workflow_validation;
use crate::workflow_validation::WorkflowDiagnostic;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HeadlessRunStatus {
    Succeeded,
    Failed,
}

impl HeadlessRunStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Succeeded => "succeeded",
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
        HeadlessRunStatus::Succeeded => HeadlessProcessResult::Success,
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
        workflow_name: summary.workflow_name.clone(),
        started_epoch_ms,
        finished_epoch_ms,
        node_count: summary.node_count,
        cache_hits: summary.cache_hits,
        failures: summary.failures,
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
    run_saved_workflow_with_executor_factory(store, workflow_name, run_id, Executor::new).await
}

// r[impl n8nclone.workflow_execution.agent_nodes.production_adapters.default_executor]
pub async fn run_saved_workflow_with_agent_runtime_config(
    store: &Store,
    workflow_name: &str,
    run_id: RunId,
    agent_runtime: AgentRuntimeConfig,
) -> Result<HeadlessRunSummary, HeadlessRunError> {
    run_saved_workflow_with_executor_factory(store, workflow_name, run_id, || {
        Executor::new_with_agent_runtime_config(agent_runtime.clone())
    })
    .await
}

// r[impl n8nclone.workflow_execution.trait_boundaries.storage_ports]
async fn run_saved_workflow_with_executor_factory<S>(
    store: &S,
    workflow_name: &str,
    run_id: RunId,
    executor_factory: impl FnOnce() -> Executor,
) -> Result<HeadlessRunSummary, HeadlessRunError>
where
    S: WorkflowRepository + ExecutionCache + RunHistoryStore,
{
    let workflow = store
        .load_workflow(workflow_name)?
        .ok_or_else(|| HeadlessRunError::WorkflowNotFound(workflow_name.to_string()))?;
    let diagnostics = workflow_validation::validate_workflow(&workflow);
    if workflow_validation::has_fatal_diagnostics(&diagnostics) {
        return Err(HeadlessRunError::Validation(workflow_validation::fatal_diagnostics(&diagnostics)));
    }

    let started_epoch_ms = chrono::Utc::now().timestamp_millis();
    let mut executor = executor_factory();
    let cache = HashMap::new();
    let (_executor_run_id, results) = executor.run(&workflow, &cache).await;
    persist_completed_outputs_to_cache(store, &results);

    let summary = summarize_run(run_id, &workflow, &results);
    let finished_epoch_ms = chrono::Utc::now().timestamp_millis();
    let record = run_record_from_summary(&summary, started_epoch_ms, Some(finished_epoch_ms));
    store.save_run(&record)?;
    let replay_events =
        replay::replay_events_from_results(run_id, &workflow, &results, started_epoch_ms, Some(finished_epoch_ms));
    store.save_run_replay_events(run_id, &replay_events)?;

    Ok(summary)
}

fn persist_completed_outputs_to_cache(store: &impl ExecutionCache, results: &[StepResult]) {
    for step in results {
        if let NodeStatus::Completed { outputs, .. } = &step.status {
            let _ = store.put_cache(&step.input_hash, outputs);
        }
    }
}

#![allow(
    unknown_lints,
    assertion_density,
    non_trait_imports,
    path_segment_repetition,
    reason = "the worker shell checks durable lifecycle boundaries before and after the existing headless runtime"
)]

use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use lattice_workflow_contract::Identity;

use super::ApplicationError;
use super::CompletionFacts;
use super::DurableNodeResult;
use super::EffectAuthorityMode;
use super::ExecutionRecord;
use super::WorkflowFacts;

const SUCCESSFUL_SHELL_EXIT_TEXT: &str = "exit code: 0";

pub(super) struct WorkerInput {
    pub store: crate::store::Store,
    pub admitted: ExecutionRecord,
    pub authority_mode: EffectAuthorityMode,
    pub workflow: crate::engine::workflow::Workflow,
    pub agent_runtime: crate::agent_node::AgentRuntimeConfig,
    pub cancellation: Arc<AtomicBool>,
}

pub(super) fn run(input: WorkerInput) {
    let WorkerInput {
        store,
        admitted,
        authority_mode,
        workflow,
        agent_runtime,
        cancellation,
    } = input;
    let Ok(running) = super::mark_running(&admitted) else {
        return;
    };
    if store.save_workflow_exchange_transition(&admitted, &running).is_err() {
        return;
    }
    let authority: Box<dyn crate::engine::effect::execution::RunContextProvider> = match authority_mode {
        EffectAuthorityMode::Unavailable => Box::new(crate::engine::effect::execution::UnavailableRunContextSource),
        EffectAuthorityMode::RouteExact => Box::new(super::RouteExactAuthorityProvider::new(
            &running.handler_profile_identity,
            &running.request.identity,
            &running.workflow_revision,
            crate::engine::RunId(running.run_id),
        )),
    };
    let completion_workflow = workflow.clone();
    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build();
    let result = match runtime {
        Ok(runtime) => {
            runtime.block_on(crate::headless::run_workflow_snapshot_with_authority_provider_and_cancellation(
                &store,
                crate::headless::SnapshotRunRequest {
                    workflow,
                    run_id: crate::engine::RunId(running.run_id),
                    owner: crate::store::RunOwner::Unscoped,
                    agent_runtime,
                    cancellation: Arc::clone(&cancellation),
                },
                authority.as_ref(),
            ))
        }
        Err(_) => return mark_unknown(&store, &running),
    };
    let current = match store.load_workflow_exchange(&running.accepted.execution_identity) {
        Ok(Some(current)) => current,
        _ => return,
    };
    let facts = completion_facts(&store, &current, &completion_workflow, result);
    let next = match super::complete(&current, &facts) {
        Ok(next) => next,
        Err(_) => return mark_unknown(&store, &current),
    };
    if store.save_workflow_exchange_transition(&current, &next).is_err() {
        mark_unknown(&store, &current);
    }
}

fn completion_facts(
    store: &crate::store::Store,
    record: &ExecutionRecord,
    workflow: &crate::engine::workflow::Workflow,
    result: Result<crate::headless::HeadlessRunSummary, crate::headless::HeadlessRunError>,
) -> CompletionFacts {
    let Ok(summary) = result else {
        return CompletionFacts {
            cancelled: false,
            suspended: false,
            failed_before_results: true,
            nodes: Vec::new(),
        };
    };
    let details = store.load_run_details(crate::engine::RunId(record.run_id)).ok().flatten();
    let nodes = details
        .map(|details| {
            details
                .nodes
                .into_iter()
                .filter_map(|node| {
                    let result = match node.status {
                        crate::store::RunNodeDetailStatus::Completed => completed_node_result(workflow, &node),
                        crate::store::RunNodeDetailStatus::Failed => DurableNodeResult::Failed,
                        crate::store::RunNodeDetailStatus::Skipped => DurableNodeResult::Skipped,
                        crate::store::RunNodeDetailStatus::Pending | crate::store::RunNodeDetailStatus::Running => {
                            return None;
                        }
                    };
                    Some((node.node_id.0, result))
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    CompletionFacts {
        cancelled: false,
        suspended: summary.status == crate::headless::HeadlessRunStatus::Suspended,
        failed_before_results: nodes.is_empty() && summary.node_count != 0,
        nodes,
    }
}

fn completed_node_result(
    workflow: &crate::engine::workflow::Workflow,
    node: &crate::store::RunNodeDetail,
) -> DurableNodeResult {
    let Some(graph_node) = workflow.graph.node(node.node_id) else {
        return DurableNodeResult::Failed;
    };
    let mut exit_ports = graph_node
        .output_ports
        .iter()
        .filter(|port| port.type_tag == crate::engine::TypeTag::ExitCode.as_str());
    let Some(exit_port) = exit_ports.next() else {
        return DurableNodeResult::Completed;
    };
    if exit_ports.next().is_some() {
        return DurableNodeResult::Failed;
    }
    let mut exit_outputs = node.outputs.iter().filter(|output| output.port_id == exit_port.id);
    let Some(exit) = exit_outputs.next() else {
        return DurableNodeResult::Failed;
    };
    if exit.text != SUCCESSFUL_SHELL_EXIT_TEXT || exit_outputs.next().is_some() {
        return DurableNodeResult::Failed;
    }
    DurableNodeResult::Completed
}

fn mark_unknown(store: &crate::store::Store, current: &ExecutionRecord) {
    let Ok(latest) = store.load_workflow_exchange(&current.accepted.execution_identity) else {
        return;
    };
    let Some(latest) = latest else {
        return;
    };
    let Ok(unknown) = super::mark_unknown(&latest) else {
        return;
    };
    if store.save_workflow_exchange_transition(&latest, &unknown).is_err() {
        tracing::warn!("workflow exchange worker could not persist unknown state");
    }
}

pub(in crate::workflow_exchange) fn workflow_facts(
    workflow: &crate::engine::workflow::Workflow,
) -> Result<WorkflowFacts, ApplicationError> {
    let revision = crate::orchestration::workflow_revision(workflow).map_err(|_| ApplicationError::Workflow)?;
    let revision = Identity::new(format!("b3:{}", revision.content_hash)).map_err(|_| ApplicationError::Workflow)?;
    let node_ids = workflow.graph.node_ids().into_iter().map(|node| node.0).collect::<Vec<_>>();
    let mut edges = workflow
        .graph
        .edges()
        .iter()
        .map(|edge| {
            let source = workflow.graph.port_owner(edge.source).ok_or(ApplicationError::Workflow)?;
            let target = workflow.graph.port_owner(edge.target).ok_or(ApplicationError::Workflow)?;
            Ok((source.0, target.0))
        })
        .collect::<Result<Vec<_>, ApplicationError>>()?;
    edges.sort_unstable();
    edges.dedup();
    Ok(WorkflowFacts {
        revision,
        node_ids,
        edges,
    })
}

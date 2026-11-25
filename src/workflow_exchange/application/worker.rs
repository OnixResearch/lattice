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

mod records;
mod snapshot;

use records::Records;

pub(super) struct WorkerInput {
    pub store: crate::store::Store,
    pub admitted: ExecutionRecord,
    pub authority_mode: EffectAuthorityMode,
    pub workflow: crate::engine::workflow::Workflow,
    pub agent_runtime: crate::agent_node::RuntimeOptions,
    pub app_config: Option<crate::config::AppConfig>,
    pub cancellation: Arc<AtomicBool>,
    /// The route's checkpoint binding; `None` or a `none` profile keeps the hook absent.
    pub checkpoint: Option<crate::checkpoint::CheckpointBinding>,
    /// Carried incarnation, budget and completed nodes of an adopted checkpoint.
    pub seed: crate::checkpoint::HookSeed,
    /// Terminal steps of an adopted checkpoint; restored, never re-run.
    pub restored: Vec<crate::engine::executor::StepResult>,
}

pub(super) fn run(input: WorkerInput) {
    let WorkerInput {
        store,
        admitted,
        authority_mode,
        workflow,
        agent_runtime,
        app_config,
        cancellation,
        checkpoint,
        seed,
        restored,
    } = input;
    let Some(running) = start_running(&store, &admitted) else {
        return;
    };
    // r[impl lattice.application_checkpoint.cancellation]
    // A run cancelled before it starts settles `cancelled` unrun: the ready frontier would fail it.
    if running.cancellation_requested || cancellation.load(std::sync::atomic::Ordering::Acquire) {
        let cancelled = CompletionFacts {
            cancelled: true,
            ..CompletionFacts::default()
        };
        return settle(&store, &running, &cancelled);
    }
    // r[impl lattice.application_checkpoint.hook]
    let Ok(hook) = crate::checkpoint::NodeBoundaryHook::for_route(&store, checkpoint.as_ref(), &running, seed) else {
        return mark_unknown(&store, &running);
    };
    let Some(wiring) = wire_checkpoint(&running, &workflow, authority_mode, hook) else {
        return mark_unknown(&store, &running);
    };
    let Some(result) = snapshot::run_snapshot(snapshot::SnapshotDispatch {
        store: &store,
        workflow,
        run_id: crate::engine::RunId(running.run_id),
        agent_runtime,
        app_config,
        cancellation: Arc::clone(&cancellation),
        authority: wiring.authority.as_ref(),
        restored,
        progress: wiring.progress,
    }) else {
        return mark_unknown(&store, &running);
    };
    if let Some(observer) = wiring.observer
        && observer.join().is_err()
    {
        tracing::warn!("application checkpoint observer stopped abnormally");
    }
    let facts = completion_facts(&store, &running, result);
    settle(&store, &running, &facts);
}

/// Mark `admitted` running. A Cancel that lands while it is still `Queued` fails this
/// compare-and-swap; while the reloaded record is a newer `Queued` one with
/// `cancellation_requested`, it is marked running instead, however many Cancels race the
/// transition, so the caller settles it `cancelled` rather than leaving it for `Unknown`.
fn start_running(records: &impl Records, admitted: &ExecutionRecord) -> Option<ExecutionRecord> {
    let mut current = admitted.clone();
    loop {
        let running = super::mark_running(&current).ok()?;
        if records.swap(&current, &running) {
            return Some(running);
        }
        current = reloaded_after_cancel(records, &current, super::DurableExecutionState::Queued)?;
    }
}

/// Record the run's terminal facts against the latest record, or `Unknown` if they do not apply.
/// A compare-and-swap lost to a Cancel is retried against the reloaded `Running` record.
fn settle(records: &impl Records, running: &ExecutionRecord, facts: &CompletionFacts) {
    let Some(mut current) = records.load(&running.accepted.execution_identity) else {
        return;
    };
    loop {
        let Ok(next) = super::complete(&current, facts) else {
            return mark_unknown(records, &current);
        };
        if records.swap(&current, &next) {
            return;
        }
        match reloaded_after_cancel(records, &current, super::DurableExecutionState::Running) {
            Some(reloaded) => current = reloaded,
            None => return mark_unknown(records, &current),
        }
    }
}

/// The stored record when a Cancel replaced `stale` since it was read: still in `state`, with a
/// later sequence and `cancellation_requested`. Each one needs another Cancel, so retries end.
fn reloaded_after_cancel(
    records: &impl Records,
    stale: &ExecutionRecord,
    state: super::DurableExecutionState,
) -> Option<ExecutionRecord> {
    records
        .load(&stale.accepted.execution_identity)
        .filter(|reloaded| reloaded.state == state && reloaded.cancellation_requested)
        .filter(|reloaded| reloaded.sequence > stale.sequence)
}

/// The run's authority source and, for a checkpointed route, its hook's progress wiring.
struct CheckpointWiring {
    authority: Box<dyn crate::engine::effect::execution::RunContextProvider>,
    progress: Option<crate::engine::executor::progress::Sender>,
    observer: Option<std::thread::JoinHandle<()>>,
}

/// `None` when the route's checkpoint hook cannot be observed.
fn wire_checkpoint(
    running: &ExecutionRecord,
    workflow: &crate::engine::workflow::Workflow,
    authority_mode: EffectAuthorityMode,
    hook: Option<Arc<crate::checkpoint::NodeBoundaryHook>>,
) -> Option<CheckpointWiring> {
    let (progress, observer) = match &hook {
        Some(hook) => {
            let (sender, receiver) = crate::engine::executor::progress::channel(workflow).ok()?;
            (Some(sender), Some(hook.observe_progress(receiver).ok()?))
        }
        None => (None, None),
    };
    let authority: Box<dyn crate::engine::effect::execution::RunContextProvider> = match authority_mode {
        EffectAuthorityMode::Unavailable => Box::new(crate::engine::effect::execution::UnavailableRunContextSource),
        EffectAuthorityMode::RouteExact => {
            let provider = super::RouteExactAuthorityProvider::new(
                &running.handler_profile_identity,
                &running.request.identity,
                &running.workflow_revision,
                crate::engine::RunId(running.run_id),
            );
            Box::new(match &hook {
                Some(hook) => provider.with_reservation_fence(Arc::clone(hook) as Arc<dyn super::ReservationFence>),
                None => provider,
            })
        }
    };
    Some(CheckpointWiring {
        authority,
        progress,
        observer,
    })
}

fn completion_facts(
    store: &crate::store::Store,
    record: &ExecutionRecord,
    result: Result<crate::headless::SnapshotRunOutcome, crate::headless::HeadlessRunError>,
) -> CompletionFacts {
    let Ok(outcome) = result else {
        return CompletionFacts {
            cancelled: false,
            suspended: false,
            failed_before_results: true,
            nodes: Vec::new(),
        };
    };
    let summary = outcome.summary;
    let details = store.load_run_details(crate::engine::RunId(record.run_id)).ok().flatten();
    let is_cancellation_stall = details.as_ref().is_some_and(|details| stalled_on_cancellation(&details.nodes));
    let nodes = details
        .map(|details| {
            details
                .nodes
                .into_iter()
                .filter_map(|node| {
                    let result = match node.status {
                        crate::store::RunNodeDetailStatus::Completed => DurableNodeResult::Completed,
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
        cancelled: outcome.cancelled || is_cancellation_stall,
        suspended: summary.status == crate::headless::HeadlessRunStatus::Suspended,
        failed_before_results: nodes.is_empty() && summary.node_count != 0,
        nodes,
    }
}

/// The ready frontier stalled on cancellation before it dispatched anything: every node carries
/// the cancellation stall diagnostic, so no node ran and no effect was dispatched. A run with any
/// other node result, such as a completed process, is never relabelled.
fn stalled_on_cancellation(nodes: &[crate::store::RunNodeDetail]) -> bool {
    !nodes.is_empty()
        && nodes.iter().all(|node| {
            node.status == crate::store::RunNodeDetailStatus::Failed
                && node.error.as_deref().is_some_and(crate::engine::frontier::is_cancellation_stall)
        })
}

fn mark_unknown(records: &impl Records, current: &ExecutionRecord) {
    let Some(latest) = records.load(&current.accepted.execution_identity) else {
        return;
    };
    let Ok(unknown) = super::mark_unknown(&latest) else {
        return;
    };
    if !records.swap(&latest, &unknown) {
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

#[cfg(test)]
mod tests;

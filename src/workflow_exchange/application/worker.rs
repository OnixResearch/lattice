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

pub(super) struct WorkerInput {
    pub store: crate::store::Store,
    pub admitted: ExecutionRecord,
    pub authority_mode: EffectAuthorityMode,
    pub workflow: crate::engine::workflow::Workflow,
    pub agent_runtime: crate::agent_node::RuntimeOptions,
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
        cancellation,
        checkpoint,
        seed,
        restored,
    } = input;
    let Ok(running) = super::mark_running(&admitted) else {
        return;
    };
    if store.save_workflow_exchange_transition(&admitted, &running).is_err() {
        return;
    }
    // r[impl lattice.application_checkpoint.cancellation]
    // A run cancelled before it starts settles `cancelled` unrun: the ready frontier would fail it.
    if cancellation.load(std::sync::atomic::Ordering::Acquire) {
        return settle(&store, &running, |_current| CompletionFacts {
            cancelled: true,
            ..CompletionFacts::default()
        });
    }
    // r[impl lattice.application_checkpoint.hook]
    let Ok(hook) = crate::checkpoint::NodeBoundaryHook::for_route(&store, checkpoint.as_ref(), &running, seed) else {
        return mark_unknown(&store, &running);
    };
    let Some(wiring) = wire_checkpoint(&running, &workflow, authority_mode, hook) else {
        return mark_unknown(&store, &running);
    };
    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build();
    let result = match runtime {
        Ok(runtime) => runtime.block_on(
            crate::headless::SnapshotRunRequest {
                workflow,
                run_id: crate::engine::RunId(running.run_id),
                owner: crate::store::RunOwner::Unscoped,
                agent_runtime,
                cancellation: Arc::clone(&cancellation),
            }
            .run_from_checkpoint(&store, wiring.authority.as_ref(), restored, wiring.progress),
        ),
        Err(_) => return mark_unknown(&store, &running),
    };
    if let Some(observer) = wiring.observer
        && observer.join().is_err()
    {
        tracing::warn!("application checkpoint observer stopped abnormally");
    }
    settle(&store, &running, |current| completion_facts(&store, current, result));
}

/// Record the run's terminal facts against the latest record, or `Unknown` if they do not apply.
fn settle(
    store: &crate::store::Store,
    running: &ExecutionRecord,
    facts: impl FnOnce(&ExecutionRecord) -> CompletionFacts,
) {
    let current = match store.load_workflow_exchange(&running.accepted.execution_identity) {
        Ok(Some(current)) => current,
        _ => return,
    };
    let next = match super::complete(&current, &facts(&current)) {
        Ok(next) => next,
        Err(_) => return mark_unknown(store, &current),
    };
    if store.save_workflow_exchange_transition(&current, &next).is_err() {
        mark_unknown(store, &current);
    }
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
        cancelled: outcome.cancelled,
        suspended: summary.status == crate::headless::HeadlessRunStatus::Suspended,
        failed_before_results: nodes.is_empty() && summary.node_count != 0,
        nodes,
    }
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

#[cfg(test)]
mod tests {
    #[test]
    fn real_completion_wins_late_durable_and_atomic_cancel_intent() {
        for (command, expected) in [
            ("printf completed", crate::workflow_exchange::DurableExecutionState::Completed),
            ("exit 23", crate::workflow_exchange::DurableExecutionState::Failed),
        ] {
            for late_cancel in [false, true] {
                let (_directory, store, workflow, _contract, request, profile) =
                    crate::workflow_exchange::tests::fixture(command);
                let route = &profile.routes[0];
                let handler_identity = crate::workflow_exchange::handler_profile_identity(&profile).unwrap();
                let admission = store
                    .admit_workflow_exchange(request, route, route.workflow_revision.clone(), handler_identity)
                    .unwrap();
                let crate::workflow_exchange::AdmissionResolution::Inserted(admitted) = admission else {
                    panic!("fresh fixture must admit once");
                };
                let running = crate::workflow_exchange::mark_running(&admitted).unwrap();
                store.save_workflow_exchange_transition(&admitted, &running).unwrap();
                let authority = crate::workflow_exchange::RouteExactAuthorityProvider::new(
                    &running.handler_profile_identity,
                    &running.request.identity,
                    &running.workflow_revision,
                    crate::engine::RunId(running.run_id),
                );
                let cancellation = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
                let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
                let outcome = runtime
                    .block_on(crate::headless::run_workflow_snapshot_with_authority_provider_and_cancellation(
                        &store,
                        crate::headless::SnapshotRunRequest {
                            workflow,
                            run_id: crate::engine::RunId(running.run_id),
                            owner: crate::store::RunOwner::Unscoped,
                            agent_runtime: crate::agent_node::RuntimeOptions::default(),
                            cancellation: std::sync::Arc::clone(&cancellation),
                        },
                        &authority,
                    ))
                    .expect("real process completion is retained");
                assert!(!outcome.cancelled, "a completed process is not cancelled");
                let current = if late_cancel {
                    cancellation.store(true, std::sync::atomic::Ordering::Release);
                    let requested = crate::workflow_exchange::request_cancellation(&running, running.sequence).unwrap();
                    store.save_workflow_exchange_transition(&running, &requested).unwrap();
                    requested
                } else {
                    running
                };
                let facts = super::completion_facts(&store, &current, Ok(outcome));
                let completed = crate::workflow_exchange::complete(&current, &facts).unwrap();
                store.save_workflow_exchange_transition(&current, &completed).unwrap();
                let durable = store.load_workflow_exchange(&completed.accepted.execution_identity).unwrap().unwrap();
                assert_eq!(durable.state, expected, "late intent must not replace the observed process outcome");
                let expected_node = if expected == crate::workflow_exchange::DurableExecutionState::Completed {
                    lattice_workflow_contract::NodeState::Completed
                } else {
                    lattice_workflow_contract::NodeState::Failed
                };
                assert_eq!(durable.nodes[0].state, expected_node);
            }
        }
    }
}

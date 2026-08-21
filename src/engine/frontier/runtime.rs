//! Event-driven ready-frontier dispatch for ordinary workflow runs.
#![allow(
    unknown_lints,
    function_length,
    non_trait_imports,
    unbounded_collection_growth,
    reason = "the explicit event loop keeps readiness, bounded reservation, dispatch, and first-completion reduction in one auditable shell"
)]

mod support;

use std::collections::BTreeMap;
use std::collections::HashMap;

use futures::StreamExt;
pub(crate) use support::RunRequest;
use support::*;

use crate::engine::executor::EffectExecution;
use crate::engine::executor::NodeExecution;
use crate::engine::executor::NodeStatus;
use crate::engine::executor::StepResult;
use crate::engine::runner;
use crate::engine::workflow::NodeId;

// r[impl lattice.workflow_execution.ready_frontier.dispatch]
// r[impl lattice.workflow_execution.ready_frontier.completion]
pub(crate) async fn run(request: RunRequest<'_>) -> RunOutcome {
    let RunRequest {
        executor,
        workflow,
        cache,
        bound_inputs,
        run_id,
        runner_plan,
        canonical_order,
        product_session,
    } = request;
    let maximum_step_count = canonical_order.len();
    let mut results_by_node = BTreeMap::new();
    let mut observed_completion_order = Vec::with_capacity(maximum_step_count);
    let mut port_values = HashMap::with_capacity(maximum_step_count);
    let mut runner_states = BTreeMap::new();
    let mut running = BTreeMap::new();
    let mut in_flight = futures::stream::FuturesUnordered::new();

    loop {
        let ready = match ready_frontier(runner_plan, &runner_states, product_session.as_ref()) {
            Ok(ready) => ready,
            Err(diagnostic) => {
                fail_unfinished(
                    workflow,
                    canonical_order,
                    &mut runner_states,
                    &mut results_by_node,
                    &mut observed_completion_order,
                    product_session.as_ref(),
                    diagnostic,
                );
                break;
            }
        };
        if ready.is_empty() && running.is_empty() {
            break;
        }

        let candidates = ready
            .iter()
            .map(|node_id| {
                let mut candidate = super::candidate(node_id.0, workflow.node_config(*node_id), super::ProductFacts {
                    is_suspended: false,
                    is_cancelled: executor.cancellation_requested(),
                });
                if let Some(conflict) = executor.frontier_conflict(*node_id) {
                    candidate.conflict = conflict.clone();
                }
                candidate
            })
            .collect::<Vec<_>>();
        let active = super::active_reservations(&running);
        let plan = match lattice_core::scheduling::plan(&executor.frontier_policy, &active, &candidates) {
            Ok(plan) => plan,
            Err(error) => {
                let diagnostic = format!("{FRONTIER_STALLED}: {error:?}");
                fail_unfinished(
                    workflow,
                    canonical_order,
                    &mut runner_states,
                    &mut results_by_node,
                    &mut observed_completion_order,
                    product_session.as_ref(),
                    &diagnostic,
                );
                break;
            }
        };

        let mut dispatched_count = 0_u32;
        for selected in plan.selected_nodes {
            let node_id = NodeId(selected);
            let mut prepared = match prepare_execution(workflow, runner_plan, node_id, &port_values, bound_inputs) {
                Ok(prepared) => prepared,
                Err(step) => {
                    let _was_prepared = start_failed_preparation(product_session.as_ref(), node_id).is_ok();
                    record_completion(
                        step,
                        &mut runner_states,
                        &mut port_values,
                        &mut results_by_node,
                        &mut observed_completion_order,
                        product_session.as_ref(),
                    );
                    dispatched_count = dispatched_count.saturating_add(1);
                    continue;
                }
            };
            if let Some(conflict) = executor.frontier_conflict(node_id) {
                prepared.reservation.conflict = conflict.clone();
            }
            if let Err(error) = mark_ready(product_session.as_ref(), node_id) {
                let step = crate::engine::executor::failed_product_step(
                    node_id,
                    prepared.input_hash,
                    &crate::engine::executor::choregraph_execution_error(error),
                );
                record_completion(
                    step,
                    &mut runner_states,
                    &mut port_values,
                    &mut results_by_node,
                    &mut observed_completion_order,
                    product_session.as_ref(),
                );
                dispatched_count = dispatched_count.saturating_add(1);
                continue;
            }

            if let Some(cached_outputs) = cache.get(&prepared.input_hash) {
                let step = cached_step(workflow, &prepared, cached_outputs, product_session.as_ref());
                record_completion(
                    step,
                    &mut runner_states,
                    &mut port_values,
                    &mut results_by_node,
                    &mut observed_completion_order,
                    product_session.as_ref(),
                );
                dispatched_count = dispatched_count.saturating_add(1);
                continue;
            }

            let has_effect =
                workflow.node_config(node_id).and_then(crate::engine::executor::effect_operation).is_some();
            if !has_effect && let Err(error) = start_pure(product_session.as_ref(), node_id) {
                let step = crate::engine::executor::failed_product_step(
                    node_id,
                    prepared.input_hash,
                    &crate::engine::executor::choregraph_execution_error(error),
                );
                record_completion(
                    step,
                    &mut runner_states,
                    &mut port_values,
                    &mut results_by_node,
                    &mut observed_completion_order,
                    product_session.as_ref(),
                );
                dispatched_count = dispatched_count.saturating_add(1);
                continue;
            }

            debug_assert!(running.len() < maximum_step_count);
            debug_assert!(runner_states.len() < maximum_step_count);
            running.insert(node_id.0, prepared.reservation.clone());
            runner_states.insert(node_id.0, runner::RunnerStepState::Running);
            let session = product_session.clone();
            in_flight.push(async move {
                let effect = match session.as_ref() {
                    Some(session) => EffectExecution::Product(session),
                    None => EffectExecution::Compatibility,
                };
                let step = executor
                    .execute_node_with_expression_context(NodeExecution {
                        workflow,
                        node_id: prepared.node_id,
                        inputs: &prepared.inputs,
                        expression_context: &prepared.expression_context,
                        input_hash: Some(prepared.input_hash),
                        parent_run_id: Some(run_id),
                        effect,
                    })
                    .await;
                (step, prepared.reservation)
            });
            dispatched_count = dispatched_count.saturating_add(1);
        }

        if let Some((step, reservation)) = in_flight.next().await {
            let removed = running.remove(&step.node_id.0);
            debug_assert_eq!(removed, Some(reservation));
            record_completion(
                step,
                &mut runner_states,
                &mut port_values,
                &mut results_by_node,
                &mut observed_completion_order,
                product_session.as_ref(),
            );
            continue;
        }

        if dispatched_count == 0 && !ready.is_empty() {
            let blocked = plan.blocked_nodes.first();
            let diagnostic = blocked.map_or_else(
                || FRONTIER_STALLED.to_owned(),
                |blocked| format!("{FRONTIER_STALLED}: {:?}", blocked.reason),
            );
            fail_unfinished(
                workflow,
                canonical_order,
                &mut runner_states,
                &mut results_by_node,
                &mut observed_completion_order,
                product_session.as_ref(),
                &diagnostic,
            );
            break;
        }
    }

    for node_id in canonical_order {
        if results_by_node.contains_key(&node_id.0) {
            continue;
        }
        let _was_cancelled = cancel(product_session.as_ref(), *node_id);
        let inputs = inputs_with_bindings(workflow, *node_id, &port_values, bound_inputs);
        let input_hash = base_input_hash(workflow, *node_id, &inputs);
        let step = StepResult {
            node_id: *node_id,
            status: NodeStatus::Skipped,
            input_hash,
        };
        runner_states.insert(node_id.0, runner::RunnerStepState::Cancelled);
        observed_completion_order.push(*node_id);
        results_by_node.insert(node_id.0, step);
    }

    let results = canonical_order.iter().filter_map(|node_id| results_by_node.remove(&node_id.0)).collect();
    RunOutcome {
        results,
        observed_completion_order,
    }
}

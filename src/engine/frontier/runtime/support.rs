//! Bounded preparation and completion helpers for the ready-frontier shell.
#![allow(
    unknown_lints,
    borrowed_argument_types,
    non_trait_imports,
    too_many_parameters,
    unbounded_collection_growth,
    reason = "all helper collections are bounded by the admitted workflow and explicit frontier policy; mutable vectors preserve owned capacity"
)]

use std::collections::BTreeMap;
use std::collections::HashMap;

use super::super::Reservation;
use crate::engine::executor::Executor;
use crate::engine::executor::NodeStatus;
use crate::engine::executor::RunId;
use crate::engine::executor::StepResult;
use crate::engine::runner;
use crate::engine::value::ContentHash;
use crate::engine::value::Value;
use crate::engine::workflow::NodeId;
use crate::engine::workflow::PortId;
use crate::engine::workflow::Workflow;
use crate::expressions::ExpressionContext;

pub(super) const FRONTIER_MISMATCH: &str = "Choregraph and compatibility readiness frontiers differ";
pub(super) const FRONTIER_STALLED: &str = "ready-frontier dispatch cannot make progress";

pub(crate) type SharedSession = std::sync::Arc<parking_lot::Mutex<crate::engine::effect::execution::Session>>;

pub(crate) struct RunOutcome {
    pub(crate) results: Vec<StepResult>,
    pub(crate) observed_completion_order: Vec<NodeId>,
}

pub(crate) struct RunRequest<'a> {
    pub(crate) executor: &'a Executor,
    pub(crate) workflow: &'a Workflow,
    pub(crate) cache: &'a HashMap<ContentHash, HashMap<PortId, Value>>,
    pub(crate) bound_inputs: &'a HashMap<PortId, Value>,
    pub(crate) run_id: RunId,
    pub(crate) runner_plan: &'a runner::RunnerWorkflow,
    pub(crate) canonical_order: &'a [NodeId],
    pub(crate) product_session: Option<SharedSession>,
}

pub(super) struct PreparedExecution {
    pub(super) node_id: NodeId,
    pub(super) inputs: HashMap<PortId, Value>,
    pub(super) expression_context: ExpressionContext,
    pub(super) input_hash: ContentHash,
    pub(super) reservation: Reservation,
}

pub(super) fn ready_frontier(
    runner_plan: &runner::RunnerWorkflow,
    runner_states: &BTreeMap<u64, runner::RunnerStepState>,
    product_session: Option<&SharedSession>,
) -> Result<Vec<NodeId>, &'static str> {
    let compatibility = runner::ready_nodes(runner_plan, runner_states).into_iter().map(NodeId).collect::<Vec<_>>();
    let Some(session) = product_session else {
        return Ok(compatibility);
    };
    let choregraph = session.lock().ready_frontier().map_err(|_| FRONTIER_MISMATCH)?;
    if choregraph != compatibility {
        return Err(FRONTIER_MISMATCH);
    }
    Ok(choregraph)
}

pub(super) fn prepare_execution(
    workflow: &Workflow,
    runner_plan: &runner::RunnerWorkflow,
    node_id: NodeId,
    port_values: &HashMap<PortId, Value>,
    bound_inputs: &HashMap<PortId, Value>,
) -> Result<PreparedExecution, StepResult> {
    let inputs = inputs_with_bindings(workflow, node_id, port_values, bound_inputs);
    let base_input_hash = base_input_hash(workflow, node_id, &inputs);
    let expression_values = crate::engine::executor::expression_values_for_consumer(runner_plan, node_id, port_values)
        .map_err(|error| crate::engine::executor::failed_product_step(node_id, base_input_hash, &error))?;
    let input_hash = match workflow.node_config(node_id) {
        Some(config) => crate::engine::executor::node_execution_cache_key_with_expression_values(
            node_id,
            config,
            workflow.node_credentials(node_id),
            &inputs,
            &expression_values,
        ),
        None => base_input_hash,
    };
    let prepared = PreparedExecution {
        node_id,
        inputs,
        expression_context: crate::engine::executor::expression_context_from_admitted_values(&expression_values),
        input_hash,
        reservation: super::super::reservation(workflow.node_config(node_id)),
    };
    debug_assert_eq!(prepared.node_id, node_id);
    debug_assert!(prepared.inputs.len() <= workflow.graph.edges().len().saturating_add(bound_inputs.len()));
    Ok(prepared)
}

pub(super) fn inputs_with_bindings(
    workflow: &Workflow,
    node_id: NodeId,
    port_values: &HashMap<PortId, Value>,
    bound_inputs: &HashMap<PortId, Value>,
) -> HashMap<PortId, Value> {
    let declared_inputs_len = workflow.graph.node(node_id).map_or(0, |node| node.input_ports.len());
    let mut inputs = crate::engine::executor::gather_inputs(workflow, node_id, port_values);
    inputs.reserve(declared_inputs_len.saturating_sub(inputs.len()));
    if let Some(node) = workflow.graph.node(node_id) {
        for port in &node.input_ports {
            if let Some(value) = bound_inputs.get(&port.id) {
                inputs.insert(port.id, value.clone());
            }
        }
    }
    inputs
}

pub(super) fn base_input_hash(workflow: &Workflow, node_id: NodeId, inputs: &HashMap<PortId, Value>) -> ContentHash {
    match workflow.node_config(node_id) {
        Some(config) => crate::engine::executor::node_execution_cache_key_with_credentials(
            node_id,
            config,
            workflow.node_credentials(node_id),
            inputs,
        ),
        None => crate::engine::executor::input_values_hash(inputs),
    }
}

pub(super) fn cached_step(
    workflow: &Workflow,
    prepared: &PreparedExecution,
    cached_outputs: &HashMap<PortId, Value>,
    product_session: Option<&SharedSession>,
) -> StepResult {
    if let Err(error) = start_pure(product_session, prepared.node_id) {
        return crate::engine::executor::failed_product_step(
            prepared.node_id,
            prepared.input_hash,
            &crate::engine::executor::choregraph_execution_error(error),
        );
    }
    let Some(node) = workflow.graph.node(prepared.node_id) else {
        return crate::engine::executor::failed_product_step(
            prepared.node_id,
            prepared.input_hash,
            "cached node is missing from workflow graph",
        );
    };
    debug_assert!(cached_outputs.len() <= node.output_ports.len());
    debug_assert!(cached_outputs.keys().all(|port_id| node.output_ports.iter().any(|port| port.id == *port_id)));
    crate::engine::executor::completed_step_result(
        prepared.node_id,
        prepared.input_hash,
        node,
        cached_outputs.clone(),
        true,
    )
}

pub(super) fn record_completion(
    mut step: StepResult,
    runner_states: &mut BTreeMap<u64, runner::RunnerStepState>,
    port_values: &mut HashMap<PortId, Value>,
    results_by_node: &mut BTreeMap<u64, StepResult>,
    observed_completion_order: &mut Vec<NodeId>,
    product_session: Option<&SharedSession>,
) {
    if let Some(session) = product_session
        && let Err(error) = session.lock().finish(step.node_id, &step.status)
    {
        step = crate::engine::executor::failed_product_step(
            step.node_id,
            step.input_hash,
            &crate::engine::executor::choregraph_execution_error(error),
        );
    }
    if let NodeStatus::Completed { outputs, .. } = &step.status {
        port_values.reserve(outputs.len());
        for (port_id, value) in outputs {
            port_values.insert(*port_id, value.clone());
        }
    }
    runner_states.insert(step.node_id.0, crate::engine::executor::runner_step_state(&step.status));
    observed_completion_order.push(step.node_id);
    let prior = results_by_node.insert(step.node_id.0, step);
    debug_assert!(prior.is_none());
    debug_assert!(results_by_node.len() <= runner_states.len());
}

pub(super) fn mark_ready(
    product_session: Option<&SharedSession>,
    node_id: NodeId,
) -> Result<(), crate::engine::effect::execution::Error> {
    match product_session {
        Some(session) => session.lock().mark_ready(node_id),
        None => Ok(()),
    }
}

pub(super) fn start_pure(
    product_session: Option<&SharedSession>,
    node_id: NodeId,
) -> Result<(), crate::engine::effect::execution::Error> {
    match product_session {
        Some(session) => session.lock().start_pure(node_id),
        None => Ok(()),
    }
}

pub(super) fn start_failed_preparation(
    product_session: Option<&SharedSession>,
    node_id: NodeId,
) -> Result<(), crate::engine::effect::execution::Error> {
    let Some(session) = product_session else {
        return Ok(());
    };
    let mut session = session.lock();
    session.mark_ready(node_id)?;
    session.start_pure(node_id)
}

pub(super) fn cancel(product_session: Option<&SharedSession>, node_id: NodeId) -> bool {
    product_session.is_none_or(|session| session.lock().cancel(node_id).is_ok())
}

pub(super) fn fail_unfinished(
    workflow: &Workflow,
    canonical_order: &[NodeId],
    runner_states: &mut BTreeMap<u64, runner::RunnerStepState>,
    results_by_node: &mut BTreeMap<u64, StepResult>,
    observed_completion_order: &mut Vec<NodeId>,
    product_session: Option<&SharedSession>,
    diagnostic: &str,
) {
    for node_id in canonical_order {
        if results_by_node.contains_key(&node_id.0) {
            continue;
        }
        let _was_prepared = start_failed_preparation(product_session, *node_id).is_ok();
        let step = crate::engine::executor::failed_product_step(*node_id, ContentHash::from_bytes(b""), diagnostic);
        record_completion(
            step,
            runner_states,
            &mut HashMap::new(),
            results_by_node,
            observed_completion_order,
            product_session,
        );
        if workflow.graph.node(*node_id).is_none() {
            debug_assert!(results_by_node.contains_key(&node_id.0));
        }
    }
}

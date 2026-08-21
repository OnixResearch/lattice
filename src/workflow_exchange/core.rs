#![allow(
    unknown_lints,
    ambiguous_params,
    assertion_density,
    non_trait_imports,
    numeric_units,
    reason = "the pure lifecycle core uses explicit branches and unit-bearing producer fields"
)]

use std::collections::BTreeMap;

use lattice_workflow_contract::ACCEPTED_SCHEMA;
use lattice_workflow_contract::Identity;
use lattice_workflow_contract::LogStream;
use lattice_workflow_contract::NodeState;
use lattice_workflow_contract::WorkflowAccepted;
use lattice_workflow_contract::WorkflowRequest;
use serde::Serialize;

use super::CompletionFacts;
use super::DurableExecutionState;
use super::DurableNode;
use super::DurableNodeResult;
use super::EXECUTION_RECORD_SCHEMA;
use super::ExecutionRecord;
use super::RetainedLog;
use super::TransitionError;
use super::WorkflowRoute;

const EXECUTION_IDENTITY_DOMAIN: &[u8] = b"lattice.workflow-exchange-execution.v1";
const ADAPTER_IDENTITY_DOMAIN: &[u8] = b"lattice.workflow-exchange-adapter.v1";
const NODE_RECEIPT_IDENTITY_DOMAIN: &[u8] = b"lattice.workflow-exchange-node-receipt.v1";
const EXECUTION_RECEIPT_IDENTITY_DOMAIN: &[u8] = b"lattice.workflow-exchange-receipt.v1";
const LOG_IDENTITY_DOMAIN: &[u8] = b"lattice.workflow-exchange-log.v1";
const ZERO_CURSOR: u64 = 0;

// r[impl lattice.workflow_interchange.application_admission]
pub fn initial_record(
    request: WorkflowRequest,
    route: &WorkflowRoute,
    execution_workflow_revision: Identity,
    profile_identity: Identity,
    run_id: u64,
) -> Result<ExecutionRecord, TransitionError> {
    let execution_identity = framed_identity(
        EXECUTION_IDENTITY_DOMAIN,
        &(request.identity.as_str(), execution_workflow_revision.as_str(), run_id, profile_identity.as_str()),
    )?;
    let accepted = WorkflowAccepted {
        schema: ACCEPTED_SCHEMA.to_string(),
        request_identity: request.identity.clone(),
        idempotency_key: request.idempotency_key.clone(),
        workflow_identity: execution_workflow_revision.clone(),
        execution_identity,
    };
    let nodes = route
        .nodes
        .iter()
        .map(|node| {
            let adapter_identity = framed_identity(
                ADAPTER_IDENTITY_DOMAIN,
                &(execution_workflow_revision.as_str(), node.workflow_node_id, node.operation_identity.as_str()),
            )?;
            Ok(DurableNode {
                request_node_id: node.request_node_id.clone(),
                workflow_node_id: node.workflow_node_id,
                adapter_identity,
                state: NodeState::Pending,
                receipt_identity: None,
            })
        })
        .collect::<Result<Vec<_>, TransitionError>>()?;
    Ok(ExecutionRecord {
        schema: EXECUTION_RECORD_SCHEMA.to_string(),
        request,
        accepted,
        handler_profile_identity: profile_identity,
        workflow_name: route.workflow_name.clone(),
        workflow_revision: execution_workflow_revision,
        run_id,
        sequence: 0,
        state: DurableExecutionState::Queued,
        cancellation_requested: false,
        nodes,
        logs: Vec::new(),
        receipt_identity: None,
    })
}

pub fn mark_running(record: &ExecutionRecord) -> Result<ExecutionRecord, TransitionError> {
    if record.state != DurableExecutionState::Queued {
        return Err(TransitionError::Terminal);
    }
    let mut next = record.clone();
    next.sequence = next.sequence.checked_add(1).ok_or(TransitionError::InvalidCompletion)?;
    next.state = DurableExecutionState::Running;
    Ok(next)
}

// r[impl lattice.workflow_interchange.application_recovery]
pub fn mark_unknown(record: &ExecutionRecord) -> Result<ExecutionRecord, TransitionError> {
    if record.state.is_terminal() || record.state == DurableExecutionState::Unknown {
        return Ok(record.clone());
    }
    let mut next = record.clone();
    next.sequence = next.sequence.checked_add(1).ok_or(TransitionError::InvalidCompletion)?;
    next.state = DurableExecutionState::Unknown;
    next.receipt_identity = None;
    Ok(next)
}

// r[impl lattice.workflow_interchange.application_lifecycle]
pub fn request_cancellation(
    record: &ExecutionRecord,
    expected_sequence: u64,
) -> Result<ExecutionRecord, TransitionError> {
    if record.sequence != expected_sequence {
        return Err(TransitionError::StaleSequence);
    }
    if record.state.is_terminal() || record.state == DurableExecutionState::Unknown {
        return Err(TransitionError::Terminal);
    }
    let mut next = record.clone();
    next.sequence = next.sequence.checked_add(1).ok_or(TransitionError::InvalidCompletion)?;
    next.cancellation_requested = true;
    Ok(next)
}

pub fn complete(record: &ExecutionRecord, facts: &CompletionFacts) -> Result<ExecutionRecord, TransitionError> {
    if record.state.is_terminal() || record.state == DurableExecutionState::Unknown {
        return Err(TransitionError::Terminal);
    }
    let mut next = record.clone();
    next.sequence = next.sequence.checked_add(1).ok_or(TransitionError::InvalidCompletion)?;
    if facts.suspended {
        apply_partial_results(&mut next, &facts.nodes)?;
        next.state = DurableExecutionState::Unknown;
        return Ok(next);
    }
    if facts.cancelled {
        for node in &mut next.nodes {
            terminalize_node(node, NodeState::Cancelled, &next.accepted.execution_identity)?;
        }
        next.state = DurableExecutionState::Cancelled;
    } else if facts.failed_before_results {
        let Some((first, remaining)) = next.nodes.split_first_mut() else {
            return Err(TransitionError::InvalidCompletion);
        };
        terminalize_node(first, NodeState::Failed, &next.accepted.execution_identity)?;
        for node in remaining {
            terminalize_node(node, NodeState::Skipped, &next.accepted.execution_identity)?;
        }
        next.state = DurableExecutionState::Failed;
    } else {
        apply_terminal_results(&mut next, &facts.nodes)?;
        next.state = if next.nodes.iter().any(|node| node.state == NodeState::Failed) {
            DurableExecutionState::Failed
        } else {
            DurableExecutionState::Completed
        };
    }
    next.logs = terminal_logs(&next)?;
    next.receipt_identity = Some(framed_identity(
        EXECUTION_RECEIPT_IDENTITY_DOMAIN,
        &(next.accepted.execution_identity.as_str(), next.sequence, next.state, &next.nodes, &next.logs),
    )?);
    Ok(next)
}

fn apply_partial_results(
    record: &mut ExecutionRecord,
    results: &[(u64, DurableNodeResult)],
) -> Result<(), TransitionError> {
    for (workflow_node_id, result) in results {
        let node = record
            .nodes
            .iter_mut()
            .find(|node| node.workflow_node_id == *workflow_node_id)
            .ok_or(TransitionError::InvalidCompletion)?;
        let state = match result {
            DurableNodeResult::Completed => NodeState::Completed,
            DurableNodeResult::Failed => NodeState::Failed,
            DurableNodeResult::Skipped => NodeState::Skipped,
        };
        terminalize_node(node, state, &record.accepted.execution_identity)?;
    }
    Ok(())
}

fn apply_terminal_results(
    record: &mut ExecutionRecord,
    results: &[(u64, DurableNodeResult)],
) -> Result<(), TransitionError> {
    let result_map = results.iter().copied().collect::<BTreeMap<_, _>>();
    if result_map.len() != results.len() {
        return Err(TransitionError::InvalidCompletion);
    }
    let has_failed = results.iter().any(|(_, result)| *result == DurableNodeResult::Failed);
    for node in &mut record.nodes {
        let missing_result = if has_failed {
            DurableNodeResult::Skipped
        } else {
            DurableNodeResult::Failed
        };
        let result = result_map.get(&node.workflow_node_id).copied().unwrap_or(missing_result);
        let state = match result {
            DurableNodeResult::Completed => NodeState::Completed,
            DurableNodeResult::Failed => NodeState::Failed,
            DurableNodeResult::Skipped => NodeState::Skipped,
        };
        terminalize_node(node, state, &record.accepted.execution_identity)?;
    }
    Ok(())
}

fn terminalize_node(
    node: &mut DurableNode,
    state: NodeState,
    execution_identity: &Identity,
) -> Result<(), TransitionError> {
    node.state = state;
    node.receipt_identity = Some(framed_identity(
        NODE_RECEIPT_IDENTITY_DOMAIN,
        &(execution_identity.as_str(), node.request_node_id.as_str(), node.workflow_node_id, state),
    )?);
    Ok(())
}

fn terminal_logs(record: &ExecutionRecord) -> Result<Vec<RetainedLog>, TransitionError> {
    let mut logs = Vec::new();
    let mut total_bytes = 0_u64;
    for node in &record.nodes {
        let maximum_references = usize::try_from(record.request.observation.maximum_log_references)
            .map_err(|_| TransitionError::LogLimit)?;
        if logs.len() >= maximum_references {
            break;
        }
        let bytes: &[u8] = match node.state {
            NodeState::Completed => b"completed\n",
            NodeState::Failed => b"failed\n",
            NodeState::Cancelled => b"cancelled\n",
            NodeState::Skipped => b"skipped\n",
            _ => continue,
        };
        let byte_count = u64::try_from(bytes.len()).map_err(|_| TransitionError::LogLimit)?;
        let next_total = total_bytes.checked_add(byte_count).ok_or(TransitionError::LogLimit)?;
        if byte_count > record.request.observation.maximum_log_read_bytes
            || next_total > record.request.observation.maximum_total_log_bytes
        {
            continue;
        }
        let reference_identity = framed_identity(
            LOG_IDENTITY_DOMAIN,
            &(
                record.accepted.execution_identity.as_str(),
                node.request_node_id.as_str(),
                LogStream::Diagnostic,
                ZERO_CURSOR,
                bytes,
            ),
        )?;
        logs.push(RetainedLog {
            node_id: node.request_node_id.clone(),
            stream: LogStream::Diagnostic,
            cursor: ZERO_CURSOR,
            reference_identity,
            bytes: bytes.to_vec(),
        });
        total_bytes = next_total;
    }
    Ok(logs)
}

pub(super) fn framed_identity<T: Serialize>(domain: &[u8], value: &T) -> Result<Identity, TransitionError> {
    let bytes = serde_json::to_vec(value).map_err(|_| TransitionError::Identity)?;
    let domain_length = u64::try_from(domain.len()).map_err(|_| TransitionError::Identity)?;
    let value_length = u64::try_from(bytes.len()).map_err(|_| TransitionError::Identity)?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(&domain_length.to_be_bytes());
    hasher.update(domain);
    hasher.update(&value_length.to_be_bytes());
    hasher.update(&bytes);
    Identity::new(format!("b3:{}", hasher.finalize().to_hex())).map_err(|_| TransitionError::Identity)
}

pub(super) fn zero_identity() -> Result<Identity, TransitionError> {
    Identity::new(format!("b3:{}", "0".repeat(blake3::OUT_LEN.saturating_mul(2))))
        .map_err(|_| TransitionError::Identity)
}

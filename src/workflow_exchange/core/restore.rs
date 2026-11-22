#![allow(
    unknown_lints,
    ambiguous_params,
    assertion_density,
    compound_condition,
    non_trait_imports,
    numeric_units,
    reason = "the pure adoption transition checks every carried identity as an explicit branch"
)]

//! Node receipts and the pure adoption of an admitted application checkpoint (Lattice #12).

use lattice_workflow_contract::ACCEPTED_SCHEMA;
use lattice_workflow_contract::Identity;
use lattice_workflow_contract::NodeState;
use lattice_workflow_contract::WorkflowAccepted;

use super::NODE_RECEIPT_IDENTITY_DOMAIN;
use super::framed_identity;
use crate::workflow_exchange::DurableExecutionState;
use crate::workflow_exchange::DurableNode;
use crate::workflow_exchange::ExecutionRecord;
use crate::workflow_exchange::TransitionError;

/// Receipt identity of one terminal node; `None` for skipped nodes. Shared by completion and
/// checkpoint publication, so a restored node keeps the receipt its producer would have issued.
pub fn node_receipt_identity(
    execution_identity: &Identity,
    request_node_id: &lattice_workflow_contract::Label,
    workflow_node_id: u64,
    state: NodeState,
) -> Result<Option<Identity>, TransitionError> {
    if state == NodeState::Skipped {
        return Ok(None);
    }
    framed_identity(
        NODE_RECEIPT_IDENTITY_DOMAIN,
        &(execution_identity.as_str(), request_node_id.as_str(), workflow_node_id, state),
    )
    .map(Some)
}

/// One node outcome carried from an admitted application checkpoint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdoptedNode {
    pub workflow_node_id: u64,
    pub adapter_identity: Identity,
    pub state: NodeState,
    pub receipt_identity: Option<Identity>,
}

/// Facts an admitted checkpoint contributes to a restarted execution record.
#[derive(Debug, Clone, Copy)]
pub struct CheckpointAdoption<'a> {
    pub accepted: &'a WorkflowAccepted,
    pub sequence: u64,
    pub cancellation_requested: bool,
    pub completed: &'a [AdoptedNode],
    pub uncertain: &'a [(u64, Identity)],
}

// r[impl lattice.application_checkpoint.restore]
/// Adopt an admitted checkpoint into a fresh record built by [`super::initial_record`] on the
/// target.
///
/// The producer's acceptance (and so its execution identity) is kept, adapter and receipt
/// identities must match the target route exactly, uncertain nodes become `Unknown` and are
/// never scheduled, and the sequence moves strictly past the checkpoint.
pub fn adopt_checkpoint(
    fresh: &ExecutionRecord,
    adoption: &CheckpointAdoption<'_>,
) -> Result<ExecutionRecord, TransitionError> {
    let accepted = adoption.accepted;
    if fresh.state != DurableExecutionState::Queued
        || fresh.sequence != 0
        || accepted.schema != ACCEPTED_SCHEMA
        || accepted.request_identity != fresh.request.identity
        || accepted.idempotency_key != fresh.request.idempotency_key
        || accepted.workflow_identity != fresh.workflow_revision
    {
        return Err(TransitionError::Identity);
    }
    let mut next = fresh.clone();
    next.accepted = accepted.clone();
    next.sequence = adoption.sequence.checked_add(1).ok_or(TransitionError::InvalidCompletion)?;
    next.cancellation_requested = adoption.cancellation_requested;
    let mut seen = std::collections::BTreeSet::new();
    for adopted in adoption.completed {
        if !seen.insert(adopted.workflow_node_id) || !adopted.state.is_terminal() {
            return Err(TransitionError::InvalidCompletion);
        }
        let node = adopted_node(&mut next.nodes, adopted.workflow_node_id, &adopted.adapter_identity)?;
        let expected = node_receipt_identity(
            &accepted.execution_identity,
            &node.request_node_id,
            node.workflow_node_id,
            adopted.state,
        )?;
        if expected != adopted.receipt_identity {
            return Err(TransitionError::Identity);
        }
        node.state = adopted.state;
        node.receipt_identity = expected;
    }
    for (workflow_node_id, adapter_identity) in adoption.uncertain {
        if !seen.insert(*workflow_node_id) {
            return Err(TransitionError::InvalidCompletion);
        }
        let node = adopted_node(&mut next.nodes, *workflow_node_id, adapter_identity)?;
        node.state = NodeState::Unknown;
        node.receipt_identity = None;
    }
    next.state = if adoption.uncertain.is_empty() {
        DurableExecutionState::Queued
    } else {
        DurableExecutionState::Unknown
    };
    next.logs = Vec::new();
    next.receipt_identity = None;
    Ok(next)
}

fn adopted_node<'a>(
    nodes: &'a mut [DurableNode],
    workflow_node_id: u64,
    adapter_identity: &Identity,
) -> Result<&'a mut DurableNode, TransitionError> {
    let node = nodes
        .iter_mut()
        .find(|node| node.workflow_node_id == workflow_node_id)
        .ok_or(TransitionError::InvalidCompletion)?;
    if node.adapter_identity != *adapter_identity {
        return Err(TransitionError::Identity);
    }
    Ok(node)
}

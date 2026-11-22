#![allow(
    unknown_lints,
    ambiguous_params,
    assertion_density,
    bool_naming,
    raw_arithmetic_overflow,
    compound_condition,
    non_trait_imports,
    numeric_units,
    path_segment_repetition,
    reason = "the pure checkpoint core keeps every publication rule as an explicit, separately tested branch"
)]

//! Pure C8 manifest construction: node outcome objects, receipts and the manifest itself.

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use lattice_workflow_contract::Identity;
use lattice_workflow_contract::NodeState;

use super::canonical;
use super::validate_manifest;
use crate::checkpoint::CheckpointBinding;
use crate::checkpoint::CheckpointExecution;
use crate::checkpoint::CheckpointProfile;
use crate::checkpoint::CompletedNode;
use crate::checkpoint::CompletedNodeState;
use crate::checkpoint::Consumed;
use crate::checkpoint::LocalRuntime;
use crate::checkpoint::Manifest;
use crate::checkpoint::ManifestProfile;
use crate::checkpoint::NodeOutputsObject;
use crate::checkpoint::Refusal;
use crate::checkpoint::UncertainNode;
use crate::checkpoint::UncertainState;
use crate::checkpoint::WorkspaceObject;

#[must_use]
pub const fn completed_state(state: crate::workflow::gate::PersistedNodeState) -> CompletedNodeState {
    match state {
        crate::workflow::gate::PersistedNodeState::Completed => CompletedNodeState::Completed,
        crate::workflow::gate::PersistedNodeState::Failed => CompletedNodeState::Failed,
        crate::workflow::gate::PersistedNodeState::Cancelled => CompletedNodeState::Skipped,
    }
}

#[must_use]
pub const fn node_state(state: CompletedNodeState) -> NodeState {
    match state {
        CompletedNodeState::Completed => NodeState::Completed,
        CompletedNodeState::Failed => NodeState::Failed,
        CompletedNodeState::Skipped => NodeState::Skipped,
    }
}

/// Canonical bytes and identity of one node outcome object.
pub fn node_outputs_object(
    workflow_node_id: u64,
    outcome: &crate::workflow::gate::PersistedNodeOutcome,
) -> Result<(Vec<u8>, Identity), Refusal> {
    if outcome.node_id.0 != workflow_node_id {
        return Err(Refusal::CheckpointInvalid);
    }
    let object = NodeOutputsObject {
        schema: crate::checkpoint::NODE_OUTPUTS_SCHEMA.to_string(),
        workflow_node_id,
        outcome: outcome.clone(),
    };
    canonical(crate::checkpoint::NODE_OUTPUTS_SCHEMA, &object)
}

// r[impl lattice.application_checkpoint.publish]
/// The manifest entry for one terminal node, with the receipt its producer would issue.
pub fn completed_node(
    record: &crate::workflow_exchange::ExecutionRecord,
    outcome: &crate::workflow::gate::PersistedNodeOutcome,
    outputs_object: Identity,
) -> Result<CompletedNode, Refusal> {
    let node = record
        .nodes
        .iter()
        .find(|node| node.workflow_node_id == outcome.node_id.0)
        .ok_or(Refusal::CrossedRoute)?;
    let state = completed_state(outcome.state);
    let receipt_identity = crate::workflow_exchange::node_receipt_identity(
        &record.accepted.execution_identity,
        &node.request_node_id,
        node.workflow_node_id,
        node_state(state),
    )
    .map_err(|_| Refusal::Encoding)?;
    Ok(CompletedNode {
        workflow_node_id: node.workflow_node_id,
        adapter_identity: node.adapter_identity.clone(),
        state,
        outputs_object,
        receipt_identity,
    })
}

/// Inputs to one manifest publication.
#[derive(Debug, Clone, Copy)]
pub struct ManifestInput<'a> {
    pub record: &'a crate::workflow_exchange::ExecutionRecord,
    pub binding: &'a CheckpointBinding,
    pub producing_incarnation: u32,
    pub local: &'a LocalRuntime,
    pub completed: &'a BTreeMap<u64, CompletedNode>,
    pub uncertain: &'a BTreeSet<u64>,
    pub workspace: &'a [WorkspaceObject],
    pub consumed: Consumed,
    pub cancellation_requested: bool,
}

pub fn build_manifest(input: &ManifestInput<'_>) -> Result<Manifest, Refusal> {
    if input.binding.checkpoint == CheckpointProfile::None {
        return Err(Refusal::CheckpointUnavailable);
    }
    let record = input.record;
    let uncertain_nodes = input
        .uncertain
        .iter()
        .filter(|id| !input.completed.contains_key(id))
        .map(|id| {
            let node = record.nodes.iter().find(|node| node.workflow_node_id == *id).ok_or(Refusal::CrossedRoute)?;
            Ok(UncertainNode {
                workflow_node_id: *id,
                adapter_identity: node.adapter_identity.clone(),
                last_state: UncertainState::Running,
            })
        })
        .collect::<Result<Vec<_>, Refusal>>()?;
    let mut workspace_objects = input.workspace.to_vec();
    workspace_objects.sort();
    let manifest = Manifest {
        schema: crate::checkpoint::MANIFEST_SCHEMA.to_string(),
        profile: ManifestProfile::WorkflowNodeBoundaryV1,
        execution: CheckpointExecution {
            request: record.request.identity.clone(),
            accepted: record.accepted.clone(),
            handler_profile: record.handler_profile_identity.clone(),
            workflow_revision: record.workflow_revision.clone(),
            sequence: record.sequence,
        },
        binding: input.binding.binding.clone(),
        producing_incarnation: input.producing_incarnation,
        architecture: input.local.architecture.clone(),
        runtime: input.local.runtime.clone(),
        completed_nodes: input.completed.values().cloned().collect(),
        uncertain_nodes,
        workspace_objects,
        consumed: input.consumed,
        deadline_unix_ms: input.binding.deadline_unix_ms,
        cancellation_requested: input.cancellation_requested || record.cancellation_requested,
    };
    validate_manifest(&manifest)?;
    Ok(manifest)
}

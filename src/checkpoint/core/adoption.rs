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
    reason = "the pure checkpoint core keeps every adoption rule as an explicit, separately tested branch"
)]

//! Pure restart classification and adoption of an admitted checkpoint onto a target route.

use std::collections::BTreeMap;

use lattice_workflow_contract::Identity;
use lattice_workflow_contract::NodeState;

use super::AdmittedCheckpoint;
use super::node_state;
use crate::checkpoint::AdoptionRecord;
use crate::checkpoint::Refusal;
use crate::checkpoint::RestartKind;

// r[impl lattice.application_checkpoint.receipt]
/// Classify how an execution came back.
///
/// - `ApplicationRestart`: a checkpoint was adopted by a strictly newer incarnation.
/// - `VmContinuation`: nothing was adopted and the producing incarnation is still the one running.
/// - `FreshRetry`: nothing was adopted and a newer incarnation starts from scratch.
pub const fn restart_kind(producing: u32, observed: u32, adopted: bool) -> Result<RestartKind, Refusal> {
    if adopted {
        if observed > producing {
            return Ok(RestartKind::ApplicationRestart);
        }
        return Err(Refusal::StaleAdmission);
    }
    if observed == producing {
        Ok(RestartKind::VmContinuation)
    } else if observed > producing {
        Ok(RestartKind::FreshRetry)
    } else {
        Err(Refusal::StaleAdmission)
    }
}

/// Verify the target route reproduces every adapter and receipt identity of the checkpoint.
pub fn admit_route(
    fresh: &crate::workflow_exchange::ExecutionRecord,
    admitted: &AdmittedCheckpoint,
) -> Result<(), Refusal> {
    let execution = &admitted.manifest.execution;
    if fresh.request.identity != execution.request {
        return Err(Refusal::CrossedRequest);
    }
    if fresh.handler_profile_identity != execution.handler_profile {
        return Err(Refusal::CrossedHandlerProfile);
    }
    if fresh.workflow_revision != execution.workflow_revision {
        return Err(Refusal::CrossedWorkflowRevision);
    }
    let nodes = fresh.nodes.iter().map(|node| (node.workflow_node_id, node)).collect::<BTreeMap<_, _>>();
    for (completed, _outcome) in &admitted.outcomes {
        let node = nodes.get(&completed.workflow_node_id).ok_or(Refusal::CrossedAdapter)?;
        if node.adapter_identity != completed.adapter_identity {
            return Err(Refusal::CrossedAdapter);
        }
        let receipt = crate::workflow_exchange::node_receipt_identity(
            &execution.accepted.execution_identity,
            &node.request_node_id,
            node.workflow_node_id,
            node_state(completed.state),
        )
        .map_err(|_| Refusal::Encoding)?;
        if receipt != completed.receipt_identity {
            return Err(Refusal::CrossedReceipt);
        }
    }
    for uncertain in &admitted.uncertain {
        let node = nodes.get(&uncertain.workflow_node_id).ok_or(Refusal::CrossedAdapter)?;
        if node.adapter_identity != uncertain.adapter_identity {
            return Err(Refusal::CrossedAdapter);
        }
    }
    Ok(())
}

/// The adoption a target records for an admitted checkpoint restored into `workspace`.
#[must_use]
pub fn adoption_record(admitted: &AdmittedCheckpoint, workspace: Option<&std::path::Path>) -> AdoptionRecord {
    AdoptionRecord {
        schema: crate::checkpoint::ADOPTION_SCHEMA.to_string(),
        execution: admitted.manifest.execution.accepted.execution_identity.clone(),
        receipt: admitted.receipt.clone(),
        producing_incarnation: admitted.manifest.producing_incarnation,
        consumed: admitted.consumed,
        deadline_unix_ms: admitted.deadline_unix_ms,
        cancellation_requested: admitted.cancellation_requested,
        binding: admitted.manifest.binding.clone(),
        environment: admitted.manifest.runtime.environment.clone(),
        workspace: Some(crate::checkpoint::RestoredWorkspace {
            path: workspace.map(std::path::Path::to_path_buf),
        }),
        completed_nodes: admitted.outcomes.iter().map(|(node, _outcome)| node.clone()).collect(),
    }
}

// r[impl lattice.application_checkpoint.resume]
/// The serving binding publishes the adopted execution's next checkpoints and runs it in its
/// environment and workspace, so it must declare a checkpoint and reproduce the deadline, attempt
/// binding, environment and workspace restore admitted and wrote into. An adoption without a
/// recorded workspace (written by `39936a19`) is served by no binding.
#[must_use]
pub fn serves_adoption(binding: &crate::checkpoint::CheckpointBinding, adoption: &AdoptionRecord) -> bool {
    binding.checkpoint != crate::checkpoint::CheckpointProfile::None
        && binding.deadline_unix_ms == adoption.deadline_unix_ms
        && binding.binding == adoption.binding
        && binding.environment == adoption.environment
        && adoption.workspace.as_ref().is_some_and(|restored| restored.path == binding.workspace)
}

/// Transition inputs for [`crate::workflow_exchange::adopt_checkpoint`].
#[must_use]
pub fn adopted_nodes(
    admitted: &AdmittedCheckpoint,
) -> (Vec<crate::workflow_exchange::AdoptedNode>, Vec<(u64, Identity)>) {
    let completed = admitted
        .outcomes
        .iter()
        .map(|(node, _outcome)| crate::workflow_exchange::AdoptedNode {
            workflow_node_id: node.workflow_node_id,
            adapter_identity: node.adapter_identity.clone(),
            state: node_state(node.state),
            receipt_identity: node.receipt_identity.clone(),
        })
        .collect();
    let uncertain = admitted
        .uncertain
        .iter()
        .map(|node| (node.workflow_node_id, node.adapter_identity.clone()))
        .collect();
    (completed, uncertain)
}

// r[impl lattice.application_checkpoint.carry_forward]
/// Independent oracle: the adopted record and adoption carry elapsed budget, deadline and every
/// cancel (manifest, sibling or Kiln's request) forward, keep every uncertain node `Unknown`, and
/// never reuse the producer's sequence.
pub fn verify_carry_forward(
    admitted: &AdmittedCheckpoint,
    adoption: &AdoptionRecord,
    record: &crate::workflow_exchange::ExecutionRecord,
) -> Result<(), Refusal> {
    let manifest = &admitted.manifest;
    let consumed_kept = adoption.consumed.elapsed_ms >= manifest.consumed.elapsed_ms;
    let must_cancel = manifest.cancellation_requested || admitted.cancellation_requested;
    let cancellation_kept = !must_cancel || (adoption.cancellation_requested && record.cancellation_requested);
    let uncertain_unknown = admitted.uncertain.iter().all(|uncertain| {
        record
            .nodes
            .iter()
            .any(|node| node.workflow_node_id == uncertain.workflow_node_id && node.state == NodeState::Unknown)
    });
    let state_honest =
        admitted.uncertain.is_empty() || record.state == crate::workflow_exchange::DurableExecutionState::Unknown;
    if !consumed_kept
        || !cancellation_kept
        || adoption.deadline_unix_ms != manifest.deadline_unix_ms
        || adoption.binding != manifest.binding
        || adoption.receipt != admitted.receipt
        || record.accepted != manifest.execution.accepted
        || record.sequence <= manifest.execution.sequence
        || !uncertain_unknown
        || !state_honest
    {
        return Err(Refusal::CarryForward);
    }
    Ok(())
}

/// Step results the executor restores; only checkpoint-complete nodes, never uncertain ones.
pub fn restored_steps(admitted: &AdmittedCheckpoint) -> Result<Vec<crate::engine::executor::StepResult>, Refusal> {
    let outcomes = admitted.outcomes.iter().map(|(_node, outcome)| outcome.clone()).collect::<Vec<_>>();
    crate::workflow::gate::shell::checkpoint_to_results(&outcomes).map_err(|_| Refusal::CheckpointInvalid)
}

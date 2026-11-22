#![allow(
    unknown_lints,
    assertion_density,
    non_trait_imports,
    path_segment_repetition,
    unbounded_collection_growth,
    reason = "the restore shell's file effects verify every object before anything is written"
)]

//! Node objects and workspace files a restore moves, each verified against its digest.

use std::path::Path;
use std::path::PathBuf;

use crate::checkpoint::AdmittedCheckpoint;
use crate::checkpoint::AdoptionRecord;
use crate::checkpoint::CheckpointRoot;
use crate::checkpoint::NODE_OUTPUTS_SCHEMA;
use crate::checkpoint::NodeOutputsObject;
use crate::checkpoint::Refusal;
use crate::checkpoint::identity_of_bytes;

/// Copy the verified node outcome objects into the target root, so this incarnation's own
/// manifests reference only objects present beside them.
pub(super) fn carry_objects(
    source: &CheckpointRoot,
    target: &CheckpointRoot,
    admitted: &AdmittedCheckpoint,
) -> Result<(), Refusal> {
    if source == target {
        return Ok(());
    }
    for (node, _outcome) in &admitted.outcomes {
        let bytes = source.node_object(&node.outputs_object)?.ok_or(Refusal::ObjectMissing)?;
        if identity_of_bytes(NODE_OUTPUTS_SCHEMA, &bytes)? != node.outputs_object {
            return Err(Refusal::ObjectMismatch);
        }
        target.put_node_object(&node.outputs_object, &bytes)?;
    }
    Ok(())
}

/// Verify every workspace object against `workspace` first, then write the missing ones.
///
/// Existing different content is refused before anything is written. Returns the files written.
pub(super) fn materialize_workspace(
    root: &CheckpointRoot,
    workspace: &Path,
    admitted: &AdmittedCheckpoint,
) -> Result<Vec<PathBuf>, Refusal> {
    let mut missing = Vec::new();
    for object in &admitted.manifest.workspace_objects {
        let bytes = root.workspace_object(&object.blake3)?.ok_or(Refusal::ObjectMissing)?;
        if crate::checkpoint::content_hex(&bytes) != object.blake3 {
            return Err(Refusal::ObjectMismatch);
        }
        let target = workspace.join(&object.path);
        match std::fs::read(&target) {
            Ok(existing) if existing == bytes => {}
            Ok(_) => return Err(Refusal::WorkspacePath),
            Err(_) => missing.push((target, bytes)),
        }
    }
    let mut created = Vec::with_capacity(missing.len());
    for (target, bytes) in missing {
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).map_err(|_| Refusal::Io)?;
        }
        std::fs::write(&target, &bytes).map_err(|_| Refusal::Io)?;
        created.push(target);
    }
    Ok(created)
}

/// Step results of an adoption's checkpoint-complete nodes, read and verified from `root`.
pub fn adopted_steps(
    root: &CheckpointRoot,
    adoption: &AdoptionRecord,
) -> Result<Vec<crate::engine::executor::StepResult>, Refusal> {
    let mut outcomes = Vec::with_capacity(adoption.completed_nodes.len());
    for node in &adoption.completed_nodes {
        let bytes = root.node_object(&node.outputs_object)?.ok_or(Refusal::ObjectMissing)?;
        if identity_of_bytes(NODE_OUTPUTS_SCHEMA, &bytes)? != node.outputs_object {
            return Err(Refusal::ObjectMismatch);
        }
        let object: NodeOutputsObject = serde_json::from_slice(&bytes).map_err(|_| Refusal::ObjectMismatch)?;
        if object.workflow_node_id != node.workflow_node_id {
            return Err(Refusal::ObjectMismatch);
        }
        outcomes.push(object.outcome);
    }
    crate::workflow::gate::shell::checkpoint_to_results(&outcomes).map_err(|_| Refusal::CheckpointInvalid)
}

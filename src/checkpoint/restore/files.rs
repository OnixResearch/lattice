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
/// Existing different content is refused before anything is written. Temporary files an
/// interrupted write left in the directories written to are removed first. Each file is written
/// through a fsynced temporary file renamed into place, so no final path ever holds part of an
/// object; when a write fails, the files this call already wrote are removed. Returns the files
/// written.
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
    remove_leftover_temporaries(&missing);
    let mut created = Vec::with_capacity(missing.len());
    for (target, bytes) in missing {
        // Recorded once placed and before its directory is synced, so a failed fsync removes it.
        let placed = place_restored(&target, &bytes);
        if placed.is_ok() {
            created.push(target);
        }
        if let Err(refusal) = placed.and_then(|()| sync_last(&created)) {
            remove_created(&created);
            return Err(refusal);
        }
    }
    Ok(created)
}

fn remove_leftover_temporaries(missing: &[(PathBuf, Vec<u8>)]) {
    let directories = missing
        .iter()
        .filter_map(|(target, _bytes)| target.parent())
        .collect::<std::collections::BTreeSet<_>>();
    for directory in directories {
        crate::checkpoint::publish::remove_temporaries(directory);
    }
}

/// Put `target` in place through a fsynced temporary file renamed into place.
fn place_restored(target: &Path, bytes: &[u8]) -> Result<(), Refusal> {
    let directory = target.parent().ok_or(Refusal::WorkspacePath)?;
    let name = target.file_name().and_then(std::ffi::OsStr::to_str).ok_or(Refusal::WorkspacePath)?;
    crate::checkpoint::publish::place_atomic(directory, name, bytes)
}

/// Sync the directory of the file placed last, making its rename durable.
fn sync_last(created: &[PathBuf]) -> Result<(), Refusal> {
    let directory = created.last().and_then(|placed| placed.parent()).ok_or(Refusal::WorkspacePath)?;
    crate::checkpoint::publish::sync_directory(directory)
}

/// Remove workspace files a restore wrote; each was missing before it, so nothing else is lost.
pub(super) fn remove_created(created: &[PathBuf]) {
    for path in created {
        let _removed = std::fs::remove_file(path);
    }
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

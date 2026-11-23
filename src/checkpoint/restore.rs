#![allow(
    unknown_lints,
    ambient_clock,
    assertion_density,
    fallible_int_fallback,
    non_trait_imports,
    path_segment_repetition,
    sentinel_fallback,
    unbounded_collection_growth,
    reason = "the restore shell reads a checkpoint root and target store, then defers every decision to the pure core"
)]

//! Restore shell: read a committed checkpoint, admit it, adopt it into a fresh target store.

use std::collections::BTreeMap;

use super::Admission;
use super::AdmittedCheckpoint;
use super::AdoptionOutcome;
use super::AdoptionRecord;
use super::CheckpointBinding;
use super::CheckpointRoot;
use super::Manifest;
use super::Refusal;
use super::RestartRequest;

mod files;

pub use files::adopted_steps;

/// Read everything a restart names from `root` and the target store, then admit it.
pub fn admit_from_root(
    root: &CheckpointRoot,
    request: &RestartRequest,
    store: &crate::store::Store,
    binding: &CheckpointBinding,
    now_unix_ms: u64,
) -> Result<Admission, Refusal> {
    let local = super::local_runtime(binding.environment.clone())?;
    let marker = root.marker(&request.checkpoint)?;
    let manifest_bytes = root.manifest_bytes(&request.checkpoint)?.ok_or(Refusal::NotCommitted)?;
    let parsed: Option<Manifest> = serde_json::from_slice(&manifest_bytes).ok();
    let mut node_objects = BTreeMap::new();
    let mut workspace_objects = BTreeMap::new();
    let mut request_object = None;
    if let Some(manifest) = &parsed {
        for node in &manifest.completed_nodes {
            if let Some(bytes) = root.node_object(&node.outputs_object)? {
                node_objects.insert(node.outputs_object.clone(), bytes);
            }
        }
        for object in &manifest.workspace_objects {
            if let Some(bytes) = root.workspace_object(&object.blake3)? {
                workspace_objects.insert(object.blake3.clone(), bytes);
            }
        }
        request_object = root.request_object(&manifest.execution.request)?;
    }
    let siblings = sibling_manifests(root, request, parsed.as_ref())?;
    let prior = store.load_workflow_exchange_adoption(&request.execution).map_err(|_| Refusal::Store)?;
    super::core::admit(&super::core::AdmissionInput {
        request,
        marker: marker.as_ref(),
        manifest_bytes: &manifest_bytes,
        local: &local,
        node_objects: &node_objects,
        workspace_objects: &workspace_objects,
        request_object: request_object.as_deref(),
        siblings: &siblings,
        prior: prior.as_ref(),
        now_unix_ms,
    })
}

/// Every other committed, parseable manifest in the root for the same execution.
fn sibling_manifests(
    root: &CheckpointRoot,
    request: &RestartRequest,
    manifest: Option<&Manifest>,
) -> Result<Vec<Manifest>, Refusal> {
    let Some(manifest) = manifest else {
        return Ok(Vec::new());
    };
    let mut siblings = Vec::new();
    for (_publication, identity) in root.committed()? {
        if identity == request.checkpoint || root.marker(&identity)?.is_none() {
            continue;
        }
        let Some(bytes) = root.manifest_bytes(&identity)? else {
            continue;
        };
        if super::core::identity_of_bytes(super::MANIFEST_SCHEMA, &bytes)? != identity {
            continue;
        }
        let Ok(sibling) = serde_json::from_slice::<Manifest>(&bytes) else {
            continue;
        };
        if sibling.execution.accepted.execution_identity == manifest.execution.accepted.execution_identity {
            siblings.push(sibling);
        }
    }
    Ok(siblings)
}

/// Facts the target route contributes to an adoption.
#[derive(Clone, Copy)]
pub struct AdoptionTarget<'a> {
    pub store: &'a crate::store::Store,
    pub source: &'a CheckpointRoot,
    pub route: &'a crate::workflow_exchange::WorkflowRoute,
    pub binding: &'a CheckpointBinding,
    pub workflow: &'a crate::engine::workflow::Workflow,
    pub execution_workflow_revision: &'a lattice_workflow_contract::Identity,
    pub handler_profile_identity: &'a lattice_workflow_contract::Identity,
}

#[derive(Debug, Clone)]
pub struct Adopted {
    pub record: crate::workflow_exchange::ExecutionRecord,
    pub adoption: AdoptionRecord,
}

// r[impl lattice.application_checkpoint.restore]
/// Adopt an admitted checkpoint into a fresh target store; nothing runs here.
///
/// Every check, the carry-forward oracle and the workspace write happen before the store
/// transaction commits, and the oracle runs again inside it, so a refused adoption leaves no
/// record behind and a committed one is complete. Workspace files this call created are removed
/// when the transaction fails.
pub fn adopt(target: &AdoptionTarget<'_>, admitted: &AdmittedCheckpoint) -> Result<Adopted, Refusal> {
    check_target(target, admitted)?;
    let steps = super::core::restored_steps(admitted)?;
    let context = crate::engine::effect::execution::RunContext::unavailable(
        crate::engine::RunId(0),
        Box::new(crate::engine::effect::execution::UnavailableAttempts),
    )
    .map_err(|_| Refusal::CheckpointInvalid)?;
    crate::engine::effect::execution::Session::new_with_checkpoint(target.workflow, &context, &steps)
        .map_err(|_| Refusal::CheckpointInvalid)?;
    let accepted = &admitted.manifest.execution.accepted;
    let execution = &accepted.execution_identity;
    if target.store.load_workflow_exchange(execution).map_err(|_| Refusal::Store)?.is_some() {
        return Err(Refusal::TargetNotFresh);
    }
    let (provisional, adoption) = build_adoption(target, admitted, 0)?;
    super::core::verify_carry_forward(admitted, &adoption, &provisional)?;
    files::carry_objects(target.source, &CheckpointRoot::new(target.binding.root.clone()), admitted)?;
    let created = match &target.binding.workspace {
        Some(workspace) => files::materialize_workspace(target.source, workspace, admitted)?,
        None => Vec::new(),
    };
    let mut refusal = None;
    let outcome = target.store.adopt_workflow_exchange_checkpoint(accepted, |run_id| {
        build_adoption(target, admitted, run_id)
            .and_then(|(record, adoption)| {
                super::core::verify_carry_forward(admitted, &adoption, &record).map(|()| (record, adoption))
            })
            .map_err(|error| {
                refusal = Some(error);
                crate::store::StoreError::Conflict(format!("checkpoint adoption refused: {error}"))
            })
    });
    match outcome {
        Ok(AdoptionOutcome::Adopted { record, adoption }) => Ok(Adopted {
            record: *record,
            adoption: *adoption,
        }),
        Ok(AdoptionOutcome::Existing(_)) | Err(_) => {
            for path in created {
                let _removed = std::fs::remove_file(path);
            }
            Err(refusal.unwrap_or(Refusal::TargetNotFresh))
        }
    }
}

// r[impl lattice.application_checkpoint.carry_forward]
/// The target route and binding must reproduce the checkpoint's deadline and attempt binding: a
/// restart never extends the deadline or moves the execution to another C2 binding.
fn check_target(target: &AdoptionTarget<'_>, admitted: &AdmittedCheckpoint) -> Result<(), Refusal> {
    if target.binding.graph_identity != target.route.graph_identity
        || target.binding.workflow_revision != target.route.workflow_revision
    {
        return Err(Refusal::CrossedRoute);
    }
    if target.binding.deadline_unix_ms != admitted.deadline_unix_ms {
        return Err(Refusal::CarryForward);
    }
    if target.binding.binding != admitted.manifest.binding {
        return Err(Refusal::CrossedBinding);
    }
    if !admitted.manifest.workspace_objects.is_empty() && target.binding.workspace.is_none() {
        return Err(Refusal::WorkspacePath);
    }
    Ok(())
}

fn build_adoption(
    target: &AdoptionTarget<'_>,
    admitted: &AdmittedCheckpoint,
    run_id: u64,
) -> Result<(crate::workflow_exchange::ExecutionRecord, AdoptionRecord), Refusal> {
    let fresh = crate::workflow_exchange::initial_record(
        admitted.workflow_request.clone(),
        target.route,
        target.execution_workflow_revision.clone(),
        target.handler_profile_identity.clone(),
        run_id,
    )
    .map_err(|_| Refusal::Encoding)?;
    super::core::admit_route(&fresh, admitted)?;
    let (completed, uncertain) = super::core::adopted_nodes(admitted);
    let record = crate::workflow_exchange::adopt_checkpoint(&fresh, &crate::workflow_exchange::CheckpointAdoption {
        accepted: &admitted.manifest.execution.accepted,
        sequence: admitted.manifest.execution.sequence,
        cancellation_requested: admitted.cancellation_requested,
        completed: &completed,
        uncertain: &uncertain,
    })
    .map_err(|_| Refusal::CrossedAdapter)?;
    Ok((record, super::core::adoption_record(admitted, target.binding.workspace.as_deref())))
}

/// Wall-clock milliseconds for the advisory deadline check; Kiln's host clock stays authoritative.
#[must_use]
pub fn now_unix_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |duration| u64::try_from(duration.as_millis()).unwrap_or(u64::MAX))
}

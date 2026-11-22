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
    reason = "the pure checkpoint core keeps every admission rule as an explicit, separately tested branch"
)]

//! Pure restart admission: every object, crossing, runtime and sibling rule, without I/O.

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use lattice_workflow_contract::Identity;
use lattice_workflow_contract::WorkflowRequest;

use super::completed_state;
use super::content_hex;
use super::identity_of_bytes;
use super::restart_kind;
use super::validate_admission_reference;
use super::validate_manifest;
use crate::checkpoint::AdoptionRecord;
use crate::checkpoint::CheckpointProfile;
use crate::checkpoint::CommitMarker;
use crate::checkpoint::CompletedNode;
use crate::checkpoint::Consumed;
use crate::checkpoint::LocalRuntime;
use crate::checkpoint::Manifest;
use crate::checkpoint::NodeOutputsObject;
use crate::checkpoint::Refusal;
use crate::checkpoint::RestartReceipt;
use crate::checkpoint::RestartRequest;
use crate::checkpoint::UncertainNode;
use crate::checkpoint::UncertainState;

/// Everything the shell read for one restart, verified here without I/O.
#[derive(Debug, Clone, Copy)]
pub struct AdmissionInput<'a> {
    pub request: &'a RestartRequest,
    pub marker: Option<&'a CommitMarker>,
    pub manifest_bytes: &'a [u8],
    pub local: &'a LocalRuntime,
    /// Node outcome objects by identity, as read from the checkpoint root.
    pub node_objects: &'a BTreeMap<Identity, Vec<u8>>,
    /// Workspace objects by content digest, as read from the checkpoint root.
    pub workspace_objects: &'a BTreeMap<String, Vec<u8>>,
    /// The `WorkflowRequest` object named by `execution.request`, if present.
    pub request_object: Option<&'a [u8]>,
    /// Other committed manifests found in the same root.
    pub siblings: &'a [Manifest],
    /// An adoption already present in the target store for this execution.
    pub prior: Option<&'a AdoptionRecord>,
    pub now_unix_ms: u64,
}

#[derive(Debug, Clone)]
pub struct AdmittedCheckpoint {
    pub identity: Identity,
    pub manifest: Manifest,
    pub workflow_request: WorkflowRequest,
    pub outcomes: Vec<(CompletedNode, crate::workflow::gate::PersistedNodeOutcome)>,
    /// The manifest's uncertain nodes plus anything a sibling shows as started or finished.
    pub uncertain: Vec<UncertainNode>,
    pub consumed: Consumed,
    pub cancellation_requested: bool,
    pub deadline_unix_ms: u64,
    pub receipt: RestartReceipt,
}

#[derive(Debug, Clone)]
pub enum Admission {
    Fresh(Box<AdmittedCheckpoint>),
    /// The identical restart was already adopted by this target.
    Idempotent(RestartReceipt),
}

// r[impl lattice.application_checkpoint.admission]
pub fn admit(input: &AdmissionInput<'_>) -> Result<Admission, Refusal> {
    let request = input.request;
    if request.schema != crate::checkpoint::RESTART_REQUEST_SCHEMA {
        return Err(Refusal::Schema);
    }
    validate_admission_reference(&request.admission)?;
    let marker = input.marker.ok_or(Refusal::NotCommitted)?;
    let identity = identity_of_bytes(crate::checkpoint::MANIFEST_SCHEMA, input.manifest_bytes)?;
    if marker.manifest != identity || request.checkpoint != identity {
        return Err(Refusal::ManifestIdentity);
    }
    let manifest: Manifest = serde_json::from_slice(input.manifest_bytes).map_err(|_| Refusal::Schema)?;
    if serde_json::to_vec(&manifest).map_err(|_| Refusal::Encoding)? != input.manifest_bytes {
        return Err(Refusal::Encoding);
    }
    validate_manifest(&manifest)?;
    admit_runtime(&manifest, input.local)?;
    admit_crossing(&manifest, request)?;
    let workflow_request = admit_request_object(&manifest, input.request_object)?;
    if request.restarted_incarnation <= manifest.producing_incarnation
        || input.siblings.iter().any(|sibling| {
            sibling.execution.accepted.execution_identity == manifest.execution.accepted.execution_identity
                && sibling.producing_incarnation >= request.restarted_incarnation
        })
    {
        return Err(Refusal::StaleAdmission);
    }
    if input.now_unix_ms >= manifest.deadline_unix_ms {
        return Err(Refusal::DeadlineExpired);
    }
    let outcomes = admit_node_objects(&manifest, input.node_objects)?;
    admit_workspace_objects(&manifest, input.workspace_objects)?;
    let (uncertain, consumed, carried_cancellation) = carry_siblings(&manifest, input.siblings)?;
    // r[impl lattice.application_checkpoint.cancellation]
    // A cancel Kiln acknowledged after the last publication lives only in Kiln's decision.
    let cancellation_requested = carried_cancellation || request.cancellation_requested;
    let receipt = RestartReceipt {
        checkpoint: identity.clone(),
        kind: restart_kind(manifest.producing_incarnation, request.restarted_incarnation, true)?,
        restarted_incarnation: request.restarted_incarnation,
        admission: request.admission.clone(),
    };
    if let Some(prior) = input.prior {
        if prior.receipt == receipt && prior.cancellation_requested == cancellation_requested {
            return Ok(Admission::Idempotent(receipt));
        }
        if prior.receipt == receipt {
            return Err(Refusal::TargetNotFresh);
        }
        if prior.receipt.restarted_incarnation >= request.restarted_incarnation {
            return Err(Refusal::StaleAdmission);
        }
        return Err(Refusal::TargetNotFresh);
    }
    let deadline_unix_ms = manifest.deadline_unix_ms;
    Ok(Admission::Fresh(Box::new(AdmittedCheckpoint {
        identity,
        manifest,
        workflow_request,
        outcomes,
        uncertain,
        consumed,
        cancellation_requested,
        deadline_unix_ms,
        receipt,
    })))
}

fn admit_runtime(manifest: &Manifest, local: &LocalRuntime) -> Result<(), Refusal> {
    admit_runtime_facts(manifest.profile.checkpoint(), &manifest.architecture, &manifest.runtime, local)
}

// r[impl lattice.application_checkpoint.portable]
/// Runtime match: the same runtime revision and environment always. A portable profile (JSON
/// node outcomes and workspace bytes, no machine state) then admits any separately built binary
/// of that revision, on any architecture. A non-portable profile also needs the same
/// architecture and the identical binary.
pub fn admit_runtime_facts(
    profile: CheckpointProfile,
    architecture: &str,
    runtime: &crate::checkpoint::CheckpointRuntime,
    local: &LocalRuntime,
) -> Result<(), Refusal> {
    if runtime.lattice_revision != local.runtime.lattice_revision {
        return Err(Refusal::Runtime);
    }
    if runtime.environment != local.runtime.environment {
        return Err(Refusal::Environment);
    }
    if profile.is_portable() {
        return Ok(());
    }
    if architecture != local.architecture {
        return Err(Refusal::Architecture);
    }
    if runtime.lattice_binary_blake3 != local.runtime.lattice_binary_blake3 {
        return Err(Refusal::Runtime);
    }
    Ok(())
}

fn admit_crossing(manifest: &Manifest, request: &RestartRequest) -> Result<(), Refusal> {
    if request.request != manifest.execution.request {
        return Err(Refusal::CrossedRequest);
    }
    if request.execution != manifest.execution.accepted.execution_identity {
        return Err(Refusal::CrossedExecution);
    }
    if request.binding != manifest.binding {
        return Err(Refusal::CrossedBinding);
    }
    Ok(())
}

fn admit_request_object(manifest: &Manifest, bytes: Option<&[u8]>) -> Result<WorkflowRequest, Refusal> {
    let bytes = bytes.ok_or(Refusal::ObjectMissing)?;
    let request: WorkflowRequest = serde_json::from_slice(bytes).map_err(|_| Refusal::ObjectMismatch)?;
    if lattice_workflow_contract::request_identity(&request) != manifest.execution.request
        || request.identity != manifest.execution.request
        || request.idempotency_key != manifest.execution.accepted.idempotency_key
    {
        return Err(Refusal::ObjectMismatch);
    }
    Ok(request)
}

fn admit_node_objects(
    manifest: &Manifest,
    objects: &BTreeMap<Identity, Vec<u8>>,
) -> Result<Vec<(CompletedNode, crate::workflow::gate::PersistedNodeOutcome)>, Refusal> {
    let mut outcomes = Vec::with_capacity(manifest.completed_nodes.len());
    for node in &manifest.completed_nodes {
        let bytes = objects.get(&node.outputs_object).ok_or(Refusal::ObjectMissing)?;
        if identity_of_bytes(crate::checkpoint::NODE_OUTPUTS_SCHEMA, bytes)? != node.outputs_object {
            return Err(Refusal::ObjectMismatch);
        }
        let object: NodeOutputsObject = serde_json::from_slice(bytes).map_err(|_| Refusal::ObjectMismatch)?;
        if object.schema != crate::checkpoint::NODE_OUTPUTS_SCHEMA
            || object.workflow_node_id != node.workflow_node_id
            || object.outcome.node_id.0 != node.workflow_node_id
            || completed_state(object.outcome.state) != node.state
        {
            return Err(Refusal::ObjectMismatch);
        }
        if object.outcome.outputs.values().any(|value| !value.sensitive_fragments.is_empty()) {
            return Err(Refusal::SensitiveOutput);
        }
        outcomes.push((node.clone(), object.outcome));
    }
    Ok(outcomes)
}

fn admit_workspace_objects(manifest: &Manifest, objects: &BTreeMap<String, Vec<u8>>) -> Result<(), Refusal> {
    for object in &manifest.workspace_objects {
        let bytes = objects.get(&object.blake3).ok_or(Refusal::ObjectMissing)?;
        let length = u64::try_from(bytes.len()).map_err(|_| Refusal::ObjectMismatch)?;
        if length != object.bytes || content_hex(bytes) != object.blake3 {
            return Err(Refusal::ObjectMismatch);
        }
    }
    Ok(())
}

// r[impl lattice.application_checkpoint.carry_forward]
/// Siblings can only add uncertainty, budget and cancellation; they never remove them.
fn carry_siblings(manifest: &Manifest, siblings: &[Manifest]) -> Result<(Vec<UncertainNode>, Consumed, bool), Refusal> {
    let completed = manifest.completed_nodes.iter().map(|node| node.workflow_node_id).collect::<BTreeSet<_>>();
    let mut uncertain = manifest
        .uncertain_nodes
        .iter()
        .map(|node| (node.workflow_node_id, node.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut consumed = manifest.consumed;
    let mut cancellation_requested = manifest.cancellation_requested;
    for sibling in siblings {
        if sibling.execution.accepted != manifest.execution.accepted {
            continue;
        }
        if sibling.deadline_unix_ms != manifest.deadline_unix_ms || sibling.binding != manifest.binding {
            return Err(Refusal::CarryForward);
        }
        consumed.elapsed_ms = consumed.elapsed_ms.max(sibling.consumed.elapsed_ms);
        cancellation_requested |= sibling.cancellation_requested;
        let started = sibling
            .completed_nodes
            .iter()
            .map(|node| (node.workflow_node_id, &node.adapter_identity))
            .chain(sibling.uncertain_nodes.iter().map(|node| (node.workflow_node_id, &node.adapter_identity)));
        for (workflow_node_id, adapter_identity) in started {
            if completed.contains(&workflow_node_id) {
                continue;
            }
            uncertain.entry(workflow_node_id).or_insert_with(|| UncertainNode {
                workflow_node_id,
                adapter_identity: adapter_identity.clone(),
                last_state: UncertainState::Running,
            });
        }
    }
    Ok((uncertain.into_values().collect(), consumed, cancellation_requested))
}

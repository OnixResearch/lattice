#![allow(
    unknown_lints,
    ambiguous_params,
    assertion_density,
    bool_naming,
    excessive_file_length,
    raw_arithmetic_overflow,
    compound_condition,
    non_trait_imports,
    numeric_units,
    path_segment_repetition,
    reason = "the pure checkpoint core keeps every admission rule as an explicit, separately tested branch"
)]

//! Pure C8 manifest construction, admission and restart classification. No I/O.

use std::collections::BTreeSet;

use lattice_workflow_contract::Identity;
use serde::Serialize;

use super::CheckpointBinding;
use super::CheckpointOffer;
use super::CheckpointProfile;
use super::Manifest;
use super::Refusal;

mod admission;
mod adoption;
mod manifest;

pub use admission::Admission;
pub use admission::AdmissionInput;
pub use admission::AdmittedCheckpoint;
pub use admission::admit;
pub use admission::admit_runtime_facts;
pub use adoption::admit_route;
pub use adoption::adopted_nodes;
pub use adoption::adoption_record;
pub use adoption::restart_kind;
pub use adoption::restored_steps;
pub use adoption::serves_adoption;
pub use adoption::verify_carry_forward;
pub use manifest::ManifestInput;
pub use manifest::build_manifest;
pub use manifest::completed_node;
pub use manifest::completed_state;
pub use manifest::node_outputs_object;
pub use manifest::node_state;

const HEX_LENGTH: usize = blake3::OUT_LEN * 2;

/// The exchange identity framing: BLAKE3 over `u64be(len(domain)) ‖ domain ‖ u64be(len(bytes)) ‖
/// bytes`.
pub fn identity_of_bytes(domain: &str, bytes: &[u8]) -> Result<Identity, Refusal> {
    let domain_length = u64::try_from(domain.len()).map_err(|_| Refusal::Encoding)?;
    let value_length = u64::try_from(bytes.len()).map_err(|_| Refusal::Encoding)?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(&domain_length.to_be_bytes());
    hasher.update(domain.as_bytes());
    hasher.update(&value_length.to_be_bytes());
    hasher.update(bytes);
    Identity::new(format!("{}{}", lattice_workflow_contract::BLAKE3_PREFIX, hasher.finalize().to_hex()))
        .map_err(|_| Refusal::Encoding)
}

/// Canonical JSON bytes and their framed identity under `domain`.
pub fn canonical<T: Serialize>(domain: &str, value: &T) -> Result<(Vec<u8>, Identity), Refusal> {
    let bytes = serde_json::to_vec(value).map_err(|_| Refusal::Encoding)?;
    let identity = identity_of_bytes(domain, &bytes)?;
    Ok((bytes, identity))
}

/// Plain BLAKE3 of raw content, as 64 lowercase hex digits.
#[must_use]
pub fn content_hex(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

/// Hex digest of a `b3:` identity, used as a content-addressed object file name.
#[must_use]
pub fn identity_hex(identity: &Identity) -> &str {
    identity.as_str().strip_prefix(lattice_workflow_contract::BLAKE3_PREFIX).unwrap_or_default()
}

fn is_hex_digest(value: &str) -> bool {
    value.len() == HEX_LENGTH && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn is_bounded_text(value: &str) -> bool {
    !value.is_empty() && value.len() <= super::MAXIMUM_TEXT_BYTES && value.bytes().all(|byte| byte.is_ascii_graphic())
}

// r[impl lattice.application_checkpoint.unavailable]
/// What a route advertises. Only a binding that declares a profile offers resume.
#[must_use]
pub fn checkpoint_offer(binding: Option<&CheckpointBinding>) -> CheckpointOffer {
    match binding.map(|binding| binding.checkpoint) {
        Some(CheckpointProfile::WorkflowNodeBoundaryV1) => {
            CheckpointOffer::Declared(CheckpointProfile::WorkflowNodeBoundaryV1)
        }
        Some(CheckpointProfile::None) | None => CheckpointOffer::Unavailable,
    }
}

pub fn validate_binding(binding: &CheckpointBinding) -> Result<(), Refusal> {
    if binding.schema != super::BINDING_SCHEMA {
        return Err(Refusal::Schema);
    }
    if !super::REQUIRED_NON_CLAIMS.iter().all(|claim| binding.non_claims.iter().any(|value| value == claim)) {
        return Err(Refusal::Schema);
    }
    if binding.environment.as_deref().is_some_and(|value| !is_bounded_text(value)) {
        return Err(Refusal::Environment);
    }
    if binding.root.as_os_str().is_empty() || !binding.root.is_absolute() {
        return Err(Refusal::WorkspacePath);
    }
    if binding.workspace.as_ref().is_some_and(|workspace| !workspace.is_absolute()) {
        return Err(Refusal::WorkspacePath);
    }
    Ok(())
}

pub fn validate_admission_reference(reference: &str) -> Result<(), Refusal> {
    if is_bounded_text(reference) {
        Ok(())
    } else {
        Err(Refusal::AdmissionReference)
    }
}

// r[impl lattice.application_checkpoint.workspace]
/// A workspace path is relative, normalized and free of parent or current-directory parts.
#[must_use]
pub fn is_workspace_path(path: &str) -> bool {
    !path.is_empty()
        && path.len() <= super::MAXIMUM_WORKSPACE_PATH_BYTES
        && !path.starts_with('/')
        && !path.contains('\0')
        && path.split('/').all(|part| !part.is_empty() && part != "." && part != "..")
}

pub fn validate_manifest(manifest: &Manifest) -> Result<(), Refusal> {
    if manifest.schema != super::MANIFEST_SCHEMA {
        return Err(Refusal::Schema);
    }
    let execution = &manifest.execution;
    if execution.accepted.schema != lattice_workflow_contract::ACCEPTED_SCHEMA
        || execution.accepted.request_identity != execution.request
        || execution.accepted.workflow_identity != execution.workflow_revision
    {
        return Err(Refusal::CrossedRequest);
    }
    if !is_bounded_text(&manifest.architecture)
        || !is_bounded_text(&manifest.runtime.lattice_revision)
        || !is_hex_digest(&manifest.runtime.lattice_binary_blake3)
        || manifest.runtime.environment.as_deref().is_some_and(|value| !is_bounded_text(value))
    {
        return Err(Refusal::Runtime);
    }
    let node_count = manifest.completed_nodes.len().saturating_add(manifest.uncertain_nodes.len());
    if node_count > super::MAXIMUM_COMPLETED_NODES {
        return Err(Refusal::NodeLimit);
    }
    let completed = strictly_increasing(manifest.completed_nodes.iter().map(|node| node.workflow_node_id))?;
    let uncertain = strictly_increasing(manifest.uncertain_nodes.iter().map(|node| node.workflow_node_id))?;
    if !completed.is_disjoint(&uncertain) {
        return Err(Refusal::DuplicateNode);
    }
    if manifest.workspace_objects.len() > super::MAXIMUM_WORKSPACE_OBJECTS {
        return Err(Refusal::NodeLimit);
    }
    let mut previous: Option<&str> = None;
    for object in &manifest.workspace_objects {
        if !is_workspace_path(&object.path)
            || !is_hex_digest(&object.blake3)
            || object.bytes > super::MAXIMUM_OBJECT_BYTES
            || previous.is_some_and(|previous| previous >= object.path.as_str())
        {
            return Err(Refusal::WorkspacePath);
        }
        previous = Some(&object.path);
    }
    Ok(())
}

fn strictly_increasing(ids: impl Iterator<Item = u64>) -> Result<BTreeSet<u64>, Refusal> {
    let mut seen = BTreeSet::new();
    let mut previous = None;
    for id in ids {
        if previous.is_some_and(|previous| previous >= id) {
            return Err(Refusal::DuplicateNode);
        }
        previous = Some(id);
        seen.insert(id);
    }
    Ok(seen)
}

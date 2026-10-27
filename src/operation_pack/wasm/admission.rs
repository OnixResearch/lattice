use std::collections::BTreeSet;

use super::super::admission;
use super::super::model::AdmittedPack;
use super::Blocker;
use super::DispatchFacts;
use super::DispatchPlan;

// r[impl lattice.operation_packs_wasm.lock]
// r[impl lattice.operation_packs_wasm.functional_core]
// r[impl lattice.operation_packs_wasm.world]
// r[impl lattice.operation_packs_wasm.fallback]
pub fn admit_dispatch(
    admitted: &AdmittedPack,
    runtime_id: &str,
    type_id: &str,
    method: &str,
    facts: &DispatchFacts,
) -> Result<DispatchPlan, Vec<Blocker>> {
    let candidate = &admitted.candidate;
    if admission::validate_candidate(candidate).is_err() {
        return Err(vec![Blocker::Candidate]);
    }
    if admission::validate_lock(candidate, &admitted.lock).is_err() {
        return Err(vec![Blocker::Lock]);
    }
    let manifest = &candidate.manifest;
    let runtime = manifest
        .runtimes
        .iter()
        .find(|runtime| runtime.id == runtime_id)
        .ok_or_else(|| vec![Blocker::RuntimeDeclaration])?;
    if runtime.kind != super::RUNTIME_KIND {
        return Err(vec![Blocker::RuntimeDeclaration]);
    }
    let descriptor = runtime.wasm.as_ref().ok_or_else(|| vec![Blocker::RuntimeDeclaration])?;
    let content_identity = |id: &str| {
        manifest
            .contents
            .iter()
            .find(|content| content.id == id)
            .map(|content| content.content_identity.as_str())
    };
    let component_identity = content_identity(&runtime.content_id).ok_or_else(|| vec![Blocker::RuntimeDeclaration])?;
    let wit_identity = content_identity(&descriptor.wit_content_id).ok_or_else(|| vec![Blocker::RuntimeDeclaration])?;
    let bundle_identity =
        content_identity(&descriptor.bundle_content_id).ok_or_else(|| vec![Blocker::RuntimeDeclaration])?;
    let mut blockers = Vec::new();
    if !facts.runtime_available {
        blockers.push(Blocker::RuntimeUnavailable);
    }
    if facts.platform != admitted.lock.platform || !runtime.platforms.contains(&facts.platform) {
        blockers.push(Blocker::Platform);
    }
    if !facts.bundle.valid
        || !facts.bundle.current
        || facts.bundle.overclaimed
        || facts.bundle.bundle_identity != bundle_identity
        || facts.bundle.component_identity != component_identity
        || facts.bundle.wit_identity != wit_identity
    {
        blockers.push(Blocker::Bundle);
    }
    if facts.bundle.verifier_revision != descriptor.verifier_revision {
        blockers.push(Blocker::VerifierRevision);
    }
    if facts.runtime_revision != descriptor.runtime_revision {
        blockers.push(Blocker::RuntimeRevision);
    }
    if facts.runtime_profile_identity != descriptor.runtime_profile_identity {
        blockers.push(Blocker::RuntimeProfile);
    }
    if facts.world.wit_package != descriptor.wit_package
        || facts.world.world != descriptor.world
        || facts.world.export_interface != descriptor.export_interface
        || facts.world.invoke != descriptor.invoke
    {
        blockers.push(Blocker::World);
    }
    if !facts.world.imports.is_empty() {
        blockers.push(Blocker::Imports);
    }
    let operation = manifest
        .operations
        .iter()
        .find(|operation| operation.type_id == type_id && operation.runtime_id == runtime_id);
    let mapping = descriptor.methods.iter().find(|mapping| mapping.type_id == type_id && mapping.method == method);
    if operation.is_none_or(|operation| !operation.methods.iter().any(|declared| declared == method))
        || mapping.is_none()
    {
        blockers.push(Blocker::Method);
    }
    if facts.resources != descriptor.resources || facts.input_bytes > descriptor.resources.maximum_input_bytes {
        blockers.push(Blocker::Resources);
    }
    let capability_set = facts.authority.capabilities.iter().collect::<BTreeSet<_>>();
    let declared_capabilities = runtime.capabilities.iter().collect::<BTreeSet<_>>();
    if !facts.authority.operation_admitted
        || facts.authority.pack_identity != candidate.pack_identity
        || facts.authority.lock_identity != admitted.lock.identity
        || facts.authority.type_id != type_id
        || facts.authority.method != method
        || operation.is_none_or(|operation| facts.authority.type_revision != operation.type_revision)
        || capability_set.len() != facts.authority.capabilities.len()
        || capability_set != declared_capabilities
    {
        blockers.push(Blocker::Authority);
    }
    if !blockers.is_empty() {
        return Err(blockers);
    }
    let operation = operation.ok_or_else(|| vec![Blocker::Method])?;
    let mapping = mapping.ok_or_else(|| vec![Blocker::Method])?;
    let cohort_identity =
        admitted.lock.component_cohort_identities.get(runtime_id).ok_or_else(|| vec![Blocker::Lock])?;
    Ok(DispatchPlan {
        runtime_id: runtime_id.to_string(),
        type_id: type_id.to_string(),
        type_revision: operation.type_revision.clone(),
        method: method.to_string(),
        guest_method: mapping.guest_method.clone(),
        component_identity: component_identity.to_string(),
        wit_identity: wit_identity.to_string(),
        bundle_identity: bundle_identity.to_string(),
        cohort_identity: cohort_identity.clone(),
        descriptor: descriptor.clone(),
    })
}

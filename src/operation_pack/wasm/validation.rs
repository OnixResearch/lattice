use std::collections::BTreeSet;

use super::super::Error;
use super::super::model;
use super::super::validation::primitives;

pub(crate) fn normalize(descriptor: &mut super::Descriptor) -> Result<(), Error> {
    descriptor.methods.sort();
    descriptor.effect_intent_targets.sort();
    primitives::sort_unique(&mut descriptor.required_non_claims)?;
    Ok(())
}

pub(crate) fn validate_descriptor(manifest: &model::Manifest, runtime: &model::RuntimeArtifact) -> Result<(), Error> {
    let descriptor = runtime.wasm.as_ref().ok_or(Error::Invalid("component descriptor missing"))?;
    if descriptor.wit_package != super::WIT_PACKAGE
        || descriptor.world != super::WORLD
        || descriptor.export_interface != super::EXPORT_INTERFACE
        || descriptor.invoke != super::INVOKE
        || descriptor.verifier_revision != super::VERIFIER_REVISION
        || descriptor.result_schema != super::RESULT_SCHEMA
        || descriptor.effect_intent_schema != super::EFFECT_INTENT_SCHEMA
    {
        return Err(Error::Invalid("component contract version"));
    }
    validate_revision(&descriptor.runtime_revision)?;
    primitives::validate_identity(&descriptor.runtime_profile_identity)?;
    let required_evidence = manifest.admission.required_evidence.iter().copied().collect::<BTreeSet<_>>();
    let declared_evidence = descriptor.evidence_identities.keys().copied().collect::<BTreeSet<_>>();
    if required_evidence != declared_evidence {
        return Err(Error::Invalid("component evidence closure"));
    }
    for identity in descriptor.evidence_identities.values() {
        primitives::validate_identity(identity)?;
    }
    let component = content(manifest, &runtime.content_id, model::ContentRole::WasmComponent)?;
    let wit = content(manifest, &descriptor.wit_content_id, model::ContentRole::WitSource)?;
    let bundle = content(manifest, &descriptor.bundle_content_id, model::ContentRole::MantleBundle)?;
    if descriptor.methods.is_empty()
        || descriptor.methods.len() > model::MAXIMUM_OPERATIONS * crate::operation::model::MAXIMUM_TYPE_METHODS
    {
        return Err(Error::Invalid("component method count"));
    }
    let mut methods = BTreeSet::new();
    let mut guest_methods = BTreeSet::new();
    for mapping in &descriptor.methods {
        primitives::validate_scoped_id(&mapping.type_id)?;
        primitives::validate_id(&mapping.method)?;
        primitives::validate_id(&mapping.guest_method)?;
        if !methods.insert((&mapping.type_id, &mapping.method)) || !guest_methods.insert(&mapping.guest_method) {
            return Err(Error::Invalid("duplicate component method"));
        }
    }
    if descriptor.required_non_claims.len() != super::REQUIRED_NON_CLAIMS.len()
        || super::REQUIRED_NON_CLAIMS
            .iter()
            .any(|required| !descriptor.required_non_claims.iter().any(|value| value == required))
    {
        return Err(Error::Invalid("component non-claims"));
    }
    let resources = &descriptor.resources;
    let input_bytes = u64::try_from(runtime.input_bytes).map_err(|_| Error::Invalid("component input bound"))?;
    let output_bytes = u64::try_from(runtime.output_bytes).map_err(|_| Error::Invalid("component output bound"))?;
    let deadline_millis =
        runtime.timeout_seconds.checked_mul(1000).ok_or(Error::Invalid("component deadline overflow"))?;
    if resources.fuel == 0
        || resources.memory_bytes == 0
        || resources.stack_bytes == 0
        || resources.table_elements == 0
        || resources.instances == 0
        || resources.tables == 0
        || resources.memories == 0
        || resources.maximum_concurrency != 1
        || resources.maximum_input_bytes != input_bytes
        || resources.maximum_output_bytes != output_bytes
        || resources.maximum_component_bytes == 0
        || resources.maximum_component_bytes > model::MAXIMUM_PACK_BYTES
        || resources.maximum_wit_bytes == 0
        || resources.maximum_wit_bytes > model::MAXIMUM_PACK_BYTES
        || component.byte_length > resources.maximum_component_bytes
        || wit.byte_length > resources.maximum_wit_bytes
        || resources.deadline_millis != deadline_millis
        || bundle.byte_length > model::MAXIMUM_MANIFEST_BYTES
    {
        return Err(Error::Invalid("component resource profile"));
    }
    validate_effect_targets(manifest, runtime, descriptor)
}

pub(crate) fn validate_method_closure(manifest: &model::Manifest) -> Result<(), Error> {
    for runtime in &manifest.runtimes {
        let Some(descriptor) = &runtime.wasm else { continue };
        let declared = descriptor
            .methods
            .iter()
            .map(|mapping| (&mapping.type_id, &mapping.method))
            .collect::<BTreeSet<_>>();
        let bound = manifest
            .operations
            .iter()
            .filter(|operation| operation.runtime_id == runtime.id)
            .flat_map(|operation| operation.methods.iter().map(move |method| (&operation.type_id, method)))
            .collect::<BTreeSet<_>>();
        if declared != bound {
            return Err(Error::Invalid("component method closure"));
        }
    }
    Ok(())
}

fn validate_effect_targets(
    manifest: &model::Manifest,
    runtime: &model::RuntimeArtifact,
    descriptor: &super::Descriptor,
) -> Result<(), Error> {
    if descriptor.maximum_effect_intents > super::MAXIMUM_EFFECT_INTENTS
        || descriptor.effect_intent_targets.len() > super::MAXIMUM_EFFECT_INTENTS
        || (descriptor.maximum_effect_intents == 0) != descriptor.effect_intent_targets.is_empty()
    {
        return Err(Error::Invalid("component effect bounds"));
    }
    let mut targets = BTreeSet::new();
    for target in &descriptor.effect_intent_targets {
        primitives::validate_scoped_id(&target.type_id)?;
        primitives::validate_id(&target.method)?;
        if !targets.insert((&target.type_id, &target.method)) {
            return Err(Error::Invalid("duplicate component effect target"));
        }
        let operation = manifest
            .operations
            .iter()
            .find(|operation| operation.type_id == target.type_id)
            .ok_or(Error::Invalid("component effect operation"))?;
        let target_runtime = manifest
            .runtimes
            .iter()
            .find(|candidate| candidate.id == operation.runtime_id)
            .ok_or(Error::Invalid("component effect runtime"))?;
        if target_runtime.kind != "bounded_process"
            || !operation.methods.contains(&target.method)
            || target_runtime.capabilities.iter().any(|capability| !runtime.capabilities.contains(capability))
        {
            return Err(Error::Invalid("component effect authority"));
        }
    }
    Ok(())
}

fn content<'a>(
    manifest: &'a model::Manifest,
    id: &str,
    role: model::ContentRole,
) -> Result<&'a model::ContentItem, Error> {
    primitives::validate_id(id)?;
    let content = manifest
        .contents
        .iter()
        .find(|content| content.id == id)
        .ok_or(Error::Invalid("component content missing"))?;
    if content.role != role {
        return Err(Error::Invalid("component content role"));
    }
    Ok(content)
}

fn validate_revision(revision: &str) -> Result<(), Error> {
    if revision.len() != 40 || !revision.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)) {
        return Err(Error::Invalid("component runtime revision"));
    }
    Ok(())
}

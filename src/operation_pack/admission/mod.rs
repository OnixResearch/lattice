#![allow(
    unknown_lints,
    non_trait_imports,
    reason = "admission formulas use ordered maps and sets throughout their exact closure checks"
)]
#![allow(
    assertion_density,
    reason = "untrusted pack, lock, and admission facts return typed errors instead of terminating the process"
)]
#![allow(
    compound_condition,
    reason = "each compound predicate is one closed exact-binding rejection rule"
)]

use std::collections::BTreeMap;
use std::collections::BTreeSet;

// r[impl lattice.operation_packs.admission.valid]
// r[impl lattice.operation_packs.admission.denied]
pub fn admit(
    candidate: &super::model::Candidate,
    lock: &super::model::Lock,
    facts: &super::model::AdmissionFacts,
) -> Result<super::model::AdmittedPack, super::Error> {
    validate_candidate(candidate)?;
    validate_facts(candidate, facts)?;
    validate_lock(candidate, lock)?;
    if lock.platform != facts.platform {
        return Err(super::Error::Invalid("lock platform fact"));
    }
    Ok(super::model::AdmittedPack {
        candidate: candidate.clone(),
        lock: lock.clone(),
    })
}

pub fn create_lock(
    candidate: &super::model::Candidate,
    facts: &super::model::AdmissionFacts,
) -> Result<super::model::Lock, super::Error> {
    validate_candidate(candidate)?;
    validate_facts(candidate, facts)?;
    let manifest = &candidate.manifest;
    let runtime_identities = manifest
        .runtimes
        .iter()
        .map(|runtime| Ok((runtime.id.clone(), super::identity::runtime_identity(runtime)?)))
        .collect::<Result<BTreeMap<_, _>, super::Error>>()?;
    let analyzer_identities = manifest
        .analyzers
        .iter()
        .map(|analyzer| Ok((analyzer.id.clone(), super::identity::analyzer_identity(analyzer)?)))
        .collect::<Result<BTreeMap<_, _>, super::Error>>()?;
    let skill_identities = manifest
        .skills
        .iter()
        .map(|skill| Ok((skill.name.clone(), super::identity::skill_identity(skill)?)))
        .collect::<Result<BTreeMap<_, _>, super::Error>>()?;
    let dependency_identities = manifest
        .dependencies
        .iter()
        .map(|dependency| (dependency.pack_id.clone(), dependency.pack_identity.clone()))
        .collect();
    let mut lock = super::model::Lock {
        schema: super::model::LOCK_SCHEMA.to_string(),
        identity: String::new(),
        pack_id: manifest.pack_id.clone(),
        pack_identity: candidate.pack_identity.clone(),
        manifest_identity: candidate.manifest_identity.clone(),
        dependency_identities,
        platform: facts.platform.clone(),
        runtime_identities,
        analyzer_identities,
        skill_identities,
        trust_profile_identity: manifest.admission.identity.clone(),
        source_locator: candidate.source_locator.clone(),
    };
    lock.identity = super::identity::lock_identity(&lock)?;
    validate_lock(candidate, &lock)?;
    Ok(lock)
}

pub fn validate_lock(candidate: &super::model::Candidate, lock: &super::model::Lock) -> Result<(), super::Error> {
    if lock.schema != super::model::LOCK_SCHEMA
        || lock.pack_id != candidate.manifest.pack_id
        || lock.pack_identity != candidate.pack_identity
        || lock.manifest_identity != candidate.manifest_identity
        || lock.trust_profile_identity != candidate.manifest.admission.identity
        || !candidate.manifest.platforms.contains(&lock.platform)
        || candidate.manifest.runtimes.iter().any(|runtime| !runtime.platforms.contains(&lock.platform))
        || super::validation::validate_identity(&lock.source_locator).is_err()
    {
        return Err(super::Error::Invalid("lock binding"));
    }
    let expected_dependencies = candidate
        .manifest
        .dependencies
        .iter()
        .map(|dependency| (dependency.pack_id.clone(), dependency.pack_identity.clone()))
        .collect::<BTreeMap<_, _>>();
    if lock.dependency_identities != expected_dependencies {
        return Err(super::Error::Invalid("lock dependency closure"));
    }
    let expected_runtimes = candidate
        .manifest
        .runtimes
        .iter()
        .map(|runtime| Ok((runtime.id.clone(), super::identity::runtime_identity(runtime)?)))
        .collect::<Result<BTreeMap<_, _>, super::Error>>()?;
    let expected_analyzers = candidate
        .manifest
        .analyzers
        .iter()
        .map(|analyzer| Ok((analyzer.id.clone(), super::identity::analyzer_identity(analyzer)?)))
        .collect::<Result<BTreeMap<_, _>, super::Error>>()?;
    let expected_skills = candidate
        .manifest
        .skills
        .iter()
        .map(|skill| Ok((skill.name.clone(), super::identity::skill_identity(skill)?)))
        .collect::<Result<BTreeMap<_, _>, super::Error>>()?;
    if lock.runtime_identities != expected_runtimes
        || lock.analyzer_identities != expected_analyzers
        || lock.skill_identities != expected_skills
        || lock.identity != super::identity::lock_identity(lock)?
    {
        return Err(super::Error::Invalid("lock identity"));
    }
    Ok(())
}

pub fn validate_candidate(candidate: &super::model::Candidate) -> Result<(), super::Error> {
    let normalized = super::validation::validate_manifest(candidate.manifest.clone())?;
    if normalized != candidate.manifest
        || candidate.manifest_identity != super::identity::manifest_identity(&candidate.manifest)?
        || candidate.pack_identity != super::identity::pack_identity(&candidate.manifest)?
    {
        return Err(super::Error::Invalid("candidate identity"));
    }
    if candidate.measured_contents.len() != candidate.manifest.contents.len() {
        return Err(super::Error::Invalid("measured content count"));
    }
    for declared in &candidate.manifest.contents {
        let measured = candidate
            .measured_contents
            .iter()
            .find(|measured| measured.id == declared.id)
            .ok_or(super::Error::Invalid("measured content"))?;
        if measured.path != declared.path
            || measured.content_identity != declared.content_identity
            || measured.byte_length != declared.byte_length
        {
            return Err(super::Error::Invalid("measured content binding"));
        }
    }
    Ok(())
}

fn validate_facts(
    candidate: &super::model::Candidate,
    facts: &super::model::AdmissionFacts,
) -> Result<(), super::Error> {
    let manifest = &candidate.manifest;
    if facts.trusted_pack_identity != candidate.pack_identity || !manifest.platforms.contains(&facts.platform) {
        return Err(super::Error::Invalid("pack trust or platform"));
    }
    for dependency in &manifest.dependencies {
        let observed = facts
            .dependency_identities
            .get(&dependency.pack_id)
            .ok_or(super::Error::Invalid("dependency missing"))?;
        if observed != &dependency.pack_identity {
            return Err(super::Error::Invalid("dependency mismatch"));
        }
    }
    let declared_dependencies =
        manifest.dependencies.iter().map(|dependency| &dependency.pack_id).collect::<BTreeSet<_>>();
    if facts.dependency_identities.keys().any(|pack_id| !declared_dependencies.contains(pack_id)) {
        return Err(super::Error::Invalid("dependency ambiguity"));
    }
    let profile_licenses = manifest.admission.allowed_licenses.iter().collect::<BTreeSet<_>>();
    let fact_licenses = facts.allowed_licenses.iter().collect::<BTreeSet<_>>();
    if profile_licenses != fact_licenses {
        return Err(super::Error::Invalid("license profile fact"));
    }
    for license in &manifest.licenses {
        if !manifest.admission.allowed_licenses.contains(license) {
            return Err(super::Error::Invalid("license policy"));
        }
    }
    let declared_capabilities =
        manifest.runtimes.iter().flat_map(|runtime| runtime.capabilities.iter()).collect::<BTreeSet<_>>();
    let profile_capabilities = manifest.admission.allowed_capabilities.iter().collect::<BTreeSet<_>>();
    let fact_capabilities = facts.allowed_capabilities.iter().collect::<BTreeSet<_>>();
    if profile_capabilities != fact_capabilities {
        return Err(super::Error::Invalid("capability profile fact"));
    }
    for capability in declared_capabilities {
        if !manifest.admission.allowed_capabilities.contains(capability) {
            return Err(super::Error::Invalid("capability widening"));
        }
    }
    for (kind, evidence) in &facts.evidence {
        if !evidence.valid || !evidence.current || evidence.overclaimed {
            return Err(super::Error::Invalid("present evidence"));
        }
        if super::validation::validate_identity(&evidence.identity).is_err()
            || !manifest.admission.required_evidence.contains(kind)
        {
            return Err(super::Error::Invalid("undeclared evidence"));
        }
    }
    for kind in &manifest.admission.required_evidence {
        if !facts.evidence.contains_key(kind) {
            return Err(super::Error::Invalid("required evidence"));
        }
    }
    Ok(())
}

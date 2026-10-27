#![allow(
    unknown_lints,
    path_segment_repetition,
    reason = "identity constructors keep their authoritative artifact kind explicit at every call site"
)]

const MANIFEST_DOMAIN: &[u8] = b"lattice.operation-pack-manifest.v1";
const PACK_DOMAIN: &[u8] = b"lattice.operation-pack.v1";
const RUNTIME_DOMAIN: &[u8] = b"lattice.operation-pack-runtime.v1";
const ANALYZER_DOMAIN: &[u8] = b"lattice.operation-pack-analyzer.v1";
const SKILL_DOMAIN: &[u8] = b"lattice.operation-pack-skill.v1";
const LOCK_DOMAIN: &[u8] = b"lattice.operation-pack-lock.v1";

pub fn content_identity(bytes: &[u8]) -> String {
    format!("b3:{}", blake3::hash(bytes).to_hex())
}

pub fn manifest_identity(manifest: &super::model::Manifest) -> Result<String, super::Error> {
    hash_serializable(MANIFEST_DOMAIN, &authoritative_manifest(manifest))
}

pub fn pack_identity(manifest: &super::model::Manifest) -> Result<String, super::Error> {
    let manifest_identity = manifest_identity(manifest)?;
    let contents = manifest
        .contents
        .iter()
        .map(|content| {
            (&content.id, &content.role, &content.content_identity, content.byte_length, &content.media_class)
        })
        .collect::<Vec<_>>();
    hash_serializable(PACK_DOMAIN, &(manifest_identity, contents))
}

pub fn runtime_identity(runtime: &super::model::RuntimeArtifact) -> Result<String, super::Error> {
    hash_serializable(RUNTIME_DOMAIN, runtime)
}

/// Component runtimes bind their content and all authoritative pack declarations.
/// The bounded-process identity contract is intentionally unchanged.
pub fn runtime_cohort_identity(
    manifest: &super::model::Manifest,
    runtime: &super::model::RuntimeArtifact,
) -> Result<String, super::Error> {
    if runtime.wasm.is_none() {
        return runtime_identity(runtime);
    }
    hash_serializable(b"lattice.operation-pack-wasm-runtime.v1", &(runtime, manifest_identity(manifest)?))
}

pub fn component_cohort_identity(
    candidate: &super::model::Candidate,
    runtime: &super::model::RuntimeArtifact,
    platform: &str,
    evidence: &std::collections::BTreeMap<super::model::EvidenceKind, String>,
) -> Result<String, super::Error> {
    hash_serializable(
        b"lattice.operation-pack-wasm-cohort.v1",
        &(
            &candidate.pack_identity,
            &candidate.manifest_identity,
            runtime_cohort_identity(&candidate.manifest, runtime)?,
            platform,
            evidence,
        ),
    )
}

pub fn analyzer_identity(analyzer: &super::model::AnalyzerManifest) -> Result<String, super::Error> {
    hash_serializable(ANALYZER_DOMAIN, analyzer)
}

pub fn skill_identity(skill: &super::model::SkillManifest) -> Result<String, super::Error> {
    hash_serializable(SKILL_DOMAIN, skill)
}

pub fn lock_identity(lock: &super::model::Lock) -> Result<String, super::Error> {
    let projection = (
        &lock.schema,
        &lock.pack_id,
        &lock.pack_identity,
        &lock.manifest_identity,
        &lock.dependency_identities,
        &lock.platform,
        &lock.runtime_identities,
        &lock.analyzer_identities,
        &lock.skill_identities,
        &lock.trust_profile_identity,
    );
    if lock.component_cohort_identities.is_empty() && lock.evidence_identities.is_empty() {
        hash_serializable(LOCK_DOMAIN, &projection)
    } else {
        hash_serializable(LOCK_DOMAIN, &(projection, &lock.component_cohort_identities, &lock.evidence_identities))
    }
}

fn authoritative_manifest(manifest: &super::model::Manifest) -> impl serde::Serialize + '_ {
    (
        &manifest.schema,
        &manifest.pack_id,
        &manifest.contents,
        &manifest.operations,
        &manifest.runtimes,
        &manifest.analyzers,
        &manifest.skills,
        &manifest.dependencies,
        &manifest.platforms,
        &manifest.admission,
        &manifest.licenses,
        &manifest.probe,
        &manifest.required_non_claims,
    )
}

fn hash_serializable(domain: &[u8], value: &impl serde::Serialize) -> Result<String, super::Error> {
    let bytes = serde_json::to_vec(value).map_err(|_| super::Error::Encoding)?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain);
    let length_bytes = u64::try_from(bytes.len()).map_err(|_| super::Error::Encoding)?;
    hasher.update(&length_bytes.to_le_bytes());
    hasher.update(&bytes);
    Ok(format!("b3:{}", hasher.finalize().to_hex()))
}

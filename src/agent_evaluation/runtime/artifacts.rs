#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArtifactCandidate {
    pub role: super::model::ArtifactRole,
    pub media: super::model::MediaClass,
    pub relative_path: String,
    pub content_identity: String,
    pub byte_count: u64,
    pub symlink: bool,
    pub remote: bool,
    pub secret_bearing: bool,
    pub hidden_material_bearing: bool,
    pub redacted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ArtifactEntry {
    pub role: super::model::ArtifactRole,
    pub media: super::model::MediaClass,
    pub safe_reference: String,
    pub content_identity: String,
    pub byte_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ArtifactManifest {
    pub entries: Vec<ArtifactEntry>,
    pub total_bytes: u64,
    pub identity: String,
}

// r[impl lattice.agent_evaluation.evidence]
pub fn admit(
    case: &super::model::AdmittedCase,
    candidates: &[ArtifactCandidate],
) -> Result<ArtifactManifest, super::Error> {
    let policy = &case.definition.artifacts;
    debug_assert!(policy.max_count > 0);
    debug_assert!(policy.max_bytes > 0);
    let candidate_count = u32::try_from(candidates.len()).map_err(|_| super::Error::InvalidArtifact)?;
    if candidate_count > policy.max_count {
        return Err(super::Error::InvalidArtifact);
    }
    let mut paths = std::collections::BTreeSet::new();
    let mut entries = Vec::with_capacity(candidates.len());
    let mut total_bytes = 0u64;
    for candidate in candidates {
        validate_candidate(policy, candidate, &mut paths)?;
        total_bytes = total_bytes.checked_add(candidate.byte_count).ok_or(super::Error::InvalidArtifact)?;
        if total_bytes > policy.max_bytes {
            return Err(super::Error::InvalidArtifact);
        }
        entries.push(ArtifactEntry {
            role: candidate.role.clone(),
            media: candidate.media.clone(),
            safe_reference: candidate.relative_path.clone(),
            content_identity: candidate.content_identity.clone(),
            byte_count: candidate.byte_count,
        });
    }
    entries.sort_by(|left, right| left.safe_reference.cmp(&right.safe_reference));
    let identity = super::identity::hash_value(
        "lattice.agent-evaluation.artifact-manifest.v1",
        &(policy.identity.as_str(), &entries, total_bytes),
    )?;
    debug_assert_eq!(entries.len(), candidates.len());
    debug_assert!(total_bytes <= policy.max_bytes);
    Ok(ArtifactManifest {
        entries,
        total_bytes,
        identity,
    })
}

fn validate_candidate(
    policy: &super::model::ArtifactPolicy,
    candidate: &ArtifactCandidate,
    paths: &mut std::collections::BTreeSet<String>,
) -> Result<(), super::Error> {
    debug_assert!(policy.max_count > 0);
    debug_assert!(policy.max_bytes > 0);
    let is_unsupported_role = !policy.allowed_roles.contains(&candidate.role);
    let is_unsupported_media = !policy.allowed_media.contains(&candidate.media);
    let is_unsafe_path = !super::identity::valid_safe_path(&candidate.relative_path);
    let is_duplicate_path = !paths.insert(candidate.relative_path.clone());
    let is_empty_artifact = candidate.byte_count == 0;
    let is_unsafe_source = candidate.symlink || candidate.remote;
    let has_private_content = candidate.secret_bearing || candidate.hidden_material_bearing;
    let is_missing_redaction = policy.redaction_required && !candidate.redacted;
    let has_invalid_candidate = [
        is_unsupported_role,
        is_unsupported_media,
        is_unsafe_path,
        is_duplicate_path,
        is_empty_artifact,
        is_unsafe_source,
        has_private_content,
        is_missing_redaction,
    ]
    .into_iter()
    .any(|value| value);
    if has_invalid_candidate {
        return Err(super::Error::InvalidArtifact);
    }
    super::identity::validate(&candidate.content_identity)
}

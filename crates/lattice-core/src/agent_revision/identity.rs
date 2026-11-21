//! Identity roles of the revision lifecycle, spelled `blake3:<hex>` and
//! framed by `content_identity_core` under Lattice-owned domains.

// r[impl lattice.agent_revisions.identities]

pub const BUNDLE_DOMAIN: &str = "lattice-agent-bundle-v1";
pub const PREPARATION_DOMAIN: &str = "lattice-agent-revision-preparation-v1";
pub const POLICY_DOMAIN: &str = "lattice-agent-revision-promotion-policy-v1";
pub const PROMOTION_DOMAIN: &str = "lattice-agent-revision-promotion-v1";
pub const PIN_DOMAIN: &str = "lattice-agent-revision-session-pin-v1";
pub const PAIRING_DOMAIN: &str = "lattice-workflow-agent-definition-v1";
pub const COMPOSITE_DOMAIN: &str = "lattice-workflow-agent-revision-v1";

/// Framing failed; only a malformed domain or oversized part can cause it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Unframed;

fn finish(hasher: content_identity_core::IdentityHasher) -> String {
    format!("blake3:{}", hasher.finish().to_hex())
}

fn hasher(domain: &str) -> Result<content_identity_core::IdentityHasher, Unframed> {
    let domain = content_identity_core::Domain::new(domain).map_err(|_| Unframed)?;
    Ok(content_identity_core::IdentityHasher::new(domain))
}

/// Identity of a resolved bundle over its canonical JSON bytes.
pub fn bundle(canonical: &[u8]) -> Result<String, Unframed> {
    let mut hasher = hasher(BUNDLE_DOMAIN)?;
    hasher.update_hashed_part(b"bundle", canonical);
    Ok(finish(hasher))
}

/// Definition identity of one workflow and worker pairing: its name and the
/// worker definition. Stable across plan and worker revisions.
pub fn pairing(binding: &super::workflow::Binding) -> Result<String, Unframed> {
    let mut hasher = hasher(PAIRING_DOMAIN)?;
    hasher
        .update_tagged_str(b"name", &binding.name)
        .map_err(|_| Unframed)?
        .update_tagged_str(b"worker-definition", &binding.worker.definition)
        .map_err(|_| Unframed)?;
    Ok(finish(hasher))
}

/// Bare-hex revision of one workflow-agent binding over its canonical JSON
/// bytes, spelled like a profile revision so the lifecycle keys it the same.
pub fn composite(canonical: &[u8]) -> Result<String, Unframed> {
    let mut hasher = hasher(COMPOSITE_DOMAIN)?;
    hasher.update_hashed_part(b"binding", canonical);
    Ok(hasher.finish().to_hex().to_string())
}

/// Identity of one preparation attempt: definition, revision and the
/// desired generation it is bound to.
pub fn preparation(definition: &str, desired: &super::state::Desired) -> Result<String, Unframed> {
    let mut hasher = hasher(PREPARATION_DOMAIN)?;
    hasher
        .update_tagged_str(b"definition", definition)
        .map_err(|_| Unframed)?
        .update_tagged_str(b"revision", &desired.revision)
        .map_err(|_| Unframed)?
        .update_tagged_u64_le(b"desired-generation", desired.generation);
    Ok(finish(hasher))
}

/// Identity of a promotion policy: the task family and the required class.
pub fn policy(policy: &super::state::Policy) -> Result<String, Unframed> {
    let class = serde_json::to_string(&policy.required_class).map_err(|_| Unframed)?;
    let mut hasher = hasher(POLICY_DOMAIN)?;
    hasher
        .update_tagged_str(b"task-family", &policy.task_family)
        .map_err(|_| Unframed)?
        .update_tagged_str(b"required-class", &class)
        .map_err(|_| Unframed)?;
    Ok(finish(hasher))
}

/// Identity of one applied selection: definition, kind, revision,
/// qualification, policy and the rollout generation it created.
pub fn promotion(definition: &str, selection: &super::state::Selection) -> Result<String, Unframed> {
    let kind = serde_json::to_string(&selection.kind).map_err(|_| Unframed)?;
    let mut hasher = hasher(PROMOTION_DOMAIN)?;
    hasher
        .update_tagged_str(b"definition", definition)
        .map_err(|_| Unframed)?
        .update_tagged_str(b"kind", &kind)
        .map_err(|_| Unframed)?
        .update_tagged_str(b"revision", &selection.revision)
        .map_err(|_| Unframed)?
        .update_tagged_str(b"qualification-ref", &selection.qualification_ref)
        .map_err(|_| Unframed)?
        .update_tagged_str(b"policy", &selection.policy_identity)
        .map_err(|_| Unframed)?
        .update_tagged_str(b"request-id", &selection.request_id)
        .map_err(|_| Unframed)?
        .update_tagged_u64_le(b"rollout-generation", selection.rollout_generation);
    debug_assert!(selection.rollout_generation > 0);
    debug_assert!(!kind.is_empty());
    Ok(finish(hasher))
}

/// Identity of one session pin.
pub fn pin(lifecycle: &super::state::Lifecycle, session: &str, pin: &super::state::Pin) -> Result<String, Unframed> {
    let mut hasher = hasher(PIN_DOMAIN)?;
    hasher
        .update_tagged_str(b"definition", &lifecycle.definition)
        .map_err(|_| Unframed)?
        .update_tagged_str(b"session", session)
        .map_err(|_| Unframed)?
        .update_tagged_str(b"revision", &pin.revision)
        .map_err(|_| Unframed)?
        .update_tagged_str(b"qualification-ref", &pin.qualification_ref)
        .map_err(|_| Unframed)?
        .update_tagged_u64_le(b"rollout-generation", pin.rollout_generation);
    Ok(finish(hasher))
}

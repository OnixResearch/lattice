//! Deterministic admission of explicitly operator-supplied local effect grants.
//!
//! This module accepts evaluated values and an observed time, never ambient authority.

use std::collections::BTreeMap;
use std::path::PathBuf;

pub const PROFILE_SCHEMA: &str = "lattice.standalone-effect-authority.v1";
pub const MAX_PROFILE_GRANTS: usize = 64;
pub const MAX_PROFILE_ROOTS: usize = 64;
pub const MAX_ROOT_ID_BYTES: usize = 128;

const PROFILE_IDENTITY_DOMAIN: &[u8] = b"lattice.standalone-effect-authority.profile.v1";
const BLAKE3_PREFIX: &str = "blake3:";
const BLAKE3_HEX_BYTES: usize = 64;
const MAX_NODE_ID_DIGITS: usize = 20;
const NODE_PREFIX: &str = "lattice-node/";
const START_TRANSITION: &str = "ready->start";
const PROCESS_RESOURCE_PREFIX: &str = "lattice://process/";
const NETWORK_RESOURCE_PREFIX: &str = "lattice://network/";
const FILE_RESOURCE_PREFIX: &str = "lattice://file/";
const AGENT_RESOURCE_PREFIX: &str = "lattice://agent/";

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub schema: String,
    pub workflow_revision: String,
    pub machine_identity: String,
    pub policy_ref: String,
    pub expires_at_unix_ms: u64,
    pub grants: Vec<Grant>,
    pub roots: BTreeMap<String, PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Grant {
    pub node_id: u64,
    pub operation: String,
    pub resource: String,
    pub ability: String,
}

impl Grant {
    /// Return the reviewed policy contract only for a complete, closed grant tuple.
    #[must_use]
    pub fn contract_id(&self) -> Option<&'static str> {
        let (resource_prefix, contract) = match (self.operation.as_str(), self.ability.as_str()) {
            ("lattice/shell" | "lattice/operation", "process/execute") => (PROCESS_RESOURCE_PREFIX, "lattice-process"),
            ("lattice/http", "network/request") => (NETWORK_RESOURCE_PREFIX, "lattice-network"),
            ("lattice/file", "file/read" | "file/write" | "file/append") => (FILE_RESOURCE_PREFIX, "lattice-file"),
            ("lattice/agent", "agent/execute") => (AGENT_RESOURCE_PREFIX, "lattice-agent"),
            _ => return None,
        };
        let suffix = self.resource.strip_prefix(resource_prefix)?;
        (canonical_node_number(suffix) == Some(self.node_id)).then_some(contract)
    }
}

#[derive(Debug, Clone)]
pub struct AdmittedProfile {
    pub profile: Profile,
    pub identity: String,
}

/// Deliberately bounded diagnostics: no input strings, paths, or serialized values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfileError {
    InvalidProfile,
    Expired,
    BindingMismatch,
}

impl std::fmt::Display for ProfileError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::InvalidProfile => "invalid standalone authority profile",
            Self::Expired => "standalone authority profile expired",
            Self::BindingMismatch => "standalone authority binding mismatch",
        })
    }
}

impl std::error::Error for ProfileError {}

/// Admit the complete evaluated typed value. Source text and import paths are not identity.
pub fn admit(profile: Profile, now_ms: u64) -> Result<AdmittedProfile, ProfileError> {
    if profile.schema != PROFILE_SCHEMA
        || profile.policy_ref != crate::engine::effect::authority::CHOREGRAPH_EFFECT_AUTHORITY_POLICY_REF
    {
        return Err(ProfileError::InvalidProfile);
    }
    if !is_blake3_reference(&profile.workflow_revision) || !is_blake3_reference(&profile.machine_identity) {
        return Err(ProfileError::InvalidProfile);
    }
    if profile.expires_at_unix_ms == 0 || profile.grants.is_empty() {
        return Err(ProfileError::InvalidProfile);
    }
    if profile.grants.len() > MAX_PROFILE_GRANTS || profile.roots.len() > MAX_PROFILE_ROOTS {
        return Err(ProfileError::InvalidProfile);
    }
    for (index, grant) in profile.grants.iter().enumerate() {
        if grant.contract_id().is_none() || profile.grants[..index].iter().any(|prior| prior.node_id == grant.node_id) {
            return Err(ProfileError::InvalidProfile);
        }
    }
    if profile
        .roots
        .iter()
        .any(|(id, path)| id.is_empty() || id.len() > MAX_ROOT_ID_BYTES || !path.is_absolute())
    {
        return Err(ProfileError::InvalidProfile);
    }
    if now_ms >= profile.expires_at_unix_ms {
        return Err(ProfileError::Expired);
    }
    // Struct field order and BTreeMap key order make this serialization deterministic.
    // Serializing PathBuf also rejects non-Unicode paths rather than hashing a lossy view.
    let canonical = serde_json::to_string(&profile).map_err(|_| ProfileError::InvalidProfile)?;
    let identity = crate::engine::effect::authority::blake3_reference(PROFILE_IDENTITY_DOMAIN, &[canonical.as_str()]);
    Ok(AdmittedProfile { profile, identity })
}

/// Check only the local grant boundary; dispatch still validates payloads and reserves intent.
pub fn check_binding<'a>(
    admitted: &'a AdmittedProfile,
    binding: &crate::engine::effect::authority::IntentBinding,
    run_ref: &str,
    now_ms: u64,
) -> Result<&'a Grant, ProfileError> {
    let profile = &admitted.profile;
    if now_ms >= profile.expires_at_unix_ms {
        return Err(ProfileError::Expired);
    }
    if binding.schema != crate::engine::effect::authority::CHOREGRAPH_EFFECT_AUTHORITY_BINDING_SCHEMA
        || binding.policy.as_str() != crate::engine::effect::authority::CHOREGRAPH_EFFECT_AUTHORITY_POLICY_REF
    {
        return Err(ProfileError::BindingMismatch);
    }
    if binding.policy.as_str() != profile.policy_ref
        || binding.workflow_revision.as_str() != profile.workflow_revision
        || binding.machine.as_str() != profile.machine_identity
    {
        return Err(ProfileError::BindingMismatch);
    }
    if binding.run.as_str() != run_ref || binding.authority.as_str() != admitted.identity {
        return Err(ProfileError::BindingMismatch);
    }
    if !matches!(binding.scope, crate::engine::effect::authority::CollaborationScopeBinding::Unscoped)
        || binding.transition.as_str() != START_TRANSITION
    {
        return Err(ProfileError::BindingMismatch);
    }
    let node_id = binding
        .node
        .as_str()
        .strip_prefix(NODE_PREFIX)
        .and_then(canonical_node_number)
        .ok_or(ProfileError::BindingMismatch)?;
    profile
        .grants
        .iter()
        .find(|grant| {
            grant.node_id == node_id
                && grant.operation == binding.operation.as_str()
                && grant.resource == binding.resource.as_str()
                && grant.ability == binding.ability.as_str()
        })
        .ok_or(ProfileError::BindingMismatch)
}

fn is_blake3_reference(value: &str) -> bool {
    value.strip_prefix(BLAKE3_PREFIX).is_some_and(|hex| {
        hex.len() == BLAKE3_HEX_BYTES && hex.bytes().all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    })
}

fn canonical_node_number(value: &str) -> Option<u64> {
    if value.is_empty() || value.len() > MAX_NODE_ID_DIGITS {
        return None;
    }
    if value.len() > 1 && value.starts_with('0') {
        return None;
    }
    if !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    value.parse().ok()
}

#[cfg(test)]
mod tests;

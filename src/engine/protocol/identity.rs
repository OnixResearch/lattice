//! Domain-separated identities for protocol runtime facts.

const BLAKE3_PREFIX: &str = "blake3:";
const BLAKE3_HEX_LENGTH: usize = 64;
const FIELD_SEPARATOR: &[u8] = &[0];

pub const SESSION_PLAN_DOMAIN: &[u8] = b"lattice.role-protocol.session-plan.v1";
pub const SESSION_DOMAIN: &[u8] = b"lattice.role-protocol.session.v1";
pub const ROLE_INSTANCE_DOMAIN: &[u8] = b"lattice.role-protocol.role-instance.v1";
pub const STEP_DOMAIN: &[u8] = b"lattice.role-protocol.step.v1";
pub const ACTION_DOMAIN: &[u8] = b"lattice.role-protocol.action.v1";
pub const MESSAGE_DRAFT_DOMAIN: &[u8] = b"lattice.role-protocol.message-draft.v1";
pub const MESSAGE_DOMAIN: &[u8] = b"lattice.role-protocol.message.v1";
pub const ATTEMPT_DOMAIN: &[u8] = b"lattice.role-protocol.attempt.v1";
pub const OBSERVATION_DOMAIN: &[u8] = b"lattice.role-protocol.observation.v1";
pub const OUTCOME_DOMAIN: &[u8] = b"lattice.role-protocol.outcome.v1";
pub const EVIDENCE_DOMAIN: &[u8] = b"lattice.role-protocol.evidence.v1";
pub const REQUEST_DOMAIN: &[u8] = b"lattice.role-protocol.transport-request.v1";

#[must_use]
pub fn valid_reference(reference: &str) -> bool {
    reference.strip_prefix(BLAKE3_PREFIX).is_some_and(|hex| {
        hex.len() == BLAKE3_HEX_LENGTH && hex.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    })
}

pub fn reference(domain: &[u8], fields: &[&str]) -> Result<super::model::Identity, super::model::Error> {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain);
    for field in fields {
        hasher.update(FIELD_SEPARATOR);
        hasher.update(field.as_bytes());
    }
    super::model::Identity::new(format!("{BLAKE3_PREFIX}{}", hasher.finalize().to_hex()))
}

pub fn serialized_reference<T: serde::Serialize>(
    domain: &[u8],
    value: &T,
) -> Result<super::model::Identity, super::model::Error> {
    let bytes = serde_json::to_vec(value).map_err(|_| super::model::Error::Serialization)?;
    let digest = blake3::hash(&bytes).to_hex().to_string();
    reference(domain, &[digest.as_str()])
}

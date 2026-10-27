use alloc::{string::String, vec::Vec};

use crate::{
    AtifDiagnostic, AtifField, AtifIssueClass, AtifLocation, AtifPolicy, CODEC_IDENTITY,
    HARD_MAX_INPUT_BYTES, SOURCE_COHORT_IDENTITY, Trajectory, ValidatedAtifPolicy,
    ValidatedTrajectory, hash_identity, validate_atif_policy, validate_trajectory,
};

/// Decoded, typed, and validated ATIF input.
#[derive(Clone, Debug, PartialEq)]
pub struct DecodedAtif {
    /// Validated value.
    pub validated: ValidatedTrajectory,
    /// Exact caller-byte identity.
    pub wire_identity: String,
    /// Exact caller byte count.
    pub wire_bytes: u64,
}

/// Encoded bytes and exact byte identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EncodedAtif {
    /// Deterministic compact JSON bytes.
    pub bytes: Vec<u8>,
    /// Exact output-byte identity.
    pub wire_identity: String,
}

/// Decode checked generated policy bytes and revalidate every policy invariant.
///
/// # Errors
/// Rejects malformed, duplicate-keyed, unknown-field, oversized, stale, or unsafe policy.
pub fn decode_atif_policy_json(bytes: &[u8]) -> Result<ValidatedAtifPolicy, AtifDiagnostic> {
    let input_len = u64::try_from(bytes.len()).map_err(|_| codec_bound())?;
    if input_len > HARD_MAX_INPUT_BYTES {
        return Err(codec_bound());
    }
    let policy = serde_json::from_slice::<AtifPolicy>(bytes).map_err(|_| {
        AtifDiagnostic::new(
            AtifIssueClass::Codec,
            AtifLocation::Profile,
            AtifField::Schema,
        )
    })?;
    validate_atif_policy(policy)
}

/// Decode caller-supplied bytes and run complete validation.
///
/// This function does not read paths, resolve URLs, invoke processes, or discover Harbor.
///
/// # Errors
/// Rejects oversized, malformed, duplicate-key, unknown-field, unsupported, or invalid input.
pub fn decode_atif_json(
    bytes: &[u8],
    policy: &ValidatedAtifPolicy,
) -> Result<DecodedAtif, AtifDiagnostic> {
    let input_len = u64::try_from(bytes.len()).map_err(|_| codec_bound())?;
    if input_len > policy.value().bounds.max_input_bytes {
        return Err(codec_bound());
    }
    let document = serde_json::from_slice::<Trajectory>(bytes).map_err(|_| {
        AtifDiagnostic::new(AtifIssueClass::Codec, AtifLocation::Root, AtifField::Schema)
    })?;
    let validated = validate_trajectory(document, policy)?;
    let wire_identity = wire_identity("input", bytes);
    Ok(DecodedAtif {
        validated,
        wire_identity,
        wire_bytes: input_len,
    })
}

/// Encode a validated typed value as deterministic compact JSON.
///
/// # Errors
/// Rejects serialization failure or an output larger than the checked byte bound.
pub fn encode_atif_json(
    validated: &ValidatedTrajectory,
    policy: &ValidatedAtifPolicy,
) -> Result<EncodedAtif, AtifDiagnostic> {
    if validated.policy_identity() != policy.identity() {
        return Err(AtifDiagnostic::new(
            AtifIssueClass::Policy,
            AtifLocation::Profile,
            AtifField::Identity,
        ));
    }
    let bytes = serde_json::to_vec(validated.document()).map_err(|_| {
        AtifDiagnostic::new(AtifIssueClass::Codec, AtifLocation::Root, AtifField::Schema)
    })?;
    let output_len = u64::try_from(bytes.len()).map_err(|_| codec_bound())?;
    if output_len > policy.value().bounds.max_input_bytes {
        return Err(codec_bound());
    }
    let wire_identity = wire_identity("output", &bytes);
    Ok(EncodedAtif {
        bytes,
        wire_identity,
    })
}

fn wire_identity(direction: &str, bytes: &[u8]) -> String {
    hash_identity(
        "animus.harbor-atif-wire.v1",
        &[
            direction.as_bytes(),
            CODEC_IDENTITY.as_bytes(),
            SOURCE_COHORT_IDENTITY.as_bytes(),
            bytes,
        ],
    )
}

const fn codec_bound() -> AtifDiagnostic {
    AtifDiagnostic::new(AtifIssueClass::Bound, AtifLocation::Root, AtifField::Bound)
}

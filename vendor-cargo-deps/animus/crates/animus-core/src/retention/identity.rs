//! Canonical BLAKE3 identity projection.

const DOMAIN: &[u8] = b"animus.tool-result-cache-key.v1";
const TOOL_IDENTITY_TAG: u8 = 1;
const NORMALIZED_ARGUMENTS_TAG: u8 = 2;
const CAPABILITY_CONTEXT_TAG: u8 = 3;

/// Byte length of one BLAKE3 digest.
pub const DIGEST_BYTES: u32 = 32;
const DIGEST_ARRAY_BYTES: usize = blake3::OUT_LEN;

/// Canonical request fields used to identify a retained tool result.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Projection<'a> {
    /// Stable tool identity.
    pub tool_identity: &'a str,
    /// Arguments after adapter-owned canonical normalization.
    pub normalized_arguments: &'a str,
    /// Optional adapter-supplied capability context projection.
    pub capability_context: Option<&'a str>,
}

/// BLAKE3 identity of one canonical request projection.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Digest([u8; DIGEST_ARRAY_BYTES]);

impl Digest {
    /// Returns the fixed-width digest bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; DIGEST_ARRAY_BYTES] {
        &self.0
    }
}

/// Projects one canonical request into a BLAKE3 digest.
///
/// The caller owns argument normalization and capability-context meaning.
/// Payload bytes that do not affect the result must not enter this projection.
///
/// # Errors
///
/// Returns [`super::DecisionError`] for blank identity fields or
/// unrepresentable field lengths.
pub fn project_key(projection: Projection<'_>) -> Result<Digest, super::DecisionError> {
    if projection.tool_identity.trim().is_empty() {
        return Err(super::DecisionError::EmptyToolIdentity);
    }
    if projection
        .capability_context
        .is_some_and(|context| context.trim().is_empty())
    {
        return Err(super::DecisionError::EmptyCapabilityContext);
    }
    debug_assert!(!projection.tool_identity.trim().is_empty());
    debug_assert!(
        projection
            .capability_context
            .is_none_or(|context| !context.trim().is_empty())
    );

    let mut hasher = blake3::Hasher::new();
    hasher.update(DOMAIN);
    update_field(&mut hasher, TOOL_IDENTITY_TAG, projection.tool_identity)?;
    update_field(
        &mut hasher,
        NORMALIZED_ARGUMENTS_TAG,
        projection.normalized_arguments,
    )?;
    update_field(
        &mut hasher,
        CAPABILITY_CONTEXT_TAG,
        projection.capability_context.unwrap_or_default(),
    )?;
    Ok(Digest(*hasher.finalize().as_bytes()))
}

fn update_field(
    hasher: &mut blake3::Hasher,
    tag: u8,
    value: &str,
) -> Result<(), super::DecisionError> {
    let length_bytes =
        u64::try_from(value.len()).map_err(|_| super::DecisionError::ProjectionTooLarge)?;
    hasher.update(&[tag]);
    hasher.update(&length_bytes.to_le_bytes());
    hasher.update(value.as_bytes());
    Ok(())
}

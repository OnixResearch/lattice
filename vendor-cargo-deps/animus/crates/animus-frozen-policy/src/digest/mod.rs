//! Domain-separated BLAKE3 identities and the label rules they rely on.
//!
//! The framing is byte-for-byte that of `animus_eval_core::hash_identity`: the domain and every
//! value are each prefixed with their little-endian `u64` length, and the digest is rendered as
//! `b3:` and 64 lowercase hexadecimal characters. A test pins the equality, so identities from
//! either crate agree without this leaf depending on the evaluation core.

/// Prefix of every identity.
pub const BLAKE3_PREFIX: &str = "b3:";
/// Hexadecimal characters of a BLAKE3 digest.
const DIGEST_HEX_CHARS: usize = 64;
/// Longest safe identity, in bytes.
const MAX_IDENTITY_BYTES: usize = 256;

/// The `b3:` identity of `values` under `domain`.
#[must_use]
pub fn domain_identity(domain: &str, values: &[&str]) -> alloc::string::String {
    let mut hasher = blake3::Hasher::new();
    frame(&mut hasher, domain.as_bytes());
    for value in values {
        frame(&mut hasher, value.as_bytes());
    }
    let identity = alloc::format!("{BLAKE3_PREFIX}{}", hasher.finalize().to_hex());
    debug_assert!(is_b3_identity(&identity));
    debug_assert_eq!(
        identity.len(),
        BLAKE3_PREFIX.len().saturating_add(DIGEST_HEX_CHARS)
    );
    identity
}

/// Whether `value` is `b3:` followed by 64 lowercase hexadecimal characters.
#[must_use]
pub fn is_b3_identity(value: &str) -> bool {
    value.strip_prefix(BLAKE3_PREFIX).is_some_and(|body| {
        body.len() == DIGEST_HEX_CHARS
            && body
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    })
}

/// Whether `value` is a non-empty identity of at most 256 bytes of ASCII letters, digits, `-`,
/// `_`, `.`, and `:`.
#[must_use]
pub fn is_safe_identity(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_IDENTITY_BYTES
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':'))
}

/// Whether every label is a safe identity of at most [`crate::MAX_LABEL_BYTES`] and none
/// repeats.
pub(crate) fn is_unique_label_set<'label>(labels: impl Iterator<Item = &'label str>) -> bool {
    let mut sorted: alloc::vec::Vec<&str> = labels.collect();
    let is_safe = sorted
        .iter()
        .all(|label| label.len() <= crate::MAX_LABEL_BYTES && is_safe_identity(label));
    sorted.sort_unstable();
    let has_repeat = sorted.windows(2).any(|pair| pair.first() == pair.last());
    is_safe && !has_repeat
}

fn frame(hasher: &mut blake3::Hasher, bytes: &[u8]) {
    // A slice length always fits `u64` on every supported target; the frame is 8 bytes on all.
    let length_bytes = bytes.len() as u64;
    hasher.update(&length_bytes.to_le_bytes());
    hasher.update(bytes);
}

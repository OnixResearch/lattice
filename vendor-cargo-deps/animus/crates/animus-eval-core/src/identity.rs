//! Bounded identity validation and domain-separated hashing.

use alloc::{format, string::String};

/// Maximum bytes in one safe public identity.
pub const MAX_IDENTITY_BYTES: usize = 256;
/// Hexadecimal characters in one BLAKE3 digest.
pub const BLAKE3_HEX_CHARS: usize = 64;
/// Prefix used by canonical BLAKE3 identities.
pub const BLAKE3_PREFIX: &str = "b3:";

/// Bounded evaluation diagnostic class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvaluationDiagnosticClass {
    /// Input identity is malformed or stale.
    Identity,
    /// A positive bound is missing or exceeded.
    Bound,
    /// Policy denies the requested operation.
    Denied,
    /// Supplied evidence does not match the plan.
    Mismatch,
    /// Metadata evidence contains private content or an overclaim.
    Evidence,
}

/// Bounded public diagnostic without raw source content.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvaluationDiagnostic {
    /// Stable diagnostic class.
    pub class: EvaluationDiagnosticClass,
    /// Bounded safe label.
    pub label: String,
}

impl EvaluationDiagnostic {
    /// Construct a bounded diagnostic label.
    ///
    /// # Errors
    /// Returns [`IdentityError::Unsafe`] for empty, oversized, or unsafe labels.
    pub fn new(class: EvaluationDiagnosticClass, label: &str) -> Result<Self, IdentityError> {
        validate_identity(label)?;
        Ok(Self {
            class,
            label: label.into(),
        })
    }
}

/// Identity validation failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IdentityError {
    /// The value is empty, too long, or contains unsafe characters.
    Unsafe,
    /// The value is not a canonical prefixed BLAKE3 identity.
    MalformedDigest,
}

/// Returns whether a value is a bounded metadata identity.
#[must_use]
pub fn is_safe_identity(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_IDENTITY_BYTES
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':'))
}

/// Validate a bounded metadata identity.
///
/// # Errors
/// Returns [`IdentityError::Unsafe`] for empty, oversized, or unsafe values.
pub fn validate_identity(value: &str) -> Result<(), IdentityError> {
    if !is_safe_identity(value) {
        return Err(IdentityError::Unsafe);
    }
    Ok(())
}

/// Validate a canonical `b3:` identity.
///
/// # Errors
/// Returns an identity error when the prefix, length, or lowercase hexadecimal
/// body is malformed.
pub fn validate_blake3_identity(value: &str) -> Result<(), IdentityError> {
    let Some(body) = value.strip_prefix(BLAKE3_PREFIX) else {
        return Err(IdentityError::MalformedDigest);
    };
    if body.len() != BLAKE3_HEX_CHARS
        || !body
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(IdentityError::MalformedDigest);
    }
    Ok(())
}

/// Build a domain-separated canonical BLAKE3 identity.
#[must_use]
pub fn hash_identity(domain: &str, values: &[&str]) -> String {
    let mut hasher = blake3::Hasher::new();
    frame(&mut hasher, domain.as_bytes());
    for value in values {
        frame(&mut hasher, value.as_bytes());
    }
    format!("{BLAKE3_PREFIX}{}", hasher.finalize().to_hex())
}

fn frame(hasher: &mut blake3::Hasher, bytes: &[u8]) {
    let length = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
    hasher.update(&length.to_le_bytes());
    hasher.update(bytes);
}

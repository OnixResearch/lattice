//! Canonical text validation helpers.

/// Validates model text and rejects leak markers.
pub(crate) fn safe_text(
    field: crate::validation::FieldName,
    raw: &str,
) -> Result<alloc::string::String, super::CanonicalModelContractError> {
    let normalized = crate::validation::normalized_adapter_field(field, raw)?;
    if contains_leak_marker(normalized.as_str(), super::LEAK_MARKER_RAW_PROVIDER_PAYLOAD)
        || contains_leak_marker(
            normalized.as_str(),
            super::LEAK_MARKER_PROVIDER_PAYLOAD_PREFIX,
        )
    {
        return Err(super::CanonicalModelContractError::UnsafeText {
            field: field.as_str(),
            reason: super::CanonicalUnsafeTextReason::RawProviderPayload,
        });
    }
    if contains_leak_marker(normalized.as_str(), super::LEAK_MARKER_CREDENTIAL_PREFIX)
        || contains_leak_marker(normalized.as_str(), super::LEAK_MARKER_SECRET_PREFIX)
        || contains_leak_marker(normalized.as_str(), super::LEAK_MARKER_AUTHORIZATION_PREFIX)
        || contains_leak_marker(normalized.as_str(), super::LEAK_MARKER_BEARER_PREFIX)
    {
        return Err(super::CanonicalModelContractError::UnsafeText {
            field: field.as_str(),
            reason: super::CanonicalUnsafeTextReason::Credential,
        });
    }
    if contains_leak_marker(normalized.as_str(), super::LEAK_MARKER_HIDDEN_REASONING) {
        return Err(super::CanonicalModelContractError::UnsafeText {
            field: field.as_str(),
            reason: super::CanonicalUnsafeTextReason::HiddenReasoning,
        });
    }
    Ok(normalized)
}

pub(crate) fn contains_leak_marker(value: &str, marker: &str) -> bool {
    assert!(!marker.is_empty(), "leak marker must not be empty");
    value
        .as_bytes()
        .windows(marker.len())
        .any(|window| ascii_case_eq(window, marker.as_bytes()))
}

pub(crate) fn ascii_case_eq(left: &[u8], right: &[u8]) -> bool {
    assert!(left.len() == right.len(), "slices must be the same length");
    left.iter()
        .zip(right.iter())
        .all(|(left_byte, right_byte)| left_byte.eq_ignore_ascii_case(right_byte))
}

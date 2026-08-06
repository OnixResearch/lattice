const CREDENTIAL_MARKERS: &[&str] = &["api_key", "token=", "credential", "secret=", "sk-"];
const ABSOLUTE_PATH_MARKERS: &[&str] = &["/home/", "/users/"];

#[derive(Clone, Copy)]
pub(super) struct ByteLimitInput {
    pub(super) byte_count: u64,
    pub(super) max_bytes: u64,
}

#[derive(Clone, Copy)]
struct PathProbe<'a> {
    value: &'a str,
    lower: &'a str,
}

pub(super) fn normalized_safe_text(
    field: crate::validation::FieldName,
    raw: &str,
) -> Result<alloc::string::String, super::Error> {
    let normalized = crate::validation::normalized_adapter_field(field, raw)?;
    if let Some(reason) = unsafe_content_reason(normalized.as_str()) {
        return Err(super::Error::UnsafeDescriptor {
            field: field.as_str(),
            reason,
        });
    }
    Ok(normalized)
}

pub(super) fn validate_hints(raw_hints: &[&str]) -> Result<alloc::vec::Vec<alloc::string::String>, super::Error> {
    assert!(raw_hints.len() <= crate::validation::MAX_ADAPTER_FIELD_CHARS);
    let mut hints = alloc::vec::Vec::with_capacity(raw_hints.len());
    for hint in raw_hints {
        hints.push(normalized_safe_text(super::SOURCE_HINT_FIELD, hint)?);
    }
    Ok(hints)
}

pub(super) fn validate_receipt_fragments(
    raw_fragments: &[&str],
) -> Result<alloc::vec::Vec<alloc::string::String>, super::Error> {
    assert!(raw_fragments.len() <= crate::validation::MAX_ADAPTER_FIELD_CHARS);
    let mut fragments = alloc::vec::Vec::with_capacity(raw_fragments.len());
    for fragment in raw_fragments {
        fragments.push(normalized_safe_text(super::METADATA_FRAGMENT_FIELD, fragment)?);
    }
    Ok(fragments)
}

pub(super) const fn validate_byte_count(input: ByteLimitInput) -> Result<(), super::Error> {
    if input.byte_count == 0 {
        return Err(super::Error::MissingByteCount);
    }
    if input.byte_count > input.max_bytes {
        return Err(super::Error::ImageTooLarge {
            max_bytes: input.max_bytes,
            actual_bytes: input.byte_count,
        });
    }
    Ok(())
}

pub(super) const fn validate_dimensions(width_px: Option<u32>, height_px: Option<u32>) -> Result<(), super::Error> {
    if invalid_dimension(width_px) {
        return Err(super::Error::InvalidDimensions { width_px, height_px });
    }
    if invalid_dimension(height_px) {
        return Err(super::Error::InvalidDimensions { width_px, height_px });
    }
    Ok(())
}

pub(super) fn validate_content_hash(raw: &str) -> Result<alloc::string::String, super::Error> {
    let normalized = normalized_safe_text(super::CONTENT_HASH_FIELD, raw)?;
    let has_expected_hash_chars = normalized.len() == super::BLAKE3_HEX_CHARS;
    let has_lower_hex_chars = normalized.bytes().all(is_ascii_lower_hex);
    if !has_expected_hash_chars {
        return Err(super::Error::InvalidContentHash {
            field: super::CONTENT_HASH_FIELD.as_str(),
        });
    }
    if !has_lower_hex_chars {
        return Err(super::Error::InvalidContentHash {
            field: super::CONTENT_HASH_FIELD.as_str(),
        });
    }
    Ok(normalized)
}

pub(super) fn usize_to_u64(value: usize) -> u64 {
    value as u64
}

fn unsafe_content_reason(value: &str) -> Option<super::UnsafeContentReason> {
    assert!(!value.is_empty());
    assert!(value.chars().count() <= crate::validation::MAX_ADAPTER_FIELD_CHARS);
    let lower = value.to_ascii_lowercase();
    if is_data_url_like(lower.as_str()) {
        return Some(super::UnsafeContentReason::DataUrl);
    }
    if lower.contains("base64") {
        return Some(super::UnsafeContentReason::Base64Payload);
    }
    if is_raw_bytes_like(lower.as_str()) {
        return Some(super::UnsafeContentReason::RawBytes);
    }
    if is_absolute_or_private_path(PathProbe {
        value,
        lower: lower.as_str(),
    }) {
        return Some(super::UnsafeContentReason::AbsolutePath);
    }
    if contains_any(lower.as_str(), CREDENTIAL_MARKERS) {
        return Some(super::UnsafeContentReason::Credential);
    }
    if is_provider_payload_like(lower.as_str()) {
        return Some(super::UnsafeContentReason::ProviderPayload);
    }
    if lower.contains("unbounded") {
        return Some(super::UnsafeContentReason::UnboundedPayload);
    }
    None
}

const fn invalid_dimension(value: Option<u32>) -> bool {
    match value {
        Some(raw) => raw < super::MIN_DIMENSION_PIXELS || raw > super::MAX_DIMENSION_PIXELS,
        None => false,
    }
}

const fn is_ascii_lower_hex(byte: u8) -> bool {
    matches!(byte, b'0'..=b'9' | b'a'..=b'f')
}

fn is_data_url_like(lower: &str) -> bool {
    lower.starts_with("data:") || lower.contains("data:image")
}

fn is_raw_bytes_like(lower: &str) -> bool {
    lower.contains("raw_bytes") || lower.contains("bytes=[")
}

fn is_provider_payload_like(lower: &str) -> bool {
    lower.contains("provider_payload") || lower.contains("request_json")
}

fn is_absolute_or_private_path(probe: PathProbe<'_>) -> bool {
    if probe.value.starts_with('/') {
        return true;
    }
    if probe.value.starts_with("~/") {
        return true;
    }
    if probe.value.contains(":\\") {
        return true;
    }
    contains_any(probe.lower, ABSOLUTE_PATH_MARKERS)
}

fn contains_any(value: &str, markers: &[&str]) -> bool {
    assert!(!markers.is_empty());
    for marker in markers {
        if value.contains(marker) {
            return true;
        }
    }
    false
}

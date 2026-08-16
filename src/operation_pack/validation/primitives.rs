pub fn validate_identity(value: &str) -> Result<(), super::super::Error> {
    let expected_length_bytes = crate::operation::model::BLAKE3_PREFIX
        .len()
        .checked_add(crate::operation::model::BLAKE3_HEX_LENGTH)
        .ok_or(super::super::Error::Invalid("BLAKE3 identity"))?;
    if value.len() != expected_length_bytes {
        return Err(super::super::Error::Invalid("BLAKE3 identity"));
    }
    let digest = value
        .strip_prefix(crate::operation::model::BLAKE3_PREFIX)
        .ok_or(super::super::Error::Invalid("BLAKE3 identity"))?;
    let has_only_hex = digest.as_bytes().iter().all(u8::is_ascii_hexdigit);
    let has_uppercase = digest.as_bytes().iter().any(u8::is_ascii_uppercase);
    if !has_only_hex || has_uppercase {
        return Err(super::super::Error::Invalid("BLAKE3 identity"));
    }
    Ok(())
}

pub fn validate_safe_relative_path(value: &str) -> Result<(), super::super::Error> {
    if value.is_empty() || value.len() > super::super::model::MAXIMUM_PATH_BYTES || !value.is_ascii() {
        return Err(super::super::Error::Invalid("content path"));
    }
    let path = std::path::Path::new(value);
    if path.is_absolute() || path.components().any(|component| !matches!(component, std::path::Component::Normal(_))) {
        return Err(super::super::Error::Invalid("content path"));
    }
    Ok(())
}

pub(super) fn validate_id(value: &str) -> Result<(), super::super::Error> {
    let has_valid_length = !value.is_empty() && value.len() <= crate::operation::model::MAXIMUM_ID_BYTES;
    let has_valid_bytes = value
        .bytes()
        .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'_'));
    if !has_valid_length || !has_valid_bytes {
        return Err(super::super::Error::Invalid("identifier"));
    }
    Ok(())
}

pub(super) fn validate_scoped_id(value: &str) -> Result<(), super::super::Error> {
    if value.split('.').count() < 2 {
        return Err(super::super::Error::Invalid("scoped identifier"));
    }
    for part in value.split('.') {
        validate_id(part)?;
    }
    Ok(())
}

pub(super) fn validate_safe_text(value: &str) -> Result<(), super::super::Error> {
    if value.is_empty()
        || value.len() > super::super::model::MAXIMUM_TEXT_BYTES
        || !value.is_ascii()
        || value.bytes().any(|byte| byte.is_ascii_control())
    {
        return Err(super::super::Error::Invalid("safe text"));
    }
    Ok(())
}

pub(super) fn validate_platform(value: &str) -> Result<(), super::super::Error> {
    let Some((architecture, system)) = value.split_once('-') else {
        return Err(super::super::Error::Invalid("platform"));
    };
    validate_id(architecture)?;
    validate_id(system)
}

pub(super) fn content_map(
    manifest: &super::super::model::Manifest,
) -> std::collections::BTreeMap<&str, &super::super::model::ContentItem> {
    manifest.contents.iter().map(|content| (content.id.as_str(), content)).collect()
}

pub(super) fn sort_unique<T: Ord>(values: &mut [T]) -> Result<(), super::super::Error> {
    values.sort();
    if values.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(super::super::Error::Invalid("duplicate normalized value"));
    }
    Ok(())
}

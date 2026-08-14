pub const BLAKE3_PREFIX: &str = "b3:";
pub const BLAKE3_HEX_BYTES: usize = 64;
const FRAME_SEPARATOR: &[u8] = b"\0";

pub fn validate(value: &str) -> Result<(), super::Error> {
    let Some(hex) = value.strip_prefix(BLAKE3_PREFIX) else {
        return Err(super::Error::InvalidIdentity);
    };
    let has_exact_length = hex.len() == BLAKE3_HEX_BYTES;
    let has_lower_hex = hex.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
    if !has_exact_length || !has_lower_hex {
        return Err(super::Error::InvalidIdentity);
    }
    Ok(())
}

pub fn hash_bytes(domain: &str, bytes: &[u8]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain.as_bytes());
    hasher.update(FRAME_SEPARATOR);
    hasher.update(bytes);
    format!("{BLAKE3_PREFIX}{}", hasher.finalize().to_hex())
}

pub fn hash_value<T: serde::Serialize + ?Sized>(domain: &str, value: &T) -> Result<String, super::Error> {
    let bytes = serde_json::to_vec(value).map_err(|_| super::Error::Encoding)?;
    Ok(hash_bytes(domain, &bytes))
}

pub fn valid_public_label(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= super::model::MAX_ID_BYTES
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'_' | b'.' | b':'))
}

pub fn valid_safe_path(value: &str) -> bool {
    let has_invalid_prefix = value.is_empty() || value.starts_with('/');
    let has_invalid_length = value.len() > super::model::MAX_PATH_BYTES;
    let has_invalid_separator = value.contains('\\') || value.contains("//");
    if has_invalid_prefix || has_invalid_length || has_invalid_separator {
        return false;
    }
    value.split('/').all(|component| {
        !component.is_empty() && component != "." && component != ".." && valid_path_component(component)
    })
}

fn valid_path_component(value: &str) -> bool {
    value.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

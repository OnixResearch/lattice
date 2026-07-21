const WORKFLOW_HASH_SEPARATOR: &str = "|";
const BLAKE3_HEX_CHARS: usize = 64;
const ASCII_HEX_RADIX: u32 = 16;

#[derive(Clone, Copy)]
pub(super) struct HashInput<'a> {
    pub(super) field: &'static str,
    pub(super) value: &'a str,
}

pub(super) fn validate_blake3_hex(input: HashInput<'_>) -> Result<(), super::RecipeWorkflowError> {
    let hash_len = input.value.chars().count();
    if hash_len != BLAKE3_HEX_CHARS {
        return Err(super::RecipeWorkflowError::InvalidHash { field: input.field });
    }
    if input
        .value
        .chars()
        .any(|character| character.to_digit(ASCII_HEX_RADIX).is_none() || character.is_uppercase())
    {
        return Err(super::RecipeWorkflowError::InvalidHash { field: input.field });
    }
    Ok(())
}

pub(super) fn hash_parts(parts: &[&str]) -> alloc::string::String {
    assert!(!parts.is_empty(), "hash input must not be empty");
    let mut text = alloc::string::String::new();
    for (index, part) in parts.iter().enumerate() {
        if index > 0 {
            text.push_str(WORKFLOW_HASH_SEPARATOR);
        }
        text.push_str(part);
    }
    alloc::format!("{}", blake3::hash(text.as_bytes()).to_hex())
}

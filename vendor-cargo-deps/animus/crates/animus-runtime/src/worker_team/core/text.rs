#[derive(Clone, Copy)]
pub(crate) struct HashInput<'a> {
    pub(crate) field: &'static str,
    pub(crate) value: &'a str,
}

#[derive(Clone, Copy)]
struct LimitInput<'a> {
    field: &'static str,
    value: &'a str,
    limit: usize,
}

pub(crate) fn validate_label(
    field: &'static str,
    value: impl AsRef<str>,
) -> Result<String, super::super::DelegationFailure> {
    validate_with_limit(LimitInput {
        field,
        value: value.as_ref(),
        limit: super::super::MAX_LABEL_CHARS,
    })
}

pub(crate) fn validate_body(
    field: &'static str,
    value: impl AsRef<str>,
) -> Result<String, super::super::DelegationFailure> {
    validate_with_limit(LimitInput {
        field,
        value: value.as_ref(),
        limit: super::super::MAX_TEXT_CHARS,
    })
}

fn validate_with_limit(input: LimitInput<'_>) -> Result<String, super::super::DelegationFailure> {
    debug_assert!(!input.field.is_empty());
    debug_assert!(input.limit > 0);
    let trimmed = input.value.trim();
    if trimmed.is_empty() {
        return Err(super::super::DelegationFailure::new(
            super::super::FailureKind::EmptyField,
            input.field,
            super::super::EMPTY_FIELD_REASON,
        ));
    }
    if trimmed.chars().count() > input.limit {
        return Err(super::super::DelegationFailure::new(
            super::super::FailureKind::FieldTooLong,
            input.field,
            super::super::TOO_LONG_REASON,
        ));
    }
    if contains_unsafe_marker(trimmed) {
        return Err(super::super::DelegationFailure::new(
            super::super::FailureKind::UnsafeText,
            input.field,
            super::super::UNSAFE_TEXT_REASON,
        ));
    }
    Ok(String::from(trimmed))
}

pub(crate) fn validate_hash(input: HashInput<'_>) -> Result<(), super::super::DelegationFailure> {
    debug_assert!(!input.field.is_empty());
    let hash = input.value.trim();
    if hash.len() != super::super::BLAKE3_HEX_CHARS || !hash.chars().all(|character| character.is_ascii_hexdigit()) {
        return Err(super::super::DelegationFailure::new(
            super::super::FailureKind::UnsafeText,
            input.field,
            "hash must be BLAKE3 hex",
        ));
    }
    Ok(())
}

fn contains_unsafe_marker(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    super::super::UNSAFE_TEXT_MARKERS.iter().any(|marker| lower.contains(&marker.to_ascii_lowercase()))
}

pub(crate) fn validate_non_empty_slice<T>(
    field: &'static str,
    values: &[T],
) -> Result<(), super::super::DelegationFailure> {
    if values.is_empty() {
        return Err(super::super::DelegationFailure::new(
            super::super::FailureKind::EmptyField,
            field,
            super::super::EMPTY_FIELD_REASON,
        ));
    }
    Ok(())
}

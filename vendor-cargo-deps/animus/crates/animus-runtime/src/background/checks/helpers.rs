#[derive(Clone, Copy)]
pub(crate) struct LabelInput<'a> {
    pub(crate) field: &'static str,
    pub(crate) value: &'a str,
    pub(crate) profile_name: &'a str,
}

#[derive(Clone, Copy)]
pub(crate) struct CommandShapeErrorInput<'a> {
    pub(crate) profile_name: &'a str,
    pub(crate) reason: &'static str,
}

#[derive(Clone, Copy)]
pub(crate) struct EnvErrorInput<'a> {
    pub(crate) profile_name: &'a str,
    pub(crate) key: &'a str,
    pub(crate) class: &'static str,
}

#[derive(Clone, Copy)]
pub(crate) struct PathErrorInput<'a> {
    pub(crate) field: &'static str,
    pub(crate) profile_name: &'a str,
}

pub(crate) fn validate_safe_label(input: LabelInput<'_>) -> Result<(), super::super::ProcessJobProfileError> {
    if input.value.trim().is_empty() {
        return Err(super::super::ProcessJobProfileError::new(
            super::super::ProcessJobProfileErrorKind::EmptyField,
            input.profile_name,
            input.field,
            "field must not be empty",
        ));
    }
    if !is_safe_label(input.value) {
        return Err(super::super::ProcessJobProfileError::new(
            super::super::ProcessJobProfileErrorKind::UnsafeIdentity,
            input.profile_name,
            input.field,
            "label must contain only ASCII alphanumeric, dash, underscore, or dot",
        ));
    }
    Ok(())
}

pub(crate) fn validate_identity_field(
    input: super::IdentityTextInput<'_>,
) -> Result<(), super::super::ProcessJobProfileError> {
    if input.value.trim().is_empty() {
        return Err(identity_error(input, "must not be empty"));
    }
    if let Some(reason) = identity_text_issue(input.value) {
        return Err(identity_error(input, reason));
    }
    Ok(())
}

pub(crate) fn validate_hash_field(input: super::HashInput<'_>) -> Result<(), super::super::ProcessJobProfileError> {
    let is_hash = input.value.len() == super::super::HASH_HEX_LENGTH
        && input.value.chars().all(|character| character.is_ascii_hexdigit());
    if is_hash {
        return Ok(());
    }
    Err(super::super::ProcessJobProfileError::new(
        super::super::ProcessJobProfileErrorKind::UnsafeIdentity,
        input.profile_name,
        input.field,
        "hash must be a BLAKE3 hex digest",
    ))
}

pub(crate) fn command_shape_error(input: CommandShapeErrorInput<'_>) -> super::super::ProcessJobProfileError {
    super::super::ProcessJobProfileError::new(
        super::super::ProcessJobProfileErrorKind::InvalidCommandShape,
        input.profile_name,
        super::super::COMMAND_FIELD,
        input.reason,
    )
}

pub(crate) fn env_error(input: EnvErrorInput<'_>) -> super::super::ProcessJobProfileError {
    super::super::ProcessJobProfileError::new(
        super::super::ProcessJobProfileErrorKind::DisallowedEnvironmentKey,
        input.profile_name,
        super::super::ENV_FIELD,
        format!("{} environment key `{}`", input.class, input.key),
    )
}

pub(crate) fn path_error(input: PathErrorInput<'_>) -> super::super::ProcessJobProfileError {
    let kind = if input.field == super::super::CWD_FIELD {
        super::super::ProcessJobProfileErrorKind::DisallowedCwd
    } else {
        super::super::ProcessJobProfileErrorKind::DisallowedWritablePath
    };
    super::super::ProcessJobProfileError::new(
        kind,
        input.profile_name,
        input.field,
        "path is outside allowed prefixes or contains parent traversal",
    )
}

pub(crate) fn is_allowed_env_key(key: &str, prefixes: &[String]) -> bool {
    key_is_upper_snake(key) && !is_secret_like_key(key) && prefixes.iter().any(|prefix| key.starts_with(prefix))
}

pub(crate) fn path_within_any_prefix(path: &str, prefixes: &[String]) -> bool {
    prefixes.iter().any(|prefix| {
        let trimmed = prefix.trim_end_matches(super::super::PATH_SEPARATOR);
        path == trimmed
            || path.strip_prefix(trimmed).is_some_and(|suffix| suffix.starts_with(super::super::PATH_SEPARATOR))
    })
}

pub(crate) fn metadata_key_is_safe(value: &str) -> bool {
    value.starts_with(super::super::METADATA_IDENTITY_PREFIX) && is_safe_label(value)
}

pub(crate) fn identity_error(
    input: super::IdentityTextInput<'_>,
    reason: &str,
) -> super::super::ProcessJobProfileError {
    super::super::ProcessJobProfileError::new(
        super::super::ProcessJobProfileErrorKind::UnsafeIdentity,
        input.profile_name,
        input.field,
        reason,
    )
}

fn key_is_upper_snake(key: &str) -> bool {
    !key.is_empty()
        && key
            .chars()
            .all(|character| character.is_ascii_uppercase() || character.is_ascii_digit() || character == '_')
}

fn is_secret_like_key(key: &str) -> bool {
    let upper = key.to_ascii_uppercase();
    super::super::SECRET_KEY_MARKERS.iter().any(|marker| upper.contains(marker))
}

fn is_safe_label(value: &str) -> bool {
    value
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.'))
}

fn identity_text_issue(value: &str) -> Option<&'static str> {
    if value.contains(super::super::PATH_SEPARATOR) || value.contains(super::super::WINDOWS_PATH_SEPARATOR) {
        return Some("contains unsafe path content");
    }
    if value.contains(super::super::REMOTE_SCHEME_MARKER) {
        return Some("contains remote locator content");
    }
    let lower = value.to_ascii_lowercase();
    super::super::IDENTITY_LEAK_MARKERS.iter().copied().find(|marker| lower.contains(marker))
}

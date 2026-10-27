const WINDOWS_PATH_SEPARATOR: char = '\\';
const PROFILE_NAME_SEPARATOR: char = '-';

#[derive(Clone, Copy)]
pub(crate) struct FieldName(&'static str);

impl FieldName {
    pub(crate) const SCRIPT_HASH: Self = Self("script_hash");

    const fn as_str(self) -> &'static str {
        self.0
    }
}

pub(crate) fn validate_profile_name(name: &str) -> Result<String, super::Error> {
    if !is_safe_profile_name(name) {
        return Err(super::Error::new(super::DenyReason::BadProfileName, "name"));
    }

    Ok(String::from(name))
}

pub(crate) fn validate_script_label(label: &str) -> Result<String, super::Error> {
    if !is_safe_script_label(label) {
        return Err(super::Error::new(
            super::DenyReason::PathEscape,
            "script_label",
        ));
    }

    Ok(String::from(label))
}

pub(crate) fn validate_hash(hash: &str, field: FieldName) -> Result<String, super::Error> {
    if blake3::Hash::from_hex(hash).is_err() {
        return Err(super::Error::new(
            super::DenyReason::HashMismatch,
            field.as_str(),
        ));
    }

    Ok(String::from(hash))
}

pub(crate) fn validate_turn_budget(max_turns: u32) -> Result<u32, super::Error> {
    if max_turns < super::config::MIN_RUNTIME_TURNS {
        return Err(super::Error::new(
            super::DenyReason::InvalidBudget,
            "max_turns",
        ));
    }

    Ok(max_turns)
}

pub(crate) fn parse_allowed_actions(
    actions: &[&str],
) -> Result<std::collections::BTreeSet<super::HostAction>, super::Error> {
    let mut parsed = std::collections::BTreeSet::new();
    for action in actions {
        parsed.insert(super::HostAction::parse(action)?);
    }
    if parsed.is_empty() {
        return Err(super::Error::new(
            super::DenyReason::HostActionDenied,
            "allowed_host_actions",
        ));
    }

    Ok(parsed)
}

pub(crate) fn is_safe_profile_name(name: &str) -> bool {
    let trimmed = name.trim();
    if trimmed.is_empty() || trimmed != name {
        return false;
    }
    let Some(first) = name.chars().next() else {
        return false;
    };
    first.is_ascii_lowercase() && has_profile_name_characters(name)
}

pub(crate) fn is_safe_script_label(label: &str) -> bool {
    let trimmed = label.trim();
    if trimmed.is_empty() || trimmed != label || label.contains(WINDOWS_PATH_SEPARATOR) {
        return false;
    }

    std::path::Path::new(label)
        .components()
        .all(|component| matches!(component, std::path::Component::Normal(_)))
}

fn has_profile_name_characters(name: &str) -> bool {
    name.chars().all(|character| {
        character.is_ascii_lowercase()
            || character.is_ascii_digit()
            || character == PROFILE_NAME_SEPARATOR
    })
}

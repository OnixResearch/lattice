pub(super) fn validate_environment_allowlist(
    entries: &[&str],
) -> Result<super::Collection<super::Text>, super::PluginDescriptorError> {
    assert!(
        entries.len() <= crate::MAX_ADAPTER_FIELD_CHARS,
        "environment allowlist count must fit the adapter field bound"
    );
    let mut names = super::Collection::with_capacity(entries.len());
    for entry in entries {
        let name = super::required_text_value(super::ENVIRONMENT_ALLOWLIST_FIELD, entry)?;
        if name.contains(super::ENVIRONMENT_VALUE_SEPARATOR) {
            return Err(super::PluginDescriptorError::EnvironmentValueLeak { entry: name });
        }
        names.push(name);
    }
    Ok(names)
}

pub(super) fn validate_writable_roots(
    entries: &[&str],
) -> Result<super::Collection<super::Text>, super::PluginDescriptorError> {
    assert!(
        entries.len() <= crate::MAX_ADAPTER_FIELD_CHARS,
        "writable root count must fit the adapter field bound"
    );
    let mut roots = super::Collection::with_capacity(entries.len());
    for entry in entries {
        let root = super::required_text_value(super::WRITABLE_ROOT_FIELD, entry)?;
        if is_absolute(&root) {
            return Err(super::PluginDescriptorError::AbsoluteWritableRoot { value: root });
        }
        if escapes_workspace(&root) {
            return Err(super::PluginDescriptorError::PathEscape { value: root });
        }
        roots.push(root);
    }
    Ok(roots)
}

fn is_absolute(path: &str) -> bool {
    path.starts_with(super::UNIX_PATH_SEPARATOR)
        || path.starts_with(super::WINDOWS_PATH_SEPARATOR)
        || has_windows_drive(path)
}

fn has_windows_drive(path: &str) -> bool {
    let bytes = path.as_bytes();
    bytes.len() > super::WINDOWS_DRIVE_SEPARATOR_INDEX
        && bytes[super::WINDOWS_DRIVE_SEPARATOR_INDEX] == super::WINDOWS_DRIVE_SEPARATOR
        && bytes.first().is_some_and(u8::is_ascii_alphabetic)
}

fn escapes_workspace(path: &str) -> bool {
    path.starts_with(super::HOME_PATH_PREFIX)
        || path.split(is_separator).any(|segment| segment == super::PATH_ESCAPE_SEGMENT)
}

fn is_separator(character: char) -> bool {
    character == super::UNIX_PATH_SEPARATOR || character == super::WINDOWS_PATH_SEPARATOR
}

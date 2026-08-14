pub fn safe_relative_path(path: &str) -> bool {
    let path = std::path::Path::new(path);
    !path.as_os_str().is_empty()
        && !path.is_absolute()
        && path.components().all(|component| matches!(component, std::path::Component::Normal(_)))
}

fn valid_identifier(value: &str) -> bool {
    if value.is_empty() || value.len() > super::model::MAX_IDENTIFIER_BYTES {
        return false;
    }
    let has_valid_characters = value
        .bytes()
        .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'.'));
    let has_lowercase_prefix = value.bytes().next().is_some_and(|byte| byte.is_ascii_lowercase());
    has_valid_characters && has_lowercase_prefix
}

fn safe_bundle_label(value: &str) -> String {
    if valid_identifier(value) {
        value.to_string()
    } else {
        "invalid-bundle".to_string()
    }
}

pub(crate) struct DiagnosticMessage(&'static str);

impl From<&'static str> for DiagnosticMessage {
    fn from(value: &'static str) -> Self {
        Self(value)
    }
}

pub(crate) fn diagnostic(
    bundle: &str,
    code: super::model::DiagnosticCode,
    message: impl Into<DiagnosticMessage>,
) -> super::model::Diagnostic {
    super::model::Diagnostic {
        bundle: safe_bundle_label(bundle),
        code,
        message: message.into().0,
    }
}

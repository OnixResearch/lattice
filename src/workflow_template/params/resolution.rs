//! Declaration shape checks, value checks, and resolution. Pure.

const MAX_ENUM_TOKEN_BYTES: usize = 64;

/// Check the declared parameters' names, types, bounds, defaults, and binding targets.
///
/// This check needs no workflow; [`super::validate_bindings`] checks targets against one.
pub fn validate_declarations(declared: &[super::ParamDeclaration]) -> Result<(), super::ParamError> {
    if declared.len() > super::MAX_PARAMS {
        return Err(super::ParamError::new(super::ParamErrorCode::InvalidDeclaration, None));
    }
    let mut names = std::collections::BTreeSet::new();
    let mut targets = std::collections::BTreeSet::new();
    for declaration in declared {
        let invalid = || super::ParamError::new(super::ParamErrorCode::InvalidDeclaration, Some(&declaration.name));
        if !valid_name(&declaration.name) || !names.insert(declaration.name.as_str()) {
            return Err(invalid());
        }
        if !valid_type(&declaration.value_type) {
            return Err(invalid());
        }
        let has_valid_default = match (&declaration.default, declaration.required) {
            (None, true) => true,
            (Some(default), false) => check_bounds(&declaration.value_type, default).is_ok(),
            (None, false) | (Some(_), true) => false,
        };
        if !has_valid_default {
            return Err(invalid());
        }
        if declaration.bindings.is_empty() || declaration.bindings.len() > super::MAX_PARAM_BINDINGS {
            return Err(invalid());
        }
        for target in &declaration.bindings {
            let has_valid_label = !target.node.is_empty() && target.node.len() <= super::MAX_TARGET_LABEL_BYTES;
            if !has_valid_label
                || !valid_env_key(&target.shell_env)
                || !targets.insert((target.node.as_str(), target.shell_env.as_str()))
            {
                return Err(invalid());
            }
        }
    }
    debug_assert!(names.len() == declared.len());
    Ok(())
}

// r[impl lattice.workflow_templates.params]
/// Resolve supplied values against declarations.
///
/// Unknown names, wrong types, out-of-bounds values, and missing required values are rejected.
/// An omitted optional parameter takes its declared default.
pub fn resolve(
    declared: &[super::ParamDeclaration],
    supplied: &std::collections::BTreeMap<String, super::ParamValue>,
) -> Result<super::Resolved, super::ParamError> {
    validate_declarations(declared)?;
    if supplied.len() > super::MAX_PARAMS {
        return Err(super::ParamError::new(super::ParamErrorCode::TooManyValues, None));
    }
    if supplied.keys().any(|name| !declared.iter().any(|declaration| &declaration.name == name)) {
        return Err(super::ParamError::new(super::ParamErrorCode::UnknownParam, None));
    }
    let mut values = Vec::with_capacity(declared.len());
    for declaration in declared {
        let value = match (supplied.get(&declaration.name), &declaration.default) {
            (Some(value), _) | (None, Some(value)) => value,
            (None, None) => {
                return Err(super::ParamError::new(super::ParamErrorCode::MissingRequired, Some(&declaration.name)));
            }
        };
        check_bounds(&declaration.value_type, value)
            .map_err(|code| super::ParamError::new(code, Some(&declaration.name)))?;
        values.push(super::ResolvedParam {
            name: declaration.name.clone(),
            value: value.clone(),
        });
    }
    debug_assert_eq!(values.len(), declared.len());
    Ok(super::Resolved::new(values))
}

/// The exact environment text of a checked value.
pub fn render(value: &super::ParamValue) -> String {
    match value {
        super::ParamValue::Bool(value) => value.to_string(),
        super::ParamValue::Integer(value) => value.to_string(),
        super::ParamValue::Text(value) => value.clone(),
    }
}

fn check_bounds(value_type: &super::ParamType, value: &super::ParamValue) -> Result<(), super::ParamErrorCode> {
    let is_in_bounds = match (value_type, value) {
        (super::ParamType::Bool {}, super::ParamValue::Bool(_)) => true,
        (super::ParamType::Integer { min, max }, super::ParamValue::Integer(value)) => (*min..=*max).contains(value),
        (super::ParamType::Enum { values }, super::ParamValue::Text(value)) => values.contains(value),
        (super::ParamType::RelativePath { max_bytes }, super::ParamValue::Text(value)) => {
            value.len() <= *max_bytes && valid_relative_path(value)
        }
        _ => return Err(super::ParamErrorCode::WrongType),
    };
    if is_in_bounds {
        Ok(())
    } else {
        Err(super::ParamErrorCode::OutOfBounds)
    }
}

fn valid_type(value_type: &super::ParamType) -> bool {
    match value_type {
        super::ParamType::Bool {} => true,
        super::ParamType::Integer { min, max } => min <= max,
        super::ParamType::Enum { values } => {
            let unique = values.iter().collect::<std::collections::BTreeSet<_>>();
            !values.is_empty()
                && values.len() <= super::MAX_ENUM_VALUES
                && unique.len() == values.len()
                && values.iter().all(|token| valid_enum_token(token))
        }
        super::ParamType::RelativePath { max_bytes } => (1..=super::MAX_PARAM_TEXT_BYTES).contains(max_bytes),
    }
}

pub(super) fn valid_name(name: &str) -> bool {
    name.len() <= super::MAX_PARAM_NAME_BYTES
        && name.bytes().next().is_some_and(|byte| byte.is_ascii_lowercase())
        && name.bytes().all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

fn valid_env_key(key: &str) -> bool {
    let Some(suffix) = key.strip_prefix(super::PARAM_ENV_PREFIX) else {
        return false;
    };
    key.len() <= super::MAX_ENV_KEY_BYTES
        && !suffix.is_empty()
        && suffix.bytes().all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_')
}

/// An enum token: lowercase ASCII letters, digits, `.`, `_`, and `-`, starting with a letter or
/// digit.
fn valid_enum_token(token: &str) -> bool {
    token.len() <= MAX_ENUM_TOKEN_BYTES
        && token.bytes().next().is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        && token
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-'))
}

/// A lexical relative path over ASCII letters, digits, `.`, `_`, `-`, and `/`.
///
/// It has no empty, parent, or option-like (`-`-prefixed) component, so it can be neither
/// absolute nor escape upward, and it contains no shell metacharacter or whitespace.
fn valid_relative_path(path: &str) -> bool {
    !path.is_empty()
        && path.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-' | b'/'))
        && path
            .split('/')
            .all(|component| !component.is_empty() && component != ".." && !component.starts_with('-'))
}

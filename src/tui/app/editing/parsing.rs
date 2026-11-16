//! Pure typed conversion of node editor drafts.

use super::*;

pub(in crate::tui::app) fn condition_branches_from_draft(
    input: &str,
) -> Result<Vec<ConditionBranch>, PropertyEditError> {
    let branches = input
        .split(CONDITION_BRANCH_ENTRY_SEPARATOR)
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
        .map(condition_branch_from_entry)
        .collect::<Result<Vec<_>, _>>()?;
    let config = ConditionNodeConfig { branches };
    control_flow::validate_condition_config(&config).map_err(|_| PropertyEditError::InvalidConditionBranches)?;
    Ok(config.branches)
}

fn condition_branch_from_entry(entry: &str) -> Result<ConditionBranch, PropertyEditError> {
    let (label, predicate) = entry
        .split_once(CONDITION_BRANCH_BINDING_SEPARATOR)
        .ok_or(PropertyEditError::InvalidConditionBranches)?;
    let label = label.trim();
    if label.is_empty() {
        return Err(PropertyEditError::InvalidConditionBranches);
    }
    let predicate = control_flow::parse_condition_predicate(predicate.trim())
        .map_err(|_| PropertyEditError::InvalidConditionBranches)?;
    Ok(ConditionBranch {
        label: label.to_string(),
        predicate,
    })
}

pub(in crate::tui::app) fn condition_branches_to_draft(config: &ConditionNodeConfig) -> String {
    config
        .branches
        .iter()
        .map(|branch| format!("{}={}", branch.label, condition_predicate_to_draft(&branch.predicate)))
        .collect::<Vec<_>>()
        .join(", ")
}

pub(in crate::tui::app) fn condition_predicate_to_draft(predicate: &control_flow::ConditionPredicate) -> String {
    match predicate {
        control_flow::ConditionPredicate::Always => "always".to_string(),
        control_flow::ConditionPredicate::Exists => "exists".to_string(),
        control_flow::ConditionPredicate::StringEquals(value) => format!("eq:{value}"),
        control_flow::ConditionPredicate::StringContains(value) => format!("contains:{value}"),
        control_flow::ConditionPredicate::NumberGreaterThan(value) => format!("gt:{value}"),
        control_flow::ConditionPredicate::JsonPointerExists(value) => format!("json_exists:{value}"),
        control_flow::ConditionPredicate::BoolEquals(value) => format!("bool:{value}"),
    }
}

pub(in crate::tui::app) fn image_input_config(path: String) -> NodeConfig {
    let allowed_root = std::path::Path::new(&path)
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| std::path::Path::new("."))
        .to_string_lossy()
        .into_owned();
    NodeConfig::ImageInput {
        path,
        allowed_roots: vec![allowed_root],
        max_bytes: crate::image_node::DEFAULT_MAX_IMAGE_BYTES,
    }
}

pub(in crate::tui::app) fn http_config_from_draft(input: &str) -> Result<(String, String), PropertyEditError> {
    let Some((method, url)) = input.trim().split_once(char::is_whitespace) else {
        return Err(PropertyEditError::InvalidHttpConfig);
    };
    let method = method.trim();
    let url = url.trim();
    if !crate::http::validate_http_method(method) || !crate::http::validate_http_url(url) {
        return Err(PropertyEditError::InvalidHttpConfig);
    }
    Ok((method.to_string(), url.to_string()))
}

pub(in crate::tui::app) fn file_config_from_draft(
    input: &str,
) -> Result<(crate::file_node::FileOperationKind, String), PropertyEditError> {
    let Some((operation, path)) = input.trim().split_once(char::is_whitespace) else {
        return Err(PropertyEditError::InvalidFileConfig);
    };
    let operation = match operation.trim().to_ascii_lowercase().as_str() {
        "read" => crate::file_node::FileOperationKind::Read,
        "write" => crate::file_node::FileOperationKind::Write,
        "append" => crate::file_node::FileOperationKind::Append,
        _ => return Err(PropertyEditError::InvalidFileConfig),
    };
    let path = path.trim();
    if path.is_empty() {
        return Err(PropertyEditError::InvalidFileConfig);
    }
    Ok((operation, path.to_string()))
}

pub(in crate::tui::app) fn merge_config_from_draft(input: &str) -> Result<MergeNodeConfig, PropertyEditError> {
    let trimmed = input.trim();
    let (policy, inputs) = match trimmed.split_once(MERGE_POLICY_INPUT_SEPARATOR) {
        Some((policy, inputs)) => (policy.trim(), inputs),
        None => (trimmed, ""),
    };
    let policy = merge_policy_from_draft(policy)?;
    let required_inputs = merge_inputs_from_draft(inputs);
    let config = MergeNodeConfig {
        policy,
        required_inputs,
    };
    control_flow::validate_merge_config(&config).map_err(|_| PropertyEditError::InvalidMergePolicy)?;
    Ok(config)
}

fn merge_policy_from_draft(policy: &str) -> Result<MergePolicy, PropertyEditError> {
    match policy {
        "all" | "" => Ok(MergePolicy::All),
        "first" => Ok(MergePolicy::First),
        "last" => Ok(MergePolicy::Last),
        "collect" => Ok(MergePolicy::Collect),
        _ => Err(PropertyEditError::InvalidMergePolicy),
    }
}

fn merge_inputs_from_draft(inputs: &str) -> Vec<String> {
    let required_inputs = inputs
        .split(MERGE_INPUT_SEPARATOR)
        .map(str::trim)
        .filter(|input| !input.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();
    if required_inputs.is_empty() {
        vec![
            DEFAULT_MERGE_INPUT_LEFT.to_string(),
            DEFAULT_MERGE_INPUT_RIGHT.to_string(),
        ]
    } else {
        required_inputs
    }
}

pub(in crate::tui::app) fn optional_draft_value(input: &str) -> Option<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

pub(in crate::tui::app) fn context_refs_from_draft(input: &str) -> Vec<String> {
    input.split(',').map(str::trim).filter(|value| !value.is_empty()).map(str::to_string).collect()
}

// r[impl n8nclone.tui_ux.credentials.providers]
pub(in crate::tui::app) fn credentials_from_draft(input: &str) -> Result<NodeCredentialRefs, PropertyEditError> {
    let mut refs = NodeCredentialRefs::default();
    for entry in input.split(CREDENTIAL_ENTRY_SEPARATOR).map(str::trim).filter(|entry| !entry.is_empty()) {
        insert_credential(&mut refs, entry)?;
    }
    Ok(refs)
}

fn insert_credential(refs: &mut NodeCredentialRefs, entry: &str) -> Result<(), PropertyEditError> {
    let (env_name, identity) =
        entry.split_once(CREDENTIAL_BINDING_SEPARATOR).ok_or(PropertyEditError::InvalidAgentCredentials)?;
    let (provider, key_with_field) = identity
        .trim()
        .split_once(SECRET_IDENTITY_SEPARATOR)
        .ok_or(PropertyEditError::InvalidAgentCredentials)?;
    let (key, field) = match key_with_field.split_once(SECRET_FIELD_SEPARATOR) {
        Some((key, field)) => (key, optional_draft_value(field)),
        None => (key_with_field, None),
    };
    let (env_name, provider, key) = (env_name.trim(), provider.trim(), key.trim());
    if [env_name, provider, key].iter().any(|value| value.is_empty()) || refs.env.contains_key(env_name) {
        return Err(PropertyEditError::InvalidAgentCredentials);
    }
    refs.env.insert(env_name.to_string(), SecretRef::new(provider, key, field));
    Ok(())
}

pub(in crate::tui) fn credentials_to_draft(credentials: Option<&NodeCredentialRefs>) -> String {
    let mut refs = credentials.map(|credentials| credentials.env.iter().collect::<Vec<_>>()).unwrap_or_default();
    refs.sort_by_key(|(env_name, _)| *env_name);
    refs.into_iter()
        .map(|(env_name, secret_ref)| format!("{}={}", env_name, secret_ref.identity()))
        .collect::<Vec<_>>()
        .join(", ")
}

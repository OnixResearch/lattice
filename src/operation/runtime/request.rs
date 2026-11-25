fn safe_label(value: &str) -> Option<&str> {
    if !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
    {
        Some(value)
    } else {
        None
    }
}

pub(super) fn resolve_credentials(
    plan: &crate::operation::model::InvocationPlan,
    provider: &dyn crate::credentials::CredentialProvider,
) -> Result<Vec<String>, super::Observation> {
    let mut values = Vec::with_capacity(plan.credentials.len());
    let workflow_id = safe_label(&plan.instance_id).unwrap_or("operation");
    let node_id = safe_label(&plan.method).unwrap_or("invoke");
    for (binding, reference) in &plan.credentials {
        let context = crate::credentials::CredentialResolutionContext {
            subject: crate::credentials::CredentialSubject::Node {
                workflow_id: workflow_id.to_string(),
                node_id: node_id.to_string(),
            },
            binding: safe_label(binding).unwrap_or("credential").to_string(),
            reference: reference.clone(),
            destination: crate::credentials::CredentialDestination::AgentEnvironment,
            reason: provider.resolution_reason(reference).unwrap_or_default().to_owned(),
        };
        let value = provider
            .resolve_in_context(reference, &context, None)
            .map_err(|_| super::deterministic("operation credential resolution failed"))?;
        values.push(value.expose_for_execution().to_string());
    }
    Ok(values)
}

pub(super) fn repository_root(plan: &crate::operation::model::InvocationPlan) -> Result<String, super::Observation> {
    let Some(root) = plan.target.get(super::REPOSITORY_ROOT_TARGET).and_then(serde_json::Value::as_str) else {
        return Err(super::deterministic("operation repository root target is missing"));
    };
    if !std::path::Path::new(root).is_absolute() {
        return Err(super::deterministic("operation repository root must be absolute"));
    }
    Ok(root.to_string())
}

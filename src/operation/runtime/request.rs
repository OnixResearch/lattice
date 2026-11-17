pub(super) fn resolve_credentials(
    plan: &crate::operation::model::InvocationPlan,
    provider: &dyn crate::credentials::CredentialProvider,
) -> Result<Vec<String>, super::Observation> {
    let mut values = Vec::with_capacity(plan.credentials.len());
    for reference in plan.credentials.values() {
        let value = provider
            .resolve(reference)
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

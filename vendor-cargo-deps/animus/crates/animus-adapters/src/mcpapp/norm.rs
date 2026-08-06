pub(super) fn required_text(
    field_name: crate::validation::FieldName,
    value: Option<&str>,
) -> Result<super::Text, super::EvaluationError> {
    match value {
        Some(text) => Ok(crate::validation::normalized_adapter_field(field_name, text)?),
        None => Err(crate::AdapterContractError::EmptyField {
            field: field_name.as_str(),
        }
        .into()),
    }
}

pub(super) fn normalize_collection(
    field_name: crate::validation::FieldName,
    values: &[&str],
    max_items: usize,
) -> Result<super::Collection<super::Text>, super::EvaluationError> {
    let actual_items = values.len();
    assert!(max_items > 0, "collection ceiling must be positive");
    if actual_items > max_items {
        return Err(super::EvaluationError::TooManyItems {
            field: field_name.as_str(),
            max_items,
            actual_items,
        });
    }
    let mut normalized = super::Collection::with_capacity(actual_items);
    for value in values {
        normalized.push(crate::validation::normalized_adapter_field(field_name, value)?);
    }
    Ok(normalized)
}

pub(super) fn parse_size_policy(value: &str) -> Option<super::SizePolicy> {
    match value {
        super::SIZE_POLICY_BOUNDED => Some(super::SizePolicy::Bounded),
        super::SIZE_POLICY_HOST_FIT => Some(super::SizePolicy::HostFit),
        _unsupported => None,
    }
}

pub(super) fn parse_lifecycle_policy(value: &str) -> Option<super::LifecyclePolicy> {
    match value {
        super::LIFECYCLE_POLICY_PASSIVE => Some(super::LifecyclePolicy::Passive),
        super::LIFECYCLE_POLICY_BOUNDED_CALLBACKS => Some(super::LifecyclePolicy::BoundedCallbacks),
        super::LIFECYCLE_POLICY_UNBOUNDED_CALLBACKS => Some(super::LifecyclePolicy::UnboundedCallbacks),
        _unsupported => None,
    }
}

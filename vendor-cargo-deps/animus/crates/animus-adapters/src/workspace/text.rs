pub(super) fn parse_value<T>(
    field_name: crate::validation::FieldName,
    value: &str,
    parser: impl FnOnce(&str) -> Option<T>,
) -> Result<T, crate::AdapterContractError> {
    let normalized = crate::validation::normalized_adapter_field(field_name, value)?;
    parser(normalized.as_str()).ok_or_else(|| {
        crate::AdapterContractError::UnsupportedWorkspaceValue {
            field: field_name.as_str(),
            value: normalized,
        }
    })
}

pub(super) fn optional_field(
    field_name: crate::validation::FieldName,
    value: Option<&str>,
) -> Result<Option<alloc::string::String>, crate::AdapterContractError> {
    value
        .map(|text| crate::validation::normalized_adapter_field(field_name, text))
        .transpose()
}

pub(super) fn required_texts(
    collection_field: crate::validation::FieldName,
    item_field: crate::validation::FieldName,
    values: &[&str],
) -> Result<alloc::vec::Vec<alloc::string::String>, crate::AdapterContractError> {
    if values.is_empty() {
        return Err(crate::AdapterContractError::EmptyCollection {
            field: collection_field.as_str(),
        });
    }

    values
        .iter()
        .map(|value| crate::validation::normalized_adapter_field(item_field, value))
        .collect()
}

pub(super) fn collect_capabilities(
    values: &[&str],
    parse: impl Fn(&str) -> Result<super::adapter::Capability, crate::AdapterContractError>,
) -> Result<alloc::vec::Vec<super::adapter::Capability>, crate::AdapterContractError> {
    values.iter().map(|value| parse(value)).collect()
}


fn normalize(
    field: crate::validation::FieldName,
    value: &str,
) -> Result<String, PromptAssemblyError> {
    crate::validation::normalized_adapter_field(field, value).map_err(PromptAssemblyError::Contract)
}

fn normalize_optional(
    field: crate::validation::FieldName,
    value: Option<&str>,
) -> Result<Option<String>, PromptAssemblyError> {
    value.map(|inner| normalize(field, inner)).transpose()
}


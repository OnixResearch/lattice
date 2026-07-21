use super::{
    Collection, PluginDescriptorError, TOOL_CAPABILITY_FIELD, TOOL_DESCRIPTION_FIELD,
    TOOL_NAME_FIELD, TOOL_RESULT_FIELD, TOOL_SCHEMA_FIELD, ToolDescriptor, ToolDescriptorDraft,
    contains_name, required_text, validate_required_texts,
};

pub(in crate::manifest) fn collect_descriptors(
    drafts: &[ToolDescriptorDraft<'_>],
    built_in_tool_names: &[&str],
    existing_catalog_tool_names: &[&str],
) -> Result<Collection<ToolDescriptor>, PluginDescriptorError> {
    assert!(
        drafts.len() <= crate::MAX_ADAPTER_FIELD_CHARS,
        "manifest tool count must fit the adapter field bound"
    );
    assert!(
        built_in_tool_names.len() <= crate::MAX_ADAPTER_FIELD_CHARS,
        "built-in tool count must fit the adapter field bound"
    );
    assert!(
        existing_catalog_tool_names.len() <= crate::MAX_ADAPTER_FIELD_CHARS,
        "catalog tool count must fit the adapter field bound"
    );
    let mut tools = Collection::with_capacity(drafts.len());
    for draft in drafts {
        let name = required_text(TOOL_NAME_FIELD, draft.name)?;
        if contains_name(built_in_tool_names, &name)
            || contains_name(existing_catalog_tool_names, &name)
        {
            return Err(PluginDescriptorError::ToolNameCollision { name });
        }
        if tools
            .iter()
            .any(|tool: &ToolDescriptor| tool.name() == name)
        {
            return Err(PluginDescriptorError::DuplicateTool { name });
        }
        let description = required_text(TOOL_DESCRIPTION_FIELD, draft.description)?;
        let parameter_schema_identity =
            required_text(TOOL_SCHEMA_FIELD, draft.parameter_schema_identity)?;
        let result_contract_identity =
            required_text(TOOL_RESULT_FIELD, draft.result_contract_identity)?;
        let capability_labels =
            validate_required_texts(TOOL_CAPABILITY_FIELD, draft.capability_labels)?;
        tools.push(ToolDescriptor {
            name,
            description,
            parameter_schema_identity,
            result_contract_identity,
            capability_labels,
        });
    }
    Ok(tools)
}

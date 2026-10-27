use super::{
    Collection, InteropError, MCP_ABILITY_POLICY_FIELD, MCP_CAPABILITY_FIELD,
    MCP_MAPPING_DESCRIPTION, MCP_MAPPING_GUIDELINES, MCP_MAPPING_PROMPT_SNIPPET,
    MCP_RESOURCE_POLICY_FIELD, MCP_RESULT_BOUNDS_FIELD, MCP_SERVER_LABEL_FIELD, MCP_SOURCE_FIELD,
    MCP_TOOL_NAME_FIELD, SCHEMA_FIELD, Text, borrowed_texts, required_optional_text,
    text_collection,
};

/// Borrowed MCP-to-Animus tool mapping before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct McpToolMappingDraft<'a> {
    /// Host-redacted server label.
    pub server_label: Option<&'a str>,
    /// Animus tool name produced from the MCP tool descriptor.
    pub tool_name: Option<&'a str>,
    /// Strict argument schema identity.
    pub schema_identity: Option<&'a str>,
    /// Animus capability labels required by the mapped tool.
    pub capability_labels: &'a [&'a str],
    /// Safe source/provenance metadata supplied by the host.
    pub source_metadata: Option<&'a str>,
    /// Bounded result contract or metadata-shape identity.
    pub result_bounds: Option<&'a str>,
    /// Resource policy references checked before dispatch.
    pub resource_policy_refs: &'a [&'a str],
    /// Ability policy references checked before dispatch.
    pub ability_policy_refs: &'a [&'a str],
}

// r[impl agent_harness_runtime.claude_code_mcp.mapping]
/// Validated MCP-to-Animus tool mapping.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct McpToolMapping {
    server_label: Text,
    tool_name: Text,
    schema_identity: Text,
    capability_labels: Collection<Text>,
    source_metadata: Text,
    result_bounds: Text,
    resource_policy_refs: Collection<Text>,
    ability_policy_refs: Collection<Text>,
}

impl McpToolMapping {
    /// Validates an MCP mapping before constructing an Animus tool definition.
    ///
    /// # Errors
    ///
    /// Returns [`InteropError`] when required mapping data is absent, malformed,
    /// duplicated, or when policy/capability collections are empty.
    pub fn validate(draft: &McpToolMappingDraft<'_>) -> Result<Self, InteropError> {
        Ok(Self {
            server_label: required_optional_text(MCP_SERVER_LABEL_FIELD, draft.server_label)?,
            tool_name: required_optional_text(MCP_TOOL_NAME_FIELD, draft.tool_name)?,
            schema_identity: required_optional_text(SCHEMA_FIELD, draft.schema_identity)?,
            capability_labels: text_collection(
                MCP_CAPABILITY_FIELD,
                draft.capability_labels,
                true,
            )?,
            source_metadata: required_optional_text(MCP_SOURCE_FIELD, draft.source_metadata)?,
            result_bounds: required_optional_text(MCP_RESULT_BOUNDS_FIELD, draft.result_bounds)?,
            resource_policy_refs: text_collection(
                MCP_RESOURCE_POLICY_FIELD,
                draft.resource_policy_refs,
                true,
            )?,
            ability_policy_refs: text_collection(
                MCP_ABILITY_POLICY_FIELD,
                draft.ability_policy_refs,
                true,
            )?,
        })
    }

    /// Converts the validated mapping into the existing Animus tool definition contract.
    ///
    /// # Errors
    ///
    /// Returns [`crate::tool::ContractError`] if the synthesized tool definition
    /// fails the common Animus tool-contract validation path.
    pub fn to_tool_definition(
        &self,
    ) -> Result<crate::tool::Definition, crate::tool::ContractError> {
        let capability_labels = borrowed_texts(&self.capability_labels);
        crate::tool::Definition::validate(&crate::tool::DefinitionDraft {
            name: Some(self.tool_name.as_str()),
            label: Some(self.tool_name.as_str()),
            description: Some(MCP_MAPPING_DESCRIPTION),
            parameter_schema_identity: Some(self.schema_identity.as_str()),
            prompt_snippet: Some(MCP_MAPPING_PROMPT_SNIPPET),
            prompt_guidelines: MCP_MAPPING_GUIDELINES,
            source_identity: Some(self.source_metadata.as_str()),
            provenance: Some(self.server_label.as_str()),
            capability_labels: capability_labels.as_slice(),
            result_metadata_shape: Some(self.result_bounds.as_str()),
            status: crate::tool::DefinitionStatus::Active,
        })
    }

    /// Returns the host-redacted server label.
    #[must_use]
    pub fn server_label(&self) -> &str {
        self.server_label.as_str()
    }

    /// Returns the Animus tool name.
    #[must_use]
    pub fn tool_name(&self) -> &str {
        self.tool_name.as_str()
    }

    /// Returns the strict schema identity.
    #[must_use]
    pub fn schema_identity(&self) -> &str {
        self.schema_identity.as_str()
    }

    /// Returns required Animus capability labels.
    #[must_use]
    pub fn capability_labels(&self) -> &[Text] {
        self.capability_labels.as_slice()
    }

    /// Returns source metadata.
    #[must_use]
    pub fn source_metadata(&self) -> &str {
        self.source_metadata.as_str()
    }

    /// Returns result bounds metadata.
    #[must_use]
    pub fn result_bounds(&self) -> &str {
        self.result_bounds.as_str()
    }

    /// Returns resource policy references.
    #[must_use]
    pub fn resource_policy_refs(&self) -> &[Text] {
        self.resource_policy_refs.as_slice()
    }

    /// Returns ability policy references.
    #[must_use]
    pub fn ability_policy_refs(&self) -> &[Text] {
        self.ability_policy_refs.as_slice()
    }
}

use super::{
    COMMAND_FIELD, CONTRACT_FIELD, Collection, ENDPOINT_FIELD, ENV_KEY_FIELD, ENV_VALUE_SEPARATOR,
    HashInput, ID_FIELD, InteropError, LABEL_SPACE, McpTransportKind, PROMPT_FIELD,
    PROVENANCE_FIELD, RAW_URL_MARKER, RESOURCE_FIELD, SCHEMA_FIELD, TIMEOUT_FIELD, TOOL_FIELD,
    Text, required_text, text_collection, validate_hash,
};

/// Untrusted MCP descriptor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct McpDescriptorDraft<'a> {
    /// Descriptor identity.
    pub identity: &'a str,
    /// MCP transport label.
    pub transport: &'a str,
    /// Host-owned command label for stdio transports.
    pub command_label: Option<&'a str>,
    /// Host-owned endpoint label for HTTP transports.
    pub endpoint_label: Option<&'a str>,
    /// Tool identities declared by the MCP server.
    pub tools: &'a [&'a str],
    /// Resource identities declared by the MCP server.
    pub resources: &'a [&'a str],
    /// Prompt identities declared by the MCP server.
    pub prompts: &'a [&'a str],
    /// Schema identity labels.
    pub schema_ids: &'a [&'a str],
    /// Result contract labels.
    pub result_contracts: &'a [&'a str],
    /// Timeout class label.
    pub timeout_class: &'a str,
    /// Environment key names required by the extension.
    pub environment_keys: &'a [&'a str],
    /// Available tool ceilings.
    pub available_tool_ceilings: &'a [&'a str],
    /// BLAKE3 provenance hash.
    pub provenance_hash: &'a str,
}

/// Validated MCP capability descriptor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct McpDescriptor {
    identity: Text,
    transport: McpTransportKind,
    command_label: Option<Text>,
    endpoint_label: Option<Text>,
    tools: Collection<Text>,
    timeout_class: Text,
    environment_keys: Collection<Text>,
    provenance_hash: Text,
}

impl McpDescriptor {
    /// Validates an MCP descriptor without starting a server or probing network.
    ///
    /// # Errors
    ///
    /// Returns [`InteropError`] when fields are malformed, transport labels are
    /// unsupported, timeout is missing, tools collide, env values leak, or raw
    /// command/endpoint material is present.
    pub fn validate(draft: &McpDescriptorDraft<'_>) -> Result<Self, InteropError> {
        let identity = required_text(ID_FIELD, draft.identity)?;
        let mcp_kind = McpTransportKind::parse(draft.transport)?;
        let command_label = optional_runtime_label(COMMAND_FIELD, draft.command_label)?;
        let endpoint_label = optional_runtime_label(ENDPOINT_FIELD, draft.endpoint_label)?;
        validate_transport_labels(mcp_kind, command_label.as_ref(), endpoint_label.as_ref())?;
        let tools = text_collection(TOOL_FIELD, draft.tools, true)?;
        text_collection(RESOURCE_FIELD, draft.resources, false)?;
        text_collection(PROMPT_FIELD, draft.prompts, false)?;
        text_collection(SCHEMA_FIELD, draft.schema_ids, true)?;
        text_collection(CONTRACT_FIELD, draft.result_contracts, true)?;
        let deadline_label =
            required_text(TIMEOUT_FIELD, draft.timeout_class).map_err(map_empty_to_timeout)?;
        let environment_keys = validate_environment_keys(draft.environment_keys)?;
        text_collection(TOOL_FIELD, draft.available_tool_ceilings, true)?;
        let provenance_hash = required_text(PROVENANCE_FIELD, draft.provenance_hash)?;
        validate_hash(HashInput {
            field: PROVENANCE_FIELD.as_str(),
            value: provenance_hash.as_str(),
        })?;
        Ok(Self {
            identity,
            transport: mcp_kind,
            command_label,
            endpoint_label,
            tools,
            timeout_class: deadline_label,
            environment_keys,
            provenance_hash,
        })
    }

    /// Returns descriptor identity.
    #[must_use]
    pub fn identity(&self) -> &str {
        self.identity.as_str()
    }

    /// Returns transport kind.
    #[must_use]
    pub const fn transport(&self) -> McpTransportKind {
        self.transport
    }

    /// Returns command label, when present.
    #[must_use]
    pub fn command_label(&self) -> Option<&str> {
        self.command_label.as_deref()
    }

    /// Returns endpoint label, when present.
    #[must_use]
    pub fn endpoint_label(&self) -> Option<&str> {
        self.endpoint_label.as_deref()
    }

    /// Returns declared tool identities.
    #[must_use]
    pub fn tools(&self) -> &[Text] {
        self.tools.as_slice()
    }

    /// Returns timeout class.
    #[must_use]
    pub fn timeout_class(&self) -> &str {
        self.timeout_class.as_str()
    }

    /// Returns environment key names.
    #[must_use]
    pub fn environment_keys(&self) -> &[Text] {
        self.environment_keys.as_slice()
    }

    /// Returns provenance hash.
    #[must_use]
    pub fn provenance_hash(&self) -> &str {
        self.provenance_hash.as_str()
    }
}
fn validate_transport_labels(
    transport: McpTransportKind,
    command_label: Option<&Text>,
    endpoint_label: Option<&Text>,
) -> Result<(), InteropError> {
    match transport {
        McpTransportKind::Stdio if command_label.is_none() => Err(InteropError::UnsafeRuntimeLabel),
        McpTransportKind::StreamableHttp if endpoint_label.is_none() => {
            Err(InteropError::UnsafeRuntimeLabel)
        }
        _ => Ok(()),
    }
}

fn validate_environment_keys(values: &[&str]) -> Result<Collection<Text>, InteropError> {
    let keys = text_collection(ENV_KEY_FIELD, values, false)?;
    if keys
        .iter()
        .any(|key| key.contains(ENV_VALUE_SEPARATOR) || key.contains(RAW_URL_MARKER))
    {
        return Err(InteropError::EnvironmentValueLeak);
    }
    Ok(keys)
}

fn optional_runtime_label(
    field_name: crate::validation::FieldName,
    value: Option<&str>,
) -> Result<Option<Text>, InteropError> {
    let Some(text) = value else {
        return Ok(None);
    };
    let normalized = required_text(field_name, text)?;
    if normalized.contains(RAW_URL_MARKER) || normalized.contains(LABEL_SPACE) {
        return Err(InteropError::UnsafeRuntimeLabel);
    }
    Ok(Some(normalized))
}
fn map_empty_to_timeout(error: InteropError) -> InteropError {
    match error {
        InteropError::Contract(crate::AdapterContractError::EmptyField { .. }) => {
            InteropError::MissingTimeout
        }
        other => other,
    }
}

use super::{Collection, InteropError, TOOL_FIELD, Text, contains_text, text_collection};

/// Tool pass-through intersection input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ToolPassThroughInput<'a> {
    /// Tool labels requested by the bridge.
    pub requested_tools: &'a [&'a str],
    /// Animus capability ceiling tool labels.
    pub animus_capability_ceilings: &'a [&'a str],
    /// Tool labels admitted by active permission mode.
    pub permission_mode_tools: &'a [&'a str],
    /// Tool labels admitted by host policy.
    pub host_admission: &'a [&'a str],
    /// Tool labels admitted by external agent mode.
    pub external_agent_mode: &'a [&'a str],
    /// Tool labels declared as available by the descriptor.
    pub available_tools: &'a [&'a str],
}

/// Tool pass-through resolution.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToolPassThroughResolution {
    allowed_tools: Collection<Text>,
    denied_tools: Collection<Text>,
}

impl ToolPassThroughResolution {
    /// Returns allowed tool labels.
    #[must_use]
    pub fn allowed_tools(&self) -> &[Text] {
        self.allowed_tools.as_slice()
    }

    /// Returns denied tool labels.
    #[must_use]
    pub fn denied_tools(&self) -> &[Text] {
        self.denied_tools.as_slice()
    }
}

/// Resolves bridge tool pass-through by full capability intersection.
///
/// # Errors
///
/// Returns [`InteropError`] when tool labels are malformed or duplicated.
///
/// # Panics
///
/// Panics if required requested-tool validation is weakened before intersection.
pub fn resolve_tool_pass_through(
    input: ToolPassThroughInput<'_>,
) -> Result<ToolPassThroughResolution, InteropError> {
    let requested = text_collection(TOOL_FIELD, input.requested_tools, true)?;
    assert!(!requested.is_empty(), "requested tools are required");
    assert!(
        requested.len() <= input.requested_tools.len(),
        "normalized tools are bounded"
    );
    let animus = text_collection(TOOL_FIELD, input.animus_capability_ceilings, true)?;
    let permission = text_collection(TOOL_FIELD, input.permission_mode_tools, true)?;
    let host = text_collection(TOOL_FIELD, input.host_admission, true)?;
    let external = text_collection(TOOL_FIELD, input.external_agent_mode, true)?;
    let available = text_collection(TOOL_FIELD, input.available_tools, true)?;
    let tool_count = requested.len();
    let mut allowed_tools = Collection::with_capacity(tool_count);
    let mut denied_tools = Collection::with_capacity(tool_count);
    let sets = ToolIntersectionSets {
        animus: &animus,
        permission: &permission,
        host: &host,
        external: &external,
        available: &available,
    };
    for tool in requested {
        if contains_everywhere(tool.as_str(), sets) {
            allowed_tools.push(tool);
        } else {
            denied_tools.push(tool);
        }
    }
    Ok(ToolPassThroughResolution {
        allowed_tools,
        denied_tools,
    })
}
#[derive(Clone, Copy)]
struct ToolIntersectionSets<'a> {
    animus: &'a [Text],
    permission: &'a [Text],
    host: &'a [Text],
    external: &'a [Text],
    available: &'a [Text],
}

fn contains_everywhere(tool: &str, sets: ToolIntersectionSets<'_>) -> bool {
    if !contains_text(sets.animus, tool) {
        return false;
    }
    if !contains_text(sets.permission, tool) {
        return false;
    }
    if !contains_text(sets.host, tool) {
        return false;
    }
    if !contains_text(sets.external, tool) {
        return false;
    }
    contains_text(sets.available, tool)
}

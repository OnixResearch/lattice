use super::McpDescriptor;

// r[impl agent_harness_runtime.claude_code_mcp.scope]
/// Trust state supplied by a host before MCP tools become model-visible.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum McpServerTrust {
    /// The host has not explicitly registered and trusted the server.
    Untrusted,
    /// The host explicitly registered and trusted the server descriptor.
    Registered,
}

/// Data-only advertisement input for MCP server descriptors.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct McpAdvertisementInput<'a> {
    /// Validated descriptor, when host registration succeeded.
    pub descriptor: Option<&'a McpDescriptor>,
    /// Host trust decision for the descriptor.
    pub trust: McpServerTrust,
}

/// Pure advertisement decision; reusable runtime code never probes transports.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct McpAdvertisementDecision {
    is_advertised: bool,
    did_start_transport_probe: bool,
}

impl McpAdvertisementDecision {
    /// Returns whether MCP tools may be advertised to the model.
    #[must_use]
    pub const fn advertised(self) -> bool {
        self.is_advertised
    }

    /// Returns whether a process, network, or transport probe was started.
    #[must_use]
    pub const fn transport_probe_started(self) -> bool {
        self.did_start_transport_probe
    }
}

// r[impl agent_harness_runtime.claude_code_mcp.scope.untrusted]
/// Plans model advertisement without server discovery, process launch, or network probes.
#[must_use]
pub const fn plan_mcp_advertisement(input: McpAdvertisementInput<'_>) -> McpAdvertisementDecision {
    let is_advertised =
        matches!(input.trust, McpServerTrust::Registered) && input.descriptor.is_some();
    McpAdvertisementDecision {
        is_advertised,
        did_start_transport_probe: false,
    }
}

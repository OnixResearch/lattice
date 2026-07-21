/// Argument validation status supplied by the normal Animus argument path.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum McpArgumentStatus {
    /// Arguments passed strict Animus validation.
    Valid,
    /// Arguments failed before any MCP adapter call.
    Invalid,
}

/// Mutation coordination status for an MCP-backed tool call.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum McpMutationStatus {
    /// The call may proceed without waiting for another mutation window.
    Independent,
    /// The call must wait for host-owned mutation coordination before dispatch.
    Serialized,
}

/// MCP-backed dispatch readiness input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct McpDispatchRequest<'a> {
    /// Existing Animus tool definition, after MCP mapping.
    pub definition: Option<&'a crate::tool::Definition>,
    /// Existing Animus argument validation result.
    pub argument_status: McpArgumentStatus,
    /// Existing injected executor status.
    pub executor_status: crate::tool::ExecutorStatus,
    /// Existing capability authorization result.
    pub capability_status: crate::tool::CapabilityStatus,
    /// Existing mutation-window coordination result.
    pub mutation_status: McpMutationStatus,
    /// Existing result-boundary status.
    pub output_boundary_status: crate::tool::OutputBoundaryStatus,
}

/// MCP-backed dispatch decision with explicit adapter-call evidence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct McpDispatchDecision {
    dispatch: crate::tool::DispatchDecision,
    is_server_adapter_called: bool,
}

impl McpDispatchDecision {
    /// Returns the underlying Animus dispatch decision.
    #[must_use]
    pub const fn dispatch(self) -> crate::tool::DispatchDecision {
        self.dispatch
    }

    /// Returns whether the MCP server adapter may be called immediately.
    #[must_use]
    pub const fn server_adapter_called(self) -> bool {
        self.is_server_adapter_called
    }
}

// r[impl agent_harness_runtime.claude_code_mcp.dispatch]
// r[impl agent_harness_runtime.claude_code_mcp.dispatch.denied]
/// Evaluates MCP dispatch through existing Animus validation and permission gates.
///
/// # Panics
///
/// Panics if status enums become non-exhaustive without updating the dispatch matrix.
#[must_use]
pub const fn evaluate_mcp_dispatch(request: McpDispatchRequest<'_>) -> McpDispatchDecision {
    assert!(
        matches!(
            request.argument_status,
            McpArgumentStatus::Valid | McpArgumentStatus::Invalid
        ),
        "argument status is exhaustive"
    );
    assert!(
        matches!(
            request.mutation_status,
            McpMutationStatus::Independent | McpMutationStatus::Serialized
        ),
        "mutation status is exhaustive"
    );
    let dispatch = dispatch_after_argument_check(request);
    let is_server_adapter_called = matches!(dispatch.status(), crate::tool::DispatchStatus::Ready)
        && matches!(request.mutation_status, McpMutationStatus::Independent);
    McpDispatchDecision {
        dispatch,
        is_server_adapter_called,
    }
}

const fn dispatch_after_argument_check(
    request: McpDispatchRequest<'_>,
) -> crate::tool::DispatchDecision {
    if matches!(request.argument_status, McpArgumentStatus::Invalid) {
        return crate::tool::evaluate_dispatch(crate::tool::DispatchRequest {
            definition: None,
            executor_status: request.executor_status,
            capability_status: request.capability_status,
            output_boundary_status: request.output_boundary_status,
        });
    }
    crate::tool::evaluate_dispatch(crate::tool::DispatchRequest {
        definition: request.definition,
        executor_status: request.executor_status,
        capability_status: request.capability_status,
        output_boundary_status: request.output_boundary_status,
    })
}

// r[impl agent_harness_runtime.claude_code_mcp.transport]
/// Host-owned transport lifecycle state for a registered MCP server adapter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum McpTransportState {
    /// Transport is available to the host adapter.
    Available,
    /// Required transport service is unavailable.
    Unavailable,
    /// Selected host adapter does not support the requested transport.
    Unsupported,
    /// Policy denied transport preparation.
    Denied,
    /// Transport preparation or call timed out.
    Timeout,
    /// Host adapter returned malformed response metadata.
    MalformedResponse,
}

/// Typed transport failure class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum McpTransportFailureClass {
    /// Required transport service is unavailable.
    Unavailable,
    /// Selected host adapter does not support the requested transport.
    Unsupported,
    /// Policy denied transport preparation.
    Denied,
    /// Transport preparation or call timed out.
    Timeout,
    /// Host adapter returned malformed response metadata.
    MalformedResponse,
}

/// Pure transport lifecycle decision; fallback effects are never attempted here.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct McpTransportDecision {
    is_available: bool,
    failure_class: Option<McpTransportFailureClass>,
    did_attempt_fallback: bool,
}

impl McpTransportDecision {
    /// Returns whether transport may proceed through the injected host adapter.
    #[must_use]
    pub const fn available(self) -> bool {
        self.is_available
    }

    /// Returns the typed failure class, if any.
    #[must_use]
    pub const fn failure_class(self) -> Option<McpTransportFailureClass> {
        self.failure_class
    }

    /// Returns whether shell, network, or provider fallback was attempted.
    #[must_use]
    pub const fn fallback_attempted(self) -> bool {
        self.did_attempt_fallback
    }
}

// r[impl agent_harness_runtime.claude_code_mcp.transport.unsupported]
/// Evaluates host-owned MCP transport readiness without fallback probing.
#[must_use]
pub const fn evaluate_mcp_transport(state: McpTransportState) -> McpTransportDecision {
    let failure_class = transport_failure_class(state);
    McpTransportDecision {
        is_available: failure_class.is_none(),
        failure_class,
        did_attempt_fallback: false,
    }
}

const fn transport_failure_class(state: McpTransportState) -> Option<McpTransportFailureClass> {
    match state {
        McpTransportState::Available => None,
        McpTransportState::Unavailable => Some(McpTransportFailureClass::Unavailable),
        McpTransportState::Unsupported => Some(McpTransportFailureClass::Unsupported),
        McpTransportState::Denied => Some(McpTransportFailureClass::Denied),
        McpTransportState::Timeout => Some(McpTransportFailureClass::Timeout),
        McpTransportState::MalformedResponse => Some(McpTransportFailureClass::MalformedResponse),
    }
}

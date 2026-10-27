const TOOL_REQUEST_SHAPE_CURRENT: &str = "current_structured_tool_call";
const TOOL_OUTPUT_BOUNDARY_BOUNDED: &str = "bounded";
const TOOL_OUTPUT_BOUNDARY_UNSAFE_REFERENCE: &str = "unsafe_reference";
const TOOL_OUTPUT_BOUNDARY_UNBOUNDED: &str = "unbounded";
const TOOL_CANCELLATION_CONTINUE: &str = "continue";
const TOOL_CANCELLATION_CANCELLED: &str = "cancelled";
const TOOL_TIMEOUT_WITHIN_DEADLINE: &str = "within_deadline";
const TOOL_TIMEOUT_TIMED_OUT: &str = "timed_out";
const TOOL_FALLBACK_DENY: &str = "deny";
const FALLBACK_REQUESTED: bool = true;
const FALLBACK_NOT_REQUESTED: bool = false;
const ISSUE_NONE: &str = "none";
const ISSUE_INVALID_REQUEST_SHAPE: &str = "invalid_request_shape";
const ISSUE_UNKNOWN_TOOL: &str = "unknown_tool";
const ISSUE_INACTIVE_TOOL: &str = "inactive_tool";
const ISSUE_UNTRUSTED_TOOL: &str = "untrusted_tool";
const ISSUE_SCHEMA_IDENTITY_MISMATCH: &str = "schema_identity_mismatch";
const ISSUE_MISSING_EXECUTOR: &str = "missing_executor";
const ISSUE_UNAVAILABLE_EXECUTOR: &str = "unavailable_executor";
const ISSUE_DENIED_CAPABILITY: &str = "denied_capability";
const ISSUE_APPROVAL_DENIED: &str = "approval_denied";
const ISSUE_CANCELLED: &str = "cancelled";
const ISSUE_TIMED_OUT: &str = "timed_out";
const ISSUE_MALFORMED_OUTPUT_POLICY: &str = "malformed_output_policy";
const ISSUE_UNSAFE_REFERENCE: &str = "unsafe_reference";
const ISSUE_UNBOUNDED_OUTPUT: &str = "unbounded_output";
const ISSUE_MUTATION_CONFLICT: &str = "mutation_conflict";
const ISSUE_FALLBACK_DENIED: &str = "fallback_denied";

/// Tool-neutral contract identity recorded by admitted tool dispatch plans.
pub const TOOL_DISPATCH_NEUTRAL_CONTRACT_IDENTITY: &str = "animus.tool-dispatch-plan.v1";

/// Pure tool dispatch plan status.
#[allow(path_segment_repetition)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ToolDispatchPlanStatus {
    /// Tool executor may be called by the imperative shell.
    Admitted,
    /// Tool executor must not be called.
    Denied,
}

/// First fail-closed reason found by tool dispatch planning.
#[allow(path_segment_repetition)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ToolDispatchPlanIssue {
    /// No issue was found.
    None,
    /// Request shape was not the current structured tool-call contract.
    InvalidRequestShape,
    /// No active catalog/manifest definition matched the requested tool.
    UnknownTool,
    /// Definition exists but is inactive.
    InactiveTool,
    /// Definition exists but is untrusted.
    UntrustedTool,
    /// Request schema identity did not match the admitted definition.
    SchemaIdentityMismatch,
    /// No executor was injected for the known tool.
    MissingExecutor,
    /// The injected executor service exists but is unavailable.
    UnavailableExecutor,
    /// Required exact capability was absent or denied.
    DeniedCapability,
    /// Approval policy denied the call.
    ApprovalDenied,
    /// Cancellation gate stopped dispatch.
    Cancelled,
    /// Timeout gate stopped dispatch.
    TimedOut,
    /// Output policy label was unsupported.
    MalformedOutputPolicy,
    /// Output reference policy was unsafe.
    UnsafeReference,
    /// Output would be unbounded.
    UnboundedOutput,
    /// Declared mutation targets conflicted.
    MutationConflict,
    /// Fallback was requested but fallback policy denies rerouting.
    FallbackDenied,
}

impl ToolDispatchPlanIssue {
    /// Returns the stable issue label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => ISSUE_NONE,
            Self::InvalidRequestShape => ISSUE_INVALID_REQUEST_SHAPE,
            Self::UnknownTool => ISSUE_UNKNOWN_TOOL,
            Self::InactiveTool => ISSUE_INACTIVE_TOOL,
            Self::UntrustedTool => ISSUE_UNTRUSTED_TOOL,
            Self::SchemaIdentityMismatch => ISSUE_SCHEMA_IDENTITY_MISMATCH,
            Self::MissingExecutor => ISSUE_MISSING_EXECUTOR,
            Self::UnavailableExecutor => ISSUE_UNAVAILABLE_EXECUTOR,
            Self::DeniedCapability => ISSUE_DENIED_CAPABILITY,
            Self::ApprovalDenied => ISSUE_APPROVAL_DENIED,
            Self::Cancelled => ISSUE_CANCELLED,
            Self::TimedOut => ISSUE_TIMED_OUT,
            Self::MalformedOutputPolicy => ISSUE_MALFORMED_OUTPUT_POLICY,
            Self::UnsafeReference => ISSUE_UNSAFE_REFERENCE,
            Self::UnboundedOutput => ISSUE_UNBOUNDED_OUTPUT,
            Self::MutationConflict => ISSUE_MUTATION_CONFLICT,
            Self::FallbackDenied => ISSUE_FALLBACK_DENIED,
        }
    }
}

/// Cancellation posture before tool dispatch.
#[allow(path_segment_repetition)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ToolDispatchCancellationPosture {
    /// No cancellation has been requested.
    Continue,
    /// Dispatch is cancelled.
    Cancelled,
}

/// Timeout posture before tool dispatch.
#[allow(path_segment_repetition)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ToolDispatchTimeoutPosture {
    /// Dispatch is within its caller-owned deadline.
    WithinDeadline,
    /// Dispatch exceeded its caller-owned deadline.
    TimedOut,
}

/// Output boundary posture before tool dispatch.
#[allow(path_segment_repetition)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ToolDispatchOutputBoundary {
    /// Output is bounded and safe for runtime-visible DTOs.
    Bounded,
    /// Output reference was unsafe.
    UnsafeReference,
    /// Output is unbounded.
    Unbounded,
}

/// Fallback behavior for tool dispatch planning.
#[allow(path_segment_repetition)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ToolDispatchFallbackPolicy {
    /// Do not fall back to shell, network, provider, plugin, or discovery paths.
    Deny,
}

/// Borrowed tool dispatch policy inputs before pure admission.
#[allow(clippy::struct_excessive_bools, path_segment_repetition)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ToolDispatchPolicyDraft<'a> {
    /// Request shape label.
    pub request_shape: &'a str,
    /// Output boundary label.
    pub output_boundary: &'a str,
    /// Cancellation posture label.
    pub cancellation_posture: &'a str,
    /// Timeout posture label.
    pub timeout_posture: &'a str,
    /// Fallback policy label.
    pub fallback_policy: &'a str,
    /// Whether dispatch requested fallback to another executor/provider.
    pub fallback_requested: bool,
    /// Whether capability policy admitted all required capabilities.
    pub capability_allowed: bool,
    /// Whether required approval has been granted.
    pub approval_granted: bool,
    /// Whether a host-owned mutation coordinator detected a conflict.
    pub mutation_conflict: bool,
    /// Whether the injected executor service is available.
    pub executor_available: bool,
}

impl ToolDispatchPolicyDraft<'_> {
    /// Builds default current-shape tool dispatch policy inputs.
    #[must_use]
    pub const fn current_defaults() -> Self {
        Self {
            request_shape: TOOL_REQUEST_SHAPE_CURRENT,
            output_boundary: TOOL_OUTPUT_BOUNDARY_BOUNDED,
            cancellation_posture: TOOL_CANCELLATION_CONTINUE,
            timeout_posture: TOOL_TIMEOUT_WITHIN_DEADLINE,
            fallback_policy: TOOL_FALLBACK_DENY,
            fallback_requested: FALLBACK_NOT_REQUESTED,
            capability_allowed: true,
            approval_granted: true,
            mutation_conflict: false,
            executor_available: true,
        }
    }
}

/// Borrowed pure planning request for one tool dispatch.
#[allow(path_segment_repetition)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ToolDispatchPlanRequest<'a> {
    /// Validated current-shape tool request selected by the loop fabric.
    pub request: &'a animus_adapters::ToolRequest,
    /// Active trusted catalog or manifest definitions.
    pub definitions: &'a [animus_adapters::tool::Definition],
    /// Host-supplied capability registry.
    pub registry: &'a crate::RuntimeCapabilityRegistry,
    /// Whether an executor was injected for tool calls.
    pub executor_present: bool,
    /// Dispatch policy inputs.
    pub policy: ToolDispatchPolicyDraft<'a>,
}

/// Pure tool dispatch plan safe to inspect before executor effects.
#[allow(path_segment_repetition)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToolDispatchPlan {
    status: ToolDispatchPlanStatus,
    issue: ToolDispatchPlanIssue,
    tool_name: String,
    definition_source_identity: Option<String>,
    request_contract_identity: &'static str,
    dispatch_contract_identity: &'static str,
    schema_identity: String,
    required_capability_labels: Vec<String>,
    mutation_targets: Vec<String>,
    approval_posture: animus_adapters::tool::ApprovalPosture,
    output_policy: animus_adapters::tool::OutputPolicy,
    cancellation_posture: Option<ToolDispatchCancellationPosture>,
    timeout_posture: Option<ToolDispatchTimeoutPosture>,
    output_boundary: Option<ToolDispatchOutputBoundary>,
    fallback_policy: Option<ToolDispatchFallbackPolicy>,
}

impl ToolDispatchPlan {
    /// Returns the plan status.
    #[must_use]
    pub const fn status(&self) -> ToolDispatchPlanStatus {
        self.status
    }

    /// Returns the first issue found by planning.
    #[must_use]
    pub const fn issue(&self) -> ToolDispatchPlanIssue {
        self.issue
    }

    /// Returns true only when tool dispatch may proceed.
    #[must_use]
    pub const fn is_admitted(&self) -> bool {
        matches!(self.status, ToolDispatchPlanStatus::Admitted)
    }

    /// Returns the requested tool name.
    #[must_use]
    pub fn tool_name(&self) -> &str {
        self.tool_name.as_str()
    }

    /// Returns the trusted definition source identity, when lookup succeeded.
    #[must_use]
    pub fn definition_source_identity(&self) -> Option<&str> {
        self.definition_source_identity.as_deref()
    }

    /// Returns the tool request contract identity.
    #[must_use]
    pub const fn request_contract_identity(&self) -> &'static str {
        self.request_contract_identity
    }

    /// Returns the tool dispatch plan contract identity.
    #[must_use]
    pub const fn dispatch_contract_identity(&self) -> &'static str {
        self.dispatch_contract_identity
    }

    /// Returns the current parameter schema identity.
    #[must_use]
    pub fn schema_identity(&self) -> &str {
        self.schema_identity.as_str()
    }

    /// Returns exact capability labels required by this plan.
    #[must_use]
    pub fn required_capability_labels(&self) -> &[String] {
        self.required_capability_labels.as_slice()
    }

    /// Returns declared mutation targets.
    #[must_use]
    pub fn mutation_targets(&self) -> &[String] {
        self.mutation_targets.as_slice()
    }

    /// Returns approval posture.
    #[must_use]
    pub const fn approval_posture(&self) -> animus_adapters::tool::ApprovalPosture {
        self.approval_posture
    }

    /// Returns output policy.
    #[must_use]
    pub const fn output_policy(&self) -> animus_adapters::tool::OutputPolicy {
        self.output_policy
    }

    /// Returns parsed cancellation posture, if supported.
    #[must_use]
    pub const fn cancellation_posture(&self) -> Option<ToolDispatchCancellationPosture> {
        self.cancellation_posture
    }

    /// Returns parsed timeout posture, if supported.
    #[must_use]
    pub const fn timeout_posture(&self) -> Option<ToolDispatchTimeoutPosture> {
        self.timeout_posture
    }

    /// Returns parsed output boundary, if supported.
    #[must_use]
    pub const fn output_boundary(&self) -> Option<ToolDispatchOutputBoundary> {
        self.output_boundary
    }

    /// Returns parsed fallback policy, if supported.
    #[must_use]
    pub const fn fallback_policy(&self) -> Option<ToolDispatchFallbackPolicy> {
        self.fallback_policy
    }
}

/// Builds a pure tool dispatch plan without executor, shell, provider, plugin,
/// filesystem, credential, network, or fallback effects.
#[allow(path_segment_repetition)]
#[must_use]
pub fn plan_tool_dispatch(request: ToolDispatchPlanRequest<'_>) -> ToolDispatchPlan {
    // r[impl agent_harness_runtime.tool_calling_breaking.dispatch_plan]
    // r[impl agent_harness_runtime.tool_calling_breaking.catalog_admission]
    let parsed = ParsedPolicy::parse(request.policy);
    let definition = find_definition(request.definitions, request.request.name());
    match (select_denial_issue(request, definition, parsed), definition) {
        (Some(issue), current_definition) => {
            denied_plan(request, current_definition, parsed, issue)
        }
        (None, Some(current_definition)) => admitted_plan(request, current_definition, parsed),
        (None, None) => denied_plan(request, None, parsed, ToolDispatchPlanIssue::UnknownTool),
    }
}

fn find_definition<'a>(
    definitions: &'a [animus_adapters::tool::Definition],
    tool_name: &str,
) -> Option<&'a animus_adapters::tool::Definition> {
    definitions
        .iter()
        .find(|definition| definition.name() == tool_name)
}

fn select_denial_issue(
    request: ToolDispatchPlanRequest<'_>,
    definition: Option<&animus_adapters::tool::Definition>,
    parsed: ParsedPolicy,
) -> Option<ToolDispatchPlanIssue> {
    let Some(definition) = definition else {
        return Some(ToolDispatchPlanIssue::UnknownTool);
    };
    definition_issue(definition)
        .or_else(|| request_shape_issue(request.policy))
        .or_else(|| schema_issue(request.request, definition))
        .or_else(|| executor_issue(request))
        .or_else(|| capability_issue(request, definition))
        .or_else(|| approval_issue(request))
        .or_else(|| parsed_policy_issue(parsed))
        .or_else(|| output_boundary_issue(parsed))
        .or_else(|| mutation_issue(request.policy))
        .or_else(|| fallback_issue(request.policy, parsed))
}

fn definition_issue(
    definition: &animus_adapters::tool::Definition,
) -> Option<ToolDispatchPlanIssue> {
    match definition.status() {
        animus_adapters::tool::DefinitionStatus::Active => None,
        animus_adapters::tool::DefinitionStatus::Inactive => {
            Some(ToolDispatchPlanIssue::InactiveTool)
        }
        animus_adapters::tool::DefinitionStatus::Untrusted => {
            Some(ToolDispatchPlanIssue::UntrustedTool)
        }
    }
}

fn request_shape_issue(policy: ToolDispatchPolicyDraft<'_>) -> Option<ToolDispatchPlanIssue> {
    (policy.request_shape.trim() != TOOL_REQUEST_SHAPE_CURRENT)
        .then_some(ToolDispatchPlanIssue::InvalidRequestShape)
}

fn schema_issue(
    request: &animus_adapters::ToolRequest,
    definition: &animus_adapters::tool::Definition,
) -> Option<ToolDispatchPlanIssue> {
    (request.schema_identity() != definition.parameter_schema_identity())
        .then_some(ToolDispatchPlanIssue::SchemaIdentityMismatch)
}

fn executor_issue(request: ToolDispatchPlanRequest<'_>) -> Option<ToolDispatchPlanIssue> {
    if !request.executor_present {
        return Some(ToolDispatchPlanIssue::MissingExecutor);
    }
    (!request.policy.executor_available).then_some(ToolDispatchPlanIssue::UnavailableExecutor)
}

fn capability_issue(
    request: ToolDispatchPlanRequest<'_>,
    definition: &animus_adapters::tool::Definition,
) -> Option<ToolDispatchPlanIssue> {
    if !request.policy.capability_allowed {
        return Some(ToolDispatchPlanIssue::DeniedCapability);
    }
    if !capabilities_match_definition(request.request, definition) {
        return Some(ToolDispatchPlanIssue::DeniedCapability);
    }
    for label in request.request.required_capability_labels() {
        if label == crate::DEFAULT_TOOL_CAPABILITY_LABEL {
            return Some(ToolDispatchPlanIssue::DeniedCapability);
        }
        if !resolves_capability(request.registry, label) {
            return Some(ToolDispatchPlanIssue::DeniedCapability);
        }
    }
    None
}

fn capabilities_match_definition(
    request: &animus_adapters::ToolRequest,
    definition: &animus_adapters::tool::Definition,
) -> bool {
    let mut request_labels = string_vec(request.required_capability_labels());
    let mut definition_labels = string_vec(definition.capability_labels());
    request_labels.sort();
    definition_labels.sort();
    request_labels == definition_labels
}

fn resolves_capability(registry: &crate::RuntimeCapabilityRegistry, label: &str) -> bool {
    let required =
        crate::RuntimeCapabilityDescriptor::from_draft(crate::RuntimeCapabilityDescriptorDraft {
            label,
            service_kind: crate::RuntimeCapabilityServiceKind::Tool.as_str(),
            scope: None,
        });
    let Ok(required) = required else {
        return false;
    };
    matches!(
        registry.resolve_exact(&required),
        crate::RuntimeCapabilityResolution::Resolved { .. }
    )
}

fn approval_issue(request: ToolDispatchPlanRequest<'_>) -> Option<ToolDispatchPlanIssue> {
    match request.request.approval_posture() {
        animus_adapters::tool::ApprovalPosture::PreApproved => None,
        animus_adapters::tool::ApprovalPosture::RequiresApproval => {
            (!request.policy.approval_granted).then_some(ToolDispatchPlanIssue::ApprovalDenied)
        }
        animus_adapters::tool::ApprovalPosture::Denied => {
            Some(ToolDispatchPlanIssue::ApprovalDenied)
        }
    }
}

fn parsed_policy_issue(parsed: ParsedPolicy) -> Option<ToolDispatchPlanIssue> {
    if parsed.output_boundary.is_none() {
        return Some(ToolDispatchPlanIssue::MalformedOutputPolicy);
    }
    if matches!(
        parsed.cancellation_posture,
        Some(ToolDispatchCancellationPosture::Cancelled)
    ) {
        return Some(ToolDispatchPlanIssue::Cancelled);
    }
    if matches!(
        parsed.timeout_posture,
        Some(ToolDispatchTimeoutPosture::TimedOut)
    ) {
        return Some(ToolDispatchPlanIssue::TimedOut);
    }
    None
}

fn output_boundary_issue(parsed: ParsedPolicy) -> Option<ToolDispatchPlanIssue> {
    match parsed.output_boundary {
        Some(ToolDispatchOutputBoundary::Bounded) => None,
        Some(ToolDispatchOutputBoundary::UnsafeReference) => {
            Some(ToolDispatchPlanIssue::UnsafeReference)
        }
        Some(ToolDispatchOutputBoundary::Unbounded) => Some(ToolDispatchPlanIssue::UnboundedOutput),
        None => Some(ToolDispatchPlanIssue::MalformedOutputPolicy),
    }
}

fn mutation_issue(policy: ToolDispatchPolicyDraft<'_>) -> Option<ToolDispatchPlanIssue> {
    policy
        .mutation_conflict
        .then_some(ToolDispatchPlanIssue::MutationConflict)
}

fn fallback_issue(
    policy: ToolDispatchPolicyDraft<'_>,
    parsed: ParsedPolicy,
) -> Option<ToolDispatchPlanIssue> {
    (policy.fallback_requested == FALLBACK_REQUESTED
        && matches!(
            parsed.fallback_policy,
            Some(ToolDispatchFallbackPolicy::Deny)
        ))
    .then_some(ToolDispatchPlanIssue::FallbackDenied)
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct ParsedPolicy {
    cancellation_posture: Option<ToolDispatchCancellationPosture>,
    timeout_posture: Option<ToolDispatchTimeoutPosture>,
    output_boundary: Option<ToolDispatchOutputBoundary>,
    fallback_policy: Option<ToolDispatchFallbackPolicy>,
}

impl ParsedPolicy {
    fn parse(policy: ToolDispatchPolicyDraft<'_>) -> Self {
        Self {
            cancellation_posture: parse_cancellation_posture(policy.cancellation_posture),
            timeout_posture: parse_timeout_posture(policy.timeout_posture),
            output_boundary: parse_output_boundary(policy.output_boundary),
            fallback_policy: parse_fallback_policy(policy.fallback_policy),
        }
    }
}

fn parse_cancellation_posture(value: &str) -> Option<ToolDispatchCancellationPosture> {
    match value.trim() {
        TOOL_CANCELLATION_CONTINUE => Some(ToolDispatchCancellationPosture::Continue),
        TOOL_CANCELLATION_CANCELLED => Some(ToolDispatchCancellationPosture::Cancelled),
        _other => None,
    }
}

fn parse_timeout_posture(value: &str) -> Option<ToolDispatchTimeoutPosture> {
    match value.trim() {
        TOOL_TIMEOUT_WITHIN_DEADLINE => Some(ToolDispatchTimeoutPosture::WithinDeadline),
        TOOL_TIMEOUT_TIMED_OUT => Some(ToolDispatchTimeoutPosture::TimedOut),
        _other => None,
    }
}

fn parse_output_boundary(value: &str) -> Option<ToolDispatchOutputBoundary> {
    match value.trim() {
        TOOL_OUTPUT_BOUNDARY_BOUNDED => Some(ToolDispatchOutputBoundary::Bounded),
        TOOL_OUTPUT_BOUNDARY_UNSAFE_REFERENCE => Some(ToolDispatchOutputBoundary::UnsafeReference),
        TOOL_OUTPUT_BOUNDARY_UNBOUNDED => Some(ToolDispatchOutputBoundary::Unbounded),
        _other => None,
    }
}

fn parse_fallback_policy(value: &str) -> Option<ToolDispatchFallbackPolicy> {
    match value.trim() {
        TOOL_FALLBACK_DENY => Some(ToolDispatchFallbackPolicy::Deny),
        _other => None,
    }
}

fn admitted_plan(
    request: ToolDispatchPlanRequest<'_>,
    definition: &animus_adapters::tool::Definition,
    parsed: ParsedPolicy,
) -> ToolDispatchPlan {
    build_plan(PlanBuildInput {
        request,
        definition: Some(definition),
        parsed,
        status: ToolDispatchPlanStatus::Admitted,
        issue: ToolDispatchPlanIssue::None,
    })
}

fn denied_plan(
    request: ToolDispatchPlanRequest<'_>,
    definition: Option<&animus_adapters::tool::Definition>,
    parsed: ParsedPolicy,
    issue: ToolDispatchPlanIssue,
) -> ToolDispatchPlan {
    build_plan(PlanBuildInput {
        request,
        definition,
        parsed,
        status: ToolDispatchPlanStatus::Denied,
        issue,
    })
}

#[derive(Clone, Copy, Debug)]
struct PlanBuildInput<'a> {
    request: ToolDispatchPlanRequest<'a>,
    definition: Option<&'a animus_adapters::tool::Definition>,
    parsed: ParsedPolicy,
    status: ToolDispatchPlanStatus,
    issue: ToolDispatchPlanIssue,
}

fn build_plan(input: PlanBuildInput<'_>) -> ToolDispatchPlan {
    ToolDispatchPlan {
        status: input.status,
        issue: input.issue,
        tool_name: String::from(input.request.request.name()),
        definition_source_identity: input
            .definition
            .map(|definition| String::from(definition.source_identity())),
        request_contract_identity: animus_adapters::TOOL_REQUEST_CONTRACT_IDENTITY,
        dispatch_contract_identity: TOOL_DISPATCH_NEUTRAL_CONTRACT_IDENTITY,
        schema_identity: String::from(input.request.request.schema_identity()),
        required_capability_labels: string_vec(input.request.request.required_capability_labels()),
        mutation_targets: string_vec(input.request.request.mutation_targets()),
        approval_posture: input.request.request.approval_posture(),
        output_policy: input.request.request.output_policy(),
        cancellation_posture: input.parsed.cancellation_posture,
        timeout_posture: input.parsed.timeout_posture,
        output_boundary: input.parsed.output_boundary,
        fallback_policy: input.parsed.fallback_policy,
    }
}

fn string_vec(values: &[String]) -> Vec<String> {
    values.to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOOL_NAME: &str = "search";
    const OTHER_TOOL_NAME: &str = "write_file";
    const LABEL: &str = "Search";
    const DESCRIPTION: &str = "Search indexed project context";
    const SCHEMA_ID: &str = "schema.search.v1";
    const OTHER_SCHEMA_ID: &str = "schema.other.v1";
    const PROMPT_SNIPPET: &str = "Use search for indexed project facts.";
    const PROMPT_GUIDELINE: &str = "Do not infer missing search results.";
    const SOURCE_IDENTITY: &str = "host.registry";
    const PROVENANCE: &str = "builtin";
    const CAPABILITY_READ: &str = "tool.read";
    const RESULT_SHAPE: &str = "text-summary";
    const ARGUMENT_NAME: &str = "query";
    const ARGUMENT_VALUE: &str = "adapter contracts";
    const CORRELATION_ID: &str = "tool-call-1";
    const VISIBLE_BUDGET: u32 = 10;
    const ARGUMENT_COUNT: usize = 1;
    const REGISTRY_DESCRIPTOR_COUNT: usize = 1;
    const POLICY_DENIAL_COUNT: usize = 8;
    const BAD_POLICY: &str = "raw-output";

    #[test]
    fn admits_current_catalog_backed_call() {
        // r[verify agent_harness_runtime.tool_calling_breaking.dispatch_plan]
        // r[verify agent_harness_runtime.tool_calling_breaking.catalog_admission]
        let request = request_for(
            CAPABILITY_READ,
            animus_adapters::tool::ApprovalPosture::PreApproved,
        );
        let definition = definition(
            animus_adapters::tool::DefinitionStatus::Active,
            CAPABILITY_READ,
        );
        let registry = registry(CAPABILITY_READ);
        let plan = plan_tool_dispatch(ToolDispatchPlanRequest {
            request: &request,
            definitions: &[definition],
            registry: &registry,
            executor_present: true,
            policy: ToolDispatchPolicyDraft::current_defaults(),
        });

        assert!(plan.is_admitted());
        assert_eq!(plan.issue(), ToolDispatchPlanIssue::None);
        assert_eq!(plan.tool_name(), TOOL_NAME);
        assert_eq!(plan.schema_identity(), SCHEMA_ID);
        assert_eq!(plan.required_capability_labels()[0], CAPABILITY_READ);
        assert_eq!(plan.definition_source_identity(), Some(SOURCE_IDENTITY));
        assert_eq!(
            plan.request_contract_identity(),
            animus_adapters::TOOL_REQUEST_CONTRACT_IDENTITY
        );
        assert_eq!(
            plan.dispatch_contract_identity(),
            TOOL_DISPATCH_NEUTRAL_CONTRACT_IDENTITY
        );
    }

    #[test]
    fn denies_catalog_identity_and_schema_failures() {
        // r[verify agent_harness_runtime.tool_calling_breaking.catalog_admission]
        let request = request_for(
            CAPABILITY_READ,
            animus_adapters::tool::ApprovalPosture::PreApproved,
        );
        let inactive = definition(
            animus_adapters::tool::DefinitionStatus::Inactive,
            CAPABILITY_READ,
        );
        let untrusted = definition(
            animus_adapters::tool::DefinitionStatus::Untrusted,
            CAPABILITY_READ,
        );
        let other_schema = other_schema_definition(CAPABILITY_READ);
        let registry = registry(CAPABILITY_READ);

        assert_eq!(request.schema_identity(), SCHEMA_ID);
        assert_eq!(registry.descriptors().len(), REGISTRY_DESCRIPTOR_COUNT);
        assert_eq!(other_schema.parameter_schema_identity(), OTHER_SCHEMA_ID);
        assert_denied(
            ToolDispatchPlanRequest {
                request: &request,
                definitions: &[],
                registry: &registry,
                executor_present: true,
                policy: ToolDispatchPolicyDraft::current_defaults(),
            },
            ToolDispatchPlanIssue::UnknownTool,
        );
        assert_denied(
            ToolDispatchPlanRequest {
                request: &request,
                definitions: &[inactive],
                registry: &registry,
                executor_present: true,
                policy: ToolDispatchPolicyDraft::current_defaults(),
            },
            ToolDispatchPlanIssue::InactiveTool,
        );
        assert_denied(
            ToolDispatchPlanRequest {
                request: &request,
                definitions: &[untrusted],
                registry: &registry,
                executor_present: true,
                policy: ToolDispatchPolicyDraft::current_defaults(),
            },
            ToolDispatchPlanIssue::UntrustedTool,
        );
        assert_denied(
            ToolDispatchPlanRequest {
                request: &request,
                definitions: &[other_schema],
                registry: &registry,
                executor_present: true,
                policy: ToolDispatchPolicyDraft::current_defaults(),
            },
            ToolDispatchPlanIssue::SchemaIdentityMismatch,
        );
    }

    #[test]
    fn denies_executor_availability_failures() {
        let request = request_for(
            CAPABILITY_READ,
            animus_adapters::tool::ApprovalPosture::PreApproved,
        );
        let active = definition(
            animus_adapters::tool::DefinitionStatus::Active,
            CAPABILITY_READ,
        );
        let registry = registry(CAPABILITY_READ);

        assert_eq!(request.name(), TOOL_NAME);
        assert_eq!(registry.descriptors().len(), REGISTRY_DESCRIPTOR_COUNT);
        assert_eq!(active.capability_labels()[0], CAPABILITY_READ);
        assert_denied(
            ToolDispatchPlanRequest {
                request: &request,
                definitions: core::slice::from_ref(&active),
                registry: &registry,
                executor_present: false,
                policy: ToolDispatchPolicyDraft::current_defaults(),
            },
            ToolDispatchPlanIssue::MissingExecutor,
        );
        assert_denied(
            ToolDispatchPlanRequest {
                request: &request,
                definitions: &[active],
                registry: &registry,
                executor_present: true,
                policy: unavailable_executor(),
            },
            ToolDispatchPlanIssue::UnavailableExecutor,
        );
    }

    #[test]
    fn denies_capability_resolution_failures() {
        let request = request_for(
            CAPABILITY_READ,
            animus_adapters::tool::ApprovalPosture::PreApproved,
        );
        let active = definition(
            animus_adapters::tool::DefinitionStatus::Active,
            CAPABILITY_READ,
        );
        let wrong_capability = definition(
            animus_adapters::tool::DefinitionStatus::Active,
            crate::DEFAULT_TOOL_CAPABILITY_LABEL,
        );
        let registry = registry(CAPABILITY_READ);
        let empty_registry = crate::RuntimeCapabilityRegistry::empty();

        assert_eq!(
            wrong_capability.capability_labels()[0],
            crate::DEFAULT_TOOL_CAPABILITY_LABEL
        );
        assert!(empty_registry.descriptors().is_empty());
        assert_denied(
            ToolDispatchPlanRequest {
                request: &request,
                definitions: &[wrong_capability],
                registry: &registry,
                executor_present: true,
                policy: ToolDispatchPolicyDraft::current_defaults(),
            },
            ToolDispatchPlanIssue::DeniedCapability,
        );
        assert_denied(
            ToolDispatchPlanRequest {
                request: &request,
                definitions: &[active],
                registry: &empty_registry,
                executor_present: true,
                policy: ToolDispatchPolicyDraft::current_defaults(),
            },
            ToolDispatchPlanIssue::DeniedCapability,
        );
    }

    #[test]
    fn denies_approval_failure() {
        let approval_request = request_for(
            CAPABILITY_READ,
            animus_adapters::tool::ApprovalPosture::RequiresApproval,
        );
        let active = definition(
            animus_adapters::tool::DefinitionStatus::Active,
            CAPABILITY_READ,
        );
        let registry = registry(CAPABILITY_READ);

        assert_eq!(
            active.status(),
            animus_adapters::tool::DefinitionStatus::Active
        );
        assert_eq!(registry.descriptors().len(), REGISTRY_DESCRIPTOR_COUNT);
        assert_eq!(
            approval_request.approval_posture(),
            animus_adapters::tool::ApprovalPosture::RequiresApproval
        );
        assert_denied(
            ToolDispatchPlanRequest {
                request: &approval_request,
                definitions: &[active],
                registry: &registry,
                executor_present: true,
                policy: approval_denied(),
            },
            ToolDispatchPlanIssue::ApprovalDenied,
        );
    }

    #[test]
    fn denies_default_capability_and_policy_failures() {
        // r[verify agent_harness_runtime.tool_calling_breaking.catalog_admission.exact_capability]
        let default_request = request_for(
            crate::DEFAULT_TOOL_CAPABILITY_LABEL,
            animus_adapters::tool::ApprovalPosture::PreApproved,
        );
        let default_definition = definition(
            animus_adapters::tool::DefinitionStatus::Active,
            crate::DEFAULT_TOOL_CAPABILITY_LABEL,
        );
        let default_registry = registry(crate::DEFAULT_TOOL_CAPABILITY_LABEL);
        assert_eq!(
            default_registry.descriptors().len(),
            REGISTRY_DESCRIPTOR_COUNT
        );
        assert_eq!(
            default_request.required_capability_labels()[0],
            crate::DEFAULT_TOOL_CAPABILITY_LABEL
        );
        assert_denied(
            ToolDispatchPlanRequest {
                request: &default_request,
                definitions: &[default_definition],
                registry: &default_registry,
                executor_present: true,
                policy: ToolDispatchPolicyDraft::current_defaults(),
            },
            ToolDispatchPlanIssue::DeniedCapability,
        );

        let request = request_for(
            CAPABILITY_READ,
            animus_adapters::tool::ApprovalPosture::PreApproved,
        );
        let definition = definition(
            animus_adapters::tool::DefinitionStatus::Active,
            CAPABILITY_READ,
        );
        let registry = registry(CAPABILITY_READ);
        assert_eq!(request.name(), TOOL_NAME);
        assert_eq!(registry.descriptors().len(), REGISTRY_DESCRIPTOR_COUNT);
        let policy_denials = [
            (bad_shape(), ToolDispatchPlanIssue::InvalidRequestShape),
            (
                bad_output_policy(),
                ToolDispatchPlanIssue::MalformedOutputPolicy,
            ),
            (unsafe_reference(), ToolDispatchPlanIssue::UnsafeReference),
            (unbounded_output(), ToolDispatchPlanIssue::UnboundedOutput),
            (cancelled(), ToolDispatchPlanIssue::Cancelled),
            (timed_out(), ToolDispatchPlanIssue::TimedOut),
            (mutation_conflict(), ToolDispatchPlanIssue::MutationConflict),
            (fallback_requested(), ToolDispatchPlanIssue::FallbackDenied),
        ];
        assert_eq!(policy_denials.len(), POLICY_DENIAL_COUNT);
        for (policy, issue) in policy_denials {
            assert_denied(
                ToolDispatchPlanRequest {
                    request: &request,
                    definitions: core::slice::from_ref(&definition),
                    registry: &registry,
                    executor_present: true,
                    policy,
                },
                issue,
            );
        }
    }

    fn assert_denied(
        plan_request: ToolDispatchPlanRequest<'_>,
        expected_issue: ToolDispatchPlanIssue,
    ) {
        let plan = plan_tool_dispatch(plan_request);

        assert_eq!(plan.status(), ToolDispatchPlanStatus::Denied);
        assert_eq!(plan.issue(), expected_issue);
        assert!(!plan.is_admitted());
    }

    fn request_for(
        capability_label: &str,
        approval_posture: animus_adapters::tool::ApprovalPosture,
    ) -> animus_adapters::ToolRequest {
        let fields = [animus_adapters::tool::ArgumentFieldDraft {
            name: Some(ARGUMENT_NAME),
            value: Some(ARGUMENT_VALUE),
        }];
        let arguments = animus_adapters::tool::PreparedArguments::prepare(
            &animus_adapters::tool::ArgumentPreparationRequest {
                current_schema_identity: SCHEMA_ID,
                allowed_field_names: &[ARGUMENT_NAME],
                compatibility_rules: &[],
                fields: &fields,
            },
        )
        .expect("arguments should validate");
        assert_eq!(arguments.schema_identity(), SCHEMA_ID);
        assert_eq!(arguments.fields().len(), ARGUMENT_COUNT);
        let request =
            animus_adapters::ToolRequest::from_draft(&animus_adapters::tool::RequestDraft {
                name: Some(TOOL_NAME),
                arguments: Some(&arguments),
                correlation_id: Some(CORRELATION_ID),
                output_policy: animus_adapters::tool::OutputPolicy {
                    visible_char_budget: VISIBLE_BUDGET,
                },
                mutation_targets: &[OTHER_TOOL_NAME],
                required_capability_labels: &[capability_label],
                approval_posture: approval_posture.as_str(),
            })
            .expect("request should validate");
        assert_eq!(request.name(), TOOL_NAME);
        assert_eq!(request.correlation_id(), CORRELATION_ID);
        request
    }

    #[derive(Clone, Copy)]
    struct DefinitionFixture<'a> {
        status: animus_adapters::tool::DefinitionStatus,
        capability_label: &'a str,
        schema_identity: &'a str,
    }

    fn definition(
        status: animus_adapters::tool::DefinitionStatus,
        capability_label: &str,
    ) -> animus_adapters::tool::Definition {
        definition_from_fixture(DefinitionFixture {
            status,
            capability_label,
            schema_identity: SCHEMA_ID,
        })
    }

    fn other_schema_definition(capability_label: &str) -> animus_adapters::tool::Definition {
        definition_from_fixture(DefinitionFixture {
            status: animus_adapters::tool::DefinitionStatus::Active,
            capability_label,
            schema_identity: OTHER_SCHEMA_ID,
        })
    }

    fn definition_from_fixture(
        fixture: DefinitionFixture<'_>,
    ) -> animus_adapters::tool::Definition {
        animus_adapters::tool::Definition::validate(&animus_adapters::tool::DefinitionDraft {
            name: Some(TOOL_NAME),
            label: Some(LABEL),
            description: Some(DESCRIPTION),
            parameter_schema_identity: Some(fixture.schema_identity),
            prompt_snippet: Some(PROMPT_SNIPPET),
            prompt_guidelines: &[PROMPT_GUIDELINE],
            source_identity: Some(SOURCE_IDENTITY),
            provenance: Some(PROVENANCE),
            capability_labels: &[fixture.capability_label],
            result_metadata_shape: Some(RESULT_SHAPE),
            status: fixture.status,
        })
        .expect("definition should validate")
    }

    fn registry(label: &str) -> crate::RuntimeCapabilityRegistry {
        let registry = crate::RuntimeCapabilityRegistry::from_drafts(&[
            crate::RuntimeCapabilityDescriptorDraft {
                label,
                service_kind: crate::RuntimeCapabilityServiceKind::Tool.as_str(),
                scope: None,
            },
        ])
        .expect("registry should validate");
        assert_eq!(registry.descriptors().len(), REGISTRY_DESCRIPTOR_COUNT);
        assert!(registry.contains_service(crate::RuntimeCapabilityServiceKind::Tool));
        registry
    }

    const fn unavailable_executor() -> ToolDispatchPolicyDraft<'static> {
        ToolDispatchPolicyDraft {
            executor_available: false,
            ..ToolDispatchPolicyDraft::current_defaults()
        }
    }

    const fn approval_denied() -> ToolDispatchPolicyDraft<'static> {
        ToolDispatchPolicyDraft {
            approval_granted: false,
            ..ToolDispatchPolicyDraft::current_defaults()
        }
    }

    const fn bad_shape() -> ToolDispatchPolicyDraft<'static> {
        ToolDispatchPolicyDraft {
            request_shape: BAD_POLICY,
            ..ToolDispatchPolicyDraft::current_defaults()
        }
    }

    const fn bad_output_policy() -> ToolDispatchPolicyDraft<'static> {
        ToolDispatchPolicyDraft {
            output_boundary: BAD_POLICY,
            ..ToolDispatchPolicyDraft::current_defaults()
        }
    }

    const fn unsafe_reference() -> ToolDispatchPolicyDraft<'static> {
        ToolDispatchPolicyDraft {
            output_boundary: TOOL_OUTPUT_BOUNDARY_UNSAFE_REFERENCE,
            ..ToolDispatchPolicyDraft::current_defaults()
        }
    }

    const fn unbounded_output() -> ToolDispatchPolicyDraft<'static> {
        ToolDispatchPolicyDraft {
            output_boundary: TOOL_OUTPUT_BOUNDARY_UNBOUNDED,
            ..ToolDispatchPolicyDraft::current_defaults()
        }
    }

    const fn cancelled() -> ToolDispatchPolicyDraft<'static> {
        ToolDispatchPolicyDraft {
            cancellation_posture: TOOL_CANCELLATION_CANCELLED,
            ..ToolDispatchPolicyDraft::current_defaults()
        }
    }

    const fn timed_out() -> ToolDispatchPolicyDraft<'static> {
        ToolDispatchPolicyDraft {
            timeout_posture: TOOL_TIMEOUT_TIMED_OUT,
            ..ToolDispatchPolicyDraft::current_defaults()
        }
    }

    const fn mutation_conflict() -> ToolDispatchPolicyDraft<'static> {
        ToolDispatchPolicyDraft {
            mutation_conflict: true,
            ..ToolDispatchPolicyDraft::current_defaults()
        }
    }

    const fn fallback_requested() -> ToolDispatchPolicyDraft<'static> {
        ToolDispatchPolicyDraft {
            fallback_requested: FALLBACK_REQUESTED,
            ..ToolDispatchPolicyDraft::current_defaults()
        }
    }
}

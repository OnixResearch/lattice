//! Coherent host-injected tool executor kit.
//!
//! The kit combines catalog metadata, pure fail-closed admission, injected
//! executor bindings, bounded output normalization, and safe receipts. Only
//! [`execute_admitted_tool`] crosses the host executor boundary; every decision
//! before that call is deterministic over explicit in-memory inputs.

const TOOL_REQUEST_SHAPE_CURRENT: &str = "current_structured_tool_call";
const TOOL_OUTPUT_BOUNDARY_BOUNDED: &str = "bounded";
const TOOL_CANCELLATION_CONTINUE: &str = "continue";
const TOOL_CANCELLATION_CANCELLED: &str = "cancelled";
const TOOL_TIMEOUT_WITHIN_DEADLINE: &str = "within_deadline";
const TOOL_TIMEOUT_TIMED_OUT: &str = "timed_out";
const TOOL_FALLBACK_DENY: &str = "deny";
const RECEIPT_HASH_DOMAIN: &str = "animus.tool-executor-kit.receipt.v1";
const REQUEST_HASH_DOMAIN: &str = "animus.tool-executor-kit.request.v1";
const REQUEST_FIELD_HASH_DOMAIN: &str = "animus.tool-executor-kit.request-field.v1";
const OUTPUT_HASH_DOMAIN: &str = "animus.tool-executor-kit.output.v1";
const HASH_SEPARATOR: &str = "|";
const RECEIPT_REDACTION_MARKER: &str = "raw-output-omitted";
const MAX_CATALOG_DEFINITIONS: usize = 32;
const MAX_ARGUMENT_NAMES: usize = 32;
const MAX_SAFE_LABEL_CHARS: usize = 128;
const BLAKE3_HEX_CHARS: usize = 64;
const EMPTY_OUTPUT_COUNT: u64 = 0;
const UNSAFE_MARKERS: &[&str] = &[
    "RAW_PROMPT:",
    "RAW_FILE_BODY:",
    "RAW_STDOUT:",
    "RAW_STDERR:",
    "PROVIDER_PAYLOAD:",
    "CREDENTIAL:",
    "BEGIN PRIVATE KEY",
    "API_KEY=",
    "AWS_SECRET_ACCESS_KEY",
    "sk-",
    "/home/",
    "/Users/",
    ":\\",
];

/// Stable contract identity for the coherent tool executor kit.
pub const TOOL_EXECUTOR_KIT_CONTRACT_IDENTITY: &str = "animus.tool-executor-kit.v1";

/// Declared mutation posture for one catalog tool definition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ToolMutationPosture {
    /// The tool may not declare mutation targets.
    ReadOnly,
    /// Every mutating call must declare one or more opaque target labels.
    DeclaredTargets,
}

/// Bounded output policy attached to a tool definition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ToolExecutorOutputPolicy {
    max_visible_chars: u32,
    max_output_bytes: u64,
    allow_safe_reference: bool,
}

impl ToolExecutorOutputPolicy {
    /// Builds a bounded output policy.
    ///
    /// # Errors
    ///
    /// Returns [`ToolExecutorKitError::InvalidOutputPolicy`] when either bound
    /// is zero.
    pub const fn new(
        max_visible_chars: u32,
        max_output_bytes: u64,
        allow_safe_reference: bool,
    ) -> Result<Self, ToolExecutorKitError> {
        if max_visible_chars == 0 || max_output_bytes == 0 {
            return Err(ToolExecutorKitError::InvalidOutputPolicy);
        }
        Ok(Self {
            max_visible_chars,
            max_output_bytes,
            allow_safe_reference,
        })
    }

    /// Returns the model-visible character bound.
    #[must_use]
    pub const fn max_visible_chars(self) -> u32 {
        self.max_visible_chars
    }

    /// Returns the accepted raw output byte bound.
    #[must_use]
    pub const fn max_output_bytes(self) -> u64 {
        self.max_output_bytes
    }

    /// Returns whether a validated host-owned output reference is allowed.
    #[must_use]
    pub const fn allows_safe_reference(self) -> bool {
        self.allow_safe_reference
    }
}

/// One validated tool definition with an explicit executor binding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToolDefinition {
    metadata: animus_adapters::tool::Definition,
    executor_binding: String,
    argument_names: Vec<String>,
    mutation_posture: ToolMutationPosture,
    output_policy: ToolExecutorOutputPolicy,
    approval_posture: animus_adapters::tool::ApprovalPosture,
}

/// Borrowed and owned inputs for one executor-bound tool definition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToolDefinitionDraft<'a> {
    /// Validated implementation-free adapter metadata.
    pub metadata: animus_adapters::tool::Definition,
    /// Explicit host executor binding label.
    pub executor_binding: &'a str,
    /// Exact current-schema argument names.
    pub argument_names: &'a [&'a str],
    /// Mutation posture admitted for this tool.
    pub mutation_posture: ToolMutationPosture,
    /// Bounded output policy.
    pub output_policy: ToolExecutorOutputPolicy,
    /// Approval posture required by the catalog.
    pub approval_posture: animus_adapters::tool::ApprovalPosture,
}

impl ToolDefinition {
    /// Builds catalog metadata without executing or discovering a tool.
    ///
    /// # Errors
    ///
    /// Returns [`ToolExecutorKitError`] when the binding or argument-name set is
    /// empty, unsafe, duplicated, or exceeds the bounded catalog contract.
    pub fn from_draft(draft: ToolDefinitionDraft<'_>) -> Result<Self, ToolExecutorKitError> {
        validate_safe_label(draft.executor_binding)?;
        let argument_names = validate_argument_names(draft.argument_names)?;
        // r[impl agent_harness_runtime.tool_executor_kit.contracts]
        Ok(Self {
            metadata: draft.metadata,
            executor_binding: String::from(draft.executor_binding),
            argument_names,
            mutation_posture: draft.mutation_posture,
            output_policy: draft.output_policy,
            approval_posture: draft.approval_posture,
        })
    }

    /// Returns implementation-free tool metadata.
    #[must_use]
    pub const fn metadata(&self) -> &animus_adapters::tool::Definition {
        &self.metadata
    }

    /// Returns the host executor binding label.
    #[must_use]
    pub fn executor_binding(&self) -> &str {
        self.executor_binding.as_str()
    }

    /// Returns the exact current-schema argument names.
    #[must_use]
    pub fn argument_names(&self) -> &[String] {
        self.argument_names.as_slice()
    }

    /// Returns the declared mutation posture.
    #[must_use]
    pub const fn mutation_posture(&self) -> ToolMutationPosture {
        self.mutation_posture
    }

    /// Returns the bounded output policy.
    #[must_use]
    pub const fn output_policy(&self) -> ToolExecutorOutputPolicy {
        self.output_policy
    }

    /// Returns the catalog approval posture.
    #[must_use]
    pub const fn approval_posture(&self) -> animus_adapters::tool::ApprovalPosture {
        self.approval_posture
    }
}

/// Bounded catalog of tool metadata and executor bindings.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ToolCatalog {
    definitions: Vec<ToolDefinition>,
}

impl ToolCatalog {
    /// Builds a catalog and rejects duplicate tool identities.
    ///
    /// # Errors
    ///
    /// Returns [`ToolExecutorKitError::CatalogTooLarge`] or
    /// [`ToolExecutorKitError::DuplicateTool`] when catalog invariants fail.
    pub fn new(definitions: Vec<ToolDefinition>) -> Result<Self, ToolExecutorKitError> {
        if definitions.len() > MAX_CATALOG_DEFINITIONS {
            return Err(ToolExecutorKitError::CatalogTooLarge {
                limit: MAX_CATALOG_DEFINITIONS,
                actual: definitions.len(),
            });
        }
        for (index, definition) in definitions.iter().enumerate() {
            if definitions[..index]
                .iter()
                .any(|seen| seen.metadata().name() == definition.metadata().name())
            {
                return Err(ToolExecutorKitError::DuplicateTool);
            }
        }
        Ok(Self { definitions })
    }

    /// Returns catalog definitions in deterministic registration order.
    #[must_use]
    pub fn definitions(&self) -> &[ToolDefinition] {
        self.definitions.as_slice()
    }

    fn find(&self, tool_name: &str) -> Option<&ToolDefinition> {
        self.definitions
            .iter()
            .find(|definition| definition.metadata().name() == tool_name)
    }

    fn adapter_definitions(&self) -> Vec<animus_adapters::tool::Definition> {
        self.definitions
            .iter()
            .map(|definition| definition.metadata.clone())
            .collect()
    }
}

/// Host-reported readiness for one explicit executor binding.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ToolExecutorReadiness {
    /// The named executor may be called after admission.
    Ready,
    /// The named executor exists but cannot currently be called.
    Unavailable,
}

/// One explicit executor binding supplied by the host.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToolExecutorBinding {
    label: String,
    readiness: ToolExecutorReadiness,
}

impl ToolExecutorBinding {
    /// Builds a host-supplied executor binding descriptor.
    ///
    /// # Errors
    ///
    /// Returns [`ToolExecutorKitError::UnsafeMetadata`] for an unsafe label.
    pub fn new(
        label: &str,
        readiness: ToolExecutorReadiness,
    ) -> Result<Self, ToolExecutorKitError> {
        validate_safe_label(label)?;
        Ok(Self {
            label: String::from(label),
            readiness,
        })
    }

    /// Returns the binding label.
    #[must_use]
    pub fn label(&self) -> &str {
        self.label.as_str()
    }

    /// Returns host-reported readiness.
    #[must_use]
    pub const fn readiness(&self) -> ToolExecutorReadiness {
        self.readiness
    }
}

/// Cancellation posture supplied before executor effects.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ToolExecutorCancellation {
    /// No cancellation is pending.
    Continue,
    /// The call was cancelled before dispatch.
    Cancelled,
}

/// Caller-owned deadline posture supplied before executor effects.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ToolExecutorDeadline {
    /// The call remains within its caller-owned deadline.
    WithinDeadline,
    /// The call timed out before dispatch.
    TimedOut,
}

/// Exact capability-policy decision before executor effects.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ToolCapabilityAdmission {
    /// Every exact required capability was admitted.
    Allowed,
    /// At least one exact required capability was denied.
    Denied,
}

/// Approval decision before executor effects.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ToolApprovalAdmission {
    /// Required approval evidence is present.
    Granted,
    /// Required approval evidence is absent or denied.
    Denied,
}

/// Host-owned mutation coordination result.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ToolMutationCoordination {
    /// No declared mutation target conflicts were found.
    Clear,
    /// At least one declared mutation target conflicts.
    Conflict,
}

/// Fallback request posture before executor effects.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ToolFallbackPosture {
    /// No alternate executor route was requested.
    NotRequested,
    /// An alternate executor route was requested and must fail closed.
    Requested,
}

/// Explicit policy inputs for pure executor-kit admission.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ToolExecutorAdmission {
    /// Exact capability-policy decision.
    pub capability: ToolCapabilityAdmission,
    /// Approval decision.
    pub approval: ToolApprovalAdmission,
    /// Cancellation posture before effects.
    pub cancellation: ToolExecutorCancellation,
    /// Deadline posture before effects.
    pub deadline: ToolExecutorDeadline,
    /// Host-owned mutation coordination result.
    pub mutation: ToolMutationCoordination,
    /// Alternate-route request posture.
    pub fallback: ToolFallbackPosture,
}

impl ToolExecutorAdmission {
    /// Returns the ordinary admitted posture before catalog-specific checks.
    #[must_use]
    pub const fn admitted() -> Self {
        Self {
            capability: ToolCapabilityAdmission::Allowed,
            approval: ToolApprovalAdmission::Granted,
            cancellation: ToolExecutorCancellation::Continue,
            deadline: ToolExecutorDeadline::WithinDeadline,
            mutation: ToolMutationCoordination::Clear,
            fallback: ToolFallbackPosture::NotRequested,
        }
    }
}

/// Borrowed inputs for pure tool executor admission.
#[derive(Clone, Copy, Debug)]
pub struct ToolExecutionPlanRequest<'a> {
    /// Current-shape validated tool request.
    pub request: &'a animus_adapters::ToolRequest,
    /// Metadata-only tool catalog.
    pub catalog: &'a ToolCatalog,
    /// Exact runtime capability registry.
    pub capabilities: &'a crate::RuntimeCapabilityRegistry,
    /// Explicit host executor bindings.
    pub executors: &'a [ToolExecutorBinding],
    /// Caller- and policy-owned admission posture.
    pub admission: ToolExecutorAdmission,
}

/// Terminal status of a pure tool execution plan.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ToolExecutionPlanStatus {
    /// The named injected executor may be called.
    Admitted,
    /// No executor effect may occur.
    Denied,
}

/// Canonical fail-closed executor-kit outcome class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ToolExecutionFailureClass {
    /// No catalog definition matched the request.
    UnknownTool,
    /// The matched definition was inactive.
    InactiveDefinition,
    /// The matched definition was untrusted.
    UntrustedDefinition,
    /// The request and definition schema identities differed.
    SchemaMismatch,
    /// Current-schema argument names were malformed or incomplete.
    MalformedArguments,
    /// No matching injected executor binding was supplied.
    MissingExecutor,
    /// The matching executor binding was unavailable.
    UnavailableExecutor,
    /// Exact capability admission failed.
    DeniedCapability,
    /// Approval posture or approval evidence failed.
    ApprovalDenied,
    /// Cancellation stopped the call before effects.
    Cancelled,
    /// The caller-owned deadline expired before effects.
    TimedOut,
    /// Output policy metadata was malformed or exceeded.
    OutputPolicyDenied,
    /// Output or receipt metadata contained unsafe material.
    UnsafeOutput,
    /// Raw output exceeded its declared byte bound.
    UnboundedOutput,
    /// Mutation targets or coordination conflicted.
    MutationConflict,
    /// A fallback route was requested and denied.
    FallbackDenied,
    /// The injected executor label did not match the admitted binding.
    ExecutorBindingMismatch,
    /// The injected executor returned a typed failure.
    ExecutorFailed,
    /// A durable receipt failed safe-field validation.
    ReceiptLeak,
}

impl ToolExecutionFailureClass {
    /// Returns the stable failure-class label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UnknownTool => "unknown_tool",
            Self::InactiveDefinition => "inactive_definition",
            Self::UntrustedDefinition => "untrusted_definition",
            Self::SchemaMismatch => "schema_mismatch",
            Self::MalformedArguments => "malformed_arguments",
            Self::MissingExecutor => "missing_executor",
            Self::UnavailableExecutor => "unavailable_executor",
            Self::DeniedCapability => "denied_capability",
            Self::ApprovalDenied => "approval_denied",
            Self::Cancelled => "cancelled",
            Self::TimedOut => "timed_out",
            Self::OutputPolicyDenied => "output_policy_denied",
            Self::UnsafeOutput => "unsafe_output",
            Self::UnboundedOutput => "unbounded_output",
            Self::MutationConflict => "mutation_conflict",
            Self::FallbackDenied => "fallback_denied",
            Self::ExecutorBindingMismatch => "executor_binding_mismatch",
            Self::ExecutorFailed => "executor_failed",
            Self::ReceiptLeak => "receipt_leak",
        }
    }
}

/// Pure final plan inspected before an injected executor effect.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToolExecutionPlan {
    status: ToolExecutionPlanStatus,
    failure_class: Option<ToolExecutionFailureClass>,
    tool_name: String,
    schema_identity: String,
    executor_binding: Option<String>,
    capability_labels: Vec<String>,
    mutation_targets: Vec<String>,
    mutation_posture: Option<ToolMutationPosture>,
    output_policy: Option<ToolExecutorOutputPolicy>,
    cancellation: ToolExecutorCancellation,
    deadline: ToolExecutorDeadline,
    request_identity: String,
    dispatch_issue: crate::ToolDispatchPlanIssue,
}

impl ToolExecutionPlan {
    /// Returns the plan status.
    #[must_use]
    pub const fn status(&self) -> ToolExecutionPlanStatus {
        self.status
    }

    /// Returns the first fail-closed outcome class, if denied.
    #[must_use]
    pub const fn failure_class(&self) -> Option<ToolExecutionFailureClass> {
        self.failure_class
    }

    /// Returns whether the executor effect is admitted.
    #[must_use]
    pub const fn is_admitted(&self) -> bool {
        matches!(self.status, ToolExecutionPlanStatus::Admitted)
    }

    /// Returns the requested tool name.
    #[must_use]
    pub fn tool_name(&self) -> &str {
        self.tool_name.as_str()
    }

    /// Returns the current schema identity.
    #[must_use]
    pub fn schema_identity(&self) -> &str {
        self.schema_identity.as_str()
    }

    /// Returns the admitted executor binding, when known.
    #[must_use]
    pub fn executor_binding(&self) -> Option<&str> {
        self.executor_binding.as_deref()
    }

    /// Returns exact required capability labels.
    #[must_use]
    pub fn capability_labels(&self) -> &[String] {
        self.capability_labels.as_slice()
    }

    /// Returns declared opaque mutation targets.
    #[must_use]
    pub fn mutation_targets(&self) -> &[String] {
        self.mutation_targets.as_slice()
    }

    /// Returns the admitted mutation posture, when a definition matched.
    #[must_use]
    pub const fn mutation_posture(&self) -> Option<ToolMutationPosture> {
        self.mutation_posture
    }

    /// Returns the admitted output policy, when a definition matched.
    #[must_use]
    pub const fn output_policy(&self) -> Option<ToolExecutorOutputPolicy> {
        self.output_policy
    }

    /// Returns the exact pre-effect cancellation posture.
    #[must_use]
    pub const fn cancellation(&self) -> ToolExecutorCancellation {
        self.cancellation
    }

    /// Returns the exact caller-owned deadline posture.
    #[must_use]
    pub const fn deadline(&self) -> ToolExecutorDeadline {
        self.deadline
    }

    /// Returns the domain-separated exact request identity.
    #[must_use]
    pub fn request_identity(&self) -> &str {
        self.request_identity.as_str()
    }

    /// Returns the underlying neutral dispatch issue.
    #[must_use]
    pub const fn dispatch_issue(&self) -> crate::ToolDispatchPlanIssue {
        self.dispatch_issue
    }
}

/// Builds a pure fail-closed plan before executor effects.
#[must_use]
pub fn plan_tool_execution(request: ToolExecutionPlanRequest<'_>) -> ToolExecutionPlan {
    // r[impl agent_harness_runtime.tool_executor_kit.admission]
    let definition = request.catalog.find(request.request.name());
    let executor = definition.and_then(|current| find_executor(request.executors, current));
    let adapter_definitions = request.catalog.adapter_definitions();
    debug_assert_eq!(
        adapter_definitions.len(),
        request.catalog.definitions().len()
    );
    let dispatch_plan = crate::plan_tool_dispatch(crate::ToolDispatchPlanRequest {
        request: request.request,
        definitions: adapter_definitions.as_slice(),
        registry: request.capabilities,
        executor_present: executor.is_some(),
        policy: dispatch_policy(request.admission, executor),
    });
    let failure_class = dispatch_failure(dispatch_plan.issue())
        .or_else(|| definition.and_then(|current| definition_failure(request.request, current)));
    let status = if failure_class.is_none() && dispatch_plan.is_admitted() {
        ToolExecutionPlanStatus::Admitted
    } else {
        ToolExecutionPlanStatus::Denied
    };
    debug_assert_eq!(
        status == ToolExecutionPlanStatus::Admitted,
        failure_class.is_none() && dispatch_plan.is_admitted()
    );
    ToolExecutionPlan {
        status,
        failure_class,
        tool_name: String::from(request.request.name()),
        schema_identity: String::from(request.request.schema_identity()),
        executor_binding: definition.map(|current| String::from(current.executor_binding())),
        capability_labels: request.request.required_capability_labels().to_vec(),
        mutation_targets: request.request.mutation_targets().to_vec(),
        mutation_posture: definition.map(ToolDefinition::mutation_posture),
        output_policy: definition.map(ToolDefinition::output_policy),
        cancellation: request.admission.cancellation,
        deadline: request.admission.deadline,
        request_identity: tool_request_binding_identity(request.request),
        dispatch_issue: dispatch_plan.issue(),
    }
}

fn dispatch_policy(
    admission: ToolExecutorAdmission,
    executor: Option<&ToolExecutorBinding>,
) -> crate::ToolDispatchPolicyDraft<'static> {
    crate::ToolDispatchPolicyDraft {
        request_shape: TOOL_REQUEST_SHAPE_CURRENT,
        output_boundary: TOOL_OUTPUT_BOUNDARY_BOUNDED,
        cancellation_posture: match admission.cancellation {
            ToolExecutorCancellation::Continue => TOOL_CANCELLATION_CONTINUE,
            ToolExecutorCancellation::Cancelled => TOOL_CANCELLATION_CANCELLED,
        },
        timeout_posture: match admission.deadline {
            ToolExecutorDeadline::WithinDeadline => TOOL_TIMEOUT_WITHIN_DEADLINE,
            ToolExecutorDeadline::TimedOut => TOOL_TIMEOUT_TIMED_OUT,
        },
        fallback_policy: TOOL_FALLBACK_DENY,
        fallback_requested: admission.fallback == ToolFallbackPosture::Requested,
        capability_allowed: admission.capability == ToolCapabilityAdmission::Allowed,
        approval_granted: admission.approval == ToolApprovalAdmission::Granted,
        mutation_conflict: admission.mutation == ToolMutationCoordination::Conflict,
        executor_available: matches!(
            executor.map(ToolExecutorBinding::readiness),
            Some(ToolExecutorReadiness::Ready)
        ),
    }
}

fn find_executor<'a>(
    executors: &'a [ToolExecutorBinding],
    definition: &ToolDefinition,
) -> Option<&'a ToolExecutorBinding> {
    executors
        .iter()
        .find(|executor| executor.label() == definition.executor_binding())
}

fn dispatch_failure(issue: crate::ToolDispatchPlanIssue) -> Option<ToolExecutionFailureClass> {
    match issue {
        crate::ToolDispatchPlanIssue::None => None,
        crate::ToolDispatchPlanIssue::InvalidRequestShape => {
            Some(ToolExecutionFailureClass::MalformedArguments)
        }
        crate::ToolDispatchPlanIssue::UnknownTool => Some(ToolExecutionFailureClass::UnknownTool),
        crate::ToolDispatchPlanIssue::InactiveTool => {
            Some(ToolExecutionFailureClass::InactiveDefinition)
        }
        crate::ToolDispatchPlanIssue::UntrustedTool => {
            Some(ToolExecutionFailureClass::UntrustedDefinition)
        }
        crate::ToolDispatchPlanIssue::SchemaIdentityMismatch => {
            Some(ToolExecutionFailureClass::SchemaMismatch)
        }
        crate::ToolDispatchPlanIssue::MissingExecutor => {
            Some(ToolExecutionFailureClass::MissingExecutor)
        }
        crate::ToolDispatchPlanIssue::UnavailableExecutor => {
            Some(ToolExecutionFailureClass::UnavailableExecutor)
        }
        crate::ToolDispatchPlanIssue::DeniedCapability => {
            Some(ToolExecutionFailureClass::DeniedCapability)
        }
        crate::ToolDispatchPlanIssue::ApprovalDenied => {
            Some(ToolExecutionFailureClass::ApprovalDenied)
        }
        crate::ToolDispatchPlanIssue::Cancelled => Some(ToolExecutionFailureClass::Cancelled),
        crate::ToolDispatchPlanIssue::TimedOut => Some(ToolExecutionFailureClass::TimedOut),
        crate::ToolDispatchPlanIssue::MalformedOutputPolicy => {
            Some(ToolExecutionFailureClass::OutputPolicyDenied)
        }
        crate::ToolDispatchPlanIssue::UnsafeReference => {
            Some(ToolExecutionFailureClass::UnsafeOutput)
        }
        crate::ToolDispatchPlanIssue::UnboundedOutput => {
            Some(ToolExecutionFailureClass::UnboundedOutput)
        }
        crate::ToolDispatchPlanIssue::MutationConflict => {
            Some(ToolExecutionFailureClass::MutationConflict)
        }
        crate::ToolDispatchPlanIssue::FallbackDenied => {
            Some(ToolExecutionFailureClass::FallbackDenied)
        }
    }
}

fn definition_failure(
    request: &animus_adapters::ToolRequest,
    definition: &ToolDefinition,
) -> Option<ToolExecutionFailureClass> {
    let is_argument_name_set_matching = argument_names_match(request, definition);
    if !is_argument_name_set_matching {
        debug_assert!(!is_argument_name_set_matching);
        return Some(ToolExecutionFailureClass::MalformedArguments);
    }
    let is_approval_matching = request.approval_posture() == definition.approval_posture();
    if !is_approval_matching {
        debug_assert!(!is_approval_matching);
        return Some(ToolExecutionFailureClass::ApprovalDenied);
    }
    if request.output_policy().visible_char_budget > definition.output_policy().max_visible_chars()
    {
        return Some(ToolExecutionFailureClass::OutputPolicyDenied);
    }
    if !mutation_posture_matches(request, definition.mutation_posture()) {
        return Some(ToolExecutionFailureClass::MutationConflict);
    }
    if request
        .required_capability_labels()
        .iter()
        .chain(request.mutation_targets().iter())
        .any(|label| !is_safe_label(label))
    {
        return Some(ToolExecutionFailureClass::UnsafeOutput);
    }
    None
}

fn argument_names_match(
    request: &animus_adapters::ToolRequest,
    definition: &ToolDefinition,
) -> bool {
    let mut actual: Vec<&str> = request
        .arguments()
        .iter()
        .map(animus_adapters::tool::ArgumentField::name)
        .collect();
    let mut expected: Vec<&str> = definition
        .argument_names()
        .iter()
        .map(String::as_str)
        .collect();
    actual.sort_unstable();
    expected.sort_unstable();
    actual == expected
}

fn mutation_posture_matches(
    request: &animus_adapters::ToolRequest,
    posture: ToolMutationPosture,
) -> bool {
    match posture {
        ToolMutationPosture::ReadOnly => request.mutation_targets().is_empty(),
        ToolMutationPosture::DeclaredTargets => !request.mutation_targets().is_empty(),
    }
}

/// Transient owned output returned by an injected executor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToolExecutorOutput {
    body: String,
    full_output_reference: Option<String>,
    redaction_class: animus_adapters::tool::OutputRedactionClass,
}

impl ToolExecutorOutput {
    /// Builds transient executor output before safe normalization.
    #[must_use]
    pub fn new(
        body: &str,
        full_output_reference: Option<&str>,
        redaction_class: animus_adapters::tool::OutputRedactionClass,
    ) -> Self {
        Self {
            body: String::from(body),
            full_output_reference: full_output_reference.map(String::from),
            redaction_class,
        }
    }

    /// Returns the transient output body.
    #[must_use]
    pub fn body(&self) -> &str {
        self.body.as_str()
    }

    /// Returns the optional host-owned full-output reference.
    #[must_use]
    pub fn full_output_reference(&self) -> Option<&str> {
        self.full_output_reference.as_deref()
    }

    /// Returns the output redaction class.
    #[must_use]
    pub const fn redaction_class(&self) -> animus_adapters::tool::OutputRedactionClass {
        self.redaction_class
    }
}

/// Typed failure returned by an injected executor service.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ToolExecutorServiceFailure {
    /// The executor failed without returning safe output.
    ExecutionFailed,
    /// The executor observed cancellation after dispatch.
    Cancelled,
    /// The executor observed a caller-owned deadline after dispatch.
    TimedOut,
}

/// Host-injected tool executor boundary.
///
/// Implementations own all concrete filesystem, process, network, plugin,
/// provider, persistence, and other host effects they choose to expose.
pub trait ToolExecutorService {
    /// Returns the explicit binding label implemented by this service.
    fn binding_label(&self) -> &str;

    /// Executes one already-admitted plan.
    ///
    /// # Errors
    ///
    /// Returns a typed safe failure without durable raw output.
    fn execute(
        &mut self,
        plan: &ToolExecutionPlan,
        request: &animus_adapters::ToolRequest,
    ) -> Result<ToolExecutorOutput, ToolExecutorServiceFailure>;
}

/// Durable tool execution status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ToolExecutionReceiptStatus {
    /// An injected executor completed and safe output metadata was recorded.
    Completed,
    /// Admission denied executor effects.
    Denied,
}

/// Safe bounded durable receipt for tool execution.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToolExecutionReceipt {
    contract_identity: &'static str,
    status: ToolExecutionReceiptStatus,
    failure_class: Option<ToolExecutionFailureClass>,
    tool_name: String,
    schema_identity: String,
    executor_binding: String,
    capability_labels: Vec<String>,
    mutation_targets: Vec<String>,
    output_byte_count: u64,
    output_char_count: u64,
    output_line_count: u64,
    output_hash: String,
    safe_output_reference: Option<String>,
    output_redaction_class: &'static str,
    redaction_marker: &'static str,
    receipt_hash: String,
}

/// Bounded model-visible output and metadata-only durable evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToolExecutionOutcome {
    response: animus_adapters::ToolResponse,
    receipt: ToolExecutionReceipt,
}

impl ToolExecutionOutcome {
    /// Returns the bounded response for model-visible context.
    #[must_use]
    pub const fn response(&self) -> &animus_adapters::ToolResponse {
        &self.response
    }

    /// Returns metadata-only durable execution evidence.
    #[must_use]
    pub const fn receipt(&self) -> &ToolExecutionReceipt {
        &self.receipt
    }
}

impl ToolExecutionReceipt {
    /// Returns the receipt contract identity.
    #[must_use]
    pub const fn contract_identity(&self) -> &'static str {
        self.contract_identity
    }

    /// Returns completed or denied status.
    #[must_use]
    pub const fn status(&self) -> ToolExecutionReceiptStatus {
        self.status
    }

    /// Returns a typed failure class for denied receipts.
    #[must_use]
    pub const fn failure_class(&self) -> Option<ToolExecutionFailureClass> {
        self.failure_class
    }

    /// Returns the safe tool label.
    #[must_use]
    pub fn tool_name(&self) -> &str {
        self.tool_name.as_str()
    }

    /// Returns the safe executor binding label.
    #[must_use]
    pub fn executor_binding(&self) -> &str {
        self.executor_binding.as_str()
    }

    /// Returns the original raw output byte count.
    #[must_use]
    pub const fn output_byte_count(&self) -> u64 {
        self.output_byte_count
    }

    /// Returns the BLAKE3 raw output identity.
    #[must_use]
    pub fn output_hash(&self) -> &str {
        self.output_hash.as_str()
    }

    /// Returns the BLAKE3 receipt identity.
    #[must_use]
    pub fn receipt_hash(&self) -> &str {
        self.receipt_hash.as_str()
    }
}

/// Error returned by executor-kit construction, execution, or receipt checks.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ToolExecutorKitError {
    /// Catalog definition count exceeded its named bound.
    CatalogTooLarge {
        /// Maximum definition count.
        limit: usize,
        /// Observed definition count.
        actual: usize,
    },
    /// A duplicate tool identity was registered.
    DuplicateTool,
    /// Argument metadata was malformed, duplicated, or unbounded.
    InvalidArgumentNames,
    /// Output bounds were zero or otherwise malformed.
    InvalidOutputPolicy,
    /// A safe label or reference contained prohibited raw material.
    UnsafeMetadata,
    /// The supplied plan was denied before execution.
    PlanDenied {
        /// First fail-closed plan class.
        failure: ToolExecutionFailureClass,
    },
    /// The supplied request values do not match the admitted plan.
    RequestBindingMismatch,
    /// The injected service did not implement the admitted binding.
    BindingMismatch,
    /// The injected executor returned a typed failure.
    ServiceFailed {
        /// Safe executor failure.
        failure: ToolExecutorServiceFailure,
    },
    /// Raw output exceeded its declared byte bound.
    OutputTooLarge {
        /// Declared byte limit.
        limit: u64,
        /// Observed output bytes.
        actual: u64,
    },
    /// Output failed adapter-level bounded-output validation.
    InvalidOutput,
    /// Durable receipt validation found a leak or inconsistent field.
    UnsafeReceipt,
}

/// Calls one matching injected executor and returns only a safe durable receipt.
///
/// This is the imperative shell of the kit. It performs no discovery or
/// fallback and calls the service only after a pure admitted plan.
///
/// # Errors
///
/// Returns [`ToolExecutorKitError`] when the plan is denied, the service binding
/// differs, the service fails, output is unsafe or unbounded, or receipt
/// validation fails.
pub fn execute_admitted_tool<S: ToolExecutorService>(
    service: &mut S,
    plan: &ToolExecutionPlan,
    request: &animus_adapters::ToolRequest,
) -> Result<ToolExecutionOutcome, ToolExecutorKitError> {
    let output = invoke_admitted_tool(service, plan, request)?;
    validate_tool_executor_output(plan, &output)
}

/// Calls one matching injected executor without normalizing its transient output.
///
/// The staged tool shell uses this seam so output handlers can only narrow the
/// transient value before the executor kit performs final validation.
///
/// # Errors
///
/// Returns [`ToolExecutorKitError`] when admission, request binding, executor
/// binding, or the injected service fails.
pub fn invoke_admitted_tool<S: ToolExecutorService>(
    service: &mut S,
    plan: &ToolExecutionPlan,
    request: &animus_adapters::ToolRequest,
) -> Result<ToolExecutorOutput, ToolExecutorKitError> {
    let Some(binding) = plan.executor_binding() else {
        return Err(plan_denied_error(plan));
    };
    if !plan.is_admitted() {
        return Err(plan_denied_error(plan));
    }
    if tool_request_binding_identity(request) != plan.request_identity {
        return Err(ToolExecutorKitError::RequestBindingMismatch);
    }
    if service.binding_label() != binding {
        return Err(ToolExecutorKitError::BindingMismatch);
    }
    service
        .execute(plan, request)
        .map_err(|failure| ToolExecutorKitError::ServiceFailed { failure })
}

/// Performs the existing final bounded-output and receipt validation.
///
/// # Errors
///
/// Returns [`ToolExecutorKitError`] when transient output is unsafe,
/// unbounded, inconsistent with policy, or cannot produce safe evidence.
pub fn validate_tool_executor_output(
    plan: &ToolExecutionPlan,
    output: &ToolExecutorOutput,
) -> Result<ToolExecutionOutcome, ToolExecutorKitError> {
    if !plan.is_admitted() {
        return Err(plan_denied_error(plan));
    }
    completed_outcome(plan, output)
}

/// Builds a safe receipt proving that a plan denied executor effects.
///
/// # Errors
///
/// Returns [`ToolExecutorKitError::UnsafeReceipt`] if plan metadata is unsafe or
/// [`ToolExecutorKitError::InvalidOutput`] if the plan was admitted.
pub fn denied_tool_receipt(
    plan: &ToolExecutionPlan,
) -> Result<ToolExecutionReceipt, ToolExecutorKitError> {
    let Some(failure_class) = plan.failure_class() else {
        return Err(ToolExecutorKitError::InvalidOutput);
    };
    debug_assert_eq!(plan.status(), ToolExecutionPlanStatus::Denied);
    debug_assert!(!plan.is_admitted());
    let mut receipt = ToolExecutionReceipt {
        contract_identity: TOOL_EXECUTOR_KIT_CONTRACT_IDENTITY,
        status: ToolExecutionReceiptStatus::Denied,
        failure_class: Some(failure_class),
        tool_name: plan.tool_name.clone(),
        schema_identity: plan.schema_identity.clone(),
        executor_binding: plan.executor_binding.clone().unwrap_or_default(),
        capability_labels: plan.capability_labels.clone(),
        mutation_targets: plan.mutation_targets.clone(),
        output_byte_count: EMPTY_OUTPUT_COUNT,
        output_char_count: EMPTY_OUTPUT_COUNT,
        output_line_count: EMPTY_OUTPUT_COUNT,
        output_hash: blake3_hash(OUTPUT_HASH_DOMAIN, &[]),
        safe_output_reference: None,
        output_redaction_class: animus_adapters::tool::OutputRedactionClass::Redacted.as_str(),
        redaction_marker: RECEIPT_REDACTION_MARKER,
        receipt_hash: String::new(),
    };
    receipt.receipt_hash = receipt_identity(&receipt);
    validate_tool_execution_receipt(&receipt)?;
    Ok(receipt)
}

fn completed_outcome(
    plan: &ToolExecutionPlan,
    output: &ToolExecutorOutput,
) -> Result<ToolExecutionOutcome, ToolExecutorKitError> {
    let policy = plan
        .output_policy()
        .ok_or(ToolExecutorKitError::InvalidOutputPolicy)?;
    if contains_unsafe_material(output.body.as_str()) {
        return Err(ToolExecutorKitError::UnsafeMetadata);
    }
    let output_bytes = usize_to_u64(output.body.len())?;
    if output_bytes > policy.max_output_bytes() {
        return Err(ToolExecutorKitError::OutputTooLarge {
            limit: policy.max_output_bytes(),
            actual: output_bytes,
        });
    }
    if output.full_output_reference.is_some() && !policy.allows_safe_reference() {
        return Err(ToolExecutorKitError::UnsafeMetadata);
    }
    let bounded = animus_adapters::tool::BoundedOutput::new(
        &animus_adapters::tool::OutputDraft {
            body: output.body.as_str(),
            full_output_reference: output.full_output_reference.as_deref(),
            hash_requirement: animus_adapters::tool::HashRequirement::Required,
        },
        animus_adapters::tool::OutputPolicy {
            visible_char_budget: policy.max_visible_chars(),
        },
    )
    .map_err(|_error| ToolExecutorKitError::InvalidOutput)?;
    debug_assert!(bounded.byte_count() <= policy.max_output_bytes());
    debug_assert!(bounded.content_hash().is_some());
    let receipt = completed_receipt(plan, &bounded, output.redaction_class)?;
    let response = animus_adapters::ToolResponse::from_bounded(bounded, output.redaction_class);
    Ok(ToolExecutionOutcome { response, receipt })
}

fn completed_receipt(
    plan: &ToolExecutionPlan,
    bounded: &animus_adapters::tool::BoundedOutput,
    redaction_class: animus_adapters::tool::OutputRedactionClass,
) -> Result<ToolExecutionReceipt, ToolExecutorKitError> {
    debug_assert!(!plan.tool_name.is_empty());
    debug_assert!(bounded.content_hash().is_some());
    let output_hash = bounded
        .content_hash()
        .map(String::from)
        .ok_or(ToolExecutorKitError::InvalidOutput)?;
    let mut receipt = ToolExecutionReceipt {
        contract_identity: TOOL_EXECUTOR_KIT_CONTRACT_IDENTITY,
        status: ToolExecutionReceiptStatus::Completed,
        failure_class: None,
        tool_name: plan.tool_name.clone(),
        schema_identity: plan.schema_identity.clone(),
        executor_binding: plan.executor_binding.clone().unwrap_or_default(),
        capability_labels: plan.capability_labels.clone(),
        mutation_targets: plan.mutation_targets.clone(),
        output_byte_count: bounded.byte_count(),
        output_char_count: bounded.char_count(),
        output_line_count: bounded.line_count(),
        output_hash,
        safe_output_reference: bounded.full_output_reference().map(String::from),
        output_redaction_class: redaction_class.as_str(),
        redaction_marker: RECEIPT_REDACTION_MARKER,
        receipt_hash: String::new(),
    };
    receipt.receipt_hash = receipt_identity(&receipt);
    validate_tool_execution_receipt(&receipt)?;
    Ok(receipt)
}

/// Validates safe bounded receipt fields without reading host state.
///
/// # Errors
///
/// Returns [`ToolExecutorKitError::UnsafeReceipt`] when labels, references,
/// hashes, status/failure combinations, or redaction markers are unsafe.
pub fn validate_tool_execution_receipt(
    receipt: &ToolExecutionReceipt,
) -> Result<(), ToolExecutorKitError> {
    // r[impl agent_harness_runtime.tool_executor_kit.receipts]
    let is_status_valid = matches!(
        (receipt.status, receipt.failure_class),
        (ToolExecutionReceiptStatus::Completed, None)
            | (ToolExecutionReceiptStatus::Denied, Some(_))
    );
    if !is_status_valid {
        return Err(ToolExecutorKitError::UnsafeReceipt);
    }
    let is_scalar_label_set_safe = receipt.contract_identity == TOOL_EXECUTOR_KIT_CONTRACT_IDENTITY
        && is_safe_label(receipt.tool_name.as_str())
        && is_safe_label(receipt.schema_identity.as_str())
        && (receipt.executor_binding.is_empty()
            || is_safe_label(receipt.executor_binding.as_str()))
        && is_safe_label(receipt.output_redaction_class)
        && receipt.redaction_marker == RECEIPT_REDACTION_MARKER;
    if !is_scalar_label_set_safe {
        return Err(ToolExecutorKitError::UnsafeReceipt);
    }
    let is_collection_label_set_safe = receipt
        .capability_labels
        .iter()
        .all(|label| is_safe_label(label))
        && receipt
            .mutation_targets
            .iter()
            .all(|label| is_safe_label(label));
    if !is_collection_label_set_safe {
        return Err(ToolExecutorKitError::UnsafeReceipt);
    }
    let is_reference_safe = receipt
        .safe_output_reference
        .as_deref()
        .is_none_or(is_safe_reference);
    if !is_reference_safe {
        return Err(ToolExecutorKitError::UnsafeReceipt);
    }
    let is_hash_set_safe =
        is_blake3_hex(receipt.output_hash.as_str()) && is_blake3_hex(receipt.receipt_hash.as_str());
    if !is_hash_set_safe {
        return Err(ToolExecutorKitError::UnsafeReceipt);
    }
    debug_assert!(is_status_valid);
    debug_assert!(is_hash_set_safe);
    Ok(())
}

fn plan_denied_error(plan: &ToolExecutionPlan) -> ToolExecutorKitError {
    ToolExecutorKitError::PlanDenied {
        failure: plan
            .failure_class()
            .unwrap_or(ToolExecutionFailureClass::MissingExecutor),
    }
}

fn validate_argument_names(values: &[&str]) -> Result<Vec<String>, ToolExecutorKitError> {
    if values.len() > MAX_ARGUMENT_NAMES {
        return Err(ToolExecutorKitError::InvalidArgumentNames);
    }
    let mut normalized = Vec::with_capacity(values.len());
    for value in values {
        validate_safe_label(value).map_err(|_error| ToolExecutorKitError::InvalidArgumentNames)?;
        if normalized.iter().any(|seen| seen == value) {
            return Err(ToolExecutorKitError::InvalidArgumentNames);
        }
        normalized.push(String::from(*value));
    }
    normalized.sort();
    Ok(normalized)
}

fn validate_safe_label(value: &str) -> Result<(), ToolExecutorKitError> {
    if is_safe_label(value) {
        Ok(())
    } else {
        Err(ToolExecutorKitError::UnsafeMetadata)
    }
}

fn is_safe_label(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty()
        && trimmed.chars().count() <= MAX_SAFE_LABEL_CHARS
        && !contains_unsafe_material(trimmed)
        && !trimmed.contains("http://")
        && !trimmed.contains("https://")
}

fn is_safe_reference(value: &str) -> bool {
    is_safe_label(value)
        && !value.starts_with('/')
        && !value.starts_with('~')
        && !value.contains("..")
        && !value.contains("://")
}

fn contains_unsafe_material(value: &str) -> bool {
    UNSAFE_MARKERS.iter().any(|marker| value.contains(marker))
}

fn is_blake3_hex(value: &str) -> bool {
    value.len() == BLAKE3_HEX_CHARS && value.as_bytes().iter().all(u8::is_ascii_hexdigit)
}

/// Returns the domain-separated exact identity of one validated tool request.
#[must_use]
pub fn tool_request_binding_identity(request: &animus_adapters::ToolRequest) -> String {
    debug_assert!(!request.name().is_empty());
    debug_assert!(!request.schema_identity().is_empty());
    let mut hasher = blake3::Hasher::new();
    hasher.update(REQUEST_HASH_DOMAIN.as_bytes());
    update_request_field(&mut hasher, ("tool_name", request.name()));
    update_request_field(&mut hasher, ("schema_identity", request.schema_identity()));
    update_request_field(&mut hasher, ("correlation_id", request.correlation_id()));
    update_request_field(
        &mut hasher,
        (
            "visible_char_budget",
            request
                .output_policy()
                .visible_char_budget
                .to_string()
                .as_str(),
        ),
    );
    update_request_field(
        &mut hasher,
        ("approval_posture", request.approval_posture().as_str()),
    );
    for argument in request.arguments() {
        update_request_field(&mut hasher, ("argument_name", argument.name()));
        update_request_field(&mut hasher, ("argument_value", argument.value()));
    }
    for capability in request.required_capability_labels() {
        update_request_field(&mut hasher, ("capability", capability));
    }
    for target in request.mutation_targets() {
        update_request_field(&mut hasher, ("mutation_target", target));
    }
    hasher.finalize().to_hex().to_string()
}

fn update_request_field(hasher: &mut blake3::Hasher, field: (&str, &str)) {
    let (label, value) = field;
    let label_hash = blake3_hash(REQUEST_FIELD_HASH_DOMAIN, label.as_bytes());
    let value_hash = blake3_hash(REQUEST_FIELD_HASH_DOMAIN, value.as_bytes());
    hasher.update(label_hash.as_bytes());
    hasher.update(value_hash.as_bytes());
}

fn receipt_identity(receipt: &ToolExecutionReceipt) -> String {
    let failure = receipt
        .failure_class
        .map_or("none", ToolExecutionFailureClass::as_str);
    let canonical = [
        receipt.contract_identity,
        receipt.tool_name.as_str(),
        receipt.schema_identity.as_str(),
        receipt.executor_binding.as_str(),
        failure,
        receipt.output_hash.as_str(),
        receipt.redaction_marker,
    ]
    .join(HASH_SEPARATOR);
    blake3_hash(RECEIPT_HASH_DOMAIN, canonical.as_bytes())
}

fn blake3_hash(domain: &str, bytes: &[u8]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain.as_bytes());
    hasher.update(HASH_SEPARATOR.as_bytes());
    hasher.update(bytes);
    hasher.finalize().to_hex().to_string()
}

fn usize_to_u64(value: usize) -> Result<u64, ToolExecutorKitError> {
    u64::try_from(value).map_err(|_error| ToolExecutorKitError::InvalidOutput)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOOL_NAME: &str = "lookup";
    const TOOL_LABEL: &str = "Lookup";
    const TOOL_DESCRIPTION: &str = "Read a deterministic fixture";
    const TOOL_SCHEMA: &str = "schema.lookup.v1";
    const OTHER_TOOL_SCHEMA: &str = "schema.lookup.v2";
    const TOOL_PROMPT: &str = "Use lookup for fixture facts.";
    const TOOL_GUIDELINE: &str = "Return bounded summaries.";
    const TOOL_SOURCE: &str = "fixture.registry";
    const TOOL_PROVENANCE: &str = "fixture";
    const TOOL_CAPABILITY: &str = "tool.lookup";
    const TOOL_RESULT_SHAPE: &str = "summary.v1";
    const EXECUTOR_BINDING: &str = "fixture.lookup.executor";
    const ARGUMENT_NAME: &str = "query";
    const EXTRA_ARGUMENT_NAME: &str = "scope";
    const ARGUMENT_VALUE: &str = "contracts";
    const OTHER_ARGUMENT_VALUE: &str = "substituted";
    const CORRELATION_ID: &str = "tool-call-lookup";
    const MUTATION_TARGET: &str = "resource:fixture";
    const OUTPUT_BODY: &str = "bounded fixture result";
    const SAFE_REFERENCE: &str = "artifact:tool-output";
    const PRIVATE_REFERENCE: &str = "/home/example/private.txt";
    const UNSAFE_OUTPUT: &str = "CREDENTIAL: secret";
    const OVERSIZED_FILL: char = 'x';
    const MAX_VISIBLE_CHARS: u32 = 64;
    const OVER_BUDGET_VISIBLE_CHARS: u32 = MAX_VISIBLE_CHARS.saturating_add(1);
    const MAX_OUTPUT_BYTES: u64 = 128;
    const OVERSIZED_OUTPUT_BYTES: usize = 129;
    const TRUNCATED_OUTPUT_CHARS: usize = 65;
    const EXPECTED_EXECUTION_COUNT: usize = 1;
    const NO_EXECUTION_COUNT: usize = 0;

    #[test]
    fn fake_executor_completes_read_only_and_declared_mutation_calls() {
        // r[verify agent_harness_runtime.tool_executor_kit.contracts]
        // r[verify agent_harness_runtime.tool_executor_kit.admission]
        // r[verify agent_harness_runtime.tool_executor_kit.receipts]
        // r[verify agent_harness_runtime.tool_executor_kit.verification]
        for mutation_posture in [
            ToolMutationPosture::ReadOnly,
            ToolMutationPosture::DeclaredTargets,
        ] {
            let fixture = Fixture::new(
                animus_adapters::tool::DefinitionStatus::Active,
                mutation_posture,
                &[ARGUMENT_NAME],
            );
            let mutation_targets = if mutation_posture == ToolMutationPosture::ReadOnly {
                &[][..]
            } else {
                &[MUTATION_TARGET][..]
            };
            let request = tool_request(&[ARGUMENT_NAME], mutation_targets);
            let plan = fixture.plan(&request, ToolExecutorAdmission::admitted());
            let mut executor = FakeExecutor::succeed(OUTPUT_BODY, Some(SAFE_REFERENCE));

            let outcome =
                execute_admitted_tool(&mut executor, &plan, &request).expect("admitted execution");
            let receipt = outcome.receipt();

            assert!(plan.is_admitted());
            assert_eq!(executor.calls, EXPECTED_EXECUTION_COUNT);
            assert_eq!(outcome.response().model_visible_text(), OUTPUT_BODY);
            assert_eq!(receipt.status(), ToolExecutionReceiptStatus::Completed);
            assert_eq!(receipt.tool_name(), TOOL_NAME);
            assert_eq!(receipt.executor_binding(), EXECUTOR_BINDING);
            assert_eq!(receipt.output_byte_count(), OUTPUT_BODY.len() as u64);
            assert!(is_blake3_hex(receipt.output_hash()));
            assert!(validate_tool_execution_receipt(receipt).is_ok());
        }
    }

    #[test]
    fn admitted_output_returns_only_the_bounded_model_visible_prefix() {
        // r[verify agent_harness_runtime.read_only_codebase_tool.outcome.bounded]
        let fixture = Fixture::new(
            animus_adapters::tool::DefinitionStatus::Active,
            ToolMutationPosture::ReadOnly,
            &[ARGUMENT_NAME],
        );
        let request = tool_request(&[ARGUMENT_NAME], &[]);
        let plan = fixture.plan(&request, ToolExecutorAdmission::admitted());
        let body: String = std::iter::repeat_n(OVERSIZED_FILL, TRUNCATED_OUTPUT_CHARS).collect();
        let mut executor = FakeExecutor::succeed(body.as_str(), None);

        let outcome =
            execute_admitted_tool(&mut executor, &plan, &request).expect("bounded execution");

        assert_eq!(
            outcome.response().status(),
            animus_adapters::tool::OutputStatus::Truncated
        );
        assert_eq!(
            outcome.response().model_visible_text().chars().count(),
            MAX_VISIBLE_CHARS as usize
        );
        assert_eq!(outcome.receipt().output_byte_count(), body.len() as u64);
        assert!(!format!("{:?}", outcome.receipt()).contains(body.as_str()));
    }

    #[test]
    fn admission_denies_missing_policy_schema_and_argument_failures_without_execution() {
        // r[verify agent_harness_runtime.tool_executor_kit.verification]
        let active = Fixture::new(
            animus_adapters::tool::DefinitionStatus::Active,
            ToolMutationPosture::ReadOnly,
            &[ARGUMENT_NAME],
        );
        let request = tool_request(&[ARGUMENT_NAME], &[]);

        assert_service_and_capability_denials(&active, &request);
        assert_schema_and_argument_denials(&active, &request);
        assert_definition_status_denials(&request);

        let mut executor = FakeExecutor::succeed(OUTPUT_BODY, None);
        let denied = active.plan_without_executors(&request);
        assert!(execute_admitted_tool(&mut executor, &denied, &request).is_err());
        assert_eq!(executor.calls, NO_EXECUTION_COUNT);
    }

    fn assert_service_and_capability_denials(
        active: &Fixture,
        request: &animus_adapters::ToolRequest,
    ) {
        assert_denied(
            &active.plan_without_executors(request),
            ToolExecutionFailureClass::MissingExecutor,
        );
        assert_denied(
            &active.plan_with_readiness(request, ToolExecutorReadiness::Unavailable),
            ToolExecutionFailureClass::UnavailableExecutor,
        );
        let mut denied_capability = ToolExecutorAdmission::admitted();
        denied_capability.capability = ToolCapabilityAdmission::Denied;
        assert_denied(
            &active.plan(request, denied_capability),
            ToolExecutionFailureClass::DeniedCapability,
        );
    }

    fn assert_schema_and_argument_denials(
        active: &Fixture,
        request: &animus_adapters::ToolRequest,
    ) {
        assert_eq!(request.name(), TOOL_NAME);
        assert_eq!(request.schema_identity(), TOOL_SCHEMA);
        let malformed_fixture = Fixture::new(
            animus_adapters::tool::DefinitionStatus::Active,
            ToolMutationPosture::ReadOnly,
            &[ARGUMENT_NAME, EXTRA_ARGUMENT_NAME],
        );
        assert_denied(
            &malformed_fixture.plan(request, ToolExecutorAdmission::admitted()),
            ToolExecutionFailureClass::MalformedArguments,
        );
        let wrong_schema_request = tool_request_with_policy(&ToolRequestOptions {
            schema_identity: OTHER_TOOL_SCHEMA,
            visible_char_budget: MAX_VISIBLE_CHARS,
            argument_names: &[ARGUMENT_NAME],
            mutation_targets: &[],
            argument_value: ARGUMENT_VALUE,
        });
        assert_denied(
            &active.plan(&wrong_schema_request, ToolExecutorAdmission::admitted()),
            ToolExecutionFailureClass::SchemaMismatch,
        );
        let over_budget_request = tool_request_with_policy(&ToolRequestOptions {
            schema_identity: TOOL_SCHEMA,
            visible_char_budget: OVER_BUDGET_VISIBLE_CHARS,
            argument_names: &[ARGUMENT_NAME],
            mutation_targets: &[],
            argument_value: ARGUMENT_VALUE,
        });
        assert_denied(
            &active.plan(&over_budget_request, ToolExecutorAdmission::admitted()),
            ToolExecutionFailureClass::OutputPolicyDenied,
        );
    }

    fn assert_definition_status_denials(request: &animus_adapters::ToolRequest) {
        for status in [
            animus_adapters::tool::DefinitionStatus::Inactive,
            animus_adapters::tool::DefinitionStatus::Untrusted,
        ] {
            let fixture = Fixture::new(status, ToolMutationPosture::ReadOnly, &[ARGUMENT_NAME]);
            let expected = if status == animus_adapters::tool::DefinitionStatus::Inactive {
                ToolExecutionFailureClass::InactiveDefinition
            } else {
                ToolExecutionFailureClass::UntrustedDefinition
            };
            assert_denied(
                &fixture.plan(request, ToolExecutorAdmission::admitted()),
                expected,
            );
        }
    }

    #[test]
    fn request_value_substitution_fails_before_executor_effects() {
        // r[verify agent_harness_runtime.read_only_codebase_tool.request_binding.substitution]
        let fixture = Fixture::new(
            animus_adapters::tool::DefinitionStatus::Active,
            ToolMutationPosture::ReadOnly,
            &[ARGUMENT_NAME],
        );
        let request = tool_request(&[ARGUMENT_NAME], &[]);
        let plan = fixture.plan(&request, ToolExecutorAdmission::admitted());
        let substituted =
            tool_request_with_argument_value(&[ARGUMENT_NAME], &[], OTHER_ARGUMENT_VALUE);
        let mut executor = FakeExecutor::succeed(OUTPUT_BODY, None);

        assert_eq!(
            execute_admitted_tool(&mut executor, &plan, &substituted),
            Err(ToolExecutorKitError::RequestBindingMismatch)
        );
        assert_eq!(executor.calls, NO_EXECUTION_COUNT);
    }

    #[test]
    fn admission_denies_mutation_cancellation_timeout_fallback_and_approval_failures() {
        let read_only = Fixture::new(
            animus_adapters::tool::DefinitionStatus::Active,
            ToolMutationPosture::ReadOnly,
            &[ARGUMENT_NAME],
        );
        let mutating_request = tool_request(&[ARGUMENT_NAME], &[MUTATION_TARGET]);
        assert_denied(
            &read_only.plan(&mutating_request, ToolExecutorAdmission::admitted()),
            ToolExecutionFailureClass::MutationConflict,
        );
        let request = tool_request(&[ARGUMENT_NAME], &[]);
        assert_eq!(
            read_only.catalog.definitions().len(),
            EXPECTED_EXECUTION_COUNT
        );
        assert!(request.mutation_targets().is_empty());
        let mut mutation_conflict = ToolExecutorAdmission::admitted();
        mutation_conflict.mutation = ToolMutationCoordination::Conflict;
        assert_denied(
            &read_only.plan(&request, mutation_conflict),
            ToolExecutionFailureClass::MutationConflict,
        );
        let mut cancellation = ToolExecutorAdmission::admitted();
        cancellation.cancellation = ToolExecutorCancellation::Cancelled;
        assert_denied(
            &read_only.plan(&request, cancellation),
            ToolExecutionFailureClass::Cancelled,
        );
        let mut timed_out_admission = ToolExecutorAdmission::admitted();
        timed_out_admission.deadline = ToolExecutorDeadline::TimedOut;
        assert_denied(
            &read_only.plan(&request, timed_out_admission),
            ToolExecutionFailureClass::TimedOut,
        );
        let mut fallback = ToolExecutorAdmission::admitted();
        fallback.fallback = ToolFallbackPosture::Requested;
        assert_denied(
            &read_only.plan(&request, fallback),
            ToolExecutionFailureClass::FallbackDenied,
        );
        let approval_fixture =
            Fixture::with_approval(animus_adapters::tool::ApprovalPosture::RequiresApproval);
        assert_denied(
            &approval_fixture.plan(&request, ToolExecutorAdmission::admitted()),
            ToolExecutionFailureClass::ApprovalDenied,
        );
    }

    #[test]
    fn output_and_receipt_validation_rejects_unsafe_unbounded_and_leaking_material() {
        // r[verify agent_harness_runtime.tool_executor_kit.receipts.rejects_leaks]
        let fixture = Fixture::new(
            animus_adapters::tool::DefinitionStatus::Active,
            ToolMutationPosture::ReadOnly,
            &[ARGUMENT_NAME],
        );
        let request = tool_request(&[ARGUMENT_NAME], &[]);
        let plan = fixture.plan(&request, ToolExecutorAdmission::admitted());

        let mut unsafe_body = FakeExecutor::succeed(UNSAFE_OUTPUT, None);
        assert_eq!(
            execute_admitted_tool(&mut unsafe_body, &plan, &request),
            Err(ToolExecutorKitError::UnsafeMetadata)
        );
        let too_large_body: String =
            std::iter::repeat_n(OVERSIZED_FILL, OVERSIZED_OUTPUT_BYTES).collect();
        let mut too_large_service = FakeExecutor::succeed(too_large_body.as_str(), None);
        assert!(matches!(
            execute_admitted_tool(&mut too_large_service, &plan, &request),
            Err(ToolExecutorKitError::OutputTooLarge { .. })
        ));
        let mut unsafe_reference = FakeExecutor::succeed(OUTPUT_BODY, Some(PRIVATE_REFERENCE));
        assert_eq!(
            execute_admitted_tool(&mut unsafe_reference, &plan, &request),
            Err(ToolExecutorKitError::InvalidOutput)
        );

        let denied = fixture.plan_without_executors(&request);
        let mut receipt = denied_tool_receipt(&denied).expect("safe denial receipt");
        receipt.executor_binding = String::from(PRIVATE_REFERENCE);
        assert_eq!(
            validate_tool_execution_receipt(&receipt),
            Err(ToolExecutorKitError::UnsafeReceipt)
        );
        assert_eq!(
            ToolCatalog::new(vec![fixture.definition.clone(), fixture.definition]),
            Err(ToolExecutorKitError::DuplicateTool)
        );
    }

    fn assert_denied(plan: &ToolExecutionPlan, expected: ToolExecutionFailureClass) {
        assert_eq!(plan.status(), ToolExecutionPlanStatus::Denied);
        assert_eq!(plan.failure_class(), Some(expected));
        assert!(!plan.is_admitted());
        let receipt = denied_tool_receipt(plan).expect("denial receipt should be safe");
        assert_eq!(receipt.status(), ToolExecutionReceiptStatus::Denied);
        assert_eq!(receipt.failure_class(), Some(expected));
    }

    #[derive(Clone)]
    struct Fixture {
        definition: ToolDefinition,
        catalog: ToolCatalog,
        registry: crate::RuntimeCapabilityRegistry,
        executor: ToolExecutorBinding,
    }

    impl Fixture {
        fn new(
            status: animus_adapters::tool::DefinitionStatus,
            mutation_posture: ToolMutationPosture,
            argument_names: &[&str],
        ) -> Self {
            Self::build(
                status,
                mutation_posture,
                argument_names,
                animus_adapters::tool::ApprovalPosture::PreApproved,
            )
        }

        fn with_approval(approval: animus_adapters::tool::ApprovalPosture) -> Self {
            Self::build(
                animus_adapters::tool::DefinitionStatus::Active,
                ToolMutationPosture::ReadOnly,
                &[ARGUMENT_NAME],
                approval,
            )
        }

        fn build(
            status: animus_adapters::tool::DefinitionStatus,
            mutation_posture: ToolMutationPosture,
            argument_names: &[&str],
            approval: animus_adapters::tool::ApprovalPosture,
        ) -> Self {
            let metadata = animus_adapters::tool::Definition::validate(
                &animus_adapters::tool::DefinitionDraft {
                    name: Some(TOOL_NAME),
                    label: Some(TOOL_LABEL),
                    description: Some(TOOL_DESCRIPTION),
                    parameter_schema_identity: Some(TOOL_SCHEMA),
                    prompt_snippet: Some(TOOL_PROMPT),
                    prompt_guidelines: &[TOOL_GUIDELINE],
                    source_identity: Some(TOOL_SOURCE),
                    provenance: Some(TOOL_PROVENANCE),
                    capability_labels: &[TOOL_CAPABILITY],
                    result_metadata_shape: Some(TOOL_RESULT_SHAPE),
                    status,
                },
            )
            .expect("valid metadata");
            let output_policy =
                ToolExecutorOutputPolicy::new(MAX_VISIBLE_CHARS, MAX_OUTPUT_BYTES, true)
                    .expect("valid output policy");
            let definition = ToolDefinition::from_draft(ToolDefinitionDraft {
                metadata,
                executor_binding: EXECUTOR_BINDING,
                argument_names,
                mutation_posture,
                output_policy,
                approval_posture: approval,
            })
            .expect("valid executor definition");
            let catalog = ToolCatalog::new(vec![definition.clone()]).expect("valid catalog");
            let registry = crate::RuntimeCapabilityRegistry::from_drafts(&[
                crate::RuntimeCapabilityDescriptorDraft {
                    label: TOOL_CAPABILITY,
                    service_kind: crate::RuntimeCapabilityServiceKind::Tool.as_str(),
                    scope: None,
                },
            ])
            .expect("valid capability registry");
            let executor = ToolExecutorBinding::new(EXECUTOR_BINDING, ToolExecutorReadiness::Ready)
                .expect("valid executor binding");
            Self {
                definition,
                catalog,
                registry,
                executor,
            }
        }

        fn plan(
            &self,
            request: &animus_adapters::ToolRequest,
            admission: ToolExecutorAdmission,
        ) -> ToolExecutionPlan {
            plan_tool_execution(ToolExecutionPlanRequest {
                request,
                catalog: &self.catalog,
                capabilities: &self.registry,
                executors: core::slice::from_ref(&self.executor),
                admission,
            })
        }

        fn plan_without_executors(
            &self,
            request: &animus_adapters::ToolRequest,
        ) -> ToolExecutionPlan {
            plan_tool_execution(ToolExecutionPlanRequest {
                request,
                catalog: &self.catalog,
                capabilities: &self.registry,
                executors: &[],
                admission: ToolExecutorAdmission::admitted(),
            })
        }

        fn plan_with_readiness(
            &self,
            request: &animus_adapters::ToolRequest,
            readiness: ToolExecutorReadiness,
        ) -> ToolExecutionPlan {
            let executor = ToolExecutorBinding::new(EXECUTOR_BINDING, readiness)
                .expect("valid executor binding");
            plan_tool_execution(ToolExecutionPlanRequest {
                request,
                catalog: &self.catalog,
                capabilities: &self.registry,
                executors: &[executor],
                admission: ToolExecutorAdmission::admitted(),
            })
        }
    }

    struct FakeExecutor {
        binding: String,
        output: ToolExecutorOutput,
        calls: usize,
    }

    impl FakeExecutor {
        fn succeed(body: &str, reference: Option<&str>) -> Self {
            Self {
                binding: String::from(EXECUTOR_BINDING),
                output: ToolExecutorOutput::new(
                    body,
                    reference,
                    animus_adapters::tool::OutputRedactionClass::Safe,
                ),
                calls: NO_EXECUTION_COUNT,
            }
        }
    }

    impl ToolExecutorService for FakeExecutor {
        fn binding_label(&self) -> &str {
            self.binding.as_str()
        }

        fn execute(
            &mut self,
            _plan: &ToolExecutionPlan,
            _request: &animus_adapters::ToolRequest,
        ) -> Result<ToolExecutorOutput, ToolExecutorServiceFailure> {
            self.calls += EXPECTED_EXECUTION_COUNT;
            Ok(self.output.clone())
        }
    }

    fn tool_request(
        argument_names: &[&str],
        mutation_targets: &[&str],
    ) -> animus_adapters::ToolRequest {
        tool_request_with_argument_value(argument_names, mutation_targets, ARGUMENT_VALUE)
    }

    fn tool_request_with_argument_value(
        argument_names: &[&str],
        mutation_targets: &[&str],
        argument_value: &str,
    ) -> animus_adapters::ToolRequest {
        tool_request_with_policy(&ToolRequestOptions {
            schema_identity: TOOL_SCHEMA,
            visible_char_budget: MAX_VISIBLE_CHARS,
            argument_names,
            mutation_targets,
            argument_value,
        })
    }

    struct ToolRequestOptions<'a> {
        schema_identity: &'a str,
        visible_char_budget: u32,
        argument_names: &'a [&'a str],
        mutation_targets: &'a [&'a str],
        argument_value: &'a str,
    }

    fn tool_request_with_policy(options: &ToolRequestOptions<'_>) -> animus_adapters::ToolRequest {
        let fields: Vec<animus_adapters::tool::ArgumentFieldDraft<'_>> = options
            .argument_names
            .iter()
            .map(|name| animus_adapters::tool::ArgumentFieldDraft {
                name: Some(*name),
                value: Some(options.argument_value),
            })
            .collect();
        assert_eq!(fields.len(), options.argument_names.len());
        let arguments = animus_adapters::tool::PreparedArguments::prepare(
            &animus_adapters::tool::ArgumentPreparationRequest {
                current_schema_identity: options.schema_identity,
                allowed_field_names: options.argument_names,
                compatibility_rules: &[],
                fields: fields.as_slice(),
            },
        )
        .expect("valid arguments");
        assert_eq!(arguments.fields().len(), options.argument_names.len());
        animus_adapters::ToolRequest::from_draft(&animus_adapters::tool::RequestDraft {
            name: Some(TOOL_NAME),
            arguments: Some(&arguments),
            correlation_id: Some(CORRELATION_ID),
            output_policy: animus_adapters::tool::OutputPolicy {
                visible_char_budget: options.visible_char_budget,
            },
            mutation_targets: options.mutation_targets,
            required_capability_labels: &[TOOL_CAPABILITY],
            approval_posture: animus_adapters::tool::ApprovalPosture::PreApproved.as_str(),
        })
        .expect("valid tool request")
    }
}

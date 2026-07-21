type Text = alloc::string::String;
type Collection<T> = alloc::vec::Vec<T>;

const DISCOVERY_QUERY_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("tool_discovery_query");
const DETAIL_TOOL_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("tool_discovery_detail_tool");
const PLAN_BATCH_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("tool_plan_batch_id");
const PLAN_STEP_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("tool_plan_step_id");
const PLAN_DEPENDENCY_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("tool_plan_dependency_id");
const PLAN_BINDING_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("tool_plan_binding_id");
const PLAN_TARGET_ARGUMENT_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("tool_plan_target_argument");
const PLAN_FABRIC_KIND_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("tool_plan_fabric_kind");
const PLAN_FABRIC_PROFILE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("tool_plan_fabric_profile");
const PLAN_FABRIC_POLICY_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("tool_plan_fabric_policy_hash");
const PLAN_FABRIC_SCRIPT_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("tool_plan_fabric_script_hash");
const PLAN_FABRIC_ACTION_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("tool_plan_fabric_action");
const PLAN_FABRIC_BUDGET_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("tool_plan_fabric_budget");
const PLAN_FABRIC_RECEIPT_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("tool_plan_fabric_receipt_posture");
const PLAN_CONTENT_HASH_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("tool_plan_intermediate_hash");

const MIN_TOOL_PLANNING_ITEMS: usize = 1;
const MAX_DISCOVERY_ITEMS: usize = super::MAX_TOOL_METADATA_ITEMS;
const MAX_PLAN_STEPS: usize = super::MAX_TOOL_METADATA_ITEMS;
const MAX_PLAN_BINDINGS: usize = super::MAX_TOOL_METADATA_ITEMS;
const MIN_INTERMEDIATE_CHARS: u32 = 0;
const MAX_INTERMEDIATE_CHARS: u32 = super::MAX_VISIBLE_OUTPUT_CHARS;
const MIN_INTERMEDIATE_CHARS_USIZE: usize = 0;
const MAX_INTERMEDIATE_CHARS_USIZE: usize = crate::MAX_ADAPTER_FIELD_CHARS;

const FABRIC_KIND_STEEL: &str = "steel";
const FABRIC_KIND_TYPED: &str = "typed";
const FABRIC_ACTION_DISCOVERY: &str = "discovery";
const FABRIC_ACTION_EMIT_PLAN: &str = "emit_tool_plan";
const RECEIPT_POSTURE_SAFE: &str = "safe_receipt";
const BATCH_POLICY_SERIAL: &str = "serial";
const BATCH_POLICY_PARALLEL_SAFE: &str = "parallel_safe";
const DISCOVERY_RECEIPT_BACKEND: &str = "tool_discovery";
const PLAN_RECEIPT_BACKEND: &str = "tool_plan";
const RECEIPT_INTERCHANGE: &str = "safe_metadata";
const COUNT_STEPS: &str = "steps";
const COUNT_BINDINGS: &str = "bindings";
const COUNT_DISCOVERY_SUMMARIES: &str = "summaries";
const COUNT_DISCOVERY_DETAILS: &str = "details";

/// Stable contract identity for progressive tool-discovery DTOs.
pub const TOOL_DISCOVERY_CONTRACT_IDENTITY: &str = "animus.tool-discovery.v1";
/// Stable contract identity for declarative tool-plan DTOs.
pub const TOOL_PLAN_CONTRACT_IDENTITY: &str = "animus.tool-plan.v1";
/// Stable contract identity for safe tool-plan receipts.
pub const TOOL_PLAN_RECEIPT_CONTRACT_IDENTITY: &str = "animus.tool-plan-receipt.v1";

/// Bounded ceilings for progressive tool discovery.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DiscoveryCeilings {
    /// Maximum returned tool summaries.
    pub max_summaries: usize,
    /// Maximum returned detailed tool metadata records.
    pub max_details: usize,
}

impl DiscoveryCeilings {
    /// Returns the repository default bounded discovery ceilings.
    #[must_use]
    pub const fn bounded() -> Self {
        Self {
            max_summaries: MAX_DISCOVERY_ITEMS,
            max_details: MAX_DISCOVERY_ITEMS,
        }
    }
}

/// Borrowed progressive-discovery request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DiscoveryRequestDraft<'a> {
    /// Safe query label for review and receipts.
    pub query_label: Option<&'a str>,
    /// Optional exact capability labels used to filter summaries.
    pub required_capability_labels: &'a [&'a str],
    /// Tool names whose details should be returned.
    pub detail_tool_names: &'a [&'a str],
    /// Discovery ceilings.
    pub ceilings: DiscoveryCeilings,
}

/// Metadata-only tool summary returned by progressive discovery.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiscoverySummary {
    tool_name: Text,
    label: Text,
    schema_identity: Text,
    capability_labels: Collection<Text>,
    result_contract: Text,
    provenance: Text,
}

impl DiscoverySummary {
    /// Returns the stable tool name.
    #[must_use]
    pub fn tool_name(&self) -> &str {
        self.tool_name.as_str()
    }

    /// Returns the human-facing label.
    #[must_use]
    pub fn label(&self) -> &str {
        self.label.as_str()
    }

    /// Returns the parameter schema identity.
    #[must_use]
    pub fn schema_identity(&self) -> &str {
        self.schema_identity.as_str()
    }

    /// Returns exact capability labels advertised by this summary.
    #[must_use]
    pub fn capability_labels(&self) -> &[Text] {
        self.capability_labels.as_slice()
    }

    /// Returns the result contract label.
    #[must_use]
    pub fn result_contract(&self) -> &str {
        self.result_contract.as_str()
    }

    /// Returns safe provenance metadata.
    #[must_use]
    pub fn provenance(&self) -> &str {
        self.provenance.as_str()
    }
}

/// Detailed metadata for one discovered tool without executable code or handles.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiscoveryDetail {
    summary: DiscoverySummary,
    description: Text,
    source_identity: Text,
}

impl DiscoveryDetail {
    /// Returns the summary portion.
    #[must_use]
    pub const fn summary(&self) -> &DiscoverySummary {
        &self.summary
    }

    /// Returns the model-facing description.
    #[must_use]
    pub fn description(&self) -> &str {
        self.description.as_str()
    }

    /// Returns safe source identity metadata.
    #[must_use]
    pub fn source_identity(&self) -> &str {
        self.source_identity.as_str()
    }
}

/// Metadata-only discovery result plus safe receipt evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiscoverySet {
    query_label: Text,
    summaries: Collection<DiscoverySummary>,
    details: Collection<DiscoveryDetail>,
    ceilings: DiscoveryCeilings,
    discovery_identity: Text,
    receipt: crate::SafeReceipt,
}

impl DiscoverySet {
    /// Returns the safe query label.
    #[must_use]
    pub fn query_label(&self) -> &str {
        self.query_label.as_str()
    }

    /// Returns returned summaries.
    #[must_use]
    pub fn summaries(&self) -> &[DiscoverySummary] {
        self.summaries.as_slice()
    }

    /// Returns returned details.
    #[must_use]
    pub fn details(&self) -> &[DiscoveryDetail] {
        self.details.as_slice()
    }

    /// Returns applied ceilings.
    #[must_use]
    pub const fn ceilings(&self) -> DiscoveryCeilings {
        self.ceilings
    }

    /// Returns a BLAKE3 identity over safe discovery metadata.
    #[must_use]
    pub fn discovery_identity(&self) -> &str {
        self.discovery_identity.as_str()
    }

    /// Returns the safe discovery receipt.
    #[must_use]
    pub const fn receipt(&self) -> &crate::SafeReceipt {
        &self.receipt
    }

    fn contains_tool(&self, tool_name: &str) -> bool {
        self.summaries
            .iter()
            .any(|summary| summary.tool_name() == tool_name)
    }
}

/// Runs metadata-only progressive discovery over already-loaded definitions.
///
/// # Errors
///
/// Returns [`super::ContractError`] when labels are malformed, ceilings are out
/// of bounds, requested detail names are duplicated or unknown, or safe receipt
/// metadata validation rejects the generated evidence.
///
/// # Panics
///
/// Panics if validated discovery ceilings drift outside their admitted non-empty bounded range.
/// This indicates an internal validation contract violation after admission.
pub fn discover_tools(
    draft: &DiscoveryRequestDraft<'_>,
    definitions: &[super::Definition],
) -> Result<DiscoverySet, super::ContractError> {
    validate_bound(BoundCheck {
        field: DISCOVERY_QUERY_FIELD,
        value: draft.ceilings.max_summaries,
        max_items: MAX_DISCOVERY_ITEMS,
    })?;
    validate_bound(BoundCheck {
        field: DETAIL_TOOL_FIELD,
        value: draft.ceilings.max_details,
        max_items: MAX_DISCOVERY_ITEMS,
    })?;
    validate_max_count(MaxCountCheck {
        field: DISCOVERY_QUERY_FIELD,
        actual_items: definitions.len(),
        max_items: MAX_DISCOVERY_ITEMS,
    })?;
    validate_max_count(MaxCountCheck {
        field: DETAIL_TOOL_FIELD,
        actual_items: draft.detail_tool_names.len(),
        max_items: draft.ceilings.max_details,
    })?;
    assert!(draft.ceilings.max_summaries >= MIN_TOOL_PLANNING_ITEMS);
    assert!(draft.ceilings.max_details <= MAX_DISCOVERY_ITEMS);

    let query_label = super::required_text(DISCOVERY_QUERY_FIELD, draft.query_label)?;
    let capability_filter =
        collect_optional_labels(super::CAPABILITY_FIELD, draft.required_capability_labels)?;
    let detail_names = collect_unique_labels(DETAIL_TOOL_FIELD, draft.detail_tool_names)?;
    let summaries = discovery_summaries(definitions, &capability_filter, draft.ceilings);
    let details = discovery_details(&summaries, definitions, &detail_names)?;
    let discovery_identity = discovery_identity(&query_label, &summaries, &details);
    let receipt = discovery_receipt(DiscoveryReceiptInput {
        query_label: &query_label,
        discovery_identity: &discovery_identity,
        summaries: &summaries,
        details: &details,
    })
    .map_err(super::ContractError::SafeReceiptMetadata)?;
    Ok(DiscoverySet {
        query_label,
        summaries,
        details,
        ceilings: draft.ceilings,
        discovery_identity,
        receipt,
    })
}

/// Tool-plan producer kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlanFabricKind {
    /// Steel-backed planning fabric.
    Steel,
    /// Typed non-Steel planning fabric.
    Typed,
}

impl PlanFabricKind {
    /// Returns the stable fabric label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Steel => FABRIC_KIND_STEEL,
            Self::Typed => FABRIC_KIND_TYPED,
        }
    }
}

/// Fabric admission status before discovery or tool-plan output can influence runtime.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlanFabricStatus {
    /// Fabric may produce declarative plan data.
    Admitted,
    /// Fabric is denied before discovery or plan output.
    Denied,
}

/// Fabric admission issue.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlanFabricIssue {
    /// No issue.
    None,
    /// Fabric kind was unsupported.
    UnsupportedKind,
    /// Required profile identity was missing.
    MissingProfile,
    /// Required policy hash was missing.
    MissingPolicyHash,
    /// Required script hash was missing for a Steel fabric.
    MissingScriptHash,
    /// Required budget label was missing.
    MissingBudget,
    /// No allowed host actions were supplied.
    MissingHostAction,
    /// Host action allowlist contained an unsupported action.
    UnsupportedHostAction,
    /// Receipt posture was not the safe receipt posture.
    UnsafeReceiptPosture,
}

impl PlanFabricIssue {
    /// Returns the stable issue label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::UnsupportedKind => "unsupported_kind",
            Self::MissingProfile => "missing_profile",
            Self::MissingPolicyHash => "missing_policy_hash",
            Self::MissingScriptHash => "missing_script_hash",
            Self::MissingBudget => "missing_budget",
            Self::MissingHostAction => "missing_host_action",
            Self::UnsupportedHostAction => "unsupported_host_action",
            Self::UnsafeReceiptPosture => "unsafe_receipt_posture",
        }
    }
}

/// Borrowed fabric admission inputs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlanFabricDraft<'a> {
    /// Fabric kind label: `steel` or `typed`.
    pub kind: Option<&'a str>,
    /// Profile identity selected by host policy.
    pub profile_identity: Option<&'a str>,
    /// Policy hash selected by host policy.
    pub policy_hash: Option<&'a str>,
    /// Script hash for Steel fabrics.
    pub script_hash: Option<&'a str>,
    /// Explicit host-action allowlist.
    pub allowed_host_actions: &'a [&'a str],
    /// Budget label selected by policy.
    pub budget_label: Option<&'a str>,
    /// Receipt posture label.
    pub receipt_posture: Option<&'a str>,
}

/// Fabric admission result with safe metadata only.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlanFabricAdmission {
    status: PlanFabricStatus,
    issue: PlanFabricIssue,
    kind: Option<PlanFabricKind>,
    profile_identity: Option<Text>,
    policy_hash: Option<Text>,
    script_hash: Option<Text>,
    allowed_host_actions: Collection<Text>,
    budget_label: Option<Text>,
    receipt_posture: Option<Text>,
}

impl PlanFabricAdmission {
    /// Returns the fabric admission status.
    #[must_use]
    pub const fn status(&self) -> PlanFabricStatus {
        self.status
    }

    /// Returns the first admission issue.
    #[must_use]
    pub const fn issue(&self) -> PlanFabricIssue {
        self.issue
    }

    /// Returns true when the fabric is admitted.
    #[must_use]
    pub const fn is_admitted(&self) -> bool {
        matches!(self.status, PlanFabricStatus::Admitted)
    }

    /// Returns the parsed fabric kind, if supported.
    #[must_use]
    pub const fn kind(&self) -> Option<PlanFabricKind> {
        self.kind
    }

    /// Returns the admitted profile identity.
    #[must_use]
    pub fn profile_identity(&self) -> Option<&str> {
        self.profile_identity.as_deref()
    }

    /// Returns the admitted policy hash.
    #[must_use]
    pub fn policy_hash(&self) -> Option<&str> {
        self.policy_hash.as_deref()
    }

    /// Returns the admitted script hash, when present.
    #[must_use]
    pub fn script_hash(&self) -> Option<&str> {
        self.script_hash.as_deref()
    }

    /// Returns admitted host actions.
    #[must_use]
    pub fn allowed_host_actions(&self) -> &[Text] {
        self.allowed_host_actions.as_slice()
    }

    /// Returns the admitted budget label.
    #[must_use]
    pub fn budget_label(&self) -> Option<&str> {
        self.budget_label.as_deref()
    }

    /// Returns receipt posture.
    #[must_use]
    pub fn receipt_posture(&self) -> Option<&str> {
        self.receipt_posture.as_deref()
    }
}

/// Admits or denies a tool-plan fabric without evaluating scripts or host code.
///
/// # Errors
///
/// Returns [`super::ContractError`] when supplied labels are malformed.
///
/// # Panics
///
/// Panics if the admitted host-action list exceeds bounds after collection, or if fabric status
/// and issue classification diverge after pure admission.
pub fn admit_plan_fabric(
    draft: &PlanFabricDraft<'_>,
) -> Result<PlanFabricAdmission, super::ContractError> {
    let kind_label = optional_text(PLAN_FABRIC_KIND_FIELD, draft.kind)?;
    let kind = kind_label.as_deref().and_then(parse_fabric_kind);
    let profile = optional_text(PLAN_FABRIC_PROFILE_FIELD, draft.profile_identity)?;
    let policy_hash = optional_text(PLAN_FABRIC_POLICY_FIELD, draft.policy_hash)?;
    let script_hash = optional_text(PLAN_FABRIC_SCRIPT_FIELD, draft.script_hash)?;
    let actions = collect_optional_labels(PLAN_FABRIC_ACTION_FIELD, draft.allowed_host_actions)?;
    let budget = optional_text(PLAN_FABRIC_BUDGET_FIELD, draft.budget_label)?;
    let receipt = optional_text(PLAN_FABRIC_RECEIPT_FIELD, draft.receipt_posture)?;
    let issue = fabric_issue(&FabricIssueInput {
        kind,
        profile: profile.as_deref(),
        policy_hash: policy_hash.as_deref(),
        script_hash: script_hash.as_deref(),
        actions: &actions,
        budget: budget.as_deref(),
        receipt: receipt.as_deref(),
    });
    let status = if issue == PlanFabricIssue::None {
        PlanFabricStatus::Admitted
    } else {
        PlanFabricStatus::Denied
    };
    assert!(actions.len() <= MAX_DISCOVERY_ITEMS);
    assert_eq!(
        status == PlanFabricStatus::Admitted,
        issue == PlanFabricIssue::None
    );
    Ok(PlanFabricAdmission {
        status,
        issue,
        kind,
        profile_identity: profile,
        policy_hash,
        script_hash,
        allowed_host_actions: actions,
        budget_label: budget,
        receipt_posture: receipt,
    })
}

/// Declarative batch policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlanBatchPolicy {
    /// Steps are serialized by the runtime shell.
    Serial,
    /// Steps may batch only when their mutation targets do not conflict.
    ParallelSafe,
}

impl PlanBatchPolicy {
    /// Returns the stable policy label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Serial => BATCH_POLICY_SERIAL,
            Self::ParallelSafe => BATCH_POLICY_PARALLEL_SAFE,
        }
    }
}

/// Cancellation posture for a planned step before dispatch.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlanCancellationPosture {
    /// No cancellation was requested.
    Continue,
    /// Caller-owned cancellation stopped the step before dispatch.
    Cancelled,
}

/// Timeout posture for a planned step before dispatch.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlanTimeoutPosture {
    /// Step remains within the caller-owned deadline.
    WithinDeadline,
    /// Caller-owned deadline expired before dispatch.
    TimedOut,
}

/// Intermediate value class for a step binding.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlanIntermediateKind {
    /// Bounded model-visible text.
    BoundedText,
    /// Host-owned safe handle.
    SafeHandle,
    /// BLAKE3 or schema-owned hash.
    Hash,
    /// Bounded count value.
    Count,
    /// Binary payload class.
    Binary,
    /// Image payload class.
    Image,
    /// Secret or credential-adjacent class.
    Secret,
    /// Oversized payload class.
    Oversized,
    /// Unsupported payload class.
    Unsupported,
}

/// How an intermediate binding is used by a later step.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlanIntermediateUse {
    /// Bind as text argument.
    ArgumentText,
    /// Bind as host-owned handle reference.
    HandleReference,
    /// Record as receipt metadata only.
    ReceiptMetadata,
}

/// Borrowed intermediate binding draft.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlanIntermediateBindingDraft<'a> {
    /// Stable binding identifier.
    pub binding_id: Option<&'a str>,
    /// Source step identifier.
    pub source_step_id: Option<&'a str>,
    /// Target argument name.
    pub target_argument: Option<&'a str>,
    /// Intermediate value class.
    pub value_kind: PlanIntermediateKind,
    /// Intended use class.
    pub use_kind: PlanIntermediateUse,
    /// Bounded character count for text-like material.
    pub char_count: u32,
    /// Optional payload or handle hash.
    pub content_hash: Option<&'a str>,
}

/// Validated intermediate binding with no raw payload.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlanIntermediateBinding {
    binding_id: Text,
    source_step_id: Text,
    target_argument: Text,
    value_kind: PlanIntermediateKind,
    use_kind: PlanIntermediateUse,
    char_count: u32,
    content_hash: Option<Text>,
}

impl PlanIntermediateBinding {
    /// Returns the binding identifier.
    #[must_use]
    pub fn binding_id(&self) -> &str {
        self.binding_id.as_str()
    }

    /// Returns the source step identifier.
    #[must_use]
    pub fn source_step_id(&self) -> &str {
        self.source_step_id.as_str()
    }

    /// Returns the target argument name.
    #[must_use]
    pub fn target_argument(&self) -> &str {
        self.target_argument.as_str()
    }

    /// Returns the intermediate value class.
    #[must_use]
    pub const fn value_kind(&self) -> PlanIntermediateKind {
        self.value_kind
    }

    /// Returns the binding use class.
    #[must_use]
    pub const fn use_kind(&self) -> PlanIntermediateUse {
        self.use_kind
    }

    /// Returns the bounded character count.
    #[must_use]
    pub const fn char_count(&self) -> u32 {
        self.char_count
    }

    /// Returns the optional content or handle hash.
    #[must_use]
    pub fn content_hash(&self) -> Option<&str> {
        self.content_hash.as_deref()
    }
}

/// Borrowed tool-plan step draft.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlanStepDraft<'a> {
    /// Stable step identifier.
    pub step_id: Option<&'a str>,
    /// Current-shape tool request.
    pub request: &'a super::Request,
    /// Cancellation posture before step dispatch.
    pub cancellation_posture: PlanCancellationPosture,
    /// Timeout posture before step dispatch.
    pub timeout_posture: PlanTimeoutPosture,
    /// Prior step identifiers required by this step.
    pub depends_on: &'a [&'a str],
    /// Bounded intermediate bindings used by this step.
    pub intermediate_bindings: &'a [PlanIntermediateBindingDraft<'a>],
}

/// Borrowed declarative tool-plan draft.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlanDraft<'a> {
    /// Stable batch identifier.
    pub batch_id: Option<&'a str>,
    /// Fabric admission metadata.
    pub fabric: PlanFabricDraft<'a>,
    /// Batch execution policy label.
    pub batch_policy: &'a str,
    /// Ordered plan steps.
    pub steps: &'a [PlanStepDraft<'a>],
}

/// Inputs needed to admit a tool plan against discovery and catalog data.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlanAdmissionContext<'a> {
    /// Discovery result visible to the planner.
    pub discovery: &'a DiscoverySet,
    /// Already-loaded active catalog or manifest definitions.
    pub definitions: &'a [super::Definition],
}

/// Tool-plan admission status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlanStatus {
    /// All steps are admitted.
    Admitted,
    /// One or more steps are denied.
    Denied,
}

/// Tool-plan or step admission issue.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlanIssue {
    /// No issue.
    None,
    /// Fabric admission denied the plan.
    FabricDenied,
    /// Step identifier was duplicated.
    DuplicateStep,
    /// Dependency referenced an unknown or future step.
    UnknownDependency,
    /// Tool was not in the admitted discovery set.
    UndiscoveredTool,
    /// Tool definition was missing.
    UnknownTool,
    /// Tool definition was inactive.
    InactiveTool,
    /// Tool definition was untrusted.
    UntrustedTool,
    /// Request schema did not match the definition.
    SchemaMismatch,
    /// Required capability labels were absent from the exact definition.
    CapabilityDenied,
    /// Request approval posture denied execution.
    ApprovalDenied,
    /// Caller-owned cancellation stopped dispatch.
    Cancelled,
    /// Caller-owned deadline expired before dispatch.
    TimedOut,
    /// Intermediate binding was unsafe for its requested use.
    UnsafeIntermediate,
    /// Mutation targets conflict under the selected batch policy.
    MutationConflict,
}

impl PlanIssue {
    /// Returns the stable issue label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::FabricDenied => "fabric_denied",
            Self::DuplicateStep => "duplicate_step",
            Self::UnknownDependency => "unknown_dependency",
            Self::UndiscoveredTool => "undiscovered_tool",
            Self::UnknownTool => "unknown_tool",
            Self::InactiveTool => "inactive_tool",
            Self::UntrustedTool => "untrusted_tool",
            Self::SchemaMismatch => "schema_mismatch",
            Self::CapabilityDenied => "capability_denied",
            Self::ApprovalDenied => "approval_denied",
            Self::Cancelled => "cancelled",
            Self::TimedOut => "timed_out",
            Self::UnsafeIntermediate => "unsafe_intermediate",
            Self::MutationConflict => "mutation_conflict",
        }
    }
}

/// Per-step admission result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlanStepAdmission {
    step_id: Text,
    tool_name: Text,
    status: PlanStatus,
    issue: PlanIssue,
    schema_identity: Text,
    required_capability_labels: Collection<Text>,
    mutation_targets: Collection<Text>,
    cancellation_posture: PlanCancellationPosture,
    timeout_posture: PlanTimeoutPosture,
    dependencies: Collection<Text>,
    intermediate_bindings: Collection<PlanIntermediateBinding>,
}

impl PlanStepAdmission {
    /// Returns the step identifier.
    #[must_use]
    pub fn step_id(&self) -> &str {
        self.step_id.as_str()
    }

    /// Returns the requested tool name.
    #[must_use]
    pub fn tool_name(&self) -> &str {
        self.tool_name.as_str()
    }

    /// Returns step admission status.
    #[must_use]
    pub const fn status(&self) -> PlanStatus {
        self.status
    }

    /// Returns first step issue.
    #[must_use]
    pub const fn issue(&self) -> PlanIssue {
        self.issue
    }

    /// Returns request schema identity.
    #[must_use]
    pub fn schema_identity(&self) -> &str {
        self.schema_identity.as_str()
    }

    /// Returns required capability labels.
    #[must_use]
    pub fn required_capability_labels(&self) -> &[Text] {
        self.required_capability_labels.as_slice()
    }

    /// Returns mutation targets declared by the request.
    #[must_use]
    pub fn mutation_targets(&self) -> &[Text] {
        self.mutation_targets.as_slice()
    }

    /// Returns cancellation posture.
    #[must_use]
    pub const fn cancellation_posture(&self) -> PlanCancellationPosture {
        self.cancellation_posture
    }

    /// Returns timeout posture.
    #[must_use]
    pub const fn timeout_posture(&self) -> PlanTimeoutPosture {
        self.timeout_posture
    }

    /// Returns dependency step IDs.
    #[must_use]
    pub fn dependencies(&self) -> &[Text] {
        self.dependencies.as_slice()
    }

    /// Returns bounded intermediate bindings.
    #[must_use]
    pub fn intermediate_bindings(&self) -> &[PlanIntermediateBinding] {
        self.intermediate_bindings.as_slice()
    }
}

/// Admitted or denied declarative tool plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Plan {
    batch_id: Text,
    fabric: PlanFabricAdmission,
    batch_policy: PlanBatchPolicy,
    status: PlanStatus,
    issue: PlanIssue,
    steps: Collection<PlanStepAdmission>,
    identity: Text,
}

impl Plan {
    /// Returns the batch identifier.
    #[must_use]
    pub fn batch_id(&self) -> &str {
        self.batch_id.as_str()
    }

    /// Returns fabric admission metadata.
    #[must_use]
    pub const fn fabric(&self) -> &PlanFabricAdmission {
        &self.fabric
    }

    /// Returns the batch policy.
    #[must_use]
    pub const fn batch_policy(&self) -> PlanBatchPolicy {
        self.batch_policy
    }

    /// Returns overall plan admission status.
    #[must_use]
    pub const fn status(&self) -> PlanStatus {
        self.status
    }

    /// Returns first plan issue.
    #[must_use]
    pub const fn issue(&self) -> PlanIssue {
        self.issue
    }

    /// Returns admitted and denied step records.
    #[must_use]
    pub fn steps(&self) -> &[PlanStepAdmission] {
        self.steps.as_slice()
    }

    /// Returns a BLAKE3 identity over safe plan metadata.
    #[must_use]
    pub fn plan_identity(&self) -> &str {
        self.identity.as_str()
    }
}

/// Admits a declarative tool plan without dispatching tools or evaluating code.
///
/// # Errors
///
/// Returns [`super::ContractError`] when plan metadata is malformed or exceeds
/// bounded limits.
///
/// # Panics
///
/// Panics if the validated plan step count drifts outside the admitted non-empty bounded range.
/// This indicates an internal validation contract violation after admission.
pub fn admit_plan(
    draft: &PlanDraft<'_>,
    context: PlanAdmissionContext<'_>,
) -> Result<Plan, super::ContractError> {
    validate_plan_inputs(draft, context)?;
    assert!(draft.steps.len() >= MIN_TOOL_PLANNING_ITEMS);
    assert!(draft.steps.len() <= MAX_PLAN_STEPS);
    let batch_id = super::required_text(PLAN_BATCH_FIELD, draft.batch_id)?;
    let fabric = admit_plan_fabric(&draft.fabric)?;
    let batch_policy = parse_batch_policy(draft.batch_policy)?;
    let mut prior_steps = Collection::with_capacity(draft.steps.len());
    let mut step_results = Collection::with_capacity(draft.steps.len());
    for step in draft.steps {
        let step_result = admit_plan_step(&PlanStepAdmissionInput {
            step,
            context,
            fabric: &fabric,
            batch_policy,
            prior_steps: &prior_steps,
        })?;
        prior_steps.push(step_result.clone());
        step_results.push(step_result);
    }
    let issue = first_plan_issue(&step_results);
    let status = status_for_issue(issue);
    let identity = plan_identity(&batch_id, &fabric, batch_policy, &step_results);
    Ok(Plan {
        batch_id,
        fabric,
        batch_policy,
        status,
        issue,
        steps: step_results,
        identity,
    })
}

/// Borrowed safe receipt draft for a tool plan.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlanReceiptDraft<'a> {
    /// Admitted or denied plan.
    pub plan: &'a Plan,
    /// Safe policy hash label.
    pub policy_hash: &'a str,
    /// Safe diagnostic labels.
    pub diagnostics: &'a [&'a str],
}

/// Safe tool-plan receipt wrapper.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlanReceipt {
    plan_identity: Text,
    receipt_hash: Text,
    record: crate::SafeReceipt,
}

impl PlanReceipt {
    /// Builds a safe receipt for a tool plan.
    ///
    /// # Errors
    ///
    /// Returns [`crate::SafeReceiptError`] when labels, diagnostics, or receipt
    /// metadata contain raw prompts, raw tool bodies, provider payloads,
    /// credentials, full paths, or other unsafe material.
    pub fn new(draft: &PlanReceiptDraft<'_>) -> Result<Self, crate::SafeReceiptError> {
        let counts = [
            crate::BoundedCountDraft::new(COUNT_STEPS, usize_to_u32(draft.plan.steps().len())),
            crate::BoundedCountDraft::new(COUNT_BINDINGS, usize_to_u32(binding_count(draft.plan))),
        ];
        let labels = receipt_labels(draft);
        let redactions = [
            crate::ReceiptRedaction::PromptOmitted,
            crate::ReceiptRedaction::ProviderPayloadOmitted,
            crate::ReceiptRedaction::CredentialOmitted,
            crate::ReceiptRedaction::ToolInputOmitted,
            crate::ReceiptRedaction::ToolOutputOmitted,
            crate::ReceiptRedaction::OutputBounded,
            crate::ReceiptRedaction::PayloadHashOnly,
        ];
        let status = receipt_status(draft.plan.status());
        let issue = receipt_issue(draft.plan.issue());
        let record = crate::safe::receipt::Record::new(
            crate::safe::receipt::Draft::new(status, issue, crate::ReceiptOperation::ToolExecution)
                .with_labels(&labels)
                .with_counts(&counts)
                .with_backend_label(PLAN_RECEIPT_BACKEND)
                .with_interchange_label(RECEIPT_INTERCHANGE)
                .with_redactions(&redactions),
        )?;
        let summary = record.review_summary();
        let receipt_hash = super::hash_parts(&[
            TOOL_PLAN_RECEIPT_CONTRACT_IDENTITY,
            draft.plan.plan_identity(),
            summary.as_str(),
        ]);
        Ok(Self {
            plan_identity: Text::from(draft.plan.plan_identity()),
            receipt_hash,
            record,
        })
    }

    /// Returns the plan identity recorded by the receipt.
    #[must_use]
    pub fn plan_identity(&self) -> &str {
        self.plan_identity.as_str()
    }

    /// Returns the BLAKE3 receipt hash.
    #[must_use]
    pub fn receipt_hash(&self) -> &str {
        self.receipt_hash.as_str()
    }

    /// Returns the underlying safe receipt record.
    #[must_use]
    pub const fn record(&self) -> &crate::SafeReceipt {
        &self.record
    }
}

#[derive(Clone, Copy)]
struct BoundCheck {
    field: crate::validation::FieldName,
    value: usize,
    max_items: usize,
}

#[derive(Clone, Copy)]
struct MaxCountCheck {
    field: crate::validation::FieldName,
    actual_items: usize,
    max_items: usize,
}

fn validate_bound(check: BoundCheck) -> Result<(), super::ContractError> {
    if (MIN_TOOL_PLANNING_ITEMS..=check.max_items).contains(&check.value) {
        Ok(())
    } else {
        Err(super::ContractError::InvalidPlanningBound {
            field: check.field.as_str(),
            min_items: MIN_TOOL_PLANNING_ITEMS,
            max_items: check.max_items,
            actual_items: check.value,
        })
    }
}

fn validate_max_count(check: MaxCountCheck) -> Result<(), super::ContractError> {
    if check.actual_items <= check.max_items {
        Ok(())
    } else {
        Err(super::ContractError::TooManyItems {
            field: check.field.as_str(),
            max_items: check.max_items,
            actual_items: check.actual_items,
        })
    }
}

fn collect_optional_labels(
    field: crate::validation::FieldName,
    values: &[&str],
) -> Result<Collection<Text>, super::ContractError> {
    validate_max_count(MaxCountCheck {
        field,
        actual_items: values.len(),
        max_items: MAX_DISCOVERY_ITEMS,
    })?;
    let mut labels = Collection::with_capacity(values.len());
    for value in values {
        labels.push(super::required_text_value(field, value)?);
    }
    labels.sort();
    labels.dedup();
    Ok(labels)
}

fn collect_unique_labels(
    field: crate::validation::FieldName,
    values: &[&str],
) -> Result<Collection<Text>, super::ContractError> {
    let labels = collect_optional_labels(field, values)?;
    if labels.len() == values.len() {
        return Ok(labels);
    }
    Err(super::ContractError::DuplicatePlanningReference {
        field: field.as_str(),
    })
}

fn optional_text(
    field: crate::validation::FieldName,
    value: Option<&str>,
) -> Result<Option<Text>, super::ContractError> {
    value
        .map(|raw| super::required_text_value(field, raw))
        .transpose()
}

fn discovery_summaries(
    definitions: &[super::Definition],
    capability_filter: &[Text],
    ceilings: DiscoveryCeilings,
) -> Collection<DiscoverySummary> {
    let mut summaries = Collection::new();
    for definition in definitions {
        if !definition_is_discoverable(definition, capability_filter) {
            continue;
        }
        if summaries.len() >= ceilings.max_summaries {
            break;
        }
        summaries.push(summary_from_definition(definition));
    }
    summaries
}

fn discovery_details(
    summaries: &[DiscoverySummary],
    definitions: &[super::Definition],
    detail_names: &[Text],
) -> Result<Collection<DiscoveryDetail>, super::ContractError> {
    let mut details = Collection::with_capacity(detail_names.len());
    assert_eq!(details.len(), 0);
    assert!(details.capacity() >= detail_names.len());
    for detail_name in detail_names {
        let summary = summaries
            .iter()
            .find(|candidate| candidate.tool_name() == detail_name)
            .ok_or_else(|| super::ContractError::UnknownPlanningReference {
                field: DETAIL_TOOL_FIELD.as_str(),
                reference: detail_name.clone(),
            })?;
        let definition = find_definition(definitions, detail_name).ok_or_else(|| {
            super::ContractError::UnknownPlanningReference {
                field: DETAIL_TOOL_FIELD.as_str(),
                reference: detail_name.clone(),
            }
        })?;
        details.push(detail_from_definition(summary, definition));
    }
    Ok(details)
}

fn definition_is_discoverable(definition: &super::Definition, capability_filter: &[Text]) -> bool {
    definition.status() == super::DefinitionStatus::Active
        && capability_filter.iter().all(|label| {
            definition
                .capability_labels()
                .iter()
                .any(|current| current == label)
        })
}

fn summary_from_definition(definition: &super::Definition) -> DiscoverySummary {
    DiscoverySummary {
        tool_name: Text::from(definition.name()),
        label: Text::from(definition.label()),
        schema_identity: Text::from(definition.parameter_schema_identity()),
        capability_labels: definition.capability_labels().to_vec(),
        result_contract: Text::from(definition.result_metadata_shape()),
        provenance: Text::from(definition.provenance()),
    }
}

fn detail_from_definition(
    summary: &DiscoverySummary,
    definition: &super::Definition,
) -> DiscoveryDetail {
    DiscoveryDetail {
        summary: summary.clone(),
        description: Text::from(definition.description()),
        source_identity: Text::from(definition.source_identity()),
    }
}

fn discovery_identity(
    query_label: &str,
    summaries: &[DiscoverySummary],
    details: &[DiscoveryDetail],
) -> Text {
    let mut parts = Collection::with_capacity(
        summaries
            .len()
            .saturating_add(details.len())
            .saturating_add(MIN_TOOL_PLANNING_ITEMS),
    );
    parts.push(query_label);
    for summary in summaries {
        parts.push(summary.tool_name());
        parts.push(summary.schema_identity());
        parts.push(summary.result_contract());
    }
    for detail in details {
        parts.push(detail.summary().tool_name());
        parts.push(detail.source_identity());
    }
    super::hash_parts(&parts)
}

#[derive(Clone, Copy)]
struct DiscoveryReceiptInput<'a> {
    query_label: &'a str,
    discovery_identity: &'a str,
    summaries: &'a [DiscoverySummary],
    details: &'a [DiscoveryDetail],
}

fn discovery_receipt(
    input: DiscoveryReceiptInput<'_>,
) -> Result<crate::SafeReceipt, crate::SafeReceiptError> {
    assert!(input.summaries.len() <= MAX_DISCOVERY_ITEMS);
    assert!(input.details.len() <= MAX_DISCOVERY_ITEMS);
    let counts = [
        crate::BoundedCountDraft::new(
            COUNT_DISCOVERY_SUMMARIES,
            usize_to_u32(input.summaries.len()),
        ),
        crate::BoundedCountDraft::new(COUNT_DISCOVERY_DETAILS, usize_to_u32(input.details.len())),
    ];
    let labels = [
        TOOL_DISCOVERY_CONTRACT_IDENTITY,
        input.query_label,
        input.discovery_identity,
    ];
    let redactions = [
        crate::ReceiptRedaction::ToolInputOmitted,
        crate::ReceiptRedaction::ToolOutputOmitted,
        crate::ReceiptRedaction::PayloadHashOnly,
    ];
    crate::safe::receipt::Record::new(
        crate::safe::receipt::Draft::new(
            crate::ReceiptStatus::Completed,
            crate::ReceiptIssueCode::ProvenanceObservation,
            crate::ReceiptOperation::ProvenanceObservation,
        )
        .with_labels(&labels)
        .with_counts(&counts)
        .with_backend_label(DISCOVERY_RECEIPT_BACKEND)
        .with_interchange_label(RECEIPT_INTERCHANGE)
        .with_redactions(&redactions),
    )
}

fn parse_fabric_kind(value: &str) -> Option<PlanFabricKind> {
    match value {
        FABRIC_KIND_STEEL => Some(PlanFabricKind::Steel),
        FABRIC_KIND_TYPED => Some(PlanFabricKind::Typed),
        _unsupported => None,
    }
}

struct FabricIssueInput<'a> {
    kind: Option<PlanFabricKind>,
    profile: Option<&'a str>,
    policy_hash: Option<&'a str>,
    script_hash: Option<&'a str>,
    actions: &'a [Text],
    budget: Option<&'a str>,
    receipt: Option<&'a str>,
}

fn fabric_issue(input: &FabricIssueInput<'_>) -> PlanFabricIssue {
    assert!(input.actions.len() <= MAX_DISCOVERY_ITEMS);
    assert!(input.actions.iter().all(|action| !action.is_empty()));
    let Some(kind) = input.kind else {
        return PlanFabricIssue::UnsupportedKind;
    };
    if input.profile.is_none() {
        return PlanFabricIssue::MissingProfile;
    }
    if input.policy_hash.is_none() {
        return PlanFabricIssue::MissingPolicyHash;
    }
    if matches!(kind, PlanFabricKind::Steel) && input.script_hash.is_none() {
        return PlanFabricIssue::MissingScriptHash;
    }
    if input.budget.is_none() {
        return PlanFabricIssue::MissingBudget;
    }
    if input.actions.is_empty() {
        return PlanFabricIssue::MissingHostAction;
    }
    if input
        .actions
        .iter()
        .any(|action| !is_allowed_fabric_action(action))
    {
        return PlanFabricIssue::UnsupportedHostAction;
    }
    if input.receipt != Some(RECEIPT_POSTURE_SAFE) {
        return PlanFabricIssue::UnsafeReceiptPosture;
    }
    PlanFabricIssue::None
}

fn is_allowed_fabric_action(action: &str) -> bool {
    matches!(action, FABRIC_ACTION_DISCOVERY | FABRIC_ACTION_EMIT_PLAN)
}

fn parse_batch_policy(value: &str) -> Result<PlanBatchPolicy, super::ContractError> {
    let normalized = super::required_text_value(PLAN_BATCH_FIELD, value)?;
    match normalized.as_str() {
        BATCH_POLICY_SERIAL => Ok(PlanBatchPolicy::Serial),
        BATCH_POLICY_PARALLEL_SAFE => Ok(PlanBatchPolicy::ParallelSafe),
        _unsupported => Err(super::ContractError::UnsupportedPlanningValue {
            field: PLAN_BATCH_FIELD.as_str(),
            value: normalized,
        }),
    }
}

fn validate_plan_inputs(
    draft: &PlanDraft<'_>,
    context: PlanAdmissionContext<'_>,
) -> Result<(), super::ContractError> {
    validate_bound(BoundCheck {
        field: PLAN_STEP_FIELD,
        value: draft.steps.len(),
        max_items: MAX_PLAN_STEPS,
    })?;
    validate_max_count(MaxCountCheck {
        field: PLAN_STEP_FIELD,
        actual_items: context.definitions.len(),
        max_items: MAX_DISCOVERY_ITEMS,
    })?;
    Ok(())
}

struct PlanStepAdmissionInput<'a> {
    step: &'a PlanStepDraft<'a>,
    context: PlanAdmissionContext<'a>,
    fabric: &'a PlanFabricAdmission,
    batch_policy: PlanBatchPolicy,
    prior_steps: &'a [PlanStepAdmission],
}

fn admit_plan_step(
    input: &PlanStepAdmissionInput<'_>,
) -> Result<PlanStepAdmission, super::ContractError> {
    let step = input.step;
    validate_max_count(MaxCountCheck {
        field: PLAN_DEPENDENCY_FIELD,
        actual_items: step.depends_on.len(),
        max_items: MAX_PLAN_STEPS,
    })?;
    validate_max_count(MaxCountCheck {
        field: PLAN_BINDING_FIELD,
        actual_items: step.intermediate_bindings.len(),
        max_items: MAX_PLAN_BINDINGS,
    })?;
    assert!(step.depends_on.len() <= MAX_PLAN_STEPS);
    assert!(step.intermediate_bindings.len() <= MAX_PLAN_BINDINGS);
    let step_id = super::required_text(PLAN_STEP_FIELD, step.step_id)?;
    let dependencies = collect_unique_labels(PLAN_DEPENDENCY_FIELD, step.depends_on)?;
    let bindings = validate_bindings(step.intermediate_bindings, input.prior_steps)?;
    let definition = find_definition(input.context.definitions, step.request.name());
    let issue = step_issue(&StepIssueInput {
        step,
        step_id: step_id.as_str(),
        request: step.request,
        context: input.context,
        fabric: input.fabric,
        batch_policy: input.batch_policy,
        prior_steps: input.prior_steps,
        dependencies: &dependencies,
        bindings: &bindings,
        definition,
    });
    let status = status_for_issue(issue);
    Ok(PlanStepAdmission {
        step_id,
        tool_name: Text::from(step.request.name()),
        status,
        issue,
        schema_identity: Text::from(step.request.schema_identity()),
        required_capability_labels: step.request.required_capability_labels().to_vec(),
        mutation_targets: step.request.mutation_targets().to_vec(),
        cancellation_posture: step.cancellation_posture,
        timeout_posture: step.timeout_posture,
        dependencies,
        intermediate_bindings: bindings,
    })
}

struct StepIssueInput<'a> {
    step_id: &'a str,
    step: &'a PlanStepDraft<'a>,
    request: &'a super::Request,
    context: PlanAdmissionContext<'a>,
    fabric: &'a PlanFabricAdmission,
    batch_policy: PlanBatchPolicy,
    prior_steps: &'a [PlanStepAdmission],
    dependencies: &'a [Text],
    bindings: &'a [PlanIntermediateBinding],
    definition: Option<&'a super::Definition>,
}

fn step_issue(input: &StepIssueInput<'_>) -> PlanIssue {
    duplicate_step_issue(input.step_id, input.prior_steps)
        .or_else(|| fabric_issue_for_plan(input.fabric))
        .or_else(|| dependency_issue(input.dependencies, input.prior_steps))
        .or_else(|| discovery_issue(input.context.discovery, input.request))
        .or_else(|| definition_issue(input.definition))
        .or_else(|| schema_issue(input.request, input.definition))
        .or_else(|| capability_issue(input.request, input.definition))
        .or_else(|| approval_issue(input.request))
        .or_else(|| cancellation_issue(input.step))
        .or_else(|| timeout_issue(input.step))
        .or_else(|| intermediate_issue(input.bindings))
        .or_else(|| mutation_issue(input.batch_policy, input.request, input.prior_steps))
        .unwrap_or(PlanIssue::None)
}

fn duplicate_step_issue(step_id: &str, prior_steps: &[PlanStepAdmission]) -> Option<PlanIssue> {
    prior_steps
        .iter()
        .any(|prior| prior.step_id() == step_id)
        .then_some(PlanIssue::DuplicateStep)
}

fn fabric_issue_for_plan(fabric: &PlanFabricAdmission) -> Option<PlanIssue> {
    (!fabric.is_admitted()).then_some(PlanIssue::FabricDenied)
}

fn dependency_issue(dependencies: &[Text], prior_steps: &[PlanStepAdmission]) -> Option<PlanIssue> {
    dependencies
        .iter()
        .any(|dependency| {
            !prior_steps
                .iter()
                .any(|prior| prior.step_id() == dependency)
        })
        .then_some(PlanIssue::UnknownDependency)
}

fn discovery_issue(discovery: &DiscoverySet, request: &super::Request) -> Option<PlanIssue> {
    (!discovery.contains_tool(request.name())).then_some(PlanIssue::UndiscoveredTool)
}

fn definition_issue(definition: Option<&super::Definition>) -> Option<PlanIssue> {
    match definition.map(super::Definition::status) {
        None => Some(PlanIssue::UnknownTool),
        Some(super::DefinitionStatus::Inactive) => Some(PlanIssue::InactiveTool),
        Some(super::DefinitionStatus::Untrusted) => Some(PlanIssue::UntrustedTool),
        Some(super::DefinitionStatus::Active) => None,
    }
}

fn schema_issue(
    request: &super::Request,
    definition: Option<&super::Definition>,
) -> Option<PlanIssue> {
    definition
        .filter(|current| current.parameter_schema_identity() != request.schema_identity())
        .map(|_current| PlanIssue::SchemaMismatch)
}

fn capability_issue(
    request: &super::Request,
    definition: Option<&super::Definition>,
) -> Option<PlanIssue> {
    definition
        .filter(|current| {
            request
                .required_capability_labels()
                .iter()
                .any(|label| !current.capability_labels().contains(label))
        })
        .map(|_current| PlanIssue::CapabilityDenied)
}

fn approval_issue(request: &super::Request) -> Option<PlanIssue> {
    matches!(request.approval_posture(), super::ApprovalPosture::Denied)
        .then_some(PlanIssue::ApprovalDenied)
}

fn cancellation_issue(step: &PlanStepDraft<'_>) -> Option<PlanIssue> {
    matches!(
        step.cancellation_posture,
        PlanCancellationPosture::Cancelled
    )
    .then_some(PlanIssue::Cancelled)
}

fn timeout_issue(step: &PlanStepDraft<'_>) -> Option<PlanIssue> {
    matches!(step.timeout_posture, PlanTimeoutPosture::TimedOut).then_some(PlanIssue::TimedOut)
}

fn intermediate_issue(bindings: &[PlanIntermediateBinding]) -> Option<PlanIssue> {
    bindings
        .iter()
        .any(|binding| !binding_is_safe(binding))
        .then_some(PlanIssue::UnsafeIntermediate)
}

fn mutation_issue(
    batch_policy: PlanBatchPolicy,
    request: &super::Request,
    prior_steps: &[PlanStepAdmission],
) -> Option<PlanIssue> {
    if matches!(batch_policy, PlanBatchPolicy::Serial) {
        return None;
    }
    request
        .mutation_targets()
        .iter()
        .any(|target| {
            prior_steps
                .iter()
                .any(|prior| prior_has_target(prior, target))
        })
        .then_some(PlanIssue::MutationConflict)
}

fn prior_has_target(prior: &PlanStepAdmission, target: &str) -> bool {
    prior
        .mutation_targets()
        .iter()
        .any(|prior_target| prior_target == target)
}

fn validate_bindings(
    drafts: &[PlanIntermediateBindingDraft<'_>],
    prior_steps: &[PlanStepAdmission],
) -> Result<Collection<PlanIntermediateBinding>, super::ContractError> {
    let mut bindings = Collection::with_capacity(drafts.len());
    let mut binding_ids = Collection::with_capacity(drafts.len());
    for draft in drafts {
        let binding = validate_binding(draft, prior_steps)?;
        if binding_ids.iter().any(|seen| seen == binding.binding_id()) {
            return Err(super::ContractError::DuplicatePlanningReference {
                field: PLAN_BINDING_FIELD.as_str(),
            });
        }
        binding_ids.push(Text::from(binding.binding_id()));
        bindings.push(binding);
    }
    Ok(bindings)
}

fn validate_binding(
    draft: &PlanIntermediateBindingDraft<'_>,
    prior_steps: &[PlanStepAdmission],
) -> Result<PlanIntermediateBinding, super::ContractError> {
    let binding_id = super::required_text(PLAN_BINDING_FIELD, draft.binding_id)?;
    let source_step_id = super::required_text(PLAN_DEPENDENCY_FIELD, draft.source_step_id)?;
    assert!(!binding_id.is_empty());
    assert!(!source_step_id.is_empty());
    if !prior_steps
        .iter()
        .any(|prior| prior.step_id() == source_step_id)
    {
        return Err(super::ContractError::UnknownPlanningReference {
            field: PLAN_DEPENDENCY_FIELD.as_str(),
            reference: source_step_id,
        });
    }
    let target_argument = super::required_text(PLAN_TARGET_ARGUMENT_FIELD, draft.target_argument)?;
    validate_intermediate_count(draft.char_count)?;
    let content_hash = optional_text(PLAN_CONTENT_HASH_FIELD, draft.content_hash)?;
    Ok(PlanIntermediateBinding {
        binding_id,
        source_step_id,
        target_argument,
        value_kind: draft.value_kind,
        use_kind: draft.use_kind,
        char_count: draft.char_count,
        content_hash,
    })
}

fn validate_intermediate_count(char_count: u32) -> Result<(), super::ContractError> {
    if (MIN_INTERMEDIATE_CHARS..=MAX_INTERMEDIATE_CHARS).contains(&char_count) {
        Ok(())
    } else {
        Err(super::ContractError::InvalidPlanningBound {
            field: PLAN_BINDING_FIELD.as_str(),
            min_items: MIN_INTERMEDIATE_CHARS_USIZE,
            max_items: MAX_INTERMEDIATE_CHARS_USIZE,
            actual_items: u32_to_usize(char_count)?,
        })
    }
}

const fn binding_is_safe(binding: &PlanIntermediateBinding) -> bool {
    !matches!(
        (binding.value_kind, binding.use_kind),
        (
            PlanIntermediateKind::Secret
                | PlanIntermediateKind::Oversized
                | PlanIntermediateKind::Unsupported,
            _,
        ) | (
            PlanIntermediateKind::Binary
                | PlanIntermediateKind::Image
                | PlanIntermediateKind::SafeHandle,
            PlanIntermediateUse::ArgumentText,
        )
    )
}

fn find_definition<'a>(
    definitions: &'a [super::Definition],
    tool_name: &str,
) -> Option<&'a super::Definition> {
    definitions
        .iter()
        .find(|definition| definition.name() == tool_name)
}

fn status_for_issue(issue: PlanIssue) -> PlanStatus {
    if issue == PlanIssue::None {
        PlanStatus::Admitted
    } else {
        PlanStatus::Denied
    }
}

fn first_plan_issue(steps: &[PlanStepAdmission]) -> PlanIssue {
    steps
        .iter()
        .find_map(|step| (step.issue() != PlanIssue::None).then_some(step.issue()))
        .unwrap_or(PlanIssue::None)
}

fn plan_identity(
    batch_id: &str,
    fabric: &PlanFabricAdmission,
    batch_policy: PlanBatchPolicy,
    steps: &[PlanStepAdmission],
) -> Text {
    let mut parts = Collection::with_capacity(steps.len().saturating_mul(MAX_PLAN_BINDINGS));
    parts.push(TOOL_PLAN_CONTRACT_IDENTITY);
    parts.push(batch_id);
    parts.push(batch_policy.as_str());
    parts.push(fabric.issue().as_str());
    for step in steps {
        parts.push(step.step_id());
        parts.push(step.tool_name());
        parts.push(step.issue().as_str());
        parts.push(step.schema_identity());
    }
    super::hash_parts(&parts)
}

fn receipt_labels<'a>(draft: &'a PlanReceiptDraft<'a>) -> Collection<&'a str> {
    let mut labels =
        Collection::with_capacity(draft.diagnostics.len().saturating_add(MAX_PLAN_BINDINGS));
    labels.push(TOOL_PLAN_RECEIPT_CONTRACT_IDENTITY);
    labels.push(draft.plan.batch_id());
    labels.push(draft.plan.plan_identity());
    labels.push(draft.policy_hash);
    labels.push(draft.plan.issue().as_str());
    for diagnostic in draft.diagnostics {
        labels.push(diagnostic);
    }
    labels
}

fn receipt_status(status: PlanStatus) -> crate::ReceiptStatus {
    match status {
        PlanStatus::Admitted => crate::ReceiptStatus::Completed,
        PlanStatus::Denied => crate::ReceiptStatus::Denied,
    }
}

fn receipt_issue(issue: PlanIssue) -> crate::ReceiptIssueCode {
    if issue == PlanIssue::None {
        crate::ReceiptIssueCode::ToolExecution
    } else {
        crate::ReceiptIssueCode::RuntimeBlocked
    }
}

fn binding_count(plan: &Plan) -> usize {
    plan.steps()
        .iter()
        .map(|step| step.intermediate_bindings().len())
        .sum()
}

fn u32_to_usize(value: u32) -> Result<usize, super::ContractError> {
    usize::try_from(value).map_err(|_error| super::ContractError::InvalidPlanningBound {
        field: PLAN_BINDING_FIELD.as_str(),
        min_items: MIN_INTERMEDIATE_CHARS_USIZE,
        max_items: MAX_INTERMEDIATE_CHARS_USIZE,
        actual_items: MAX_INTERMEDIATE_CHARS_USIZE.saturating_add(MIN_TOOL_PLANNING_ITEMS),
    })
}

fn usize_to_u32(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(crate::safe::receipt::MAX_COUNT)
}

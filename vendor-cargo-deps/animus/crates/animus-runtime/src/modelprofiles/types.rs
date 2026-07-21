//! Typed in-memory contracts for per-model runtime profiles.

/// Current model runtime profile-set schema identity.
pub const MODEL_RUNTIME_PROFILE_SCHEMA: &str = "animus.model-runtime-profiles.v1";
/// Domain separator for profile content identities.
pub const MODEL_RUNTIME_PROFILE_HASH_DOMAIN: &str = "animus.model-runtime-profile.v1";
/// Domain separator for profile resolution identities.
pub const MODEL_RUNTIME_RESOLUTION_HASH_DOMAIN: &str = "animus.model-runtime-resolution.v1";
/// Domain separator for invocation plan identities.
pub const MODEL_INVOCATION_PLAN_HASH_DOMAIN: &str = "animus.model-invocation-plan.v1";
/// Domain separator for invocation receipt identities.
pub const MODEL_INVOCATION_RECEIPT_HASH_DOMAIN: &str = "animus.model-invocation-receipt.v1";

pub(super) const MAX_PROFILE_COUNT: usize = 64;
pub(super) const MAX_PROFILE_RESOURCE_REFS: usize = 64;
pub(super) const MAX_BUDGET_POLICY_COUNT: usize = 32;
pub(super) const MAX_TOOLSET_COUNT: usize = 32;
pub(super) const MAX_SAFE_LABEL_CHARS: usize = 128;
pub(super) const BLAKE3_HEX_CHARS: usize = 64;
pub(super) const MAX_CONTEXT_BUDGET_UNITS: u32 = 1_000_000;
pub(super) const MAX_TOOL_CALL_BUDGET: u32 = 1_024;
pub(super) const MAX_TOOL_PARALLELISM: u16 = 128;
pub(super) const HASH_SEPARATOR: &str = "|";

/// Host policy controlling whether per-model profile resolution is required.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModelProfileEnablementPolicy {
    /// Every dispatch must resolve and bind an admitted profile.
    Required,
    /// The existing explicitly host-composed compatibility path remains active.
    CompatibilityDisabled,
}

impl ModelProfileEnablementPolicy {
    /// Returns the stable policy label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Required => "required",
            Self::CompatibilityDisabled => "compatibility_disabled",
        }
    }
}

/// Profile source scope.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModelProfileSourceScope {
    /// Repository or host-owned profile.
    Host,
    /// Project-supplied profile requiring explicit admission.
    Project,
}

impl ModelProfileSourceScope {
    /// Returns the stable source-scope label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Host => "host",
            Self::Project => "project",
        }
    }
}

/// Profile source trust status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModelProfileTrustStatus {
    /// The source was admitted for profile influence.
    Trusted,
    /// The source remains untrusted.
    Untrusted,
}

impl ModelProfileTrustStatus {
    /// Returns the stable trust label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Trusted => "trusted",
            Self::Untrusted => "untrusted",
        }
    }
}

/// Profile admission status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModelProfileAdmissionStatus {
    /// The host admitted this profile for resolution.
    Admitted,
    /// Policy denied this profile.
    Denied,
}

impl ModelProfileAdmissionStatus {
    /// Returns the stable admission label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Admitted => "admitted",
            Self::Denied => "denied",
        }
    }
}

/// Native model tool-calling posture.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeToolCallingMode {
    /// Advertise no tools and reject returned tool calls.
    Disabled,
    /// Advertise the admitted intersection when native calls are supported.
    Optional,
    /// Require native support and at least one admitted declaration.
    CapabilityRequired,
}

impl NativeToolCallingMode {
    /// Returns the stable tool-calling label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Disabled => "disabled",
            Self::Optional => "optional",
            Self::CapabilityRequired => "capability_required",
        }
    }
}

/// Model-specific access posture for an existing validated memory policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModelMemoryAccessMode {
    /// Memory cannot influence the invocation or receive generated records.
    Disabled,
    /// Existing policy may admit memory use but never generation.
    ReadOnly,
    /// Existing policy must independently admit both use and generation.
    ReadWrite,
}

impl ModelMemoryAccessMode {
    /// Returns the stable memory-access label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Disabled => "disabled",
            Self::ReadOnly => "read_only",
            Self::ReadWrite => "read_write",
        }
    }
}

/// Per-profile skill disclosure posture.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModelSkillResourcePosture {
    /// Advertise bounded metadata only.
    MetadataOnly,
    /// Advertise metadata and admit an independently selected body.
    Selectable,
    /// Require a separately admitted selected body.
    RequiredSelected,
}

impl ModelSkillResourcePosture {
    /// Returns the stable posture label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MetadataOnly => "metadata_only",
            Self::Selectable => "selectable",
            Self::RequiredSelected => "required_selected",
        }
    }
}

/// Exact or explicit-default profile binding supplied before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModelRuntimeBindingDraft<'a> {
    /// Exact provider/model binding with optional router-role refinement.
    Exact {
        /// Provider identity.
        provider_id: &'a str,
        /// Model identity.
        model_id: &'a str,
        /// Optional exact router role.
        router_role: Option<&'a str>,
        /// Current BLAKE3 model-capability identity.
        capability_identity: &'a str,
    },
    /// Explicit default profile selected only after exact tiers miss.
    Default,
}

/// Validated exact or explicit-default profile binding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ModelRuntimeBinding {
    /// Exact provider/model binding with optional router-role refinement.
    Exact {
        /// Provider identity.
        provider_id: String,
        /// Model identity.
        model_id: String,
        /// Optional exact router role.
        router_role: Option<String>,
        /// Current BLAKE3 model-capability identity.
        capability_identity: String,
    },
    /// Explicit default profile selected only after exact tiers miss.
    Default,
}

/// Named finite context and tool budget policy supplied before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ModelRuntimeBudgetPolicyDraft<'a> {
    /// Stable budget policy ID.
    pub policy_id: &'a str,
    /// Maximum model-visible context units.
    pub max_context_units: u32,
    /// Maximum tool calls for one invocation.
    pub max_tool_calls: u32,
    /// Maximum parallel tool calls.
    pub max_parallel_tool_calls: u16,
}

/// Validated named finite context and tool budget policy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelRuntimeBudgetPolicy {
    pub(super) policy_id: String,
    pub(super) max_context_units: u32,
    pub(super) max_tool_calls: u32,
    pub(super) max_parallel_tool_calls: u16,
}

impl ModelRuntimeBudgetPolicy {
    /// Returns the stable policy ID.
    #[must_use]
    pub fn policy_id(&self) -> &str {
        self.policy_id.as_str()
    }

    /// Returns the model-visible context ceiling.
    #[must_use]
    pub const fn max_context_units(&self) -> u32 {
        self.max_context_units
    }

    /// Returns the tool-call ceiling.
    #[must_use]
    pub const fn max_tool_calls(&self) -> u32 {
        self.max_tool_calls
    }

    /// Returns the parallel tool-call ceiling.
    #[must_use]
    pub const fn max_parallel_tool_calls(&self) -> u16 {
        self.max_parallel_tool_calls
    }
}

/// Named toolset supplied before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ModelToolsetDraft<'a> {
    /// Stable toolset ID.
    pub toolset_id: &'a str,
    /// Exact tool names in the set.
    pub tool_names: &'a [&'a str],
}

/// Validated named toolset.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelToolset {
    pub(super) toolset_id: String,
    pub(super) tool_names: Vec<String>,
}

impl ModelToolset {
    /// Returns the stable toolset ID.
    #[must_use]
    pub fn toolset_id(&self) -> &str {
        self.toolset_id.as_str()
    }

    /// Returns exact tool names.
    #[must_use]
    pub fn tool_names(&self) -> &[String] {
        self.tool_names.as_slice()
    }
}

/// Prompt-source reference supplied before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ModelPromptSourceReferenceDraft<'a> {
    /// Exact prompt source label.
    pub label: &'a str,
    /// Whether composition must include the source completely.
    pub required: bool,
}

/// Validated prompt-source reference.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelPromptSourceReference {
    pub(super) label: String,
    pub(super) required: bool,
}

impl ModelPromptSourceReference {
    /// Returns the exact source label.
    #[must_use]
    pub fn label(&self) -> &str {
        self.label.as_str()
    }

    /// Returns whether complete inclusion is required.
    #[must_use]
    pub const fn required(&self) -> bool {
        self.required
    }
}

/// Skill reference supplied before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ModelSkillReferenceDraft<'a> {
    /// Exact skill name.
    pub name: &'a str,
    /// Requested progressive-disclosure posture.
    pub posture: ModelSkillResourcePosture,
}

/// Validated skill reference.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelSkillReference {
    pub(super) name: String,
    pub(super) posture: ModelSkillResourcePosture,
}

impl ModelSkillReference {
    /// Returns the exact skill name.
    #[must_use]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns the requested skill posture.
    #[must_use]
    pub const fn posture(&self) -> ModelSkillResourcePosture {
        self.posture
    }
}

/// Tool or toolset reference supplied before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ModelNamedResourceReferenceDraft<'a> {
    /// Exact resource name or ID.
    pub name: &'a str,
    /// Whether the reference must produce at least one admitted resource.
    pub required: bool,
}

/// Validated tool or toolset reference.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelNamedResourceReference {
    pub(super) name: String,
    pub(super) required: bool,
}

impl ModelNamedResourceReference {
    /// Returns the exact resource name or ID.
    #[must_use]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns whether the resource is required.
    #[must_use]
    pub const fn required(&self) -> bool {
        self.required
    }
}

/// Borrowed host-loaded profile fields before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ModelRuntimeProfileDraft<'a> {
    /// Stable profile ID.
    pub profile_id: &'a str,
    /// Exact or explicit-default binding.
    pub binding: ModelRuntimeBindingDraft<'a>,
    /// Source scope.
    pub source_scope: ModelProfileSourceScope,
    /// Source trust status.
    pub trust_status: ModelProfileTrustStatus,
    /// Admission status.
    pub admission_status: ModelProfileAdmissionStatus,
    /// Explicit project-profile admission bit.
    pub project_profile_admitted: bool,
    /// Prompt-mode profile reference.
    pub prompt_mode_ref: &'a str,
    /// Exact prompt-source references.
    pub prompt_sources: &'a [ModelPromptSourceReferenceDraft<'a>],
    /// Exact skill references.
    pub skills: &'a [ModelSkillReferenceDraft<'a>],
    /// Explicit memory access posture.
    pub memory_access_mode: ModelMemoryAccessMode,
    /// Exact BLAKE3 identity of the existing validated memory policy.
    pub memory_policy_hash: &'a str,
    /// Exact direct tool references.
    pub tools: &'a [ModelNamedResourceReferenceDraft<'a>],
    /// Exact toolset references.
    pub toolsets: &'a [ModelNamedResourceReferenceDraft<'a>],
    /// Explicit native tool-calling mode.
    pub tool_calling_mode: NativeToolCallingMode,
    /// Named context budget policy reference.
    pub context_budget_policy: &'a str,
    /// Named tool budget policy reference.
    pub tool_budget_policy: &'a str,
    /// Active policy BLAKE3 identity expected by this profile.
    pub policy_hash: &'a str,
    /// BLAKE3 profile content identity.
    pub profile_hash: &'a str,
}

/// Validated host-loaded model runtime profile.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelRuntimeProfile {
    pub(super) profile_id: String,
    pub(super) binding: ModelRuntimeBinding,
    pub(super) source_scope: ModelProfileSourceScope,
    pub(super) trust_status: ModelProfileTrustStatus,
    pub(super) admission_status: ModelProfileAdmissionStatus,
    pub(super) project_profile_admitted: bool,
    pub(super) prompt_mode_ref: String,
    pub(super) prompt_sources: Vec<ModelPromptSourceReference>,
    pub(super) skills: Vec<ModelSkillReference>,
    pub(super) memory_access_mode: ModelMemoryAccessMode,
    pub(super) memory_policy_hash: String,
    pub(super) tools: Vec<ModelNamedResourceReference>,
    pub(super) toolsets: Vec<ModelNamedResourceReference>,
    pub(super) tool_calling_mode: NativeToolCallingMode,
    pub(super) context_budget_policy: String,
    pub(super) tool_budget_policy: String,
    pub(super) policy_hash: String,
    pub(super) profile_hash: String,
}

impl ModelRuntimeProfile {
    /// Returns the stable profile ID.
    #[must_use]
    pub fn profile_id(&self) -> &str {
        self.profile_id.as_str()
    }

    /// Returns the exact or explicit-default binding.
    #[must_use]
    pub const fn binding(&self) -> &ModelRuntimeBinding {
        &self.binding
    }

    /// Returns the source scope.
    #[must_use]
    pub const fn source_scope(&self) -> ModelProfileSourceScope {
        self.source_scope
    }

    /// Returns the source trust status.
    #[must_use]
    pub const fn trust_status(&self) -> ModelProfileTrustStatus {
        self.trust_status
    }

    /// Returns the admission status.
    #[must_use]
    pub const fn admission_status(&self) -> ModelProfileAdmissionStatus {
        self.admission_status
    }

    /// Returns whether a project profile was explicitly admitted.
    #[must_use]
    pub const fn project_profile_admitted(&self) -> bool {
        self.project_profile_admitted
    }

    /// Returns the prompt-mode profile reference.
    #[must_use]
    pub fn prompt_mode_ref(&self) -> &str {
        self.prompt_mode_ref.as_str()
    }

    /// Returns exact prompt-source references.
    #[must_use]
    pub fn prompt_sources(&self) -> &[ModelPromptSourceReference] {
        self.prompt_sources.as_slice()
    }

    /// Returns exact skill references.
    #[must_use]
    pub fn skills(&self) -> &[ModelSkillReference] {
        self.skills.as_slice()
    }

    /// Returns explicit model-specific memory access posture.
    #[must_use]
    pub const fn memory_access_mode(&self) -> ModelMemoryAccessMode {
        self.memory_access_mode
    }

    /// Returns the exact existing memory-policy identity.
    #[must_use]
    pub fn memory_policy_hash(&self) -> &str {
        self.memory_policy_hash.as_str()
    }

    /// Returns exact direct tool references.
    #[must_use]
    pub fn tools(&self) -> &[ModelNamedResourceReference] {
        self.tools.as_slice()
    }

    /// Returns exact toolset references.
    #[must_use]
    pub fn toolsets(&self) -> &[ModelNamedResourceReference] {
        self.toolsets.as_slice()
    }

    /// Returns the explicit native tool-calling mode.
    #[must_use]
    pub const fn tool_calling_mode(&self) -> NativeToolCallingMode {
        self.tool_calling_mode
    }

    /// Returns the context budget-policy reference.
    #[must_use]
    pub fn context_budget_policy(&self) -> &str {
        self.context_budget_policy.as_str()
    }

    /// Returns the tool budget-policy reference.
    #[must_use]
    pub fn tool_budget_policy(&self) -> &str {
        self.tool_budget_policy.as_str()
    }

    /// Returns the active policy identity expected by this profile.
    #[must_use]
    pub fn policy_hash(&self) -> &str {
        self.policy_hash.as_str()
    }

    /// Returns the BLAKE3 profile content identity.
    #[must_use]
    pub fn profile_hash(&self) -> &str {
        self.profile_hash.as_str()
    }
}

/// Borrowed profile-set fields before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ModelRuntimeProfileSetDraft<'a> {
    /// Exact set schema identity.
    pub schema: &'a str,
    /// Explicit enablement policy.
    pub enablement: ModelProfileEnablementPolicy,
    /// Explicitly named default profile, when configured.
    pub explicit_default_profile_id: Option<&'a str>,
    /// Named finite budget policies.
    pub budget_policies: &'a [ModelRuntimeBudgetPolicyDraft<'a>],
    /// Named toolsets.
    pub toolsets: &'a [ModelToolsetDraft<'a>],
    /// Profile rows.
    pub profiles: &'a [ModelRuntimeProfileDraft<'a>],
}

/// Validated host-loaded profile set.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelRuntimeProfileSet {
    pub(super) enablement: ModelProfileEnablementPolicy,
    pub(super) explicit_default_profile_id: Option<String>,
    pub(super) budget_policies: Vec<ModelRuntimeBudgetPolicy>,
    pub(super) toolsets: Vec<ModelToolset>,
    pub(super) profiles: Vec<ModelRuntimeProfile>,
}

impl ModelRuntimeProfileSet {
    /// Returns explicit enablement policy.
    #[must_use]
    pub const fn enablement(&self) -> ModelProfileEnablementPolicy {
        self.enablement
    }

    /// Returns the explicitly named default profile ID.
    #[must_use]
    pub fn explicit_default_profile_id(&self) -> Option<&str> {
        self.explicit_default_profile_id.as_deref()
    }

    /// Returns named finite budget policies.
    #[must_use]
    pub fn budget_policies(&self) -> &[ModelRuntimeBudgetPolicy] {
        self.budget_policies.as_slice()
    }

    /// Returns named toolsets.
    #[must_use]
    pub fn toolsets(&self) -> &[ModelToolset] {
        self.toolsets.as_slice()
    }

    /// Returns validated profiles.
    #[must_use]
    pub fn profiles(&self) -> &[ModelRuntimeProfile] {
        self.profiles.as_slice()
    }
}

/// Selected model target supplied to the pure resolver.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ModelRuntimeTargetDraft<'a> {
    /// Exact provider ID.
    pub provider_id: &'a str,
    /// Exact model ID.
    pub model_id: &'a str,
    /// Optional exact router role.
    pub router_role: Option<&'a str>,
    /// Current BLAKE3 model-capability identity.
    pub capability_identity: &'a str,
}

/// Validated selected model target.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelRuntimeTarget {
    pub(super) provider_id: String,
    pub(super) model_id: String,
    pub(super) router_role: Option<String>,
    pub(super) capability_identity: String,
    pub(super) target_hash: String,
}

impl ModelRuntimeTarget {
    /// Returns the exact provider ID.
    #[must_use]
    pub fn provider_id(&self) -> &str {
        self.provider_id.as_str()
    }

    /// Returns the exact model ID.
    #[must_use]
    pub fn model_id(&self) -> &str {
        self.model_id.as_str()
    }

    /// Returns the optional exact router role.
    #[must_use]
    pub fn router_role(&self) -> Option<&str> {
        self.router_role.as_deref()
    }

    /// Returns the current model-capability identity.
    #[must_use]
    pub fn capability_identity(&self) -> &str {
        self.capability_identity.as_str()
    }

    /// Returns the BLAKE3 target identity.
    #[must_use]
    pub fn target_hash(&self) -> &str {
        self.target_hash.as_str()
    }
}

/// Fixed profile-resolution specificity tier.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModelProfileSpecificity {
    /// Exact provider/model plus router role.
    ExactTargetAndRole,
    /// Exact provider/model with no role refinement.
    ExactTarget,
    /// Explicitly named default profile.
    ExplicitDefault,
}

impl ModelProfileSpecificity {
    /// Returns the stable specificity label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ExactTargetAndRole => "exact_target_and_role",
            Self::ExactTarget => "exact_target",
            Self::ExplicitDefault => "explicit_default",
        }
    }
}

/// Successfully resolved profile plus bound policies and toolsets.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedModelRuntimeProfile {
    pub(super) profile: ModelRuntimeProfile,
    pub(super) target: ModelRuntimeTarget,
    pub(super) specificity: ModelProfileSpecificity,
    pub(super) context_budget: ModelRuntimeBudgetPolicy,
    pub(super) tool_budget: ModelRuntimeBudgetPolicy,
    pub(super) expanded_tools: Vec<ModelNamedResourceReference>,
    pub(super) resolution_hash: String,
}

impl ResolvedModelRuntimeProfile {
    /// Returns the selected profile.
    #[must_use]
    pub const fn profile(&self) -> &ModelRuntimeProfile {
        &self.profile
    }

    /// Returns the selected target.
    #[must_use]
    pub const fn target(&self) -> &ModelRuntimeTarget {
        &self.target
    }

    /// Returns the selected specificity tier.
    #[must_use]
    pub const fn specificity(&self) -> ModelProfileSpecificity {
        self.specificity
    }

    /// Returns the bound context policy.
    #[must_use]
    pub const fn context_budget(&self) -> &ModelRuntimeBudgetPolicy {
        &self.context_budget
    }

    /// Returns the bound tool policy.
    #[must_use]
    pub const fn tool_budget(&self) -> &ModelRuntimeBudgetPolicy {
        &self.tool_budget
    }

    /// Returns expanded direct and toolset tool references.
    #[must_use]
    pub fn expanded_tools(&self) -> &[ModelNamedResourceReference] {
        self.expanded_tools.as_slice()
    }

    /// Returns the BLAKE3 resolution identity.
    #[must_use]
    pub fn resolution_hash(&self) -> &str {
        self.resolution_hash.as_str()
    }
}

/// Compatibility-disabled or required-mode profile resolution.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ModelRuntimeProfileResolution {
    /// Compatibility mode was explicitly selected; no profile was inferred.
    CompatibilityDisabled,
    /// Required mode selected a validated profile.
    Resolved(Box<ResolvedModelRuntimeProfile>),
}

/// Stable model-runtime profile issue class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModelRuntimeProfileIssue {
    /// Schema identity was unsupported.
    InvalidSchema,
    /// A safe label was absent or malformed.
    UnsafeLabel,
    /// A lowercase BLAKE3 identity was malformed.
    InvalidHash,
    /// A collection exceeded its declared bound.
    CollectionTooLarge,
    /// A profile, binding, policy, toolset, or resource identity was duplicated.
    DuplicateIdentity,
    /// A referenced policy, toolset, prompt, skill, tool, or executor was missing.
    MissingReference,
    /// A numeric budget was zero, excessive, or internally inconsistent.
    InvalidBudget,
    /// A profile content identity was stale.
    StaleProfileIdentity,
    /// A target capability or bound plan identity was stale.
    StaleBinding,
    /// A profile source was untrusted or unadmitted.
    UntrustedProfile,
    /// A required profile could not be resolved.
    MissingProfile,
    /// More than one profile matched one specificity tier.
    AmbiguousProfile,
    /// Active policy or verified grants denied a requested resource.
    AuthorityDenied,
    /// Required prompt assembly was denied, missing, truncated, or conflicting.
    PromptDenied,
    /// Requested memory access was denied, stale, or unavailable.
    MemoryDenied,
    /// Required skill disclosure or body admission failed.
    SkillDenied,
    /// Native tool support or exact tool admission failed.
    ToolDenied,
    /// A model dispatch was not admitted.
    DispatchDenied,
    /// A fallback or retry transition requires explicit recomposition.
    RecompositionRequired,
    /// A durable receipt candidate contained forbidden material.
    ReceiptLeak,
}

/// Bounded deterministic model-runtime profile error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelRuntimeProfileError {
    issue: ModelRuntimeProfileIssue,
    field: &'static str,
    detail: String,
}

impl ModelRuntimeProfileError {
    pub(super) fn new(
        issue: ModelRuntimeProfileIssue,
        field: &'static str,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            issue,
            field,
            detail: detail.into(),
        }
    }

    /// Returns the stable issue class.
    #[must_use]
    pub const fn issue(&self) -> ModelRuntimeProfileIssue {
        self.issue
    }

    /// Returns the stable field label.
    #[must_use]
    pub const fn field(&self) -> &'static str {
        self.field
    }

    /// Returns the bounded safe detail.
    #[must_use]
    pub fn detail(&self) -> &str {
        self.detail.as_str()
    }
}

impl core::fmt::Display for ModelRuntimeProfileError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            formatter,
            "{}: {} ({:?})",
            self.field, self.detail, self.issue
        )
    }
}

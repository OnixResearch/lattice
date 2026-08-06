//! Typed session runtime authority-profile data.

#![allow(missing_docs)]

pub const SESSION_AUTHORITY_PROFILE_SCHEMA: &str = "animus.session-authority-profile.v1";
pub const SESSION_AUTHORITY_BINDING_SCHEMA: &str = "animus.session-authority-binding.v1";
pub const SESSION_AUTHORITY_RECEIPT_SCHEMA: &str = "animus.session-authority-receipt.v1";
pub const SESSION_AUTHORITY_PROFILE_HASH_DOMAIN: &str = "animus.session-authority-profile.v1";
pub const SESSION_AUTHORITY_BINDING_HASH_DOMAIN: &str = "animus.session-authority-binding.v1";
pub const SESSION_AUTHORITY_ENFORCEMENT_HASH_DOMAIN: &str = "animus.session-authority-enforcement.v1";
pub const SESSION_AUTHORITY_RECEIPT_HASH_DOMAIN: &str = "animus.session-authority-receipt.v1";

pub(super) const HASH_HEX_BYTES: usize = 64;
pub(super) const COMPONENT_CLASS_COUNT: usize = 10;
pub(super) const ENFORCEMENT_IDENTITY_PART_COUNT: usize = 8;
pub(super) const MAX_SAFE_LABEL_BYTES: usize = 128;
pub(super) const HASH_SEPARATOR: &[u8] = b"\x1f";
pub(super) const PAYLOAD_SEPARATOR: char = '|';
pub(super) const ABSENT_IDENTITY: &str = "none";
pub(super) const REDACTION_MARKER: &str =
    "raw-policy-prompt-tool-extension-credential-path-and-provider-material-omitted";
pub(super) const NON_CLAIMS: &[&str] = &[
    "profile identity continuity is not authorization",
    "profile equality is not sandbox or host capability proof",
    "profile equality is not provider or tool readiness",
    "profile equality is not behavioral replay equivalence",
    "authority-profile evidence is not whole-session correctness or release eligibility",
];

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum SessionAuthorityComponentClass {
    PolicyAuthorization,
    Permission,
    SandboxIntent,
    ModelRuntime,
    ToolCatalog,
    CapabilityCeiling,
    HookPluginSet,
    AgentDefinitionSet,
    ContextRetrieval,
    AdapterCohort,
}

impl SessionAuthorityComponentClass {
    pub const ALL: [Self; COMPONENT_CLASS_COUNT] = [
        Self::PolicyAuthorization,
        Self::Permission,
        Self::SandboxIntent,
        Self::ModelRuntime,
        Self::ToolCatalog,
        Self::CapabilityCeiling,
        Self::HookPluginSet,
        Self::AgentDefinitionSet,
        Self::ContextRetrieval,
        Self::AdapterCohort,
    ];

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PolicyAuthorization => "policy_authorization",
            Self::Permission => "permission",
            Self::SandboxIntent => "sandbox_intent",
            Self::ModelRuntime => "model_runtime",
            Self::ToolCatalog => "tool_catalog",
            Self::CapabilityCeiling => "capability_ceiling",
            Self::HookPluginSet => "hook_plugin_set",
            Self::AgentDefinitionSet => "agent_definition_set",
            Self::ContextRetrieval => "context_retrieval",
            Self::AdapterCohort => "adapter_cohort",
        }
    }

    #[must_use]
    pub fn from_label(value: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|class| class.as_str() == value)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SessionAuthorityComponentDraft<'a> {
    pub class: SessionAuthorityComponentClass,
    pub identity: &'a str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionAuthorityComponent {
    pub(super) class: SessionAuthorityComponentClass,
    pub(super) identity: String,
}

impl SessionAuthorityComponent {
    #[must_use]
    pub const fn class(&self) -> SessionAuthorityComponentClass {
        self.class
    }

    #[must_use]
    pub fn identity(&self) -> &str {
        self.identity.as_str()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SessionAuthorityProfileDraft<'a> {
    pub components: &'a [SessionAuthorityComponentDraft<'a>],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionRuntimeAuthorityProfile {
    components: Vec<SessionAuthorityComponent>,
    profile_identity: String,
}

impl SessionRuntimeAuthorityProfile {
    #[must_use]
    pub fn components(&self) -> &[SessionAuthorityComponent] {
        self.components.as_slice()
    }

    #[must_use]
    pub fn profile_identity(&self) -> &str {
        self.profile_identity.as_str()
    }

    #[must_use]
    pub fn component(&self, class: SessionAuthorityComponentClass) -> Option<&SessionAuthorityComponent> {
        self.components.iter().find(|component| component.class == class)
    }

    pub(super) fn from_parts(components: Vec<SessionAuthorityComponent>, profile_identity: String) -> Self {
        Self {
            components,
            profile_identity,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SessionAuthorityBindingDraft<'a> {
    pub profile: &'a SessionRuntimeAuthorityProfile,
    pub authorization_identity: &'a str,
    pub branch_id: &'a str,
    pub prior_head_hash: &'a str,
    pub prior_profile_identity: Option<&'a str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionAuthorityBinding {
    profile: SessionRuntimeAuthorityProfile,
    authorization_identity: String,
    branch_id: String,
    prior_head_hash: String,
    prior_profile_identity: Option<String>,
    binding_identity: String,
}

impl SessionAuthorityBinding {
    #[must_use]
    pub const fn profile(&self) -> &SessionRuntimeAuthorityProfile {
        &self.profile
    }

    #[must_use]
    pub fn authorization_identity(&self) -> &str {
        self.authorization_identity.as_str()
    }

    #[must_use]
    pub fn branch_id(&self) -> &str {
        self.branch_id.as_str()
    }

    #[must_use]
    pub fn prior_head_hash(&self) -> &str {
        self.prior_head_hash.as_str()
    }

    #[must_use]
    pub fn prior_profile_identity(&self) -> Option<&str> {
        self.prior_profile_identity.as_deref()
    }

    #[must_use]
    pub fn binding_identity(&self) -> &str {
        self.binding_identity.as_str()
    }

    pub(super) fn from_validated_draft(draft: SessionAuthorityBindingDraft<'_>, binding_identity: String) -> Self {
        Self {
            profile: draft.profile.clone(),
            authorization_identity: String::from(draft.authorization_identity),
            branch_id: String::from(draft.branch_id),
            prior_head_hash: String::from(draft.prior_head_hash),
            prior_profile_identity: draft.prior_profile_identity.map(String::from),
            binding_identity,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SessionAuthorityResumeStatus {
    ExactMatch,
    DriftBlocked,
    LegacyUnbound,
}

impl SessionAuthorityResumeStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ExactMatch => "exact_match",
            Self::DriftBlocked => "drift_blocked",
            Self::LegacyUnbound => "legacy_unbound",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionAuthorityComparison {
    status: SessionAuthorityResumeStatus,
    committed_profile_identity: Option<String>,
    current_profile_identity: String,
    changed_components: Vec<SessionAuthorityComponentClass>,
}

impl SessionAuthorityComparison {
    #[must_use]
    pub const fn status(&self) -> SessionAuthorityResumeStatus {
        self.status
    }

    #[must_use]
    pub fn committed_profile_identity(&self) -> Option<&str> {
        self.committed_profile_identity.as_deref()
    }

    #[must_use]
    pub fn current_profile_identity(&self) -> &str {
        self.current_profile_identity.as_str()
    }

    #[must_use]
    pub fn changed_components(&self) -> &[SessionAuthorityComponentClass] {
        self.changed_components.as_slice()
    }

    pub(super) fn from_parts(
        status: SessionAuthorityResumeStatus,
        committed_profile_identity: Option<String>,
        current_profile_identity: String,
        changed_components: Vec<SessionAuthorityComponentClass>,
    ) -> Self {
        Self {
            status,
            committed_profile_identity,
            current_profile_identity,
            changed_components,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SessionAuthorityTransitionAction<'a> {
    Abandon,
    Fork {
        branch_id: &'a str,
        authorization_identity: &'a str,
    },
    Rebind {
        branch_id: &'a str,
        authorization_identity: &'a str,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SessionAuthorityTransitionRequest<'a> {
    pub comparison: &'a SessionAuthorityComparison,
    pub requested_profile: &'a SessionRuntimeAuthorityProfile,
    pub prior_head_hash: &'a str,
    pub action: SessionAuthorityTransitionAction<'a>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SessionAuthorityTransitionDisposition {
    Abandoned,
    ForkCommitRequired,
    RebindCommitRequired,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionAuthorityTransitionPlan {
    disposition: SessionAuthorityTransitionDisposition,
    binding: Option<SessionAuthorityBinding>,
}

impl SessionAuthorityTransitionPlan {
    #[must_use]
    pub const fn disposition(&self) -> SessionAuthorityTransitionDisposition {
        self.disposition
    }

    #[must_use]
    pub const fn binding(&self) -> Option<&SessionAuthorityBinding> {
        self.binding.as_ref()
    }

    pub(super) const fn from_parts(
        disposition: SessionAuthorityTransitionDisposition,
        binding: Option<SessionAuthorityBinding>,
    ) -> Self {
        Self { disposition, binding }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SessionAuthorityEnforcementDraft<'a> {
    pub profile_identity: &'a str,
    pub authorization_identity: &'a str,
    pub adapter_cohort_identity: &'a str,
    pub capability_observation_identity: &'a str,
    pub tool_readiness_identity: &'a str,
    pub provider_readiness_identity: &'a str,
    pub sandbox_observation_identity: &'a str,
    pub admitted: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionAuthorityEnforcementObservation {
    profile_identity: String,
    authorization_identity: String,
    adapter_cohort_identity: String,
    capability_observation_identity: String,
    tool_readiness_identity: String,
    provider_readiness_identity: String,
    sandbox_observation_identity: String,
    admitted: bool,
    observation_identity: String,
}

impl SessionAuthorityEnforcementObservation {
    #[must_use]
    pub fn profile_identity(&self) -> &str {
        self.profile_identity.as_str()
    }

    #[must_use]
    pub fn authorization_identity(&self) -> &str {
        self.authorization_identity.as_str()
    }

    #[must_use]
    pub const fn admitted(&self) -> bool {
        self.admitted
    }

    #[must_use]
    pub fn observation_identity(&self) -> &str {
        self.observation_identity.as_str()
    }

    pub(super) fn from_draft(draft: SessionAuthorityEnforcementDraft<'_>, observation_identity: String) -> Self {
        Self {
            profile_identity: String::from(draft.profile_identity),
            authorization_identity: String::from(draft.authorization_identity),
            adapter_cohort_identity: String::from(draft.adapter_cohort_identity),
            capability_observation_identity: String::from(draft.capability_observation_identity),
            tool_readiness_identity: String::from(draft.tool_readiness_identity),
            provider_readiness_identity: String::from(draft.provider_readiness_identity),
            sandbox_observation_identity: String::from(draft.sandbox_observation_identity),
            admitted: draft.admitted,
            observation_identity,
        }
    }

    pub(super) fn identity_parts(&self) -> [&str; ENFORCEMENT_IDENTITY_PART_COUNT] {
        [
            self.profile_identity.as_str(),
            self.authorization_identity.as_str(),
            self.adapter_cohort_identity.as_str(),
            self.capability_observation_identity.as_str(),
            self.tool_readiness_identity.as_str(),
            self.provider_readiness_identity.as_str(),
            self.sandbox_observation_identity.as_str(),
            if self.admitted { "admitted" } else { "denied" },
        ]
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SessionAuthorityIssue {
    MissingComponent,
    DuplicateComponent,
    UnsafeIdentity,
    IdentityMismatch,
    LegacyUnbound,
    ProfileDrift,
    ReauthorizationRequired,
    EnforcementUnavailable,
    StaleHead,
    InPlaceRewrite,
    InvalidPayload,
    UnsafeReceipt,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionAuthorityError {
    issue: SessionAuthorityIssue,
    field: &'static str,
}

impl SessionAuthorityError {
    pub(super) const fn new(issue: SessionAuthorityIssue, field: &'static str) -> Self {
        Self { issue, field }
    }

    #[must_use]
    pub const fn issue(&self) -> SessionAuthorityIssue {
        self.issue
    }

    #[must_use]
    pub const fn field(&self) -> &'static str {
        self.field
    }
}

impl core::fmt::Display for SessionAuthorityError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "session authority profile rejected {}", self.field)
    }
}

impl std::error::Error for SessionAuthorityError {}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SessionAuthoritySensitiveDraft<'a> {
    pub raw_policy: Option<&'a str>,
    pub prompt: Option<&'a str>,
    pub tool_schema: Option<&'a str>,
    pub extension_body: Option<&'a str>,
    pub credential: Option<&'a str>,
    pub environment_value: Option<&'a str>,
    pub host_path: Option<&'a str>,
    pub provider_payload: Option<&'a str>,
    pub raw_diagnostic: Option<&'a str>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SessionAuthorityReceiptDraft<'a> {
    pub session_id: &'a str,
    pub branch_id: &'a str,
    pub head_hash: &'a str,
    pub comparison: &'a SessionAuthorityComparison,
    pub binding: Option<&'a SessionAuthorityBinding>,
    pub enforcement: Option<&'a SessionAuthorityEnforcementObservation>,
    pub sensitive: SessionAuthoritySensitiveDraft<'a>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionAuthorityReceipt {
    pub schema: &'static str,
    pub status: SessionAuthorityResumeStatus,
    pub session_id: String,
    pub branch_id: String,
    pub head_hash: String,
    pub committed_profile_identity: Option<String>,
    pub current_profile_identity: String,
    pub binding_identity: Option<String>,
    pub enforcement_observation_identity: Option<String>,
    pub changed_components: Vec<String>,
    pub redaction_marker: &'static str,
    pub non_claims: Vec<String>,
    pub receipt_identity: String,
}

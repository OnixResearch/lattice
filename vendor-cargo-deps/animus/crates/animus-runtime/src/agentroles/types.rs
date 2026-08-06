//! Agent-role catalog data and validation diagnostics.

#![allow(unknown_lints)]
#![cfg_attr(animus_octet, allow(tigerstyle::path_segment_repetition))]

/// Versioned Nickel and Rust agent-role catalog schema.
pub const AGENT_ROLE_CATALOG_SCHEMA: &str = "animus.agent-role-catalog.v1";
/// BLAKE3 domain for canonical agent-role profile identities.
pub const AGENT_ROLE_PROFILE_HASH_DOMAIN: &str = "animus.agent-role-profile.v1";
/// BLAKE3 domain for role-bound model-plan identities.
pub const AGENT_ROLE_MODEL_PLAN_HASH_DOMAIN: &str = "animus.agent-role-model-plan.v1";
/// BLAKE3 domain for admitted assignment identities.
pub const AGENT_ROLE_ASSIGNMENT_HASH_DOMAIN: &str = "animus.agent-role-assignment.v1";
/// BLAKE3 domain for capability-decision identities.
pub const AGENT_ROLE_CAPABILITY_HASH_DOMAIN: &str = "animus.agent-role-capability.v1";
/// BLAKE3 domain for separation-decision identities.
pub const AGENT_ROLE_SEPARATION_HASH_DOMAIN: &str = "animus.agent-role-separation.v1";
/// BLAKE3 domain for immutable role-plan identities.
pub const AGENT_ROLE_PLAN_HASH_DOMAIN: &str = "animus.agent-role-plan.v1";
/// BLAKE3 domain for role-output identities.
pub const AGENT_ROLE_OUTPUT_HASH_DOMAIN: &str = "animus.agent-role-output.v1";
/// BLAKE3 domain for role handoff identities.
pub const AGENT_ROLE_HANDOFF_HASH_DOMAIN: &str = "animus.agent-role-handoff.v1";
/// BLAKE3 domain for metadata-only role receipts.
pub const AGENT_ROLE_RECEIPT_HASH_DOMAIN: &str = "animus.agent-role-receipt.v1";

pub(super) const BLAKE3_HEX_CHARS: usize = 64;
pub(super) const MAX_ROLE_PROFILES: usize = 64;
pub(super) const MAX_ROLE_REFERENCES: usize = 128;
pub(super) const MAX_ROLE_POLICIES: usize = 64;
pub(super) const MAX_ROLE_LABEL_CHARS: usize = 128;
pub(super) const MAX_ROLE_PURPOSE_CHARS: usize = 1_024;
pub(super) const MAX_ROLE_ACTION_BUDGET: u32 = 1_024;
pub(super) const MAX_ROLE_OUTPUT_BUDGET: u16 = 128;
pub(super) const MAX_ROLE_CONTEXT_BUDGET: u32 = 1_000_000;
pub(super) const HASH_SEPARATOR: &str = "\u{1f}";

/// Supported semantic agent-role class.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum AgentRoleClass {
    /// Produces bounded implementation candidates and mutation intents.
    Implementation,
    /// Produces read-only findings and tighten-only recommendations.
    Review,
    /// Requests named host checks for a pinned subject.
    Verification,
    /// Ranks, selects, vetoes, ties, or abstains under a typed rubric.
    Judgment,
    /// Plans, delegates, aggregates, and hands off through explicit services.
    Coordination,
    /// Observes admitted context and emits read-only reports.
    Observation,
}

impl AgentRoleClass {
    /// Returns the stable Nickel and receipt label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Implementation => "implementation",
            Self::Review => "review",
            Self::Verification => "verification",
            Self::Judgment => "judgment",
            Self::Coordination => "coordination",
            Self::Observation => "observation",
        }
    }
}

/// Mutation posture enforced for one semantic role.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AgentRoleMutationPosture {
    /// Scoped mutation intents may be requested but never applied by this core.
    ScopedIntent,
    /// The pinned subject is read-only.
    SubjectReadOnly,
    /// Named host checks may execute while subject and verifier remain immutable.
    VerificationServiceOnly,
    /// No mutation or effectful execution may be requested.
    None,
}

impl AgentRoleMutationPosture {
    /// Returns the stable posture label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ScopedIntent => "scoped_intent",
            Self::SubjectReadOnly => "subject_read_only",
            Self::VerificationServiceOnly => "verification_service_only",
            Self::None => "none",
        }
    }
}

/// Context posture for semantic role transitions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AgentRoleContextPosture {
    /// Recompose from explicitly admitted shared records with clean role context.
    CleanRecompose,
    /// Consume only bounded shared subject and evidence records.
    BoundedShared,
    /// Consume read-only observation records.
    ReadOnlyObservation,
}

impl AgentRoleContextPosture {
    /// Returns the stable posture label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CleanRecompose => "clean_recompose",
            Self::BoundedShared => "bounded_shared",
            Self::ReadOnlyObservation => "read_only_observation",
        }
    }
}

/// Evidence posture for role output.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AgentRoleEvidencePosture {
    /// Candidate and validation-request evidence only.
    Candidate,
    /// Structured findings, uncertainty, and recommendations.
    Findings,
    /// Supplied named-check evidence without verifier-soundness claims.
    CheckEvidence,
    /// Rubric-bound tighten-only judgment evidence.
    Judgment,
    /// Coordination and handoff metadata.
    Coordination,
    /// Read-only observation metadata.
    Observation,
}

impl AgentRoleEvidencePosture {
    /// Returns the stable posture label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Candidate => "candidate",
            Self::Findings => "findings",
            Self::CheckEvidence => "check_evidence",
            Self::Judgment => "judgment",
            Self::Coordination => "coordination",
            Self::Observation => "observation",
        }
    }
}

/// Typed output contract class for a semantic role.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AgentRoleOutputKind {
    /// Implementation artifact or change candidate.
    ImplementationCandidate,
    /// Structured review report.
    ReviewReport,
    /// Named-check verification report.
    VerificationReport,
    /// Rubric-bound judgment report.
    JudgmentReport,
    /// Coordination or handoff report.
    CoordinationReport,
    /// Read-only observation report.
    ObservationReport,
}

impl AgentRoleOutputKind {
    /// Returns the stable output-kind label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ImplementationCandidate => "implementation_candidate",
            Self::ReviewReport => "review_report",
            Self::VerificationReport => "verification_report",
            Self::JudgmentReport => "judgment_report",
            Self::CoordinationReport => "coordination_report",
            Self::ObservationReport => "observation_report",
        }
    }
}

/// Named finite role budget before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AgentRoleBudgetPolicyDraft<'a> {
    /// Stable budget-policy identity.
    pub policy_id: &'a str,
    /// Maximum admitted role-bound action requests.
    pub max_actions: u32,
    /// Maximum admitted role outputs.
    pub max_outputs: u16,
    /// Maximum role-specific context units.
    pub max_context_units: u32,
}

/// Validated named finite role budget.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentRoleBudgetPolicy {
    pub(super) policy_id: String,
    pub(super) max_actions: u32,
    pub(super) max_outputs: u16,
    pub(super) max_context_units: u32,
}

impl AgentRoleBudgetPolicy {
    /// Returns the policy identity.
    #[must_use]
    pub fn policy_id(&self) -> &str {
        self.policy_id.as_str()
    }

    /// Returns the role action limit.
    #[must_use]
    pub const fn max_actions(&self) -> u32 {
        self.max_actions
    }

    /// Returns the role output limit.
    #[must_use]
    pub const fn max_outputs(&self) -> u16 {
        self.max_outputs
    }

    /// Returns the context-unit limit.
    #[must_use]
    pub const fn max_context_units(&self) -> u32 {
        self.max_context_units
    }
}

/// Named role toolset before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AgentRoleToolsetDraft<'a> {
    /// Stable toolset identity.
    pub toolset_id: &'a str,
    /// Exact tool names in the set.
    pub tool_names: &'a [&'a str],
}

/// Validated named role toolset.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentRoleToolset {
    pub(super) toolset_id: String,
    pub(super) tool_names: Vec<String>,
}

impl AgentRoleToolset {
    /// Returns the toolset identity.
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

/// Explicit identity or state dimension checked for separation of duty.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum AgentRoleSeparationDimension {
    /// Assignment identity must differ.
    Assignment,
    /// Actor identity must differ.
    Actor,
    /// Run or thread identity must differ.
    RunThread,
    /// Session identity must differ.
    Session,
    /// Model/runtime-profile identity must differ.
    ModelProfile,
    /// Workspace identity must differ.
    Workspace,
    /// Mutation lineage must differ.
    MutationLineage,
    /// Role context must be reconstructed from admitted bounded records.
    CleanContext,
    /// Subject must remain immutable.
    SubjectImmutable,
    /// Verifier or check identity must remain immutable.
    VerifierImmutable,
    /// Rubric identity must remain immutable.
    RubricImmutable,
}

impl AgentRoleSeparationDimension {
    /// Returns the stable dimension label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Assignment => "assignment",
            Self::Actor => "actor",
            Self::RunThread => "run_thread",
            Self::Session => "session",
            Self::ModelProfile => "model_profile",
            Self::Workspace => "workspace",
            Self::MutationLineage => "mutation_lineage",
            Self::CleanContext => "clean_context",
            Self::SubjectImmutable => "subject_immutable",
            Self::VerifierImmutable => "verifier_immutable",
            Self::RubricImmutable => "rubric_immutable",
        }
    }
}

/// Separation-of-duty dimensions required by one named policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AgentRoleSeparationPolicyDraft<'a> {
    /// Stable policy identity.
    pub policy_id: &'a str,
    /// Exact required identity and state dimensions.
    pub required_dimensions: &'a [AgentRoleSeparationDimension],
}

/// Validated separation-of-duty policy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentRoleSeparationPolicy {
    pub(super) policy_id: String,
    pub(super) required_dimensions: Vec<AgentRoleSeparationDimension>,
}

impl AgentRoleSeparationPolicy {
    /// Returns the policy identity.
    #[must_use]
    pub fn policy_id(&self) -> &str {
        self.policy_id.as_str()
    }

    /// Returns exact required separation dimensions.
    #[must_use]
    pub fn required_dimensions(&self) -> &[AgentRoleSeparationDimension] {
        self.required_dimensions.as_slice()
    }

    /// Returns whether this policy requires one dimension.
    #[must_use]
    pub fn requires(&self, dimension: AgentRoleSeparationDimension) -> bool {
        self.required_dimensions.contains(&dimension)
    }
}

/// Borrowed semantic role profile before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AgentRoleProfileDraft<'a> {
    /// Stable semantic role identity.
    pub role_id: &'a str,
    /// Supported semantic class.
    pub class: AgentRoleClass,
    /// Bounded safe role purpose.
    pub purpose: &'a str,
    /// Source scope.
    pub source_scope: crate::ModelProfileSourceScope,
    /// Trust status.
    pub trust_status: crate::ModelProfileTrustStatus,
    /// Admission status.
    pub admission_status: crate::ModelProfileAdmissionStatus,
    /// Explicit host admission for project profiles.
    pub project_profile_admitted: bool,
    /// Capabilities required for activation eligibility.
    pub required_capabilities: &'a [&'a str],
    /// Maximum capabilities this role may request.
    pub capability_ceiling: &'a [&'a str],
    /// Eligible per-model runtime profile IDs.
    pub eligible_model_profiles: &'a [&'a str],
    /// Exact provider/model targets admitted by this role.
    pub model_targets: &'a [&'a str],
    /// Prompt modes admitted by this role.
    pub prompt_modes: &'a [&'a str],
    /// Prompt sources admitted by this role.
    pub prompt_sources: &'a [&'a str],
    /// Skill names admitted by this role.
    pub skills: &'a [&'a str],
    /// Existing memory-policy identities admitted by this role.
    pub memory_policy_hashes: &'a [&'a str],
    /// Direct tool names admitted by this role.
    pub tools: &'a [&'a str],
    /// Named toolset references admitted by this role.
    pub toolsets: &'a [&'a str],
    /// Tool capability labels admitted by this role.
    pub tool_capabilities: &'a [&'a str],
    /// Workspace scopes admitted by this role.
    pub workspace_scopes: &'a [&'a str],
    /// Mutation posture.
    pub mutation_posture: AgentRoleMutationPosture,
    /// Context posture.
    pub context_posture: AgentRoleContextPosture,
    /// Evidence posture.
    pub evidence_posture: AgentRoleEvidencePosture,
    /// Output kind.
    pub output_kind: AgentRoleOutputKind,
    /// Input contract reference.
    pub input_contract: &'a str,
    /// Output contract reference.
    pub output_contract: &'a str,
    /// Evidence contract reference.
    pub evidence_contract: &'a str,
    /// Separation-policy reference.
    pub separation_policy_ref: &'a str,
    /// Named budget-policy reference.
    pub budget_policy_ref: &'a str,
    /// Active policy BLAKE3 identity.
    pub policy_hash: &'a str,
    /// Canonical profile BLAKE3 identity.
    pub profile_hash: &'a str,
}

/// Validated semantic agent-role profile.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentRoleProfile {
    pub(super) role_id: String,
    pub(super) class: AgentRoleClass,
    pub(super) purpose: String,
    pub(super) source_scope: crate::ModelProfileSourceScope,
    pub(super) trust_status: crate::ModelProfileTrustStatus,
    pub(super) admission_status: crate::ModelProfileAdmissionStatus,
    pub(super) project_profile_admitted: bool,
    pub(super) required_capabilities: Vec<String>,
    pub(super) capability_ceiling: Vec<String>,
    pub(super) eligible_model_profiles: Vec<String>,
    pub(super) model_targets: Vec<String>,
    pub(super) prompt_modes: Vec<String>,
    pub(super) prompt_sources: Vec<String>,
    pub(super) skills: Vec<String>,
    pub(super) memory_policy_hashes: Vec<String>,
    pub(super) tools: Vec<String>,
    pub(super) toolsets: Vec<String>,
    pub(super) expanded_tools: Vec<String>,
    pub(super) tool_capabilities: Vec<String>,
    pub(super) workspace_scopes: Vec<String>,
    pub(super) mutation_posture: AgentRoleMutationPosture,
    pub(super) context_posture: AgentRoleContextPosture,
    pub(super) evidence_posture: AgentRoleEvidencePosture,
    pub(super) output_kind: AgentRoleOutputKind,
    pub(super) input_contract: String,
    pub(super) output_contract: String,
    pub(super) evidence_contract: String,
    pub(super) separation_policy_ref: String,
    pub(super) budget_policy_ref: String,
    pub(super) policy_hash: String,
    pub(super) profile_hash: String,
}

impl AgentRoleProfile {
    /// Returns the semantic role identity.
    #[must_use]
    pub fn role_id(&self) -> &str {
        self.role_id.as_str()
    }

    /// Returns the semantic class.
    #[must_use]
    pub const fn class(&self) -> AgentRoleClass {
        self.class
    }

    /// Returns the bounded purpose.
    #[must_use]
    pub fn purpose(&self) -> &str {
        self.purpose.as_str()
    }

    /// Returns source scope.
    #[must_use]
    pub const fn source_scope(&self) -> crate::ModelProfileSourceScope {
        self.source_scope
    }

    /// Returns trust status.
    #[must_use]
    pub const fn trust_status(&self) -> crate::ModelProfileTrustStatus {
        self.trust_status
    }

    /// Returns admission status.
    #[must_use]
    pub const fn admission_status(&self) -> crate::ModelProfileAdmissionStatus {
        self.admission_status
    }

    /// Returns required capability labels.
    #[must_use]
    pub fn required_capabilities(&self) -> &[String] {
        self.required_capabilities.as_slice()
    }

    /// Returns capability ceiling labels.
    #[must_use]
    pub fn capability_ceiling(&self) -> &[String] {
        self.capability_ceiling.as_slice()
    }

    /// Returns eligible model-runtime profile IDs.
    #[must_use]
    pub fn eligible_model_profiles(&self) -> &[String] {
        self.eligible_model_profiles.as_slice()
    }

    /// Returns exact model target labels.
    #[must_use]
    pub fn model_targets(&self) -> &[String] {
        self.model_targets.as_slice()
    }

    /// Returns admitted prompt modes.
    #[must_use]
    pub fn prompt_modes(&self) -> &[String] {
        self.prompt_modes.as_slice()
    }

    /// Returns admitted prompt sources.
    #[must_use]
    pub fn prompt_sources(&self) -> &[String] {
        self.prompt_sources.as_slice()
    }

    /// Returns admitted skills.
    #[must_use]
    pub fn skills(&self) -> &[String] {
        self.skills.as_slice()
    }

    /// Returns admitted existing memory-policy identities.
    #[must_use]
    pub fn memory_policy_hashes(&self) -> &[String] {
        self.memory_policy_hashes.as_slice()
    }

    /// Returns expanded exact tool names.
    #[must_use]
    pub fn expanded_tools(&self) -> &[String] {
        self.expanded_tools.as_slice()
    }

    /// Returns admitted tool capability labels.
    #[must_use]
    pub fn tool_capabilities(&self) -> &[String] {
        self.tool_capabilities.as_slice()
    }

    /// Returns admitted workspace scopes.
    #[must_use]
    pub fn workspace_scopes(&self) -> &[String] {
        self.workspace_scopes.as_slice()
    }

    /// Returns mutation posture.
    #[must_use]
    pub const fn mutation_posture(&self) -> AgentRoleMutationPosture {
        self.mutation_posture
    }

    /// Returns context posture.
    #[must_use]
    pub const fn context_posture(&self) -> AgentRoleContextPosture {
        self.context_posture
    }

    /// Returns evidence posture.
    #[must_use]
    pub const fn evidence_posture(&self) -> AgentRoleEvidencePosture {
        self.evidence_posture
    }

    /// Returns output kind.
    #[must_use]
    pub const fn output_kind(&self) -> AgentRoleOutputKind {
        self.output_kind
    }

    /// Returns input contract reference.
    #[must_use]
    pub fn input_contract(&self) -> &str {
        self.input_contract.as_str()
    }

    /// Returns output contract reference.
    #[must_use]
    pub fn output_contract(&self) -> &str {
        self.output_contract.as_str()
    }

    /// Returns evidence contract reference.
    #[must_use]
    pub fn evidence_contract(&self) -> &str {
        self.evidence_contract.as_str()
    }

    /// Returns separation-policy reference.
    #[must_use]
    pub fn separation_policy_ref(&self) -> &str {
        self.separation_policy_ref.as_str()
    }

    /// Returns budget-policy reference.
    #[must_use]
    pub fn budget_policy_ref(&self) -> &str {
        self.budget_policy_ref.as_str()
    }

    /// Returns active policy identity.
    #[must_use]
    pub fn policy_hash(&self) -> &str {
        self.policy_hash.as_str()
    }

    /// Returns canonical profile identity.
    #[must_use]
    pub fn profile_hash(&self) -> &str {
        self.profile_hash.as_str()
    }
}

/// Borrowed role-catalog values before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AgentRoleCatalogDraft<'a> {
    /// Exact schema identity.
    pub schema: &'a str,
    /// Known stable capability labels.
    pub known_capabilities: &'a [&'a str],
    /// Known input contract labels.
    pub input_contracts: &'a [&'a str],
    /// Known output contract labels.
    pub output_contracts: &'a [&'a str],
    /// Known evidence contract labels.
    pub evidence_contracts: &'a [&'a str],
    /// Named finite role budgets.
    pub budget_policies: &'a [AgentRoleBudgetPolicyDraft<'a>],
    /// Named role toolsets.
    pub toolsets: &'a [AgentRoleToolsetDraft<'a>],
    /// Named separation policies.
    pub separation_policies: &'a [AgentRoleSeparationPolicyDraft<'a>],
    /// Semantic role profiles.
    pub profiles: &'a [AgentRoleProfileDraft<'a>],
}

/// Validated owned role catalog.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentRoleCatalog {
    pub(super) known_capabilities: Vec<String>,
    pub(super) input_contracts: Vec<String>,
    pub(super) output_contracts: Vec<String>,
    pub(super) evidence_contracts: Vec<String>,
    pub(super) budget_policies: Vec<AgentRoleBudgetPolicy>,
    pub(super) toolsets: Vec<AgentRoleToolset>,
    pub(super) separation_policies: Vec<AgentRoleSeparationPolicy>,
    pub(super) profiles: Vec<AgentRoleProfile>,
}

impl AgentRoleCatalog {
    /// Returns known capability labels.
    #[must_use]
    pub fn known_capabilities(&self) -> &[String] {
        self.known_capabilities.as_slice()
    }

    /// Returns named role budgets.
    #[must_use]
    pub fn budget_policies(&self) -> &[AgentRoleBudgetPolicy] {
        self.budget_policies.as_slice()
    }

    /// Returns named role toolsets.
    #[must_use]
    pub fn toolsets(&self) -> &[AgentRoleToolset] {
        self.toolsets.as_slice()
    }

    /// Returns named separation policies.
    #[must_use]
    pub fn separation_policies(&self) -> &[AgentRoleSeparationPolicy] {
        self.separation_policies.as_slice()
    }

    /// Returns validated semantic role profiles.
    #[must_use]
    pub fn profiles(&self) -> &[AgentRoleProfile] {
        self.profiles.as_slice()
    }

    /// Finds a role by exact semantic identity.
    #[must_use]
    pub fn profile(&self, role_id: &str) -> Option<&AgentRoleProfile> {
        self.profiles.iter().find(|profile| profile.role_id() == role_id)
    }

    /// Finds a separation policy by exact identity.
    #[must_use]
    pub fn separation_policy(&self, policy_id: &str) -> Option<&AgentRoleSeparationPolicy> {
        self.separation_policies.iter().find(|policy| policy.policy_id() == policy_id)
    }
}

/// Stable semantic role validation and admission failure class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AgentRoleIssue {
    /// Catalog schema is unsupported.
    InvalidSchema,
    /// A label or bounded text is unsafe.
    UnsafeLabel,
    /// A BLAKE3 identity is malformed.
    InvalidHash,
    /// A bounded collection is empty or too large.
    InvalidCollection,
    /// A normalized identity is duplicated.
    DuplicateIdentity,
    /// A referenced policy, contract, toolset, or capability is missing.
    MissingReference,
    /// Canonical content identity is stale.
    StaleIdentity,
    /// Profile trust or admission state is insufficient.
    UntrustedProfile,
    /// Semantic class posture is contradictory.
    ClassInvariant,
    /// Guarded assignment or output authority is absent or mismatched.
    AuthorityDenied,
    /// Assignment, subject, model, policy, or lineage binding is stale.
    BindingMismatch,
    /// A required role capability is absent from the final intersection.
    MissingRequiredCapability,
    /// A capability was denied by one or more independent ceilings.
    CapabilityDenied,
    /// A role/model resource intersection is incompatible.
    ResourceDenied,
    /// Required separation dimensions did not pass.
    SeparationDenied,
    /// Role output violates class, capability, or claim boundaries.
    OutputDenied,
    /// Role transition or handoff tried to retain stale authority.
    TransitionDenied,
    /// Durable receipt candidate contains forbidden material.
    ReceiptLeak,
}

/// Safe semantic role validation or admission error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentRoleError {
    issue: AgentRoleIssue,
    field: &'static str,
    detail: String,
}

impl AgentRoleError {
    pub(super) fn new(issue: AgentRoleIssue, field: &'static str, detail: impl Into<String>) -> Self {
        Self {
            issue,
            field,
            detail: detail.into(),
        }
    }

    /// Returns the stable issue class.
    #[must_use]
    pub const fn issue(&self) -> AgentRoleIssue {
        self.issue
    }

    /// Returns the safe field label.
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

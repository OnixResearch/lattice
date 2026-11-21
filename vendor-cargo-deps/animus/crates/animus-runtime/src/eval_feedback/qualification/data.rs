//! Qualification subject, attachment, answer, and applicability types.

use alloc::string::String;
use alloc::vec::Vec;

/// One closed component of the evaluated subject.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ComponentKind {
    /// Agent definition and revision.
    AgentDefinition,
    /// Budget and resource envelope.
    Budget,
    /// Effective context-window control.
    ContextControls,
    /// Evaluation configuration.
    EvaluationConfig,
    /// Task corpus and partitions.
    Corpus,
    /// Execution environment.
    Environment,
    /// Executable identity.
    Executable,
    /// Harness and runtime.
    Harness,
    /// Workspace and isolation posture.
    Isolation,
    /// Memory policy.
    MemoryPolicy,
    /// Model route and configuration.
    ModelRoute,
    /// Network posture.
    Network,
    /// Effective output control.
    OutputControls,
    /// Effective permissions.
    Permissions,
    /// System and task prompt templates.
    Prompt,
    /// Effective sampling control.
    SamplingControls,
    /// Selection policy over trials.
    SelectionPolicy,
    /// Semantic-role profile.
    SemanticRole,
    /// Skill set.
    Skills,
    /// Strategy or scaffold revision.
    Strategy,
    /// Task protocol.
    TaskProtocol,
    /// Tool implementation.
    ToolImplementation,
    /// Tool schemas and protocols.
    ToolSchemas,
    /// Topology and concurrency.
    Topology,
    /// Independent verifier.
    Verifier,
}

impl ComponentKind {
    /// Every component, in canonical (wire-spelling) order.
    pub const ALL: [Self; 25] = [
        Self::AgentDefinition,
        Self::Budget,
        Self::ContextControls,
        Self::Corpus,
        Self::Environment,
        Self::EvaluationConfig,
        Self::Executable,
        Self::Harness,
        Self::Isolation,
        Self::MemoryPolicy,
        Self::ModelRoute,
        Self::Network,
        Self::OutputControls,
        Self::Permissions,
        Self::Prompt,
        Self::SamplingControls,
        Self::SelectionPolicy,
        Self::SemanticRole,
        Self::Skills,
        Self::Strategy,
        Self::TaskProtocol,
        Self::ToolImplementation,
        Self::ToolSchemas,
        Self::Topology,
        Self::Verifier,
    ];

    /// Components that define what evidence measured; a compatibility decision
    /// can never waive them.
    pub const EVALUATOR: [Self; 5] = [
        Self::Budget,
        Self::Corpus,
        Self::EvaluationConfig,
        Self::TaskProtocol,
        Self::Verifier,
    ];

    /// Wire spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::AgentDefinition => "agent_definition",
            Self::Budget => "budget",
            Self::ContextControls => "context_controls",
            Self::Corpus => "corpus",
            Self::Environment => "environment",
            Self::EvaluationConfig => "evaluation_config",
            Self::Executable => "executable",
            Self::Harness => "harness",
            Self::Isolation => "isolation",
            Self::MemoryPolicy => "memory_policy",
            Self::ModelRoute => "model_route",
            Self::Network => "network",
            Self::OutputControls => "output_controls",
            Self::Permissions => "permissions",
            Self::Prompt => "prompt",
            Self::SamplingControls => "sampling_controls",
            Self::SelectionPolicy => "selection_policy",
            Self::SemanticRole => "semantic_role",
            Self::Skills => "skills",
            Self::Strategy => "strategy",
            Self::TaskProtocol => "task_protocol",
            Self::ToolImplementation => "tool_implementation",
            Self::ToolSchemas => "tool_schemas",
            Self::Topology => "topology",
            Self::Verifier => "verifier",
        }
    }

    /// Parses a wire spelling.
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| kind.as_str() == value)
    }
}

/// How a component identity is known.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ComponentBasis {
    /// Declared by configuration; not observed during the run.
    Declared,
    /// Observed from the exact run.
    Observed,
    /// No identity is available.
    Unavailable,
}

impl ComponentBasis {
    /// Wire spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Declared => "declared",
            Self::Observed => "observed",
            Self::Unavailable => "unavailable",
        }
    }
}

/// One subject component.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Component {
    /// Component kind.
    pub kind: ComponentKind,
    /// Declared, observed, or unavailable.
    pub basis: ComponentBasis,
    /// `blake3:` identity; absent exactly when unavailable.
    pub identity: Option<String>,
}

/// Where the evaluated model ran.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModelDeployment {
    /// Provider-hosted model.
    Hosted,
    /// Locally served model.
    Local,
    /// Deterministic scripted route with no model.
    Scripted,
}

/// What supports the recorded model version.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VersionBasis {
    /// Version facts the provider exposes; weights are not verified.
    ProviderDeclared,
    /// Verified artifact identity.
    VerifiedArtifact,
    /// Version declared by configuration.
    Declared,
    /// No version fact is available.
    Unavailable,
}

/// Model version facts of the subject.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelFacts {
    /// Where the model ran.
    pub deployment: ModelDeployment,
    /// What supports the version.
    pub version_basis: VersionBasis,
    /// Version token, as exposed or declared.
    pub version: Option<String>,
    /// Verified weight or artifact identity.
    pub weights_identity: Option<String>,
}

/// The complete evaluated subject.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QualificationSubject {
    /// Lattice `lattice-worker-definition-v1` identity, `blake3:<hex>`.
    pub lattice_definition_identity: String,
    /// Lattice profile revision, bare 64 lowercase hex.
    pub lattice_profile_revision: String,
    /// Task family, a Lattice capability name.
    pub task_family: String,
    /// Every component kind exactly once.
    pub components: Vec<Component>,
    /// Model version facts.
    pub model: ModelFacts,
}

impl QualificationSubject {
    /// The component of `kind`.
    #[must_use]
    pub fn component(&self, kind: ComponentKind) -> Option<&Component> {
        self.components
            .iter()
            .find(|component| component.kind == kind)
    }
}

/// Independent evidence classes; never a ladder.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvidenceClass {
    /// Declared support.
    DeclaredSupport,
    /// Exact contract-test coverage.
    ContractConformance,
    /// Observed task-family evaluation.
    TaskFamilyEvaluation,
    /// Host-supplied deployment or promotion admission.
    DeploymentAdmission,
}

impl EvidenceClass {
    /// Wire spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DeclaredSupport => "declared_support",
            Self::ContractConformance => "contract_conformance",
            Self::TaskFamilyEvaluation => "task_family_evaluation",
            Self::DeploymentAdmission => "deployment_admission",
        }
    }

    /// Parses a wire spelling.
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        [
            Self::DeclaredSupport,
            Self::ContractConformance,
            Self::TaskFamilyEvaluation,
            Self::DeploymentAdmission,
        ]
        .into_iter()
        .find(|class| class.as_str() == value)
    }

    /// Whether the class reports outcome counts over a denominator.
    #[must_use]
    pub const fn has_outcomes(self) -> bool {
        matches!(self, Self::ContractConformance | Self::TaskFamilyEvaluation)
    }
}

/// Exact source, protocol, corpus, and verifier identities.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvidenceSource {
    /// Evidence class.
    pub class: EvidenceClass,
    /// Protocol identity: task protocol, declaration schema, or admission policy.
    pub protocol_identity: String,
    /// Verifier identity: independent verifier, declaring owner, or admitting host.
    pub verifier_identity: String,
    /// Corpus identity for classes with outcomes.
    pub corpus_identity: Option<String>,
    /// Identity of the source run, declaration, or admission record.
    pub source_identity: String,
}

/// Outcome counts; `planned` is the denominator and every other count sums to it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Outcomes {
    /// Planned trials or checks.
    pub planned: u64,
    /// Independently accepted completions.
    pub accepted: u64,
    /// Completed but rejected by the independent verifier.
    pub rejected: u64,
    /// Failed trials.
    pub failed: u64,
    /// Cancelled trials.
    pub cancelled: u64,
    /// Incomplete or timed-out trials.
    pub incomplete: u64,
}

/// Known costs; `None` is unknown, never zero.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Costs {
    /// Cost unit token.
    pub unit: String,
    /// Model cost.
    pub model: Option<u64>,
    /// Tool cost.
    pub tool: Option<u64>,
    /// Manager cost.
    pub manager: Option<u64>,
    /// Verifier cost.
    pub verifier: Option<u64>,
    /// Whether some cost was not measured at all.
    pub unknown_remainder: bool,
}

/// Covered time per phase in milliseconds; `None` is unmeasured.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[expect(
    clippy::struct_field_names,
    reason = "every covered time is milliseconds; the suffix names the unit"
)]
pub struct CoveredTime {
    /// Queue time.
    pub queue_ms: Option<u64>,
    /// Repair time.
    pub repair_ms: Option<u64>,
    /// Verification time.
    pub verification_ms: Option<u64>,
}

/// Task-family measures reported by the existing metrics owners.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Measures {
    /// Trials per task.
    pub repetitions: u64,
    /// Tasks with at least one completed trial.
    pub tasks_covered: u64,
    /// Tasks the protocol declares.
    pub tasks_declared: u64,
    /// Total time to acceptance in milliseconds.
    pub time_to_acceptance_ms: Option<u64>,
    /// Covered queue, repair, and verification time.
    pub covered_time: CoveredTime,
    /// Known costs and unknown remainder.
    pub costs: Costs,
    /// Declared review or intervention count.
    pub review_interventions: Option<u64>,
    /// Invalid actions.
    pub invalid_actions: Option<u64>,
    /// False completion attempts.
    pub false_completion_attempts: Option<u64>,
}

/// Scope, limitations, and missing modes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Scope {
    /// Corpus partitions covered, sorted and unique.
    pub partitions: Vec<String>,
    /// Limitations, sorted, unique, and non-empty.
    pub limitations: Vec<String>,
    /// Modes the evidence does not cover, sorted and unique.
    pub missing_modes: Vec<String>,
}

/// Fixture or live evidence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvidenceLabel {
    /// Checked fixture evidence.
    Fixture,
    /// Live run evidence.
    Live,
}

impl EvidenceLabel {
    /// Wire spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Fixture => "fixture",
            Self::Live => "live",
        }
    }
}

/// Provenance and freshness.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Provenance {
    /// Fixture or live.
    pub label: EvidenceLabel,
    /// Producer token.
    pub producer: String,
    /// Producer revision token.
    pub producer_revision: String,
    /// Host-supplied observation time, milliseconds since the epoch.
    pub observed_at_ms: u64,
    /// First time the evidence is no longer current.
    pub valid_until_ms: u64,
}

/// Bounded artifact or ledger reference.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Reference {
    /// Reference kind token.
    pub kind: String,
    /// `blake3:` identity.
    pub identity: String,
}

/// One attachment document before encoding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttachmentDocument {
    /// The complete subject.
    pub subject: QualificationSubject,
    /// Evidence class and source identities.
    pub evidence: EvidenceSource,
    /// Outcome counts for classes with outcomes.
    pub outcomes: Option<Outcomes>,
    /// Measures for task-family evaluations.
    pub measures: Option<Measures>,
    /// Scope.
    pub scope: Scope,
    /// Provenance.
    pub provenance: Provenance,
    /// References, sorted and unique.
    pub references: Vec<Reference>,
}

/// A canonical, structurally valid attachment.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Attachment {
    /// `blake3:` over the canonical bytes; Lattice's `qualification_ref`.
    pub identity: String,
    /// Canonical bytes.
    pub canonical: Vec<u8>,
    /// The decoded document.
    pub document: AttachmentDocument,
}

/// Closed rejection reasons, ordered by validation stage.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum RejectionReason {
    /// Bytes, shape, spelling, bounds, or redaction failed.
    Malformed,
    /// Definition identity or profile revision differs.
    SubjectMismatch,
    /// Task family is outside the attachment's scope.
    ScopeMismatch,
    /// Evidence class is not the required class.
    InsufficientClass,
    /// A required component is unavailable now or in the evidence.
    Unusable,
    /// Freshness expired or a current component changed.
    Stale,
    /// Distinct attachments claim the same source differently.
    Conflicting,
}

impl RejectionReason {
    /// Wire spelling, equal to Lattice's `NotQualified` reason.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Malformed => "malformed",
            Self::SubjectMismatch => "subject_mismatch",
            Self::ScopeMismatch => "scope_mismatch",
            Self::InsufficientClass => "insufficient_class",
            Self::Unusable => "unusable",
            Self::Stale => "stale",
            Self::Conflicting => "conflicting",
        }
    }
}

/// A rejection with the field that caused it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Rejection {
    /// Closed reason.
    pub reason: RejectionReason,
    /// Stable field label.
    pub field: &'static str,
}

impl Rejection {
    pub(crate) const fn new(reason: RejectionReason, field: &'static str) -> Self {
        Self { reason, field }
    }
}

/// The current component identity a consumer compares against.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CurrentComponent<'a> {
    /// Component kind.
    pub kind: ComponentKind,
    /// Current identity; `None` when currently unavailable.
    pub identity: Option<&'a str>,
}

/// What the consumer asks to be qualified.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExpectedSubject<'a> {
    /// Lattice definition identity, `blake3:<hex>`.
    pub lattice_definition_identity: &'a str,
    /// Lattice profile revision, bare 64 lowercase hex.
    pub lattice_profile_revision: &'a str,
    /// Task family.
    pub task_family: &'a str,
    /// Class the promotion policy requires.
    pub required_class: EvidenceClass,
    /// Consumer's current time, milliseconds since the epoch.
    pub now_ms: u64,
    /// Components whose evidence must not be unavailable.
    pub required_components: &'a [ComponentKind],
    /// Current identities of components outside the profile revision.
    pub current_components: &'a [CurrentComponent<'a>],
}

/// Facts of a qualified answer: admissible evidence exists for the exact
/// subject, task family, and class. It is not a favorable verdict; read the
/// outcomes from the attachment.
#[derive(Clone, Debug, Eq, PartialEq)]
#[expect(
    clippy::struct_field_names,
    reason = "`qualification_ref` is Lattice's port field spelling"
)]
pub struct Qualification {
    /// Attachment identity.
    pub qualification_ref: String,
    /// Evidence class.
    pub evidence_class: EvidenceClass,
    /// Protocol identity.
    pub protocol_identity: String,
    /// Verifier identity.
    pub verifier_identity: String,
    /// Observation time, milliseconds since the epoch.
    pub observed_at_ms: u64,
    /// Provenance label; fixture evidence is not live evidence.
    pub provenance_label: EvidenceLabel,
}

/// Port answer for one expected subject over a set of attachments.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QualificationAnswer {
    /// One current attachment is admissible evidence for the exact subject.
    Qualified(Qualification),
    /// No attachment qualifies; the furthest-stage reason.
    NotQualified(RejectionReason),
    /// No attachment was supplied.
    Unavailable,
}

/// An explicit, scoped cross-revision compatibility decision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompatibilityDecision {
    /// Identity of the reviewed decision record.
    pub decision_identity: String,
    /// The one attachment the decision covers.
    pub attachment_identity: String,
    /// Profile revision the evidence was produced for.
    pub from_profile_revision: String,
    /// Profile revision the decision allows reuse under.
    pub to_profile_revision: String,
    /// Task family the decision covers.
    pub task_family: String,
    /// Evidence class the decision covers.
    pub evidence_class: EvidenceClass,
    /// Components whose change the decision accepts.
    pub waived: Vec<ComponentKind>,
}

/// Applicability of one attachment to the complete current subject.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Applicability {
    /// Every key and component equals the current subject and evidence is fresh.
    Current,
    /// Historical evidence: retained, never current.
    Historical {
        /// Components that changed.
        changed: Vec<ComponentKind>,
        /// Whether the profile revision changed.
        revision_changed: bool,
        /// Whether freshness expired.
        expired: bool,
    },
    /// Reuse under an explicit scoped decision; claims are unchanged and the
    /// answer is never `Qualified`.
    CompatibleReuse {
        /// Decision identity.
        decision_identity: String,
        /// Components the decision waived.
        changed: Vec<ComponentKind>,
    },
    /// A required component is unavailable.
    Unusable {
        /// The component.
        component: ComponentKind,
    },
    /// The definition or task family differs; no reuse is possible.
    Inapplicable(RejectionReason),
}

/// Cost per accepted completion.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CostPerAccepted {
    /// Every cost is known: `known_total / accepted`.
    Defined {
        /// Sum of known costs.
        known_total: u64,
        /// Accepted completions.
        accepted: u64,
    },
    /// Nothing was accepted.
    NoAcceptedCompletion,
    /// Some cost is unknown.
    UnknownCost,
    /// The class reports no costs.
    NotApplicable,
}

/// Result of an equal-budget comparison.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EqualBudgetComparison {
    /// The candidate has more independently accepted completions.
    CandidateMoreAccepted {
        /// Baseline accepted.
        baseline: u64,
        /// Candidate accepted.
        candidate: u64,
    },
    /// The candidate has no more accepted completions.
    NoAcceptedGain {
        /// Baseline accepted.
        baseline: u64,
        /// Candidate accepted.
        candidate: u64,
    },
}

/// Why two attachments cannot be compared.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ComparisonRefusal {
    /// One side is not a task-family evaluation.
    NotEvaluation,
    /// Task family, denominator, or a control component differs.
    UnmatchedControls(Option<ComponentKind>),
}

/// Result of replaying stored attachments without rerunning anything.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ReplayReport {
    /// Distinct valid attachment identities, in input order.
    pub accepted: Vec<String>,
    /// Exact duplicates collapsed onto an earlier identity.
    pub duplicates: u64,
    /// Input index and reason of each rejected document.
    pub rejected: Vec<(u64, RejectionReason)>,
    /// Source identities claimed by more than one distinct attachment.
    pub conflicting_sources: Vec<String>,
}

/// Current-subject policy for applicability classification.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ApplicabilityPolicy<'a> {
    /// Consumer's current time, milliseconds since the epoch.
    pub now_ms: u64,
    /// Components whose evidence must not be unavailable.
    pub required: &'a [ComponentKind],
}

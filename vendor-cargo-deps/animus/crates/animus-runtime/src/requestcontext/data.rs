//! Typed request-context contribution and skill-turn data.

macro_rules! nominal_identity {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
        pub struct $name(String);

        impl $name {
            /// Creates an identity before validation.
            #[must_use]
            pub fn new(value: impl Into<String>) -> Self {
                Self(value.into())
            }

            /// Returns the identity text.
            #[must_use]
            pub fn as_str(&self) -> &str {
                self.0.as_str()
            }
        }
    };
}

nominal_identity!(ContributionIdentity, "Nominal contribution identity.");
nominal_identity!(PayloadIdentity, "BLAKE3 payload identity.");
nominal_identity!(
    ContributionPlanIdentity,
    "BLAKE3 contribution-plan identity."
);
nominal_identity!(SessionIdentity, "Nominal session identity.");
nominal_identity!(LogicalTurnIdentity, "Nominal logical-turn identity.");
nominal_identity!(ModelTargetIdentity, "Nominal model-target identity.");
nominal_identity!(ModelProfileIdentity, "Nominal model-profile identity.");
nominal_identity!(PromptPlanIdentity, "Nominal prompt-plan identity.");
nominal_identity!(
    ModelInvocationIdentity,
    "Nominal model-invocation identity."
);
nominal_identity!(PolicyIdentity, "Nominal policy identity.");
nominal_identity!(AuthorityIdentity, "Nominal authority-ceiling identity.");
nominal_identity!(
    ProviderRequestIdentity,
    "Nominal provider-request identity."
);
nominal_identity!(
    ProviderResponseIdentity,
    "Nominal provider-response identity."
);
nominal_identity!(SkillManifestIdentity, "Nominal skill-manifest identity.");
nominal_identity!(SkillSelectionIdentity, "Nominal skill-selection identity.");
nominal_identity!(ReceiptIdentity, "BLAKE3 metadata-only receipt identity.");

/// Existing owner class that produced an admitted contribution.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ContributionSource {
    /// Existing prompt owner.
    Prompt,
    /// Existing skill owner.
    Skill,
    /// Existing memory owner.
    Memory,
    /// Existing project-context owner.
    ProjectContext,
    /// Existing MCP owner.
    Mcp,
    /// Existing runtime owner.
    Runtime,
}

impl ContributionSource {
    /// Returns the stable source label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Prompt => "prompt",
            Self::Skill => "skill",
            Self::Memory => "memory",
            Self::ProjectContext => "project-context",
            Self::Mcp => "mcp",
            Self::Runtime => "runtime",
        }
    }
}

/// Trust class supplied by the current owner.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ContributionTrust {
    /// Owner admitted the contribution as trusted context.
    Trusted,
    /// Owner classified the contribution as untrusted context.
    Untrusted,
}

impl ContributionTrust {
    /// Returns the stable trust label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Trusted => "trusted",
            Self::Untrusted => "untrusted",
        }
    }
}

/// Whether omission can change the admitted request meaning.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ContributionPosture {
    /// Omission blocks provider composition.
    Mandatory,
    /// Omission is allowed with bounded diagnostics.
    Advisory,
}

impl ContributionPosture {
    /// Returns the stable posture label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Mandatory => "mandatory",
            Self::Advisory => "advisory",
        }
    }
}

/// Finite contribution lifetime.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ContributionLifetime {
    /// Expires after the current provider request is admitted.
    CurrentRequest,
    /// Expires after the next correlated provider response becomes terminal.
    UntilNextProviderResponse,
    /// Expires when the current logical turn becomes terminal.
    CurrentLogicalTurn,
}

impl ContributionLifetime {
    /// Returns the stable lifetime label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CurrentRequest => "current-request",
            Self::UntilNextProviderResponse => "until-next-provider-response",
            Self::CurrentLogicalTurn => "current-logical-turn",
        }
    }
}

/// Exact terminal observation that can expire one finite contribution lifetime.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContributionTerminalObservation {
    /// The bound provider request was admitted.
    RequestAdmitted(ProviderRequestIdentity),
    /// The bound provider request reached a terminal response.
    ProviderResponse {
        /// Exact request identity.
        request_id: ProviderRequestIdentity,
        /// Exact response identity.
        response_id: ProviderResponseIdentity,
    },
    /// The bound logical turn reached terminal completion.
    LogicalTurnCompleted(LogicalTurnIdentity),
}

/// Pure finite-lifetime transition result.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContributionLifetimeState {
    /// The contribution remains available before its terminal observation.
    Active,
    /// The exact terminal observation consumed the contribution lifetime.
    Expired,
}

/// Payload sensitivity class.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ContributionSensitivity {
    /// Payload is admitted as public request context.
    Public,
    /// Payload is valid only for the current request flow.
    RequestLocal,
    /// Payload contains sensitive request material.
    Sensitive,
}

impl ContributionSensitivity {
    /// Returns the stable sensitivity label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Public => "public",
            Self::RequestLocal => "request-local",
            Self::Sensitive => "sensitive",
        }
    }
}

/// Borrowed draft supplied by an existing context owner.
#[derive(Clone, Copy, Debug)]
pub struct ContributionDraft<'a> {
    /// Contribution identity.
    pub contribution_id: &'a str,
    /// Existing owner class.
    pub source: ContributionSource,
    /// Owner-supplied trust class.
    pub trust: ContributionTrust,
    /// Mandatory or advisory posture.
    pub posture: ContributionPosture,
    /// Higher values select first within the same posture.
    pub priority: i16,
    /// Finite prompt-budget cost.
    pub budget_cost: u32,
    /// Lowercase BLAKE3 payload identity.
    pub payload_identity: &'a str,
    /// Safe exact-deduplication key.
    pub deduplication_key: &'a str,
    /// Contribution identities explicitly superseded by this contribution.
    pub supersedes: &'a [&'a str],
    /// Optional safe conflict-group label.
    pub conflict_group: Option<&'a str>,
    /// Finite lifetime.
    pub lifetime: ContributionLifetime,
    /// Payload sensitivity.
    pub sensitivity: ContributionSensitivity,
    /// Safe diagnostic label.
    pub safe_label: &'a str,
}

/// Validated contribution metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Contribution {
    pub(super) id: ContributionIdentity,
    pub(super) source: ContributionSource,
    pub(super) trust: ContributionTrust,
    pub(super) posture: ContributionPosture,
    pub(super) priority: i16,
    pub(super) budget_cost: u32,
    pub(super) payload_identity: PayloadIdentity,
    pub(super) deduplication_key: String,
    pub(super) supersedes: Vec<ContributionIdentity>,
    pub(super) conflict_group: Option<String>,
    pub(super) lifetime: ContributionLifetime,
    pub(super) sensitivity: ContributionSensitivity,
    pub(super) safe_label: String,
}

impl Contribution {
    /// Returns the contribution identity.
    #[must_use]
    pub const fn contribution_id(&self) -> &ContributionIdentity {
        &self.id
    }

    /// Returns the owner class.
    #[must_use]
    pub const fn source(&self) -> ContributionSource {
        self.source
    }

    /// Returns the trust class.
    #[must_use]
    pub const fn trust(&self) -> ContributionTrust {
        self.trust
    }

    /// Returns the contribution posture.
    #[must_use]
    pub const fn posture(&self) -> ContributionPosture {
        self.posture
    }

    /// Returns the declared priority.
    #[must_use]
    pub const fn priority(&self) -> i16 {
        self.priority
    }

    /// Returns the finite budget cost.
    #[must_use]
    pub const fn budget_cost(&self) -> u32 {
        self.budget_cost
    }

    /// Returns the payload identity.
    #[must_use]
    pub const fn payload_identity(&self) -> &PayloadIdentity {
        &self.payload_identity
    }

    /// Returns the finite lifetime.
    #[must_use]
    pub const fn lifetime(&self) -> ContributionLifetime {
        self.lifetime
    }

    /// Returns the sensitivity class.
    #[must_use]
    pub const fn sensitivity(&self) -> ContributionSensitivity {
        self.sensitivity
    }

    /// Returns the safe diagnostic label.
    #[must_use]
    pub fn safe_label(&self) -> &str {
        self.safe_label.as_str()
    }
}

/// Exact model and policy binding for one reduction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContributionBinding {
    /// Exact target identity.
    pub target: ModelTargetIdentity,
    /// Exact model-profile identity.
    pub profile: ModelProfileIdentity,
    /// Existing prompt-plan identity.
    pub prompt_plan: PromptPlanIdentity,
    /// Existing immutable model-invocation identity.
    pub invocation: ModelInvocationIdentity,
    /// Current policy identity.
    pub policy: PolicyIdentity,
    /// Current authority-ceiling identity.
    pub authority: AuthorityIdentity,
    /// Planned provider request identity.
    pub provider_request: ProviderRequestIdentity,
}

/// Finite reducer policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ContributionPolicy {
    /// Maximum admitted candidate count.
    pub max_contributions: u16,
    /// Total context budget.
    pub context_budget: u32,
    /// Maximum retained omission diagnostics.
    pub max_omissions: u16,
}

/// Borrowed input for reduction with caller-declared mandatory identities.
#[derive(Clone, Copy, Debug)]
pub struct ContributionReductionInput<'a> {
    /// Admitted contribution candidates.
    pub candidates: &'a [Contribution],
    /// Contribution identities that must exist with mandatory posture.
    pub required: &'a [ContributionIdentity],
    /// Exact invocation binding.
    pub binding: &'a ContributionBinding,
    /// Finite reduction policy.
    pub policy: ContributionPolicy,
}

/// Why an advisory contribution was omitted.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ContributionOmissionClass {
    /// Another admitted contribution explicitly superseded it.
    Superseded,
    /// An equal deduplication group selected another contribution.
    Deduplicated,
    /// A conflict group selected another contribution.
    Conflict,
    /// Remaining context budget could not admit it.
    Budget,
}

impl ContributionOmissionClass {
    /// Returns the stable omission label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Superseded => "superseded",
            Self::Deduplicated => "deduplicated",
            Self::Conflict => "conflict",
            Self::Budget => "budget",
        }
    }
}

/// Bounded metadata-only omission.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContributionOmission {
    /// Omitted contribution identity.
    pub contribution_id: ContributionIdentity,
    /// Omission class.
    pub class: ContributionOmissionClass,
    /// Safe contribution label.
    pub safe_label: String,
}

/// Selected request-context plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContributionPlan {
    /// BLAKE3 plan identity.
    pub plan_id: ContributionPlanIdentity,
    /// Exact invocation binding.
    pub binding: ContributionBinding,
    /// Selected contributions in canonical prompt order.
    pub selected: Vec<Contribution>,
    /// Bounded omission metadata.
    pub omissions: Vec<ContributionOmission>,
    /// Total selected budget cost.
    pub selected_budget: u32,
}

/// Borrowed payload supplied only at prompt-assembly time.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ContributionPayloadView<'a> {
    /// Contribution identity.
    pub contribution_id: &'a ContributionIdentity,
    /// Exact payload identity.
    pub payload_identity: &'a PayloadIdentity,
    /// Request-local payload body.
    pub body: &'a str,
}

/// Exact skill selection before body loading.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SkillSelectionBinding {
    /// Current session.
    pub session: SessionIdentity,
    /// Current logical turn.
    pub turn: LogicalTurnIdentity,
    /// Exact provider target.
    pub target: ModelTargetIdentity,
    /// Exact model profile.
    pub profile: ModelProfileIdentity,
    /// Exact admitted manifest.
    pub manifest: SkillManifestIdentity,
    /// Expected current body identity.
    pub body: PayloadIdentity,
    /// Selection identity.
    pub selection: SkillSelectionIdentity,
    /// Contribution identity reserved for the body.
    pub contribution: ContributionIdentity,
}

/// Finite policy for one selected skill contribution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SkillContributionPolicy<'a> {
    /// Selection priority.
    pub priority: i16,
    /// Finite context-budget cost.
    pub budget_cost: u32,
    /// Safe metadata label.
    pub safe_label: &'a str,
}

/// Request sent to an injected skill-body service.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SkillBodyLoad {
    /// Exact selection binding.
    pub binding: SkillSelectionBinding,
}

/// Body returned by an injected skill-body service.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoadedSkillBody {
    /// Exact body identity measured by the host.
    pub body_id: PayloadIdentity,
    /// Current skill body, never suitable for receipt projection.
    pub body: String,
}

/// Safe skill-selection result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SkillSelectionResult {
    /// Selection identity.
    pub selection_id: SkillSelectionIdentity,
    /// Manifest identity.
    pub manifest_id: SkillManifestIdentity,
    /// Body identity.
    pub body_id: PayloadIdentity,
    /// Contribution identity.
    pub contribution_id: ContributionIdentity,
    /// Finite next-response lifetime.
    pub lifetime: ContributionLifetime,
    /// Whether one terminal next response is required.
    pub next_response_required: bool,
}

/// Skill-turn lifecycle state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SkillTurnState {
    /// Exact selection exists, but no body has been loaded.
    Selected,
    /// Exact current body was loaded.
    BodyLoaded,
    /// One provider request consumed the contribution.
    RequestBound {
        /// Exact request identity.
        request_id: ProviderRequestIdentity,
        /// Latest admitted same-request attempt.
        attempt: u16,
    },
    /// Correlated provider response reached a terminal outcome.
    Terminal {
        /// Exact response identity.
        response_id: ProviderResponseIdentity,
        /// Terminal outcome class.
        outcome: ProviderTerminalOutcome,
    },
    /// Selection expired without successful consumption.
    Expired,
}

/// Complete pure skill-turn state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SkillTurn {
    /// Exact selection binding.
    pub binding: SkillSelectionBinding,
    /// Current lifecycle state.
    pub state: SkillTurnState,
}

/// Exact provider request observation for skill consumption.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SkillProviderObservation {
    /// Provider request identity.
    pub request_id: ProviderRequestIdentity,
    /// Exact target identity.
    pub target_id: ModelTargetIdentity,
    /// Exact model-profile identity.
    pub profile_id: ModelProfileIdentity,
    /// Exact selected contribution-plan identity.
    pub contribution_plan_id: ContributionPlanIdentity,
    /// Whether that plan contains the selected skill contribution.
    pub contribution_selected: bool,
    /// Same-request attempt number.
    pub attempt: u16,
}

/// Compatibility alias for the original host request DTO.
pub use SkillBodyLoad as SkillBodyLoadRequest;
/// Compatibility alias for the original provider observation DTO.
pub use SkillProviderObservation as SkillProviderRequestObservation;

/// Terminal provider outcome.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProviderTerminalOutcome {
    /// Provider returned a terminal response.
    Completed,
    /// Provider returned a terminal failure.
    Failed,
    /// Current request was cancelled terminally.
    Cancelled,
}

/// Exact terminal provider response observation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SkillProviderResponseObservation {
    /// Correlated request identity.
    pub request_id: ProviderRequestIdentity,
    /// Response identity.
    pub response_id: ProviderResponseIdentity,
    /// Terminal outcome.
    pub outcome: ProviderTerminalOutcome,
}

/// Durable sink class that cannot receive request-local payloads.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RetentionSink {
    /// Session conversation ledger.
    Conversation,
    /// Compaction input.
    Compaction,
    /// Memory generation or storage.
    Memory,
    /// Quality record.
    Quality,
    /// Lifecycle or release receipt.
    Receipt,
}

/// Metadata-only contribution receipt entry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContributionReceiptEntry {
    /// Contribution identity.
    pub contribution_id: ContributionIdentity,
    /// Owner class.
    pub source: ContributionSource,
    /// Trust class.
    pub trust: ContributionTrust,
    /// Posture.
    pub posture: ContributionPosture,
    /// Priority.
    pub priority: i16,
    /// Budget cost.
    pub budget_cost: u32,
    /// Payload identity.
    pub payload_identity: PayloadIdentity,
    /// Deduplication key.
    pub deduplication_key: String,
    /// Explicitly superseded contribution identities.
    pub supersedes: Vec<ContributionIdentity>,
    /// Optional conflict group.
    pub conflict_group: Option<String>,
    /// Lifetime.
    pub lifetime: ContributionLifetime,
    /// Sensitivity class.
    pub sensitivity: ContributionSensitivity,
    /// Safe label.
    pub safe_label: String,
}

/// Metadata-only contribution receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContributionReceipt {
    /// BLAKE3 receipt identity.
    pub receipt_id: ReceiptIdentity,
    /// Plan identity.
    pub plan_id: ContributionPlanIdentity,
    /// Selected metadata.
    pub selected: Vec<ContributionReceiptEntry>,
    /// Omission metadata.
    pub omissions: Vec<ContributionOmission>,
    /// Selected finite budget.
    pub selected_budget: u32,
    /// Bounded claims.
    pub claims: Vec<String>,
    /// Required non-claims.
    pub non_claims: Vec<String>,
}

/// Candidate sensitive receipt fields.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ContributionReceiptSensitiveDraft<'a> {
    /// Prompt body.
    pub prompt_body: Option<&'a str>,
    /// Skill body.
    pub skill_body: Option<&'a str>,
    /// MCP instruction body.
    pub mcp_instructions: Option<&'a str>,
    /// Memory record.
    pub memory_record: Option<&'a str>,
    /// Project path.
    pub path: Option<&'a str>,
    /// Credential.
    pub credential: Option<&'a str>,
    /// Provider data.
    pub provider_data: Option<&'a str>,
    /// Hidden directive.
    pub hidden_directive: Option<&'a str>,
}

/// Fail-closed contribution or skill-turn issue.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContributionIssue {
    /// Required identity or safe label is absent.
    MissingIdentity,
    /// Identity is not lowercase BLAKE3 text where required.
    InvalidPayloadIdentity,
    /// Safe label, deduplication key, or conflict group is malformed.
    UnsafeMetadata,
    /// Candidate or collection exceeds a named bound.
    BoundExceeded,
    /// Contribution identity is duplicated.
    DuplicateContribution,
    /// A caller-declared mandatory contribution is absent.
    MissingMandatoryContribution,
    /// Supersession refers to the contribution itself or an unknown identity.
    InvalidSupersession,
    /// Mandatory contribution was superseded, deduplicated, or conflicted unsafely.
    MandatoryOmitted,
    /// Multiple mandatory contributions occupy one conflict group.
    MandatoryConflict,
    /// Mandatory contributions exceed the context budget.
    MandatoryBudgetExceeded,
    /// Exact model, profile, prompt, policy, authority, request, or plan binding drifted.
    InvocationDrift,
    /// Skill body or manifest binding is stale.
    StaleSkillBody,
    /// Skill lifecycle transition is invalid or stale.
    InvalidSkillTransition,
    /// Logical turn completion is still blocked by an unconsumed skill.
    SkillResponsePending,
    /// Request-local payload retention was requested.
    RetentionDenied,
    /// Receipt candidate contains sensitive material.
    ReceiptLeak,
    /// Receipt claims exceed the bounded evidence class.
    ReceiptOverclaim,
    /// Required receipt non-claim is absent.
    MissingNonClaim,
    /// Injected skill-body service failed.
    SkillBodyServiceFailed,
}

use alloc::string::String;
use alloc::vec::Vec;

/// Stable validated snapshot schema.
pub const MODEL_REQUEST_SNAPSHOT_SCHEMA: &str = "animus.model-request-snapshot.v2";
/// Stable private durable-entry schema.
pub const MODEL_REQUEST_SNAPSHOT_ENTRY_SCHEMA: &str =
    "animus.model-request-snapshot-private-entry.v2";
/// Stable metadata-only receipt schema.
pub const MODEL_REQUEST_SNAPSHOT_RECEIPT_SCHEMA: &str = "animus.model-request-snapshot-receipt.v2";
/// Stable snapshot identity domain.
pub const MODEL_REQUEST_SNAPSHOT_IDENTITY_DOMAIN: &str =
    "animus.model-request-snapshot.identity.v2";
/// Stable canonical request identity domain.
pub const MODEL_REQUEST_CANONICAL_IDENTITY_DOMAIN: &str =
    "animus.model-request-snapshot.canonical-request.v1";
/// Stable model-route identity domain.
pub const MODEL_REQUEST_ROUTE_IDENTITY_DOMAIN: &str = "animus.model-request-snapshot.route.v2";
/// Stable tool-schema-set identity domain.
pub const MODEL_REQUEST_TOOL_SET_IDENTITY_DOMAIN: &str =
    "animus.model-request-snapshot.tool-set.v1";
/// Stable output-control identity domain.
pub const MODEL_REQUEST_OUTPUT_IDENTITY_DOMAIN: &str = "animus.model-request-snapshot.output.v1";
/// Stable budget identity domain.
pub const MODEL_REQUEST_BUDGET_IDENTITY_DOMAIN: &str = "animus.model-request-snapshot.budget.v1";
/// Stable private payload domain.
pub const MODEL_REQUEST_PRIVATE_PAYLOAD_DOMAIN: &str =
    "animus.model-request-snapshot.private-payload.v2";
/// Stable commit-plan identity domain.
pub const MODEL_REQUEST_COMMIT_PLAN_IDENTITY_DOMAIN: &str =
    "animus.model-request-snapshot.commit-plan.v1";
/// Stable dispatch-plan identity domain.
pub const MODEL_REQUEST_DISPATCH_PLAN_IDENTITY_DOMAIN: &str =
    "animus.model-request-snapshot.dispatch-plan.v1";
/// Stable terminal-observation identity domain.
pub const MODEL_REQUEST_TERMINAL_IDENTITY_DOMAIN: &str =
    "animus.model-request-snapshot.terminal.v1";
/// Stable private terminal-observation payload domain.
pub const MODEL_REQUEST_TERMINAL_PRIVATE_PAYLOAD_DOMAIN: &str =
    "animus.model-request-snapshot.terminal-private-payload.v1";
/// Stable receipt identity domain.
pub const MODEL_REQUEST_SNAPSHOT_RECEIPT_IDENTITY_DOMAIN: &str =
    "animus.model-request-snapshot.receipt.v1";
/// Private durable record kind.
pub const MODEL_REQUEST_SNAPSHOT_RECORD_KIND: &str = "model_request_snapshot";
/// Private terminal provider record kind.
pub const MODEL_REQUEST_TERMINAL_RECORD_KIND: &str = "model_request_terminal";
/// Public redaction marker.
pub const MODEL_REQUEST_SNAPSHOT_REDACTION: &str =
    "model-visible-request-tool-schema-contribution-and-provider-material-omitted";
/// Required public claim boundaries.
pub const MODEL_REQUEST_SNAPSHOT_NON_CLAIMS: &[&str] = &[
    "not-provider-delivery",
    "not-provider-wire-equivalence",
    "not-hidden-provider-defaults",
    "not-credential-safety",
    "not-response-correctness",
    "not-safe-automatic-replay",
    "not-task-success",
    "not-production-readiness",
];

pub(crate) const MAX_SNAPSHOT_LABEL_BYTES: usize = 256;
pub(crate) const MAX_SNAPSHOT_IDENTITIES: usize = 256;
pub(crate) const MAX_PRIVATE_PAYLOAD_BYTES: usize = 8_388_608;
pub(crate) const BLAKE3_HEX_LENGTH: usize = 64;

/// Exact selected provider-neutral model route.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ModelRequestRouteDraft<'input> {
    /// Selected provider account or service ID.
    pub provider_id: &'input str,
    /// Selected versioned provider-transport ID.
    pub provider_transport_id: &'input str,
    /// Selected model ID.
    pub model_id: &'input str,
    /// Selected versioned model-facing dialect ID.
    pub model_dialect_id: &'input str,
    /// Recomputed route identity expected by the caller.
    pub route_identity: &'input str,
}

/// Candidate unsupported or out-of-boundary request material.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ModelRequestSnapshotUnsupportedDraft<'input> {
    /// Credential material.
    pub credential: Option<&'input str>,
    /// Transport header material.
    pub transport_header: Option<&'input str>,
    /// Provider SDK object or serialized provider-native object.
    pub provider_sdk_object: Option<&'input str>,
    /// Hidden provider default.
    pub hidden_provider_default: Option<&'input str>,
    /// Provider-native request payload.
    pub provider_payload: Option<&'input str>,
    /// Private filesystem path.
    pub private_path: Option<&'input str>,
}

/// Complete borrowed provider-neutral snapshot before validation.
#[derive(Clone, Copy, Debug)]
pub struct ModelRequestSnapshotDraft<'input> {
    /// Stable schema tag.
    pub schema: &'input str,
    /// Safe session lineage ID.
    pub session_id: &'input str,
    /// Safe branch lineage ID.
    pub branch_id: &'input str,
    /// Exact active leaf at request composition.
    pub active_leaf_id: &'input str,
    /// Current turn or authority generation.
    pub turn_generation: u64,
    /// Monotonic request sequence within the active turn.
    pub request_sequence: u64,
    /// Exact provider-neutral canonical request.
    pub canonical_request: &'input animus_adapters::CanonicalModelRequest,
    /// Exact selected model route.
    pub route: ModelRequestRouteDraft<'input>,
    /// Existing model-invocation plan identity.
    pub model_invocation_identity: &'input str,
    /// Existing request-context plan identity.
    pub request_context_plan_identity: &'input str,
    /// Ordered selected request-context contribution identities.
    pub selected_context_contribution_identities: &'input [&'input str],
    /// Recomputed tool-schema-set identity expected by the caller.
    pub tool_schema_set_identity: &'input str,
    /// Recomputed output-control identity expected by the caller.
    pub output_control_identity: &'input str,
    /// Recomputed budget identity expected by the caller.
    pub budget_identity: &'input str,
    /// Current runtime-authority seal identity.
    pub authority_seal_identity: &'input str,
    /// Exact provider-adapter cohort identity.
    pub adapter_cohort_identity: &'input str,
    /// Unsupported fields, all of which must be absent.
    pub unsupported: ModelRequestSnapshotUnsupportedDraft<'input>,
}

/// Validated private provider-neutral model-request snapshot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelRequestSnapshot {
    pub(super) schema: &'static str,
    pub(super) session_id: String,
    pub(super) branch_id: String,
    pub(super) active_leaf_id: String,
    pub(super) turn_generation: u64,
    pub(super) request_sequence: u64,
    pub(super) canonical_request: animus_adapters::CanonicalModelRequest,
    pub(super) canonical_request_identity: String,
    pub(super) provider_id: String,
    pub(super) provider_transport_id: String,
    pub(super) model_id: String,
    pub(super) model_dialect_id: String,
    pub(super) route_identity: String,
    pub(super) model_invocation_identity: String,
    pub(super) request_context_plan_identity: String,
    pub(super) selected_context_contribution_identities: Vec<String>,
    pub(super) tool_schema_set_identity: String,
    pub(super) output_control_identity: String,
    pub(super) budget_identity: String,
    pub(super) authority_seal_identity: String,
    pub(super) adapter_cohort_identity: String,
    pub(super) private_payload: Vec<u8>,
    pub(super) snapshot_identity: String,
}

impl ModelRequestSnapshot {
    /// Returns the stable schema.
    #[must_use]
    pub const fn schema(&self) -> &'static str {
        self.schema
    }

    /// Returns the session lineage ID.
    #[must_use]
    pub fn session_id(&self) -> &str {
        self.session_id.as_str()
    }

    /// Returns the branch lineage ID.
    #[must_use]
    pub fn branch_id(&self) -> &str {
        self.branch_id.as_str()
    }

    /// Returns the active leaf ID.
    #[must_use]
    pub fn active_leaf_id(&self) -> &str {
        self.active_leaf_id.as_str()
    }

    /// Returns the generation.
    #[must_use]
    pub const fn turn_generation(&self) -> u64 {
        self.turn_generation
    }

    /// Returns the request sequence.
    #[must_use]
    pub const fn request_sequence(&self) -> u64 {
        self.request_sequence
    }

    /// Reconstructs the exact provider-neutral request supplied to projection.
    #[must_use]
    pub const fn canonical_request(&self) -> &animus_adapters::CanonicalModelRequest {
        &self.canonical_request
    }

    /// Returns the canonical request identity.
    #[must_use]
    pub fn canonical_request_identity(&self) -> &str {
        self.canonical_request_identity.as_str()
    }

    /// Returns the selected provider ID.
    #[must_use]
    pub fn provider_id(&self) -> &str {
        self.provider_id.as_str()
    }

    /// Returns the selected versioned provider-transport ID.
    #[must_use]
    pub fn provider_transport_id(&self) -> &str {
        self.provider_transport_id.as_str()
    }

    /// Returns the selected model ID.
    #[must_use]
    pub fn model_id(&self) -> &str {
        self.model_id.as_str()
    }

    /// Returns the selected versioned model-facing dialect ID.
    #[must_use]
    pub fn model_dialect_id(&self) -> &str {
        self.model_dialect_id.as_str()
    }

    /// Returns the route identity.
    #[must_use]
    pub fn route_identity(&self) -> &str {
        self.route_identity.as_str()
    }

    /// Returns the model-invocation identity.
    #[must_use]
    pub fn model_invocation_identity(&self) -> &str {
        self.model_invocation_identity.as_str()
    }

    /// Returns the request-context plan identity.
    #[must_use]
    pub fn request_context_plan_identity(&self) -> &str {
        self.request_context_plan_identity.as_str()
    }

    /// Returns ordered selected context-contribution identities.
    #[must_use]
    pub fn selected_context_contribution_identities(&self) -> &[String] {
        self.selected_context_contribution_identities.as_slice()
    }

    /// Returns the tool-schema-set identity.
    #[must_use]
    pub fn tool_schema_set_identity(&self) -> &str {
        self.tool_schema_set_identity.as_str()
    }

    /// Returns the output-control identity.
    #[must_use]
    pub fn output_control_identity(&self) -> &str {
        self.output_control_identity.as_str()
    }

    /// Returns the budget identity.
    #[must_use]
    pub fn budget_identity(&self) -> &str {
        self.budget_identity.as_str()
    }

    /// Returns the runtime-authority seal identity.
    #[must_use]
    pub fn authority_seal_identity(&self) -> &str {
        self.authority_seal_identity.as_str()
    }

    /// Returns the adapter cohort identity.
    #[must_use]
    pub fn adapter_cohort_identity(&self) -> &str {
        self.adapter_cohort_identity.as_str()
    }

    /// Returns the exact versioned private durable payload.
    #[must_use]
    pub fn private_payload(&self) -> &[u8] {
        self.private_payload.as_slice()
    }

    /// Returns the complete snapshot identity.
    #[must_use]
    pub fn snapshot_identity(&self) -> &str {
        self.snapshot_identity.as_str()
    }

    /// Returns the number of ordered canonical messages.
    #[must_use]
    pub fn message_count(&self) -> usize {
        self.canonical_request.messages.len()
    }

    /// Returns the number of system and developer sections.
    #[must_use]
    pub fn system_section_count(&self) -> usize {
        self.canonical_request
            .messages
            .iter()
            .filter(|message| {
                matches!(
                    message.role(),
                    animus_adapters::CanonicalModelRole::System
                        | animus_adapters::CanonicalModelRole::Developer
                )
            })
            .count()
    }

    /// Returns the number of canonical tool declarations.
    #[must_use]
    pub fn tool_count(&self) -> usize {
        self.canonical_request.tools.len()
    }
}

/// Exact private durable append plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelRequestSnapshotCommitPlan {
    pub(super) entry_schema: &'static str,
    pub(super) record_kind: &'static str,
    pub(super) entry_id: String,
    pub(super) parent_id: String,
    pub(super) prior_head_hash: String,
    pub(super) snapshot: ModelRequestSnapshot,
    pub(super) plan_identity: String,
}

impl ModelRequestSnapshotCommitPlan {
    /// Returns the private entry schema.
    #[must_use]
    pub const fn entry_schema(&self) -> &'static str {
        self.entry_schema
    }

    /// Returns the durable record kind.
    #[must_use]
    pub const fn record_kind(&self) -> &'static str {
        self.record_kind
    }

    /// Returns the deterministic private entry ID.
    #[must_use]
    pub fn entry_id(&self) -> &str {
        self.entry_id.as_str()
    }

    /// Returns the active leaf parent ID.
    #[must_use]
    pub fn parent_id(&self) -> &str {
        self.parent_id.as_str()
    }

    /// Returns the expected prior durable head.
    #[must_use]
    pub fn prior_head_hash(&self) -> &str {
        self.prior_head_hash.as_str()
    }

    /// Returns the private validated snapshot.
    #[must_use]
    pub const fn snapshot(&self) -> &ModelRequestSnapshot {
        &self.snapshot
    }

    /// Returns the commit-plan identity.
    #[must_use]
    pub fn plan_identity(&self) -> &str {
        self.plan_identity.as_str()
    }
}

/// Durable store observation for one private snapshot append.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelRequestSnapshotCommitObservation {
    /// Observed commit-plan identity.
    pub plan_identity: String,
    /// Observed snapshot identity.
    pub snapshot_identity: String,
    /// Observed session ID.
    pub session_id: String,
    /// Observed branch ID.
    pub branch_id: String,
    /// Observed active leaf ID.
    pub active_leaf_id: String,
    /// Observed turn generation.
    pub turn_generation: u64,
    /// Observed request sequence.
    pub request_sequence: u64,
    /// New durable head hash.
    pub new_head_hash: String,
    /// Durable cursor after append.
    pub durable_cursor: u64,
    /// True only for a complete append.
    pub complete: bool,
    /// True only when required synchronization completed.
    pub synchronized: bool,
}

/// Snapshot with exact durable commit evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommittedModelRequestSnapshot {
    pub(super) plan: ModelRequestSnapshotCommitPlan,
    pub(super) new_head_hash: String,
    pub(super) durable_cursor: u64,
}

impl CommittedModelRequestSnapshot {
    /// Returns the committed snapshot.
    #[must_use]
    pub const fn snapshot(&self) -> &ModelRequestSnapshot {
        self.plan.snapshot()
    }

    /// Returns the commit plan.
    #[must_use]
    pub const fn plan(&self) -> &ModelRequestSnapshotCommitPlan {
        &self.plan
    }

    /// Returns the new durable head.
    #[must_use]
    pub fn new_head_hash(&self) -> &str {
        self.new_head_hash.as_str()
    }

    /// Returns the durable cursor.
    #[must_use]
    pub const fn durable_cursor(&self) -> u64 {
        self.durable_cursor
    }
}

/// Current facts that must remain exact before provider contact.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ModelRequestDispatchFacts<'input> {
    /// Current active leaf.
    pub active_leaf_id: &'input str,
    /// Current turn generation.
    pub turn_generation: u64,
    /// Current request sequence.
    pub request_sequence: u64,
    /// Current model-invocation identity.
    pub model_invocation_identity: &'input str,
    /// Current request-context plan identity.
    pub request_context_plan_identity: &'input str,
    /// Current tool-schema-set identity.
    pub tool_schema_set_identity: &'input str,
    /// Current output-control identity.
    pub output_control_identity: &'input str,
    /// Current budget identity.
    pub budget_identity: &'input str,
    /// Current runtime-authority seal identity.
    pub authority_seal_identity: &'input str,
    /// Current adapter cohort identity.
    pub adapter_cohort_identity: &'input str,
}

/// Provider attempt class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModelRequestAttemptKind {
    /// First attempt for one committed request.
    Initial,
    /// Existing provider-retry owner admitted the exact same request.
    SameRequestRetry,
}

impl ModelRequestAttemptKind {
    /// Returns the stable class label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Initial => "initial",
            Self::SameRequestRetry => "same_request_retry",
        }
    }
}

/// Existing provider-retry owner's bounded decision.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModelRequestRetryOwnerDecision {
    /// The existing owner admitted the exact request.
    Admitted,
    /// The existing owner denied retry.
    Denied,
}

/// Exact provider dispatch plan bound to a committed private snapshot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelRequestDispatchPlan {
    pub(super) snapshot_identity: String,
    pub(super) canonical_request_identity: String,
    pub(super) adapter_cohort_identity: String,
    pub(super) active_leaf_id: String,
    pub(super) turn_generation: u64,
    pub(super) request_sequence: u64,
    pub(super) attempt_number: u32,
    pub(super) attempt_kind: ModelRequestAttemptKind,
    pub(super) plan_identity: String,
}

impl ModelRequestDispatchPlan {
    /// Returns the committed snapshot identity.
    #[must_use]
    pub fn snapshot_identity(&self) -> &str {
        self.snapshot_identity.as_str()
    }

    /// Returns the canonical request identity.
    #[must_use]
    pub fn canonical_request_identity(&self) -> &str {
        self.canonical_request_identity.as_str()
    }

    /// Returns the exact adapter cohort.
    #[must_use]
    pub fn adapter_cohort_identity(&self) -> &str {
        self.adapter_cohort_identity.as_str()
    }

    /// Returns the active leaf.
    #[must_use]
    pub fn active_leaf_id(&self) -> &str {
        self.active_leaf_id.as_str()
    }

    /// Returns the turn generation.
    #[must_use]
    pub const fn turn_generation(&self) -> u64 {
        self.turn_generation
    }

    /// Returns the request sequence.
    #[must_use]
    pub const fn request_sequence(&self) -> u64 {
        self.request_sequence
    }

    /// Returns the bounded attempt number.
    #[must_use]
    pub const fn attempt_number(&self) -> u32 {
        self.attempt_number
    }

    /// Returns the attempt class.
    #[must_use]
    pub const fn attempt_kind(&self) -> ModelRequestAttemptKind {
        self.attempt_kind
    }

    /// Returns the dispatch-plan identity.
    #[must_use]
    pub fn plan_identity(&self) -> &str {
        self.plan_identity.as_str()
    }
}

/// Correlated terminal provider status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModelRequestProviderTerminalStatus {
    /// A terminal provider success was observed.
    Succeeded,
    /// A terminal provider failure was observed.
    Failed,
    /// Correlated cancellation was observed.
    Cancelled,
}

impl ModelRequestProviderTerminalStatus {
    /// Returns the stable status label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }
}

/// Borrowed terminal provider facts before correlation validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ModelRequestProviderTerminalDraft<'input> {
    /// Snapshot identity supplied by the host.
    pub snapshot_identity: &'input str,
    /// Adapter cohort identity supplied by the host.
    pub adapter_cohort_identity: &'input str,
    /// Exact provider attempt number.
    pub attempt_number: u32,
    /// Safe terminal status.
    pub status: ModelRequestProviderTerminalStatus,
}

/// Metadata-only correlated terminal observation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelRequestProviderTerminalObservation {
    /// Snapshot identity.
    pub snapshot_identity: String,
    /// Dispatch-plan identity.
    pub dispatch_plan_identity: String,
    /// Adapter cohort identity.
    pub adapter_cohort_identity: String,
    /// Attempt number.
    pub attempt_number: u32,
    /// Terminal status.
    pub status: ModelRequestProviderTerminalStatus,
    /// Observation identity.
    pub observation_identity: String,
}

/// Restart classification for one committed snapshot.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModelRequestRecoveryStatus {
    /// A correlated terminal provider observation is durable.
    TerminalObserved,
    /// The request committed, but provider outcome is unknown.
    CommittedUnobserved,
}

impl ModelRequestRecoveryStatus {
    /// Returns the stable status label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::TerminalObserved => "terminal_observed",
            Self::CommittedUnobserved => "committed_unobserved",
        }
    }
}

/// Recovery decision with no automatic replay authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelRequestRecoveryDecision {
    /// Snapshot identity.
    pub snapshot_identity: String,
    /// Restart status.
    pub status: ModelRequestRecoveryStatus,
    /// True only when a separate provider-retry decision is still required.
    pub retry_owner_decision_required: bool,
    /// Automatic replay is always false.
    pub automatic_replay: bool,
}

/// Safe public receipt status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModelRequestSnapshotReceiptStatus {
    /// Private snapshot committed before provider contact.
    Committed,
    /// A correlated terminal provider observation is durable.
    TerminalObserved,
    /// Snapshot is committed without a terminal observation.
    CommittedUnobserved,
}

impl ModelRequestSnapshotReceiptStatus {
    /// Returns the stable status label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Committed => "committed",
            Self::TerminalObserved => "terminal_observed",
            Self::CommittedUnobserved => "committed_unobserved",
        }
    }
}

/// Candidate private fields that public evidence must reject.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ModelRequestSnapshotReceiptSensitiveDraft<'input> {
    /// Raw prompt or message body.
    pub raw_prompt: Option<&'input str>,
    /// Tool schema body.
    pub tool_schema: Option<&'input str>,
    /// Request-context contribution body.
    pub contribution_body: Option<&'input str>,
    /// Credential material.
    pub credential: Option<&'input str>,
    /// Provider-native payload.
    pub provider_payload: Option<&'input str>,
    /// Private path.
    pub private_path: Option<&'input str>,
    /// Raw error.
    pub raw_error: Option<&'input str>,
}

/// Bounded metadata-only public receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelRequestSnapshotReceipt {
    /// Receipt schema.
    pub schema: &'static str,
    /// Snapshot identity.
    pub snapshot_identity: String,
    /// Canonical request identity.
    pub canonical_request_identity: String,
    /// Model route identity.
    pub route_identity: String,
    /// Model-invocation identity.
    pub model_invocation_identity: String,
    /// Request-context plan identity.
    pub request_context_plan_identity: String,
    /// Tool-schema-set identity.
    pub tool_schema_set_identity: String,
    /// Output-control identity.
    pub output_control_identity: String,
    /// Budget identity.
    pub budget_identity: String,
    /// Runtime-authority seal identity.
    pub authority_seal_identity: String,
    /// Adapter cohort identity.
    pub adapter_cohort_identity: String,
    /// Ordered message count.
    pub message_count: u32,
    /// System/developer section count.
    pub system_section_count: u32,
    /// Selected request-context contribution count.
    pub contribution_count: u32,
    /// Tool declaration count.
    pub tool_count: u32,
    /// Safe status.
    pub status: ModelRequestSnapshotReceiptStatus,
    /// Redaction marker.
    pub redaction_marker: &'static str,
    /// Explicit claim boundaries.
    pub non_claims: Vec<String>,
    /// Receipt identity.
    pub receipt_identity: String,
}

/// Stable typed rejection classes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModelRequestSnapshotIssue {
    /// Required schema or section metadata is missing or unsupported.
    InvalidSchema,
    /// Safe lineage or cohort label is invalid.
    InvalidLabel,
    /// Required BLAKE3 identity is invalid.
    InvalidIdentity,
    /// A recomputed identity does not match the draft.
    IdentityMismatch,
    /// A required request collection is empty.
    MissingSection,
    /// A selected identity occurs more than once.
    DuplicateIdentity,
    /// An explicit bound was exceeded.
    BoundExceeded,
    /// Credential, transport, provider-native, or hidden-default material was supplied.
    UnsupportedField,
    /// Durable commit evidence is partial, stale, or mismatched.
    CommitMismatch,
    /// Current composition, authority, or adapter facts drifted.
    DispatchDrift,
    /// The existing provider-retry owner denied or mismatched the request.
    RetryDenied,
    /// Terminal provider evidence is stale or mismatched.
    TerminalMismatch,
    /// Public evidence contains private data or an overclaim.
    UnsafeEvidence,
}

/// Deterministic snapshot rejection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelRequestSnapshotError {
    issue: ModelRequestSnapshotIssue,
    field: &'static str,
}

impl ModelRequestSnapshotError {
    #[must_use]
    pub(super) const fn new(issue: ModelRequestSnapshotIssue, field: &'static str) -> Self {
        Self { issue, field }
    }

    /// Returns the stable issue class.
    #[must_use]
    pub const fn issue(&self) -> ModelRequestSnapshotIssue {
        self.issue
    }

    /// Returns the safe field class.
    #[must_use]
    pub const fn field(&self) -> &'static str {
        self.field
    }
}

impl core::fmt::Display for ModelRequestSnapshotError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "model request snapshot rejected {}", self.field)
    }
}

impl core::error::Error for ModelRequestSnapshotError {}

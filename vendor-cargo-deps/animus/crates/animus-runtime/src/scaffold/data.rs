// Cairn adaptation compatibility modules intentionally keep public DTO names and table-driven
// validation compact while drained changes are archived.
#![allow(
    unknown_lints,
    ambiguous_params,
    assertion_density,
    bool_naming,
    borrowed_argument_types
)]
#![allow(compound_condition, excessive_file_length, function_length, non_trait_imports)]
#![allow(
    numeric_units,
    path_segment_repetition,
    raw_arithmetic_overflow,
    unbounded_collection_growth
)]
/// BLAKE3 or explicitly declared interoperability fingerprint.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScaffoldFingerprint {
    /// Name of the hash algorithm.
    pub algorithm: String,
    /// Lowercase hexadecimal digest value.
    pub value_hex: String,
}

impl ScaffoldFingerprint {
    /// Builds a scaffold fingerprint from precomputed parts.
    #[must_use]
    pub fn new(algorithm: impl Into<String>, value_hex: impl Into<String>) -> Self {
        Self {
            algorithm: algorithm.into(),
            value_hex: value_hex.into(),
        }
    }
}

/// Review state for a scaffold profile revision.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScaffoldReviewStatus {
    /// Revision is proposed and cannot influence turns yet.
    Proposed,
    /// Revision is reviewed and may influence turns when selected.
    Reviewed,
    /// Revision is retained for history but cannot influence new turns.
    Deprecated,
}

/// Narrow strategy knobs that a scaffold profile may declare.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ScaffoldStrategyKnob {
    /// Retry guidance such as one retry, no retry, or bounded retry.
    RetryPosture,
    /// Validation checklist guidance before final response.
    ValidationChecklist,
    /// Preferred safe ordering for already-admitted tools.
    ToolOrdering,
    /// Compaction or summary preference labels.
    CompactionPreference,
    /// Memory/context inclusion policy labels.
    MemoryInclusion,
    /// Task-category hint labels.
    TaskCategoryHint,
    /// Unknown or unsupported knob label supplied by a host fixture.
    Unsupported(String),
}

/// Provenance for a scaffold profile.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScaffoldProfileProvenance {
    /// Source label for the reviewed strategy.
    pub source: String,
    /// Optional reviewer or owner label.
    pub reviewed_by: Option<String>,
    /// Optional bounded rationale.
    pub rationale: Option<String>,
}

/// One immutable scaffold profile revision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScaffoldRevision {
    /// Stable revision identifier.
    pub revision_id: String,
    /// Strategy knobs enabled by this revision.
    pub strategy_knobs: Vec<ScaffoldStrategyKnob>,
    /// Review status for this revision.
    pub review_status: ScaffoldReviewStatus,
    /// BLAKE3 hash for the reviewed scaffold body or canonical descriptor.
    pub content_hash: ScaffoldFingerprint,
    /// Whether the supplied content hash matches the reviewed descriptor bytes.
    pub hash_matches_content: bool,
    /// Whether the revision attempts to include executable harness behavior.
    pub executable_harness: bool,
}

/// Reviewed scaffold profile descriptor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScaffoldProfile {
    /// Stable profile identifier.
    pub profile_id: String,
    /// Accepted task-class labels for selection.
    pub task_classes: Vec<String>,
    /// Active revision identifier.
    pub active_revision_id: String,
    /// All known revisions for the profile.
    pub revisions: Vec<ScaffoldRevision>,
    /// Route capability labels required by this profile.
    pub route_constraints: Vec<String>,
    /// Host policy switch for profile availability.
    pub enabled: bool,
    /// Review provenance for the profile.
    pub provenance: ScaffoldProfileProvenance,
}

/// Scaffold selection request supplied by a host or runtime shell.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScaffoldSelectionRequest {
    /// Requested task-class label.
    pub task_class: String,
    /// Capability labels available on the selected route.
    pub route_capability_labels: Vec<String>,
    /// Profile IDs disabled by active host policy.
    pub disabled_profile_ids: Vec<String>,
    /// Optional fallback task-class label.
    pub fallback_task_class: Option<String>,
}

/// Outcome of scaffold selection planning.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScaffoldSelectionOutcome {
    /// An exact task-class profile was selected.
    ExactMatch,
    /// A fallback task-class profile was selected.
    Fallback,
    /// No compatible reviewed profile was available.
    NoCompatibleProfile,
}

/// Pure scaffold selection plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScaffoldSelectionPlan {
    /// Selection outcome.
    pub outcome: ScaffoldSelectionOutcome,
    /// Selected profile identifier, when available.
    pub profile_id: Option<String>,
    /// Selected revision identifier, when available.
    pub revision_id: Option<String>,
    /// Deterministic diagnostics describing rejected candidates.
    pub diagnostics: Vec<ScaffoldDiagnostic>,
}

/// Proposed scaffold revision body metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScaffoldRevisionProposal {
    /// Stable proposal identifier.
    pub proposal_id: String,
    /// Target profile identifier.
    pub profile_id: String,
    /// Base revision identifier expected by the proposer.
    pub base_revision_id: String,
    /// Proposed revision identifier.
    pub proposed_revision_id: String,
    /// BLAKE3 hash of the proposed reviewed descriptor or body.
    pub proposed_content_hash: ScaffoldFingerprint,
    /// Strategy knobs requested by the proposal.
    pub proposed_strategy_knobs: Vec<ScaffoldStrategyKnob>,
}

/// Admission request for a scaffold revision proposal.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScaffoldAdmissionRequest {
    /// Revision proposal requiring host admission.
    pub proposal: ScaffoldRevisionProposal,
}

/// Host admission outcome for a proposed scaffold revision.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScaffoldAdmissionOutcome {
    /// Host admitted the proposal for persistence and future influence.
    Admitted,
    /// Host denied the proposal.
    Denied,
    /// Admission service was unavailable.
    Unavailable,
    /// Proposal was based on a stale active revision.
    StaleBase,
}

/// Resulting effect of applying an admission outcome.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScaffoldAdmissionEffect {
    /// The proposal may be persisted as a reviewed active revision.
    PersistAndActivate,
    /// The proposal is recorded but must not influence future turns.
    RecordOnly,
}

/// Deterministic scaffold admission decision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScaffoldAdmissionDecision {
    /// Host admission outcome.
    pub outcome: ScaffoldAdmissionOutcome,
    /// Whether the proposal may be persisted.
    pub persist: bool,
    /// Whether the proposal may become active.
    pub activate: bool,
    /// Whether future prompt or Steel inputs may use the proposal.
    pub future_influence: bool,
    /// Summary effect class.
    pub effect: ScaffoldAdmissionEffect,
    /// Diagnostics explaining denied, unavailable, or stale decisions.
    pub diagnostics: Vec<ScaffoldDiagnostic>,
}

/// Safe scaffold influence receipt metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScaffoldInfluenceReceipt {
    /// Profile identifier that influenced a turn or evaluation.
    pub profile_id: String,
    /// Revision identifier that influenced a turn or evaluation.
    pub revision_id: String,
    /// Task class used for selection.
    pub task_class: String,
    /// Review status of the revision.
    pub review_status: ScaffoldReviewStatus,
    /// Route capability labels used during selection.
    pub route_labels: Vec<String>,
    /// Validation outcome summary.
    pub validation_outcome: ScaffoldInfluenceValidationOutcome,
    /// Profile descriptor hash.
    pub profile_hash: ScaffoldFingerprint,
    /// Revision descriptor hash.
    pub revision_hash: ScaffoldFingerprint,
    /// Negative-test-only sensitive evidence labels that must be rejected.
    pub sensitive_evidence: Vec<ScaffoldReceiptSensitiveEvidence>,
}

/// Scaffold influence validation outcome.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScaffoldInfluenceValidationOutcome {
    /// Scaffold metadata was validated and used.
    Accepted,
    /// Scaffold metadata was rejected before influence.
    Rejected,
}

/// Sensitive receipt evidence classes that must not appear in accepted receipts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ScaffoldReceiptSensitiveEvidence {
    /// Raw prompt body.
    RawPrompt,
    /// Raw scaffold body or hidden strategy text.
    ScaffoldBody,
    /// Provider-native payload.
    ProviderPayload,
    /// Credential-looking value.
    Credential,
    /// Full absolute host path dump.
    FullPath,
    /// Raw or unbounded tool output.
    UnboundedToolOutput,
}

/// Scaffold validation diagnostic class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScaffoldDiagnosticKind {
    /// Required field is missing or blank.
    MissingField,
    /// Identifier is malformed.
    MalformedId,
    /// Required collection is empty or has a blank entry.
    EmptyCollection,
    /// A revision id appears more than once.
    DuplicateRevision,
    /// Strategy knob is unsupported.
    UnsupportedKnob,
    /// Provenance is missing.
    MissingProvenance,
    /// Active revision is absent, duplicated, stale, or unknown.
    ActiveRevisionInvalid,
    /// Active revision is not reviewed.
    UnreviewedActiveRevision,
    /// Content hash is missing, not BLAKE3, or mismatched.
    ContentHashInvalid,
    /// Profile is disabled by descriptor or host policy.
    DisabledProfile,
    /// Route labels do not satisfy profile constraints.
    RouteMismatch,
    /// No compatible scaffold profile exists.
    NoCompatibleProfile,
    /// Admission did not allow persistence or future influence.
    AdmissionRejected,
    /// Receipt contains unsafe sensitive evidence.
    ReceiptLeak,
    /// Scaffold attempts executable harness behavior.
    ExecutableHarness,
}

/// Scaffold validation diagnostic.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScaffoldDiagnostic {
    /// Diagnostic class.
    pub kind: ScaffoldDiagnosticKind,
    /// Stable field or relation label.
    pub field: String,
    /// Safe human-readable detail.
    pub detail: String,
}

impl ScaffoldDiagnostic {
    /// Builds a scaffold diagnostic.
    #[must_use]
    pub fn new(kind: ScaffoldDiagnosticKind, field: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            kind,
            field: field.into(),
            detail: detail.into(),
        }
    }
}

/// Validation result for scaffold descriptors and receipts.
pub type ScaffoldValidationResult = Result<(), Vec<ScaffoldDiagnostic>>;

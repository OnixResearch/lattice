//! Typed data for rejected-attempt context planning.

macro_rules! nominal_identity {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
        pub struct $name(String);

        impl $name {
            /// Creates an identity without performing admission.
            #[must_use]
            pub fn new(value: impl Into<String>) -> Self {
                Self(value.into())
            }

            /// Returns the identity text.
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
    };
}

nominal_identity!(TaskIdentity, "Nominal task identity.");
nominal_identity!(RunIdentity, "Nominal run identity.");
nominal_identity!(AttemptIdentity, "Nominal attempt identity.");
nominal_identity!(RevisionIdentity, "Nominal VCS revision identity.");
nominal_identity!(
    WorkspaceSnapshotIdentity,
    "Nominal workspace snapshot identity."
);
nominal_identity!(ContentIdentity, "Nominal attempt-content identity.");
nominal_identity!(ReviewHandoffIdentity, "Nominal review-handoff identity.");
nominal_identity!(SummaryIdentity, "Nominal bounded-summary identity.");
nominal_identity!(ReceiptIdentity, "Nominal metadata-only receipt identity.");

/// Closed VCS provider classes admitted by the reusable contract.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VcsProvider {
    /// Jujutsu host adapter.
    Jujutsu,
    /// Git host adapter.
    Git,
    /// A policy-declared VCS-neutral adapter.
    PolicyDeclared,
}

/// Review evidence profile.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewEvidenceProfile {
    /// Cairn revision-handoff receipt.
    CairnRevisionHandoff,
    /// Policy-declared equivalent review evidence.
    PolicyDeclaredEquivalent,
}

/// Review disposition supplied by the review owner.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReviewDisposition {
    /// The attempt was rejected.
    Rejected,
    /// The attempt was accepted.
    Accepted,
    /// The disposition is absent or ambiguous.
    Unknown,
}

/// Raw-diff projection disposition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RawDiffDisposition {
    /// Policy denied raw diff projection.
    DeniedByPolicy,
    /// Classification denied raw diff projection.
    DeniedByClassification,
    /// The byte budget denied raw diff projection.
    DeniedByBudget,
    /// Explicit policy, capability, classification, and budget admitted the diff.
    Admitted,
    /// No raw diff was supplied.
    NotSupplied,
}

/// Metadata-only review handoff facts loaded by a host.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewHandoffFacts {
    /// Handoff identity.
    pub handoff_id: ReviewHandoffIdentity,
    /// Review profile.
    pub profile: ReviewEvidenceProfile,
    /// Task bound by the review.
    pub task_id: TaskIdentity,
    /// Run bound by the review.
    pub run_id: RunIdentity,
    /// Attempt bound by the review.
    pub attempt_id: AttemptIdentity,
    /// Revision bound by the review.
    pub revision_id: RevisionIdentity,
    /// Review disposition.
    pub disposition: ReviewDisposition,
    /// Number of matching attempt records in the supplied evidence.
    pub matching_attempts: u32,
    /// Whether the review receipt passed its owner validation.
    pub validated: bool,
}

/// Current VCS facts observed by a host.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VcsFacts {
    /// Provider class.
    pub provider: VcsProvider,
    /// Attempt identity.
    pub attempt_id: AttemptIdentity,
    /// Current revision identity.
    pub revision_id: RevisionIdentity,
    /// Current workspace snapshot identity.
    pub workspace_snapshot_id: WorkspaceSnapshotIdentity,
    /// Current content identity.
    pub content_id: ContentIdentity,
    /// Whether the supplied observation is current.
    pub current: bool,
}

/// Redaction and classification facts for optional model-visible context.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RejectionContextFacts {
    /// Safe bounded summary identity.
    pub summary_id: SummaryIdentity,
    /// Redacted summary text.
    pub redacted_summary: String,
    /// Whether redaction passed the host classification boundary.
    pub summary_classification_safe: bool,
    /// Optional raw diff bytes supplied by the host.
    pub raw_diff: Option<Vec<u8>>,
    /// Whether raw-diff classification passed.
    pub raw_diff_classification_safe: bool,
    /// Whether the caller supplied an explicit raw-diff capability.
    pub raw_diff_capability_admitted: bool,
}

/// Versioned planning policy projected from Nickel.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlanningContract {
    /// Policy identity.
    pub policy_id: String,
    /// Admitted review profiles.
    pub accepted_review_profiles: Vec<ReviewEvidenceProfile>,
    /// Admitted VCS providers.
    pub accepted_vcs_providers: Vec<VcsProvider>,
    /// Maximum redacted summary bytes.
    pub max_summary_bytes: u32,
    /// Maximum raw diff bytes.
    pub max_raw_diff_bytes: u32,
    /// Whether raw diff projection can be admitted.
    pub allow_raw_diff: bool,
    /// Required receipt non-claims.
    pub required_non_claims: Vec<String>,
}

/// Complete already-loaded input to the pure planner.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Request {
    /// Expected task identity.
    pub task_id: TaskIdentity,
    /// Expected run identity.
    pub run_id: RunIdentity,
    /// Expected attempt identity.
    pub attempt_id: AttemptIdentity,
    /// Expected revision identity.
    pub revision_id: RevisionIdentity,
    /// Expected workspace identity.
    pub workspace_snapshot_id: WorkspaceSnapshotIdentity,
    /// Expected content identity.
    pub content_id: ContentIdentity,
    /// Review facts.
    pub review: ReviewHandoffFacts,
    /// Current VCS facts.
    pub vcs: VcsFacts,
    /// Model-visible context candidates.
    pub context: RejectionContextFacts,
}

/// Safe context plan for a later fresh iteration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextPlan {
    /// Bound task identity.
    pub task_id: TaskIdentity,
    /// Bound run identity.
    pub run_id: RunIdentity,
    /// Bound attempt identity.
    pub attempt_id: AttemptIdentity,
    /// Bound review handoff identity.
    pub handoff_id: ReviewHandoffIdentity,
    /// Bound revision identity.
    pub revision_id: RevisionIdentity,
    /// Bound summary identity.
    pub summary_id: SummaryIdentity,
    /// Redacted summary selected for projection.
    pub redacted_summary: String,
    /// Raw-diff disposition.
    pub raw_diff_disposition: RawDiffDisposition,
    /// Fresh-context composition label.
    pub context_profile: String,
    /// Ralph composition label.
    pub loop_profile: String,
    /// Explicit omissions.
    pub omissions: Vec<String>,
}

/// Metadata-only rejected-attempt receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvidenceReceipt {
    /// Receipt identity.
    pub receipt_id: ReceiptIdentity,
    /// Task identity.
    pub task_id: TaskIdentity,
    /// Run identity.
    pub run_id: RunIdentity,
    /// Attempt identity.
    pub attempt_id: AttemptIdentity,
    /// Revision identity.
    pub revision_id: RevisionIdentity,
    /// Handoff identity.
    pub handoff_id: ReviewHandoffIdentity,
    /// Summary identity.
    pub summary_id: SummaryIdentity,
    /// Raw-diff disposition.
    pub raw_diff_disposition: RawDiffDisposition,
    /// Bounded claims.
    pub claims: Vec<String>,
    /// Required non-claims.
    pub non_claims: Vec<String>,
}

/// Fail-closed planning issue.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Issue {
    /// A required nominal identity is blank.
    MissingIdentity,
    /// Review evidence is absent, invalid, or not admitted.
    ReviewEvidenceDenied,
    /// Review evidence does not bind the expected task or run.
    WrongTaskOrRun,
    /// More or fewer than one attempt matched.
    AmbiguousAttempt,
    /// The VCS provider is not admitted.
    VcsProviderDenied,
    /// Current VCS facts differ from the bound attempt.
    StaleAttempt,
    /// Summary content failed classification.
    UnsafeSummary,
    /// Summary exceeded the policy byte budget.
    OversizedSummary,
    /// Receipt claims exceeded the advisory boundary.
    ReceiptOverclaim,
    /// Required receipt non-claims were weakened.
    MissingNonClaim,
}

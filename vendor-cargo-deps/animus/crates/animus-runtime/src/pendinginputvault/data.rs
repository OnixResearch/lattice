//! Typed data for explicit pending-entry recovery.

// The public module and type-level comments define this compact DTO vocabulary.
#![allow(missing_docs)]

/// Maximum safe metadata bytes for one identity or label.
pub const MAX_METADATA_BYTES: u32 = 256;
/// Maximum entries accepted by the reusable core.
pub const MAX_ENTRY_COUNT: u32 = 256;
/// Maximum attachment references accepted for one entry.
pub const MAX_ATTACHMENT_COUNT: u32 = 32;
/// BLAKE3 hexadecimal identity length.
pub const BLAKE3_HEX_LENGTH: u32 = 64;

macro_rules! nominal_identity {
    ($name:ident) => {
        #[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
        pub struct $name(String);

        impl $name {
            /// Creates an identity for later validation at the pure boundary.
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

nominal_identity!(Identity);
nominal_identity!(ConversationIdentity);
nominal_identity!(BranchIdentity);
nominal_identity!(StoreIdentity);
nominal_identity!(AuthorityIdentity);
nominal_identity!(PolicyIdentity);
nominal_identity!(EntryIdentity);
nominal_identity!(DispositionIdentity);
nominal_identity!(BodyIdentity);
nominal_identity!(AttachmentIdentity);
nominal_identity!(ProvenanceIdentity);
nominal_identity!(PlanIdentity);
nominal_identity!(ReceiptIdentity);

/// Monotonic store generation.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Generation(u64);

impl Generation {
    /// Creates a generation.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the generation value.
    #[must_use]
    pub const fn value(self) -> u64 {
        self.0
    }
}

/// Monotonic entry sequence.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Sequence(u64);

impl Sequence {
    /// Creates a sequence.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the sequence value.
    #[must_use]
    pub const fn value(self) -> u64 {
        self.0
    }
}

/// Bounded byte count.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ByteCount(u32);

impl ByteCount {
    /// Creates a byte count.
    #[must_use]
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    /// Returns the byte count.
    #[must_use]
    pub const fn value(self) -> u32 {
        self.0
    }
}

/// Original route and semantic class of one durable entry.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum EntryClass {
    ActiveAccepted,
    SteeringInFlight,
    SteeringQueued,
    FollowUp,
    Command,
    AttachmentReference,
}

impl EntryClass {
    /// Returns a stable safe label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ActiveAccepted => "active-accepted",
            Self::SteeringInFlight => "steering-in-flight",
            Self::SteeringQueued => "steering-queued",
            Self::FollowUp => "follow-up",
            Self::Command => "command",
            Self::AttachmentReference => "attachment-reference",
        }
    }
}

/// Caller-supplied entry facts before validation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EntryDraft {
    pub id: EntryIdentity,
    pub conversation: ConversationIdentity,
    pub generation: Generation,
    pub sequence: Sequence,
    pub class: EntryClass,
    pub body_identity: BodyIdentity,
    pub body_bytes: ByteCount,
    pub attachments: Vec<AttachmentIdentity>,
    pub provenance: ProvenanceIdentity,
}

/// Validated entry that retains its original class and order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Entry {
    pub id: EntryIdentity,
    pub conversation: ConversationIdentity,
    pub generation: Generation,
    pub sequence: Sequence,
    pub class: EntryClass,
    pub body_identity: BodyIdentity,
    pub body_bytes: ByteCount,
    pub attachments: Vec<AttachmentIdentity>,
    pub provenance: ProvenanceIdentity,
}

/// Explicit host and authority boundary for one recovery operation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Boundary {
    pub id: Identity,
    pub conversation: ConversationIdentity,
    pub branch: BranchIdentity,
    pub store: StoreIdentity,
    pub authority: AuthorityIdentity,
    pub policy: PolicyIdentity,
    pub generation: Generation,
}

/// Finite admission limits.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Policy {
    pub entries: u32,
    pub entry_bytes: ByteCount,
    pub total_bytes: u64,
    pub attachments_per_entry: u32,
}

/// Supplied durable facts. The core performs no discovery or storage access.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StartupFacts {
    pub boundary: Boundary,
    pub entries: Vec<Entry>,
    pub completed: Option<CompletedDisposition>,
    pub cleanup: CleanupState,
    pub uncertain_commit: bool,
}

/// Corruption class that blocks ordinary conversation ingress.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CorruptionClass {
    Malformed,
    MissingSequence,
    Reordered,
    Duplicate,
    MixedGeneration,
    OverBudget,
    CrossConversation,
    UncertainCommit,
}

impl CorruptionClass {
    /// Returns a stable safe label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Malformed => "malformed",
            Self::MissingSequence => "missing-sequence",
            Self::Reordered => "reordered",
            Self::Duplicate => "duplicate",
            Self::MixedGeneration => "mixed-generation",
            Self::OverBudget => "over-budget",
            Self::CrossConversation => "cross-conversation",
            Self::UncertainCommit => "uncertain-commit",
        }
    }
}

/// Cleanup status after durable disposition evidence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CleanupState {
    NotRequired,
    Required,
    Complete,
    Failed,
}

impl CleanupState {
    /// Returns a stable safe label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NotRequired => "not-required",
            Self::Required => "required",
            Self::Complete => "complete",
            Self::Failed => "failed",
        }
    }
}

/// Startup state for one supplied conversation boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RecoveryState {
    NoVault,
    ReviewRequired,
    DispositionPending,
    DispositionComplete,
    CleanupRequired,
    BlockedCorrupt(CorruptionClass),
}

impl RecoveryState {
    /// Returns a stable safe label.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::NoVault => "no-vault",
            Self::ReviewRequired => "review-required",
            Self::DispositionPending => "disposition-pending",
            Self::DispositionComplete => "disposition-complete",
            Self::CleanupRequired => "cleanup-required",
            Self::BlockedCorrupt(_) => "blocked-corrupt",
        }
    }
}

/// Validated startup projection with no dispatch authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Projection {
    pub boundary: Boundary,
    pub entries: Vec<Entry>,
    pub state: RecoveryState,
    pub identity: PlanIdentity,
}

/// Metadata-only entry inspection row.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InspectionEntry {
    pub id: EntryIdentity,
    pub sequence: Sequence,
    pub class: EntryClass,
    pub body_identity: BodyIdentity,
    pub body_bytes: ByteCount,
    pub attachment_count: u32,
    pub provenance: ProvenanceIdentity,
}

/// Bounded metadata-only inspection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Inspection {
    pub identity: PlanIdentity,
    pub state: RecoveryState,
    pub entries: Vec<InspectionEntry>,
}

/// Explicit operator disposition kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DispositionKind {
    Restore,
    Discard,
}

impl DispositionKind {
    /// Returns a stable safe label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Restore => "restore",
            Self::Discard => "discard",
        }
    }
}

/// Explicit disposition request over one exact projection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DispositionRequest {
    pub id: DispositionIdentity,
    pub projection: PlanIdentity,
    pub boundary: Boundary,
    pub expected_generation: Generation,
    pub selected: Vec<EntryIdentity>,
    pub kind: DispositionKind,
    pub authorized: bool,
}

/// Pure mutation plan returned before any host effect.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DispositionPlan {
    pub id: DispositionIdentity,
    pub plan_identity: PlanIdentity,
    pub projection: PlanIdentity,
    pub boundary: Boundary,
    pub expected_generation: Generation,
    pub selected: Vec<EntryIdentity>,
    pub kind: DispositionKind,
}

/// Reference that must pass normal current admission after restore.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdmissionRequest {
    pub entry: EntryIdentity,
    pub conversation: ConversationIdentity,
    pub generation: Generation,
    pub class: EntryClass,
    pub body_identity: BodyIdentity,
}

/// Durable disposition evidence retained across restart.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompletedDisposition {
    pub id: DispositionIdentity,
    pub plan_identity: PlanIdentity,
    pub projection: PlanIdentity,
    pub boundary: Boundary,
    pub selected: Vec<EntryIdentity>,
    pub kind: DispositionKind,
    pub admission_requests: Vec<AdmissionRequest>,
    pub restore_material_retained: bool,
    pub cleanup: CleanupState,
}

/// Idempotent planning result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DispositionDecision {
    Planned(DispositionPlan),
    Repeated(CompletedDisposition),
}

/// Host commit status for one exact disposition plan.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommitStatus {
    Committed,
    Failed,
    Uncertain,
}

/// Host observation returned after disposition evidence commit.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommitObservation {
    pub plan_identity: PlanIdentity,
    pub generation: Generation,
    pub status: CommitStatus,
    pub restore_material_retained: bool,
}

/// Result after reducing one commit observation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CommitOutcome {
    Completed(Box<CompletedDisposition>),
    ReviewRequired,
    BlockedCorrupt(CorruptionClass),
}

/// Separately authorized recovery control for corrupt state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecoveryControlKind {
    Inspect,
    QuarantineExport,
    RepairImport,
    DestructiveDiscard,
}

/// Request for a recovery-only control.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecoveryControlRequest {
    pub boundary: Boundary,
    pub projection: PlanIdentity,
    pub kind: RecoveryControlKind,
    pub authorized: bool,
}

/// Recovery-only plan with no provider or transcript authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecoveryControlPlan {
    pub boundary: Boundary,
    pub projection: PlanIdentity,
    pub kind: RecoveryControlKind,
    pub plan_identity: PlanIdentity,
}

/// Sensitive fields that evidence projection must reject.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SensitiveDraft<'a> {
    pub body: Option<&'a str>,
    pub attachment: Option<&'a str>,
    pub prompt: Option<&'a str>,
    pub tool_data: Option<&'a str>,
    pub provider_data: Option<&'a str>,
    pub credential: Option<&'a str>,
    pub environment_value: Option<&'a str>,
    pub physical_path: Option<&'a str>,
    pub raw_store_error: Option<&'a str>,
}

/// Metadata-only evidence receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Receipt {
    pub id: ReceiptIdentity,
    pub projection: PlanIdentity,
    pub conversation: ConversationIdentity,
    pub branch: BranchIdentity,
    pub store: StoreIdentity,
    pub authority: AuthorityIdentity,
    pub policy: PolicyIdentity,
    pub generation: Generation,
    pub entry_count: u32,
    pub class_counts: Vec<(EntryClass, u32)>,
    pub state: RecoveryState,
    pub disposition: Option<DispositionKind>,
    pub cleanup: CleanupState,
    pub claims: Vec<String>,
    pub non_claims: Vec<String>,
}

/// Stable fail-closed issue classes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Issue {
    MissingBoundary,
    UnsafeMetadata,
    InvalidBodyIdentity,
    EntryLimit,
    EntryByteLimit,
    AttachmentLimit,
    DuplicateEntry,
    MissingSequence,
    ReorderedSequence,
    MixedGeneration,
    CrossConversation,
    TotalByteLimit,
    Stale,
    Conflict,
    Unauthorized,
    InvalidState,
    ReceiptLeak,
    InvalidReceipt,
    ReceiptOverclaim,
    MissingNonClaim,
    ServiceUnavailable,
    ServiceFailure,
}

/// Error that never echoes private input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Error {
    issue: Issue,
}

impl Error {
    /// Creates a bounded error.
    #[must_use]
    pub const fn new(issue: Issue) -> Self {
        Self { issue }
    }

    /// Returns the stable issue class.
    #[must_use]
    pub const fn issue(self) -> Issue {
        self.issue
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            formatter,
            "pending entry recovery rejected: {:?}",
            self.issue
        )
    }
}

impl std::error::Error for Error {}

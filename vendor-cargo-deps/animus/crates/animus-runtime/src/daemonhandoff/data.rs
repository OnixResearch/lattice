//! Typed data for transactional daemon handoff.

type String = std::string::String;

macro_rules! text_identity {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
        pub struct $name(String);

        impl $name {
            /// Creates an identity for later boundary validation.
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

macro_rules! generation {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
        pub struct $name(u64);

        impl $name {
            /// Creates a generation for later boundary validation.
            #[must_use]
            pub const fn new(value: u64) -> Self {
                Self(value)
            }

            /// Returns the numeric generation.
            #[must_use]
            pub const fn value(self) -> u64 {
                self.0
            }
        }
    };
}

text_identity!(WorkerIdentity, "One daemon worker identity.");
text_identity!(SessionIdentity, "One durable session identity.");
text_identity!(BranchIdentity, "One active session branch identity.");
text_identity!(
    AuthorityProfileIdentity,
    "One current authority-profile identity."
);
text_identity!(SessionHeadIdentity, "One committed session-head identity.");
text_identity!(
    ComposerDraftIdentity,
    "One host-retained composer-draft identity."
);
text_identity!(PolicyIdentity, "One handoff policy identity.");
text_identity!(HandoffIdentity, "One canonical handoff identity.");
text_identity!(TransitionIdentity, "One canonical transition identity.");
text_identity!(SnapshotIdentity, "One strict-idle snapshot identity.");
text_identity!(
    ProcessAttemptIdentity,
    "One replacement process-attempt identity."
);
text_identity!(ProcessIdentity, "One replacement process identity.");
text_identity!(
    EndpointIdentity,
    "One authenticated local endpoint identity."
);
text_identity!(ClaimantIdentity, "One replacement claimant identity.");
text_identity!(FreshnessNonce, "One handoff-local freshness nonce.");
text_identity!(ClaimIdentity, "One canonical replacement-claim identity.");
text_identity!(
    ReadinessIdentity,
    "One canonical replacement-readiness identity."
);
text_identity!(CancellationIdentity, "One canonical cancellation identity.");
text_identity!(
    ServiceReadinessIdentity,
    "One replacement service-readiness identity."
);
text_identity!(
    CommitIdentity,
    "One durable handoff-record commit identity."
);
text_identity!(
    TerminationIdentity,
    "One complete process-tree termination identity."
);
text_identity!(ObservationIdentity, "One host observation identity.");

generation!(LeaseGeneration, "One monotonic session-lease generation.");
generation!(QueueGeneration, "One monotonic queue generation.");
generation!(
    ActiveTurnGeneration,
    "One monotonic active-turn generation."
);
generation!(
    VaultGeneration,
    "One monotonic pending-input vault generation."
);
generation!(
    TransitionSequence,
    "One monotonic handoff transition sequence."
);

/// Activity state supplied for one strict-idle work class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActivityState {
    /// The work class is absent or terminal.
    Idle,
    /// The work class contains active work.
    Active,
    /// The host cannot establish whether the work class is idle.
    Ambiguous,
}

/// Work classes that can deny strict idle.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkClass {
    /// Active provider attempt.
    Provider,
    /// Active tool effect.
    Tool,
    /// Unresolved repair.
    Repair,
    /// Unresolved confirmation.
    Confirmation,
    /// Active or ambiguous cancellation.
    Cancellation,
    /// Non-empty primary queue.
    PrimaryQueue,
    /// Non-empty child queue.
    ChildQueue,
    /// Undelivered steering entry.
    Steering,
    /// Pending durable commit.
    DurableCommit,
    /// Unresolved terminal transition.
    TerminalTransition,
}

/// Exact worker facts used to establish strict idle.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StrictIdleSnapshot {
    /// Current worker.
    pub worker: WorkerIdentity,
    /// Current durable session.
    pub session: SessionIdentity,
    /// Current active branch.
    pub branch: BranchIdentity,
    /// Current lease generation.
    pub lease_generation: LeaseGeneration,
    /// Current primary queue generation.
    pub primary_queue_generation: QueueGeneration,
    /// Current child queue generation.
    pub child_queue_generation: QueueGeneration,
    /// Current steering queue generation.
    pub steering_generation: QueueGeneration,
    /// Current active-turn generation.
    pub active_turn_generation: ActiveTurnGeneration,
    /// Current authority profile.
    pub authority_profile: AuthorityProfileIdentity,
    /// Provider activity.
    pub provider: ActivityState,
    /// Tool activity.
    pub tool: ActivityState,
    /// Repair activity.
    pub repair: ActivityState,
    /// Confirmation activity.
    pub confirmation: ActivityState,
    /// Cancellation activity.
    pub cancellation: ActivityState,
    /// Durable commit activity.
    pub durable_commit: ActivityState,
    /// Terminal transition activity.
    pub terminal_transition: ActivityState,
    /// Primary queue depth.
    pub primary_queue_depth: u32,
    /// Child queue depth.
    pub child_queue_depth: u32,
    /// Undelivered steering depth.
    pub steering_depth: u32,
}

/// Admitted strict-idle decision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StrictIdleDecision {
    /// Exact admitted snapshot.
    pub snapshot: StrictIdleSnapshot,
    /// Canonical snapshot identity.
    pub snapshot_identity: SnapshotIdentity,
}

/// Immutable handoff binding copied from strict idle.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HandoffBinding {
    /// Old worker.
    pub old_worker: WorkerIdentity,
    /// Durable session.
    pub session: SessionIdentity,
    /// Active branch.
    pub branch: BranchIdentity,
    /// Released lease generation.
    pub lease_generation: LeaseGeneration,
    /// Primary queue generation.
    pub primary_queue_generation: QueueGeneration,
    /// Child queue generation.
    pub child_queue_generation: QueueGeneration,
    /// Steering generation.
    pub steering_generation: QueueGeneration,
    /// Active-turn generation.
    pub active_turn_generation: ActiveTurnGeneration,
    /// Authority profile.
    pub authority_profile: AuthorityProfileIdentity,
    /// Source strict-idle identity.
    pub idle_snapshot: SnapshotIdentity,
}

/// Durable session input state committed before lease release.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DurableHandoffState {
    /// Exact session head.
    pub session_head: SessionHeadIdentity,
    /// Host-retained composer draft identity.
    pub composer_draft: ComposerDraftIdentity,
    /// Pending-input vault generation.
    pub vault_generation: VaultGeneration,
    /// Active-turn generation.
    pub active_turn_generation: ActiveTurnGeneration,
    /// Authority profile.
    pub authority_profile: AuthorityProfileIdentity,
    /// Handoff policy.
    pub policy: PolicyIdentity,
    /// Durable handoff-record commit.
    pub commit: CommitIdentity,
}

/// Current handoff state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HandoffState {
    /// Strict idle admitted and the transaction identity exists.
    Prepared,
    /// New ingress is paused.
    IngressPaused,
    /// Durable state committed and the old lease released.
    LeaseReleased,
    /// One replacement process tree started.
    ReplacementStarted,
    /// One authenticated claimant was admitted.
    Claimed,
    /// Exact replacement readiness committed.
    Ready,
    /// Cancellation committed.
    Cancelled,
    /// Old worker acquired a later lease after complete teardown.
    LeaseRestored,
    /// Old worker exited after exact replacement readiness.
    Complete,
}

impl HandoffState {
    /// Returns the stable state label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Prepared => "prepared",
            Self::IngressPaused => "ingress_paused",
            Self::LeaseReleased => "lease_released",
            Self::ReplacementStarted => "replacement_started",
            Self::Claimed => "claimed",
            Self::Ready => "ready",
            Self::Cancelled => "cancelled",
            Self::LeaseRestored => "lease_restored",
            Self::Complete => "complete",
        }
    }
}

/// Ingress-pause observation from the old worker host.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IngressPauseObservation {
    /// Handoff being paused.
    pub handoff: HandoffIdentity,
    /// Observed primary queue generation.
    pub primary_queue_generation: QueueGeneration,
    /// Observed child queue generation.
    pub child_queue_generation: QueueGeneration,
    /// Observed steering generation.
    pub steering_generation: QueueGeneration,
    /// Whether prompt, steer, follow-up, and external ingress are all paused.
    pub all_ingress_paused: bool,
}

/// Lease-release observation after durable state commit.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LeaseReleaseObservation {
    /// Handoff being released.
    pub handoff: HandoffIdentity,
    /// Released lease generation.
    pub released_generation: LeaseGeneration,
    /// Durable commit observed by the lease owner.
    pub durable_commit: CommitIdentity,
    /// Whether the old lease was released.
    pub released: bool,
}

/// Replacement process-start observation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProcessStartObservation {
    /// Handoff that owns the process tree.
    pub handoff: HandoffIdentity,
    /// Process attempt.
    pub process_attempt: ProcessAttemptIdentity,
    /// Replacement leader process.
    pub process: ProcessIdentity,
    /// Local endpoint selected for the attempt.
    pub endpoint: EndpointIdentity,
    /// Whether process-tree ownership was established.
    pub started: bool,
}

/// Replacement claim supplied through the authenticated local endpoint.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplacementClaim {
    /// Handoff identity.
    pub handoff: HandoffIdentity,
    /// Session identity.
    pub session: SessionIdentity,
    /// Released lease generation.
    pub released_generation: LeaseGeneration,
    /// Replacement process attempt.
    pub process_attempt: ProcessAttemptIdentity,
    /// Replacement process identity.
    pub process: ProcessIdentity,
    /// Endpoint identity.
    pub endpoint: EndpointIdentity,
    /// Claimant identity.
    pub claimant: ClaimantIdentity,
    /// Protocol version.
    pub protocol_version: u16,
    /// Handoff-local freshness nonce.
    pub freshness_nonce: FreshnessNonce,
}

/// Current host authentication observation for a replacement claim.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthenticationObservation {
    /// Endpoint that was authenticated.
    pub endpoint: EndpointIdentity,
    /// Process bound by authentication.
    pub process: ProcessIdentity,
    /// Whether authentication passed.
    pub authenticated: bool,
    /// Whether the observation is current for this claim.
    pub current: bool,
    /// Safe host observation identity.
    pub observation: ObservationIdentity,
}

/// Result of one exact replacement-readiness gate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReadinessCheck {
    /// The gate passed.
    Passed,
    /// The gate failed.
    Failed,
}

/// Replacement readiness observation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReadinessObservation {
    /// Handoff identity.
    pub handoff: HandoffIdentity,
    /// Claim identity.
    pub claim: ClaimIdentity,
    /// Session head loaded by the replacement.
    pub session_head: SessionHeadIdentity,
    /// Composer draft visible to the replacement.
    pub composer_draft: ComposerDraftIdentity,
    /// Pending-input vault generation visible to the replacement.
    pub vault_generation: VaultGeneration,
    /// Active-turn generation admitted by the replacement.
    pub active_turn_generation: ActiveTurnGeneration,
    /// Authority profile admitted by the replacement.
    pub authority_profile: AuthorityProfileIdentity,
    /// Policy admitted by the replacement.
    pub policy: PolicyIdentity,
    /// Replacement service-readiness identity.
    pub services: ServiceReadinessIdentity,
    /// Exact durable session-load result.
    pub session_load: ReadinessCheck,
    /// Pending-input vault-visibility result.
    pub vault_visibility: ReadinessCheck,
    /// Authority-admission result.
    pub authority_admission: ReadinessCheck,
    /// Active-turn recovery-admission result.
    pub recovery_admission: ReadinessCheck,
    /// Required service-readiness result.
    pub service_readiness: ReadinessCheck,
    /// Matching readiness-record commit result.
    pub readiness_commit: ReadinessCheck,
}

/// Old-worker exit observation after readiness.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompletionObservation {
    /// Old worker that exited.
    pub old_worker: WorkerIdentity,
    /// Readiness identity that authorized exit.
    pub readiness: ReadinessIdentity,
    /// Whether the old worker exited.
    pub exited: bool,
    /// Safe exit observation identity.
    pub observation: ObservationIdentity,
}

/// Cancellation cause.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CancellationCause {
    /// Readiness deadline elapsed.
    Deadline,
    /// Authorized operator requested cancellation.
    Operator,
    /// Replacement process could not start.
    SpawnFailure,
    /// Replacement claim failed authentication or binding.
    ClaimFailure,
    /// Replacement readiness failed or drifted.
    ReadinessFailure,
}

impl CancellationCause {
    /// Returns the stable cause label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Deadline => "deadline",
            Self::Operator => "operator",
            Self::SpawnFailure => "spawn_failure",
            Self::ClaimFailure => "claim_failure",
            Self::ReadinessFailure => "readiness_failure",
        }
    }
}

/// Pure cancellation plan committed before process termination.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CancellationPlan {
    /// Handoff identity.
    pub handoff: HandoffIdentity,
    /// Transition being cancelled.
    pub prior_transition: TransitionIdentity,
    /// Owned replacement attempt, when one started.
    pub process_attempt: Option<ProcessAttemptIdentity>,
    /// Cancellation cause.
    pub cause: CancellationCause,
    /// Canonical cancellation identity.
    pub cancellation: CancellationIdentity,
}

/// Durable cancellation commit observation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CancellationCommitObservation {
    /// Cancellation identity committed by the host.
    pub cancellation: CancellationIdentity,
    /// Whether cancelled state is durable.
    pub committed: bool,
    /// Safe commit observation identity.
    pub observation: ObservationIdentity,
}

/// Complete owned process-tree termination observation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProcessTreeTerminationObservation {
    /// Owned attempt, or none when spawn never established ownership.
    pub process_attempt: Option<ProcessAttemptIdentity>,
    /// Whether the leader is terminal.
    pub leader_terminal: bool,
    /// Whether every owned descendant is terminal.
    pub descendants_terminal: bool,
    /// Number of owned processes.
    pub owned_processes: u32,
    /// Number of terminal owned processes.
    pub terminal_processes: u32,
    /// Number of live owned descendants.
    pub live_descendants: u32,
    /// Canonical terminal observation identity.
    pub termination: TerminationIdentity,
}

/// Old-worker authority and store observation for rollback.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LeaseRestoreObservation {
    /// Handoff being restored.
    pub handoff: HandoffIdentity,
    /// Old worker reclaiming the session.
    pub old_worker: WorkerIdentity,
    /// Current authority profile.
    pub authority_profile: AuthorityProfileIdentity,
    /// Current session head.
    pub session_head: SessionHeadIdentity,
    /// New lease generation.
    pub restored_generation: LeaseGeneration,
    /// Whether current authority was revalidated.
    pub authority_current: bool,
    /// Whether the durable session head is unchanged.
    pub store_head_current: bool,
    /// Whether the next lease generation was acquired.
    pub acquired: bool,
    /// Safe lease observation identity.
    pub observation: ObservationIdentity,
}

/// Complete monotonic handoff transaction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HandoffTransaction {
    /// Immutable handoff binding.
    pub binding: HandoffBinding,
    /// Canonical handoff identity.
    pub handoff: HandoffIdentity,
    /// Current state.
    pub state: HandoffState,
    /// Current transition sequence.
    pub sequence: TransitionSequence,
    /// Prior transition identity.
    pub prior_transition: Option<TransitionIdentity>,
    /// Current transition identity.
    pub transition: TransitionIdentity,
    /// Canonical identity of the host observation reduced into this transition.
    pub observation: ObservationIdentity,
    /// Durable state, after lease release.
    pub durable_state: Option<DurableHandoffState>,
    /// Released generation, after lease release.
    pub released_generation: Option<LeaseGeneration>,
    /// Replacement attempt, after process start.
    pub process_attempt: Option<ProcessAttemptIdentity>,
    /// Replacement process, after process start.
    pub process: Option<ProcessIdentity>,
    /// Replacement endpoint, after process start.
    pub endpoint: Option<EndpointIdentity>,
    /// Admitted claimant, after claim.
    pub claimant: Option<ClaimantIdentity>,
    /// Canonical claim identity, after claim.
    pub claim: Option<ClaimIdentity>,
    /// Canonical readiness identity, after readiness.
    pub readiness: Option<ReadinessIdentity>,
    /// Canonical cancellation identity, after cancellation.
    pub cancellation: Option<CancellationIdentity>,
    /// Restored lease generation, after rollback.
    pub restored_generation: Option<LeaseGeneration>,
}

/// Receipt-sensitive input class. Values never enter a receipt.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SensitiveEvidence {
    /// No sensitive evidence was supplied.
    #[default]
    None,
    /// Composer text was supplied.
    ComposerText,
    /// Pending-input body was supplied.
    PendingInput,
    /// Prompt or transcript text was supplied.
    PromptOrTranscript,
    /// Credential material was supplied.
    Credentials,
    /// Socket path was supplied.
    SocketPath,
    /// Process arguments were supplied.
    ProcessArguments,
    /// Provider data was supplied.
    ProviderData,
    /// Terminal or PTY data was supplied.
    TerminalData,
    /// Raw unrestricted error was supplied.
    RawError,
}

/// Metadata-only handoff receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HandoffReceipt {
    /// Handoff identity.
    pub handoff: HandoffIdentity,
    /// Worker identity.
    pub worker: WorkerIdentity,
    /// Session identity.
    pub session: SessionIdentity,
    /// Lease generation.
    pub lease_generation: LeaseGeneration,
    /// Authority profile.
    pub authority_profile: AuthorityProfileIdentity,
    /// Current transition identity.
    pub transition: TransitionIdentity,
    /// Current state.
    pub state: HandoffState,
    /// Optional endpoint identity.
    pub endpoint: Option<EndpointIdentity>,
    /// Optional process attempt identity.
    pub process_attempt: Option<ProcessAttemptIdentity>,
    /// Optional readiness identity.
    pub readiness: Option<ReadinessIdentity>,
    /// Optional cancellation identity.
    pub cancellation: Option<CancellationIdentity>,
    /// Optional vault generation.
    pub vault_generation: Option<VaultGeneration>,
    /// Required redaction marker.
    pub redaction: String,
    /// Mandatory non-claims.
    pub non_claims: Vec<String>,
    /// Canonical receipt identity.
    pub receipt_identity: String,
}

/// Stable issue classes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Issue {
    /// A supplied identity is empty, unsafe, or overbound.
    InvalidIdentity,
    /// A required generation is zero or overflows.
    InvalidGeneration,
    /// A specific strict-idle work class remains active or ambiguous.
    Busy(WorkClass),
    /// A transition is out of order or terminal.
    InvalidTransition,
    /// A supplied fact is stale for the current transition.
    Stale,
    /// A supplied fact conflicts with committed evidence.
    Conflicting,
    /// Lease generation or ownership does not match.
    LeaseMismatch,
    /// Endpoint authentication failed or is stale.
    Unauthenticated,
    /// A second claimant attempted to enter one handoff.
    DuplicateClaimant,
    /// Cancelled handoff cannot accept a claim or readiness.
    Cancelled,
    /// Durable session state differs from the committed handoff record.
    SessionStateDrift,
    /// Complete owned process-tree termination is not established.
    TeardownIncomplete,
    /// Old-worker authority or durable store head changed before rollback.
    RollbackDrift,
    /// Sensitive data was supplied for receipt projection.
    ReceiptLeak,
    /// Receipt metadata makes an unsupported claim or is tampered.
    ReceiptOverclaim,
    /// An injected host service is unavailable.
    ServiceUnavailable,
    /// An injected host service failed.
    ServiceFailure,
}

/// Bounded handoff error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Error {
    issue: Issue,
}

impl Error {
    /// Creates an error from one stable issue class.
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

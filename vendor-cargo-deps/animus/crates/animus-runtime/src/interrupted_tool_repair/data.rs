//! Typed interrupted-tool repair inputs and decisions.

/// Versioned interrupted-tool repair contract.
pub const INTERRUPTED_TOOL_REPAIR_SCHEMA: &str = "animus.interrupted-tool-repair.v1";
/// BLAKE3 domain for active-branch snapshot identities.
pub const INTERRUPTED_TOOL_SNAPSHOT_HASH_DOMAIN: &str = "animus.interrupted-tool-snapshot.v1";
/// BLAKE3 domain for synthetic repair actions.
pub const INTERRUPTED_TOOL_ACTION_HASH_DOMAIN: &str = "animus.interrupted-tool-action.v1";
/// BLAKE3 domain for repair plans.
pub const INTERRUPTED_TOOL_PLAN_HASH_DOMAIN: &str = "animus.interrupted-tool-plan.v1";
/// BLAKE3 domain for proposed repaired leaves.
pub const INTERRUPTED_TOOL_LEAF_HASH_DOMAIN: &str = "animus.interrupted-tool-leaf.v1";
/// BLAKE3 domain for metadata-only repair receipts.
pub const INTERRUPTED_TOOL_RECEIPT_HASH_DOMAIN: &str = "animus.interrupted-tool-receipt.v1";

pub(super) const BLAKE3_HEX_CHARS: usize = 64;
pub(super) const MAX_BRANCH_ENTRIES: usize = 4_096;
pub(super) const MAX_CALLS_PER_ENTRY: usize = 64;
pub(super) const MAX_TOTAL_TOOL_CALLS: usize = 4_096;
pub(super) const MAX_INTERRUPTION_OBSERVATIONS: usize = 256;
pub(super) const MAX_PREVIOUSLY_APPLIED_PLANS: usize = 256;
pub(super) const MAX_SAFE_LABEL_CHARS: usize = 256;
pub(super) const HASH_SEPARATOR: &str = "\u{1f}";

/// Model-visible active-branch entry kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InterruptedToolEntryKind {
    /// System message or policy marker with no tool binding.
    SystemMessage,
    /// User message with no tool binding.
    UserMessage,
    /// Assistant message with no tool calls.
    AssistantMessage,
    /// Assistant message carrying one or more tool calls.
    AssistantToolCalls,
    /// Terminal tool result for one prior call.
    ToolResult,
    /// Previously committed synthetic terminal result.
    SyntheticToolResult,
    /// Compaction or replay metadata that remains provider-visible only through a validated
    /// projection.
    ReplayMetadata,
    /// Unsupported entry used to prove fail-closed behavior.
    Unsupported,
}

impl InterruptedToolEntryKind {
    /// Returns the stable identity label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SystemMessage => "system_message",
            Self::UserMessage => "user_message",
            Self::AssistantMessage => "assistant_message",
            Self::AssistantToolCalls => "assistant_tool_calls",
            Self::ToolResult => "tool_result",
            Self::SyntheticToolResult => "synthetic_tool_result",
            Self::ReplayMetadata => "replay_metadata",
            Self::Unsupported => "unsupported",
        }
    }
}

/// One assistant tool-call binding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InterruptedToolCall {
    /// Stable call identifier used by provider correlation.
    pub call_id: String,
    /// BLAKE3 identity of the tool definition, never raw arguments.
    pub tool_identity_hash: String,
}

/// One terminal tool-result binding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InterruptedToolResult {
    /// Stable call identifier satisfied by this result.
    pub call_id: String,
    /// BLAKE3 identity of the compatible tool definition.
    pub tool_identity_hash: String,
}

/// Already-loaded entry on the selected active branch.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InterruptedToolBranchEntry {
    /// Stable entry label.
    pub entry_id: String,
    /// BLAKE3 identity of the immutable entry body.
    pub entry_hash: String,
    /// Run identity used to correlate interruption evidence.
    pub run_id: String,
    /// Entry kind.
    pub kind: InterruptedToolEntryKind,
    /// Tool calls carried by an assistant tool-call entry.
    pub tool_calls: Vec<InterruptedToolCall>,
    /// Tool result carried by a result entry.
    pub tool_result: Option<InterruptedToolResult>,
}

/// Already-loaded selected active-branch snapshot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InterruptedToolBranchSnapshot {
    /// Safe session identity.
    pub session_id: String,
    /// Safe branch identity.
    pub branch_id: String,
    /// Current active-leaf BLAKE3 identity observed by the host.
    pub active_leaf_hash: String,
    /// Leaf identity bound when the snapshot was loaded.
    pub snapshot_leaf_hash: String,
    /// Ordered active-branch entries.
    pub entries: Vec<InterruptedToolBranchEntry>,
}

/// Structural diagnosis state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InterruptedToolDiagnosisStatus {
    /// Every call has exactly one compatible result.
    Clean,
    /// One or more calls lack a terminal result and are repairable.
    Repairable,
}

/// One call lacking a terminal result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InterruptedToolGap {
    pub(super) entry_index: usize,
    pub(super) call_index: usize,
    pub(super) entry_id: String,
    pub(super) entry_hash: String,
    pub(super) run_id: String,
    pub(super) call: InterruptedToolCall,
}

impl InterruptedToolGap {
    /// Returns the entry that introduced the unmatched call.
    #[must_use]
    pub fn entry_id(&self) -> &str {
        self.entry_id.as_str()
    }

    /// Returns the unmatched call identifier.
    #[must_use]
    pub fn call_id(&self) -> &str {
        self.call.call_id.as_str()
    }

    /// Returns the call's tool-definition identity.
    #[must_use]
    pub fn tool_identity_hash(&self) -> &str {
        self.call.tool_identity_hash.as_str()
    }

    /// Returns the run identity used for evidence correlation.
    #[must_use]
    pub fn run_id(&self) -> &str {
        self.run_id.as_str()
    }
}

/// Pure active-branch binding diagnosis.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InterruptedToolDiagnosis {
    pub(super) status: InterruptedToolDiagnosisStatus,
    pub(super) snapshot_hash: String,
    pub(super) active_leaf_hash: String,
    pub(super) matched_call_count: u32,
    pub(super) unmatched: Vec<InterruptedToolGap>,
}

impl InterruptedToolDiagnosis {
    /// Returns clean or repairable status.
    #[must_use]
    pub const fn status(&self) -> InterruptedToolDiagnosisStatus {
        self.status
    }

    /// Returns the canonical snapshot identity.
    #[must_use]
    pub fn snapshot_hash(&self) -> &str {
        self.snapshot_hash.as_str()
    }

    /// Returns the leaf identity bound by this diagnosis.
    #[must_use]
    pub fn active_leaf_hash(&self) -> &str {
        self.active_leaf_hash.as_str()
    }

    /// Returns the number of calls with one compatible result.
    #[must_use]
    pub const fn matched_call_count(&self) -> u32 {
        self.matched_call_count
    }

    /// Returns calls requiring synthetic terminal results.
    #[must_use]
    pub fn unmatched(&self) -> &[InterruptedToolGap] {
        self.unmatched.as_slice()
    }
}

/// Host-observed interruption class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InterruptedToolObservationKind {
    /// Explicit user cancellation correlated to the run and call batch.
    UserCancellation,
    /// Explicit host cancellation correlated to the run and call batch.
    HostCancellation,
    /// Host observed an unclean shutdown for the run and call batch.
    UncleanShutdown,
}

impl InterruptedToolObservationKind {
    /// Returns the stable observation label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UserCancellation => "user_cancellation",
            Self::HostCancellation => "host_cancellation",
            Self::UncleanShutdown => "unclean_shutdown",
        }
    }
}

/// Correlated host interruption evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InterruptedToolObservation {
    /// Observation class.
    pub kind: InterruptedToolObservationKind,
    /// Exact run identity.
    pub run_id: String,
    /// Exact affected call IDs.
    pub affected_call_ids: Vec<String>,
}

/// Evidence-bound synthetic result reason.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum InterruptedToolReason {
    /// Explicit correlated user cancellation.
    UserCancelled,
    /// Explicit correlated host cancellation.
    HostCancelled,
    /// Explicit correlated unclean shutdown.
    UncleanShutdown,
    /// No correlated causal observation was supplied.
    UnknownInterruption,
}

impl InterruptedToolReason {
    /// Returns the stable result and receipt label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UserCancelled => "user_cancelled",
            Self::HostCancelled => "host_cancelled",
            Self::UncleanShutdown => "unclean_shutdown",
            Self::UnknownInterruption => "unknown_interruption",
        }
    }
}

/// Append-only application shape.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InterruptedToolRepairMode {
    /// Append synthetic results at the current tail.
    TailAppend,
    /// Append a repaired active branch that replays the preserved suffix.
    HistoricalBranch,
}

impl InterruptedToolRepairMode {
    /// Returns the stable mode label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::TailAppend => "tail_append",
            Self::HistoricalBranch => "historical_branch",
        }
    }
}

/// One synthetic terminal-failure action.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InterruptedToolRepairAction {
    pub(super) call_id: String,
    pub(super) tool_identity_hash: String,
    pub(super) run_id: String,
    pub(super) anchor_entry_id: String,
    pub(super) reason: InterruptedToolReason,
    pub(super) action_hash: String,
}

impl InterruptedToolRepairAction {
    /// Returns the call receiving a synthetic result.
    #[must_use]
    pub fn call_id(&self) -> &str {
        self.call_id.as_str()
    }

    /// Returns the compatible tool-definition identity.
    #[must_use]
    pub fn tool_identity_hash(&self) -> &str {
        self.tool_identity_hash.as_str()
    }

    /// Returns the run identity correlated to the interruption evidence.
    #[must_use]
    pub fn run_id(&self) -> &str {
        self.run_id.as_str()
    }

    /// Returns the assistant entry after which the synthetic result belongs.
    #[must_use]
    pub fn anchor_entry_id(&self) -> &str {
        self.anchor_entry_id.as_str()
    }

    /// Returns the evidence-bound interruption reason.
    #[must_use]
    pub const fn reason(&self) -> InterruptedToolReason {
        self.reason
    }

    /// Returns the action identity.
    #[must_use]
    pub fn action_hash(&self) -> &str {
        self.action_hash.as_str()
    }
}

/// Deterministic append-only repair plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InterruptedToolRepairPlan {
    pub(super) mode: InterruptedToolRepairMode,
    pub(super) session_id: String,
    pub(super) branch_id: String,
    pub(super) old_leaf_hash: String,
    pub(super) proposed_new_leaf_hash: String,
    pub(super) repair_anchor_entry_id: String,
    pub(super) affected_suffix_entry_hashes: Vec<String>,
    pub(super) actions: Vec<InterruptedToolRepairAction>,
    pub(super) snapshot_hash: String,
    pub(super) plan_hash: String,
}

impl InterruptedToolRepairPlan {
    /// Returns tail or historical append-only mode.
    #[must_use]
    pub const fn mode(&self) -> InterruptedToolRepairMode {
        self.mode
    }

    /// Returns the old active-leaf identity.
    #[must_use]
    pub fn old_leaf_hash(&self) -> &str {
        self.old_leaf_hash.as_str()
    }

    /// Returns the proposed repaired-leaf identity.
    #[must_use]
    pub fn proposed_new_leaf_hash(&self) -> &str {
        self.proposed_new_leaf_hash.as_str()
    }

    /// Returns the repair anchor entry.
    #[must_use]
    pub fn repair_anchor_entry_id(&self) -> &str {
        self.repair_anchor_entry_id.as_str()
    }

    /// Returns preserved suffix entry identities.
    #[must_use]
    pub fn affected_suffix_entry_hashes(&self) -> &[String] {
        self.affected_suffix_entry_hashes.as_slice()
    }

    /// Returns ordered synthetic-result actions.
    #[must_use]
    pub fn actions(&self) -> &[InterruptedToolRepairAction] {
        self.actions.as_slice()
    }

    /// Returns the source snapshot identity.
    #[must_use]
    pub fn snapshot_hash(&self) -> &str {
        self.snapshot_hash.as_str()
    }

    /// Returns the canonical plan identity.
    #[must_use]
    pub fn plan_hash(&self) -> &str {
        self.plan_hash.as_str()
    }
}

/// Host application request effect class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InterruptedToolRepairEffect {
    /// Append synthetic terminal failures and optionally replay a preserved suffix.
    AppendSyntheticFailures,
}

/// Metadata-only append request passed to a host-owned store.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InterruptedToolRepairApplicationRequest {
    /// The only admitted repair effect.
    pub effect: InterruptedToolRepairEffect,
    /// Tail or historical append mode.
    pub mode: InterruptedToolRepairMode,
    /// Exact session identity selected by the host.
    pub session_id: String,
    /// Exact active-branch identity selected by the host.
    pub branch_id: String,
    /// Plan identity.
    pub plan_hash: String,
    /// Expected current leaf.
    pub expected_old_leaf_hash: String,
    /// Proposed replacement leaf.
    pub proposed_new_leaf_hash: String,
    /// Entry after which repair starts.
    pub repair_anchor_entry_id: String,
    /// Ordered synthetic terminal-failure actions.
    pub actions: Vec<InterruptedToolRepairAction>,
    /// Ordered action identities.
    pub action_hashes: Vec<String>,
    /// Preserved suffix entry identities for historical replay.
    pub affected_suffix_entry_hashes: Vec<String>,
}

/// Host durability observation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InterruptedToolDurability {
    /// Every action and the new active leaf were durably committed.
    Durable,
    /// Only part of the plan was committed or durability is uncertain.
    Partial,
    /// Store application failed.
    StoreFailure,
}

/// Host replay revalidation observation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InterruptedToolReplayState {
    /// Replayed context satisfies the terminal-binding invariant.
    Clean,
    /// Replayed context remains invalid.
    Invalid,
    /// Replay validation did not run.
    NotValidated,
}

/// Explicit observation proving whether repair invoked a tool executor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InterruptedToolExecutorObservation {
    /// No tool executor was invoked.
    NotInvoked,
    /// A tool executor was invoked, which invalidates repair.
    Invoked,
}

/// Borrowed host application evidence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InterruptedToolRepairApplicationDraft<'a> {
    /// Plan the host attempted to apply.
    pub plan_hash: &'a str,
    /// Leaf observed before application, or the prior leaf when checking idempotence.
    pub observed_old_leaf_hash: &'a str,
    /// Leaf observed after application.
    pub observed_new_leaf_hash: &'a str,
    /// Ordered committed action identities.
    pub committed_action_hashes: &'a [&'a str],
    /// Previously durably applied plan identities.
    pub previously_applied_plan_hashes: &'a [&'a str],
    /// Durability observation.
    pub durability: InterruptedToolDurability,
    /// Replay validation observation.
    pub replay_state: InterruptedToolReplayState,
    /// Executor non-invocation observation.
    pub executor_observation: InterruptedToolExecutorObservation,
}

/// Repair application outcome class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InterruptedToolRepairApplicationStatus {
    /// The complete plan was durably applied and replay is clean.
    Applied,
    /// The same plan was already applied and the repaired leaf remains current.
    AlreadyApplied,
    /// The plan or leaf identity was stale.
    BlockedStale,
    /// Application was partial or action coverage was incomplete.
    BlockedPartial,
    /// The store reported failure.
    BlockedStore,
    /// Replay was invalid or absent.
    BlockedReplay,
    /// Repair attempted tool re-execution.
    BlockedToolExecution,
}

impl InterruptedToolRepairApplicationStatus {
    /// Returns the stable outcome label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Applied => "applied",
            Self::AlreadyApplied => "already_applied",
            Self::BlockedStale => "blocked_stale",
            Self::BlockedPartial => "blocked_partial",
            Self::BlockedStore => "blocked_store",
            Self::BlockedReplay => "blocked_replay",
            Self::BlockedToolExecution => "blocked_tool_execution",
        }
    }

    /// Returns true only when provider continuation may rely on the application.
    #[must_use]
    pub const fn permits_provider_continuation(self) -> bool {
        matches!(self, Self::Applied | Self::AlreadyApplied)
    }
}

/// Validated host application outcome.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InterruptedToolRepairApplicationOutcome {
    pub(super) status: InterruptedToolRepairApplicationStatus,
    pub(super) plan_hash: String,
    pub(super) old_leaf_hash: String,
    pub(super) new_leaf_hash: String,
    pub(super) committed_action_count: u32,
}

impl InterruptedToolRepairApplicationOutcome {
    /// Returns application status.
    #[must_use]
    pub const fn status(&self) -> InterruptedToolRepairApplicationStatus {
        self.status
    }

    /// Returns the bound plan identity.
    #[must_use]
    pub fn plan_hash(&self) -> &str {
        self.plan_hash.as_str()
    }

    /// Returns the resulting leaf identity.
    #[must_use]
    pub fn new_leaf_hash(&self) -> &str {
        self.new_leaf_hash.as_str()
    }

    /// Returns the number of committed actions reported by the host.
    #[must_use]
    pub const fn committed_action_count(&self) -> u32 {
        self.committed_action_count
    }
}

/// Provider continuation gate class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InterruptedToolProviderGateStatus {
    /// Original active branch was clean.
    ReadyClean,
    /// A complete durable repair made replay clean.
    ReadyRepaired,
    /// Unmatched calls still require repair.
    BlockedRepairRequired,
    /// Repair application did not pass.
    BlockedApplication,
}

/// Pure provider-continuation decision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InterruptedToolProviderGateDecision {
    pub(super) status: InterruptedToolProviderGateStatus,
    pub(super) repair_plan_hash: Option<String>,
}

impl InterruptedToolProviderGateDecision {
    /// Returns gate status.
    #[must_use]
    pub const fn status(&self) -> InterruptedToolProviderGateStatus {
        self.status
    }

    /// Returns true only for clean original or durably repaired replay.
    #[must_use]
    pub const fn provider_dispatch_allowed(&self) -> bool {
        matches!(
            self.status,
            InterruptedToolProviderGateStatus::ReadyClean | InterruptedToolProviderGateStatus::ReadyRepaired
        )
    }

    /// Returns the repair plan identity when repair was evaluated.
    #[must_use]
    pub fn repair_plan_hash(&self) -> Option<&str> {
        self.repair_plan_hash.as_deref()
    }
}

/// Receipt input fields that must never be durable evidence.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct InterruptedToolRepairSensitiveDraft<'a> {
    /// Raw prompt or instruction.
    pub raw_prompt: Option<&'a str>,
    /// Raw tool arguments.
    pub raw_tool_arguments: Option<&'a str>,
    /// Raw tool output.
    pub raw_tool_output: Option<&'a str>,
    /// Raw provider payload.
    pub provider_payload: Option<&'a str>,
    /// Credential or secret.
    pub credential: Option<&'a str>,
    /// Private or absolute path.
    pub private_path: Option<&'a str>,
    /// Raw exception body.
    pub exception_body: Option<&'a str>,
    /// Raw or unbounded transcript.
    pub transcript: Option<&'a str>,
}

/// Metadata-only interrupted-tool repair receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InterruptedToolRepairReceipt {
    pub(super) schema: &'static str,
    pub(super) status: InterruptedToolRepairApplicationStatus,
    pub(super) mode: InterruptedToolRepairMode,
    pub(super) old_leaf_hash: String,
    pub(super) new_leaf_hash: String,
    pub(super) plan_hash: String,
    pub(super) action_count: u32,
    pub(super) reason_classes: Vec<InterruptedToolReason>,
    pub(super) redaction_marker: &'static str,
    pub(super) non_claim: &'static str,
    pub(super) receipt_hash: String,
}

impl InterruptedToolRepairReceipt {
    /// Returns the versioned receipt schema.
    #[must_use]
    pub const fn schema(&self) -> &'static str {
        self.schema
    }

    /// Returns repair application status.
    #[must_use]
    pub const fn status(&self) -> InterruptedToolRepairApplicationStatus {
        self.status
    }

    /// Returns the plan identity.
    #[must_use]
    pub fn plan_hash(&self) -> &str {
        self.plan_hash.as_str()
    }

    /// Returns the number of synthetic actions.
    #[must_use]
    pub const fn action_count(&self) -> u32 {
        self.action_count
    }

    /// Returns sorted reason classes.
    #[must_use]
    pub fn reason_classes(&self) -> &[InterruptedToolReason] {
        self.reason_classes.as_slice()
    }

    /// Returns the metadata-only redaction marker.
    #[must_use]
    pub const fn redaction_marker(&self) -> &'static str {
        self.redaction_marker
    }

    /// Returns the bounded non-claim.
    #[must_use]
    pub const fn non_claim(&self) -> &'static str {
        self.non_claim
    }

    /// Returns the receipt identity.
    #[must_use]
    pub fn receipt_hash(&self) -> &str {
        self.receipt_hash.as_str()
    }
}

/// Interrupted-tool diagnosis, planning, application, or receipt issue.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InterruptedToolRepairIssue {
    /// A label, hash, or collection was malformed or out of bounds.
    InvalidInput,
    /// Active leaf and loaded snapshot were stale or inconsistent.
    StaleLeaf,
    /// An entry shape did not match its declared kind.
    InvalidEntryShape,
    /// An unsupported entry appeared on the active branch.
    UnsupportedEntry,
    /// A call identity appeared more than once.
    DuplicateCall,
    /// A call received more than one result.
    DuplicateResult,
    /// A result appeared before its call or referenced no call.
    ResultBeforeCall,
    /// Result and call run identities differed.
    RunMismatch,
    /// Result and call tool identities differed.
    ToolMismatch,
    /// Interruption observations made conflicting causal claims.
    ConflictingEvidence,
    /// Diagnosis and snapshot or anchor identities did not match.
    MissingRepairAnchor,
    /// Receipt input contained private or raw material.
    ReceiptLeak,
    /// Provider projection was attempted without passing repair evidence.
    ProviderGateBlocked,
}

/// Safe interrupted-tool repair error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InterruptedToolRepairError {
    issue: InterruptedToolRepairIssue,
    field: &'static str,
}

impl InterruptedToolRepairError {
    pub(super) const fn new(issue: InterruptedToolRepairIssue, field: &'static str) -> Self {
        Self { issue, field }
    }

    /// Returns the typed issue class.
    #[must_use]
    pub const fn issue(&self) -> InterruptedToolRepairIssue {
        self.issue
    }

    /// Returns the safe field label.
    #[must_use]
    pub const fn field(&self) -> &'static str {
        self.field
    }
}

impl core::fmt::Display for InterruptedToolRepairError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "interrupted-tool repair rejected {}", self.field)
    }
}

impl std::error::Error for InterruptedToolRepairError {}

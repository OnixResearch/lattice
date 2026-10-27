//! Typed active-turn recovery inputs, plans, outcomes, and receipts.

// Repeated active-turn prefixes keep public DTO ownership explicit at the compatibility boundary.
#![allow(unknown_lints, path_segment_repetition)]

#[allow(unused_imports)]
use crate::prelude::*;

/// Versioned active-turn recovery contract.
pub const ACTIVE_TURN_RECOVERY_SCHEMA: &str = "animus.active-turn-recovery.v1";
/// BLAKE3 domain for policy identities.
pub const ACTIVE_TURN_POLICY_HASH_DOMAIN: &str = "animus.active-turn-recovery-policy.v1";
/// BLAKE3 domain for turn identities.
pub const ACTIVE_TURN_HASH_DOMAIN: &str = "animus.active-turn.v1";
/// BLAKE3 domain for transaction identities.
pub const ACTIVE_TURN_TRANSACTION_HASH_DOMAIN: &str = "animus.active-turn-transaction.v1";
/// BLAKE3 domain for checkpoint identities.
pub const ACTIVE_TURN_CHECKPOINT_HASH_DOMAIN: &str = "animus.active-turn-checkpoint.v1";
/// BLAKE3 domain for steering identities.
pub const ACTIVE_TURN_STEERING_HASH_DOMAIN: &str = "animus.active-turn-steering.v1";
/// BLAKE3 domain for durable agent-control identities.
pub const ACTIVE_TURN_CONTROL_HASH_DOMAIN: &str = "animus.active-turn-control.v1";
/// BLAKE3 domain for recovery-plan identities.
pub const ACTIVE_TURN_PLAN_HASH_DOMAIN: &str = "animus.active-turn-plan.v1";
/// BLAKE3 domain for metadata-only receipts.
pub const ACTIVE_TURN_RECEIPT_HASH_DOMAIN: &str = "animus.active-turn-receipt.v1";

pub(super) const BLAKE3_HEX_CHARS: usize = 64;
pub(super) const MAX_SAFE_LABEL_CHARS: usize = 128;
pub(super) const MAX_POLICY_COLLECTION_ITEMS: usize = 64;
pub(super) const HASH_SEPARATOR: &str = "\u{1f}";

/// Generated policy input before strict Rust validation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActiveTurnRecoveryPolicyInput {
    /// Policy schema.
    pub schema: String,
    /// Safe policy label.
    pub policy_id: String,
    /// Maximum journal entries.
    pub max_entries: u32,
    /// Maximum bytes in one prepared entry.
    pub max_entry_bytes: u32,
    /// Maximum bytes in one checkpoint.
    pub max_aggregate_bytes: u64,
    /// Maximum pending steering entries.
    pub max_steering_entries: u32,
    /// Required fail-closed corruption posture.
    pub corruption_posture: String,
    /// Required compare-and-swap replacement posture.
    pub replacement_posture: String,
    /// Whether transaction evidence is mandatory.
    pub transaction_evidence_required: bool,
    /// Safe retention class.
    pub retention_class: String,
    /// Allowed receipt field labels.
    pub receipt_fields: Vec<String>,
    /// Required non-claim labels.
    pub non_claims: Vec<String>,
    /// Source-policy BLAKE3 hash.
    pub policy_hash: String,
}

/// Strict validated active-turn recovery policy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActiveTurnRecoveryPolicy {
    pub(super) input: ActiveTurnRecoveryPolicyInput,
    pub(super) identity: String,
}

impl ActiveTurnRecoveryPolicy {
    /// Returns the domain-separated policy identity.
    #[must_use]
    pub fn identity(&self) -> &str {
        self.identity.as_str()
    }

    /// Returns the maximum journal entries.
    #[must_use]
    pub const fn max_entries(&self) -> u32 {
        self.input.max_entries
    }

    /// Returns the maximum entry bytes.
    #[must_use]
    pub const fn max_entry_bytes(&self) -> u32 {
        self.input.max_entry_bytes
    }

    /// Returns the maximum aggregate bytes.
    #[must_use]
    pub const fn max_aggregate_bytes(&self) -> u64 {
        self.input.max_aggregate_bytes
    }

    /// Returns the maximum pending steering entries.
    #[must_use]
    pub const fn max_steering_entries(&self) -> u32 {
        self.input.max_steering_entries
    }
}

/// One accepted-message identity without transcript content.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActiveTurnAcceptanceDraft {
    /// Safe session identity.
    pub session_id: String,
    /// Safe branch identity.
    pub branch_id: String,
    /// Monotonic turn sequence.
    pub turn_sequence: u64,
    /// BLAKE3 identity of the accepted message.
    pub accepted_message_hash: String,
    /// Expected prior journal generation.
    pub expected_generation: u64,
}

/// Atomic durable-acceptance transaction plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActiveTurnAcceptancePlan {
    pub(super) session_id: String,
    pub(super) branch_id: String,
    pub(super) turn_sequence: u64,
    pub(super) accepted_message_hash: String,
    pub(super) expected_generation: u64,
    pub(super) next_generation: u64,
    pub(super) policy_hash: String,
    pub(super) turn_hash: String,
    pub(super) transaction_hash: String,
}

impl ActiveTurnAcceptancePlan {
    /// Returns the stable active-turn identity.
    #[must_use]
    pub fn turn_hash(&self) -> &str {
        self.turn_hash.as_str()
    }

    /// Returns the safe session identity.
    #[must_use]
    pub fn session_id(&self) -> &str {
        self.session_id.as_str()
    }

    /// Returns the safe branch identity.
    #[must_use]
    pub fn branch_id(&self) -> &str {
        self.branch_id.as_str()
    }

    /// Returns the atomic transaction identity.
    #[must_use]
    pub fn transaction_hash(&self) -> &str {
        self.transaction_hash.as_str()
    }

    /// Returns the expected prior generation.
    #[must_use]
    pub const fn expected_generation(&self) -> u64 {
        self.expected_generation
    }

    /// Returns the generation produced by a complete transaction.
    #[must_use]
    pub const fn next_generation(&self) -> u64 {
        self.next_generation
    }
}

/// Host transaction result class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActiveTurnTransactionStatus {
    /// The complete transaction committed durably.
    Committed,
    /// The expected generation was stale.
    Stale,
    /// Only part of the mutation was observed.
    Partial,
    /// The store rejected the transaction.
    StoreFailure,
}

impl ActiveTurnTransactionStatus {
    /// Returns the stable status label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Committed => "committed",
            Self::Stale => "stale",
            Self::Partial => "partial",
            Self::StoreFailure => "store_failure",
        }
    }
}

/// Host evidence for one acceptance or replacement transaction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActiveTurnTransactionObservation {
    /// Exact transaction identity.
    pub transaction_hash: String,
    /// Generation before the transaction.
    pub observed_prior_generation: u64,
    /// Generation visible after the transaction.
    pub observed_generation: u64,
    /// Transaction result.
    pub status: ActiveTurnTransactionStatus,
    /// Whether all planned identities became visible together.
    pub complete_manifest: bool,
}

/// Durable acceptance outcome.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActiveTurnAcceptanceOutcome {
    pub(super) turn_hash: String,
    pub(super) generation: u64,
    pub(super) status: ActiveTurnTransactionStatus,
    pub(super) provider_eligible: bool,
}

impl ActiveTurnAcceptanceOutcome {
    /// Returns whether provider dispatch can use this evidence.
    #[must_use]
    pub const fn provider_eligible(&self) -> bool {
        self.provider_eligible
    }

    /// Returns the transaction status.
    #[must_use]
    pub const fn status(&self) -> ActiveTurnTransactionStatus {
        self.status
    }

    /// Returns the active-turn identity.
    #[must_use]
    pub fn turn_hash(&self) -> &str {
        self.turn_hash.as_str()
    }
}

/// Journal entry class used for recovery ordering.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActiveTurnJournalEntryKind {
    /// Accepted user work.
    AcceptedMessage,
    /// Provider-visible message or event.
    ProviderEvent,
    /// Tool call preserved for interrupted-tool repair.
    ToolCall,
    /// Real terminal tool result.
    ToolResult,
    /// Synthetic terminal result produced by the accepted repair core.
    SyntheticToolResult,
    /// Durable steering work.
    Steering,
    /// Exact control state parks the active turn.
    ControlParked,
    /// Exact control state makes the active turn runnable.
    ControlRunning,
    /// Terminal turn fact.
    Terminal,
    /// Unsupported record used for fail-closed tests.
    Unsupported,
}

impl ActiveTurnJournalEntryKind {
    /// Returns the stable entry label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::AcceptedMessage => "accepted_message",
            Self::ProviderEvent => "provider_event",
            Self::ToolCall => "tool_call",
            Self::ToolResult => "tool_result",
            Self::SyntheticToolResult => "synthetic_tool_result",
            Self::Steering => "steering",
            Self::ControlParked => "control_parked",
            Self::ControlRunning => "control_running",
            Self::Terminal => "terminal",
            Self::Unsupported => "unsupported",
        }
    }
}

/// Input for one durable agent-control checkpoint entry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ActiveTurnControlEntryDraft<'a> {
    /// Exact active-turn identity.
    pub turn_hash: &'a str,
    /// Monotonic journal sequence.
    pub sequence: u32,
    /// Prepared serialized size.
    pub prepared_bytes: u32,
    /// Exact validated control state.
    pub control: &'a crate::AgentControlState,
}

/// One fully prepared journal entry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActiveTurnPreparedEntry {
    /// Monotonic entry sequence.
    pub sequence: u32,
    /// Safe entry class.
    pub kind: ActiveTurnJournalEntryKind,
    /// BLAKE3 identity of the immutable entry body.
    pub entry_hash: String,
    /// Prepared serialized size.
    pub prepared_bytes: u32,
    /// Optional tool-call correlation identity.
    pub tool_call_id: Option<String>,
}

/// Complete checkpoint replacement input.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActiveTurnReplacementDraft {
    /// Exact active-turn identity.
    pub turn_hash: String,
    /// Expected prior generation.
    pub expected_generation: u64,
    /// Fully prepared ordered entries.
    pub entries: Vec<ActiveTurnPreparedEntry>,
}

/// Atomic checkpoint replacement plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActiveTurnReplacementPlan {
    pub(super) turn_hash: String,
    pub(super) expected_generation: u64,
    pub(super) next_generation: u64,
    pub(super) entry_hashes: Vec<String>,
    pub(super) aggregate_bytes: u64,
    pub(super) checkpoint_hash: String,
    pub(super) transaction_hash: String,
}

impl ActiveTurnReplacementPlan {
    /// Returns the checkpoint identity.
    #[must_use]
    pub fn checkpoint_hash(&self) -> &str {
        self.checkpoint_hash.as_str()
    }

    /// Returns the exact active-turn identity.
    #[must_use]
    pub fn turn_hash(&self) -> &str {
        self.turn_hash.as_str()
    }

    /// Returns the expected prior generation.
    #[must_use]
    pub const fn expected_generation(&self) -> u64 {
        self.expected_generation
    }

    /// Returns the next generation.
    #[must_use]
    pub const fn next_generation(&self) -> u64 {
        self.next_generation
    }

    /// Returns the transaction identity.
    #[must_use]
    pub fn transaction_hash(&self) -> &str {
        self.transaction_hash.as_str()
    }

    /// Returns ordered entry identities.
    #[must_use]
    pub fn entry_hashes(&self) -> &[String] {
        self.entry_hashes.as_slice()
    }
}

/// Steering append input before validation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActiveTurnSteeringDraft {
    /// Exact active-turn identity.
    pub turn_hash: String,
    /// Expected current generation.
    pub expected_generation: u64,
    /// Current pending steering count.
    pub pending_count: u32,
    /// Next monotonic steering sequence.
    pub steering_sequence: u32,
    /// BLAKE3 identity of the steering body.
    pub steering_body_hash: String,
}

/// Steering append plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActiveTurnSteeringPlan {
    pub(super) turn_hash: String,
    pub(super) steering_sequence: u32,
    pub(super) steering_hash: String,
    pub(super) expected_generation: u64,
    pub(super) append_hash: String,
}

impl ActiveTurnSteeringPlan {
    /// Returns the durable steering identity.
    #[must_use]
    pub fn steering_hash(&self) -> &str {
        self.steering_hash.as_str()
    }

    /// Returns the exact active-turn identity.
    #[must_use]
    pub fn turn_hash(&self) -> &str {
        self.turn_hash.as_str()
    }

    /// Returns the steering sequence.
    #[must_use]
    pub const fn steering_sequence(&self) -> u32 {
        self.steering_sequence
    }

    /// Returns the expected generation.
    #[must_use]
    pub const fn expected_generation(&self) -> u64 {
        self.expected_generation
    }

    /// Returns the append transaction identity.
    #[must_use]
    pub fn append_hash(&self) -> &str {
        self.append_hash.as_str()
    }
}

/// Terminal state supplied by the host.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActiveTurnTerminalClass {
    /// No terminal fact exists.
    Active,
    /// Turn completed under its separate completion contract.
    Completed,
    /// Turn was cancelled.
    Cancelled,
    /// Turn was abandoned.
    Abandoned,
    /// Conflicting terminal facts exist.
    Conflicting,
}

impl ActiveTurnTerminalClass {
    /// Returns the stable terminal label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
            Self::Abandoned => "abandoned",
            Self::Conflicting => "conflicting",
        }
    }
}

/// Existing interrupted-tool repair state supplied to recovery.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActiveTurnRepairState {
    /// No unmatched tool call exists.
    Clean,
    /// The accepted interrupted-tool repair core must plan a synthetic result.
    RepairRequired,
    /// A complete durable repair exists.
    Repaired,
    /// Repair evidence is invalid or incomplete.
    Blocked,
}

/// Durable acceptance evidence state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActiveTurnAcceptanceState {
    /// Exact durable acceptance evidence exists.
    Durable,
    /// Durable acceptance evidence is absent or mismatched.
    Missing,
}

/// Continuation authority state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActiveTurnContinuationAuthority {
    /// Current authority admits continuation.
    Allowed,
    /// Current authority denies continuation.
    Denied,
}

/// Cancellation state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActiveTurnCancellationState {
    /// No current cancellation blocks continuation.
    Active,
    /// Current cancellation blocks continuation.
    Cancelled,
}

/// Terminal-cleanup state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActiveTurnCleanupState {
    /// Terminal recovery state remains retained.
    Retained,
    /// Terminal recovery state was cleaned.
    Complete,
}

/// Validated active-turn snapshot for recovery planning.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActiveTurnRecoverySnapshot {
    /// Exact active-turn identity.
    pub turn_hash: String,
    /// Current complete checkpoint identity.
    pub checkpoint_hash: String,
    /// Current generation.
    pub generation: u64,
    /// Ordered checkpoint entries.
    pub entries: Vec<ActiveTurnPreparedEntry>,
    /// Durable acceptance evidence state.
    pub acceptance: ActiveTurnAcceptanceState,
    /// Current continuation authority.
    pub authority: ActiveTurnContinuationAuthority,
    /// Current cancellation state.
    pub cancellation: ActiveTurnCancellationState,
    /// Existing interrupted-tool repair state.
    pub repair_state: ActiveTurnRepairState,
    /// Durable terminal observation.
    pub terminal: ActiveTurnTerminalClass,
    /// Terminal-cleanup state.
    pub cleanup: ActiveTurnCleanupState,
}

/// Recovery action class. No variant can request tool execution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActiveTurnRecoveryAction {
    /// Continue through the existing provider-plan gate.
    ContinueProvider,
    /// Invoke the pure interrupted-tool repair planner.
    PlanInterruptedToolRepair,
    /// Observe the durable terminal fact without provider or tool effects.
    ObserveTerminal,
    /// Clean retained active state after the terminal commit.
    CleanupRequired,
    /// Keep the durable active turn parked without provider or tool dispatch.
    Parked,
    /// Stop because supplied facts do not admit continuation.
    Blocked,
}

impl ActiveTurnRecoveryAction {
    /// Returns the stable action label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ContinueProvider => "continue_provider",
            Self::PlanInterruptedToolRepair => "plan_interrupted_tool_repair",
            Self::ObserveTerminal => "observe_terminal",
            Self::CleanupRequired => "cleanup_required",
            Self::Parked => "parked",
            Self::Blocked => "blocked",
        }
    }
}

/// Pure recovery plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActiveTurnRecoveryPlan {
    pub(super) turn_hash: String,
    pub(super) checkpoint_hash: String,
    pub(super) generation: u64,
    pub(super) action: ActiveTurnRecoveryAction,
    pub(super) terminal: ActiveTurnTerminalClass,
    pub(super) plan_hash: String,
}

impl ActiveTurnRecoveryPlan {
    /// Returns the planned recovery action.
    #[must_use]
    pub const fn action(&self) -> ActiveTurnRecoveryAction {
        self.action
    }

    /// Returns the recovery plan identity.
    #[must_use]
    pub fn plan_hash(&self) -> &str {
        self.plan_hash.as_str()
    }

    /// Returns whether provider continuation is permitted by this plan.
    #[must_use]
    pub const fn provider_continuation_allowed(&self) -> bool {
        matches!(self.action, ActiveTurnRecoveryAction::ContinueProvider)
    }
}

/// Receipt fields that must never enter durable recovery evidence.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ActiveTurnRecoverySensitiveDraft<'a> {
    /// Raw message or prompt.
    pub message: Option<&'a str>,
    /// Raw reasoning.
    pub reasoning: Option<&'a str>,
    /// Raw tool body.
    pub tool_body: Option<&'a str>,
    /// Raw provider payload.
    pub provider_payload: Option<&'a str>,
    /// Credential material.
    pub credential: Option<&'a str>,
    /// Environment value.
    pub environment_value: Option<&'a str>,
    /// Physical path.
    pub physical_path: Option<&'a str>,
    /// Raw store error.
    pub store_error: Option<&'a str>,
    /// Unsupported durability or completion claim.
    pub unsupported_claim: Option<&'a str>,
}

/// Metadata-only active-turn recovery receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActiveTurnRecoveryReceipt {
    pub(super) schema: &'static str,
    pub(super) policy_hash: String,
    pub(super) turn_hash: String,
    pub(super) checkpoint_hash: String,
    pub(super) plan_hash: String,
    pub(super) generation: u64,
    pub(super) entry_count: u32,
    pub(super) action: ActiveTurnRecoveryAction,
    pub(super) terminal: ActiveTurnTerminalClass,
    pub(super) redaction_marker: &'static str,
    pub(super) non_claim: &'static str,
    pub(super) receipt_hash: String,
}

impl ActiveTurnRecoveryReceipt {
    /// Returns the receipt schema.
    #[must_use]
    pub const fn schema(&self) -> &'static str {
        self.schema
    }

    /// Returns the recovery action.
    #[must_use]
    pub const fn action(&self) -> ActiveTurnRecoveryAction {
        self.action
    }

    /// Returns the receipt identity.
    #[must_use]
    pub fn receipt_hash(&self) -> &str {
        self.receipt_hash.as_str()
    }

    /// Returns the bounded non-claim.
    #[must_use]
    pub const fn non_claim(&self) -> &'static str {
        self.non_claim
    }
}

/// Typed active-turn recovery issue.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActiveTurnRecoveryIssue {
    /// A label, hash, sequence, bound, or collection is invalid.
    InvalidInput,
    /// Generated policy is unsupported or unsafe.
    InvalidPolicy,
    /// Durable acceptance evidence is absent or mismatched.
    AcceptanceRequired,
    /// A supplied generation or identity is stale.
    StaleGeneration,
    /// Prepared checkpoint data is partial or overbound.
    IncompleteReplacement,
    /// Journal order or record shape is corrupt.
    CorruptJournal,
    /// Current authority or cancellation blocks continuation.
    ContinuationBlocked,
    /// Receipt input contains private material or an unsupported claim.
    ReceiptLeak,
}

/// Safe active-turn recovery error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActiveTurnRecoveryError {
    issue: ActiveTurnRecoveryIssue,
    field: &'static str,
}

impl ActiveTurnRecoveryError {
    pub(super) const fn new(issue: ActiveTurnRecoveryIssue, field: &'static str) -> Self {
        Self { issue, field }
    }

    /// Returns the typed issue.
    #[must_use]
    pub const fn issue(&self) -> ActiveTurnRecoveryIssue {
        self.issue
    }

    /// Returns the safe field label.
    #[must_use]
    pub const fn field(&self) -> &'static str {
        self.field
    }
}

impl core::fmt::Display for ActiveTurnRecoveryError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "active-turn recovery rejected {}", self.field)
    }
}

impl core::error::Error for ActiveTurnRecoveryError {}

/// Host-owned durable acceptance port.
pub trait ActiveTurnAcceptanceStore {
    /// Apply one atomic acceptance plan and return bounded transaction evidence.
    fn commit_acceptance(
        &mut self,
        plan: &ActiveTurnAcceptancePlan,
    ) -> ActiveTurnTransactionObservation;
}

/// Host-owned checkpoint transaction port.
pub trait ActiveTurnCheckpointStore {
    /// Apply one complete compare-and-swap checkpoint replacement.
    fn replace_checkpoint(
        &mut self,
        plan: &ActiveTurnReplacementPlan,
    ) -> ActiveTurnTransactionObservation;
}

/// Host-owned steering append port.
pub trait ActiveTurnSteeringStore {
    /// Append one durable steering identity.
    fn append_steering(
        &mut self,
        plan: &ActiveTurnSteeringPlan,
    ) -> ActiveTurnTransactionObservation;
}

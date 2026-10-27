//! Typed coding-session state, commands, effects, outcomes, and evidence.

#![allow(missing_docs)]

pub const CODING_SESSION_RECEIPT_SCHEMA: &str = "animus.coding-session-receipt.v1";
pub const CODING_SESSION_EFFECT_HASH_DOMAIN: &str = "animus.coding-session-effect.v1";
pub const CODING_SESSION_MESSAGE_HASH_DOMAIN: &str = "animus.coding-session-message.v1";
pub const CODING_SESSION_RECEIPT_HASH_DOMAIN: &str = "animus.coding-session-receipt.v1";
pub const CODING_SESSION_EMPTY_HEAD_HASH: &str =
    crate::durable_session_store::DURABLE_SESSION_EMPTY_HEAD_HASH;

pub(super) const MAX_SAFE_ID_BYTES: usize = 128;
pub(super) const MAX_MESSAGE_BYTES: usize = 8_192;
pub(super) const MAX_RECOVERY_MESSAGES: usize = 4_096;
pub(super) const MAX_EFFECT_STEPS_PER_COMMAND: u32 = 64;
pub(super) const MESSAGE_WIRE_VERSION: u8 = 1;
pub(super) const MESSAGE_KIND_USER_CODE: u8 = 1;
pub(super) const MESSAGE_KIND_ASSISTANT_CODE: u8 = 2;
pub(super) const MESSAGE_KIND_ASSISTANT_TOOL_CALL_CODE: u8 = 3;
pub(super) const MESSAGE_KIND_TOOL_RESULT_CODE: u8 = 4;
pub(super) const MESSAGE_KIND_SUMMARY_CODE: u8 = 5;
pub(super) const MESSAGE_KIND_REPAIR_CODE: u8 = 6;
pub(super) const MESSAGE_KIND_BRANCH_MOVE_CODE: u8 = 7;
pub(super) const MESSAGE_KIND_AUTHORITY_PROFILE_BINDING_CODE: u8 = 8;
pub(super) const MESSAGE_WIRE_PREFIX_BYTES: usize = 8;
pub(super) const HASH_SEPARATOR: &[u8] = b"\x1f";
pub(super) const REDACTION_MARKER: &str =
    "prompt-provider-tool-store-and-capability-bodies-omitted";
pub(super) const NON_CLAIMS: &[&str] = &[
    "fake-backed session evidence does not prove live-provider quality",
    "session evidence does not prove concrete product-tool behavior",
    "cancellation requests do not prove external effect rollback",
    "local durability evidence does not prove filesystem or power-loss correctness",
    "session evidence does not prove TUI dogfood, daemon attach, plugins, OAuth, production readiness, or release eligibility",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CodingSessionServicePosture {
    Available,
    Missing,
    Denied,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CodingSessionServiceSet {
    pub provider: CodingSessionServicePosture,
    pub tool: CodingSessionServicePosture,
    pub persistence: CodingSessionServicePosture,
    pub policy: CodingSessionServicePosture,
    pub cancellation: CodingSessionServicePosture,
    pub timer: CodingSessionServicePosture,
    pub summarizer: CodingSessionServicePosture,
}

impl CodingSessionServiceSet {
    #[must_use]
    pub const fn all_available() -> Self {
        Self {
            provider: CodingSessionServicePosture::Available,
            tool: CodingSessionServicePosture::Available,
            persistence: CodingSessionServicePosture::Available,
            policy: CodingSessionServicePosture::Available,
            cancellation: CodingSessionServicePosture::Available,
            timer: CodingSessionServicePosture::Available,
            summarizer: CodingSessionServicePosture::Available,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CodingSessionQueueProfile {
    Sequential,
    AllAtOnce,
}

impl CodingSessionQueueProfile {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Sequential => "one_at_a_time",
            Self::AllAtOnce => "all_at_once",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CodingSessionInputPosture {
    ExpandedAndAdmitted,
    UnexpandedWorkflowCommand,
    Denied,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CodingSessionInput {
    pub body: String,
    pub posture: CodingSessionInputPosture,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CodingSessionConfig {
    pub session_id: String,
    pub branch_id: String,
    pub queue_profile: CodingSessionQueueProfile,
    pub services: CodingSessionServiceSet,
    pub authority_profile: crate::session_authority_profile::SessionRuntimeAuthorityProfile,
    pub authority_authorization_identity: String,
    pub resume_required: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CodingSessionRunStatus {
    Active,
    Cancelling,
}

impl CodingSessionRunStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Cancelling => "cancelling",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CodingSessionRun {
    pub(super) run_id: String,
    pub(super) status: CodingSessionRunStatus,
}

impl CodingSessionRun {
    #[must_use]
    pub fn run_id(&self) -> &str {
        self.run_id.as_str()
    }

    #[must_use]
    pub const fn status(&self) -> CodingSessionRunStatus {
        self.status
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CodingSessionRecoveryPosture {
    NewStoreReady,
    ResumeRequired,
    Ready,
    RepairRequired,
    Blocked,
}

impl CodingSessionRecoveryPosture {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NewStoreReady => "new_store_ready",
            Self::ResumeRequired => "resume_required",
            Self::Ready => "ready",
            Self::RepairRequired => "repair_required",
            Self::Blocked => "blocked",
        }
    }

    #[must_use]
    pub const fn permits_dispatch(self) -> bool {
        matches!(self, Self::NewStoreReady | Self::Ready)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CodingSessionMessageKind {
    User,
    Assistant,
    AssistantToolCall,
    ToolResult,
    Summary,
    Repair,
    BranchMove,
    AuthorityProfileBinding,
}

impl CodingSessionMessageKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Assistant => "assistant",
            Self::AssistantToolCall => "assistant_tool_call",
            Self::ToolResult => "tool_result",
            Self::Summary => "summary",
            Self::Repair => "repair",
            Self::BranchMove => "branch_move",
            Self::AuthorityProfileBinding => "authority_profile_binding",
        }
    }

    #[must_use]
    pub const fn wire_code(self) -> u8 {
        match self {
            Self::User => MESSAGE_KIND_USER_CODE,
            Self::Assistant => MESSAGE_KIND_ASSISTANT_CODE,
            Self::AssistantToolCall => MESSAGE_KIND_ASSISTANT_TOOL_CALL_CODE,
            Self::ToolResult => MESSAGE_KIND_TOOL_RESULT_CODE,
            Self::Summary => MESSAGE_KIND_SUMMARY_CODE,
            Self::Repair => MESSAGE_KIND_REPAIR_CODE,
            Self::BranchMove => MESSAGE_KIND_BRANCH_MOVE_CODE,
            Self::AuthorityProfileBinding => MESSAGE_KIND_AUTHORITY_PROFILE_BINDING_CODE,
        }
    }

    #[must_use]
    pub const fn from_wire_code(value: u8) -> Option<Self> {
        match value {
            MESSAGE_KIND_USER_CODE => Some(Self::User),
            MESSAGE_KIND_ASSISTANT_CODE => Some(Self::Assistant),
            MESSAGE_KIND_ASSISTANT_TOOL_CALL_CODE => Some(Self::AssistantToolCall),
            MESSAGE_KIND_TOOL_RESULT_CODE => Some(Self::ToolResult),
            MESSAGE_KIND_SUMMARY_CODE => Some(Self::Summary),
            MESSAGE_KIND_REPAIR_CODE => Some(Self::Repair),
            MESSAGE_KIND_BRANCH_MOVE_CODE => Some(Self::BranchMove),
            MESSAGE_KIND_AUTHORITY_PROFILE_BINDING_CODE => Some(Self::AuthorityProfileBinding),
            _unsupported => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CodingSessionMessage {
    pub(super) entry_id: String,
    pub(super) parent_id: Option<String>,
    pub(super) run_id: String,
    pub(super) kind: CodingSessionMessageKind,
    pub(super) body: String,
    pub(super) tool_correlation_id: Option<String>,
    pub(super) tool_name: Option<String>,
    pub(super) message_hash: String,
}

impl CodingSessionMessage {
    #[must_use]
    pub fn entry_id(&self) -> &str {
        self.entry_id.as_str()
    }

    #[must_use]
    pub fn parent_id(&self) -> Option<&str> {
        self.parent_id.as_deref()
    }

    #[must_use]
    pub fn run_id(&self) -> &str {
        self.run_id.as_str()
    }

    #[must_use]
    pub const fn kind(&self) -> CodingSessionMessageKind {
        self.kind
    }

    #[must_use]
    pub fn body(&self) -> &str {
        self.body.as_str()
    }

    #[must_use]
    pub fn tool_correlation_id(&self) -> Option<&str> {
        self.tool_correlation_id.as_deref()
    }

    #[must_use]
    pub fn tool_name(&self) -> Option<&str> {
        self.tool_name.as_deref()
    }

    #[must_use]
    pub fn message_hash(&self) -> &str {
        self.message_hash.as_str()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CodingSessionQueueBehavior {
    Steering,
    FollowUp,
}

impl CodingSessionQueueBehavior {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Steering => "steering",
            Self::FollowUp => "follow_up",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct CodingSessionQueuedBody {
    pub(super) item_id: String,
    pub(super) entry_id: String,
    pub(super) behavior: CodingSessionQueueBehavior,
    pub(super) body: String,
    pub(super) body_hash: String,
}

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct CodingSessionQueueGeneration(u64);

impl CodingSessionQueueGeneration {
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn value(self) -> u64 {
        self.0
    }

    #[must_use]
    pub const fn checked_next(self) -> Option<Self> {
        match self.0.checked_add(1) {
            Some(value) => Some(Self(value)),
            None => None,
        }
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct CodingSessionQueueMutationPrecondition {
    entry_id: String,
    expected_generation: CodingSessionQueueGeneration,
    expected_position: u32,
    expected_text_identity: String,
}

impl CodingSessionQueueMutationPrecondition {
    #[must_use]
    pub fn new(
        entry_id: String,
        expected_generation: CodingSessionQueueGeneration,
        expected_position: u32,
        expected_text_identity: String,
    ) -> Self {
        Self {
            entry_id,
            expected_generation,
            expected_position,
            expected_text_identity,
        }
    }

    #[must_use]
    pub fn entry_id(&self) -> &str {
        self.entry_id.as_str()
    }

    #[must_use]
    pub const fn expected_generation(&self) -> CodingSessionQueueGeneration {
        self.expected_generation
    }

    #[must_use]
    pub const fn expected_position(&self) -> u32 {
        self.expected_position
    }

    #[must_use]
    pub fn expected_text_identity(&self) -> &str {
        self.expected_text_identity.as_str()
    }
}

impl core::fmt::Debug for CodingSessionQueueMutationPrecondition {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("CodingSessionQueueMutationPrecondition")
            .field("entry_id", &self.entry_id)
            .field("expected_generation", &self.expected_generation)
            .field("expected_position", &self.expected_position)
            .field("expected_text_identity", &"<private>")
            .finish()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CodingSessionQueueSafeEntry {
    pub item_id: String,
    pub entry_id: String,
    pub position: u32,
    pub behavior: CodingSessionQueueBehavior,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CodingSessionQueueSafeSnapshot {
    pub generation: CodingSessionQueueGeneration,
    pub queue_identity: String,
    pub entries: Vec<CodingSessionQueueSafeEntry>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CodingSessionQueueMutationOperation {
    Edit,
    Remove,
}

impl CodingSessionQueueMutationOperation {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Edit => "edit",
            Self::Remove => "remove",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CodingSessionQueueConflictKind {
    MissingItem,
    WrongEntry,
    MovedPosition,
    StaleGeneration,
    StaleText,
}

impl CodingSessionQueueConflictKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MissingItem => "missing_item",
            Self::WrongEntry => "wrong_entry",
            Self::MovedPosition => "moved_position",
            Self::StaleGeneration => "stale_generation",
            Self::StaleText => "stale_text",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CodingSessionQueueConflict {
    pub operation: CodingSessionQueueMutationOperation,
    pub item_id: String,
    pub entry_id: String,
    pub kind: CodingSessionQueueConflictKind,
    pub snapshot: CodingSessionQueueSafeSnapshot,
    pub redaction_marker: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CodingSessionQueueMutationEvent {
    pub operation: CodingSessionQueueMutationOperation,
    pub status: &'static str,
    pub item_id: String,
    pub entry_id: String,
    pub position: Option<u32>,
    pub generation: CodingSessionQueueGeneration,
    pub queue_identity: String,
    pub conflict: Option<CodingSessionQueueConflictKind>,
    pub redaction_marker: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CodingSessionQueueMutationReceipt {
    pub schema: &'static str,
    pub operation: CodingSessionQueueMutationOperation,
    pub status: &'static str,
    pub item_id: String,
    pub entry_id: String,
    pub position: Option<u32>,
    pub generation_before: CodingSessionQueueGeneration,
    pub generation_after: Option<CodingSessionQueueGeneration>,
    pub queue_identity: String,
    pub conflict: Option<CodingSessionQueueConflictKind>,
    pub redaction_marker: &'static str,
    pub receipt_hash: String,
}

#[derive(Clone, Copy, Debug)]
pub struct CodingSessionQueueAcceptedMutation<'a> {
    pub operation: CodingSessionQueueMutationOperation,
    pub item_id: &'a str,
    pub precondition: &'a CodingSessionQueueMutationPrecondition,
}

#[derive(Clone, Copy, Default, Eq, PartialEq)]
pub struct CodingSessionQueueSensitiveDraft<'a> {
    pub current: Option<&'a str>,
    pub expected: Option<&'a str>,
    pub replacement: Option<&'a str>,
}

impl core::fmt::Debug for CodingSessionQueueSensitiveDraft<'_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("CodingSessionQueueSensitiveDraft")
            .field("current", &self.current.map(|_value| "<private>"))
            .field("expected", &self.expected.map(|_value| "<private>"))
            .field("replacement", &self.replacement.map(|_value| "<private>"))
            .finish()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CodingSessionQueueReceiptError {
    UnsafePrivateText,
    InvalidIdentity,
}

impl core::fmt::Display for CodingSessionQueueReceiptError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::UnsafePrivateText => formatter.write_str("queue receipt rejected private text"),
            Self::InvalidIdentity => formatter.write_str("queue receipt rejected identity"),
        }
    }
}

impl std::error::Error for CodingSessionQueueReceiptError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CodingSessionTerminalKind {
    Completed,
    Cancelled,
    Blocked,
}

impl CodingSessionTerminalKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
            Self::Blocked => "blocked",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CodingSessionCommand {
    Prompt {
        run_id: String,
        entry_id: String,
        input: CodingSessionInput,
    },
    Continue {
        run_id: String,
    },
    Steer {
        item_id: String,
        entry_id: String,
        input: CodingSessionInput,
    },
    FollowUp {
        item_id: String,
        entry_id: String,
        input: CodingSessionInput,
    },
    EditQueued {
        item_id: String,
        precondition: CodingSessionQueueMutationPrecondition,
        input: CodingSessionInput,
    },
    RemoveQueued {
        item_id: String,
        precondition: CodingSessionQueueMutationPrecondition,
    },
    Cancel {
        run_id: String,
    },
    Resume,
    Compact {
        entry_id: String,
    },
    RebindAuthority {
        entry_id: String,
        profile: crate::session_authority_profile::SessionRuntimeAuthorityProfile,
        authorization_identity: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CodingSessionAfterCommit {
    DispatchProvider,
    DispatchTool {
        assistant_response: animus_adapters::ModelResponse,
        tool_request: animus_adapters::ToolRequest,
    },
    ContinueProvider {
        tool_response: animus_adapters::ToolResponse,
    },
    Complete {
        response: animus_adapters::ModelResponse,
    },
    InjectNextOrProvider,
    RecoveryReady,
    CompactionReady,
    AuthorityRebound,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CodingSessionCommitRequest {
    pub(super) effect_id: String,
    pub(super) run_id: Option<String>,
    pub(super) session_id: String,
    pub(super) branch_id: String,
    pub(super) prior_head_hash: String,
    pub(super) prior_active_leaf_id: Option<String>,
    pub(super) messages: Vec<CodingSessionMessage>,
    pub(super) queued_item_id: Option<String>,
    pub(super) after_commit: CodingSessionAfterCommit,
}

impl CodingSessionCommitRequest {
    #[must_use]
    pub fn effect_id(&self) -> &str {
        self.effect_id.as_str()
    }

    #[must_use]
    pub fn run_id(&self) -> Option<&str> {
        self.run_id.as_deref()
    }

    #[must_use]
    pub fn session_id(&self) -> &str {
        self.session_id.as_str()
    }

    #[must_use]
    pub fn branch_id(&self) -> &str {
        self.branch_id.as_str()
    }

    #[must_use]
    pub fn prior_head_hash(&self) -> &str {
        self.prior_head_hash.as_str()
    }

    #[must_use]
    pub fn prior_active_leaf_id(&self) -> Option<&str> {
        self.prior_active_leaf_id.as_deref()
    }

    #[must_use]
    pub fn messages(&self) -> &[CodingSessionMessage] {
        self.messages.as_slice()
    }

    #[must_use]
    pub fn proposed_active_leaf_id(&self) -> &str {
        self.messages
            .last()
            .map_or("", CodingSessionMessage::entry_id)
    }

    /// Returns the exact tool response whose commit precedes provider continuation.
    #[must_use]
    pub const fn tool_response_after_commit(&self) -> Option<&animus_adapters::ToolResponse> {
        match &self.after_commit {
            CodingSessionAfterCommit::ContinueProvider { tool_response } => Some(tool_response),
            CodingSessionAfterCommit::DispatchProvider
            | CodingSessionAfterCommit::DispatchTool { .. }
            | CodingSessionAfterCommit::Complete { .. }
            | CodingSessionAfterCommit::InjectNextOrProvider
            | CodingSessionAfterCommit::RecoveryReady
            | CodingSessionAfterCommit::CompactionReady
            | CodingSessionAfterCommit::AuthorityRebound => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CodingSessionProviderRequest {
    pub(super) effect_id: String,
    pub(super) run_id: String,
    pub(super) session_id: String,
    pub(super) branch_id: String,
    pub(super) active_leaf_id: String,
    pub(super) durable_head_hash: String,
    pub(super) request_sequence: u64,
    pub(super) request: animus_adapters::ModelRequest,
}

impl CodingSessionProviderRequest {
    #[must_use]
    pub fn effect_id(&self) -> &str {
        self.effect_id.as_str()
    }

    #[must_use]
    pub fn run_id(&self) -> &str {
        self.run_id.as_str()
    }

    #[must_use]
    pub fn session_id(&self) -> &str {
        self.session_id.as_str()
    }

    #[must_use]
    pub fn branch_id(&self) -> &str {
        self.branch_id.as_str()
    }

    #[must_use]
    pub fn active_leaf_id(&self) -> &str {
        self.active_leaf_id.as_str()
    }

    #[must_use]
    pub fn durable_head_hash(&self) -> &str {
        self.durable_head_hash.as_str()
    }

    #[must_use]
    pub const fn request_sequence(&self) -> u64 {
        self.request_sequence
    }

    #[must_use]
    pub const fn request(&self) -> &animus_adapters::ModelRequest {
        &self.request
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CodingSessionToolRequest {
    pub(super) effect_id: String,
    pub(super) run_id: String,
    pub(super) request: animus_adapters::ToolRequest,
}

impl CodingSessionToolRequest {
    #[must_use]
    pub fn effect_id(&self) -> &str {
        self.effect_id.as_str()
    }

    #[must_use]
    pub fn run_id(&self) -> &str {
        self.run_id.as_str()
    }

    #[must_use]
    pub const fn request(&self) -> &animus_adapters::ToolRequest {
        &self.request
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CodingSessionCancellationRequest {
    pub(super) effect_id: String,
    pub(super) run_id: String,
}

impl CodingSessionCancellationRequest {
    #[must_use]
    pub fn effect_id(&self) -> &str {
        self.effect_id.as_str()
    }

    #[must_use]
    pub fn run_id(&self) -> &str {
        self.run_id.as_str()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CodingSessionRecoveryRequest {
    pub(super) effect: String,
    pub(super) session: String,
    pub(super) branch: String,
}

impl CodingSessionRecoveryRequest {
    #[must_use]
    pub fn effect_id(&self) -> &str {
        self.effect.as_str()
    }

    #[must_use]
    pub fn session_id(&self) -> &str {
        self.session.as_str()
    }

    #[must_use]
    pub fn branch_id(&self) -> &str {
        self.branch.as_str()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CodingSessionSummarizerRequest {
    pub(super) effect_id: String,
    pub(super) run_id: String,
    pub(super) entry_id: String,
    pub(super) messages: Vec<CodingSessionMessage>,
}

impl CodingSessionSummarizerRequest {
    #[must_use]
    pub fn effect_id(&self) -> &str {
        self.effect_id.as_str()
    }

    #[must_use]
    pub fn run_id(&self) -> &str {
        self.run_id.as_str()
    }

    #[must_use]
    pub fn entry_id(&self) -> &str {
        self.entry_id.as_str()
    }

    #[must_use]
    pub fn messages(&self) -> &[CodingSessionMessage] {
        self.messages.as_slice()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CodingSessionEffectRequest {
    Commit(CodingSessionCommitRequest),
    Provider(CodingSessionProviderRequest),
    Tool(CodingSessionToolRequest),
    Cancellation(CodingSessionCancellationRequest),
    Recovery(CodingSessionRecoveryRequest),
    Summarizer(CodingSessionSummarizerRequest),
}

impl CodingSessionEffectRequest {
    #[must_use]
    pub fn effect_id(&self) -> &str {
        match self {
            Self::Commit(request) => request.effect_id(),
            Self::Provider(request) => request.effect_id(),
            Self::Tool(request) => request.effect_id(),
            Self::Cancellation(request) => request.effect_id(),
            Self::Recovery(request) => request.effect_id(),
            Self::Summarizer(request) => request.effect_id(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CodingSessionProviderReply {
    Final(animus_adapters::ModelResponse),
    ToolCall {
        assistant_response: animus_adapters::ModelResponse,
        tool_request: animus_adapters::ToolRequest,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CodingSessionRecoveryStatus {
    Clean,
    Empty,
    IncompleteTail,
    Corrupt,
}

impl CodingSessionRecoveryStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Clean => "clean",
            Self::Empty => "empty",
            Self::IncompleteTail => "incomplete_tail",
            Self::Corrupt => "corrupt",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CodingSessionRecoverySnapshot {
    pub status: CodingSessionRecoveryStatus,
    pub committed_head_hash: String,
    pub active_leaf_id: Option<String>,
    pub durable_cursor: u64,
    pub messages: Vec<CodingSessionMessage>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CodingSessionEffectService {
    Persistence,
    Provider,
    Tool,
    Cancellation,
    Timer,
    Policy,
    Authority,
    Summarizer,
    Recovery,
}

impl CodingSessionEffectService {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Persistence => "persistence",
            Self::Provider => "provider",
            Self::Tool => "tool",
            Self::Cancellation => "cancellation",
            Self::Timer => "timer",
            Self::Policy => "policy",
            Self::Authority => "authority",
            Self::Summarizer => "summarizer",
            Self::Recovery => "recovery",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CodingSessionServiceIssue {
    Unavailable,
    Denied,
    Failed,
    TimedOut,
    Cancelled,
    Stale,
    Malformed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CodingSessionEffectOutcome {
    CommitSucceeded {
        effect_id: String,
        new_head_hash: String,
        active_leaf_id: String,
        durable_cursor: u64,
        synchronized: bool,
    },
    ProviderSucceeded {
        effect_id: String,
        reply: CodingSessionProviderReply,
    },
    ToolSucceeded {
        effect_id: String,
        response: animus_adapters::ToolResponse,
    },
    CancellationObserved {
        effect_id: String,
        run_id: String,
        observed: bool,
    },
    RecoverySucceeded {
        effect_id: String,
        snapshot: CodingSessionRecoverySnapshot,
    },
    SummarizerSucceeded {
        effect_id: String,
        entry_id: String,
        summary: String,
    },
    Failed {
        effect_id: String,
        service: CodingSessionEffectService,
        issue: CodingSessionServiceIssue,
    },
}

impl CodingSessionEffectOutcome {
    #[must_use]
    pub fn effect_id(&self) -> &str {
        match self {
            Self::CommitSucceeded { effect_id, .. }
            | Self::ProviderSucceeded { effect_id, .. }
            | Self::ToolSucceeded { effect_id, .. }
            | Self::CancellationObserved { effect_id, .. }
            | Self::RecoverySucceeded { effect_id, .. }
            | Self::SummarizerSucceeded { effect_id, .. }
            | Self::Failed { effect_id, .. } => effect_id.as_str(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CodingSessionIssue {
    InvalidInput,
    InputNotAdmitted,
    UnavailableService,
    AlreadyRunning,
    NotRunning,
    StaleRun,
    BusyInputRequiresQueue,
    InvalidState,
    QueueRejected,
    QueueConflict,
    StaleEffect,
    PersistenceFailure,
    ProviderFailure,
    ToolFailure,
    CancellationFailure,
    SummarizerFailure,
    RecoveryBlocked,
    AuthorityProfileUnbound,
    AuthorityProfileMismatch,
    AuthorityReauthorizationRequired,
    AuthorityEnforcementUnavailable,
    BoundExceeded,
    UnsafeReceipt,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CodingSessionError {
    issue: CodingSessionIssue,
    field: &'static str,
    queue_conflict: Option<Box<CodingSessionQueueConflict>>,
}

impl CodingSessionError {
    pub(super) const fn new(issue: CodingSessionIssue, field: &'static str) -> Self {
        Self {
            issue,
            field,
            queue_conflict: None,
        }
    }

    pub(super) fn queue_conflict(conflict: CodingSessionQueueConflict) -> Self {
        Self {
            issue: CodingSessionIssue::QueueConflict,
            field: "queue",
            queue_conflict: Some(Box::new(conflict)),
        }
    }

    #[must_use]
    pub const fn issue(&self) -> CodingSessionIssue {
        self.issue
    }

    #[must_use]
    pub const fn field(&self) -> &'static str {
        self.field
    }

    #[must_use]
    pub fn queue_conflict_snapshot(&self) -> Option<&CodingSessionQueueConflict> {
        self.queue_conflict.as_deref()
    }
}

impl core::fmt::Display for CodingSessionError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "coding session rejected {}", self.field)
    }
}

impl std::error::Error for CodingSessionError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CodingSessionState {
    pub(super) config: CodingSessionConfig,
    pub(super) queue_policy: crate::mailbox::MessageQueuePolicy,
    pub(super) run: Option<CodingSessionRun>,
    pub(super) queue: Option<crate::mailbox::MessageQueueState>,
    pub(super) queued_bodies: Vec<CodingSessionQueuedBody>,
    pub(super) queue_generation: CodingSessionQueueGeneration,
    pub(super) messages: Vec<CodingSessionMessage>,
    pub(super) active_leaf_id: Option<String>,
    pub(super) durable_head_hash: String,
    pub(super) durable_cursor: u64,
    pub(super) committed_authority_profile_identity: Option<String>,
    pub(super) authority_binding_identity: Option<String>,
    pub(super) pending_effect: Option<CodingSessionEffectRequest>,
    pub(super) recovery: CodingSessionRecoveryPosture,
    pub(super) terminal: Option<CodingSessionTerminalKind>,
    pub(super) last_issue: Option<CodingSessionIssue>,
    pub(super) commit_count: u32,
    pub(super) provider_dispatch_count: u32,
    pub(super) tool_dispatch_count: u32,
    pub(super) emitted_event_count: u32,
}

impl CodingSessionState {
    #[must_use]
    pub fn session_id(&self) -> &str {
        self.config.session_id.as_str()
    }

    #[must_use]
    pub fn branch_id(&self) -> &str {
        self.config.branch_id.as_str()
    }

    #[must_use]
    pub const fn run(&self) -> Option<&CodingSessionRun> {
        self.run.as_ref()
    }

    #[must_use]
    pub const fn pending_effect(&self) -> Option<&CodingSessionEffectRequest> {
        self.pending_effect.as_ref()
    }

    #[must_use]
    pub fn messages(&self) -> &[CodingSessionMessage] {
        self.messages.as_slice()
    }

    #[must_use]
    pub fn active_leaf_id(&self) -> Option<&str> {
        self.active_leaf_id.as_deref()
    }

    #[must_use]
    pub fn durable_head_hash(&self) -> &str {
        self.durable_head_hash.as_str()
    }

    #[must_use]
    pub const fn durable_cursor(&self) -> u64 {
        self.durable_cursor
    }

    #[must_use]
    pub const fn authority_profile(
        &self,
    ) -> &crate::session_authority_profile::SessionRuntimeAuthorityProfile {
        &self.config.authority_profile
    }

    #[must_use]
    pub fn authority_authorization_identity(&self) -> &str {
        self.config.authority_authorization_identity.as_str()
    }

    #[must_use]
    pub fn committed_authority_profile_identity(&self) -> Option<&str> {
        self.committed_authority_profile_identity.as_deref()
    }

    #[must_use]
    pub fn authority_binding_identity(&self) -> Option<&str> {
        self.authority_binding_identity.as_deref()
    }

    #[must_use]
    pub const fn recovery_posture(&self) -> CodingSessionRecoveryPosture {
        self.recovery
    }

    #[must_use]
    pub const fn terminal(&self) -> Option<CodingSessionTerminalKind> {
        self.terminal
    }

    #[must_use]
    pub const fn last_issue(&self) -> Option<CodingSessionIssue> {
        self.last_issue
    }

    #[must_use]
    pub fn queued_count(&self) -> Option<u32> {
        u32::try_from(self.queued_bodies.len()).ok()
    }

    #[must_use]
    pub const fn queue_generation(&self) -> CodingSessionQueueGeneration {
        self.queue_generation
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CodingSessionTransition {
    pub state: CodingSessionState,
    pub events: Vec<crate::telemetry::RuntimeEvent>,
    pub effect: Option<CodingSessionEffectRequest>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CodingSessionFrontendState {
    pub lifecycle_status: String,
    pub committed_message_count: u32,
    pub queue_count: u32,
    pub queue_generation: CodingSessionQueueGeneration,
    pub queue_identity: Option<String>,
    pub queue_mutation_status: Option<String>,
    pub queue_conflict: Option<CodingSessionQueueConflictKind>,
    pub active_tool_correlation: Option<String>,
    pub cancellation_observed: bool,
    pub terminal_status: Option<String>,
}

impl Default for CodingSessionFrontendState {
    fn default() -> Self {
        Self {
            lifecycle_status: String::from("idle"),
            committed_message_count: 0,
            queue_count: 0,
            queue_generation: CodingSessionQueueGeneration::default(),
            queue_identity: None,
            queue_mutation_status: None,
            queue_conflict: None,
            active_tool_correlation: None,
            cancellation_observed: false,
            terminal_status: None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CodingSessionReceipt {
    pub schema: &'static str,
    pub session_id: String,
    pub branch_id: String,
    pub terminal_status: CodingSessionTerminalKind,
    pub committed_message_count: u32,
    pub commit_count: u32,
    pub provider_dispatch_count: u32,
    pub tool_dispatch_count: u32,
    pub pending_queue_count: u32,
    pub durable_cursor: u64,
    pub committed_head_hash: String,
    pub authority_profile_identity: String,
    pub authority_binding_identity: String,
    pub redaction_marker: &'static str,
    pub non_claims: Vec<String>,
    pub receipt_hash: String,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CodingSessionSensitiveDraft<'a> {
    pub raw_prompt: Option<&'a str>,
    pub provider_payload: Option<&'a str>,
    pub tool_output: Option<&'a str>,
    pub credential: Option<&'a str>,
    pub capability_path: Option<&'a str>,
    pub store_path: Option<&'a str>,
    pub raw_error: Option<&'a str>,
}

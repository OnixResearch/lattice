//! Typed durable-session framing, policy, recovery, and receipt values.

/// Generated policy schema.
pub const DURABLE_SESSION_STORE_POLICY_SCHEMA: &str = "animus.durable-session-store-policy.v1";
/// BLAKE3 domain for generated policy content.
pub const DURABLE_SESSION_STORE_POLICY_HASH_DOMAIN: &str =
    "animus.durable-session-store-policy-content.v1";
/// BLAKE3 domain for private frame identities.
pub const DURABLE_SESSION_FRAME_HASH_DOMAIN: &str = "animus.durable-session-frame.v1";
/// BLAKE3 domain for transaction identities.
pub const DURABLE_SESSION_TRANSACTION_HASH_DOMAIN: &str = "animus.durable-session-transaction.v1";
/// BLAKE3 domain for commit identities.
pub const DURABLE_SESSION_COMMIT_HASH_DOMAIN: &str = "animus.durable-session-commit.v1";
/// BLAKE3 domain for safe sidecar identities.
pub const DURABLE_SESSION_SIDECAR_HASH_DOMAIN: &str = "animus.durable-session-sidecar.v1";
/// BLAKE3 domain for metadata-only receipts.
pub const DURABLE_SESSION_RECEIPT_HASH_DOMAIN: &str = "animus.durable-session-receipt.v1";
/// Receipt schema.
pub const DURABLE_SESSION_RECEIPT_SCHEMA: &str = "animus.durable-session-receipt.v1";
/// Empty-store committed head identity.
pub const DURABLE_SESSION_EMPTY_HEAD_HASH: &str =
    "0000000000000000000000000000000000000000000000000000000000000000";

pub(super) const BLAKE3_HEX_CHARS: usize = 64;
pub(super) const HASH_SEPARATOR: &[u8] = b"\x1f";
pub(super) const MAX_FRAME_PAYLOAD_BYTES: u32 = 1_048_576;
pub(super) const MAX_TRANSACTION_FRAMES: u32 = 256;
pub(super) const MAX_FILE_BYTES: u64 = 1_073_741_824;
pub(super) const MIN_COUNT: u32 = 1;
pub(super) const PRIVATE_FILE_MODE: u32 = 0o600;
pub(super) const SUPPORTED_FRAME_SCHEMA: u16 = 1;
pub(super) const REDACTION_MARKER: &str =
    "private-session-payloads-omitted-from-sidecars-receipts-diagnostics-and-default-exports";
pub(super) const NON_CLAIM: &str = "local commit evidence reports supplied framing and synchronization outcomes, not encryption, authentication, filesystem correctness, power-loss proof, backup, replication, or production recovery";

/// Host-loaded generated policy input.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DurableSessionStorePolicyInput {
    /// Policy schema.
    pub schema: String,
    /// Policy label.
    pub policy_id: String,
    /// Maximum private payload bytes in one frame.
    pub max_frame_payload_bytes: u32,
    /// Maximum private frames in one transaction.
    pub max_transaction_frames: u32,
    /// Maximum journal bytes admitted for reads and appends.
    pub max_file_bytes: u64,
    /// Required private file mode.
    pub file_mode: u32,
    /// Durability label.
    pub durability: String,
    /// Lock policy label.
    pub locking: String,
    /// Private-data posture label.
    pub private_data: String,
    /// Supported encryption posture labels.
    pub encryption_postures: Vec<String>,
    /// Tail recovery label.
    pub tail_recovery: String,
    /// Committed corruption label.
    pub committed_corruption: String,
    /// Exact receipt field classes.
    pub receipt_fields: Vec<String>,
    /// Canonical policy identity.
    pub policy_hash: String,
}

/// Validated generated durable store policy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DurableSessionStorePolicy {
    pub(super) policy_id: String,
    pub(super) max_frame_payload_bytes: u32,
    pub(super) max_transaction_frames: u32,
    pub(super) max_file_bytes: u64,
    pub(super) file_mode: u32,
    pub(super) encryption_postures: Vec<DurableSessionEncryptionPosture>,
    pub(super) receipt_fields: Vec<String>,
    pub(super) policy_hash: String,
}

impl DurableSessionStorePolicy {
    /// Returns the policy label.
    #[must_use]
    pub fn policy_id(&self) -> &str {
        self.policy_id.as_str()
    }

    /// Returns the canonical policy identity.
    #[must_use]
    pub fn policy_hash(&self) -> &str {
        self.policy_hash.as_str()
    }

    /// Returns the private payload bound.
    #[must_use]
    pub const fn max_frame_payload_bytes(&self) -> u32 {
        self.max_frame_payload_bytes
    }

    /// Returns the transaction frame bound.
    #[must_use]
    pub const fn max_transaction_frames(&self) -> u32 {
        self.max_transaction_frames
    }

    /// Returns the journal byte bound.
    #[must_use]
    pub const fn max_file_bytes(&self) -> u64 {
        self.max_file_bytes
    }

    /// Returns the required private file mode.
    #[must_use]
    pub const fn file_mode(&self) -> u32 {
        self.file_mode
    }

    /// Returns whether an encryption posture is admitted.
    #[must_use]
    pub fn admits_encryption(&self, posture: DurableSessionEncryptionPosture) -> bool {
        self.encryption_postures.contains(&posture)
    }
}

/// Declared synchronization policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DurableSessionDurability {
    /// Synchronize file data and metadata after every committed group.
    SyncAllEachCommit,
}

impl DurableSessionDurability {
    /// Returns the stable durability claim label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SyncAllEachCommit => "sync_all_each_commit",
        }
    }
}

/// Declared local writer lock policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DurableSessionLockPolicy {
    /// Acquire a capability-local lock file with create-new semantics.
    ExclusiveCreateNew,
}

/// Declared private replay-content posture.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DurableSessionPrivacyPosture {
    /// Journal payloads are private replay material.
    ReplayPrivate,
}

/// Declared encryption posture.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DurableSessionEncryptionPosture {
    /// Unencrypted storage is explicitly accepted by the caller.
    AbsentExplicit,
    /// An externally supplied encryption service is required.
    ExternalRequired,
}

impl DurableSessionEncryptionPosture {
    /// Parses a generated-policy label.
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "absent_explicit" => Some(Self::AbsentExplicit),
            "external_required" => Some(Self::ExternalRequired),
            _other => None,
        }
    }

    /// Returns the stable label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::AbsentExplicit => "absent_explicit",
            Self::ExternalRequired => "external_required",
        }
    }
}

/// Explicit store admission descriptor without ambient paths.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DurableSessionStoreDescriptor {
    /// Safe store identity label.
    pub store_id: String,
    /// Safe capability-location label, never a path.
    pub location_capability: String,
    /// Session identity.
    pub session_id: String,
    /// Branch identity.
    pub branch_id: String,
    /// Generated policy identity.
    pub policy_hash: String,
    /// Durability policy.
    pub durability: DurableSessionDurability,
    /// Lock policy.
    pub locking: DurableSessionLockPolicy,
    /// Private-data posture.
    pub private_data: DurableSessionPrivacyPosture,
    /// Encryption posture.
    pub encryption: DurableSessionEncryptionPosture,
    /// Whether an external encryption service was explicitly admitted.
    pub encryption_service_available: bool,
    /// Safe receipt policy label.
    pub receipt_policy: String,
}

const MODEL_REQUEST_SNAPSHOT_WIRE_CODE: u8 = 9;
const MODEL_REQUEST_TERMINAL_WIRE_CODE: u8 = 10;

/// Private replay record kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DurableSessionRecordKind {
    /// User model-visible message.
    User,
    /// Assistant model-visible message.
    Assistant,
    /// Tool result body.
    ToolResult,
    /// Compaction summary body.
    Summary,
    /// Interrupted-tool repair result.
    Repair,
    /// Branch navigation marker.
    BranchMove,
    /// Metadata-only session authority-profile binding.
    AuthorityProfileBinding,
    /// Private committed Steel kernel safe-point snapshot.
    SteelKernelState,
    /// Private complete provider-neutral model-request snapshot.
    ModelRequestSnapshot,
    /// Private correlated terminal provider observation.
    ModelRequestTerminal,
}

impl DurableSessionRecordKind {
    /// Returns the stable wire code.
    #[must_use]
    pub(super) const fn wire_code(self) -> u8 {
        match self {
            Self::User => 1,
            Self::Assistant => 2,
            Self::ToolResult => 3,
            Self::Summary => 4,
            Self::Repair => 5,
            Self::BranchMove => 6,
            Self::AuthorityProfileBinding => 7,
            Self::SteelKernelState => 8,
            Self::ModelRequestSnapshot => MODEL_REQUEST_SNAPSHOT_WIRE_CODE,
            Self::ModelRequestTerminal => MODEL_REQUEST_TERMINAL_WIRE_CODE,
        }
    }

    /// Parses a stable wire code.
    #[must_use]
    pub(super) const fn from_wire_code(value: u8) -> Option<Self> {
        match value {
            1 => Some(Self::User),
            2 => Some(Self::Assistant),
            3 => Some(Self::ToolResult),
            4 => Some(Self::Summary),
            5 => Some(Self::Repair),
            6 => Some(Self::BranchMove),
            7 => Some(Self::AuthorityProfileBinding),
            8 => Some(Self::SteelKernelState),
            MODEL_REQUEST_SNAPSHOT_WIRE_CODE => Some(Self::ModelRequestSnapshot),
            MODEL_REQUEST_TERMINAL_WIRE_CODE => Some(Self::ModelRequestTerminal),
            _other => None,
        }
    }

    /// Returns the safe label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Assistant => "assistant",
            Self::ToolResult => "tool_result",
            Self::Summary => "summary",
            Self::Repair => "repair",
            Self::BranchMove => "branch_move",
            Self::AuthorityProfileBinding => "authority_profile_binding",
            Self::SteelKernelState => "steel_kernel_state",
            Self::ModelRequestSnapshot => "model_request_snapshot",
            Self::ModelRequestTerminal => "model_request_terminal",
        }
    }
}

/// One caller-supplied private transaction entry.
#[derive(Clone, Copy)]
pub struct DurableSessionPrivateEntryDraft<'a> {
    /// Stable entry identity within the committed branch lineage.
    pub entry_id: &'a str,
    /// Optional parent entry identity.
    pub parent_id: Option<&'a str>,
    /// Private record kind.
    pub kind: DurableSessionRecordKind,
    /// Bounded private replay payload.
    pub payload: &'a [u8],
}

/// Transaction planning input.
#[derive(Clone, Copy)]
pub struct DurableSessionTransactionDraft<'a> {
    /// Validated store descriptor.
    pub descriptor: &'a DurableSessionStoreDescriptor,
    /// Prior committed head identity, or the zero hash for a new store.
    pub prior_head_hash: &'a str,
    /// Prior committed active leaf, absent only for a new branch journal.
    pub prior_active_leaf_id: Option<&'a str>,
    /// First logical sequence in the group.
    pub first_sequence: u64,
    /// Proposed active leaf identity after commit.
    pub active_leaf_id: &'a str,
    /// Ordered private entries.
    pub entries: &'a [DurableSessionPrivateEntryDraft<'a>],
}

/// Validated private frame.
#[derive(Clone, Eq, PartialEq)]
pub struct DurableSessionFrame {
    pub(super) schema_version: u16,
    pub(super) sequence: u64,
    pub(super) branch_id: String,
    pub(super) entry_id: String,
    pub(super) parent_id: Option<String>,
    pub(super) kind: DurableSessionRecordKind,
    pub(super) payload: Vec<u8>,
    pub(super) payload_hash: String,
    pub(super) previous_frame_hash: String,
    pub(super) frame_hash: String,
}

impl core::fmt::Debug for DurableSessionFrame {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("DurableSessionFrame")
            .field("schema_version", &self.schema_version)
            .field("sequence", &self.sequence)
            .field("branch_id", &self.branch_id)
            .field("entry_id", &self.entry_id)
            .field("parent_id", &self.parent_id)
            .field("kind", &self.kind)
            .field("payload_bytes", &self.payload.len())
            .field("payload_hash", &self.payload_hash)
            .field("previous_frame_hash", &self.previous_frame_hash)
            .field("frame_hash", &self.frame_hash)
            .finish()
    }
}

impl DurableSessionFrame {
    /// Returns logical sequence.
    #[must_use]
    pub const fn sequence(&self) -> u64 {
        self.sequence
    }

    /// Returns the committed entry identity.
    #[must_use]
    pub fn entry_id(&self) -> &str {
        self.entry_id.as_str()
    }

    /// Returns the optional parent entry identity.
    #[must_use]
    pub fn parent_id(&self) -> Option<&str> {
        self.parent_id.as_deref()
    }

    /// Returns the record kind.
    #[must_use]
    pub const fn kind(&self) -> DurableSessionRecordKind {
        self.kind
    }

    /// Returns private replay bytes to the admitted host only.
    #[must_use]
    pub fn private_payload(&self) -> &[u8] {
        self.payload.as_slice()
    }

    /// Returns the prior durable frame or commit identity.
    #[must_use]
    pub fn previous_frame_hash(&self) -> &str {
        self.previous_frame_hash.as_str()
    }

    /// Returns frame identity.
    #[must_use]
    pub fn frame_hash(&self) -> &str {
        self.frame_hash.as_str()
    }

    /// Returns payload identity.
    #[must_use]
    pub fn payload_hash(&self) -> &str {
        self.payload_hash.as_str()
    }
}

/// Metadata-only sidecar for one private frame.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DurableSessionSafeSidecar {
    /// Logical sequence.
    pub sequence: u64,
    /// Safe entry identity.
    pub entry_id: String,
    /// Safe kind.
    pub kind: DurableSessionRecordKind,
    /// Bounded payload byte count.
    pub payload_bytes: u32,
    /// Private payload identity.
    pub payload_hash: String,
    /// Private frame identity.
    pub frame_hash: String,
    /// Sidecar identity.
    pub sidecar_hash: String,
    /// Redaction marker.
    pub redaction_marker: String,
}

/// Validated logical commit.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DurableSessionCommit {
    pub(super) transaction_hash: String,
    pub(super) prior_head_hash: String,
    pub(super) active_leaf_id: String,
    pub(super) frame_hashes: Vec<String>,
    pub(super) commit_hash: String,
}

impl DurableSessionCommit {
    /// Returns transaction identity.
    #[must_use]
    pub fn transaction_hash(&self) -> &str {
        self.transaction_hash.as_str()
    }

    /// Returns prior committed head identity.
    #[must_use]
    pub fn prior_head_hash(&self) -> &str {
        self.prior_head_hash.as_str()
    }

    /// Returns committed head identity.
    #[must_use]
    pub fn commit_hash(&self) -> &str {
        self.commit_hash.as_str()
    }

    /// Returns proposed active leaf.
    #[must_use]
    pub fn active_leaf_id(&self) -> &str {
        self.active_leaf_id.as_str()
    }
}

/// Validated transaction group.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DurableSessionTransaction {
    pub(super) frames: Vec<DurableSessionFrame>,
    pub(super) sidecars: Vec<DurableSessionSafeSidecar>,
    pub(super) commit: DurableSessionCommit,
}

impl DurableSessionTransaction {
    /// Returns ordered private frames.
    #[must_use]
    pub fn frames(&self) -> &[DurableSessionFrame] {
        self.frames.as_slice()
    }

    /// Returns metadata-only sidecars.
    #[must_use]
    pub fn sidecars(&self) -> &[DurableSessionSafeSidecar] {
        self.sidecars.as_slice()
    }

    /// Returns the logical commit.
    #[must_use]
    pub const fn commit(&self) -> &DurableSessionCommit {
        &self.commit
    }
}

/// Recovery classification.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DurableSessionRecoveryStatus {
    /// Empty new journal.
    Empty,
    /// Clean committed end.
    Clean,
    /// Incomplete final record bytes were ignored.
    IncompleteTailFrame,
    /// Complete private frames without a commit were ignored.
    IncompleteTailTransaction,
}

/// Validated committed journal snapshot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DurableSessionRecoveryReport {
    pub(super) status: DurableSessionRecoveryStatus,
    pub(super) committed_frames: Vec<DurableSessionFrame>,
    pub(super) active_leaf_id: Option<String>,
    pub(super) committed_head_hash: String,
    pub(super) ignored_tail_bytes: u64,
}

impl DurableSessionRecoveryReport {
    /// Returns recovery class.
    #[must_use]
    pub const fn status(&self) -> DurableSessionRecoveryStatus {
        self.status
    }

    /// Returns committed private frames only.
    #[must_use]
    pub fn committed_frames(&self) -> &[DurableSessionFrame] {
        self.committed_frames.as_slice()
    }

    /// Returns committed active leaf.
    #[must_use]
    pub fn active_leaf_id(&self) -> Option<&str> {
        self.active_leaf_id.as_deref()
    }

    /// Returns committed head identity.
    #[must_use]
    pub fn committed_head_hash(&self) -> &str {
        self.committed_head_hash.as_str()
    }

    /// Returns ignored uncommitted tail bytes.
    #[must_use]
    pub const fn ignored_tail_bytes(&self) -> u64 {
        self.ignored_tail_bytes
    }
}

/// Durable store issue class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DurableSessionStoreIssue {
    /// Generated policy shape was invalid.
    InvalidPolicy,
    /// Generated policy identity was stale.
    StalePolicy,
    /// Store descriptor was missing, unsafe, or unsupported.
    InvalidDescriptor,
    /// Encryption was required but unavailable.
    EncryptionUnavailable,
    /// Private frame was malformed or exceeded bounds.
    InvalidFrame,
    /// Journal record was incomplete at the tail.
    IncompleteTail,
    /// Logical sequence contained a gap or overflow.
    SequenceGap,
    /// Parent, payload, frame, transaction, or commit identity differed.
    IntegrityMismatch,
    /// Commit did not extend the current head.
    StaleHead,
    /// Schema version was unsupported.
    UnsupportedSchema,
    /// Complete record corruption occurred before the clean end.
    MidFileCorruption,
    /// Journal exceeded a configured bound.
    BoundExceeded,
    /// Metadata-only receipt input contained private material.
    ReceiptLeak,
}

/// Safe durable store error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DurableSessionStoreError {
    issue: DurableSessionStoreIssue,
    field: &'static str,
}

impl DurableSessionStoreError {
    pub(super) const fn new(issue: DurableSessionStoreIssue, field: &'static str) -> Self {
        Self { issue, field }
    }

    /// Returns the issue class.
    #[must_use]
    pub const fn issue(&self) -> DurableSessionStoreIssue {
        self.issue
    }

    /// Returns the bounded field label.
    #[must_use]
    pub const fn field(&self) -> &'static str {
        self.field
    }
}

impl core::fmt::Display for DurableSessionStoreError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "durable session store rejected {}", self.field)
    }
}

impl std::error::Error for DurableSessionStoreError {}

/// Safe durable store operation class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DurableSessionStoreOperation {
    /// Store open.
    Open,
    /// Transaction append and commit.
    Commit,
    /// Committed replay.
    Replay,
    /// Recovery observation.
    Recovery,
    /// Writer lock observation.
    Lock,
    /// Explicit close.
    Close,
}

impl DurableSessionStoreOperation {
    /// Returns the stable label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Commit => "commit",
            Self::Replay => "replay",
            Self::Recovery => "recovery",
            Self::Lock => "lock",
            Self::Close => "close",
        }
    }
}

/// Sensitive receipt fields that must remain absent.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DurableSessionStoreSensitiveDraft<'a> {
    /// Private message or summary body.
    pub private_payload: Option<&'a [u8]>,
    /// Raw tool result body.
    pub tool_result: Option<&'a [u8]>,
    /// Provider payload.
    pub provider_payload: Option<&'a str>,
    /// Credential or key material.
    pub credential: Option<&'a str>,
    /// Capability or store path.
    pub private_path: Option<&'a str>,
    /// Raw encoded private frame.
    pub raw_frame: Option<&'a [u8]>,
    /// Lock implementation dump.
    pub lock_dump: Option<&'a str>,
    /// Unbounded error body.
    pub raw_error: Option<&'a str>,
}

/// Metadata-only receipt projection input.
#[derive(Clone, Copy, Debug)]
pub struct DurableSessionStoreReceiptInput<'a> {
    /// Validated generated policy.
    pub policy: &'a DurableSessionStorePolicy,
    /// Descriptor revalidated against the generated policy.
    pub descriptor: &'a DurableSessionStoreDescriptor,
    /// Operation class.
    pub operation: DurableSessionStoreOperation,
    /// Declared durability class bound to the operation.
    pub durability: DurableSessionDurability,
    /// Safe operation status.
    pub status: &'a str,
    /// Prior committed head identity.
    pub prior_head_hash: &'a str,
    /// Resulting committed head identity.
    pub new_head_hash: &'a str,
    /// Committed frame count.
    pub committed_frame_count: u32,
    /// Recovery class.
    pub recovery_status: DurableSessionRecoveryStatus,
    /// Safe lock status.
    pub lock_status: &'a str,
    /// Sensitive values that must all be absent.
    pub sensitive: DurableSessionStoreSensitiveDraft<'a>,
}

/// Metadata-only durable store receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DurableSessionStoreReceipt {
    /// Receipt schema.
    pub schema: String,
    /// Store identity.
    pub store_id: String,
    /// Policy identity.
    pub policy_hash: String,
    /// Session identity.
    pub session_id: String,
    /// Branch identity.
    pub branch_id: String,
    /// Operation class.
    pub operation: DurableSessionStoreOperation,
    /// Declared durability class.
    pub durability: DurableSessionDurability,
    /// Safe status.
    pub status: String,
    /// Prior committed head.
    pub prior_head_hash: String,
    /// New committed head.
    pub new_head_hash: String,
    /// Committed frame count.
    pub committed_frame_count: u32,
    /// Recovery class.
    pub recovery_status: DurableSessionRecoveryStatus,
    /// Safe lock status.
    pub lock_status: String,
    /// Redaction marker.
    pub redaction_marker: String,
    /// Explicit non-claim.
    pub non_claim: String,
    /// Receipt identity.
    pub receipt_hash: String,
}

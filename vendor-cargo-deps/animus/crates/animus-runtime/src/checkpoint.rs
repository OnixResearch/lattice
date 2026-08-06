//! Pure VCS-neutral snapshot ledger contracts.
//!
//! Runtime logic validates already-loaded snapshot records and operation
//! requests. Hosts own snapshot creation, storage, VCS commands, branch/ref/stash
//! management, worktree mutation, cleanup, UI selection, timers, and automatic
//! scheduling.

const SNAPSHOT_RECEIPT_SCHEMA: &str = "animus.vcs-snapshot.receipt.v1";
const HASH_HEX_LENGTH: usize = 64;
const MAX_SAFE_TEXT_BYTES: usize = 160;
const MAX_CHANGED_ARTIFACTS: u32 = 1_000_000;
const HASH_PART_SEPARATOR: char = '|';
const HASH_LENGTH_SEPARATOR: char = ':';
const EMPTY_SUBJECT: &str = "empty";
const UNKNOWN_SUBJECT: &str = "unknown";
const DUPLICATE_SUBJECT: &str = "duplicate";
const SAFE_RECEIPT_STATUS: &str = "metadata_only";

const FIELD_SNAPSHOT: &str = "snapshot_id_hash";
const FIELD_PARENT: &str = "parent_snapshot_id_hash";
const FIELD_WORKSPACE: &str = "workspace_hash";
const FIELD_CONTENT: &str = "content_hash";
const FIELD_ACTOR: &str = "actor_label";
const FIELD_SESSION: &str = "session_label";
const FIELD_RECEIPT_HASH: &str = "receipt_hash";
const FIELD_POLICY: &str = "policy_hash";
const FIELD_CAPABILITY: &str = "required_capability";
const FIELD_COMMAND: &str = "backend_command";
const FIELD_SERVICE: &str = "snapshot_service";
const FIELD_COUNT: &str = "changed_artifact_count";
const FIELD_RECEIPT: &str = "snapshot_receipt";

const LEAK_MARKERS: &[LeakMarker] = &[
    LeakMarker {
        needle: "diff --git",
        class: "raw_diff",
    },
    LeakMarker {
        needle: "unified diff",
        class: "raw_diff",
    },
    LeakMarker {
        needle: "file_body:",
        class: "file_body",
    },
    LeakMarker {
        needle: "command:",
        class: "command_line",
    },
    LeakMarker {
        needle: "git stash",
        class: "backend_command",
    },
    LeakMarker {
        needle: "jj ",
        class: "backend_command",
    },
    LeakMarker {
        needle: "refs/",
        class: "unsafe_backend_ref",
    },
    LeakMarker {
        needle: "/home/",
        class: "full_path",
    },
    LeakMarker {
        needle: "provider payload",
        class: "provider_payload",
    },
    LeakMarker {
        needle: "authorization:",
        class: "credential",
    },
    LeakMarker {
        needle: "api_key",
        class: "credential",
    },
    LeakMarker {
        needle: "tool_output:",
        class: "raw_tool_output",
    },
];

type Text = String;
type Collection<T> = Vec<T>;

#[derive(Clone, Copy)]
struct LeakMarker {
    needle: &'static str,
    class: &'static str,
}

/// Backend family for a host-owned snapshot record.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum VcsSnapshotBackendKind {
    /// Jujutsu-backed host snapshot.
    Jujutsu,
    /// Git-backed host snapshot.
    Git,
    /// BranchFS-like host snapshot.
    BranchFs,
    /// In-memory host snapshot.
    InMemory,
    /// Custom host backend represented only by safe labels and hashes.
    Custom,
}

impl VcsSnapshotBackendKind {
    /// Returns the stable backend label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Jujutsu => "jujutsu",
            Self::Git => "git",
            Self::BranchFs => "branchfs",
            Self::InMemory => "in_memory",
            Self::Custom => "custom",
        }
    }
}

/// Snapshot intent class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VcsSnapshotIntent {
    /// Operator requested a snapshot manually.
    Manual,
    /// Host requested a snapshot before a protected mutation.
    PreMutation,
    /// Host requested a snapshot after a turn through explicit policy.
    PostTurnPolicy,
    /// Host created a restore point.
    RestorePoint,
}

impl VcsSnapshotIntent {
    /// Returns the stable intent label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Manual => "manual",
            Self::PreMutation => "pre_mutation",
            Self::PostTurnPolicy => "post_turn_policy",
            Self::RestorePoint => "restore_point",
        }
    }
}

/// Snapshot operation requested from reusable planning logic.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VcsSnapshotOperation {
    /// Accept a snapshot as the reviewed state.
    Accept,
    /// Restore a snapshot through a host-owned service.
    Restore,
    /// Drop a snapshot.
    Drop,
    /// Squash multiple snapshot records.
    Squash,
    /// Compare snapshots without mutation.
    Compare,
    /// Cleanup host-owned snapshot state.
    Cleanup,
}

impl VcsSnapshotOperation {
    /// Returns the stable operation label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Accept => "accept",
            Self::Restore => "restore",
            Self::Drop => "drop",
            Self::Squash => "squash",
            Self::Compare => "compare",
            Self::Cleanup => "cleanup",
        }
    }
}

/// Protected action class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VcsSnapshotProtectedAction {
    /// Operation only reads or compares metadata.
    ReadOnly,
    /// Operation may mutate host-owned recovery state.
    ProtectedMutation,
}

impl VcsSnapshotProtectedAction {
    /// Returns the stable protected-action label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ReadOnly => "read_only",
            Self::ProtectedMutation => "protected_mutation",
        }
    }
}

/// Freshness class for a snapshot record or proof.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VcsSnapshotFreshness {
    /// Freshness proof matches the current workspace identity.
    Fresh,
    /// Freshness proof is stale.
    Stale,
    /// Freshness proof is unknown.
    Unknown,
}

impl VcsSnapshotFreshness {
    /// Returns the stable freshness label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Fresh => "fresh",
            Self::Stale => "stale",
            Self::Unknown => "unknown",
        }
    }
}

/// Confirmation status for protected snapshot operations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VcsSnapshotConfirmationStatus {
    /// User or policy confirmation is present.
    Confirmed,
    /// Confirmation is missing.
    Missing,
    /// Confirmation was denied.
    Denied,
    /// Confirmation is not required.
    NotRequired,
}

impl VcsSnapshotConfirmationStatus {
    /// Returns the stable confirmation label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Confirmed => "confirmed",
            Self::Missing => "missing",
            Self::Denied => "denied",
            Self::NotRequired => "not_required",
        }
    }
}

/// Host snapshot service status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VcsSnapshotServiceStatus {
    /// Service is available.
    Available,
    /// No service was injected.
    Missing,
    /// Service is injected but unavailable.
    Unavailable,
    /// Service does not support the requested operation.
    Unsupported,
    /// Service or policy denied the operation.
    Denied,
}

impl VcsSnapshotServiceStatus {
    /// Returns the stable service status label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Available => "available",
            Self::Missing => "missing",
            Self::Unavailable => "unavailable",
            Self::Unsupported => "unsupported",
            Self::Denied => "denied",
        }
    }
}

/// Snapshot plan status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VcsSnapshotPlanStatus {
    /// Host may perform the planned operation through its own service.
    Admitted,
    /// Host must not mutate snapshot or VCS state.
    Denied,
}

impl VcsSnapshotPlanStatus {
    /// Returns the stable plan status label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Admitted => "admitted",
            Self::Denied => "denied",
        }
    }
}

/// Snapshot validation error class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VcsSnapshotErrorKind {
    /// A required field was empty.
    EmptyField,
    /// A safe field exceeded the configured bound.
    FieldTooLong,
    /// A field contained unsafe raw material.
    UnsafeText,
    /// A hash field was malformed.
    InvalidHash,
    /// A count exceeded the configured receipt bound.
    CountTooLarge,
}

/// Safe snapshot validation error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VcsSnapshotError {
    kind: VcsSnapshotErrorKind,
    field: &'static str,
    subject: Text,
    reason: Text,
}

impl VcsSnapshotError {
    fn new(kind: VcsSnapshotErrorKind, field: &'static str, subject: impl Into<Text>, reason: impl Into<Text>) -> Self {
        Self {
            kind,
            field,
            subject: subject.into(),
            reason: reason.into(),
        }
    }

    /// Returns the stable error class.
    #[must_use]
    pub const fn kind(&self) -> VcsSnapshotErrorKind {
        self.kind
    }

    /// Returns the failed field.
    #[must_use]
    pub const fn field(&self) -> &'static str {
        self.field
    }

    /// Returns a safe subject label.
    #[must_use]
    pub fn subject(&self) -> &str {
        self.subject.as_str()
    }

    /// Returns a safe reason.
    #[must_use]
    pub fn reason(&self) -> &str {
        self.reason.as_str()
    }
}

impl std::fmt::Display for VcsSnapshotError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "vcs snapshot field `{}` failed ({:?}) for `{}`: {}",
            self.field, self.kind, self.subject, self.reason
        )
    }
}

impl std::error::Error for VcsSnapshotError {}

/// Snapshot planner diagnostic class.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum VcsSnapshotDiagnosticKind {
    /// Target snapshot was missing from the supplied ledger.
    MissingSnapshot,
    /// Snapshot parent lineage was malformed.
    MalformedLineage,
    /// Snapshot service was missing.
    MissingService,
    /// Snapshot service was unavailable.
    UnavailableService,
    /// Snapshot service did not support the requested operation or capability.
    UnsupportedOperation,
    /// Snapshot service or policy denied the request.
    DeniedService,
    /// Freshness proof was stale or unknown for a protected operation.
    StaleSnapshot,
    /// Protected operation lacked confirmation.
    MissingConfirmation,
    /// Confirmation explicitly denied the protected operation.
    ConfirmationDenied,
    /// Backend command syntax or private ref escaped into reusable inputs.
    BackendCommandRejected,
    /// Caller attempted automatic snapshot creation without injected host policy.
    AutomaticSnapshotDenied,
}

impl VcsSnapshotDiagnosticKind {
    /// Returns the stable diagnostic label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MissingSnapshot => "missing_snapshot",
            Self::MalformedLineage => "malformed_lineage",
            Self::MissingService => "missing_service",
            Self::UnavailableService => "unavailable_service",
            Self::UnsupportedOperation => "unsupported_operation",
            Self::DeniedService => "denied_service",
            Self::StaleSnapshot => "stale_snapshot",
            Self::MissingConfirmation => "missing_confirmation",
            Self::ConfirmationDenied => "confirmation_denied",
            Self::BackendCommandRejected => "backend_command_rejected",
            Self::AutomaticSnapshotDenied => "automatic_snapshot_denied",
        }
    }
}

/// Safe snapshot planner diagnostic.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VcsSnapshotDiagnostic {
    kind: VcsSnapshotDiagnosticKind,
    field: &'static str,
    detail: Text,
}

impl VcsSnapshotDiagnostic {
    fn new(kind: VcsSnapshotDiagnosticKind, field: &'static str, detail: impl Into<Text>) -> Self {
        Self {
            kind,
            field,
            detail: detail.into(),
        }
    }

    /// Returns the diagnostic class.
    #[must_use]
    pub const fn kind(&self) -> VcsSnapshotDiagnosticKind {
        self.kind
    }

    /// Returns the diagnostic field.
    #[must_use]
    pub const fn field(&self) -> &'static str {
        self.field
    }

    /// Returns safe diagnostic detail.
    #[must_use]
    pub fn detail(&self) -> &str {
        self.detail.as_str()
    }
}

/// Borrowed snapshot record before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VcsSnapshotRecordDraft<'a> {
    /// Snapshot intent class.
    pub intent: VcsSnapshotIntent,
    /// Host backend kind.
    pub backend_kind: VcsSnapshotBackendKind,
    /// Opaque snapshot identity represented as a hash.
    pub snapshot_id_hash: &'a str,
    /// Optional parent snapshot hash.
    pub parent_snapshot_id_hash: Option<&'a str>,
    /// Workspace label or identity hash.
    pub workspace_hash: &'a str,
    /// Content or artifact identity hashes.
    pub content_hashes: &'a [&'a str],
    /// Changed artifact count.
    pub changed_artifact_count: u32,
    /// Safe actor label.
    pub actor_label: &'a str,
    /// Safe session label.
    pub session_label: &'a str,
    /// Freshness metadata class.
    pub freshness: VcsSnapshotFreshness,
    /// Safe receipt hash for the record.
    pub receipt_hash: &'a str,
}

/// Validated backend-neutral snapshot record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VcsSnapshotRecord {
    intent: VcsSnapshotIntent,
    backend_kind: VcsSnapshotBackendKind,
    snapshot_id_hash: Text,
    parent_snapshot_id_hash: Option<Text>,
    workspace_hash: Text,
    content_hashes: Collection<Text>,
    changed_artifact_count: u32,
    actor_label: Text,
    session_label: Text,
    freshness: VcsSnapshotFreshness,
    receipt_hash: Text,
}

impl VcsSnapshotRecord {
    /// Builds a validated backend-neutral snapshot record.
    ///
    /// # Errors
    ///
    /// Returns [`VcsSnapshotError`] when hashes, labels, or counts are malformed
    /// or contain raw backend/private material.
    pub fn new(draft: &VcsSnapshotRecordDraft<'_>) -> Result<Self, VcsSnapshotError> {
        if draft.changed_artifact_count > MAX_CHANGED_ARTIFACTS {
            return Err(VcsSnapshotError::new(
                VcsSnapshotErrorKind::CountTooLarge,
                FIELD_COUNT,
                draft.changed_artifact_count.to_string(),
                "changed artifact count exceeds receipt bound",
            ));
        }
        Ok(Self {
            intent: draft.intent,
            backend_kind: draft.backend_kind,
            snapshot_id_hash: validate_hash(SafeFieldValue::new(FIELD_SNAPSHOT, draft.snapshot_id_hash))?,
            parent_snapshot_id_hash: draft
                .parent_snapshot_id_hash
                .map(|hash| validate_hash(SafeFieldValue::new(FIELD_PARENT, hash)))
                .transpose()?,
            workspace_hash: validate_hash(SafeFieldValue::new(FIELD_WORKSPACE, draft.workspace_hash))?,
            content_hashes: validate_hashes(FIELD_CONTENT, draft.content_hashes)?,
            changed_artifact_count: draft.changed_artifact_count,
            actor_label: validate_safe_text(SafeFieldValue::new(FIELD_ACTOR, draft.actor_label))?,
            session_label: validate_safe_text(SafeFieldValue::new(FIELD_SESSION, draft.session_label))?,
            freshness: draft.freshness,
            receipt_hash: validate_hash(SafeFieldValue::new(FIELD_RECEIPT_HASH, draft.receipt_hash))?,
        })
    }

    /// Returns snapshot intent.
    #[must_use]
    pub const fn intent(&self) -> VcsSnapshotIntent {
        self.intent
    }

    /// Returns backend kind.
    #[must_use]
    pub const fn backend_kind(&self) -> VcsSnapshotBackendKind {
        self.backend_kind
    }

    /// Returns snapshot identity hash.
    #[must_use]
    pub fn snapshot_id_hash(&self) -> &str {
        self.snapshot_id_hash.as_str()
    }

    /// Returns parent snapshot hash.
    #[must_use]
    pub fn parent_snapshot_id_hash(&self) -> Option<&str> {
        self.parent_snapshot_id_hash.as_deref()
    }

    /// Returns workspace hash.
    #[must_use]
    pub fn workspace_hash(&self) -> &str {
        self.workspace_hash.as_str()
    }

    /// Returns content hashes.
    #[must_use]
    pub fn content_hashes(&self) -> &[Text] {
        self.content_hashes.as_slice()
    }

    /// Returns changed artifact count.
    #[must_use]
    pub const fn changed_artifact_count(&self) -> u32 {
        self.changed_artifact_count
    }

    /// Returns safe actor label.
    #[must_use]
    pub fn actor_label(&self) -> &str {
        self.actor_label.as_str()
    }

    /// Returns safe session label.
    #[must_use]
    pub fn session_label(&self) -> &str {
        self.session_label.as_str()
    }

    /// Returns freshness class.
    #[must_use]
    pub const fn freshness(&self) -> VcsSnapshotFreshness {
        self.freshness
    }

    /// Returns receipt hash.
    #[must_use]
    pub fn receipt_hash(&self) -> &str {
        self.receipt_hash.as_str()
    }
}

/// Borrowed operation request before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VcsSnapshotPlanRequestDraft<'a> {
    /// Requested operation.
    pub operation: VcsSnapshotOperation,
    /// Target snapshot identity hash.
    pub target_snapshot_id_hash: &'a str,
    /// Current workspace identity hash.
    pub current_workspace_hash: &'a str,
    /// Freshness proof workspace hash.
    pub freshness_workspace_hash: &'a str,
    /// Policy identity hash.
    pub policy_hash: &'a str,
    /// Protected action class.
    pub protected_action: VcsSnapshotProtectedAction,
    /// Confirmation status.
    pub confirmation: VcsSnapshotConfirmationStatus,
    /// Required backend capability labels.
    pub required_capabilities: &'a [&'a str],
    /// Optional backend command text supplied by mistake.
    pub backend_command: Option<&'a str>,
    /// Whether the caller requested automatic snapshot creation.
    pub automatic_snapshot_requested: bool,
}

/// Validated operation request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VcsSnapshotPlanRequest {
    operation: VcsSnapshotOperation,
    target_snapshot_id_hash: Text,
    current_workspace_hash: Text,
    freshness_workspace_hash: Text,
    policy_hash: Text,
    protected_action: VcsSnapshotProtectedAction,
    confirmation: VcsSnapshotConfirmationStatus,
    required_capabilities: Collection<Text>,
    backend_command: Option<Text>,
    automatic_snapshot_requested: bool,
}

impl VcsSnapshotPlanRequest {
    /// Builds a validated snapshot operation request.
    ///
    /// # Errors
    ///
    /// Returns [`VcsSnapshotError`] when hashes or labels are malformed or raw
    /// command/private backend material is present.
    pub fn new(draft: &VcsSnapshotPlanRequestDraft<'_>) -> Result<Self, VcsSnapshotError> {
        Ok(Self {
            operation: draft.operation,
            target_snapshot_id_hash: validate_hash(SafeFieldValue::new(FIELD_SNAPSHOT, draft.target_snapshot_id_hash))?,
            current_workspace_hash: validate_hash(SafeFieldValue::new(FIELD_WORKSPACE, draft.current_workspace_hash))?,
            freshness_workspace_hash: validate_hash(SafeFieldValue::new(
                FIELD_WORKSPACE,
                draft.freshness_workspace_hash,
            ))?,
            policy_hash: validate_hash(SafeFieldValue::new(FIELD_POLICY, draft.policy_hash))?,
            protected_action: draft.protected_action,
            confirmation: draft.confirmation,
            required_capabilities: validate_safe_texts(FIELD_CAPABILITY, draft.required_capabilities)?,
            backend_command: draft
                .backend_command
                .map(|command| validate_safe_text(SafeFieldValue::new(FIELD_COMMAND, command)))
                .transpose()?,
            automatic_snapshot_requested: draft.automatic_snapshot_requested,
        })
    }

    /// Returns requested operation.
    #[must_use]
    pub const fn operation(&self) -> VcsSnapshotOperation {
        self.operation
    }

    /// Returns target snapshot hash.
    #[must_use]
    pub fn target_snapshot_id_hash(&self) -> &str {
        self.target_snapshot_id_hash.as_str()
    }

    /// Returns current workspace hash.
    #[must_use]
    pub fn current_workspace_hash(&self) -> &str {
        self.current_workspace_hash.as_str()
    }

    /// Returns freshness proof workspace hash.
    #[must_use]
    pub fn freshness_workspace_hash(&self) -> &str {
        self.freshness_workspace_hash.as_str()
    }

    /// Returns policy hash.
    #[must_use]
    pub fn policy_hash(&self) -> &str {
        self.policy_hash.as_str()
    }

    /// Returns protected action class.
    #[must_use]
    pub const fn protected_action(&self) -> VcsSnapshotProtectedAction {
        self.protected_action
    }

    /// Returns confirmation status.
    #[must_use]
    pub const fn confirmation(&self) -> VcsSnapshotConfirmationStatus {
        self.confirmation
    }

    /// Returns required capabilities.
    #[must_use]
    pub fn required_capabilities(&self) -> &[Text] {
        self.required_capabilities.as_slice()
    }

    /// Returns backend command text if one was supplied.
    #[must_use]
    pub fn backend_command(&self) -> Option<&str> {
        self.backend_command.as_deref()
    }

    /// Returns true when the request asked reusable logic to create snapshots automatically.
    #[must_use]
    pub const fn automatic_snapshot_requested(&self) -> bool {
        self.automatic_snapshot_requested
    }
}

/// Pure snapshot planner input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VcsSnapshotPlannerInput<'a> {
    /// Validated operation request.
    pub request: &'a VcsSnapshotPlanRequest,
    /// Validated snapshot records.
    pub records: &'a [VcsSnapshotRecord],
    /// Host snapshot service status.
    pub service_status: VcsSnapshotServiceStatus,
    /// Host-advertised capability labels.
    pub supported_capabilities: &'a [&'a str],
}

/// Safe snapshot operation receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VcsSnapshotReceipt {
    operation: VcsSnapshotOperation,
    status: VcsSnapshotPlanStatus,
    backend_kind: Option<VcsSnapshotBackendKind>,
    snapshot_id_hash: Text,
    workspace_hash: Text,
    changed_artifact_count: u32,
    freshness: VcsSnapshotFreshness,
    confirmation: VcsSnapshotConfirmationStatus,
    policy_hash: Text,
    receipt_hash: Text,
    diagnostics: Collection<VcsSnapshotDiagnosticKind>,
}

impl VcsSnapshotReceipt {
    /// Returns operation.
    #[must_use]
    pub const fn operation(&self) -> VcsSnapshotOperation {
        self.operation
    }

    /// Returns plan status.
    #[must_use]
    pub const fn status(&self) -> VcsSnapshotPlanStatus {
        self.status
    }

    /// Returns backend kind, when a record was found.
    #[must_use]
    pub const fn backend_kind(&self) -> Option<VcsSnapshotBackendKind> {
        self.backend_kind
    }

    /// Returns snapshot hash.
    #[must_use]
    pub fn snapshot_id_hash(&self) -> &str {
        self.snapshot_id_hash.as_str()
    }

    /// Returns workspace hash.
    #[must_use]
    pub fn workspace_hash(&self) -> &str {
        self.workspace_hash.as_str()
    }

    /// Returns changed artifact count.
    #[must_use]
    pub const fn changed_artifact_count(&self) -> u32 {
        self.changed_artifact_count
    }

    /// Returns freshness class.
    #[must_use]
    pub const fn freshness(&self) -> VcsSnapshotFreshness {
        self.freshness
    }

    /// Returns confirmation status.
    #[must_use]
    pub const fn confirmation(&self) -> VcsSnapshotConfirmationStatus {
        self.confirmation
    }

    /// Returns policy hash.
    #[must_use]
    pub fn policy_hash(&self) -> &str {
        self.policy_hash.as_str()
    }

    /// Returns receipt hash.
    #[must_use]
    pub fn receipt_hash(&self) -> &str {
        self.receipt_hash.as_str()
    }

    /// Returns diagnostic classes.
    #[must_use]
    pub fn diagnostics(&self) -> &[VcsSnapshotDiagnosticKind] {
        self.diagnostics.as_slice()
    }
}

/// Pure snapshot operation plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VcsSnapshotPlan {
    status: VcsSnapshotPlanStatus,
    operation: VcsSnapshotOperation,
    target_snapshot_id_hash: Text,
    diagnostics: Collection<VcsSnapshotDiagnostic>,
    receipt: VcsSnapshotReceipt,
}

impl VcsSnapshotPlan {
    /// Returns plan status.
    #[must_use]
    pub const fn status(&self) -> VcsSnapshotPlanStatus {
        self.status
    }

    /// Returns operation.
    #[must_use]
    pub const fn operation(&self) -> VcsSnapshotOperation {
        self.operation
    }

    /// Returns target snapshot hash.
    #[must_use]
    pub fn target_snapshot_id_hash(&self) -> &str {
        self.target_snapshot_id_hash.as_str()
    }

    /// Returns safe diagnostics.
    #[must_use]
    pub fn diagnostics(&self) -> &[VcsSnapshotDiagnostic] {
        self.diagnostics.as_slice()
    }

    /// Returns metadata-only receipt.
    #[must_use]
    pub const fn receipt(&self) -> &VcsSnapshotReceipt {
        &self.receipt
    }
}

/// Plans a VCS-neutral snapshot operation without mutating host state.
#[must_use]
pub fn plan_vcs_snapshot_operation(input: VcsSnapshotPlannerInput<'_>) -> VcsSnapshotPlan {
    // r[impl agent_harness_runtime.hermes_vcs_snapshot_ledger.plans]
    debug_assert!(!input.request.target_snapshot_id_hash().is_empty());
    debug_assert!(!input.request.policy_hash().is_empty());
    let mut diagnostics = Collection::new();
    let index = record_index(input.records, &mut diagnostics);
    let target = index.get(input.request.target_snapshot_id_hash()).copied();
    if target.is_none() {
        diagnostics.push(VcsSnapshotDiagnostic::new(
            VcsSnapshotDiagnosticKind::MissingSnapshot,
            FIELD_SNAPSHOT,
            "target snapshot is absent from supplied ledger",
        ));
    }
    apply_host_service_diagnostics(input, &mut diagnostics);
    apply_freshness_diagnostics(input.request, target, &mut diagnostics);
    apply_confirmation_diagnostics(input.request, &mut diagnostics);
    apply_capability_diagnostics(input, &mut diagnostics);
    apply_command_diagnostics(input.request, &mut diagnostics);
    let status = if diagnostics.is_empty() {
        VcsSnapshotPlanStatus::Admitted
    } else {
        VcsSnapshotPlanStatus::Denied
    };
    let receipt = snapshot_receipt(ReceiptInput {
        request: input.request,
        record: target,
        status,
        diagnostics: diagnostics.as_slice(),
    });
    VcsSnapshotPlan {
        status,
        operation: input.request.operation(),
        target_snapshot_id_hash: input.request.target_snapshot_id_hash().to_owned(),
        diagnostics,
        receipt,
    }
}

/// Validates receipt text for unsafe snapshot material.
///
/// # Errors
///
/// Returns [`VcsSnapshotError`] when raw diffs, file bodies, command lines,
/// full private paths, provider payloads, credentials, raw tool output, or
/// unsafe backend refs are present.
pub fn validate_vcs_snapshot_receipt_text(text: &str) -> Result<(), VcsSnapshotError> {
    if let Some(marker) = contains_unsafe_material(text) {
        return Err(VcsSnapshotError::new(
            VcsSnapshotErrorKind::UnsafeText,
            FIELD_RECEIPT,
            marker.class,
            "snapshot receipt text contains unsafe material",
        ));
    }
    Ok(())
}

fn record_index<'a>(
    records: &'a [VcsSnapshotRecord],
    diagnostics: &mut Collection<VcsSnapshotDiagnostic>,
) -> std::collections::BTreeMap<&'a str, &'a VcsSnapshotRecord> {
    // r[impl agent_harness_runtime.hermes_vcs_snapshot_ledger.records]
    debug_assert!(diagnostics.is_empty());
    debug_assert!(records.len() <= records.len().saturating_add(usize::from(1_u8)));
    let mut index = std::collections::BTreeMap::new();
    for record in records {
        if index.insert(record.snapshot_id_hash(), record).is_some() {
            diagnostics.push(VcsSnapshotDiagnostic::new(
                VcsSnapshotDiagnosticKind::MalformedLineage,
                FIELD_SNAPSHOT,
                DUPLICATE_SUBJECT,
            ));
        }
    }
    for record in records {
        if let Some(parent) = record.parent_snapshot_id_hash() {
            if !index.contains_key(parent) {
                diagnostics.push(VcsSnapshotDiagnostic::new(
                    VcsSnapshotDiagnosticKind::MalformedLineage,
                    FIELD_PARENT,
                    "parent snapshot is missing",
                ));
            }
        }
    }
    index
}

fn apply_host_service_diagnostics(
    input: VcsSnapshotPlannerInput<'_>,
    diagnostics: &mut Collection<VcsSnapshotDiagnostic>,
) {
    // r[impl agent_harness_runtime.hermes_vcs_snapshot_ledger.host_owned]
    debug_assert!(!input.request.target_snapshot_id_hash().is_empty());
    debug_assert!(!input.request.policy_hash().is_empty());
    match input.service_status {
        VcsSnapshotServiceStatus::Available => {}
        VcsSnapshotServiceStatus::Missing => diagnostics.push(VcsSnapshotDiagnostic::new(
            VcsSnapshotDiagnosticKind::MissingService,
            FIELD_SERVICE,
            "snapshot service was not injected",
        )),
        VcsSnapshotServiceStatus::Unavailable => diagnostics.push(VcsSnapshotDiagnostic::new(
            VcsSnapshotDiagnosticKind::UnavailableService,
            FIELD_SERVICE,
            "snapshot service is unavailable",
        )),
        VcsSnapshotServiceStatus::Unsupported => diagnostics.push(VcsSnapshotDiagnostic::new(
            VcsSnapshotDiagnosticKind::UnsupportedOperation,
            FIELD_SERVICE,
            "snapshot service does not support operation",
        )),
        VcsSnapshotServiceStatus::Denied => diagnostics.push(VcsSnapshotDiagnostic::new(
            VcsSnapshotDiagnosticKind::DeniedService,
            FIELD_SERVICE,
            "snapshot service denied operation",
        )),
    }
    if input.request.automatic_snapshot_requested() {
        diagnostics.push(VcsSnapshotDiagnostic::new(
            VcsSnapshotDiagnosticKind::AutomaticSnapshotDenied,
            FIELD_SERVICE,
            "automatic snapshot creation stays host-owned",
        ));
    }
}

fn apply_freshness_diagnostics(
    request: &VcsSnapshotPlanRequest,
    target: Option<&VcsSnapshotRecord>,
    diagnostics: &mut Collection<VcsSnapshotDiagnostic>,
) {
    if request.operation() == VcsSnapshotOperation::Compare {
        return;
    }
    let is_proof_matching_workspace = request.current_workspace_hash() == request.freshness_workspace_hash();
    let is_record_fresh = target.is_some_and(|record| record.freshness() == VcsSnapshotFreshness::Fresh);
    if !is_proof_matching_workspace || !is_record_fresh {
        diagnostics.push(VcsSnapshotDiagnostic::new(
            VcsSnapshotDiagnosticKind::StaleSnapshot,
            FIELD_WORKSPACE,
            "freshness proof does not match current workspace",
        ));
    }
}

fn apply_confirmation_diagnostics(
    request: &VcsSnapshotPlanRequest,
    diagnostics: &mut Collection<VcsSnapshotDiagnostic>,
) {
    debug_assert!(!request.target_snapshot_id_hash().is_empty());
    debug_assert!(!request.policy_hash().is_empty());
    if request.protected_action() == VcsSnapshotProtectedAction::ReadOnly {
        return;
    }
    match request.confirmation() {
        VcsSnapshotConfirmationStatus::Confirmed => {}
        VcsSnapshotConfirmationStatus::Missing | VcsSnapshotConfirmationStatus::NotRequired => {
            diagnostics.push(VcsSnapshotDiagnostic::new(
                VcsSnapshotDiagnosticKind::MissingConfirmation,
                FIELD_POLICY,
                "protected operation requires confirmation",
            ));
        }
        VcsSnapshotConfirmationStatus::Denied => diagnostics.push(VcsSnapshotDiagnostic::new(
            VcsSnapshotDiagnosticKind::ConfirmationDenied,
            FIELD_POLICY,
            "confirmation denied protected operation",
        )),
    }
}

fn apply_capability_diagnostics(
    input: VcsSnapshotPlannerInput<'_>,
    diagnostics: &mut Collection<VcsSnapshotDiagnostic>,
) {
    let capability_allowlist = input.supported_capabilities.iter().copied().collect::<std::collections::BTreeSet<_>>();
    for required in input.request.required_capabilities() {
        if !capability_allowlist.contains(required.as_str()) {
            diagnostics.push(VcsSnapshotDiagnostic::new(
                VcsSnapshotDiagnosticKind::UnsupportedOperation,
                FIELD_CAPABILITY,
                "required snapshot capability is unsupported",
            ));
        }
    }
}

fn apply_command_diagnostics(request: &VcsSnapshotPlanRequest, diagnostics: &mut Collection<VcsSnapshotDiagnostic>) {
    if request.backend_command().is_some() {
        diagnostics.push(VcsSnapshotDiagnostic::new(
            VcsSnapshotDiagnosticKind::BackendCommandRejected,
            FIELD_COMMAND,
            "backend command syntax is host-owned",
        ));
    }
}

#[derive(Clone, Copy)]
struct ReceiptInput<'a> {
    request: &'a VcsSnapshotPlanRequest,
    record: Option<&'a VcsSnapshotRecord>,
    status: VcsSnapshotPlanStatus,
    diagnostics: &'a [VcsSnapshotDiagnostic],
}

fn snapshot_receipt(input: ReceiptInput<'_>) -> VcsSnapshotReceipt {
    // r[impl agent_harness_runtime.hermes_vcs_snapshot_ledger.receipts]
    debug_assert!(!input.request.target_snapshot_id_hash().is_empty());
    debug_assert!(!input.request.policy_hash().is_empty());
    let diagnostics = input.diagnostics.iter().map(VcsSnapshotDiagnostic::kind).collect::<Collection<_>>();
    let backend_kind = input.record.map(VcsSnapshotRecord::backend_kind);
    let changed_artifact_count = input.record.map_or(u32::from(0_u8), VcsSnapshotRecord::changed_artifact_count);
    let freshness = input.record.map_or(VcsSnapshotFreshness::Unknown, VcsSnapshotRecord::freshness);
    let workspace_hash = input
        .record
        .map_or_else(|| input.request.current_workspace_hash(), VcsSnapshotRecord::workspace_hash)
        .to_owned();
    let receipt_hash = snapshot_receipt_hash(ReceiptHashInput {
        request: input.request,
        status: input.status,
        backend_kind,
        changed_artifact_count,
        freshness,
        workspace_hash: workspace_hash.as_str(),
        diagnostics: diagnostics.as_slice(),
    });
    VcsSnapshotReceipt {
        operation: input.request.operation(),
        status: input.status,
        backend_kind,
        snapshot_id_hash: input.request.target_snapshot_id_hash().to_owned(),
        workspace_hash,
        changed_artifact_count,
        freshness,
        confirmation: input.request.confirmation(),
        policy_hash: input.request.policy_hash().to_owned(),
        receipt_hash,
        diagnostics,
    }
}

#[derive(Clone, Copy)]
struct ReceiptHashInput<'a> {
    request: &'a VcsSnapshotPlanRequest,
    status: VcsSnapshotPlanStatus,
    backend_kind: Option<VcsSnapshotBackendKind>,
    changed_artifact_count: u32,
    freshness: VcsSnapshotFreshness,
    workspace_hash: &'a str,
    diagnostics: &'a [VcsSnapshotDiagnosticKind],
}

fn snapshot_receipt_hash(input: ReceiptHashInput<'_>) -> Text {
    debug_assert!(!input.request.target_snapshot_id_hash().is_empty());
    debug_assert!(!input.workspace_hash.is_empty());
    let mut normalized = Text::from(SNAPSHOT_RECEIPT_SCHEMA);
    push_hash_part(&mut normalized, SAFE_RECEIPT_STATUS);
    push_hash_part(&mut normalized, input.request.operation().as_str());
    push_hash_part(&mut normalized, input.status.as_str());
    push_hash_part(&mut normalized, input.backend_kind.map_or(UNKNOWN_SUBJECT, VcsSnapshotBackendKind::as_str));
    push_hash_part(&mut normalized, input.request.target_snapshot_id_hash());
    push_hash_part(&mut normalized, input.workspace_hash);
    push_hash_part(&mut normalized, &input.changed_artifact_count.to_string());
    push_hash_part(&mut normalized, input.freshness.as_str());
    push_hash_part(&mut normalized, input.request.confirmation().as_str());
    push_hash_part(&mut normalized, input.request.policy_hash());
    for diagnostic in input.diagnostics {
        push_hash_part(&mut normalized, diagnostic.as_str());
    }
    blake3::hash(normalized.as_bytes()).to_hex().to_string()
}

fn validate_hashes(field: &'static str, values: &[&str]) -> Result<Collection<Text>, VcsSnapshotError> {
    values.iter().map(|value| validate_hash(SafeFieldValue::new(field, value))).collect()
}

fn validate_safe_texts(field: &'static str, values: &[&str]) -> Result<Collection<Text>, VcsSnapshotError> {
    values.iter().map(|value| validate_safe_text(SafeFieldValue::new(field, value))).collect()
}

#[derive(Clone, Copy)]
struct SafeFieldValue<'a> {
    field: &'static str,
    value: &'a str,
}

impl<'a> SafeFieldValue<'a> {
    const fn new(field: &'static str, value: &'a str) -> Self {
        Self { field, value }
    }
}

fn validate_hash(input: SafeFieldValue<'_>) -> Result<Text, VcsSnapshotError> {
    let text = validate_safe_text(input)?;
    if text.len() != HASH_HEX_LENGTH || !text.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(VcsSnapshotError::new(
            VcsSnapshotErrorKind::InvalidHash,
            input.field,
            EMPTY_SUBJECT,
            "value must be a BLAKE3 hexadecimal digest",
        ));
    }
    Ok(text)
}

fn validate_safe_text(input: SafeFieldValue<'_>) -> Result<Text, VcsSnapshotError> {
    debug_assert!(!input.field.is_empty());
    if input.value.is_empty() {
        return Err(VcsSnapshotError::new(
            VcsSnapshotErrorKind::EmptyField,
            input.field,
            EMPTY_SUBJECT,
            "value must not be empty",
        ));
    }
    if input.value.len() > MAX_SAFE_TEXT_BYTES {
        return Err(VcsSnapshotError::new(
            VcsSnapshotErrorKind::FieldTooLong,
            input.field,
            EMPTY_SUBJECT,
            "value exceeds safe text bound",
        ));
    }
    if let Some(marker) = contains_unsafe_material(input.value) {
        return Err(VcsSnapshotError::new(
            VcsSnapshotErrorKind::UnsafeText,
            input.field,
            marker.class,
            "value contains unsafe material marker",
        ));
    }
    Ok(input.value.to_owned())
}

fn contains_unsafe_material(value: &str) -> Option<LeakMarker> {
    let lower = value.to_ascii_lowercase();
    LEAK_MARKERS.iter().copied().find(|marker| lower.contains(marker.needle))
}

fn push_hash_part(normalized: &mut Text, part: &str) {
    normalized.push(HASH_PART_SEPARATOR);
    normalized.push_str(&part.len().to_string());
    normalized.push(HASH_LENGTH_SEPARATOR);
    normalized.push_str(part);
}

#[cfg(test)]
mod tests {
    use super::*;

    const HASH_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const HASH_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const HASH_C: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
    const HASH_D: &str = "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";
    const HASH_E: &str = "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";
    const ACTOR: &str = "actor.host";
    const SESSION: &str = "session.safe";
    const CAP_RESTORE: &str = "snapshot.restore";
    const CAP_COMPARE: &str = "snapshot.compare";
    const EXPECTED_LINEAGE_RECORDS: usize = 2;

    #[test]
    fn backend_neutral_records_accept_jj_and_git_lineage() {
        // r[verify agent_harness_runtime.hermes_vcs_snapshot_ledger.records.backend_neutral]
        let records = records().expect("records should validate");
        assert_eq!(records.len(), EXPECTED_LINEAGE_RECORDS);
        assert_eq!(records[0].backend_kind(), VcsSnapshotBackendKind::Jujutsu);
        assert_eq!(records[1].backend_kind(), VcsSnapshotBackendKind::Git);
        assert_eq!(records[1].parent_snapshot_id_hash(), Some(HASH_A));
    }

    #[test]
    fn restore_plan_requires_fresh_proof_and_confirmation() {
        // r[verify agent_harness_runtime.hermes_vcs_snapshot_ledger.plans.restore_freshness]
        let records = records().expect("records should validate");
        let request = request(RequestFixture::protected_restore()).expect("request should validate");
        let plan = plan_vcs_snapshot_operation(VcsSnapshotPlannerInput {
            request: &request,
            records: records.as_slice(),
            service_status: VcsSnapshotServiceStatus::Available,
            supported_capabilities: &[CAP_RESTORE, CAP_COMPARE],
        });

        assert_eq!(plan.status(), VcsSnapshotPlanStatus::Admitted);
        assert_eq!(plan.target_snapshot_id_hash(), HASH_B);
        assert_eq!(plan.receipt().backend_kind(), Some(VcsSnapshotBackendKind::Git));
        assert_eq!(plan.receipt().receipt_hash().len(), HASH_HEX_LENGTH);
    }

    #[test]
    fn compare_plan_is_read_only_and_needs_no_confirmation() {
        let records = records().expect("records should validate");
        let request = VcsSnapshotPlanRequest::new(&VcsSnapshotPlanRequestDraft {
            required_capabilities: &[CAP_COMPARE],
            ..request_draft(RequestFixture::compare())
        })
        .expect("request should validate");
        let plan = plan_vcs_snapshot_operation(VcsSnapshotPlannerInput {
            request: &request,
            records: records.as_slice(),
            service_status: VcsSnapshotServiceStatus::Available,
            supported_capabilities: &[CAP_COMPARE],
        });

        assert_eq!(plan.status(), VcsSnapshotPlanStatus::Admitted);
        assert_eq!(plan.operation(), VcsSnapshotOperation::Compare);
        assert!(plan.diagnostics().is_empty());
    }

    #[test]
    fn stale_missing_service_no_confirmation_and_auto_snapshot_fail_closed() {
        // r[verify agent_harness_runtime.hermes_vcs_snapshot_ledger.host_owned.no_autocommit]
        let records = records().expect("records should validate");
        let stale = request(RequestFixture::stale_restore()).expect("stale request should validate");
        let stale_plan = plan_vcs_snapshot_operation(VcsSnapshotPlannerInput {
            request: &stale,
            records: records.as_slice(),
            service_status: VcsSnapshotServiceStatus::Missing,
            supported_capabilities: &[CAP_RESTORE],
        });
        assert_eq!(stale_plan.status(), VcsSnapshotPlanStatus::Denied);
        assert_diagnostic(&stale_plan, VcsSnapshotDiagnosticKind::StaleSnapshot);
        assert_diagnostic(&stale_plan, VcsSnapshotDiagnosticKind::MissingService);

        let unconfirmed = VcsSnapshotPlanRequest::new(&VcsSnapshotPlanRequestDraft {
            confirmation: VcsSnapshotConfirmationStatus::Missing,
            automatic_snapshot_requested: true,
            ..request_draft(RequestFixture::protected_drop())
        })
        .expect("unconfirmed request should validate");
        let unconfirmed_plan = plan_vcs_snapshot_operation(VcsSnapshotPlannerInput {
            request: &unconfirmed,
            records: records.as_slice(),
            service_status: VcsSnapshotServiceStatus::Available,
            supported_capabilities: &[CAP_RESTORE],
        });
        assert_eq!(unconfirmed_plan.status(), VcsSnapshotPlanStatus::Denied);
        assert_diagnostic(&unconfirmed_plan, VcsSnapshotDiagnosticKind::MissingConfirmation);
        assert_diagnostic(&unconfirmed_plan, VcsSnapshotDiagnosticKind::AutomaticSnapshotDenied);
    }

    #[test]
    fn unsupported_capability_and_backend_command_are_rejected() {
        let records = records().expect("records should validate");
        let command_error = request(RequestFixture::backend_command("git stash refs/private"))
            .expect_err("backend command syntax should fail validation");
        assert_eq!(command_error.kind(), VcsSnapshotErrorKind::UnsafeText);

        let safe_command = request(RequestFixture::backend_command("host_command_redacted"))
            .expect("safe command label should validate before planning");
        let command_plan = plan_vcs_snapshot_operation(VcsSnapshotPlannerInput {
            request: &safe_command,
            records: records.as_slice(),
            service_status: VcsSnapshotServiceStatus::Available,
            supported_capabilities: &[],
        });
        assert_eq!(command_plan.status(), VcsSnapshotPlanStatus::Denied);
        assert_diagnostic(&command_plan, VcsSnapshotDiagnosticKind::BackendCommandRejected);
        assert_diagnostic(&command_plan, VcsSnapshotDiagnosticKind::UnsupportedOperation);
    }

    #[test]
    fn receipt_leak_fixtures_are_rejected() {
        // r[verify agent_harness_runtime.hermes_vcs_snapshot_ledger.receipts.rejects_raw_diff]
        for fixture in [
            "diff --git a/private b/private",
            "/home/user/project/file.rs",
            "command: git stash",
            "authorization: token",
        ] {
            let error =
                validate_vcs_snapshot_receipt_text(fixture).expect_err("unsafe snapshot receipt text should fail");
            assert_eq!(error.kind(), VcsSnapshotErrorKind::UnsafeText);
        }
    }

    fn assert_diagnostic(plan: &VcsSnapshotPlan, expected: VcsSnapshotDiagnosticKind) {
        assert!(
            plan.diagnostics().iter().any(|diagnostic| diagnostic.kind() == expected),
            "expected {expected:?} in {:?}",
            plan.diagnostics()
        );
    }

    fn records() -> Result<Collection<VcsSnapshotRecord>, VcsSnapshotError> {
        [
            VcsSnapshotRecordDraft {
                intent: VcsSnapshotIntent::Manual,
                backend_kind: VcsSnapshotBackendKind::Jujutsu,
                snapshot_id_hash: HASH_A,
                parent_snapshot_id_hash: None,
                workspace_hash: HASH_C,
                content_hashes: &[HASH_D],
                changed_artifact_count: u32::from(3_u8),
                actor_label: ACTOR,
                session_label: SESSION,
                freshness: VcsSnapshotFreshness::Fresh,
                receipt_hash: HASH_E,
            },
            VcsSnapshotRecordDraft {
                intent: VcsSnapshotIntent::RestorePoint,
                backend_kind: VcsSnapshotBackendKind::Git,
                snapshot_id_hash: HASH_B,
                parent_snapshot_id_hash: Some(HASH_A),
                workspace_hash: HASH_C,
                content_hashes: &[HASH_D, HASH_E],
                changed_artifact_count: u32::from(5_u8),
                actor_label: ACTOR,
                session_label: SESSION,
                freshness: VcsSnapshotFreshness::Fresh,
                receipt_hash: HASH_E,
            },
        ]
        .iter()
        .map(VcsSnapshotRecord::new)
        .collect()
    }

    #[derive(Clone, Copy)]
    struct RequestFixture<'a> {
        operation: VcsSnapshotOperation,
        snapshot_hash: &'a str,
        freshness_hash: &'a str,
        protected: bool,
        backend_command: Option<&'a str>,
    }

    impl RequestFixture<'_> {
        const fn protected_restore() -> Self {
            Self {
                operation: VcsSnapshotOperation::Restore,
                snapshot_hash: HASH_B,
                freshness_hash: HASH_C,
                protected: true,
                backend_command: None,
            }
        }

        const fn stale_restore() -> Self {
            Self {
                operation: VcsSnapshotOperation::Restore,
                snapshot_hash: HASH_B,
                freshness_hash: HASH_D,
                protected: true,
                backend_command: None,
            }
        }

        const fn protected_drop() -> Self {
            Self {
                operation: VcsSnapshotOperation::Drop,
                snapshot_hash: HASH_B,
                freshness_hash: HASH_C,
                protected: true,
                backend_command: None,
            }
        }

        const fn compare() -> Self {
            Self {
                operation: VcsSnapshotOperation::Compare,
                snapshot_hash: HASH_A,
                freshness_hash: HASH_D,
                protected: false,
                backend_command: None,
            }
        }

        const fn backend_command(command: &str) -> RequestFixture<'_> {
            RequestFixture {
                operation: VcsSnapshotOperation::Accept,
                snapshot_hash: HASH_B,
                freshness_hash: HASH_C,
                protected: true,
                backend_command: Some(command),
            }
        }
    }

    fn request(fixture: RequestFixture<'_>) -> Result<VcsSnapshotPlanRequest, VcsSnapshotError> {
        VcsSnapshotPlanRequest::new(&request_draft(fixture))
    }

    fn request_draft(fixture: RequestFixture<'_>) -> VcsSnapshotPlanRequestDraft<'_> {
        VcsSnapshotPlanRequestDraft {
            operation: fixture.operation,
            target_snapshot_id_hash: fixture.snapshot_hash,
            current_workspace_hash: HASH_C,
            freshness_workspace_hash: fixture.freshness_hash,
            policy_hash: HASH_A,
            protected_action: if fixture.protected {
                VcsSnapshotProtectedAction::ProtectedMutation
            } else {
                VcsSnapshotProtectedAction::ReadOnly
            },
            confirmation: if fixture.protected {
                VcsSnapshotConfirmationStatus::Confirmed
            } else {
                VcsSnapshotConfirmationStatus::NotRequired
            },
            required_capabilities: &[CAP_RESTORE],
            backend_command: fixture.backend_command,
            automatic_snapshot_requested: false,
        }
    }
}

//! Pure session branch planning contracts.
//!
//! Hosts own user interfaces, durable session stores, provider replay, live
//! processes, checkpoints, rollback execution, clocks, and filesystem access.
//! This module validates already-loaded session branch DTOs and returns a
//! deterministic branch plan plus metadata-only receipts.

const BRANCH_RECEIPT_SCHEMA: &str = "animus.session-branching.receipt.v1";
const HASH_HEX_LENGTH: usize = 64;
const MAX_SAFE_TEXT_BYTES: usize = 160;
const HASH_PART_SEPARATOR: char = '|';
const HASH_LENGTH_SEPARATOR: char = ':';
const EMPTY_OPTION_PART: &str = "none";
const EMPTY_FIELD_SUBJECT: &str = "empty";
const DUPLICATE_ENTRY_SUBJECT: &str = "duplicate_entry";
const MISSING_PARENT_SUBJECT: &str = "missing_parent";
const UNKNOWN_ENTRY_SUBJECT: &str = "unknown_entry";
const CYCLE_SUBJECT: &str = "cycle";
const SAFE_BRANCH_LABEL_STATUS: &str = "safe_label";
const OMITTED_BRANCH_LABEL_STATUS: &str = "omitted";

const FIELD_SOURCE_SESSION_HASH: &str = "source_session_hash";
const FIELD_TARGET_SESSION_HASH: &str = "target_session_hash";
const FIELD_SOURCE_HEAD_HASH: &str = "source_head_hash";
const FIELD_CUT_POINT_HASH: &str = "cut_point_hash";
const FIELD_EDITED_BODY_CLASS: &str = "edited_body_class";
const FIELD_BRANCH_LABEL: &str = "branch_label";
const FIELD_POLICY_HASH: &str = "policy_hash";
const FIELD_PROVENANCE_LABEL: &str = "provenance_label";
const FIELD_DESCRIPTOR_LABEL: &str = "copy_descriptor_label";
const FIELD_LEDGER_ENTRY_HASH: &str = "ledger_entry_hash";
const FIELD_PARENT_HASH: &str = "parent_hash";
const FIELD_PLANNER: &str = "session_branch_planner";
const FIELD_RECEIPT: &str = "session_branch_receipt";

const LEAK_MARKERS: &[LeakMarker] = &[
    LeakMarker {
        needle: "raw_prompt:",
        class: "raw_prompt",
    },
    LeakMarker {
        needle: "raw transcript",
        class: "raw_transcript",
    },
    LeakMarker {
        needle: "transcript:",
        class: "raw_transcript",
    },
    LeakMarker {
        needle: "provider_json",
        class: "provider_payload",
    },
    LeakMarker {
        needle: "provider payload",
        class: "provider_payload",
    },
    LeakMarker {
        needle: "tool_output:",
        class: "raw_tool_output",
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
        needle: "secret=",
        class: "credential",
    },
    LeakMarker {
        needle: "/home/",
        class: "full_path",
    },
    LeakMarker {
        needle: "host_database_row:",
        class: "host_database_row",
    },
];

type Text = String;
type Collection<T> = Vec<T>;

#[derive(Clone, Copy)]
struct LeakMarker {
    needle: &'static str,
    class: &'static str,
}

/// Session branch request operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SessionBranchOperation {
    /// Edit a prior ledger entry and preserve the original branch by forking.
    ForkFromMessage,
    /// Edit the current branch in place after explicit suffix-discard admission.
    EditInPlace,
    /// Duplicate the source session metadata into a new session identity.
    DuplicateSession,
    /// Update safe session or branch labels without replaying transcript bodies.
    LabelUpdate,
}

impl SessionBranchOperation {
    /// Returns the stable operation label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ForkFromMessage => "fork_from_message",
            Self::EditInPlace => "edit_in_place",
            Self::DuplicateSession => "duplicate_session",
            Self::LabelUpdate => "label_update",
        }
    }
}

/// Continuation target selected after branch planning.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SessionBranchContinuationTarget {
    /// Continue on the original source branch.
    Original,
    /// Continue on the newly forked or edited branch.
    New,
    /// Continue on the duplicated session.
    Duplicate,
}

impl SessionBranchContinuationTarget {
    /// Returns the stable continuation label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Original => "original_branch",
            Self::New => "new_branch",
            Self::Duplicate => "duplicate_branch",
        }
    }
}

/// Safe or unsafe descriptor class requested for branch copies.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum SessionCopyDescriptorClass {
    /// Safe session label metadata.
    SessionLabel,
    /// Safe recipe reference identity.
    RecipeReference,
    /// Safe extension label metadata.
    ExtensionLabel,
    /// Safe model profile label metadata.
    ModelProfileLabel,
    /// Safe policy label metadata.
    PolicyLabel,
    /// Bounded ledger reference by identity or hash.
    BoundedLedgerReference,
    /// Credential material or credential handle, which is never copied.
    Credential,
    /// Live process handle, which is never copied.
    LiveProcess,
    /// Provider session handle, which is never copied.
    ProviderSession,
    /// Raw transcript material, which is never copied.
    RawTranscript,
    /// Raw tool output material, which is never copied.
    RawToolOutput,
    /// Host database row, which is never copied.
    HostDatabaseRow,
}

impl SessionCopyDescriptorClass {
    /// Returns true when reusable logic may copy this descriptor class.
    #[must_use]
    pub const fn is_safe_copy(self) -> bool {
        matches!(
            self,
            Self::SessionLabel
                | Self::RecipeReference
                | Self::ExtensionLabel
                | Self::ModelProfileLabel
                | Self::PolicyLabel
                | Self::BoundedLedgerReference
        )
    }

    /// Returns the stable descriptor class label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SessionLabel => "session_label",
            Self::RecipeReference => "recipe_reference",
            Self::ExtensionLabel => "extension_label",
            Self::ModelProfileLabel => "model_profile_label",
            Self::PolicyLabel => "policy_label",
            Self::BoundedLedgerReference => "bounded_ledger_reference",
            Self::Credential => "credential",
            Self::LiveProcess => "live_process",
            Self::ProviderSession => "provider_session",
            Self::RawTranscript => "raw_transcript",
            Self::RawToolOutput => "raw_tool_output",
            Self::HostDatabaseRow => "host_database_row",
        }
    }
}

/// Branch-planning checkpoint evidence status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SessionBranchCheckpointStatus {
    /// Checkpoint or rollback evidence is available.
    Available,
    /// Checkpoint or rollback evidence is missing.
    Missing,
    /// No checkpoint evidence is required for this operation.
    NotRequired,
}

impl SessionBranchCheckpointStatus {
    /// Returns the stable checkpoint status label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Available => "available",
            Self::Missing => "missing",
            Self::NotRequired => "not_required",
        }
    }
}

/// Branch policy admission state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SessionBranchPolicyAdmission {
    /// Policy admits branch planning.
    Admitted,
    /// Policy denies branch planning.
    Denied,
}

impl SessionBranchPolicyAdmission {
    /// Returns the stable policy admission label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Admitted => "admitted",
            Self::Denied => "denied",
        }
    }
}

/// Final branch plan status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SessionBranchPlanStatus {
    /// Host may apply the plan using its own storage authority.
    Admitted,
    /// Host must not mutate storage for this request.
    Denied,
}

impl SessionBranchPlanStatus {
    /// Returns the stable plan status label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Admitted => "admitted",
            Self::Denied => "denied",
        }
    }
}

/// Branch validation error class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SessionBranchErrorKind {
    /// A required field was empty.
    EmptyField,
    /// A safe field exceeded the bounded length policy.
    FieldTooLong,
    /// A field contained raw or unsafe material.
    UnsafeText,
    /// A hash field was not a BLAKE3 hexadecimal digest.
    InvalidHash,
}

/// Safe branch validation error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionBranchError {
    kind: SessionBranchErrorKind,
    field: &'static str,
    subject: Text,
    reason: Text,
}

impl SessionBranchError {
    fn new(
        kind: SessionBranchErrorKind,
        field: &'static str,
        subject: impl Into<Text>,
        reason: impl Into<Text>,
    ) -> Self {
        Self {
            kind,
            field,
            subject: subject.into(),
            reason: reason.into(),
        }
    }

    /// Returns the stable error class.
    #[must_use]
    pub const fn kind(&self) -> SessionBranchErrorKind {
        self.kind
    }

    /// Returns the field that failed validation.
    #[must_use]
    pub const fn field(&self) -> &'static str {
        self.field
    }

    /// Returns a safe subject label.
    #[must_use]
    pub fn subject(&self) -> &str {
        self.subject.as_str()
    }

    /// Returns a safe diagnostic reason.
    #[must_use]
    pub fn reason(&self) -> &str {
        self.reason.as_str()
    }
}

impl std::fmt::Display for SessionBranchError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "session branch field `{}` failed ({:?}) for `{}`: {}",
            self.field, self.kind, self.subject, self.reason
        )
    }
}

impl std::error::Error for SessionBranchError {}

/// Branch planner diagnostic class.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum SessionBranchDiagnosticKind {
    /// Policy denied the request.
    PolicyDenied,
    /// Ambient storage or process authority was requested.
    AmbientAuthorityDenied,
    /// Source head was absent from the in-memory ledger snapshot.
    MissingSourceHead,
    /// Cut point was not reachable from the current source head.
    StaleCutPoint,
    /// Edit-in-place suffix discard was not admitted.
    SuffixDiscardDenied,
    /// Checkpoint or rollback evidence was required but missing.
    MissingCheckpointEvidence,
    /// Target session identity duplicates the source session identity.
    DuplicateTargetSession,
    /// Copy policy requested unsafe descriptor inheritance.
    UnsafeCopyDescriptor,
    /// Ledger snapshot had duplicate entries.
    DuplicateLedgerEntry,
    /// Ledger snapshot referenced an unknown parent.
    MissingLedgerParent,
    /// Ledger snapshot ancestry contained a cycle.
    LedgerCycle,
    /// Ledger ancestry path exceeded the bounded receipt span type.
    LedgerPathTooLong,
}

impl SessionBranchDiagnosticKind {
    /// Returns the stable diagnostic label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PolicyDenied => "policy_denied",
            Self::AmbientAuthorityDenied => "ambient_authority_denied",
            Self::MissingSourceHead => "missing_source_head",
            Self::StaleCutPoint => "stale_cut_point",
            Self::SuffixDiscardDenied => "suffix_discard_denied",
            Self::MissingCheckpointEvidence => "missing_checkpoint_evidence",
            Self::DuplicateTargetSession => "duplicate_target_session",
            Self::UnsafeCopyDescriptor => "unsafe_copy_descriptor",
            Self::DuplicateLedgerEntry => "duplicate_ledger_entry",
            Self::MissingLedgerParent => "missing_ledger_parent",
            Self::LedgerCycle => "ledger_cycle",
            Self::LedgerPathTooLong => "ledger_path_too_long",
        }
    }
}

/// Safe planner diagnostic.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionBranchDiagnostic {
    kind: SessionBranchDiagnosticKind,
    field: &'static str,
    detail: Text,
}

impl SessionBranchDiagnostic {
    fn new(kind: SessionBranchDiagnosticKind, field: &'static str, detail: impl Into<Text>) -> Self {
        Self {
            kind,
            field,
            detail: detail.into(),
        }
    }

    /// Returns the diagnostic class.
    #[must_use]
    pub const fn kind(&self) -> SessionBranchDiagnosticKind {
        self.kind
    }

    /// Returns the field associated with the diagnostic.
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

/// Borrowed descriptor requested for copying into a branch or duplicate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SessionCopyDescriptorDraft<'a> {
    /// Descriptor class.
    pub class: SessionCopyDescriptorClass,
    /// Safe descriptor label or hash.
    pub label: &'a str,
}

/// Validated copy descriptor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionCopyDescriptor {
    class: SessionCopyDescriptorClass,
    label: Text,
}

impl SessionCopyDescriptor {
    /// Builds a validated copy descriptor.
    ///
    /// # Errors
    ///
    /// Returns [`SessionBranchError`] when the descriptor label is empty,
    /// overlong, or contains unsafe raw material.
    pub fn new(draft: SessionCopyDescriptorDraft<'_>) -> Result<Self, SessionBranchError> {
        Ok(Self {
            class: draft.class,
            label: validate_safe_text(SafeFieldValue::new(FIELD_DESCRIPTOR_LABEL, draft.label))?,
        })
    }

    /// Returns the descriptor class.
    #[must_use]
    pub const fn class(&self) -> SessionCopyDescriptorClass {
        self.class
    }

    /// Returns the safe descriptor label.
    #[must_use]
    pub fn label(&self) -> &str {
        self.label.as_str()
    }
}

/// Borrowed branch request before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SessionBranchRequestDraft<'a> {
    /// Requested operation.
    pub operation: SessionBranchOperation,
    /// Source session identity hash.
    pub source_session_hash: &'a str,
    /// Target session identity hash.
    pub target_session_hash: &'a str,
    /// Source ledger head identity hash.
    pub source_head_hash: &'a str,
    /// Cut-point ledger identity hash.
    pub cut_point_hash: &'a str,
    /// Edited message body class or hash-only summary.
    pub edited_body_class: &'a str,
    /// Requested safe branch label.
    pub requested_branch_label: &'a str,
    /// Requested continuation target.
    pub continuation_target: SessionBranchContinuationTarget,
    /// Safe provenance label.
    pub provenance_label: &'a str,
    /// Descriptors requested for copying.
    pub copy_descriptors: &'a [SessionCopyDescriptorDraft<'a>],
    /// Whether the caller tried to request ambient storage authority.
    pub ambient_storage_requested: bool,
}

/// Validated branch request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionBranchRequest {
    operation: SessionBranchOperation,
    source_session_hash: Text,
    target_session_hash: Text,
    source_head_hash: Text,
    cut_point_hash: Text,
    edited_body_class: Text,
    requested_branch_label: Text,
    continuation_target: SessionBranchContinuationTarget,
    provenance_label: Text,
    copy_descriptors: Collection<SessionCopyDescriptor>,
    ambient_storage_requested: bool,
}

impl SessionBranchRequest {
    /// Builds a validated branch request DTO.
    ///
    /// # Errors
    ///
    /// Returns [`SessionBranchError`] when hashes are malformed or safe labels
    /// contain raw prompts, transcripts, provider payloads, credentials, full
    /// paths, raw tool outputs, or host database rows.
    pub fn new(draft: &SessionBranchRequestDraft<'_>) -> Result<Self, SessionBranchError> {
        let copy_descriptors = draft
            .copy_descriptors
            .iter()
            .map(|descriptor| SessionCopyDescriptor::new(*descriptor))
            .collect::<Result<Collection<_>, _>>()?;
        Ok(Self {
            operation: draft.operation,
            source_session_hash: validate_hash(SafeFieldValue::new(
                FIELD_SOURCE_SESSION_HASH,
                draft.source_session_hash,
            ))?,
            target_session_hash: validate_hash(SafeFieldValue::new(
                FIELD_TARGET_SESSION_HASH,
                draft.target_session_hash,
            ))?,
            source_head_hash: validate_hash(SafeFieldValue::new(FIELD_SOURCE_HEAD_HASH, draft.source_head_hash))?,
            cut_point_hash: validate_hash(SafeFieldValue::new(FIELD_CUT_POINT_HASH, draft.cut_point_hash))?,
            edited_body_class: validate_safe_text(SafeFieldValue::new(
                FIELD_EDITED_BODY_CLASS,
                draft.edited_body_class,
            ))?,
            requested_branch_label: validate_safe_text(SafeFieldValue::new(
                FIELD_BRANCH_LABEL,
                draft.requested_branch_label,
            ))?,
            continuation_target: draft.continuation_target,
            provenance_label: validate_safe_text(SafeFieldValue::new(FIELD_PROVENANCE_LABEL, draft.provenance_label))?,
            copy_descriptors,
            ambient_storage_requested: draft.ambient_storage_requested,
        })
    }

    /// Returns the requested operation.
    #[must_use]
    pub const fn operation(&self) -> SessionBranchOperation {
        self.operation
    }

    /// Returns the source session identity hash.
    #[must_use]
    pub fn source_session_hash(&self) -> &str {
        self.source_session_hash.as_str()
    }

    /// Returns the target session identity hash.
    #[must_use]
    pub fn target_session_hash(&self) -> &str {
        self.target_session_hash.as_str()
    }

    /// Returns the source head hash.
    #[must_use]
    pub fn source_head_hash(&self) -> &str {
        self.source_head_hash.as_str()
    }

    /// Returns the cut-point hash.
    #[must_use]
    pub fn cut_point_hash(&self) -> &str {
        self.cut_point_hash.as_str()
    }

    /// Returns the safe edited body class.
    #[must_use]
    pub fn edited_body_class(&self) -> &str {
        self.edited_body_class.as_str()
    }

    /// Returns the requested branch label.
    #[must_use]
    pub fn requested_branch_label(&self) -> &str {
        self.requested_branch_label.as_str()
    }

    /// Returns the continuation target.
    #[must_use]
    pub const fn continuation_target(&self) -> SessionBranchContinuationTarget {
        self.continuation_target
    }

    /// Returns the safe provenance label.
    #[must_use]
    pub fn provenance_label(&self) -> &str {
        self.provenance_label.as_str()
    }

    /// Returns descriptors requested for copy planning.
    #[must_use]
    pub fn copy_descriptors(&self) -> &[SessionCopyDescriptor] {
        self.copy_descriptors.as_slice()
    }

    /// Returns true when the request tried to use ambient storage authority.
    #[must_use]
    pub const fn ambient_storage_requested(&self) -> bool {
        self.ambient_storage_requested
    }
}

/// Borrowed policy inputs before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SessionBranchPolicyDraft<'a> {
    /// Policy identity hash.
    pub policy_hash: &'a str,
    /// Branch-policy admission state.
    pub admission: SessionBranchPolicyAdmission,
    /// Whether edit-in-place suffix discard is admitted.
    pub allow_suffix_discard: bool,
    /// Whether suffix discard requires checkpoint evidence.
    pub require_checkpoint_for_discard: bool,
    /// Whether unsafe copy descriptors deny the whole plan instead of only being omitted.
    pub deny_unsafe_copy: bool,
}

/// Validated policy inputs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionBranchPolicy {
    policy_hash: Text,
    admission: SessionBranchPolicyAdmission,
    allow_suffix_discard: bool,
    require_checkpoint_for_discard: bool,
    deny_unsafe_copy: bool,
}

impl SessionBranchPolicy {
    /// Builds validated policy inputs.
    ///
    /// # Errors
    ///
    /// Returns [`SessionBranchError`] when the policy hash is malformed.
    pub fn new(draft: &SessionBranchPolicyDraft<'_>) -> Result<Self, SessionBranchError> {
        Ok(Self {
            policy_hash: validate_hash(SafeFieldValue::new(FIELD_POLICY_HASH, draft.policy_hash))?,
            admission: draft.admission,
            allow_suffix_discard: draft.allow_suffix_discard,
            require_checkpoint_for_discard: draft.require_checkpoint_for_discard,
            deny_unsafe_copy: draft.deny_unsafe_copy,
        })
    }

    /// Returns policy hash.
    #[must_use]
    pub fn policy_hash(&self) -> &str {
        self.policy_hash.as_str()
    }

    /// Returns policy admission.
    #[must_use]
    pub const fn admission(&self) -> SessionBranchPolicyAdmission {
        self.admission
    }

    /// Returns true when suffix discard is admitted.
    #[must_use]
    pub const fn allow_suffix_discard(&self) -> bool {
        self.allow_suffix_discard
    }

    /// Returns true when suffix discard requires checkpoint evidence.
    #[must_use]
    pub const fn require_checkpoint_for_discard(&self) -> bool {
        self.require_checkpoint_for_discard
    }

    /// Returns true when unsafe copy descriptors deny the plan.
    #[must_use]
    pub const fn deny_unsafe_copy(&self) -> bool {
        self.deny_unsafe_copy
    }
}

/// Borrowed ledger entry before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SessionLedgerEntryDraft<'a> {
    /// Ledger entry identity hash.
    pub entry_hash: &'a str,
    /// Optional parent entry hash.
    pub parent_hash: Option<&'a str>,
    /// Safe body class for the entry.
    pub body_class: &'a str,
}

/// Validated in-memory ledger entry summary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionLedgerEntry {
    entry_hash: Text,
    parent_hash: Option<Text>,
    body_class: Text,
}

impl SessionLedgerEntry {
    /// Builds a validated ledger entry summary.
    ///
    /// # Errors
    ///
    /// Returns [`SessionBranchError`] when hashes are malformed or body class
    /// metadata contains unsafe raw material.
    pub fn new(draft: &SessionLedgerEntryDraft<'_>) -> Result<Self, SessionBranchError> {
        Ok(Self {
            entry_hash: validate_hash(SafeFieldValue::new(FIELD_LEDGER_ENTRY_HASH, draft.entry_hash))?,
            parent_hash: draft
                .parent_hash
                .map(|hash| validate_hash(SafeFieldValue::new(FIELD_PARENT_HASH, hash)))
                .transpose()?,
            body_class: validate_safe_text(SafeFieldValue::new(FIELD_EDITED_BODY_CLASS, draft.body_class))?,
        })
    }

    /// Returns the entry hash.
    #[must_use]
    pub fn entry_hash(&self) -> &str {
        self.entry_hash.as_str()
    }

    /// Returns the parent hash, when present.
    #[must_use]
    pub fn parent_hash(&self) -> Option<&str> {
        self.parent_hash.as_deref()
    }

    /// Returns the safe body class.
    #[must_use]
    pub fn body_class(&self) -> &str {
        self.body_class.as_str()
    }
}

/// Pure planner input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SessionBranchPlannerInput<'a> {
    /// Validated branch request.
    pub request: &'a SessionBranchRequest,
    /// Validated policy inputs.
    pub policy: &'a SessionBranchPolicy,
    /// Already-loaded in-memory ledger snapshot.
    pub ledger: &'a [SessionLedgerEntry],
    /// Host-supplied checkpoint evidence status.
    pub checkpoint_status: SessionBranchCheckpointStatus,
}

/// Safe session branch receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionBranchReceipt {
    operation: SessionBranchOperation,
    status: SessionBranchPlanStatus,
    source_session_hash: Text,
    target_session_hash: Text,
    source_head_hash: Text,
    cut_point_hash: Text,
    suffix_span_count: u32,
    branch_label_status: Text,
    copied_descriptor_classes: Collection<SessionCopyDescriptorClass>,
    omitted_descriptor_classes: Collection<SessionCopyDescriptorClass>,
    policy_hash: Text,
    checkpoint_status: SessionBranchCheckpointStatus,
    continuation_target: SessionBranchContinuationTarget,
    diagnostics: Collection<SessionBranchDiagnosticKind>,
    receipt_hash: Text,
}

impl SessionBranchReceipt {
    /// Returns operation label.
    #[must_use]
    pub const fn operation(&self) -> SessionBranchOperation {
        self.operation
    }

    /// Returns plan status.
    #[must_use]
    pub const fn status(&self) -> SessionBranchPlanStatus {
        self.status
    }

    /// Returns source session hash.
    #[must_use]
    pub fn source_session_hash(&self) -> &str {
        self.source_session_hash.as_str()
    }

    /// Returns target session hash.
    #[must_use]
    pub fn target_session_hash(&self) -> &str {
        self.target_session_hash.as_str()
    }

    /// Returns source head hash.
    #[must_use]
    pub fn source_head_hash(&self) -> &str {
        self.source_head_hash.as_str()
    }

    /// Returns cut-point hash.
    #[must_use]
    pub fn cut_point_hash(&self) -> &str {
        self.cut_point_hash.as_str()
    }

    /// Returns suffix span count.
    #[must_use]
    pub const fn suffix_span_count(&self) -> u32 {
        self.suffix_span_count
    }

    /// Returns branch-label status.
    #[must_use]
    pub fn branch_label_status(&self) -> &str {
        self.branch_label_status.as_str()
    }

    /// Returns copied descriptor classes.
    #[must_use]
    pub fn copied_descriptor_classes(&self) -> &[SessionCopyDescriptorClass] {
        self.copied_descriptor_classes.as_slice()
    }

    /// Returns omitted descriptor classes.
    #[must_use]
    pub fn omitted_descriptor_classes(&self) -> &[SessionCopyDescriptorClass] {
        self.omitted_descriptor_classes.as_slice()
    }

    /// Returns policy hash.
    #[must_use]
    pub fn policy_hash(&self) -> &str {
        self.policy_hash.as_str()
    }

    /// Returns checkpoint status.
    #[must_use]
    pub const fn checkpoint_status(&self) -> SessionBranchCheckpointStatus {
        self.checkpoint_status
    }

    /// Returns continuation target.
    #[must_use]
    pub const fn continuation_target(&self) -> SessionBranchContinuationTarget {
        self.continuation_target
    }

    /// Returns diagnostic classes.
    #[must_use]
    pub fn diagnostics(&self) -> &[SessionBranchDiagnosticKind] {
        self.diagnostics.as_slice()
    }

    /// Returns receipt hash.
    #[must_use]
    pub fn receipt_hash(&self) -> &str {
        self.receipt_hash.as_str()
    }
}

/// Pure session branch plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionBranchPlan {
    status: SessionBranchPlanStatus,
    operation: SessionBranchOperation,
    target_parent_hash: Option<Text>,
    suffix_span_count: u32,
    copied_descriptor_classes: Collection<SessionCopyDescriptorClass>,
    omitted_descriptor_classes: Collection<SessionCopyDescriptorClass>,
    diagnostics: Collection<SessionBranchDiagnostic>,
    receipt: SessionBranchReceipt,
}

impl SessionBranchPlan {
    /// Returns plan status.
    #[must_use]
    pub const fn status(&self) -> SessionBranchPlanStatus {
        self.status
    }

    /// Returns requested operation.
    #[must_use]
    pub const fn operation(&self) -> SessionBranchOperation {
        self.operation
    }

    /// Returns target parent hash when planning was admitted.
    #[must_use]
    pub fn target_parent_hash(&self) -> Option<&str> {
        self.target_parent_hash.as_deref()
    }

    /// Returns suffix span count after the cut point.
    #[must_use]
    pub const fn suffix_span_count(&self) -> u32 {
        self.suffix_span_count
    }

    /// Returns copied descriptor classes.
    #[must_use]
    pub fn copied_descriptor_classes(&self) -> &[SessionCopyDescriptorClass] {
        self.copied_descriptor_classes.as_slice()
    }

    /// Returns omitted descriptor classes.
    #[must_use]
    pub fn omitted_descriptor_classes(&self) -> &[SessionCopyDescriptorClass] {
        self.omitted_descriptor_classes.as_slice()
    }

    /// Returns safe diagnostics.
    #[must_use]
    pub fn diagnostics(&self) -> &[SessionBranchDiagnostic] {
        self.diagnostics.as_slice()
    }

    /// Returns metadata-only receipt.
    #[must_use]
    pub const fn receipt(&self) -> &SessionBranchReceipt {
        &self.receipt
    }
}

/// Plans a host-owned session branch operation from already-loaded inputs.
#[must_use]
pub fn plan_session_branch(input: SessionBranchPlannerInput<'_>) -> SessionBranchPlan {
    // r[impl agent_harness_runtime.goose_session_branching.pure_planner]
    debug_assert!(!input.request.source_session_hash().is_empty());
    debug_assert!(!input.policy.policy_hash().is_empty());
    let mut diagnostics = Collection::new();
    apply_policy_diagnostics(input, &mut diagnostics);
    let (index, ledger_diagnostics) = ledger_index(input.ledger);
    diagnostics.extend(ledger_diagnostics);
    let suffix_span_count = suffix_span_count(input.request, &index, &mut diagnostics);
    apply_operation_diagnostics(input, suffix_span_count, &mut diagnostics);
    let (copied_descriptor_classes, omitted_descriptor_classes) =
        copy_descriptor_classes(input.request, input.policy, &mut diagnostics);
    let status = if diagnostics.is_empty() {
        SessionBranchPlanStatus::Admitted
    } else {
        SessionBranchPlanStatus::Denied
    };
    let target_parent_hash = admitted_target_parent(input.request, status);
    let receipt = branch_receipt(ReceiptInput {
        request: input.request,
        policy: input.policy,
        status,
        suffix_span_count,
        checkpoint_status: input.checkpoint_status,
        copied_descriptor_classes: copied_descriptor_classes.as_slice(),
        omitted_descriptor_classes: omitted_descriptor_classes.as_slice(),
        diagnostics: diagnostics.as_slice(),
    });
    SessionBranchPlan {
        status,
        operation: input.request.operation(),
        target_parent_hash,
        suffix_span_count,
        copied_descriptor_classes,
        omitted_descriptor_classes,
        diagnostics,
        receipt,
    }
}

/// Validates that receipt text does not leak raw session material.
///
/// # Errors
///
/// Returns [`SessionBranchError`] when the text contains raw prompts,
/// transcripts, provider payloads, credentials, full paths, raw tool outputs,
/// or host database rows.
pub fn validate_session_branch_receipt_text(text: &str) -> Result<(), SessionBranchError> {
    if let Some(marker) = contains_unsafe_material(text) {
        return Err(SessionBranchError::new(
            SessionBranchErrorKind::UnsafeText,
            FIELD_RECEIPT,
            marker.class,
            "receipt text contains unsafe session material",
        ));
    }
    Ok(())
}

#[derive(Clone, Copy)]
struct ReceiptInput<'a> {
    request: &'a SessionBranchRequest,
    policy: &'a SessionBranchPolicy,
    status: SessionBranchPlanStatus,
    suffix_span_count: u32,
    checkpoint_status: SessionBranchCheckpointStatus,
    copied_descriptor_classes: &'a [SessionCopyDescriptorClass],
    omitted_descriptor_classes: &'a [SessionCopyDescriptorClass],
    diagnostics: &'a [SessionBranchDiagnostic],
}

fn apply_policy_diagnostics(
    input: SessionBranchPlannerInput<'_>,
    diagnostics: &mut Collection<SessionBranchDiagnostic>,
) {
    if input.policy.admission() == SessionBranchPolicyAdmission::Denied {
        diagnostics.push(SessionBranchDiagnostic::new(
            SessionBranchDiagnosticKind::PolicyDenied,
            FIELD_POLICY_HASH,
            "branch policy denied request",
        ));
    }
    if input.request.ambient_storage_requested() {
        diagnostics.push(SessionBranchDiagnostic::new(
            SessionBranchDiagnosticKind::AmbientAuthorityDenied,
            FIELD_PLANNER,
            "request attempted ambient storage authority",
        ));
    }
}

fn apply_operation_diagnostics(
    input: SessionBranchPlannerInput<'_>,
    suffix_span_count: u32,
    diagnostics: &mut Collection<SessionBranchDiagnostic>,
) {
    if input.request.source_session_hash() == input.request.target_session_hash() {
        diagnostics.push(SessionBranchDiagnostic::new(
            SessionBranchDiagnosticKind::DuplicateTargetSession,
            FIELD_TARGET_SESSION_HASH,
            "target session must differ from source session",
        ));
    }
    if input.request.operation() == SessionBranchOperation::EditInPlace {
        apply_edit_in_place_diagnostics(input, suffix_span_count, diagnostics);
    }
}

fn apply_edit_in_place_diagnostics(
    input: SessionBranchPlannerInput<'_>,
    suffix_span_count: u32,
    diagnostics: &mut Collection<SessionBranchDiagnostic>,
) {
    // r[impl agent_harness_runtime.goose_session_branching.edit_in_place]
    debug_assert!(!input.request.cut_point_hash().is_empty());
    debug_assert!(!input.request.source_head_hash().is_empty());
    if suffix_span_count == 0 {
        return;
    }
    if !input.policy.allow_suffix_discard() {
        diagnostics.push(SessionBranchDiagnostic::new(
            SessionBranchDiagnosticKind::SuffixDiscardDenied,
            FIELD_CUT_POINT_HASH,
            "suffix discard requires explicit policy admission",
        ));
    }
    if input.policy.require_checkpoint_for_discard()
        && input.checkpoint_status != SessionBranchCheckpointStatus::Available
    {
        diagnostics.push(SessionBranchDiagnostic::new(
            SessionBranchDiagnosticKind::MissingCheckpointEvidence,
            FIELD_CUT_POINT_HASH,
            "suffix discard requires checkpoint or rollback evidence",
        ));
    }
}

fn ledger_index(
    ledger: &[SessionLedgerEntry],
) -> (std::collections::BTreeMap<&str, &SessionLedgerEntry>, Collection<SessionBranchDiagnostic>) {
    let mut index = std::collections::BTreeMap::new();
    let mut diagnostics = Collection::with_capacity(ledger.len());
    for entry in ledger {
        if index.insert(entry.entry_hash(), entry).is_some() {
            diagnostics.push(SessionBranchDiagnostic::new(
                SessionBranchDiagnosticKind::DuplicateLedgerEntry,
                FIELD_LEDGER_ENTRY_HASH,
                DUPLICATE_ENTRY_SUBJECT,
            ));
        }
    }
    for entry in ledger {
        if let Some(parent_hash) = entry.parent_hash() {
            if !index.contains_key(parent_hash) {
                diagnostics.push(SessionBranchDiagnostic::new(
                    SessionBranchDiagnosticKind::MissingLedgerParent,
                    FIELD_PARENT_HASH,
                    MISSING_PARENT_SUBJECT,
                ));
            }
        }
    }
    assert!(index.len() <= ledger.len(), "ledger index cannot contain more unique entries than source ledger");
    assert!(
        diagnostics.iter().all(|diagnostic| !diagnostic.field().is_empty()),
        "ledger diagnostics must identify a field"
    );
    (index, diagnostics)
}

fn suffix_span_count(
    request: &SessionBranchRequest,
    index: &std::collections::BTreeMap<&str, &SessionLedgerEntry>,
    diagnostics: &mut Collection<SessionBranchDiagnostic>,
) -> u32 {
    debug_assert!(!request.source_head_hash().is_empty());
    debug_assert!(!request.cut_point_hash().is_empty());
    let Some(path) = path_from_head(request.source_head_hash(), index, diagnostics) else {
        return 0;
    };
    let Some(cut_position) = path.iter().position(|entry| entry.entry_hash() == request.cut_point_hash()) else {
        diagnostics.push(SessionBranchDiagnostic::new(
            SessionBranchDiagnosticKind::StaleCutPoint,
            FIELD_CUT_POINT_HASH,
            "cut point is not reachable from source head",
        ));
        return 0;
    };
    let Ok(span_count) = u32::try_from(cut_position) else {
        diagnostics.push(SessionBranchDiagnostic::new(
            SessionBranchDiagnosticKind::LedgerPathTooLong,
            FIELD_LEDGER_ENTRY_HASH,
            "ledger path exceeds bounded span count",
        ));
        return 0;
    };
    span_count
}

fn path_from_head<'a>(
    source_head_hash: &str,
    index: &std::collections::BTreeMap<&'a str, &'a SessionLedgerEntry>,
    diagnostics: &mut Collection<SessionBranchDiagnostic>,
) -> Option<Collection<&'a SessionLedgerEntry>> {
    debug_assert!(!source_head_hash.is_empty());
    debug_assert!(diagnostics.capacity() >= diagnostics.len());
    let mut path = Collection::with_capacity(index.len());
    diagnostics.reserve(index.len());
    let mut seen = std::collections::BTreeSet::new();
    let mut next_hash = Some(source_head_hash);
    while let Some(current_hash) = next_hash {
        if !seen.insert(current_hash) {
            diagnostics.push(SessionBranchDiagnostic::new(
                SessionBranchDiagnosticKind::LedgerCycle,
                FIELD_LEDGER_ENTRY_HASH,
                CYCLE_SUBJECT,
            ));
            return None;
        }
        let Some(entry) = index.get(current_hash).copied() else {
            diagnostics.push(SessionBranchDiagnostic::new(
                SessionBranchDiagnosticKind::MissingSourceHead,
                FIELD_SOURCE_HEAD_HASH,
                UNKNOWN_ENTRY_SUBJECT,
            ));
            return None;
        };
        next_hash = entry.parent_hash();
        path.push(entry);
    }
    Some(path)
}

fn copy_descriptor_classes(
    request: &SessionBranchRequest,
    policy: &SessionBranchPolicy,
    diagnostics: &mut Collection<SessionBranchDiagnostic>,
) -> (Collection<SessionCopyDescriptorClass>, Collection<SessionCopyDescriptorClass>) {
    // r[impl agent_harness_runtime.goose_session_branching.copy_boundaries]
    debug_assert!(!request.source_session_hash().is_empty());
    debug_assert!(!policy.policy_hash().is_empty());
    let mut copied = std::collections::BTreeSet::new();
    let mut omitted = std::collections::BTreeSet::new();
    let mut has_unsafe_descriptor = false;
    for descriptor in request.copy_descriptors() {
        let class = descriptor.class();
        if class.is_safe_copy() {
            copied.insert(class);
        } else {
            omitted.insert(class);
            has_unsafe_descriptor = true;
        }
    }
    if has_unsafe_descriptor && policy.deny_unsafe_copy() {
        diagnostics.push(SessionBranchDiagnostic::new(
            SessionBranchDiagnosticKind::UnsafeCopyDescriptor,
            FIELD_DESCRIPTOR_LABEL,
            "copy policy requested unsafe descriptor inheritance",
        ));
    }
    (copied.into_iter().collect(), omitted.into_iter().collect())
}

fn admitted_target_parent(request: &SessionBranchRequest, status: SessionBranchPlanStatus) -> Option<Text> {
    if status == SessionBranchPlanStatus::Denied {
        return None;
    }
    let hash = match request.operation() {
        SessionBranchOperation::ForkFromMessage | SessionBranchOperation::EditInPlace => request.cut_point_hash(),
        SessionBranchOperation::DuplicateSession | SessionBranchOperation::LabelUpdate => request.source_head_hash(),
    };
    Some(hash.to_owned())
}

fn branch_receipt(input: ReceiptInput<'_>) -> SessionBranchReceipt {
    // r[impl agent_harness_runtime.goose_session_branching.receipts]
    debug_assert!(!input.request.source_session_hash().is_empty());
    debug_assert!(!input.policy.policy_hash().is_empty());
    let branch_label_status = if input.request.requested_branch_label().is_empty() {
        OMITTED_BRANCH_LABEL_STATUS.to_owned()
    } else {
        SAFE_BRANCH_LABEL_STATUS.to_owned()
    };
    let diagnostics = input.diagnostics.iter().map(SessionBranchDiagnostic::kind).collect::<Collection<_>>();
    let receipt_hash = branch_receipt_hash(ReceiptHashInput {
        request: input.request,
        policy: input.policy,
        status: input.status,
        suffix_span_count: input.suffix_span_count,
        branch_label_status: branch_label_status.as_str(),
        checkpoint_status: input.checkpoint_status,
        copied_descriptor_classes: input.copied_descriptor_classes,
        omitted_descriptor_classes: input.omitted_descriptor_classes,
        diagnostics: diagnostics.as_slice(),
    });
    SessionBranchReceipt {
        operation: input.request.operation(),
        status: input.status,
        source_session_hash: input.request.source_session_hash().to_owned(),
        target_session_hash: input.request.target_session_hash().to_owned(),
        source_head_hash: input.request.source_head_hash().to_owned(),
        cut_point_hash: input.request.cut_point_hash().to_owned(),
        suffix_span_count: input.suffix_span_count,
        branch_label_status,
        copied_descriptor_classes: input.copied_descriptor_classes.to_vec(),
        omitted_descriptor_classes: input.omitted_descriptor_classes.to_vec(),
        policy_hash: input.policy.policy_hash().to_owned(),
        checkpoint_status: input.checkpoint_status,
        continuation_target: input.request.continuation_target(),
        diagnostics,
        receipt_hash,
    }
}

#[derive(Clone, Copy)]
struct ReceiptHashInput<'a> {
    request: &'a SessionBranchRequest,
    policy: &'a SessionBranchPolicy,
    status: SessionBranchPlanStatus,
    suffix_span_count: u32,
    branch_label_status: &'a str,
    checkpoint_status: SessionBranchCheckpointStatus,
    copied_descriptor_classes: &'a [SessionCopyDescriptorClass],
    omitted_descriptor_classes: &'a [SessionCopyDescriptorClass],
    diagnostics: &'a [SessionBranchDiagnosticKind],
}

fn branch_receipt_hash(input: ReceiptHashInput<'_>) -> Text {
    debug_assert!(!input.request.source_session_hash().is_empty());
    debug_assert!(!input.policy.policy_hash().is_empty());
    let mut normalized = Text::from(BRANCH_RECEIPT_SCHEMA);
    push_hash_part(&mut normalized, input.request.operation().as_str());
    push_hash_part(&mut normalized, input.status.as_str());
    push_hash_part(&mut normalized, input.request.source_session_hash());
    push_hash_part(&mut normalized, input.request.target_session_hash());
    push_hash_part(&mut normalized, input.request.source_head_hash());
    push_hash_part(&mut normalized, input.request.cut_point_hash());
    push_hash_part(&mut normalized, &input.suffix_span_count.to_string());
    push_hash_part(&mut normalized, input.branch_label_status);
    for class in input.copied_descriptor_classes {
        push_hash_part(&mut normalized, class.as_str());
    }
    push_hash_part(&mut normalized, EMPTY_OPTION_PART);
    for class in input.omitted_descriptor_classes {
        push_hash_part(&mut normalized, class.as_str());
    }
    push_hash_part(&mut normalized, EMPTY_OPTION_PART);
    push_hash_part(&mut normalized, input.policy.policy_hash());
    push_hash_part(&mut normalized, input.checkpoint_status.as_str());
    push_hash_part(&mut normalized, input.request.continuation_target().as_str());
    for diagnostic in input.diagnostics {
        push_hash_part(&mut normalized, diagnostic.as_str());
    }
    blake3::hash(normalized.as_bytes()).to_hex().to_string()
}

fn push_hash_part(normalized: &mut Text, part: &str) {
    normalized.push(HASH_PART_SEPARATOR);
    normalized.push_str(&part.len().to_string());
    normalized.push(HASH_LENGTH_SEPARATOR);
    normalized.push_str(part);
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

fn validate_hash(input: SafeFieldValue<'_>) -> Result<Text, SessionBranchError> {
    let text = validate_safe_text(input)?;
    if text.len() != HASH_HEX_LENGTH || !text.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(SessionBranchError::new(
            SessionBranchErrorKind::InvalidHash,
            input.field,
            EMPTY_FIELD_SUBJECT,
            "value must be a BLAKE3 hexadecimal digest",
        ));
    }
    Ok(text)
}

fn validate_safe_text(input: SafeFieldValue<'_>) -> Result<Text, SessionBranchError> {
    debug_assert!(!input.field.is_empty());
    if input.value.is_empty() {
        return Err(SessionBranchError::new(
            SessionBranchErrorKind::EmptyField,
            input.field,
            EMPTY_FIELD_SUBJECT,
            "value must not be empty",
        ));
    }
    if input.value.len() > MAX_SAFE_TEXT_BYTES {
        return Err(SessionBranchError::new(
            SessionBranchErrorKind::FieldTooLong,
            input.field,
            EMPTY_FIELD_SUBJECT,
            "value exceeds safe text bound",
        ));
    }
    if let Some(marker) = contains_unsafe_material(input.value) {
        return Err(SessionBranchError::new(
            SessionBranchErrorKind::UnsafeText,
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

#[cfg(test)]
mod tests {
    use super::*;

    const HASH_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const HASH_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const HASH_C: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
    const HASH_D: &str = "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";
    const HASH_E: &str = "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";
    const HASH_F: &str = "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";
    const SAFE_BODY_CLASS: &str = "redacted_user_message";
    const SAFE_BRANCH_LABEL: &str = "branch-redacted-summary";
    const SAFE_PROVENANCE: &str = "goose-session-branching";
    const SAFE_DESCRIPTOR_LABEL: &str = "safe-session-descriptor";
    const RAW_PROMPT: &str = "RAW_PROMPT: edit this private transcript";
    const RAW_TRANSCRIPT: &str = "raw transcript body leaked";
    const EXPECTED_SUFFIX_COUNT: u32 = 2;

    #[test]
    fn fork_branch_plan_preserves_source_branch_and_receipt() {
        // r[verify agent_harness_runtime.goose_session_branching.requests.fork]
        // r[verify agent_harness_runtime.goose_session_branching.pure_planner.deterministic]
        let request =
            request(SessionBranchOperation::ForkFromMessage, HASH_B, false).expect("fork request should validate");
        let policy = admitted_policy(DEFAULT_POLICY_FIXTURE).expect("policy should validate");
        let plan = plan_session_branch(SessionBranchPlannerInput {
            request: &request,
            policy: &policy,
            ledger: ledger().as_slice(),
            checkpoint_status: SessionBranchCheckpointStatus::NotRequired,
        });

        assert_eq!(plan.status(), SessionBranchPlanStatus::Admitted);
        assert_eq!(plan.target_parent_hash(), Some(HASH_B));
        assert_eq!(plan.suffix_span_count(), EXPECTED_SUFFIX_COUNT);
        assert_eq!(plan.copied_descriptor_classes(), [SessionCopyDescriptorClass::SessionLabel]);
        assert_eq!(plan.receipt().status(), SessionBranchPlanStatus::Admitted);
        assert_eq!(plan.receipt().receipt_hash().len(), HASH_HEX_LENGTH);
    }

    #[test]
    fn duplicate_plan_copies_safe_descriptors_and_omits_live_state() {
        // r[verify agent_harness_runtime.goose_session_branching.copy_boundaries.omits_live_state]
        let request = duplicate_request(false).expect("duplicate request should validate");
        let policy = admitted_policy(DEFAULT_POLICY_FIXTURE).expect("policy should validate");
        let plan = plan_session_branch(SessionBranchPlannerInput {
            request: &request,
            policy: &policy,
            ledger: ledger().as_slice(),
            checkpoint_status: SessionBranchCheckpointStatus::NotRequired,
        });

        assert_eq!(plan.status(), SessionBranchPlanStatus::Admitted);
        assert_eq!(plan.target_parent_hash(), Some(HASH_D));
        assert!(plan.copied_descriptor_classes().contains(&SessionCopyDescriptorClass::PolicyLabel));
        assert!(plan.omitted_descriptor_classes().contains(&SessionCopyDescriptorClass::ProviderSession));
        assert!(plan.receipt().omitted_descriptor_classes().contains(&SessionCopyDescriptorClass::ProviderSession));
    }

    #[test]
    fn label_update_records_safe_branch_label() {
        // r[verify agent_harness_runtime.goose_session_branching.verification]
        let request =
            request(SessionBranchOperation::LabelUpdate, HASH_D, false).expect("label request should validate");
        let policy = admitted_policy(DEFAULT_POLICY_FIXTURE).expect("policy should validate");
        let plan = plan_session_branch(SessionBranchPlannerInput {
            request: &request,
            policy: &policy,
            ledger: ledger().as_slice(),
            checkpoint_status: SessionBranchCheckpointStatus::NotRequired,
        });

        assert_eq!(plan.status(), SessionBranchPlanStatus::Admitted);
        assert_eq!(plan.target_parent_hash(), Some(HASH_D));
        assert_eq!(plan.receipt().branch_label_status(), SAFE_BRANCH_LABEL_STATUS);
    }

    #[test]
    fn stale_cutpoint_and_suffix_discard_fail_closed() {
        // r[verify agent_harness_runtime.goose_session_branching.edit_in_place.stale]
        let stale_request =
            request(SessionBranchOperation::EditInPlace, HASH_E, false).expect("stale request should validate");
        let policy = admitted_policy(PolicyFixture {
            require_checkpoint_for_discard: true,
            ..DEFAULT_POLICY_FIXTURE
        })
        .expect("policy should validate");
        let stale_plan = plan_session_branch(SessionBranchPlannerInput {
            request: &stale_request,
            policy: &policy,
            ledger: ledger().as_slice(),
            checkpoint_status: SessionBranchCheckpointStatus::Available,
        });

        assert_eq!(stale_plan.status(), SessionBranchPlanStatus::Denied);
        assert_diagnostic(&stale_plan, SessionBranchDiagnosticKind::StaleCutPoint);

        let discard_request =
            request(SessionBranchOperation::EditInPlace, HASH_B, false).expect("discard request should validate");
        let discard_plan = plan_session_branch(SessionBranchPlannerInput {
            request: &discard_request,
            policy: &policy,
            ledger: ledger().as_slice(),
            checkpoint_status: SessionBranchCheckpointStatus::Missing,
        });

        assert_eq!(discard_plan.status(), SessionBranchPlanStatus::Denied);
        assert_diagnostic(&discard_plan, SessionBranchDiagnosticKind::SuffixDiscardDenied);
        assert_diagnostic(&discard_plan, SessionBranchDiagnosticKind::MissingCheckpointEvidence);
    }

    #[test]
    fn unsafe_body_copy_identity_and_ambient_authority_are_rejected() {
        // r[verify agent_harness_runtime.goose_session_branching.verification]
        let unsafe_body = SessionBranchRequest::new(&SessionBranchRequestDraft {
            edited_body_class: RAW_PROMPT,
            ..request_draft(SessionBranchOperation::ForkFromMessage, HASH_B, false)
        })
        .expect_err("raw prompt body class should fail validation");
        assert_eq!(unsafe_body.kind(), SessionBranchErrorKind::UnsafeText);

        let duplicate_identity = duplicate_request(true).expect("duplicate request should validate");
        let strict_policy = admitted_policy(PolicyFixture {
            deny_unsafe_copy: true,
            ..DEFAULT_POLICY_FIXTURE
        })
        .expect("policy should validate");
        let duplicate_plan = plan_session_branch(SessionBranchPlannerInput {
            request: &duplicate_identity,
            policy: &strict_policy,
            ledger: ledger().as_slice(),
            checkpoint_status: SessionBranchCheckpointStatus::NotRequired,
        });

        assert_eq!(duplicate_plan.status(), SessionBranchPlanStatus::Denied);
        assert_diagnostic(&duplicate_plan, SessionBranchDiagnosticKind::DuplicateTargetSession);
        assert_diagnostic(&duplicate_plan, SessionBranchDiagnosticKind::UnsafeCopyDescriptor);

        let ambient = request(SessionBranchOperation::ForkFromMessage, HASH_B, true)
            .expect("ambient request should validate before planning");
        let policy = admitted_policy(DEFAULT_POLICY_FIXTURE).expect("policy should validate");
        let ambient_plan = plan_session_branch(SessionBranchPlannerInput {
            request: &ambient,
            policy: &policy,
            ledger: ledger().as_slice(),
            checkpoint_status: SessionBranchCheckpointStatus::NotRequired,
        });
        assert_diagnostic(&ambient_plan, SessionBranchDiagnosticKind::AmbientAuthorityDenied);
    }

    #[test]
    fn receipt_leak_fixtures_are_rejected() {
        // r[verify agent_harness_runtime.goose_session_branching.receipts.rejects_leaks]
        for fixture in [RAW_PROMPT, RAW_TRANSCRIPT, "provider payload", "authorization: bearer"] {
            let error = validate_session_branch_receipt_text(fixture).expect_err("receipt leak fixture should fail");
            assert_eq!(error.kind(), SessionBranchErrorKind::UnsafeText);
        }
    }

    fn assert_diagnostic(plan: &SessionBranchPlan, expected: SessionBranchDiagnosticKind) {
        assert!(
            plan.diagnostics().iter().any(|diagnostic| diagnostic.kind() == expected),
            "expected diagnostic {expected:?} in {:?}",
            plan.diagnostics()
        );
    }

    fn ledger() -> Collection<SessionLedgerEntry> {
        [
            SessionLedgerEntryDraft {
                entry_hash: HASH_A,
                parent_hash: None,
                body_class: SAFE_BODY_CLASS,
            },
            SessionLedgerEntryDraft {
                entry_hash: HASH_B,
                parent_hash: Some(HASH_A),
                body_class: SAFE_BODY_CLASS,
            },
            SessionLedgerEntryDraft {
                entry_hash: HASH_C,
                parent_hash: Some(HASH_B),
                body_class: SAFE_BODY_CLASS,
            },
            SessionLedgerEntryDraft {
                entry_hash: HASH_D,
                parent_hash: Some(HASH_C),
                body_class: SAFE_BODY_CLASS,
            },
        ]
        .iter()
        .map(|draft| SessionLedgerEntry::new(draft).expect("ledger draft should validate"))
        .collect()
    }

    fn request(
        operation: SessionBranchOperation,
        cut_point_hash: &str,
        ambient_storage_requested: bool,
    ) -> Result<SessionBranchRequest, SessionBranchError> {
        SessionBranchRequest::new(&request_draft(operation, cut_point_hash, ambient_storage_requested))
    }

    fn duplicate_request(duplicate_identity: bool) -> Result<SessionBranchRequest, SessionBranchError> {
        debug_assert_ne!(HASH_E, HASH_F);
        debug_assert!(!HASH_D.is_empty());
        let target_session_hash = if duplicate_identity { HASH_F } else { HASH_E };
        SessionBranchRequest::new(&SessionBranchRequestDraft {
            operation: SessionBranchOperation::DuplicateSession,
            source_session_hash: HASH_F,
            target_session_hash,
            source_head_hash: HASH_D,
            cut_point_hash: HASH_D,
            edited_body_class: SAFE_BODY_CLASS,
            requested_branch_label: SAFE_BRANCH_LABEL,
            continuation_target: SessionBranchContinuationTarget::Duplicate,
            provenance_label: SAFE_PROVENANCE,
            copy_descriptors: &[
                SessionCopyDescriptorDraft {
                    class: SessionCopyDescriptorClass::PolicyLabel,
                    label: SAFE_DESCRIPTOR_LABEL,
                },
                SessionCopyDescriptorDraft {
                    class: SessionCopyDescriptorClass::ProviderSession,
                    label: SAFE_DESCRIPTOR_LABEL,
                },
            ],
            ambient_storage_requested: false,
        })
    }

    fn request_draft(
        operation: SessionBranchOperation,
        cut_point_hash: &str,
        ambient_storage_requested: bool,
    ) -> SessionBranchRequestDraft<'_> {
        SessionBranchRequestDraft {
            operation,
            source_session_hash: HASH_E,
            target_session_hash: HASH_F,
            source_head_hash: HASH_D,
            cut_point_hash,
            edited_body_class: SAFE_BODY_CLASS,
            requested_branch_label: SAFE_BRANCH_LABEL,
            continuation_target: SessionBranchContinuationTarget::New,
            provenance_label: SAFE_PROVENANCE,
            copy_descriptors: &[SessionCopyDescriptorDraft {
                class: SessionCopyDescriptorClass::SessionLabel,
                label: SAFE_DESCRIPTOR_LABEL,
            }],
            ambient_storage_requested,
        }
    }

    #[derive(Clone, Copy)]
    struct PolicyFixture {
        allow_suffix_discard: bool,
        require_checkpoint_for_discard: bool,
        deny_unsafe_copy: bool,
    }

    const DEFAULT_POLICY_FIXTURE: PolicyFixture = PolicyFixture {
        allow_suffix_discard: false,
        require_checkpoint_for_discard: false,
        deny_unsafe_copy: false,
    };

    fn admitted_policy(fixture: PolicyFixture) -> Result<SessionBranchPolicy, SessionBranchError> {
        SessionBranchPolicy::new(&SessionBranchPolicyDraft {
            policy_hash: HASH_A,
            admission: SessionBranchPolicyAdmission::Admitted,
            allow_suffix_discard: fixture.allow_suffix_discard,
            require_checkpoint_for_discard: fixture.require_checkpoint_for_discard,
            deny_unsafe_copy: fixture.deny_unsafe_copy,
        })
    }
}

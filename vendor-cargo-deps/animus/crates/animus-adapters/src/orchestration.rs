//! Public orchestration adapter-family contracts.
//!
//! The family is implementation-free and `no_std`: hosts inject board, task,
//! scheduling, cancellation, and lifecycle evidence services; reusable crates
//! validate typed safe DTOs and fail closed. Concrete board databases, worker
//! schedulers, process supervision, filesystem authority, VCS commands, clocks,
//! networks, and persistence remain host-owned.

#![cfg_attr(
    animus_octet,
    allow(
        tigerstyle::non_trait_imports,
        tigerstyle::path_segment_repetition,
        tigerstyle::usize_in_public_api
    )
)]

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use alloc::vec::Vec;

const BOARD_LABEL_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("orchestration_board_label");
const TASK_ID_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("orchestration_task_id");
const TITLE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("orchestration_task_title");
const BODY_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("orchestration_task_body");
const ACTOR_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("orchestration_actor_label");
const POLICY_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("orchestration_policy_label");
const TENANT_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("orchestration_tenant_label");
const ASSIGNEE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("orchestration_assignee_label");
const PARENT_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("orchestration_parent_id");
const CHILD_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("orchestration_child_id");
const ROOT_CAPABILITY_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("orchestration_root_capability");
const TARGET_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("orchestration_target_label");
const APPROVAL_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("orchestration_approval_label");
const HANDOFF_ID_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("orchestration_handoff_id");
const HANDOFF_BODY_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("orchestration_handoff_body");
const SUMMARY_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("orchestration_summary");
const REASON_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("orchestration_reason");
const CONTEXT_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("orchestration_context");
const NEXT_ACTION_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("orchestration_next_action");
const SEQUENCE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("orchestration_sequence");
const REVISION_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("orchestration_task_revision");
const METADATA_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("orchestration_metadata");
const REDACTION_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("orchestration_redaction_marker");
const CONTRACT_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("orchestration_contract_identity");
const FAMILY_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("orchestration_family_label");
const HASH_SEPARATOR: &str = "\u{1f}";
const UNSAFE_RAW_PROMPT: &str = "RAW_PROMPT:";
const UNSAFE_PROVIDER_PAYLOAD: &str = "PROVIDER_PAYLOAD:";
const UNSAFE_CREDENTIAL: &str = "CREDENTIAL:";
const UNSAFE_TOKEN: &str = "TOKEN=";
const UNSAFE_HOME_PATH: &str = "/home/";
const UNSAFE_HIDDEN_REASONING: &str = "HIDDEN_REASONING:";
const UNSAFE_UNBOUNDED_LOG: &str = "UNBOUNDED_LOG:";
const UNSAFE_UNBOUNDED_OUTPUT: &str = "UNBOUNDED_OUTPUT:";
const UNSAFE_HOST_DATABASE: &str = "HOST_DATABASE_ROW:";
const UNSAFE_RAW_TASK_BODY: &str = "RAW_TASK_BODY:";
const UNSAFE_PRIVATE_KEY: &str = "BEGIN PRIVATE KEY";
const UNSAFE_API_KEY: &str = "API_KEY=";
const UNSAFE_AWS_SECRET: &str = "AWS_SECRET_ACCESS_KEY";
const UNSAFE_OPENAI_KEY_PREFIX: &str = "sk-";
const MAX_ORCHESTRATION_TASKS: usize = 256;
const MAX_ORCHESTRATION_LINKS: usize = 64;
const MAX_ORCHESTRATION_RECORDS: usize = 32;
const MAX_ORCHESTRATION_METADATA_FIELDS: usize = 32;
const MAX_ORCHESTRATION_REDACTIONS: usize = 16;
const MIN_ORCHESTRATION_SEQUENCE: u64 = 1;
const MIN_ORCHESTRATION_REVISION: u64 = 1;

const _: () = assert!(MAX_ORCHESTRATION_TASKS > 0);
const _: () = assert!(MAX_ORCHESTRATION_LINKS > 0);
const _: () = assert!(MAX_ORCHESTRATION_RECORDS > 0);
const _: () = assert!(MAX_ORCHESTRATION_METADATA_FIELDS > 0);
const _: () = assert!(MAX_ORCHESTRATION_REDACTIONS > 0);
const _: () = assert!(MIN_ORCHESTRATION_SEQUENCE > 0);
const _: () = assert!(MIN_ORCHESTRATION_REVISION > 0);

const UNSAFE_TEXT_MARKERS: &[&str] = &[
    UNSAFE_RAW_PROMPT,
    UNSAFE_PROVIDER_PAYLOAD,
    UNSAFE_CREDENTIAL,
    UNSAFE_TOKEN,
    UNSAFE_HOME_PATH,
    UNSAFE_HIDDEN_REASONING,
    UNSAFE_UNBOUNDED_LOG,
    UNSAFE_UNBOUNDED_OUTPUT,
    UNSAFE_HOST_DATABASE,
    UNSAFE_RAW_TASK_BODY,
    UNSAFE_PRIVATE_KEY,
    UNSAFE_API_KEY,
    UNSAFE_AWS_SECRET,
    UNSAFE_OPENAI_KEY_PREFIX,
];

/// Stable family label used by the orchestration adapter-family index.
pub const ORCHESTRATION_FAMILY_LABEL: &str = "family:orchestration";
/// Stable contract identity for safe orchestration receipts.
pub const ORCHESTRATION_RECEIPT_CONTRACT_IDENTITY: &str = "animus.orchestration-receipt.v1";

/// Error returned by orchestration adapter-family validation or dispatch.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OrchestrationError {
    /// Generic adapter field validation failed.
    Contract(crate::AdapterContractError),
    /// No service was injected for a host-owned operation.
    MissingService,
    /// A known service reported a safely bounded backend outage.
    BackendUnavailable,
    /// A policy, capability, or tenant admission check denied the operation.
    CapabilityDenied,
    /// A board or task graph snapshot was malformed or unsafe.
    InvalidGraph,
    /// A handoff contained unsafe data or failed handoff-specific admission.
    UnsafeHandoff,
    /// A dependency or handoff crossed tenant boundaries.
    CrossTenantLeak,
    /// A requested mutation targeted data outside the declared workspace scope.
    OutOfScopeMutation,
    /// The service does not support the requested scheduling operation.
    UnsupportedOperation,
    /// A DTO or receipt declared an unsupported contract identity.
    SchemaMismatch,
    /// Bounded lifecycle evidence contained raw private material.
    UnsafeEvidence {
        /// Field containing unsafe material.
        field: &'static str,
        /// Marker that caused rejection.
        marker: &'static str,
    },
    /// A required collection was empty.
    EmptyCollection {
        /// Collection field name.
        field: &'static str,
    },
    /// A collection exceeded its configured bound.
    TooManyItems {
        /// Collection field name.
        field: &'static str,
        /// Maximum accepted item count.
        max_items: usize,
        /// Actual item count.
        actual_items: usize,
    },
    /// A sequence or revision was below the accepted minimum.
    NonMonotonicSequence {
        /// Field name.
        field: &'static str,
        /// Minimum accepted value.
        min_value: u64,
        /// Actual value.
        actual_value: u64,
    },
}

impl From<crate::AdapterContractError> for OrchestrationError {
    fn from(error: crate::AdapterContractError) -> Self {
        Self::Contract(error)
    }
}

impl core::fmt::Display for OrchestrationError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Contract(error) => write!(formatter, "orchestration contract error: {error}"),
            Self::MissingService => formatter.write_str("orchestration service is missing"),
            Self::BackendUnavailable => formatter.write_str("orchestration backend is unavailable"),
            Self::CapabilityDenied => formatter.write_str("orchestration capability was denied"),
            Self::InvalidGraph => formatter.write_str("orchestration task graph is invalid"),
            Self::UnsafeHandoff => formatter.write_str("orchestration handoff is unsafe"),
            Self::CrossTenantLeak => formatter.write_str("orchestration crosses tenant boundary"),
            Self::OutOfScopeMutation => {
                formatter.write_str("orchestration mutation is outside workspace scope")
            }
            Self::UnsupportedOperation => {
                formatter.write_str("orchestration operation is unsupported")
            }
            Self::SchemaMismatch => formatter.write_str("orchestration schema mismatch"),
            Self::UnsafeEvidence { field, marker } => {
                write!(formatter, "{field} contains unsafe marker {marker}")
            }
            Self::EmptyCollection { field } => write!(formatter, "{field} must not be empty"),
            Self::TooManyItems {
                field,
                max_items,
                actual_items,
            } => write!(
                formatter,
                "{field} must contain at most {max_items} items; got {actual_items}"
            ),
            Self::NonMonotonicSequence {
                field,
                min_value,
                actual_value,
            } => write!(
                formatter,
                "{field} must be at least {min_value}; got {actual_value}"
            ),
        }
    }
}

/// Host-owned orchestration operation kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OrchestrationOperationKind {
    /// Load an already-materialized task graph snapshot.
    LoadTaskGraph,
    /// Append a bounded handoff through host-owned storage.
    AppendHandoff,
    /// Ask the host scheduler whether a task may run.
    AdmitScheduling,
    /// Request cancellation for a task.
    CancelTask,
    /// Record bounded lifecycle evidence.
    RecordLifecycleEvidence,
}

impl OrchestrationOperationKind {
    /// Returns the stable operation label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::LoadTaskGraph => "load_task_graph",
            Self::AppendHandoff => "append_handoff",
            Self::AdmitScheduling => "admit_scheduling",
            Self::CancelTask => "cancel_task",
            Self::RecordLifecycleEvidence => "record_lifecycle_evidence",
        }
    }
}

/// Safe status class for orchestration outcomes and receipts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OrchestrationStatusClass {
    /// Operation completed with accepted input.
    Accepted,
    /// Operation was denied by policy or capability checks.
    Denied,
    /// Operation could not run because the host backend was unavailable.
    Unavailable,
    /// Operation was cancelled.
    Cancelled,
    /// Operation is unsupported by this host service.
    Unsupported,
    /// Operation recorded evidence without dispatching work.
    Recorded,
}

impl OrchestrationStatusClass {
    /// Returns the stable status label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Accepted => "accepted",
            Self::Denied => "denied",
            Self::Unavailable => "unavailable",
            Self::Cancelled => "cancelled",
            Self::Unsupported => "unsupported",
            Self::Recorded => "recorded",
        }
    }
}

/// Lifecycle state for a durable orchestration task.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OrchestrationTaskState {
    /// Task exists but is not ready to run.
    Draft,
    /// Task can be scheduled by a host.
    Ready,
    /// Task is running in a host-owned worker.
    InProgress,
    /// Task is blocked.
    Blocked,
    /// Task completed.
    Completed,
    /// Task was cancelled.
    Cancelled,
}

/// Priority metadata for host scheduling inputs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OrchestrationPriority {
    /// Low-priority durable task.
    Low,
    /// Default-priority durable task.
    Normal,
    /// High-priority durable task.
    High,
    /// Urgent durable task.
    Urgent,
}

/// Explicit workspace capability class declared by a task.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OrchestrationWorkspaceKind {
    /// Scratch workspace with a host-supplied root capability.
    Scratch,
    /// Directory workspace identified by safe host labels.
    Directory,
    /// Isolated branch workspace identified by safe host labels.
    IsolatedBranch,
}

/// Explicit workspace scope for a task.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrchestrationWorkspaceScope {
    kind: OrchestrationWorkspaceKind,
    root_capability: String,
    writable_targets: Vec<String>,
    approval_required_targets: Vec<String>,
}

impl OrchestrationWorkspaceScope {
    /// Builds a validated workspace scope.
    ///
    /// # Errors
    ///
    /// Returns [`OrchestrationError`] when labels are empty, unsafe, or the
    /// target collections are both empty or oversized.
    pub fn new(
        kind: OrchestrationWorkspaceKind,
        root_capability: &str,
        writable_targets: &[&str],
        approval_required_targets: &[&str],
    ) -> Result<Self, OrchestrationError> {
        let writable = safe_text_list(TARGET_FIELD, writable_targets, true)?;
        let approvals = safe_text_list(TARGET_FIELD, approval_required_targets, true)?;
        if writable.is_empty() && approvals.is_empty() {
            return Err(OrchestrationError::EmptyCollection {
                field: TARGET_FIELD.as_str(),
            });
        }
        Ok(Self {
            kind,
            root_capability: safe_text(ROOT_CAPABILITY_FIELD, root_capability)?,
            writable_targets: writable,
            approval_required_targets: approvals,
        })
    }

    /// Returns the workspace kind.
    #[must_use]
    pub const fn kind(&self) -> OrchestrationWorkspaceKind {
        self.kind
    }

    /// Returns the safe root capability label.
    #[must_use]
    pub fn root_capability(&self) -> &str {
        self.root_capability.as_str()
    }

    /// Returns writable target labels.
    #[must_use]
    pub fn writable_targets(&self) -> &[String] {
        self.writable_targets.as_slice()
    }

    /// Returns target labels that require separate host approval.
    #[must_use]
    pub fn approval_required_targets(&self) -> &[String] {
        self.approval_required_targets.as_slice()
    }
}

/// Mutation attempt checked against a declared workspace scope.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrchestrationMutationIntent {
    target_label: String,
    approval_label: Option<String>,
}

impl OrchestrationMutationIntent {
    /// Builds a validated mutation intent.
    ///
    /// # Errors
    ///
    /// Returns [`OrchestrationError`] when labels are empty or unsafe.
    pub fn new(
        target_label: &str,
        approval_label: Option<&str>,
    ) -> Result<Self, OrchestrationError> {
        Ok(Self {
            target_label: safe_text(TARGET_FIELD, target_label)?,
            approval_label: optional_safe_text(APPROVAL_FIELD, approval_label)?,
        })
    }

    /// Returns the target label.
    #[must_use]
    pub fn target_label(&self) -> &str {
        self.target_label.as_str()
    }

    /// Returns the optional approval label.
    #[must_use]
    pub fn approval_label(&self) -> Option<&str> {
        self.approval_label.as_deref()
    }
}

/// Workspace mutation admission result.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OrchestrationWorkspaceAdmission {
    /// Mutation is within declared writable targets.
    Allowed,
    /// Mutation targets a guarded label and needs separate host approval.
    RequiresHostApproval,
}

/// Checks a mutation against an explicit orchestration workspace scope.
///
/// # Errors
///
/// Returns [`OrchestrationError::OutOfScopeMutation`] when the target is not
/// declared in the scope, or validation errors when labels are unsafe.
///
/// # Panics
///
/// Panics if validated target or scope-count post-conditions are violated.
pub fn validate_orchestration_workspace_mutation(
    scope: &OrchestrationWorkspaceScope,
    mutation: &OrchestrationMutationIntent,
) -> Result<OrchestrationWorkspaceAdmission, OrchestrationError> {
    let target = safe_text(TARGET_FIELD, mutation.target_label())?;
    assert!(!target.is_empty());
    let scope_target_count = scope
        .writable_targets()
        .len()
        .saturating_add(scope.approval_required_targets().len());
    assert!(scope_target_count > 0);
    if scope
        .writable_targets()
        .iter()
        .any(|allowed| allowed == &target)
    {
        return Ok(OrchestrationWorkspaceAdmission::Allowed);
    }
    if scope
        .approval_required_targets()
        .iter()
        .any(|guarded| guarded == &target)
    {
        if let Some(approval_label) = mutation.approval_label() {
            let _approval = safe_text(APPROVAL_FIELD, approval_label)?;
            return Ok(OrchestrationWorkspaceAdmission::Allowed);
        }
        return Ok(OrchestrationWorkspaceAdmission::RequiresHostApproval);
    }
    Err(OrchestrationError::OutOfScopeMutation)
}

/// Comment record attached to a task graph snapshot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrchestrationComment {
    actor_label: String,
    sequence: u64,
    body: String,
    tenant_label: Option<String>,
    metadata: BTreeMap<String, String>,
}

/// Borrowed input used to build a bounded comment record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrchestrationCommentInput<'a> {
    /// Actor label supplied by the host.
    pub actor_label: &'a str,
    /// Monotonic sequence supplied by the host.
    pub sequence: u64,
    /// Bounded comment body.
    pub body: &'a str,
    /// Tenant label preserved when supplied by the host.
    pub tenant_label: Option<&'a str>,
    /// Bounded safe metadata.
    pub metadata: &'a BTreeMap<String, String>,
}

impl OrchestrationComment {
    /// Builds a bounded comment record.
    ///
    /// # Errors
    ///
    /// Returns [`OrchestrationError`] when fields are unsafe, metadata is
    /// oversized, or the sequence is below the accepted minimum.
    pub fn new(input: &OrchestrationCommentInput<'_>) -> Result<Self, OrchestrationError> {
        validate_minimum(
            SEQUENCE_FIELD.as_str(),
            input.sequence,
            MIN_ORCHESTRATION_SEQUENCE,
        )?;
        Ok(Self {
            actor_label: safe_text(ACTOR_FIELD, input.actor_label)?,
            sequence: input.sequence,
            body: safe_text(BODY_FIELD, input.body)?,
            tenant_label: optional_safe_text(TENANT_FIELD, input.tenant_label)?,
            metadata: safe_metadata(input.metadata)?,
        })
    }

    /// Returns actor label.
    #[must_use]
    pub fn actor_label(&self) -> &str {
        self.actor_label.as_str()
    }

    /// Returns monotonic sequence.
    #[must_use]
    pub const fn sequence(&self) -> u64 {
        self.sequence
    }

    /// Returns bounded body.
    #[must_use]
    pub fn body(&self) -> &str {
        self.body.as_str()
    }

    /// Returns tenant label when present.
    #[must_use]
    pub fn tenant_label(&self) -> Option<&str> {
        self.tenant_label.as_deref()
    }

    /// Returns safe metadata.
    #[must_use]
    pub fn metadata(&self) -> &BTreeMap<String, String> {
        &self.metadata
    }
}

/// Heartbeat record attached to a task graph snapshot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrchestrationHeartbeat {
    actor_label: String,
    sequence: u64,
    summary: String,
    tenant_label: Option<String>,
    metadata: BTreeMap<String, String>,
}

/// Borrowed input used to build a bounded heartbeat record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrchestrationHeartbeatInput<'a> {
    /// Actor label supplied by the host.
    pub actor_label: &'a str,
    /// Monotonic sequence supplied by the host.
    pub sequence: u64,
    /// Bounded status summary.
    pub summary: &'a str,
    /// Tenant label preserved when supplied by the host.
    pub tenant_label: Option<&'a str>,
    /// Bounded safe metadata.
    pub metadata: &'a BTreeMap<String, String>,
}

impl OrchestrationHeartbeat {
    /// Builds a bounded heartbeat record.
    ///
    /// # Errors
    ///
    /// Returns [`OrchestrationError`] when fields are unsafe, metadata is
    /// oversized, or the sequence is below the accepted minimum.
    pub fn new(input: &OrchestrationHeartbeatInput<'_>) -> Result<Self, OrchestrationError> {
        validate_minimum(
            SEQUENCE_FIELD.as_str(),
            input.sequence,
            MIN_ORCHESTRATION_SEQUENCE,
        )?;
        Ok(Self {
            actor_label: safe_text(ACTOR_FIELD, input.actor_label)?,
            sequence: input.sequence,
            summary: safe_text(SUMMARY_FIELD, input.summary)?,
            tenant_label: optional_safe_text(TENANT_FIELD, input.tenant_label)?,
            metadata: safe_metadata(input.metadata)?,
        })
    }

    /// Returns actor label.
    #[must_use]
    pub fn actor_label(&self) -> &str {
        self.actor_label.as_str()
    }

    /// Returns monotonic sequence.
    #[must_use]
    pub const fn sequence(&self) -> u64 {
        self.sequence
    }

    /// Returns bounded status summary.
    #[must_use]
    pub fn summary(&self) -> &str {
        self.summary.as_str()
    }

    /// Returns tenant label when present.
    #[must_use]
    pub fn tenant_label(&self) -> Option<&str> {
        self.tenant_label.as_deref()
    }

    /// Returns safe metadata.
    #[must_use]
    pub fn metadata(&self) -> &BTreeMap<String, String> {
        &self.metadata
    }
}

/// Block record attached to a task graph snapshot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrchestrationBlockRecord {
    actor_label: String,
    sequence: u64,
    reason: String,
    context: Option<String>,
    next_action: Option<String>,
    tenant_label: Option<String>,
    metadata: BTreeMap<String, String>,
}

/// Borrowed input used to build a bounded block record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrchestrationBlockRecordInput<'a> {
    /// Actor label supplied by the host.
    pub actor_label: &'a str,
    /// Monotonic sequence supplied by the host.
    pub sequence: u64,
    /// Short actionable block reason.
    pub reason: &'a str,
    /// Optional bounded context comment.
    pub context: Option<&'a str>,
    /// Optional next-action hint.
    pub next_action: Option<&'a str>,
    /// Tenant label preserved when supplied by the host.
    pub tenant_label: Option<&'a str>,
    /// Bounded safe metadata.
    pub metadata: &'a BTreeMap<String, String>,
}

impl OrchestrationBlockRecord {
    /// Builds a bounded block record.
    ///
    /// # Errors
    ///
    /// Returns [`OrchestrationError`] when fields are unsafe, metadata is
    /// oversized, or the sequence is below the accepted minimum.
    pub fn new(input: &OrchestrationBlockRecordInput<'_>) -> Result<Self, OrchestrationError> {
        validate_minimum(
            SEQUENCE_FIELD.as_str(),
            input.sequence,
            MIN_ORCHESTRATION_SEQUENCE,
        )?;
        Ok(Self {
            actor_label: safe_text(ACTOR_FIELD, input.actor_label)?,
            sequence: input.sequence,
            reason: safe_text(REASON_FIELD, input.reason)?,
            context: optional_safe_text(CONTEXT_FIELD, input.context)?,
            next_action: optional_safe_text(NEXT_ACTION_FIELD, input.next_action)?,
            tenant_label: optional_safe_text(TENANT_FIELD, input.tenant_label)?,
            metadata: safe_metadata(input.metadata)?,
        })
    }

    /// Returns actor label.
    #[must_use]
    pub fn actor_label(&self) -> &str {
        self.actor_label.as_str()
    }

    /// Returns monotonic sequence.
    #[must_use]
    pub const fn sequence(&self) -> u64 {
        self.sequence
    }

    /// Returns block reason.
    #[must_use]
    pub fn reason(&self) -> &str {
        self.reason.as_str()
    }

    /// Returns optional context.
    #[must_use]
    pub fn context(&self) -> Option<&str> {
        self.context.as_deref()
    }

    /// Returns optional next action.
    #[must_use]
    pub fn next_action(&self) -> Option<&str> {
        self.next_action.as_deref()
    }

    /// Returns tenant label when present.
    #[must_use]
    pub fn tenant_label(&self) -> Option<&str> {
        self.tenant_label.as_deref()
    }

    /// Returns safe metadata.
    #[must_use]
    pub fn metadata(&self) -> &BTreeMap<String, String> {
        &self.metadata
    }
}

/// Completion record attached to a completed task.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrchestrationCompletion {
    actor_label: String,
    sequence: u64,
    summary: String,
    task_revision: u64,
    tenant_label: Option<String>,
    metadata: BTreeMap<String, String>,
}

/// Borrowed input used to build a bounded completion record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrchestrationCompletionInput<'a> {
    /// Actor label supplied by the host.
    pub actor_label: &'a str,
    /// Monotonic sequence supplied by the host.
    pub sequence: u64,
    /// Bounded completion summary.
    pub summary: &'a str,
    /// Task revision observed by the completing worker.
    pub task_revision: u64,
    /// Tenant label preserved when supplied by the host.
    pub tenant_label: Option<&'a str>,
    /// Bounded safe metadata.
    pub metadata: &'a BTreeMap<String, String>,
}

impl OrchestrationCompletion {
    /// Builds a bounded completion record.
    ///
    /// # Errors
    ///
    /// Returns [`OrchestrationError`] when fields are unsafe, metadata is
    /// oversized, or sequence/revision values are below accepted minimums.
    pub fn new(input: &OrchestrationCompletionInput<'_>) -> Result<Self, OrchestrationError> {
        validate_minimum(
            SEQUENCE_FIELD.as_str(),
            input.sequence,
            MIN_ORCHESTRATION_SEQUENCE,
        )?;
        validate_minimum(
            REVISION_FIELD.as_str(),
            input.task_revision,
            MIN_ORCHESTRATION_REVISION,
        )?;
        Ok(Self {
            actor_label: safe_text(ACTOR_FIELD, input.actor_label)?,
            sequence: input.sequence,
            summary: safe_text(SUMMARY_FIELD, input.summary)?,
            task_revision: input.task_revision,
            tenant_label: optional_safe_text(TENANT_FIELD, input.tenant_label)?,
            metadata: safe_metadata(input.metadata)?,
        })
    }

    /// Returns actor label.
    #[must_use]
    pub fn actor_label(&self) -> &str {
        self.actor_label.as_str()
    }

    /// Returns monotonic sequence.
    #[must_use]
    pub const fn sequence(&self) -> u64 {
        self.sequence
    }

    /// Returns completion summary.
    #[must_use]
    pub fn summary(&self) -> &str {
        self.summary.as_str()
    }

    /// Returns observed task revision.
    #[must_use]
    pub const fn task_revision(&self) -> u64 {
        self.task_revision
    }

    /// Returns tenant label when present.
    #[must_use]
    pub fn tenant_label(&self) -> Option<&str> {
        self.tenant_label.as_deref()
    }

    /// Returns safe metadata.
    #[must_use]
    pub fn metadata(&self) -> &BTreeMap<String, String> {
        &self.metadata
    }
}

/// Borrowed input used to build a validated task draft.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrchestrationTaskDraftInput<'a> {
    /// Stable task identity.
    pub task_id: &'a str,
    /// Human-readable task title.
    pub title: &'a str,
    /// Bounded task body or safe summary.
    pub body: &'a str,
    /// Optional worker or assignee label.
    pub assignee_label: Option<&'a str>,
    /// Lifecycle state supplied by the host snapshot.
    pub state: OrchestrationTaskState,
    /// Parent dependencies by task ID.
    pub parent_ids: &'a [&'a str],
    /// Child task links by task ID.
    pub child_ids: &'a [&'a str],
    /// Priority metadata.
    pub priority: OrchestrationPriority,
    /// Explicit workspace scope.
    pub workspace_scope: Option<OrchestrationWorkspaceScope>,
    /// Optional tenant label preserved through handoffs.
    pub tenant_label: Option<&'a str>,
    /// Comments attached to the task.
    pub comments: &'a [OrchestrationComment],
    /// Heartbeats attached to the task.
    pub heartbeats: &'a [OrchestrationHeartbeat],
    /// Block records attached to the task.
    pub block_records: &'a [OrchestrationBlockRecord],
    /// Completion record for completed tasks.
    pub completion: Option<OrchestrationCompletion>,
    /// Bounded safe metadata.
    pub metadata: &'a BTreeMap<String, String>,
    /// Host-supplied task revision.
    pub revision: u64,
}

/// Validated task draft supplied by a host service.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrchestrationTaskDraft {
    task_id: String,
    title: String,
    body: String,
    assignee_label: Option<String>,
    state: OrchestrationTaskState,
    parent_ids: Vec<String>,
    child_ids: Vec<String>,
    priority: OrchestrationPriority,
    workspace_scope: Option<OrchestrationWorkspaceScope>,
    tenant_label: Option<String>,
    comments: Vec<OrchestrationComment>,
    heartbeats: Vec<OrchestrationHeartbeat>,
    block_records: Vec<OrchestrationBlockRecord>,
    completion: Option<OrchestrationCompletion>,
    metadata: BTreeMap<String, String>,
    revision: u64,
}

impl OrchestrationTaskDraft {
    /// Builds a validated task draft from borrowed input.
    ///
    /// # Errors
    ///
    /// Returns [`OrchestrationError`] when fields are unsafe, collections are
    /// oversized, or the revision is below the accepted minimum.
    pub fn new(input: OrchestrationTaskDraftInput<'_>) -> Result<Self, OrchestrationError> {
        validate_minimum(
            REVISION_FIELD.as_str(),
            input.revision,
            MIN_ORCHESTRATION_REVISION,
        )?;
        validate_collection_count(
            PARENT_FIELD,
            input.parent_ids.len(),
            MAX_ORCHESTRATION_LINKS,
        )?;
        validate_collection_count(CHILD_FIELD, input.child_ids.len(), MAX_ORCHESTRATION_LINKS)?;
        validate_collection_count(
            SUMMARY_FIELD,
            input.comments.len(),
            MAX_ORCHESTRATION_RECORDS,
        )?;
        validate_collection_count(
            SUMMARY_FIELD,
            input.heartbeats.len(),
            MAX_ORCHESTRATION_RECORDS,
        )?;
        validate_collection_count(
            REASON_FIELD,
            input.block_records.len(),
            MAX_ORCHESTRATION_RECORDS,
        )?;
        Ok(Self {
            task_id: safe_text(TASK_ID_FIELD, input.task_id)?,
            title: safe_text(TITLE_FIELD, input.title)?,
            body: safe_text(BODY_FIELD, input.body)?,
            assignee_label: optional_safe_text(ASSIGNEE_FIELD, input.assignee_label)?,
            state: input.state,
            parent_ids: safe_text_list(PARENT_FIELD, input.parent_ids, true)?,
            child_ids: safe_text_list(CHILD_FIELD, input.child_ids, true)?,
            priority: input.priority,
            workspace_scope: input.workspace_scope,
            tenant_label: optional_safe_text(TENANT_FIELD, input.tenant_label)?,
            comments: input.comments.to_vec(),
            heartbeats: input.heartbeats.to_vec(),
            block_records: input.block_records.to_vec(),
            completion: input.completion,
            metadata: safe_metadata(input.metadata)?,
            revision: input.revision,
        })
    }

    /// Returns task identity.
    #[must_use]
    pub fn task_id(&self) -> &str {
        self.task_id.as_str()
    }

    /// Returns task title.
    #[must_use]
    pub fn title(&self) -> &str {
        self.title.as_str()
    }

    /// Returns bounded task body or safe summary.
    #[must_use]
    pub fn body(&self) -> &str {
        self.body.as_str()
    }

    /// Returns optional assignee label.
    #[must_use]
    pub fn assignee_label(&self) -> Option<&str> {
        self.assignee_label.as_deref()
    }

    /// Returns task state.
    #[must_use]
    pub const fn state(&self) -> OrchestrationTaskState {
        self.state
    }

    /// Returns parent IDs.
    #[must_use]
    pub fn parent_ids(&self) -> &[String] {
        self.parent_ids.as_slice()
    }

    /// Returns child IDs.
    #[must_use]
    pub fn child_ids(&self) -> &[String] {
        self.child_ids.as_slice()
    }

    /// Returns priority.
    #[must_use]
    pub const fn priority(&self) -> OrchestrationPriority {
        self.priority
    }

    /// Returns workspace scope when present.
    #[must_use]
    pub const fn workspace_scope(&self) -> Option<&OrchestrationWorkspaceScope> {
        self.workspace_scope.as_ref()
    }

    /// Returns tenant label when present.
    #[must_use]
    pub fn tenant_label(&self) -> Option<&str> {
        self.tenant_label.as_deref()
    }

    /// Returns comments.
    #[must_use]
    pub fn comments(&self) -> &[OrchestrationComment] {
        self.comments.as_slice()
    }

    /// Returns heartbeats.
    #[must_use]
    pub fn heartbeats(&self) -> &[OrchestrationHeartbeat] {
        self.heartbeats.as_slice()
    }

    /// Returns block records.
    #[must_use]
    pub fn block_records(&self) -> &[OrchestrationBlockRecord] {
        self.block_records.as_slice()
    }

    /// Returns completion record when present.
    #[must_use]
    pub const fn completion(&self) -> Option<&OrchestrationCompletion> {
        self.completion.as_ref()
    }

    /// Returns safe metadata.
    #[must_use]
    pub fn metadata(&self) -> &BTreeMap<String, String> {
        &self.metadata
    }

    /// Returns host-supplied revision.
    #[must_use]
    pub const fn revision(&self) -> u64 {
        self.revision
    }
}

/// Validated task graph snapshot supplied by a host service.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrchestrationTaskGraphDraft {
    tasks: Vec<OrchestrationTaskDraft>,
}

impl OrchestrationTaskGraphDraft {
    /// Builds a validated task graph draft.
    ///
    /// # Errors
    ///
    /// Returns [`OrchestrationError`] when the task list is empty, oversized,
    /// contains duplicate IDs, contains one-sided links, or crosses tenants.
    pub fn new(tasks: Vec<OrchestrationTaskDraft>) -> Result<Self, OrchestrationError> {
        validate_task_graph(tasks.as_slice())?;
        Ok(Self { tasks })
    }

    /// Returns validated task drafts.
    #[must_use]
    pub fn tasks(&self) -> &[OrchestrationTaskDraft] {
        self.tasks.as_slice()
    }
}

/// Handoff projected from one orchestration task to another.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrchestrationContextHandoff {
    handoff_id: String,
    task_id: String,
    tenant_label: Option<String>,
    body: String,
    sequence: u64,
    metadata: BTreeMap<String, String>,
}

/// Borrowed input used to build a handoff.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrchestrationContextHandoffDraft<'a> {
    /// Stable handoff identity.
    pub handoff_id: &'a str,
    /// Source task identity.
    pub task_id: &'a str,
    /// Tenant label preserved when supplied by the host.
    pub tenant_label: Option<&'a str>,
    /// Bounded safe handoff body.
    pub body: &'a str,
    /// Monotonic sequence supplied by the host.
    pub sequence: u64,
    /// Bounded safe metadata.
    pub metadata: &'a BTreeMap<String, String>,
}

impl OrchestrationContextHandoff {
    /// Builds a validated handoff.
    ///
    /// # Errors
    ///
    /// Returns [`OrchestrationError`] when fields are unsafe, metadata is
    /// oversized, or the sequence is below the accepted minimum.
    pub fn new(input: &OrchestrationContextHandoffDraft<'_>) -> Result<Self, OrchestrationError> {
        validate_minimum(
            SEQUENCE_FIELD.as_str(),
            input.sequence,
            MIN_ORCHESTRATION_SEQUENCE,
        )?;
        Ok(Self {
            handoff_id: safe_text(HANDOFF_ID_FIELD, input.handoff_id)?,
            task_id: safe_text(TASK_ID_FIELD, input.task_id)?,
            tenant_label: optional_safe_text(TENANT_FIELD, input.tenant_label)?,
            body: safe_text(HANDOFF_BODY_FIELD, input.body)?,
            sequence: input.sequence,
            metadata: safe_metadata(input.metadata)?,
        })
    }

    /// Returns handoff identity.
    #[must_use]
    pub fn handoff_id(&self) -> &str {
        self.handoff_id.as_str()
    }

    /// Returns source task identity.
    #[must_use]
    pub fn task_id(&self) -> &str {
        self.task_id.as_str()
    }

    /// Returns tenant label when present.
    #[must_use]
    pub fn tenant_label(&self) -> Option<&str> {
        self.tenant_label.as_deref()
    }

    /// Returns bounded safe body.
    #[must_use]
    pub fn body(&self) -> &str {
        self.body.as_str()
    }

    /// Returns monotonic sequence.
    #[must_use]
    pub const fn sequence(&self) -> u64 {
        self.sequence
    }

    /// Returns safe metadata.
    #[must_use]
    pub fn metadata(&self) -> &BTreeMap<String, String> {
        &self.metadata
    }
}

/// Safe metadata attached to host-owned orchestration operations.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrchestrationOperationMetadata {
    actor: String,
    policy: String,
    tenant: Option<String>,
}

impl OrchestrationOperationMetadata {
    /// Builds safe operation metadata.
    ///
    /// # Errors
    ///
    /// Returns [`OrchestrationError`] when labels are empty or unsafe.
    pub fn new(
        actor_label: &str,
        policy_label: &str,
        tenant_label: Option<&str>,
    ) -> Result<Self, OrchestrationError> {
        Ok(Self {
            actor: safe_text(ACTOR_FIELD, actor_label)?,
            policy: safe_text(POLICY_FIELD, policy_label)?,
            tenant: optional_safe_text(TENANT_FIELD, tenant_label)?,
        })
    }

    /// Returns actor label.
    #[must_use]
    pub fn actor_label(&self) -> &str {
        self.actor.as_str()
    }

    /// Returns policy label.
    #[must_use]
    pub fn policy_label(&self) -> &str {
        self.policy.as_str()
    }

    /// Returns tenant label when present.
    #[must_use]
    pub fn tenant_label(&self) -> Option<&str> {
        self.tenant.as_deref()
    }
}

/// Request to load a task graph by safe board label.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrchestrationTaskGraphLoadRequest {
    board_label: String,
    metadata: OrchestrationOperationMetadata,
}

impl OrchestrationTaskGraphLoadRequest {
    /// Builds a graph-load request.
    ///
    /// # Errors
    ///
    /// Returns [`OrchestrationError`] when labels are empty or unsafe.
    pub fn new(
        board_label: &str,
        metadata: OrchestrationOperationMetadata,
    ) -> Result<Self, OrchestrationError> {
        Ok(Self {
            board_label: safe_text(BOARD_LABEL_FIELD, board_label)?,
            metadata,
        })
    }

    /// Returns board label.
    #[must_use]
    pub fn board_label(&self) -> &str {
        self.board_label.as_str()
    }

    /// Returns operation metadata.
    #[must_use]
    pub const fn metadata(&self) -> &OrchestrationOperationMetadata {
        &self.metadata
    }
}

/// Request to append a handoff through host-owned storage.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrchestrationHandoffAppendRequest {
    handoff: OrchestrationContextHandoff,
    metadata: OrchestrationOperationMetadata,
}

impl OrchestrationHandoffAppendRequest {
    /// Builds a handoff append request.
    #[must_use]
    pub const fn new(
        handoff: OrchestrationContextHandoff,
        metadata: OrchestrationOperationMetadata,
    ) -> Self {
        Self { handoff, metadata }
    }

    /// Returns the handoff.
    #[must_use]
    pub const fn handoff(&self) -> &OrchestrationContextHandoff {
        &self.handoff
    }

    /// Returns operation metadata.
    #[must_use]
    pub const fn metadata(&self) -> &OrchestrationOperationMetadata {
        &self.metadata
    }
}

/// Host scheduler operation requested for admission.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OrchestrationSchedulingOperation {
    /// Start a ready task.
    Start,
    /// Resume a paused or blocked task.
    Resume,
    /// Promote a task whose dependencies completed.
    Promote,
}

impl OrchestrationSchedulingOperation {
    /// Returns the stable scheduling operation label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Start => "start_task",
            Self::Resume => "resume_task",
            Self::Promote => "promote_task",
        }
    }
}

/// Request to admit a host-owned scheduling operation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrchestrationSchedulingAdmissionRequest {
    board_label: String,
    task_id: String,
    operation: OrchestrationSchedulingOperation,
    metadata: OrchestrationOperationMetadata,
}

impl OrchestrationSchedulingAdmissionRequest {
    /// Builds a scheduling admission request.
    ///
    /// # Errors
    ///
    /// Returns [`OrchestrationError`] when labels are empty or unsafe.
    pub fn new(
        board_label: &str,
        task_id: &str,
        operation: OrchestrationSchedulingOperation,
        metadata: OrchestrationOperationMetadata,
    ) -> Result<Self, OrchestrationError> {
        Ok(Self {
            board_label: safe_text(BOARD_LABEL_FIELD, board_label)?,
            task_id: safe_text(TASK_ID_FIELD, task_id)?,
            operation,
            metadata,
        })
    }

    /// Returns board label.
    #[must_use]
    pub fn board_label(&self) -> &str {
        self.board_label.as_str()
    }

    /// Returns task identity.
    #[must_use]
    pub fn task_id(&self) -> &str {
        self.task_id.as_str()
    }

    /// Returns scheduling operation.
    #[must_use]
    pub const fn operation(&self) -> OrchestrationSchedulingOperation {
        self.operation
    }

    /// Returns operation metadata.
    #[must_use]
    pub const fn metadata(&self) -> &OrchestrationOperationMetadata {
        &self.metadata
    }
}

/// Scheduling admission result.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OrchestrationSchedulingAdmission {
    /// Host scheduler admitted the operation.
    Admitted,
    /// Host scheduler denied the operation.
    Denied,
    /// Host scheduler requires separate approval.
    RequiresHostApproval,
    /// Host scheduler does not support the operation.
    Unsupported,
}

/// Request to cancel a host-owned task.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrchestrationCancellationRequest {
    board_label: String,
    task_id: String,
    reason_label: String,
    metadata: OrchestrationOperationMetadata,
}

impl OrchestrationCancellationRequest {
    /// Builds a cancellation request.
    ///
    /// # Errors
    ///
    /// Returns [`OrchestrationError`] when labels are empty or unsafe.
    pub fn new(
        board_label: &str,
        task_id: &str,
        reason_label: &str,
        metadata: OrchestrationOperationMetadata,
    ) -> Result<Self, OrchestrationError> {
        Ok(Self {
            board_label: safe_text(BOARD_LABEL_FIELD, board_label)?,
            task_id: safe_text(TASK_ID_FIELD, task_id)?,
            reason_label: safe_text(REASON_FIELD, reason_label)?,
            metadata,
        })
    }

    /// Returns board label.
    #[must_use]
    pub fn board_label(&self) -> &str {
        self.board_label.as_str()
    }

    /// Returns task identity.
    #[must_use]
    pub fn task_id(&self) -> &str {
        self.task_id.as_str()
    }

    /// Returns safe cancellation reason label.
    #[must_use]
    pub fn reason_label(&self) -> &str {
        self.reason_label.as_str()
    }

    /// Returns operation metadata.
    #[must_use]
    pub const fn metadata(&self) -> &OrchestrationOperationMetadata {
        &self.metadata
    }
}

/// Cancellation result supplied by a host service.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OrchestrationCancellationOutcome {
    /// Task was cancelled.
    Cancelled,
    /// Task had already reached a terminal state.
    AlreadyTerminal,
    /// Task was unknown to the host service.
    UnknownTask,
    /// Host service does not support cancellation.
    Unsupported,
}

/// Bounded lifecycle evidence kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OrchestrationLifecycleEvidenceKind {
    /// Comment-like lifecycle note.
    Comment,
    /// Heartbeat-like status note.
    Heartbeat,
    /// Blocked state note.
    Blocked,
    /// Completion note.
    Completed,
}

impl OrchestrationLifecycleEvidenceKind {
    /// Returns the stable evidence kind label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Comment => "comment",
            Self::Heartbeat => "heartbeat",
            Self::Blocked => "blocked",
            Self::Completed => "completed",
        }
    }
}

/// Bounded lifecycle evidence supplied by a host.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrchestrationLifecycleEvidence {
    kind: OrchestrationLifecycleEvidenceKind,
    actor_label: String,
    sequence: u64,
    summary: String,
    evidence_hash: crate::SafeHash,
    redaction_markers: Vec<String>,
}

/// Borrowed input used to build bounded lifecycle evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrchestrationLifecycleEvidenceInput<'a> {
    /// Evidence kind.
    pub kind: OrchestrationLifecycleEvidenceKind,
    /// Actor label supplied by the host.
    pub actor_label: &'a str,
    /// Monotonic sequence supplied by the host.
    pub sequence: u64,
    /// Safe lifecycle summary.
    pub summary: &'a str,
    /// BLAKE3 hash of the bounded evidence projection.
    pub evidence_hash: &'a str,
    /// Redaction markers explaining omitted raw material.
    pub redaction_markers: &'a [&'a str],
}

impl OrchestrationLifecycleEvidence {
    /// Builds bounded lifecycle evidence.
    ///
    /// # Errors
    ///
    /// Returns [`OrchestrationError`] when labels, summaries, redactions, or
    /// hashes are unsafe, or the sequence is below the accepted minimum.
    pub fn new(
        input: &OrchestrationLifecycleEvidenceInput<'_>,
    ) -> Result<Self, OrchestrationError> {
        validate_minimum(
            SEQUENCE_FIELD.as_str(),
            input.sequence,
            MIN_ORCHESTRATION_SEQUENCE,
        )?;
        validate_collection_count(
            REDACTION_FIELD,
            input.redaction_markers.len(),
            MAX_ORCHESTRATION_REDACTIONS,
        )?;
        Ok(Self {
            kind: input.kind,
            actor_label: safe_text(ACTOR_FIELD, input.actor_label)?,
            sequence: input.sequence,
            summary: safe_text(SUMMARY_FIELD, input.summary)?,
            evidence_hash: crate::SafeHash::new(input.evidence_hash)?,
            redaction_markers: safe_text_list(REDACTION_FIELD, input.redaction_markers, true)?,
        })
    }

    /// Returns evidence kind.
    #[must_use]
    pub const fn kind(&self) -> OrchestrationLifecycleEvidenceKind {
        self.kind
    }

    /// Returns actor label.
    #[must_use]
    pub fn actor_label(&self) -> &str {
        self.actor_label.as_str()
    }

    /// Returns monotonic sequence.
    #[must_use]
    pub const fn sequence(&self) -> u64 {
        self.sequence
    }

    /// Returns safe summary.
    #[must_use]
    pub fn summary(&self) -> &str {
        self.summary.as_str()
    }

    /// Returns evidence hash.
    #[must_use]
    pub const fn evidence_hash(&self) -> &crate::SafeHash {
        &self.evidence_hash
    }

    /// Returns redaction markers.
    #[must_use]
    pub fn redaction_markers(&self) -> &[String] {
        self.redaction_markers.as_slice()
    }
}

/// Request to record bounded lifecycle evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrchestrationLifecycleEvidenceRequest {
    board_label: String,
    task_id: String,
    evidence: OrchestrationLifecycleEvidence,
    metadata: OrchestrationOperationMetadata,
}

impl OrchestrationLifecycleEvidenceRequest {
    /// Builds a lifecycle evidence request.
    ///
    /// # Errors
    ///
    /// Returns [`OrchestrationError`] when labels are empty or unsafe.
    pub fn new(
        board_label: &str,
        task_id: &str,
        evidence: OrchestrationLifecycleEvidence,
        metadata: OrchestrationOperationMetadata,
    ) -> Result<Self, OrchestrationError> {
        Ok(Self {
            board_label: safe_text(BOARD_LABEL_FIELD, board_label)?,
            task_id: safe_text(TASK_ID_FIELD, task_id)?,
            evidence,
            metadata,
        })
    }

    /// Returns board label.
    #[must_use]
    pub fn board_label(&self) -> &str {
        self.board_label.as_str()
    }

    /// Returns task identity.
    #[must_use]
    pub fn task_id(&self) -> &str {
        self.task_id.as_str()
    }

    /// Returns lifecycle evidence.
    #[must_use]
    pub const fn evidence(&self) -> &OrchestrationLifecycleEvidence {
        &self.evidence
    }

    /// Returns operation metadata.
    #[must_use]
    pub const fn metadata(&self) -> &OrchestrationOperationMetadata {
        &self.metadata
    }
}

/// Draft used to build a safe orchestration receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrchestrationReceiptDraft<'a> {
    /// Declared receipt contract identity.
    pub contract_identity: &'a str,
    /// Operation recorded by the receipt.
    pub operation: OrchestrationOperationKind,
    /// Safe status class.
    pub status: OrchestrationStatusClass,
    /// Adapter family label.
    pub family_label: &'a str,
    /// Safe board label.
    pub board_label: &'a str,
    /// Optional safe task label.
    pub task_label: Option<&'a str>,
    /// Safe policy label.
    pub policy_label: &'a str,
    /// Redaction markers explaining omitted raw data.
    pub redaction_markers: &'a [&'a str],
    /// Bounded count of items represented by the receipt.
    pub item_count: usize,
    /// BLAKE3 hash of the bounded DTO or host-side evidence projection.
    pub dto_hash: &'a str,
    /// Optional safe summary.
    pub summary: Option<&'a str>,
}

/// Safe orchestration receipt containing only labels, counts, statuses, redactions, and hashes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrchestrationReceipt {
    operation: OrchestrationOperationKind,
    status: OrchestrationStatusClass,
    family_label: String,
    board_label: String,
    task_label: Option<String>,
    policy_label: String,
    redaction_markers: Vec<String>,
    item_count: usize,
    dto_hash: crate::SafeHash,
    receipt_hash: crate::SafeHash,
    summary: Option<String>,
}

impl OrchestrationReceipt {
    /// Builds a validated safe receipt.
    ///
    /// # Errors
    ///
    /// Returns [`OrchestrationError`] when the contract identity mismatches,
    /// safe fields are malformed, hashes are invalid, or leak markers are found.
    pub fn new(draft: &OrchestrationReceiptDraft<'_>) -> Result<Self, OrchestrationError> {
        let contract = safe_text(CONTRACT_FIELD, draft.contract_identity)?;
        if contract != ORCHESTRATION_RECEIPT_CONTRACT_IDENTITY {
            return Err(OrchestrationError::SchemaMismatch);
        }
        let family_label = safe_text(FAMILY_FIELD, draft.family_label)?;
        if family_label != ORCHESTRATION_FAMILY_LABEL {
            return Err(OrchestrationError::SchemaMismatch);
        }
        validate_collection_count(
            REDACTION_FIELD,
            draft.redaction_markers.len(),
            MAX_ORCHESTRATION_REDACTIONS,
        )?;
        let board_label = safe_text(BOARD_LABEL_FIELD, draft.board_label)?;
        let task_label = optional_safe_text(TASK_ID_FIELD, draft.task_label)?;
        let policy_label = safe_text(POLICY_FIELD, draft.policy_label)?;
        let redaction_markers = safe_text_list(REDACTION_FIELD, draft.redaction_markers, true)?;
        let dto_hash = crate::SafeHash::new(draft.dto_hash)?;
        let summary = optional_safe_text(SUMMARY_FIELD, draft.summary)?;
        let item_count_text = alloc::format!("{}", draft.item_count);
        let task_label_text = task_label.as_deref().unwrap_or("none");
        let summary_text = summary.as_deref().unwrap_or("none");
        let receipt_hash = blake3_orchestration_hash(&[
            draft.operation.as_str(),
            draft.status.as_str(),
            family_label.as_str(),
            board_label.as_str(),
            task_label_text,
            policy_label.as_str(),
            item_count_text.as_str(),
            dto_hash.as_str(),
            summary_text,
        ]);
        Ok(Self {
            operation: draft.operation,
            status: draft.status,
            family_label,
            board_label,
            task_label,
            policy_label,
            redaction_markers,
            item_count: draft.item_count,
            dto_hash,
            receipt_hash,
            summary,
        })
    }

    /// Returns operation kind.
    #[must_use]
    pub const fn operation(&self) -> OrchestrationOperationKind {
        self.operation
    }

    /// Returns status class.
    #[must_use]
    pub const fn status(&self) -> OrchestrationStatusClass {
        self.status
    }

    /// Returns family label.
    #[must_use]
    pub fn family_label(&self) -> &str {
        self.family_label.as_str()
    }

    /// Returns board label.
    #[must_use]
    pub fn board_label(&self) -> &str {
        self.board_label.as_str()
    }

    /// Returns task label when present.
    #[must_use]
    pub fn task_label(&self) -> Option<&str> {
        self.task_label.as_deref()
    }

    /// Returns policy label.
    #[must_use]
    pub fn policy_label(&self) -> &str {
        self.policy_label.as_str()
    }

    /// Returns redaction markers.
    #[must_use]
    pub fn redaction_markers(&self) -> &[String] {
        self.redaction_markers.as_slice()
    }

    /// Returns represented item count.
    #[must_use]
    pub const fn item_count(&self) -> usize {
        self.item_count
    }

    /// Returns DTO/evidence projection hash.
    #[must_use]
    pub const fn dto_hash(&self) -> &crate::SafeHash {
        &self.dto_hash
    }

    /// Returns receipt hash.
    #[must_use]
    pub const fn receipt_hash(&self) -> &crate::SafeHash {
        &self.receipt_hash
    }

    /// Returns optional safe summary.
    #[must_use]
    pub fn summary(&self) -> Option<&str> {
        self.summary.as_deref()
    }
}

/// Host-injected orchestration service contract.
pub trait OrchestrationService {
    /// Loads an in-memory graph draft for a safe board label.
    ///
    /// # Errors
    ///
    /// Returns [`OrchestrationError`] when the host service is unavailable,
    /// denied, unsupported, or cannot provide a safe graph draft.
    fn load_task_graph(
        &mut self,
        request: &OrchestrationTaskGraphLoadRequest,
    ) -> Result<OrchestrationTaskGraphDraft, OrchestrationError>;

    /// Appends a handoff through host-owned storage.
    ///
    /// # Errors
    ///
    /// Returns [`OrchestrationError`] when the host service rejects the handoff.
    fn append_handoff(
        &mut self,
        request: &OrchestrationHandoffAppendRequest,
    ) -> Result<OrchestrationReceipt, OrchestrationError>;

    /// Admits or rejects a scheduling operation.
    ///
    /// # Errors
    ///
    /// Returns [`OrchestrationError`] when the host service is unavailable,
    /// denied, or does not support the requested operation.
    fn admit_scheduling(
        &mut self,
        request: &OrchestrationSchedulingAdmissionRequest,
    ) -> Result<OrchestrationSchedulingAdmission, OrchestrationError>;

    /// Requests cancellation of a task.
    ///
    /// # Errors
    ///
    /// Returns [`OrchestrationError`] when the host service is unavailable,
    /// denied, or cancellation cannot be represented safely.
    fn cancel_task(
        &mut self,
        request: &OrchestrationCancellationRequest,
    ) -> Result<OrchestrationCancellationOutcome, OrchestrationError>;

    /// Records bounded lifecycle evidence.
    ///
    /// # Errors
    ///
    /// Returns [`OrchestrationError`] when the host service rejects the evidence
    /// or receipt validation fails.
    fn record_lifecycle_evidence(
        &mut self,
        request: &OrchestrationLifecycleEvidenceRequest,
    ) -> Result<OrchestrationReceipt, OrchestrationError>;
}

/// Loads a graph through an injected orchestration service.
///
/// # Errors
///
/// Returns [`OrchestrationError::MissingService`] when no service was supplied,
/// or the service's typed fail-closed error.
pub fn load_orchestration_task_graph(
    service: Option<&mut dyn OrchestrationService>,
    request: &OrchestrationTaskGraphLoadRequest,
) -> Result<OrchestrationTaskGraphDraft, OrchestrationError> {
    injected_service(service)?.load_task_graph(request)
}

/// Appends a handoff through an injected orchestration service.
///
/// # Errors
///
/// Returns [`OrchestrationError::MissingService`] when no service was supplied,
/// or the service's typed fail-closed error.
pub fn append_orchestration_handoff(
    service: Option<&mut dyn OrchestrationService>,
    request: &OrchestrationHandoffAppendRequest,
) -> Result<OrchestrationReceipt, OrchestrationError> {
    injected_service(service)?.append_handoff(request)
}

/// Admits scheduling through an injected orchestration service.
///
/// # Errors
///
/// Returns [`OrchestrationError::MissingService`] when no service was supplied,
/// or the service's typed fail-closed error.
pub fn admit_orchestration_scheduling(
    service: Option<&mut dyn OrchestrationService>,
    request: &OrchestrationSchedulingAdmissionRequest,
) -> Result<OrchestrationSchedulingAdmission, OrchestrationError> {
    injected_service(service)?.admit_scheduling(request)
}

/// Requests task cancellation through an injected orchestration service.
///
/// # Errors
///
/// Returns [`OrchestrationError::MissingService`] when no service was supplied,
/// or the service's typed fail-closed error.
pub fn cancel_orchestration_task(
    service: Option<&mut dyn OrchestrationService>,
    request: &OrchestrationCancellationRequest,
) -> Result<OrchestrationCancellationOutcome, OrchestrationError> {
    injected_service(service)?.cancel_task(request)
}

/// Records lifecycle evidence through an injected orchestration service.
///
/// # Errors
///
/// Returns [`OrchestrationError::MissingService`] when no service was supplied,
/// or the service's typed fail-closed error.
pub fn record_orchestration_lifecycle_evidence(
    service: Option<&mut dyn OrchestrationService>,
    request: &OrchestrationLifecycleEvidenceRequest,
) -> Result<OrchestrationReceipt, OrchestrationError> {
    injected_service(service)?.record_lifecycle_evidence(request)
}

/// Builds a BLAKE3 safe hash from a deterministic list of safe parts.
#[must_use]
pub fn blake3_orchestration_hash(parts: &[&str]) -> crate::SafeHash {
    let mut canonical = String::new();
    for part in parts {
        canonical.push_str(part);
        canonical.push_str(HASH_SEPARATOR);
    }
    crate::SafeHash::from_blake3_bytes(canonical.as_bytes())
}

/// Validates free-form receipt text before it is used as durable evidence.
///
/// # Errors
///
/// Returns [`OrchestrationError::UnsafeEvidence`] when raw prompts, provider
/// payloads, credentials, full paths, hidden reasoning, unbounded logs, host
/// database rows, or raw task bodies are present.
pub fn validate_orchestration_receipt_text(text: &str) -> Result<(), OrchestrationError> {
    let normalized = safe_text(SUMMARY_FIELD, text)?;
    reject_unsafe_text(SUMMARY_FIELD, normalized.as_str())
}

fn injected_service(
    service: Option<&mut dyn OrchestrationService>,
) -> Result<&mut dyn OrchestrationService, OrchestrationError> {
    service.ok_or(OrchestrationError::MissingService)
}

fn validate_task_graph(tasks: &[OrchestrationTaskDraft]) -> Result<(), OrchestrationError> {
    if tasks.is_empty() {
        return Err(OrchestrationError::EmptyCollection { field: "tasks" });
    }
    validate_collection_count(
        crate::validation::FieldName::new("orchestration_tasks"),
        tasks.len(),
        MAX_ORCHESTRATION_TASKS,
    )?;
    validate_unique_task_ids(tasks)?;
    validate_graph_links(tasks)?;
    validate_tenant_dependencies(tasks)?;
    Ok(())
}

fn validate_unique_task_ids(tasks: &[OrchestrationTaskDraft]) -> Result<(), OrchestrationError> {
    let mut seen = BTreeSet::new();
    for task in tasks {
        if !seen.insert(task.task_id()) {
            return Err(OrchestrationError::InvalidGraph);
        }
    }
    Ok(())
}

fn validate_graph_links(tasks: &[OrchestrationTaskDraft]) -> Result<(), OrchestrationError> {
    assert!(!tasks.is_empty());
    assert!(tasks.len() <= MAX_ORCHESTRATION_TASKS);
    for task in tasks {
        validate_unique_links(task.parent_ids())?;
        validate_unique_links(task.child_ids())?;
        for parent_id in task.parent_ids() {
            let Some(parent) = find_task(tasks, parent_id.as_str()) else {
                return Err(OrchestrationError::InvalidGraph);
            };
            if !parent
                .child_ids()
                .iter()
                .any(|child| child == task.task_id())
            {
                return Err(OrchestrationError::InvalidGraph);
            }
        }
        for child_id in task.child_ids() {
            let Some(child) = find_task(tasks, child_id.as_str()) else {
                return Err(OrchestrationError::InvalidGraph);
            };
            if !child
                .parent_ids()
                .iter()
                .any(|parent| parent == task.task_id())
            {
                return Err(OrchestrationError::InvalidGraph);
            }
        }
    }
    Ok(())
}

fn validate_unique_links(links: &[String]) -> Result<(), OrchestrationError> {
    let mut seen = BTreeSet::new();
    for link in links {
        if !seen.insert(link.as_str()) {
            return Err(OrchestrationError::InvalidGraph);
        }
    }
    Ok(())
}

fn validate_tenant_dependencies(
    tasks: &[OrchestrationTaskDraft],
) -> Result<(), OrchestrationError> {
    for task in tasks {
        for parent_id in task.parent_ids() {
            let Some(parent) = find_task(tasks, parent_id.as_str()) else {
                return Err(OrchestrationError::InvalidGraph);
            };
            if tenant_mismatch(task.tenant_label(), parent.tenant_label()) {
                return Err(OrchestrationError::CrossTenantLeak);
            }
        }
    }
    Ok(())
}

fn find_task<'a>(
    tasks: &'a [OrchestrationTaskDraft],
    task_id: &str,
) -> Option<&'a OrchestrationTaskDraft> {
    tasks.iter().find(|task| task.task_id() == task_id)
}

fn tenant_mismatch(left: Option<&str>, right: Option<&str>) -> bool {
    matches!((left, right), (Some(left), Some(right)) if left != right)
}

fn safe_text(
    field: crate::validation::FieldName,
    value: &str,
) -> Result<String, OrchestrationError> {
    let normalized = crate::validation::normalized_adapter_field(field, value)?;
    reject_unsafe_text(field, normalized.as_str())?;
    Ok(normalized)
}

fn optional_safe_text(
    field: crate::validation::FieldName,
    value: Option<&str>,
) -> Result<Option<String>, OrchestrationError> {
    value.map(|raw| safe_text(field, raw)).transpose()
}

fn safe_text_list(
    field: crate::validation::FieldName,
    values: &[&str],
    allow_empty: bool,
) -> Result<Vec<String>, OrchestrationError> {
    if !allow_empty && values.is_empty() {
        return Err(OrchestrationError::EmptyCollection {
            field: field.as_str(),
        });
    }
    validate_collection_count(field, values.len(), MAX_ORCHESTRATION_LINKS)?;
    values.iter().map(|value| safe_text(field, value)).collect()
}

#[cfg_attr(animus_octet, allow(tigerstyle::unbounded_collection_growth))]
fn safe_metadata(
    metadata: &BTreeMap<String, String>,
) -> Result<BTreeMap<String, String>, OrchestrationError> {
    validate_collection_count(
        METADATA_FIELD,
        metadata.len(),
        MAX_ORCHESTRATION_METADATA_FIELDS,
    )?;
    let mut safe = BTreeMap::new();
    for (key, value) in metadata {
        let safe_key = safe_text(METADATA_FIELD, key.as_str())?;
        let safe_value = safe_text(METADATA_FIELD, value.as_str())?;
        safe.insert(safe_key, safe_value);
    }
    Ok(safe)
}

fn reject_unsafe_text(
    field: crate::validation::FieldName,
    value: &str,
) -> Result<(), OrchestrationError> {
    for marker in UNSAFE_TEXT_MARKERS {
        if contains_unsafe_marker(value, marker) {
            return Err(OrchestrationError::UnsafeEvidence {
                field: field.as_str(),
                marker,
            });
        }
    }
    Ok(())
}

#[cfg_attr(animus_octet, allow(tigerstyle::ambiguous_params))]
fn contains_unsafe_marker(value: &str, marker: &'static str) -> bool {
    if marker == UNSAFE_OPENAI_KEY_PREFIX {
        return value.starts_with(marker) || value.contains(" sk-");
    }
    value.contains(marker)
}

#[cfg_attr(animus_octet, allow(tigerstyle::ambiguous_params))]
fn validate_collection_count(
    field: crate::validation::FieldName,
    actual_items: usize,
    max_items: usize,
) -> Result<(), OrchestrationError> {
    if actual_items > max_items {
        return Err(OrchestrationError::TooManyItems {
            field: field.as_str(),
            max_items,
            actual_items,
        });
    }
    Ok(())
}

#[cfg_attr(animus_octet, allow(tigerstyle::ambiguous_params))]
fn validate_minimum(
    field: &'static str,
    actual_value: u64,
    min_value: u64,
) -> Result<(), OrchestrationError> {
    if actual_value < min_value {
        return Err(OrchestrationError::NonMonotonicSequence {
            field,
            min_value,
            actual_value,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg_attr(animus_octet, allow(tigerstyle::non_trait_imports))]
    use alloc::vec;

    const BOARD: &str = "board-main";
    const TASK: &str = "task-main";
    const CHILD: &str = "task-child";
    const ACTOR: &str = "actor-host";
    const POLICY: &str = "policy-strict";
    const TENANT_ALPHA: &str = "tenant-alpha";
    const TENANT_BETA: &str = "tenant-beta";
    const ROOT: &str = "workspace-root";
    const TARGET: &str = "target-src";
    const GUARDED_TARGET: &str = "target-release";
    const APPROVAL: &str = "approval-release";
    const BODY: &str = "safe bounded task summary";
    const TITLE: &str = "safe title";
    const HANDOFF: &str = "handoff-main";
    const REDACTION: &str = "payload_not_captured";
    const EXPECTED_SINGLE_ITEM: usize = 1;
    const EXPECTED_TWO_ITEMS: usize = 2;
    const FIRST_SEQUENCE: u64 = 1;
    const FIRST_REVISION: u64 = 1;

    #[derive(Default)]
    struct NoopHost {
        appended: usize,
        recorded: usize,
    }

    impl OrchestrationService for NoopHost {
        fn load_task_graph(
            &mut self,
            _request: &OrchestrationTaskGraphLoadRequest,
        ) -> Result<OrchestrationTaskGraphDraft, OrchestrationError> {
            valid_graph()
        }

        fn append_handoff(
            &mut self,
            request: &OrchestrationHandoffAppendRequest,
        ) -> Result<OrchestrationReceipt, OrchestrationError> {
            assert_eq!(request.handoff().handoff_id(), HANDOFF);
            self.appended = self
                .appended
                .checked_add(EXPECTED_SINGLE_ITEM)
                .expect("fixture append count remains bounded");
            safe_receipt(OrchestrationOperationKind::AppendHandoff)
        }

        fn admit_scheduling(
            &mut self,
            request: &OrchestrationSchedulingAdmissionRequest,
        ) -> Result<OrchestrationSchedulingAdmission, OrchestrationError> {
            assert_eq!(request.operation(), OrchestrationSchedulingOperation::Start);
            Ok(OrchestrationSchedulingAdmission::Admitted)
        }

        fn cancel_task(
            &mut self,
            request: &OrchestrationCancellationRequest,
        ) -> Result<OrchestrationCancellationOutcome, OrchestrationError> {
            assert_eq!(request.reason_label(), "operator-requested");
            Ok(OrchestrationCancellationOutcome::Cancelled)
        }

        fn record_lifecycle_evidence(
            &mut self,
            request: &OrchestrationLifecycleEvidenceRequest,
        ) -> Result<OrchestrationReceipt, OrchestrationError> {
            assert_eq!(
                request.evidence().kind(),
                OrchestrationLifecycleEvidenceKind::Heartbeat
            );
            self.recorded = self
                .recorded
                .checked_add(EXPECTED_SINGLE_ITEM)
                .expect("fixture record count remains bounded");
            safe_receipt(OrchestrationOperationKind::RecordLifecycleEvidence)
        }
    }

    struct FailingHost {
        error: OrchestrationError,
    }

    impl OrchestrationService for FailingHost {
        fn load_task_graph(
            &mut self,
            _request: &OrchestrationTaskGraphLoadRequest,
        ) -> Result<OrchestrationTaskGraphDraft, OrchestrationError> {
            Err(self.error.clone())
        }

        fn append_handoff(
            &mut self,
            _request: &OrchestrationHandoffAppendRequest,
        ) -> Result<OrchestrationReceipt, OrchestrationError> {
            Err(self.error.clone())
        }

        fn admit_scheduling(
            &mut self,
            _request: &OrchestrationSchedulingAdmissionRequest,
        ) -> Result<OrchestrationSchedulingAdmission, OrchestrationError> {
            Err(self.error.clone())
        }

        fn cancel_task(
            &mut self,
            _request: &OrchestrationCancellationRequest,
        ) -> Result<OrchestrationCancellationOutcome, OrchestrationError> {
            Err(self.error.clone())
        }

        fn record_lifecycle_evidence(
            &mut self,
            _request: &OrchestrationLifecycleEvidenceRequest,
        ) -> Result<OrchestrationReceipt, OrchestrationError> {
            Err(self.error.clone())
        }
    }

    #[test]
    fn service_contract_load_append_schedule_cancel_and_receipt_pass() {
        // r[verify agent_harness_runtime.orchestration_adapters.service_contract]
        // r[verify agent_harness_runtime.orchestration_adapters.safe_evidence]
        // r[verify agent_harness_runtime.orchestration_adapters.verification]
        let mut host = NoopHost::default();
        let load_request = load_request();
        let graph = load_orchestration_task_graph(Some(&mut host), &load_request)
            .expect("service should load graph");
        assert_eq!(graph.tasks().len(), EXPECTED_TWO_ITEMS);

        let append_request = append_request();
        let append_receipt = append_orchestration_handoff(Some(&mut host), &append_request)
            .expect("service should append handoff");
        assert_eq!(
            append_receipt.operation(),
            OrchestrationOperationKind::AppendHandoff
        );
        assert_eq!(
            append_receipt.redaction_markers().len(),
            EXPECTED_SINGLE_ITEM
        );
        assert_eq!(host.appended, EXPECTED_SINGLE_ITEM);

        let scheduling_request = OrchestrationSchedulingAdmissionRequest::new(
            BOARD,
            TASK,
            OrchestrationSchedulingOperation::Start,
            metadata(),
        )
        .expect("scheduling request should validate");
        let admission = admit_orchestration_scheduling(Some(&mut host), &scheduling_request)
            .expect("scheduling should be admitted");
        assert_eq!(admission, OrchestrationSchedulingAdmission::Admitted);

        let cancellation_request =
            OrchestrationCancellationRequest::new(BOARD, TASK, "operator-requested", metadata())
                .expect("cancellation request should validate");
        let cancellation = cancel_orchestration_task(Some(&mut host), &cancellation_request)
            .expect("cancellation should be admitted");
        assert_eq!(cancellation, OrchestrationCancellationOutcome::Cancelled);

        let evidence_hash = blake3_orchestration_hash(&["heartbeat", TASK]);
        let evidence = OrchestrationLifecycleEvidence::new(&OrchestrationLifecycleEvidenceInput {
            kind: OrchestrationLifecycleEvidenceKind::Heartbeat,
            actor_label: ACTOR,
            sequence: FIRST_SEQUENCE,
            summary: "worker heartbeat redacted",
            evidence_hash: evidence_hash.as_str(),
            redaction_markers: &[REDACTION],
        })
        .expect("lifecycle evidence should validate");
        let evidence_request =
            OrchestrationLifecycleEvidenceRequest::new(BOARD, TASK, evidence, metadata())
                .expect("evidence request should validate");
        let receipt = record_orchestration_lifecycle_evidence(Some(&mut host), &evidence_request)
            .expect("evidence should record");
        assert_eq!(receipt.status(), OrchestrationStatusClass::Recorded);
        assert_eq!(host.recorded, EXPECTED_SINGLE_ITEM);
        assert!(validate_orchestration_receipt_text("status=recorded hash=redacted").is_ok());
    }

    #[test]
    fn fail_closed_for_missing_denied_unavailable_and_rejected_services() {
        // r[verify agent_harness_runtime.orchestration_adapters.fail_closed]
        // r[verify agent_harness_runtime.orchestration_adapters.verification]
        let missing = load_orchestration_task_graph(None, &load_request())
            .expect_err("missing service must fail closed");
        assert_eq!(missing, OrchestrationError::MissingService);

        let mut denied = FailingHost {
            error: OrchestrationError::CapabilityDenied,
        };
        let denied_error = load_orchestration_task_graph(Some(&mut denied), &load_request())
            .expect_err("denied service must fail closed");
        assert_eq!(denied_error, OrchestrationError::CapabilityDenied);

        let mut unavailable = FailingHost {
            error: OrchestrationError::BackendUnavailable,
        };
        let unavailable_error =
            append_orchestration_handoff(Some(&mut unavailable), &append_request())
                .expect_err("unavailable service must fail closed");
        assert_eq!(unavailable_error, OrchestrationError::BackendUnavailable);

        let mut rejected_host = FailingHost {
            error: OrchestrationError::UnsupportedOperation,
        };
        let scheduling_request = OrchestrationSchedulingAdmissionRequest::new(
            BOARD,
            TASK,
            OrchestrationSchedulingOperation::Promote,
            metadata(),
        )
        .expect("scheduling request should validate");
        let rejected_error =
            admit_orchestration_scheduling(Some(&mut rejected_host), &scheduling_request)
                .expect_err("unsupported operation must fail closed");
        assert_eq!(rejected_error, OrchestrationError::UnsupportedOperation);
    }

    #[test]
    fn invalid_graph_scope_and_redaction_bounds_fail_closed() {
        // r[verify agent_harness_runtime.orchestration_adapters.fail_closed]
        // r[verify agent_harness_runtime.orchestration_adapters.safe_evidence]
        // r[verify agent_harness_runtime.orchestration_adapters.verification]
        let invalid_link = invalid_graph().expect_err("one-sided task link should fail");
        assert_eq!(invalid_link, OrchestrationError::InvalidGraph);

        let duplicate = duplicate_graph().expect_err("duplicate task identity should fail");
        assert_eq!(duplicate, OrchestrationError::InvalidGraph);

        let redaction_marker_overflow_count = MAX_ORCHESTRATION_REDACTIONS
            .checked_add(1)
            .expect("redaction overflow fixture count remains bounded");
        let redaction_marker_overflow = alloc::vec![REDACTION; redaction_marker_overflow_count];
        let redaction_overflow_error =
            OrchestrationLifecycleEvidence::new(&OrchestrationLifecycleEvidenceInput {
                kind: OrchestrationLifecycleEvidenceKind::Heartbeat,
                actor_label: ACTOR,
                sequence: FIRST_SEQUENCE,
                summary: "worker heartbeat redacted",
                evidence_hash: blake3_orchestration_hash(&["heartbeat", TASK]).as_str(),
                redaction_markers: &redaction_marker_overflow,
            })
            .expect_err("oversized redaction list should fail");
        assert!(matches!(
            redaction_overflow_error,
            OrchestrationError::TooManyItems { .. }
        ));

        let cross_tenant = cross_tenant_graph().expect_err("cross tenant dependency should fail");
        assert_eq!(cross_tenant, OrchestrationError::CrossTenantLeak);

        let scope = workspace_scope();
        let out_of_scope = OrchestrationMutationIntent::new("target-secret", None)
            .and_then(|mutation| validate_orchestration_workspace_mutation(&scope, &mutation))
            .expect_err("out-of-scope target must fail closed");
        assert_eq!(out_of_scope, OrchestrationError::OutOfScopeMutation);

        let guarded = OrchestrationMutationIntent::new(GUARDED_TARGET, None)
            .and_then(|mutation| validate_orchestration_workspace_mutation(&scope, &mutation))
            .expect("guarded target should request approval");
        assert_eq!(
            guarded,
            OrchestrationWorkspaceAdmission::RequiresHostApproval
        );
        let approved = OrchestrationMutationIntent::new(GUARDED_TARGET, Some(APPROVAL))
            .and_then(|mutation| validate_orchestration_workspace_mutation(&scope, &mutation))
            .expect("approved guarded target should pass");
        assert_eq!(approved, OrchestrationWorkspaceAdmission::Allowed);
    }

    #[test]
    fn unsafe_handoff_receipt_leak_and_schema_mismatch_fail_closed() {
        // r[verify agent_harness_runtime.orchestration_adapters.safe_evidence]
        // r[verify agent_harness_runtime.orchestration_adapters.verification]
        let unsafe_handoff = OrchestrationContextHandoff::new(&OrchestrationContextHandoffDraft {
            handoff_id: HANDOFF,
            task_id: TASK,
            tenant_label: Some(TENANT_ALPHA),
            body: "RAW_PROMPT: hidden",
            sequence: FIRST_SEQUENCE,
            metadata: &empty_metadata(),
        })
        .expect_err("raw prompt marker should fail");
        assert!(matches!(
            unsafe_handoff,
            OrchestrationError::UnsafeEvidence { .. }
        ));

        let leak = validate_orchestration_receipt_text("HOST_DATABASE_ROW: private row")
            .expect_err("host database evidence should fail");
        assert!(matches!(leak, OrchestrationError::UnsafeEvidence { .. }));

        let bad_schema = OrchestrationReceipt::new(&OrchestrationReceiptDraft {
            contract_identity: "animus.orchestration-receipt.v0",
            operation: OrchestrationOperationKind::LoadTaskGraph,
            status: OrchestrationStatusClass::Accepted,
            family_label: ORCHESTRATION_FAMILY_LABEL,
            board_label: BOARD,
            task_label: Some(TASK),
            policy_label: POLICY,
            redaction_markers: &[REDACTION],
            item_count: EXPECTED_SINGLE_ITEM,
            dto_hash: blake3_orchestration_hash(&["dto"]).as_str(),
            summary: Some("safe summary"),
        })
        .expect_err("schema mismatch should fail");
        assert_eq!(bad_schema, OrchestrationError::SchemaMismatch);
    }

    fn valid_graph() -> Result<OrchestrationTaskGraphDraft, OrchestrationError> {
        let parent = task(TASK, &[], &[CHILD], Some(TENANT_ALPHA))?;
        let child = task(CHILD, &[TASK], &[], Some(TENANT_ALPHA))?;
        OrchestrationTaskGraphDraft::new(vec![parent, child])
    }

    fn invalid_graph() -> Result<OrchestrationTaskGraphDraft, OrchestrationError> {
        let parent = task(TASK, &[], &[CHILD], Some(TENANT_ALPHA))?;
        let child = task(CHILD, &[], &[], Some(TENANT_ALPHA))?;
        OrchestrationTaskGraphDraft::new(vec![parent, child])
    }

    fn cross_tenant_graph() -> Result<OrchestrationTaskGraphDraft, OrchestrationError> {
        let parent = task(TASK, &[], &[CHILD], Some(TENANT_ALPHA))?;
        let child = task(CHILD, &[TASK], &[], Some(TENANT_BETA))?;
        OrchestrationTaskGraphDraft::new(vec![parent, child])
    }

    fn duplicate_graph() -> Result<OrchestrationTaskGraphDraft, OrchestrationError> {
        let first = task(TASK, &[], &[], Some(TENANT_ALPHA))?;
        let second = task(TASK, &[], &[], Some(TENANT_ALPHA))?;
        OrchestrationTaskGraphDraft::new(vec![first, second])
    }

    fn task(
        task_id: &str,
        parent_ids: &[&str],
        child_ids: &[&str],
        tenant_label: Option<&str>,
    ) -> Result<OrchestrationTaskDraft, OrchestrationError> {
        OrchestrationTaskDraft::new(OrchestrationTaskDraftInput {
            task_id,
            title: TITLE,
            body: BODY,
            assignee_label: Some(ACTOR),
            state: OrchestrationTaskState::Ready,
            parent_ids,
            child_ids,
            priority: OrchestrationPriority::Normal,
            workspace_scope: Some(workspace_scope()),
            tenant_label,
            comments: &[],
            heartbeats: &[],
            block_records: &[],
            completion: None,
            metadata: &empty_metadata(),
            revision: FIRST_REVISION,
        })
    }

    fn workspace_scope() -> OrchestrationWorkspaceScope {
        OrchestrationWorkspaceScope::new(
            OrchestrationWorkspaceKind::Directory,
            ROOT,
            &[TARGET],
            &[GUARDED_TARGET],
        )
        .expect("workspace scope should validate")
    }

    fn metadata() -> OrchestrationOperationMetadata {
        OrchestrationOperationMetadata::new(ACTOR, POLICY, Some(TENANT_ALPHA))
            .expect("metadata should validate")
    }

    fn load_request() -> OrchestrationTaskGraphLoadRequest {
        OrchestrationTaskGraphLoadRequest::new(BOARD, metadata())
            .expect("load request should validate")
    }

    fn append_request() -> OrchestrationHandoffAppendRequest {
        let handoff = OrchestrationContextHandoff::new(&OrchestrationContextHandoffDraft {
            handoff_id: HANDOFF,
            task_id: TASK,
            tenant_label: Some(TENANT_ALPHA),
            body: "safe handoff summary",
            sequence: FIRST_SEQUENCE,
            metadata: &empty_metadata(),
        })
        .expect("handoff should validate");
        OrchestrationHandoffAppendRequest::new(handoff, metadata())
    }

    fn safe_receipt(
        operation: OrchestrationOperationKind,
    ) -> Result<OrchestrationReceipt, OrchestrationError> {
        assert!(!BOARD.is_empty());
        assert!(!TASK.is_empty());
        let dto_hash = blake3_orchestration_hash(&[operation.as_str(), BOARD, TASK]);
        let status = if operation == OrchestrationOperationKind::RecordLifecycleEvidence {
            OrchestrationStatusClass::Recorded
        } else {
            OrchestrationStatusClass::Accepted
        };
        OrchestrationReceipt::new(&OrchestrationReceiptDraft {
            contract_identity: ORCHESTRATION_RECEIPT_CONTRACT_IDENTITY,
            operation,
            status,
            family_label: ORCHESTRATION_FAMILY_LABEL,
            board_label: BOARD,
            task_label: Some(TASK),
            policy_label: POLICY,
            redaction_markers: &[REDACTION],
            item_count: EXPECTED_SINGLE_ITEM,
            dto_hash: dto_hash.as_str(),
            summary: Some("safe receipt summary"),
        })
    }

    fn empty_metadata() -> BTreeMap<String, String> {
        BTreeMap::new()
    }
}

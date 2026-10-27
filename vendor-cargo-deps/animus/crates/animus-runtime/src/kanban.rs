//! Pure Kanban-style orchestration DTOs and validation helpers.
//!
//! The module adapts durable task-graph ideas from Hermes without adopting a
//! board database, dispatcher, worker roster, process supervisor, filesystem
//! traversal, VCS command, or storage backend. Hosts may inject a board service;
//! reusable runtime logic operates only on already-loaded in-memory DTOs.

// Kanban orchestration remains in one compatibility facade until the archived Cairn API is split by owner.
#![allow(unknown_lints)]
#![cfg_attr(animus_octet, allow(tigerstyle::path_segment_repetition))]
const MAX_KANBAN_LABEL_CHARS: usize = 128;
const MAX_KANBAN_TEXT_CHARS: usize = 4_096;
const MAX_KANBAN_METADATA_FIELDS: usize = 32;
const MAX_KANBAN_TASKS: usize = 256;
const MAX_KANBAN_LINKS: usize = 64;
const MAX_KANBAN_RECORDS: usize = 32;
const MIN_KANBAN_SEQUENCE: u64 = 1;
const MIN_KANBAN_REVISION: u64 = 1;
const EMPTY_FIELD_REASON: &str = "field must be non-empty";
const MISSING_FIELD_REASON: &str = "required field is missing";
const TOO_LONG_REASON: &str = "field exceeds safe length limit";
const TOO_MANY_ITEMS_REASON: &str = "collection exceeds safe count limit";
const UNSAFE_TEXT_REASON: &str = "field contains unsafe raw or leaking marker";
const INVALID_LINK_REASON: &str = "task link must reference an existing reciprocal task";
const DUPLICATE_TASK_REASON: &str = "task identity must be unique";
const DUPLICATE_LINK_REASON: &str = "task links must be unique";
const CYCLE_REASON: &str = "task dependencies must be acyclic";
const CROSS_TENANT_REASON: &str = "cross-tenant dependency or handoff is not admitted";
const AMBIGUOUS_DEPENDENCY_REASON: &str = "dependency state cannot safely promote the child";
const STALE_COMPLETION_REASON: &str = "completion record does not match the task revision";
const OUT_OF_SCOPE_REASON: &str = "mutation target is outside declared workspace scope";
const MISSING_BOARD_SERVICE_REASON: &str =
    "Kanban orchestration requires an injected board service";
const ADAPTER_BRIDGE_ACTOR_LABEL: &str = "runtime-kanban-bridge";
const ADAPTER_BRIDGE_POLICY_LABEL: &str = "kanban-compatibility";
const ROOT_CAPABILITY_FIELD: &str = "workspace_scope.root_capability";
const WORKSPACE_TARGET_FIELD: &str = "workspace_scope.targets";
const TASK_ID_FIELD: &str = "task_id";
const TITLE_FIELD: &str = "title";
const BODY_FIELD: &str = "body";
const ASSIGNEE_FIELD: &str = "assignee_label";
const PARENT_FIELD: &str = "parent_ids";
const CHILD_FIELD: &str = "child_ids";
const TENANT_FIELD: &str = "tenant_label";
const WORKSPACE_FIELD: &str = "workspace_scope";
const METADATA_FIELD: &str = "metadata";
const REVISION_FIELD: &str = "revision";
const COMMENT_FIELD: &str = "comments";
const HEARTBEAT_FIELD: &str = "heartbeats";
const BLOCK_FIELD: &str = "block_records";
const COMPLETION_FIELD: &str = "completion";
const ACTOR_FIELD: &str = "actor_label";
const SEQUENCE_FIELD: &str = "sequence";
const SUMMARY_FIELD: &str = "summary";
const REASON_FIELD: &str = "reason";
const CONTEXT_FIELD: &str = "context";
const NEXT_ACTION_FIELD: &str = "next_action";
const BOARD_SERVICE_FIELD: &str = "kanban_board_service";
const MUTATION_TARGET_FIELD: &str = "mutation.target_label";
const MUTATION_APPROVAL_FIELD: &str = "mutation.approval_label";
const HANDOFF_ID_FIELD: &str = "handoff_id";
const HANDOFF_BODY_FIELD: &str = "handoff.body";

const UNSAFE_TEXT_MARKERS: &[&str] = &[
    "RAW_PROMPT:",
    "PROVIDER_PAYLOAD:",
    "UNBOUNDED_OUTPUT:",
    "BEGIN PRIVATE KEY",
    "API_KEY=",
    "AWS_SECRET_ACCESS_KEY",
    "sk-",
    "/home/",
];
const OPENAI_KEY_PREFIX_MARKER: &str = "sk-";

#[derive(Clone, Copy)]
struct StringInput<'a> {
    field: &'static str,
    value: &'a str,
}

#[derive(Clone, Copy)]
struct LimitedStringInput<'a> {
    field: &'static str,
    value: &'a str,
    limit: usize,
}

#[derive(Clone, Copy)]
struct TaskLookup<'a> {
    tasks: &'a [KanbanTask],
    task_id: &'a str,
    field: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct CycleFrame<'a> {
    task_id: &'a str,
    next_parent_index: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum ParentPromotion {
    Complete,
    Waiting { parent_id: String },
}

macro_rules! label {
    ($field:expr, $value:expr) => {
        validate_label(StringInput {
            field: $field,
            value: $value,
        })
    };
}

macro_rules! text {
    ($field:expr, $value:expr) => {
        validate_text(StringInput {
            field: $field,
            value: $value,
        })
    };
}

/// Error class for Kanban task-graph validation and injected service calls.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KanbanErrorKind {
    /// Required data was omitted.
    MissingField,
    /// Text or a collection was empty after validation.
    EmptyField,
    /// Text exceeded the bounded safe length.
    FieldTooLong,
    /// A collection exceeded its bounded safe length.
    TooManyItems,
    /// Text included raw private or unsafe material.
    UnsafeText,
    /// Two tasks declared the same identity.
    DuplicateTaskId,
    /// A task link referenced an unknown task.
    UnknownTaskId,
    /// A task link was duplicate, one-sided, or otherwise malformed.
    InvalidLink,
    /// Dependency links formed a cycle.
    Cycle,
    /// A dependency or handoff crossed tenant boundaries.
    CrossTenantLeak,
    /// A dependency had a state that cannot safely promote a child.
    AmbiguousDependencyState,
    /// A completion record was stale or absent for a completed dependency.
    StaleCompletion,
    /// A mutation target was outside the declared workspace.
    OutOfScopeMutation,
    /// A mutation target requires host approval before dispatch.
    HostApprovalRequired,
    /// No host board service was supplied for a shell-owned operation.
    BoardServiceUnavailable,
    /// The injected board service failed safely.
    InjectedServiceFailure,
}

/// Safe Kanban validation diagnostic.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KanbanError {
    kind: KanbanErrorKind,
    field: &'static str,
    reason: String,
}

impl KanbanError {
    fn new(kind: KanbanErrorKind, field: &'static str, reason: impl Into<String>) -> Self {
        Self {
            kind,
            field,
            reason: reason.into(),
        }
    }

    /// Returns the error class.
    #[must_use]
    pub const fn kind(&self) -> KanbanErrorKind {
        self.kind
    }

    /// Returns the safe field label.
    #[must_use]
    pub const fn field(&self) -> &'static str {
        self.field
    }

    /// Returns the bounded safe reason string.
    #[must_use]
    pub fn reason(&self) -> &str {
        self.reason.as_str()
    }
}

impl std::fmt::Display for KanbanError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "kanban field `{}` failed ({:?}): {}",
            self.field, self.kind, self.reason
        )
    }
}

impl std::error::Error for KanbanError {}

/// Lifecycle state for a durable orchestration task.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KanbanTaskState {
    /// Task exists but is not ready to run.
    Draft,
    /// Task can be scheduled by a host.
    Ready,
    /// Task is currently assigned or running.
    InProgress,
    /// Task is blocked on an upstream or human decision.
    Blocked,
    /// Task completed with an accepted completion record.
    Completed,
    /// Task was cancelled and must not promote dependants automatically.
    Cancelled,
}

/// Priority metadata for deterministic host scheduling inputs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KanbanPriority {
    /// Low-priority durable task.
    Low,
    /// Default-priority durable task.
    Normal,
    /// High-priority durable task.
    High,
    /// Urgent task supplied by the host.
    Urgent,
}

/// Explicit workspace capability class declared by a task.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KanbanWorkspaceKind {
    /// Scratch workspace with a host-supplied root capability.
    Scratch,
    /// Directory workspace identified by safe host labels.
    Directory,
    /// VCS-neutral worktree-style workspace identified by safe host labels.
    Worktree,
}

/// Explicit workspace scope for a task.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KanbanWorkspaceScope {
    /// Workspace class.
    pub kind: KanbanWorkspaceKind,
    /// Safe label for the host-supplied root capability.
    pub root_capability: String,
    /// Safe labels that may be mutated without additional approval.
    pub writable_targets: Vec<String>,
    /// Safe labels that require separate host approval before mutation.
    pub approval_required_targets: Vec<String>,
}

/// Mutation attempt checked against a declared workspace scope.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KanbanMutationIntent {
    /// Safe target label selected by the host.
    pub target_label: String,
    /// Optional safe host-approval label for guarded targets.
    pub approval_label: Option<String>,
}

/// Workspace mutation admission result.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KanbanWorkspaceAdmission {
    /// Mutation is within declared writable targets.
    Allowed,
    /// Mutation targets a guarded label and needs separate host approval.
    RequiresHostApproval,
}

/// Comment record attached to a task.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KanbanComment {
    /// Actor label supplied by the shell.
    pub actor_label: String,
    /// Monotonic sequence supplied by the shell.
    pub sequence: u64,
    /// Bounded comment body.
    pub body: String,
    /// Tenant label preserved when supplied by the host.
    pub tenant_label: Option<String>,
    /// Bounded safe metadata.
    pub metadata: std::collections::BTreeMap<String, String>,
}

/// Heartbeat record attached to a task.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KanbanHeartbeat {
    /// Actor label supplied by the shell.
    pub actor_label: String,
    /// Monotonic sequence supplied by the shell.
    pub sequence: u64,
    /// Bounded status summary.
    pub summary: String,
    /// Tenant label preserved when supplied by the host.
    pub tenant_label: Option<String>,
    /// Bounded safe metadata.
    pub metadata: std::collections::BTreeMap<String, String>,
}

/// Block record attached to a task.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KanbanBlockRecord {
    /// Actor label supplied by the shell.
    pub actor_label: String,
    /// Monotonic sequence supplied by the shell.
    pub sequence: u64,
    /// Short actionable block reason.
    pub reason: String,
    /// Optional bounded context comment.
    pub context: Option<String>,
    /// Optional next-action hint.
    pub next_action: Option<String>,
    /// Tenant label preserved when supplied by the host.
    pub tenant_label: Option<String>,
    /// Bounded safe metadata.
    pub metadata: std::collections::BTreeMap<String, String>,
}

/// Completion record attached to a completed task.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KanbanCompletion {
    /// Actor label supplied by the shell.
    pub actor_label: String,
    /// Monotonic sequence supplied by the shell.
    pub sequence: u64,
    /// Bounded completion summary.
    pub summary: String,
    /// Task revision observed by the completing worker.
    pub task_revision: u64,
    /// Tenant label preserved when supplied by the host.
    pub tenant_label: Option<String>,
    /// Bounded safe completion metadata.
    pub metadata: std::collections::BTreeMap<String, String>,
}

/// Draft task supplied by a host before pure validation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KanbanTaskDraft {
    /// Stable task identity.
    pub task_id: String,
    /// Human-readable task title.
    pub title: String,
    /// Bounded task body.
    pub body: String,
    /// Optional worker or assignee label.
    pub assignee_label: Option<String>,
    /// Lifecycle state supplied by the host snapshot.
    pub state: KanbanTaskState,
    /// Parent dependencies by task ID.
    pub parent_ids: Vec<String>,
    /// Child task links by task ID.
    pub child_ids: Vec<String>,
    /// Priority metadata.
    pub priority: KanbanPriority,
    /// Explicit workspace scope.
    pub workspace_scope: Option<KanbanWorkspaceScope>,
    /// Optional tenant label preserved through handoffs.
    pub tenant_label: Option<String>,
    /// Comments attached to the task.
    pub comments: Vec<KanbanComment>,
    /// Heartbeats attached to the task.
    pub heartbeats: Vec<KanbanHeartbeat>,
    /// Block records attached to the task.
    pub block_records: Vec<KanbanBlockRecord>,
    /// Completion record for completed tasks.
    pub completion: Option<KanbanCompletion>,
    /// Bounded safe priority or host metadata.
    pub metadata: std::collections::BTreeMap<String, String>,
    /// Host-supplied task revision.
    pub revision: u64,
}

/// Validated task graph snapshot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KanbanTaskGraph {
    tasks: Vec<KanbanTask>,
}

impl KanbanTaskGraph {
    /// Returns tasks sorted by task identity.
    #[must_use]
    pub fn tasks(&self) -> &[KanbanTask] {
        self.tasks.as_slice()
    }

    /// Returns a validated task by identity.
    #[must_use]
    pub fn task(&self, task_id: &str) -> Option<&KanbanTask> {
        self.tasks.iter().find(|task| task.task_id == task_id)
    }
}

/// Validated Kanban task.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KanbanTask {
    task_id: String,
    title: String,
    body: String,
    assignee_label: Option<String>,
    state: KanbanTaskState,
    parent_ids: Vec<String>,
    child_ids: Vec<String>,
    priority: KanbanPriority,
    workspace_scope: KanbanWorkspaceScope,
    tenant_label: Option<String>,
    comments: Vec<KanbanComment>,
    heartbeats: Vec<KanbanHeartbeat>,
    block_records: Vec<KanbanBlockRecord>,
    completion: Option<KanbanCompletion>,
    metadata: std::collections::BTreeMap<String, String>,
    revision: u64,
}

impl KanbanTask {
    /// Returns the task identity.
    #[must_use]
    pub fn task_id(&self) -> &str {
        self.task_id.as_str()
    }

    /// Returns the task title.
    #[must_use]
    pub fn title(&self) -> &str {
        self.title.as_str()
    }

    /// Returns the task body.
    #[must_use]
    pub fn body(&self) -> &str {
        self.body.as_str()
    }

    /// Returns the optional assignee label.
    #[must_use]
    pub fn assignee_label(&self) -> Option<&str> {
        self.assignee_label.as_deref()
    }

    /// Returns the lifecycle state.
    #[must_use]
    pub const fn state(&self) -> KanbanTaskState {
        self.state
    }

    /// Returns parent dependency task IDs.
    #[must_use]
    pub fn parent_ids(&self) -> &[String] {
        self.parent_ids.as_slice()
    }

    /// Returns child task IDs.
    #[must_use]
    pub fn child_ids(&self) -> &[String] {
        self.child_ids.as_slice()
    }

    /// Returns priority metadata.
    #[must_use]
    pub const fn priority(&self) -> KanbanPriority {
        self.priority
    }

    /// Returns the explicit workspace scope.
    #[must_use]
    pub const fn workspace_scope(&self) -> &KanbanWorkspaceScope {
        &self.workspace_scope
    }

    /// Returns the optional tenant label.
    #[must_use]
    pub fn tenant_label(&self) -> Option<&str> {
        self.tenant_label.as_deref()
    }

    /// Returns task comments.
    #[must_use]
    pub fn comments(&self) -> &[KanbanComment] {
        self.comments.as_slice()
    }

    /// Returns task heartbeats.
    #[must_use]
    pub fn heartbeats(&self) -> &[KanbanHeartbeat] {
        self.heartbeats.as_slice()
    }

    /// Returns task block records.
    #[must_use]
    pub fn block_records(&self) -> &[KanbanBlockRecord] {
        self.block_records.as_slice()
    }

    /// Returns the completion record when present.
    #[must_use]
    pub const fn completion(&self) -> Option<&KanbanCompletion> {
        self.completion.as_ref()
    }

    /// Returns bounded safe task metadata.
    #[must_use]
    pub const fn metadata(&self) -> &std::collections::BTreeMap<String, String> {
        &self.metadata
    }

    /// Returns the task revision.
    #[must_use]
    pub const fn revision(&self) -> u64 {
        self.revision
    }
}

/// Draft task graph supplied by a host before validation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KanbanTaskGraphDraft {
    /// Task snapshot entries.
    pub tasks: Vec<KanbanTaskDraft>,
}

/// Dependency-promotion result for a selected task.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KanbanPromotionDecision {
    /// Child task under evaluation.
    pub task_id: String,
    /// Deterministic promotion outcome.
    pub outcome: KanbanPromotionOutcome,
}

/// Deterministic dependency-promotion outcome.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum KanbanPromotionOutcome {
    /// All parents completed with accepted completion records.
    Ready,
    /// Some parents are still in non-terminal active states.
    Waiting {
        /// Sorted parent IDs that still block promotion.
        parent_ids: Vec<String>,
    },
}

/// Cross-tenant context handling policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KanbanTenantPolicy {
    /// Reject the context assembly request on the first cross-tenant handoff.
    RejectCrossTenant,
    /// Omit cross-tenant bodies and report quarantined handoff IDs.
    QuarantineCrossTenant,
}

/// Handoff that may be projected into downstream task context.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KanbanContextHandoff {
    /// Stable handoff identity.
    pub handoff_id: String,
    /// Source task identity.
    pub task_id: String,
    /// Tenant label preserved when supplied by the host.
    pub tenant_label: Option<String>,
    /// Bounded body eligible for model context only after policy admission.
    pub body: String,
    /// Monotonic sequence supplied by the shell.
    pub sequence: u64,
    /// Bounded safe metadata.
    pub metadata: std::collections::BTreeMap<String, String>,
}

/// Admitted context item with tenant label preserved.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KanbanContextItem {
    /// Source handoff identity.
    pub handoff_id: String,
    /// Tenant label preserved in context assembly.
    pub tenant_label: Option<String>,
    /// Bounded summary admitted into model-visible context.
    pub safe_summary: String,
}

/// Quarantined handoff marker that omits the source body.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KanbanQuarantinedHandoff {
    /// Source handoff identity.
    pub handoff_id: String,
    /// Tenant label that caused quarantine.
    pub tenant_label: Option<String>,
}

/// Context assembly output for a child task.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KanbanContextAssembly {
    /// Child task identity.
    pub child_task_id: String,
    /// Handoffs admitted into context.
    pub admitted: Vec<KanbanContextItem>,
    /// Cross-tenant handoffs quarantined without their bodies.
    pub quarantined: Vec<KanbanQuarantinedHandoff>,
}

/// Host-injected optional Kanban board service.
pub trait KanbanBoardService {
    /// Loads an in-memory graph draft for a safe board label.
    ///
    /// # Errors
    ///
    /// Returns [`KanbanError`] when the injected service cannot provide a safe
    /// graph draft.
    fn load_task_graph(&mut self, board_label: &str) -> Result<KanbanTaskGraphDraft, KanbanError>;

    /// Appends a handoff through host-owned storage.
    ///
    /// # Errors
    ///
    /// Returns [`KanbanError`] when the injected service rejects the handoff.
    fn append_handoff(&mut self, handoff: &KanbanContextHandoff) -> Result<(), KanbanError>;
}

/// Compatibility bridge from the public orchestration adapter family to the
/// existing runtime-local Kanban board-service seam.
#[cfg_attr(animus_octet, allow(tigerstyle::path_segment_repetition))]
pub struct OrchestrationAdapterBoardService<'a> {
    service: &'a mut dyn animus_adapters::orchestration::OrchestrationService,
}

impl<'a> OrchestrationAdapterBoardService<'a> {
    /// Builds a compatibility bridge over an injected orchestration service.
    #[must_use]
    pub const fn new(
        service: &'a mut dyn animus_adapters::orchestration::OrchestrationService,
    ) -> Self {
        Self { service }
    }
}

impl KanbanBoardService for OrchestrationAdapterBoardService<'_> {
    fn load_task_graph(&mut self, board_label: &str) -> Result<KanbanTaskGraphDraft, KanbanError> {
        let metadata = adapter_operation_metadata()?;
        let request = animus_adapters::orchestration::OrchestrationTaskGraphLoadRequest::new(
            board_label,
            metadata,
        )
        .map_err(adapter_error_to_kanban)?;
        let adapter_graph = animus_adapters::orchestration::load_orchestration_task_graph(
            Some(self.service),
            &request,
        )
        .map_err(adapter_error_to_kanban)?;
        Ok(adapter_graph_to_kanban(&adapter_graph))
    }

    fn append_handoff(&mut self, handoff: &KanbanContextHandoff) -> Result<(), KanbanError> {
        let metadata = adapter_operation_metadata()?;
        let adapter_handoff =
            kanban_handoff_to_adapter(handoff).map_err(adapter_error_to_kanban)?;
        let request = animus_adapters::orchestration::OrchestrationHandoffAppendRequest::new(
            adapter_handoff,
            metadata,
        );
        animus_adapters::orchestration::append_orchestration_handoff(Some(self.service), &request)
            .map(|_receipt| ())
            .map_err(adapter_error_to_kanban)
    }
}

/// Validates an in-memory Kanban task graph snapshot.
///
/// # Errors
///
/// Returns [`KanbanError`] when required fields are missing, text is unsafe,
/// links are malformed, dependencies are cyclic, tenants cross, or completion
/// records are stale.
pub fn validate_kanban_graph(draft: KanbanTaskGraphDraft) -> Result<KanbanTaskGraph, KanbanError> {
    let task_count = draft.tasks.len();
    validate_task_count(draft.tasks.as_slice())?;
    debug_assert!(task_count > 0);
    debug_assert!(task_count <= MAX_KANBAN_TASKS);
    let mut tasks = Vec::with_capacity(task_count);
    for task_draft in draft.tasks {
        tasks.push(validate_task(task_draft)?);
    }
    tasks.sort_by(|left, right| left.task_id.cmp(&right.task_id));
    validate_unique_task_ids(tasks.as_slice())?;
    validate_links(tasks.as_slice())?;
    validate_tenant_dependencies(tasks.as_slice())?;
    validate_acyclic(tasks.as_slice())?;
    Ok(KanbanTaskGraph { tasks })
}

/// Evaluates dependency promotion for a child task without mutating the graph.
///
/// # Errors
///
/// Returns [`KanbanError`] when the graph or target ID is invalid, a dependency
/// has an ambiguous terminal state, a completed parent has stale completion
/// evidence, or a cross-tenant handoff would leak.
pub fn evaluate_kanban_promotion(
    graph: &KanbanTaskGraph,
    task_id: &str,
) -> Result<KanbanPromotionDecision, KanbanError> {
    debug_assert!(graph.tasks().len() <= MAX_KANBAN_TASKS);
    debug_assert!(!task_id.trim().is_empty());
    let child = graph.task(task_id).ok_or_else(|| {
        KanbanError::new(
            KanbanErrorKind::UnknownTaskId,
            TASK_ID_FIELD,
            INVALID_LINK_REASON,
        )
    })?;
    let mut waiting_parent_ids = Vec::with_capacity(child.parent_ids().len());
    for parent_id in child.parent_ids() {
        let parent = graph.task(parent_id).ok_or_else(|| {
            KanbanError::new(
                KanbanErrorKind::UnknownTaskId,
                PARENT_FIELD,
                INVALID_LINK_REASON,
            )
        })?;
        match evaluate_parent_for_promotion(parent, child)? {
            ParentPromotion::Complete => {}
            ParentPromotion::Waiting { parent_id } => waiting_parent_ids.push(parent_id),
        }
    }
    waiting_parent_ids.sort();
    let outcome = if waiting_parent_ids.is_empty() {
        KanbanPromotionOutcome::Ready
    } else {
        KanbanPromotionOutcome::Waiting {
            parent_ids: waiting_parent_ids,
        }
    };
    Ok(KanbanPromotionDecision {
        task_id: String::from(child.task_id()),
        outcome,
    })
}

/// Checks a mutation against an explicit workspace scope.
///
/// # Errors
///
/// Returns [`KanbanError`] when labels are unsafe or the target is outside the
/// declared workspace. Guarded targets without approval return
/// [`KanbanWorkspaceAdmission::RequiresHostApproval`].
pub fn validate_kanban_workspace_mutation(
    scope: &KanbanWorkspaceScope,
    mutation: &KanbanMutationIntent,
) -> Result<KanbanWorkspaceAdmission, KanbanError> {
    debug_assert!(!scope.root_capability.trim().is_empty());
    debug_assert!(
        !scope.writable_targets.is_empty() || !scope.approval_required_targets.is_empty()
    );
    let target = label!(MUTATION_TARGET_FIELD, mutation.target_label.as_str())?;
    if scope
        .writable_targets
        .iter()
        .any(|allowed| allowed == &target)
    {
        return Ok(KanbanWorkspaceAdmission::Allowed);
    }
    if scope
        .approval_required_targets
        .iter()
        .any(|guarded| guarded == &target)
    {
        if let Some(approval_label) = mutation.approval_label.as_deref() {
            label!(MUTATION_APPROVAL_FIELD, approval_label)?;
            return Ok(KanbanWorkspaceAdmission::Allowed);
        }
        return Ok(KanbanWorkspaceAdmission::RequiresHostApproval);
    }
    Err(KanbanError::new(
        KanbanErrorKind::OutOfScopeMutation,
        MUTATION_TARGET_FIELD,
        OUT_OF_SCOPE_REASON,
    ))
}

/// Assembles tenant-safe downstream context from parent handoffs.
///
/// # Errors
///
/// Returns [`KanbanError`] when handoff fields are unsafe or the policy rejects a
/// cross-tenant handoff.
pub fn assemble_kanban_context(
    child: &KanbanTask,
    handoffs: &[KanbanContextHandoff],
    tenant_policy: KanbanTenantPolicy,
) -> Result<KanbanContextAssembly, KanbanError> {
    debug_assert!(!child.task_id().is_empty());
    debug_assert!(handoffs.len() <= MAX_KANBAN_LINKS);
    let mut admitted = Vec::with_capacity(handoffs.len());
    let mut quarantined = Vec::with_capacity(handoffs.len());
    for handoff in handoffs {
        let validated = validate_context_handoff(handoff)?;
        if tenant_mismatch(child.tenant_label(), validated.tenant_label.as_deref()) {
            match tenant_policy {
                KanbanTenantPolicy::RejectCrossTenant => {
                    return Err(KanbanError::new(
                        KanbanErrorKind::CrossTenantLeak,
                        TENANT_FIELD,
                        CROSS_TENANT_REASON,
                    ));
                }
                KanbanTenantPolicy::QuarantineCrossTenant => {
                    quarantined.push(KanbanQuarantinedHandoff {
                        handoff_id: validated.handoff_id,
                        tenant_label: validated.tenant_label,
                    });
                }
            }
        } else {
            admitted.push(KanbanContextItem {
                handoff_id: validated.handoff_id,
                tenant_label: validated.tenant_label,
                safe_summary: validated.body,
            });
        }
    }
    Ok(KanbanContextAssembly {
        child_task_id: String::from(child.task_id()),
        admitted,
        quarantined,
    })
}

/// Loads and validates a graph through an injected board service.
///
/// # Errors
///
/// Returns [`KanbanError`] when no service is supplied, the board label is
/// unsafe, the service fails, or the returned graph is invalid.
pub fn load_kanban_graph(
    service: Option<&mut dyn KanbanBoardService>,
    board_label: &str,
) -> Result<KanbanTaskGraph, KanbanError> {
    let label = label!("board_label", board_label)?;
    let draft = injected_board_service(service)?.load_task_graph(label.as_str())?;
    validate_kanban_graph(draft)
}

/// Appends a handoff through an injected board service.
///
/// # Errors
///
/// Returns [`KanbanError`] when no service is supplied, the handoff is unsafe, or
/// the service rejects the handoff.
pub fn append_kanban_handoff(
    service: Option<&mut dyn KanbanBoardService>,
    handoff: &KanbanContextHandoff,
) -> Result<(), KanbanError> {
    let validated = validate_context_handoff(handoff)?;
    injected_board_service(service)?.append_handoff(&validated)
}

fn injected_board_service(
    service: Option<&mut dyn KanbanBoardService>,
) -> Result<&mut dyn KanbanBoardService, KanbanError> {
    service.ok_or_else(|| {
        KanbanError::new(
            KanbanErrorKind::BoardServiceUnavailable,
            BOARD_SERVICE_FIELD,
            MISSING_BOARD_SERVICE_REASON,
        )
    })
}

fn adapter_operation_metadata()
-> Result<animus_adapters::orchestration::OrchestrationOperationMetadata, KanbanError> {
    animus_adapters::orchestration::OrchestrationOperationMetadata::new(
        ADAPTER_BRIDGE_ACTOR_LABEL,
        ADAPTER_BRIDGE_POLICY_LABEL,
        None,
    )
    .map_err(adapter_error_to_kanban)
}

fn adapter_graph_to_kanban(
    graph: &animus_adapters::orchestration::OrchestrationTaskGraphDraft,
) -> KanbanTaskGraphDraft {
    let mut tasks = Vec::with_capacity(graph.tasks().len());
    for task in graph.tasks() {
        tasks.push(adapter_task_to_kanban(task));
    }
    KanbanTaskGraphDraft { tasks }
}

fn adapter_task_to_kanban(
    task: &animus_adapters::orchestration::OrchestrationTaskDraft,
) -> KanbanTaskDraft {
    KanbanTaskDraft {
        task_id: String::from(task.task_id()),
        title: String::from(task.title()),
        body: String::from(task.body()),
        assignee_label: task.assignee_label().map(String::from),
        state: adapter_task_state_to_kanban(task.state()),
        parent_ids: task.parent_ids().to_vec(),
        child_ids: task.child_ids().to_vec(),
        priority: adapter_priority_to_kanban(task.priority()),
        workspace_scope: task
            .workspace_scope()
            .map(adapter_workspace_scope_to_kanban),
        tenant_label: task.tenant_label().map(String::from),
        comments: adapter_comments_to_kanban(task.comments()),
        heartbeats: adapter_heartbeats_to_kanban(task.heartbeats()),
        block_records: adapter_blocks_to_kanban(task.block_records()),
        completion: task.completion().map(adapter_completion_to_kanban),
        metadata: task.metadata().clone(),
        revision: task.revision(),
    }
}

fn adapter_workspace_scope_to_kanban(
    scope: &animus_adapters::orchestration::OrchestrationWorkspaceScope,
) -> KanbanWorkspaceScope {
    KanbanWorkspaceScope {
        kind: match scope.kind() {
            animus_adapters::orchestration::OrchestrationWorkspaceKind::Scratch => {
                KanbanWorkspaceKind::Scratch
            }
            animus_adapters::orchestration::OrchestrationWorkspaceKind::Directory => {
                KanbanWorkspaceKind::Directory
            }
            animus_adapters::orchestration::OrchestrationWorkspaceKind::IsolatedBranch => {
                KanbanWorkspaceKind::Worktree
            }
        },
        root_capability: String::from(scope.root_capability()),
        writable_targets: scope.writable_targets().to_vec(),
        approval_required_targets: scope.approval_required_targets().to_vec(),
    }
}

fn adapter_task_state_to_kanban(
    state: animus_adapters::orchestration::OrchestrationTaskState,
) -> KanbanTaskState {
    match state {
        animus_adapters::orchestration::OrchestrationTaskState::Draft => KanbanTaskState::Draft,
        animus_adapters::orchestration::OrchestrationTaskState::Ready => KanbanTaskState::Ready,
        animus_adapters::orchestration::OrchestrationTaskState::InProgress => {
            KanbanTaskState::InProgress
        }
        animus_adapters::orchestration::OrchestrationTaskState::Blocked => KanbanTaskState::Blocked,
        animus_adapters::orchestration::OrchestrationTaskState::Completed => {
            KanbanTaskState::Completed
        }
        animus_adapters::orchestration::OrchestrationTaskState::Cancelled => {
            KanbanTaskState::Cancelled
        }
    }
}

fn adapter_priority_to_kanban(
    priority: animus_adapters::orchestration::OrchestrationPriority,
) -> KanbanPriority {
    match priority {
        animus_adapters::orchestration::OrchestrationPriority::Low => KanbanPriority::Low,
        animus_adapters::orchestration::OrchestrationPriority::Normal => KanbanPriority::Normal,
        animus_adapters::orchestration::OrchestrationPriority::High => KanbanPriority::High,
        animus_adapters::orchestration::OrchestrationPriority::Urgent => KanbanPriority::Urgent,
    }
}

fn adapter_comments_to_kanban(
    comments: &[animus_adapters::orchestration::OrchestrationComment],
) -> Vec<KanbanComment> {
    comments
        .iter()
        .map(|comment| KanbanComment {
            actor_label: String::from(comment.actor_label()),
            sequence: comment.sequence(),
            body: String::from(comment.body()),
            tenant_label: comment.tenant_label().map(String::from),
            metadata: comment.metadata().clone(),
        })
        .collect()
}

fn adapter_heartbeats_to_kanban(
    heartbeats: &[animus_adapters::orchestration::OrchestrationHeartbeat],
) -> Vec<KanbanHeartbeat> {
    heartbeats
        .iter()
        .map(|heartbeat| KanbanHeartbeat {
            actor_label: String::from(heartbeat.actor_label()),
            sequence: heartbeat.sequence(),
            summary: String::from(heartbeat.summary()),
            tenant_label: heartbeat.tenant_label().map(String::from),
            metadata: heartbeat.metadata().clone(),
        })
        .collect()
}

fn adapter_blocks_to_kanban(
    blocks: &[animus_adapters::orchestration::OrchestrationBlockRecord],
) -> Vec<KanbanBlockRecord> {
    blocks
        .iter()
        .map(|block| KanbanBlockRecord {
            actor_label: String::from(block.actor_label()),
            sequence: block.sequence(),
            reason: String::from(block.reason()),
            context: block.context().map(String::from),
            next_action: block.next_action().map(String::from),
            tenant_label: block.tenant_label().map(String::from),
            metadata: block.metadata().clone(),
        })
        .collect()
}

fn adapter_completion_to_kanban(
    completion: &animus_adapters::orchestration::OrchestrationCompletion,
) -> KanbanCompletion {
    KanbanCompletion {
        actor_label: String::from(completion.actor_label()),
        sequence: completion.sequence(),
        summary: String::from(completion.summary()),
        task_revision: completion.task_revision(),
        tenant_label: completion.tenant_label().map(String::from),
        metadata: completion.metadata().clone(),
    }
}

fn kanban_handoff_to_adapter(
    handoff: &KanbanContextHandoff,
) -> Result<
    animus_adapters::orchestration::OrchestrationContextHandoff,
    animus_adapters::orchestration::OrchestrationError,
> {
    animus_adapters::orchestration::OrchestrationContextHandoff::new(
        &animus_adapters::orchestration::OrchestrationContextHandoffDraft {
            handoff_id: handoff.handoff_id.as_str(),
            task_id: handoff.task_id.as_str(),
            tenant_label: handoff.tenant_label.as_deref(),
            body: handoff.body.as_str(),
            sequence: handoff.sequence,
            metadata: &handoff.metadata,
        },
    )
}

fn adapter_error_to_kanban(
    error: animus_adapters::orchestration::OrchestrationError,
) -> KanbanError {
    let reason = error.to_string();
    assert!(!reason.is_empty());
    assert!(!BOARD_SERVICE_FIELD.is_empty());
    assert!(!HANDOFF_BODY_FIELD.is_empty());
    assert!(!TENANT_FIELD.is_empty());
    assert!(!MUTATION_TARGET_FIELD.is_empty());
    assert!(!SEQUENCE_FIELD.is_empty());
    match error {
        animus_adapters::orchestration::OrchestrationError::Contract(contract) => {
            adapter_contract_error_to_kanban(&contract, reason)
        }
        animus_adapters::orchestration::OrchestrationError::MissingService => KanbanError::new(
            KanbanErrorKind::BoardServiceUnavailable,
            BOARD_SERVICE_FIELD,
            MISSING_BOARD_SERVICE_REASON,
        ),
        animus_adapters::orchestration::OrchestrationError::BackendUnavailable
        | animus_adapters::orchestration::OrchestrationError::CapabilityDenied
        | animus_adapters::orchestration::OrchestrationError::InvalidGraph
        | animus_adapters::orchestration::OrchestrationError::UnsupportedOperation
        | animus_adapters::orchestration::OrchestrationError::SchemaMismatch => KanbanError::new(
            KanbanErrorKind::InjectedServiceFailure,
            BOARD_SERVICE_FIELD,
            reason,
        ),
        animus_adapters::orchestration::OrchestrationError::UnsafeHandoff
        | animus_adapters::orchestration::OrchestrationError::UnsafeEvidence { .. } => {
            KanbanError::new(KanbanErrorKind::UnsafeText, HANDOFF_BODY_FIELD, reason)
        }
        animus_adapters::orchestration::OrchestrationError::CrossTenantLeak => {
            KanbanError::new(KanbanErrorKind::CrossTenantLeak, TENANT_FIELD, reason)
        }
        animus_adapters::orchestration::OrchestrationError::OutOfScopeMutation => KanbanError::new(
            KanbanErrorKind::OutOfScopeMutation,
            MUTATION_TARGET_FIELD,
            reason,
        ),
        animus_adapters::orchestration::OrchestrationError::EmptyCollection { .. } => {
            KanbanError::new(KanbanErrorKind::EmptyField, BOARD_SERVICE_FIELD, reason)
        }
        animus_adapters::orchestration::OrchestrationError::TooManyItems { .. } => {
            KanbanError::new(KanbanErrorKind::TooManyItems, BOARD_SERVICE_FIELD, reason)
        }
        animus_adapters::orchestration::OrchestrationError::NonMonotonicSequence { .. } => {
            KanbanError::new(KanbanErrorKind::EmptyField, SEQUENCE_FIELD, reason)
        }
    }
}

fn adapter_contract_error_to_kanban(
    contract: &animus_adapters::AdapterContractError,
    reason: String,
) -> KanbanError {
    match contract {
        animus_adapters::AdapterContractError::EmptyField { field }
        | animus_adapters::AdapterContractError::EmptyCollection { field } => {
            KanbanError::new(KanbanErrorKind::EmptyField, field, reason)
        }
        animus_adapters::AdapterContractError::FieldTooLong { field, .. } => {
            KanbanError::new(KanbanErrorKind::FieldTooLong, field, reason)
        }
        animus_adapters::AdapterContractError::NulCharacter { field, .. }
        | animus_adapters::AdapterContractError::InvalidEvidenceHash { field, .. } => {
            KanbanError::new(KanbanErrorKind::UnsafeText, field, reason)
        }
        animus_adapters::AdapterContractError::UnsupportedStatusState { field, .. }
        | animus_adapters::AdapterContractError::UnsupportedStatusObservationKind {
            field, ..
        }
        | animus_adapters::AdapterContractError::UnsupportedSessionReferenceKind {
            field, ..
        }
        | animus_adapters::AdapterContractError::UnsupportedWorkspaceValue { field, .. }
        | animus_adapters::AdapterContractError::UnsupportedDataspaceValue { field, .. }
        | animus_adapters::AdapterContractError::UnsupportedRouterValue { field, .. } => {
            KanbanError::new(KanbanErrorKind::InjectedServiceFailure, field, reason)
        }
    }
}

fn validate_task_count(tasks: &[KanbanTaskDraft]) -> Result<(), KanbanError> {
    if tasks.is_empty() {
        return Err(KanbanError::new(
            KanbanErrorKind::EmptyField,
            "tasks",
            EMPTY_FIELD_REASON,
        ));
    }
    if tasks.len() > MAX_KANBAN_TASKS {
        return Err(KanbanError::new(
            KanbanErrorKind::TooManyItems,
            "tasks",
            TOO_MANY_ITEMS_REASON,
        ));
    }
    Ok(())
}

fn validate_unique_task_ids(tasks: &[KanbanTask]) -> Result<(), KanbanError> {
    for pair in tasks.windows(2) {
        if pair[0].task_id == pair[1].task_id {
            return Err(KanbanError::new(
                KanbanErrorKind::DuplicateTaskId,
                TASK_ID_FIELD,
                DUPLICATE_TASK_REASON,
            ));
        }
    }
    Ok(())
}

fn validate_task(draft: KanbanTaskDraft) -> Result<KanbanTask, KanbanError> {
    debug_assert!(draft.parent_ids.len() <= MAX_KANBAN_LINKS);
    debug_assert!(draft.child_ids.len() <= MAX_KANBAN_LINKS);
    let task_id = label!(TASK_ID_FIELD, draft.task_id.as_str())?;
    let title = label!(TITLE_FIELD, draft.title.as_str())?;
    let body = text!(BODY_FIELD, draft.body.as_str())?;
    let assignee_label = validate_optional_label(ASSIGNEE_FIELD, draft.assignee_label.as_deref())?;
    let parent_ids = validate_link_ids(PARENT_FIELD, draft.parent_ids.as_slice())?;
    let child_ids = validate_link_ids(CHILD_FIELD, draft.child_ids.as_slice())?;
    let workspace_scope = validate_workspace_scope(draft.workspace_scope)?;
    let tenant_label = validate_optional_label(TENANT_FIELD, draft.tenant_label.as_deref())?;
    let metadata = validate_metadata(METADATA_FIELD, draft.metadata)?;
    validate_revision(draft.revision)?;
    let comments = validate_comments(draft.comments.as_slice(), tenant_label.as_deref())?;
    let heartbeats = validate_heartbeats(draft.heartbeats.as_slice(), tenant_label.as_deref())?;
    let block_records =
        validate_block_records(draft.block_records.as_slice(), tenant_label.as_deref())?;
    let completion = validate_completion_record(
        draft.completion.as_ref(),
        draft.state,
        draft.revision,
        tenant_label.as_deref(),
    )?;
    Ok(KanbanTask {
        task_id,
        title,
        body,
        assignee_label,
        state: draft.state,
        parent_ids,
        child_ids,
        priority: draft.priority,
        workspace_scope,
        tenant_label,
        comments,
        heartbeats,
        block_records,
        completion,
        metadata,
        revision: draft.revision,
    })
}

fn validate_workspace_scope(
    scope: Option<KanbanWorkspaceScope>,
) -> Result<KanbanWorkspaceScope, KanbanError> {
    let scope = scope.ok_or_else(|| {
        KanbanError::new(
            KanbanErrorKind::MissingField,
            WORKSPACE_FIELD,
            MISSING_FIELD_REASON,
        )
    })?;
    let root_capability = label!(ROOT_CAPABILITY_FIELD, scope.root_capability.as_str())?;
    let writable_targets = validate_label_list(
        WORKSPACE_TARGET_FIELD,
        scope.writable_targets.as_slice(),
        MAX_KANBAN_LINKS,
    )?;
    let approval_required_targets = validate_label_list(
        WORKSPACE_TARGET_FIELD,
        scope.approval_required_targets.as_slice(),
        MAX_KANBAN_LINKS,
    )?;
    if writable_targets.is_empty() && approval_required_targets.is_empty() {
        return Err(KanbanError::new(
            KanbanErrorKind::EmptyField,
            WORKSPACE_TARGET_FIELD,
            EMPTY_FIELD_REASON,
        ));
    }
    assert!(
        !root_capability.is_empty(),
        "workspace root capability must be validated"
    );
    assert!(
        writable_targets.len() <= MAX_KANBAN_LINKS,
        "writable target bounds are enforced"
    );
    assert!(
        approval_required_targets.len() <= MAX_KANBAN_LINKS,
        "approval target bounds are enforced"
    );
    Ok(KanbanWorkspaceScope {
        kind: scope.kind,
        root_capability,
        writable_targets,
        approval_required_targets,
    })
}

fn validate_link_ids(field: &'static str, values: &[String]) -> Result<Vec<String>, KanbanError> {
    validate_label_list(field, values, MAX_KANBAN_LINKS)
}

fn validate_label_list(
    field: &'static str,
    values: &[String],
    limit: usize,
) -> Result<Vec<String>, KanbanError> {
    debug_assert!(!field.is_empty());
    debug_assert!(limit <= MAX_KANBAN_LINKS);
    if values.len() > limit {
        return Err(KanbanError::new(
            KanbanErrorKind::TooManyItems,
            field,
            TOO_MANY_ITEMS_REASON,
        ));
    }
    let mut result = Vec::with_capacity(values.len());
    for value in values {
        let label = label!(field, value.as_str())?;
        if result.iter().any(|existing| existing == &label) {
            return Err(KanbanError::new(
                KanbanErrorKind::InvalidLink,
                field,
                DUPLICATE_LINK_REASON,
            ));
        }
        result.push(label);
    }
    result.sort();
    Ok(result)
}

fn validate_comments(
    comments: &[KanbanComment],
    task_tenant: Option<&str>,
) -> Result<Vec<KanbanComment>, KanbanError> {
    if comments.len() > MAX_KANBAN_RECORDS {
        return Err(too_many_records(COMMENT_FIELD));
    }
    let mut validated = Vec::with_capacity(comments.len());
    for comment in comments {
        let tenant_label = validate_record_tenant(comment.tenant_label.as_deref(), task_tenant)?;
        validated.push(KanbanComment {
            actor_label: label!(ACTOR_FIELD, comment.actor_label.as_str())?,
            sequence: validate_sequence(comment.sequence)?,
            body: text!(BODY_FIELD, comment.body.as_str())?,
            tenant_label,
            metadata: validate_metadata(METADATA_FIELD, comment.metadata.clone())?,
        });
    }
    Ok(validated)
}

fn validate_heartbeats(
    heartbeats: &[KanbanHeartbeat],
    task_tenant: Option<&str>,
) -> Result<Vec<KanbanHeartbeat>, KanbanError> {
    if heartbeats.len() > MAX_KANBAN_RECORDS {
        return Err(too_many_records(HEARTBEAT_FIELD));
    }
    let mut validated = Vec::with_capacity(heartbeats.len());
    for heartbeat in heartbeats {
        let tenant_label = validate_record_tenant(heartbeat.tenant_label.as_deref(), task_tenant)?;
        validated.push(KanbanHeartbeat {
            actor_label: label!(ACTOR_FIELD, heartbeat.actor_label.as_str())?,
            sequence: validate_sequence(heartbeat.sequence)?,
            summary: text!(SUMMARY_FIELD, heartbeat.summary.as_str())?,
            tenant_label,
            metadata: validate_metadata(METADATA_FIELD, heartbeat.metadata.clone())?,
        });
    }
    Ok(validated)
}

fn validate_block_records(
    blocks: &[KanbanBlockRecord],
    task_tenant: Option<&str>,
) -> Result<Vec<KanbanBlockRecord>, KanbanError> {
    if blocks.len() > MAX_KANBAN_RECORDS {
        return Err(too_many_records(BLOCK_FIELD));
    }
    let mut validated = Vec::with_capacity(blocks.len());
    for block in blocks {
        let tenant_label = validate_record_tenant(block.tenant_label.as_deref(), task_tenant)?;
        validated.push(KanbanBlockRecord {
            actor_label: label!(ACTOR_FIELD, block.actor_label.as_str())?,
            sequence: validate_sequence(block.sequence)?,
            reason: label!(REASON_FIELD, block.reason.as_str())?,
            context: validate_optional_text(CONTEXT_FIELD, block.context.as_deref())?,
            next_action: validate_optional_text(NEXT_ACTION_FIELD, block.next_action.as_deref())?,
            tenant_label,
            metadata: validate_metadata(METADATA_FIELD, block.metadata.clone())?,
        });
    }
    Ok(validated)
}

fn validate_completion_record(
    completion: Option<&KanbanCompletion>,
    state: KanbanTaskState,
    task_revision: u64,
    task_tenant: Option<&str>,
) -> Result<Option<KanbanCompletion>, KanbanError> {
    match completion {
        Some(record) => {
            let validated = validate_completion(record, task_revision, task_tenant)?;
            Ok(Some(validated))
        }
        None if matches!(state, KanbanTaskState::Completed) => Err(KanbanError::new(
            KanbanErrorKind::StaleCompletion,
            COMPLETION_FIELD,
            STALE_COMPLETION_REASON,
        )),
        None => Ok(None),
    }
}

fn validate_completion(
    record: &KanbanCompletion,
    task_revision: u64,
    task_tenant: Option<&str>,
) -> Result<KanbanCompletion, KanbanError> {
    if record.task_revision != task_revision || record.task_revision < MIN_KANBAN_REVISION {
        return Err(KanbanError::new(
            KanbanErrorKind::StaleCompletion,
            COMPLETION_FIELD,
            STALE_COMPLETION_REASON,
        ));
    }
    let tenant_label = validate_record_tenant(record.tenant_label.as_deref(), task_tenant)?;
    Ok(KanbanCompletion {
        actor_label: label!(ACTOR_FIELD, record.actor_label.as_str())?,
        sequence: validate_sequence(record.sequence)?,
        summary: text!(SUMMARY_FIELD, record.summary.as_str())?,
        task_revision: record.task_revision,
        tenant_label,
        metadata: validate_metadata(METADATA_FIELD, record.metadata.clone())?,
    })
}

fn validate_context_handoff(
    handoff: &KanbanContextHandoff,
) -> Result<KanbanContextHandoff, KanbanError> {
    Ok(KanbanContextHandoff {
        handoff_id: label!(HANDOFF_ID_FIELD, handoff.handoff_id.as_str())?,
        task_id: label!(TASK_ID_FIELD, handoff.task_id.as_str())?,
        tenant_label: validate_optional_label(TENANT_FIELD, handoff.tenant_label.as_deref())?,
        body: text!(HANDOFF_BODY_FIELD, handoff.body.as_str())?,
        sequence: validate_sequence(handoff.sequence)?,
        metadata: validate_metadata(METADATA_FIELD, handoff.metadata.clone())?,
    })
}

fn validate_record_tenant(
    record_tenant: Option<&str>,
    task_tenant: Option<&str>,
) -> Result<Option<String>, KanbanError> {
    let tenant_label = validate_optional_label(TENANT_FIELD, record_tenant)?;
    if tenant_mismatch(task_tenant, tenant_label.as_deref()) {
        return Err(KanbanError::new(
            KanbanErrorKind::CrossTenantLeak,
            TENANT_FIELD,
            CROSS_TENANT_REASON,
        ));
    }
    Ok(tenant_label)
}

fn validate_revision(revision: u64) -> Result<(), KanbanError> {
    if revision < MIN_KANBAN_REVISION {
        return Err(KanbanError::new(
            KanbanErrorKind::EmptyField,
            REVISION_FIELD,
            EMPTY_FIELD_REASON,
        ));
    }
    Ok(())
}

fn validate_sequence(sequence: u64) -> Result<u64, KanbanError> {
    if sequence < MIN_KANBAN_SEQUENCE {
        return Err(KanbanError::new(
            KanbanErrorKind::EmptyField,
            SEQUENCE_FIELD,
            EMPTY_FIELD_REASON,
        ));
    }
    Ok(sequence)
}

fn validate_metadata(
    field: &'static str,
    metadata: std::collections::BTreeMap<String, String>,
) -> Result<std::collections::BTreeMap<String, String>, KanbanError> {
    if metadata.len() > MAX_KANBAN_METADATA_FIELDS {
        return Err(KanbanError::new(
            KanbanErrorKind::TooManyItems,
            field,
            TOO_MANY_ITEMS_REASON,
        ));
    }
    let mut entries = Vec::with_capacity(metadata.len());
    for (key, value) in metadata {
        let safe_key = label!(field, key.as_str())?;
        let safe_value = text!(field, value.as_str())?;
        entries.push((safe_key, safe_value));
    }
    Ok(entries.into_iter().collect())
}

fn validate_optional_label(
    field: &'static str,
    value: Option<&str>,
) -> Result<Option<String>, KanbanError> {
    value
        .map(|label| {
            validate_label(StringInput {
                field,
                value: label,
            })
        })
        .transpose()
}

fn validate_optional_text(
    field: &'static str,
    value: Option<&str>,
) -> Result<Option<String>, KanbanError> {
    value
        .map(|text| validate_text(StringInput { field, value: text }))
        .transpose()
}

fn validate_label(input: StringInput<'_>) -> Result<String, KanbanError> {
    validate_string(LimitedStringInput {
        field: input.field,
        value: input.value,
        limit: MAX_KANBAN_LABEL_CHARS,
    })
}

fn validate_text(input: StringInput<'_>) -> Result<String, KanbanError> {
    validate_string(LimitedStringInput {
        field: input.field,
        value: input.value,
        limit: MAX_KANBAN_TEXT_CHARS,
    })
}

fn validate_string(input: LimitedStringInput<'_>) -> Result<String, KanbanError> {
    debug_assert!(!input.field.is_empty());
    debug_assert!(input.limit > 0);
    let trimmed = input.value.trim();
    if trimmed.is_empty() {
        return Err(KanbanError::new(
            KanbanErrorKind::EmptyField,
            input.field,
            EMPTY_FIELD_REASON,
        ));
    }
    if trimmed.chars().count() > input.limit {
        return Err(KanbanError::new(
            KanbanErrorKind::FieldTooLong,
            input.field,
            TOO_LONG_REASON,
        ));
    }
    if contains_unsafe_marker(trimmed) {
        return Err(KanbanError::new(
            KanbanErrorKind::UnsafeText,
            input.field,
            UNSAFE_TEXT_REASON,
        ));
    }
    Ok(String::from(trimmed))
}

fn contains_unsafe_marker(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    UNSAFE_TEXT_MARKERS.iter().any(|marker| {
        contains_unsafe_marker_value(UnsafeMarkerCheck {
            normalized_value: lower.as_str(),
            marker,
        })
    })
}

#[derive(Clone, Copy)]
struct UnsafeMarkerCheck<'a> {
    normalized_value: &'a str,
    marker: &'a str,
}

fn contains_unsafe_marker_value(check: UnsafeMarkerCheck<'_>) -> bool {
    if check.marker == OPENAI_KEY_PREFIX_MARKER {
        return check.normalized_value.starts_with(check.marker)
            || check.normalized_value.contains(" sk-");
    }
    check
        .normalized_value
        .contains(&check.marker.to_ascii_lowercase())
}

fn validate_links(tasks: &[KanbanTask]) -> Result<(), KanbanError> {
    for task in tasks {
        validate_parent_links(task, tasks)?;
        validate_child_links(task, tasks)?;
    }
    Ok(())
}

fn validate_parent_links(task: &KanbanTask, tasks: &[KanbanTask]) -> Result<(), KanbanError> {
    debug_assert!(!task.task_id().is_empty());
    debug_assert!(tasks.len() <= MAX_KANBAN_TASKS);
    for parent_id in task.parent_ids() {
        let parent = task_by_id(TaskLookup {
            tasks,
            task_id: parent_id,
            field: PARENT_FIELD,
        })?;
        if !parent
            .child_ids()
            .iter()
            .any(|child_id| child_id == task.task_id())
        {
            return Err(KanbanError::new(
                KanbanErrorKind::InvalidLink,
                PARENT_FIELD,
                INVALID_LINK_REASON,
            ));
        }
    }
    Ok(())
}

fn validate_child_links(task: &KanbanTask, tasks: &[KanbanTask]) -> Result<(), KanbanError> {
    debug_assert!(!task.task_id().is_empty());
    debug_assert!(tasks.len() <= MAX_KANBAN_TASKS);
    for child_id in task.child_ids() {
        let child = task_by_id(TaskLookup {
            tasks,
            task_id: child_id,
            field: CHILD_FIELD,
        })?;
        if !child
            .parent_ids()
            .iter()
            .any(|parent_id| parent_id == task.task_id())
        {
            return Err(KanbanError::new(
                KanbanErrorKind::InvalidLink,
                CHILD_FIELD,
                INVALID_LINK_REASON,
            ));
        }
    }
    Ok(())
}

fn task_by_id(input: TaskLookup<'_>) -> Result<&KanbanTask, KanbanError> {
    input
        .tasks
        .iter()
        .find(|task| task.task_id() == input.task_id)
        .ok_or_else(|| {
            KanbanError::new(
                KanbanErrorKind::UnknownTaskId,
                input.field,
                INVALID_LINK_REASON,
            )
        })
}

fn validate_tenant_dependencies(tasks: &[KanbanTask]) -> Result<(), KanbanError> {
    for child in tasks {
        for parent_id in child.parent_ids() {
            let parent = task_by_id(TaskLookup {
                tasks,
                task_id: parent_id,
                field: PARENT_FIELD,
            })?;
            if tenant_mismatch(child.tenant_label(), parent.tenant_label()) {
                return Err(KanbanError::new(
                    KanbanErrorKind::CrossTenantLeak,
                    TENANT_FIELD,
                    CROSS_TENANT_REASON,
                ));
            }
        }
    }
    Ok(())
}

fn tenant_mismatch(left: Option<&str>, right: Option<&str>) -> bool {
    matches!((left, right), (Some(left_label), Some(right_label)) if left_label != right_label)
}

fn validate_acyclic(tasks: &[KanbanTask]) -> Result<(), KanbanError> {
    debug_assert!(tasks.len() <= MAX_KANBAN_TASKS);
    for task in tasks {
        validate_task_acyclic(tasks, task.task_id())?;
    }
    Ok(())
}

fn validate_task_acyclic(tasks: &[KanbanTask], start_task_id: &str) -> Result<(), KanbanError> {
    debug_assert!(!start_task_id.is_empty());
    debug_assert!(tasks.len() <= MAX_KANBAN_TASKS);
    let mut stack = Vec::with_capacity(tasks.len());
    stack.push(CycleFrame {
        task_id: start_task_id,
        next_parent_index: 0,
    });
    while !stack.is_empty() {
        let Some(top_index) = stack.len().checked_sub(1) else {
            break;
        };
        let task_id = stack[top_index].task_id;
        let task = task_by_id(TaskLookup {
            tasks,
            task_id,
            field: TASK_ID_FIELD,
        })?;
        let parent_index = stack[top_index].next_parent_index;
        if parent_index >= task.parent_ids().len() {
            stack.pop();
            continue;
        }
        let parent_id = task.parent_ids()[parent_index].as_str();
        let Some(next_parent_index) = parent_index.checked_add(1) else {
            return Err(KanbanError::new(
                KanbanErrorKind::InvalidLink,
                PARENT_FIELD,
                INVALID_LINK_REASON,
            ));
        };
        stack[top_index].next_parent_index = next_parent_index;
        if stack.iter().any(|frame| frame.task_id == parent_id) || stack.len() >= tasks.len() {
            return Err(KanbanError::new(
                KanbanErrorKind::Cycle,
                PARENT_FIELD,
                CYCLE_REASON,
            ));
        }
        stack.push(CycleFrame {
            task_id: parent_id,
            next_parent_index: 0,
        });
    }
    Ok(())
}

fn evaluate_parent_for_promotion(
    parent: &KanbanTask,
    child: &KanbanTask,
) -> Result<ParentPromotion, KanbanError> {
    debug_assert!(!parent.task_id().is_empty());
    debug_assert!(!child.task_id().is_empty());
    if tenant_mismatch(parent.tenant_label(), child.tenant_label()) {
        return Err(KanbanError::new(
            KanbanErrorKind::CrossTenantLeak,
            TENANT_FIELD,
            CROSS_TENANT_REASON,
        ));
    }
    match parent.state() {
        KanbanTaskState::Completed => {
            validate_parent_completion(parent, child)?;
            Ok(ParentPromotion::Complete)
        }
        KanbanTaskState::Draft | KanbanTaskState::Ready | KanbanTaskState::InProgress => {
            Ok(ParentPromotion::Waiting {
                parent_id: String::from(parent.task_id()),
            })
        }
        KanbanTaskState::Blocked | KanbanTaskState::Cancelled => Err(KanbanError::new(
            KanbanErrorKind::AmbiguousDependencyState,
            PARENT_FIELD,
            AMBIGUOUS_DEPENDENCY_REASON,
        )),
    }
}

fn validate_parent_completion(parent: &KanbanTask, child: &KanbanTask) -> Result<(), KanbanError> {
    debug_assert_eq!(parent.state(), KanbanTaskState::Completed);
    debug_assert!(!child.task_id().is_empty());
    let completion = parent.completion().ok_or_else(|| {
        KanbanError::new(
            KanbanErrorKind::StaleCompletion,
            COMPLETION_FIELD,
            STALE_COMPLETION_REASON,
        )
    })?;
    if completion.task_revision != parent.revision() {
        return Err(KanbanError::new(
            KanbanErrorKind::StaleCompletion,
            COMPLETION_FIELD,
            STALE_COMPLETION_REASON,
        ));
    }
    if tenant_mismatch(child.tenant_label(), completion.tenant_label.as_deref()) {
        return Err(KanbanError::new(
            KanbanErrorKind::CrossTenantLeak,
            TENANT_FIELD,
            CROSS_TENANT_REASON,
        ));
    }
    Ok(())
}

fn too_many_records(field: &'static str) -> KanbanError {
    KanbanError::new(KanbanErrorKind::TooManyItems, field, TOO_MANY_ITEMS_REASON)
}

#[cfg(test)]
mod tests {
    use super::*;

    const PARENT_A: &str = "parent-a";
    const PARENT_B: &str = "parent-b";
    const CHILD: &str = "child";
    const TITLE: &str = "Durable orchestration task";
    const BODY: &str = "Coordinate a bounded worker handoff";
    const ACTOR: &str = "worker-alpha";
    const TENANT_ALPHA: &str = "tenant-alpha";
    const TENANT_BETA: &str = "tenant-beta";
    const ROOT: &str = "workspace-root";
    const WRITABLE_DOCS: &str = "docs";
    const GUARDED_POLICY: &str = "policy";
    const APPROVAL: &str = "approval-record";
    const BOARD: &str = "board-alpha";
    const HANDOFF: &str = "handoff-parent";
    const SAFE_HANDOFF_BODY: &str = "parent completed bounded docs work";
    const EXPECTED_PARENT_COUNT: usize = 2;
    const EXPECTED_SINGLE_ITEM: usize = 1;
    const EXPECTED_GRAPH_TASKS: usize = 3;
    const NEXT_SEQUENCE: u64 = 2;

    #[test]
    fn valid_graph_promotes_child_after_all_parents_complete() {
        let graph = validate_kanban_graph(valid_graph_draft()).expect("valid graph");
        assert_eq!(graph.tasks().len(), EXPECTED_GRAPH_TASKS);
        let child = graph.task(CHILD).expect("child task");
        assert_eq!(child.parent_ids().len(), EXPECTED_PARENT_COUNT);
        let decision = evaluate_kanban_promotion(&graph, CHILD).expect("promotion decision");
        assert_eq!(decision.outcome, KanbanPromotionOutcome::Ready);
    }

    #[test]
    fn workspace_mutation_admission_is_explicit() {
        let graph = validate_kanban_graph(valid_graph_draft()).expect("valid graph");
        let child = graph.task(CHILD).expect("child task");
        let allowed = KanbanMutationIntent {
            target_label: String::from(WRITABLE_DOCS),
            approval_label: None,
        };
        let guarded = KanbanMutationIntent {
            target_label: String::from(GUARDED_POLICY),
            approval_label: None,
        };
        let approved = KanbanMutationIntent {
            target_label: String::from(GUARDED_POLICY),
            approval_label: Some(String::from(APPROVAL)),
        };
        assert_eq!(
            validate_kanban_workspace_mutation(child.workspace_scope(), &allowed),
            Ok(KanbanWorkspaceAdmission::Allowed)
        );
        assert_eq!(
            validate_kanban_workspace_mutation(child.workspace_scope(), &guarded),
            Ok(KanbanWorkspaceAdmission::RequiresHostApproval)
        );
        assert_eq!(
            validate_kanban_workspace_mutation(child.workspace_scope(), &approved),
            Ok(KanbanWorkspaceAdmission::Allowed)
        );
    }

    #[test]
    fn tenant_labels_are_preserved_in_context_assembly() {
        let graph = validate_kanban_graph(valid_graph_draft()).expect("valid graph");
        let child = graph.task(CHILD).expect("child task");
        let handoff = KanbanContextHandoff {
            handoff_id: String::from(HANDOFF),
            task_id: String::from(PARENT_A),
            tenant_label: Some(String::from(TENANT_ALPHA)),
            body: String::from(SAFE_HANDOFF_BODY),
            sequence: MIN_KANBAN_SEQUENCE,
            metadata: std::collections::BTreeMap::new(),
        };
        let assembly =
            assemble_kanban_context(child, &[handoff], KanbanTenantPolicy::RejectCrossTenant)
                .expect("tenant-safe assembly");
        assert_eq!(assembly.admitted.len(), EXPECTED_SINGLE_ITEM);
        assert_eq!(
            assembly.admitted[0].tenant_label.as_deref(),
            Some(TENANT_ALPHA)
        );
        assert_eq!(assembly.admitted[0].safe_summary, SAFE_HANDOFF_BODY);
    }

    #[test]
    fn board_service_is_optional_and_injected() {
        let unavailable = load_kanban_graph(None, BOARD).expect_err("missing service fails");
        assert_eq!(unavailable.kind(), KanbanErrorKind::BoardServiceUnavailable);
        let mut service = RecordingBoardService::default();
        let graph = load_kanban_graph(Some(&mut service), BOARD).expect("loaded graph");
        assert_eq!(graph.tasks().len(), EXPECTED_GRAPH_TASKS);
        assert_eq!(service.loads, EXPECTED_SINGLE_ITEM);
    }

    #[test]
    fn graph_validation_rejects_missing_duplicate_and_cycle_cases() {
        let missing_workspace = KanbanTaskDraft {
            workspace_scope: None,
            ..base_task(CHILD, KanbanTaskState::Draft)
        };
        let missing_error = validate_kanban_graph(KanbanTaskGraphDraft {
            tasks: vec![missing_workspace],
        })
        .expect_err("missing workspace rejected");
        assert_eq!(missing_error.kind(), KanbanErrorKind::MissingField);

        let duplicate_error = validate_kanban_graph(KanbanTaskGraphDraft {
            tasks: vec![
                base_task(CHILD, KanbanTaskState::Draft),
                base_task(CHILD, KanbanTaskState::Draft),
            ],
        })
        .expect_err("duplicate id rejected");
        assert_eq!(duplicate_error.kind(), KanbanErrorKind::DuplicateTaskId);

        let cycle_error = validate_kanban_graph(cyclic_graph_draft()).expect_err("cycle rejected");
        assert_eq!(cycle_error.kind(), KanbanErrorKind::Cycle);
    }

    #[test]
    fn dependency_promotion_rejects_ambiguous_and_stale_parents() {
        let blocked_parent = KanbanTaskDraft {
            state: KanbanTaskState::Blocked,
            child_ids: vec![String::from(CHILD)],
            ..base_task(PARENT_A, KanbanTaskState::Blocked)
        };
        let child = KanbanTaskDraft {
            parent_ids: vec![String::from(PARENT_A)],
            ..base_task(CHILD, KanbanTaskState::Draft)
        };
        let graph = validate_kanban_graph(KanbanTaskGraphDraft {
            tasks: vec![blocked_parent, child],
        })
        .expect("valid blocked graph");
        let ambiguous =
            evaluate_kanban_promotion(&graph, CHILD).expect_err("blocked parent ambiguous");
        assert_eq!(ambiguous.kind(), KanbanErrorKind::AmbiguousDependencyState);

        let stale = KanbanTaskDraft {
            state: KanbanTaskState::Completed,
            child_ids: vec![String::from(CHILD)],
            completion: Some(completion_with_revision(NEXT_SEQUENCE)),
            ..base_task(PARENT_A, KanbanTaskState::Completed)
        };
        let child = KanbanTaskDraft {
            parent_ids: vec![String::from(PARENT_A)],
            ..base_task(CHILD, KanbanTaskState::Draft)
        };
        let stale_error = validate_kanban_graph(KanbanTaskGraphDraft {
            tasks: vec![stale, child],
        })
        .expect_err("stale completion rejected");
        assert_eq!(stale_error.kind(), KanbanErrorKind::StaleCompletion);
    }

    #[test]
    fn cross_tenant_dependency_and_context_body_are_rejected_or_quarantined() {
        let parent = KanbanTaskDraft {
            child_ids: vec![String::from(CHILD)],
            tenant_label: Some(String::from(TENANT_BETA)),
            completion: Some(completion_with_tenant(TENANT_BETA)),
            ..base_task(PARENT_A, KanbanTaskState::Completed)
        };
        let child = KanbanTaskDraft {
            parent_ids: vec![String::from(PARENT_A)],
            tenant_label: Some(String::from(TENANT_ALPHA)),
            ..base_task(CHILD, KanbanTaskState::Draft)
        };
        let tenant_error = validate_kanban_graph(KanbanTaskGraphDraft {
            tasks: vec![parent, child],
        })
        .expect_err("cross-tenant dependency rejected");
        assert_eq!(tenant_error.kind(), KanbanErrorKind::CrossTenantLeak);

        let graph = validate_kanban_graph(valid_graph_draft()).expect("valid graph");
        let child = graph.task(CHILD).expect("child task");
        let beta_handoff = KanbanContextHandoff {
            handoff_id: String::from(HANDOFF),
            task_id: String::from(PARENT_A),
            tenant_label: Some(String::from(TENANT_BETA)),
            body: String::from(SAFE_HANDOFF_BODY),
            sequence: MIN_KANBAN_SEQUENCE,
            metadata: std::collections::BTreeMap::new(),
        };
        let reject = assemble_kanban_context(
            child,
            std::slice::from_ref(&beta_handoff),
            KanbanTenantPolicy::RejectCrossTenant,
        )
        .expect_err("cross-tenant context rejected");
        assert_eq!(reject.kind(), KanbanErrorKind::CrossTenantLeak);
        let quarantine = assemble_kanban_context(
            child,
            &[beta_handoff],
            KanbanTenantPolicy::QuarantineCrossTenant,
        )
        .expect("cross-tenant context quarantined");
        assert!(quarantine.admitted.is_empty());
        assert_eq!(quarantine.quarantined.len(), EXPECTED_SINGLE_ITEM);
    }

    #[test]
    fn unsafe_handoff_and_out_of_scope_mutation_fail_closed() {
        let graph = validate_kanban_graph(valid_graph_draft()).expect("valid graph");
        let child = graph.task(CHILD).expect("child task");
        let unsafe_handoff = KanbanContextHandoff {
            handoff_id: String::from(HANDOFF),
            task_id: String::from(PARENT_A),
            tenant_label: Some(String::from(TENANT_ALPHA)),
            body: String::from("RAW_PROMPT: hidden"),
            sequence: MIN_KANBAN_SEQUENCE,
            metadata: std::collections::BTreeMap::new(),
        };
        let unsafe_error = assemble_kanban_context(
            child,
            &[unsafe_handoff],
            KanbanTenantPolicy::RejectCrossTenant,
        )
        .expect_err("unsafe handoff rejected");
        assert_eq!(unsafe_error.kind(), KanbanErrorKind::UnsafeText);

        let out_of_scope = KanbanMutationIntent {
            target_label: String::from("private-root"),
            approval_label: Some(String::from(APPROVAL)),
        };
        let scope_error =
            validate_kanban_workspace_mutation(child.workspace_scope(), &out_of_scope)
                .expect_err("out-of-scope mutation rejected");
        assert_eq!(scope_error.kind(), KanbanErrorKind::OutOfScopeMutation);
    }

    fn valid_graph_draft() -> KanbanTaskGraphDraft {
        KanbanTaskGraphDraft {
            tasks: vec![
                completed_parent(PARENT_A),
                completed_parent(PARENT_B),
                ready_child(),
            ],
        }
    }

    fn completed_parent(task_id: &str) -> KanbanTaskDraft {
        KanbanTaskDraft {
            state: KanbanTaskState::Completed,
            child_ids: vec![String::from(CHILD)],
            completion: Some(completion_with_tenant(TENANT_ALPHA)),
            ..base_task(task_id, KanbanTaskState::Completed)
        }
    }

    fn ready_child() -> KanbanTaskDraft {
        KanbanTaskDraft {
            parent_ids: vec![String::from(PARENT_A), String::from(PARENT_B)],
            ..base_task(CHILD, KanbanTaskState::Draft)
        }
    }

    fn base_task(task_id: &str, state: KanbanTaskState) -> KanbanTaskDraft {
        KanbanTaskDraft {
            task_id: String::from(task_id),
            title: String::from(TITLE),
            body: String::from(BODY),
            assignee_label: Some(String::from(ACTOR)),
            state,
            parent_ids: Vec::new(),
            child_ids: Vec::new(),
            priority: KanbanPriority::Normal,
            workspace_scope: Some(workspace_scope()),
            tenant_label: Some(String::from(TENANT_ALPHA)),
            comments: vec![comment()],
            heartbeats: vec![heartbeat()],
            block_records: vec![block_record()],
            completion: None,
            metadata: std::collections::BTreeMap::from([(
                String::from("priority-source"),
                String::from("host"),
            )]),
            revision: MIN_KANBAN_REVISION,
        }
    }

    fn workspace_scope() -> KanbanWorkspaceScope {
        KanbanWorkspaceScope {
            kind: KanbanWorkspaceKind::Scratch,
            root_capability: String::from(ROOT),
            writable_targets: vec![String::from(WRITABLE_DOCS)],
            approval_required_targets: vec![String::from(GUARDED_POLICY)],
        }
    }

    fn comment() -> KanbanComment {
        KanbanComment {
            actor_label: String::from(ACTOR),
            sequence: MIN_KANBAN_SEQUENCE,
            body: String::from("bounded comment"),
            tenant_label: Some(String::from(TENANT_ALPHA)),
            metadata: std::collections::BTreeMap::new(),
        }
    }

    fn heartbeat() -> KanbanHeartbeat {
        KanbanHeartbeat {
            actor_label: String::from(ACTOR),
            sequence: MIN_KANBAN_SEQUENCE,
            summary: String::from("still working"),
            tenant_label: Some(String::from(TENANT_ALPHA)),
            metadata: std::collections::BTreeMap::new(),
        }
    }

    fn block_record() -> KanbanBlockRecord {
        KanbanBlockRecord {
            actor_label: String::from(ACTOR),
            sequence: MIN_KANBAN_SEQUENCE,
            reason: String::from("needs-review"),
            context: Some(String::from("bounded context")),
            next_action: Some(String::from("review")),
            tenant_label: Some(String::from(TENANT_ALPHA)),
            metadata: std::collections::BTreeMap::new(),
        }
    }

    fn completion_with_tenant(tenant: &str) -> KanbanCompletion {
        KanbanCompletion {
            actor_label: String::from(ACTOR),
            sequence: MIN_KANBAN_SEQUENCE,
            summary: String::from("completed with bounded metadata"),
            task_revision: MIN_KANBAN_REVISION,
            tenant_label: Some(String::from(tenant)),
            metadata: std::collections::BTreeMap::from([(
                String::from("result"),
                String::from("accepted"),
            )]),
        }
    }

    fn completion_with_revision(task_revision: u64) -> KanbanCompletion {
        KanbanCompletion {
            task_revision,
            ..completion_with_tenant(TENANT_ALPHA)
        }
    }

    fn cyclic_graph_draft() -> KanbanTaskGraphDraft {
        let first = KanbanTaskDraft {
            parent_ids: vec![String::from(PARENT_B)],
            child_ids: vec![String::from(PARENT_B)],
            ..base_task(PARENT_A, KanbanTaskState::Draft)
        };
        let second = KanbanTaskDraft {
            parent_ids: vec![String::from(PARENT_A)],
            child_ids: vec![String::from(PARENT_A)],
            ..base_task(PARENT_B, KanbanTaskState::Draft)
        };
        KanbanTaskGraphDraft {
            tasks: vec![first, second],
        }
    }

    #[test]
    fn adapter_family_bridge_loads_graph_and_appends_handoff() {
        // r[verify agent_harness_runtime.orchestration_adapters.runtime_bridge]
        // r[verify agent_harness_runtime.orchestration_adapters.verification]
        let mut adapter = RecordingOrchestrationService::default();
        {
            let mut bridge = OrchestrationAdapterBoardService::new(&mut adapter);
            let graph = load_kanban_graph(Some(&mut bridge), BOARD).expect("bridge graph load");
            assert_eq!(graph.tasks().len(), EXPECTED_PARENT_COUNT);

            let handoff = KanbanContextHandoff {
                handoff_id: String::from(HANDOFF),
                task_id: String::from(PARENT_A),
                tenant_label: Some(String::from(TENANT_ALPHA)),
                body: String::from(SAFE_HANDOFF_BODY),
                sequence: MIN_KANBAN_SEQUENCE,
                metadata: std::collections::BTreeMap::new(),
            };
            append_kanban_handoff(Some(&mut bridge), &handoff).expect("bridge handoff append");
        }
        assert_eq!(adapter.loads, EXPECTED_SINGLE_ITEM);
        assert_eq!(adapter.handoffs, EXPECTED_SINGLE_ITEM);
    }

    #[test]
    fn adapter_family_bridge_maps_fail_closed_errors() {
        // r[verify agent_harness_runtime.orchestration_adapters.runtime_bridge]
        // r[verify agent_harness_runtime.orchestration_adapters.fail_closed]
        let mut failing = FailingOrchestrationService {
            error: animus_adapters::orchestration::OrchestrationError::BackendUnavailable,
        };
        let mut bridge = OrchestrationAdapterBoardService::new(&mut failing);
        let unavailable = load_kanban_graph(Some(&mut bridge), BOARD)
            .expect_err("backend unavailable should fail closed");
        assert_eq!(unavailable.kind(), KanbanErrorKind::InjectedServiceFailure);

        let unsafe_handoff = KanbanContextHandoff {
            handoff_id: String::from(HANDOFF),
            task_id: String::from(PARENT_A),
            tenant_label: Some(String::from(TENANT_ALPHA)),
            body: String::from("RAW_PROMPT: hidden"),
            sequence: MIN_KANBAN_SEQUENCE,
            metadata: std::collections::BTreeMap::new(),
        };
        let unsafe_error = append_kanban_handoff(Some(&mut bridge), &unsafe_handoff)
            .expect_err("unsafe handoff should fail before host append");
        assert_eq!(unsafe_error.kind(), KanbanErrorKind::UnsafeText);
    }

    #[derive(Default)]
    #[cfg_attr(animus_octet, allow(tigerstyle::path_segment_repetition))]
    struct RecordingOrchestrationService {
        loads: usize,
        handoffs: usize,
    }

    impl animus_adapters::orchestration::OrchestrationService for RecordingOrchestrationService {
        fn load_task_graph(
            &mut self,
            request: &animus_adapters::orchestration::OrchestrationTaskGraphLoadRequest,
        ) -> Result<
            animus_adapters::orchestration::OrchestrationTaskGraphDraft,
            animus_adapters::orchestration::OrchestrationError,
        > {
            assert_eq!(request.board_label(), BOARD);
            self.loads = self
                .loads
                .checked_add(EXPECTED_SINGLE_ITEM)
                .expect("adapter load count stays within fixture bound");
            adapter_graph_draft()
        }

        fn append_handoff(
            &mut self,
            request: &animus_adapters::orchestration::OrchestrationHandoffAppendRequest,
        ) -> Result<
            animus_adapters::orchestration::OrchestrationReceipt,
            animus_adapters::orchestration::OrchestrationError,
        > {
            assert_eq!(request.handoff().handoff_id(), HANDOFF);
            self.handoffs = self
                .handoffs
                .checked_add(EXPECTED_SINGLE_ITEM)
                .expect("adapter handoff count stays within fixture bound");
            adapter_receipt(
                animus_adapters::orchestration::OrchestrationOperationKind::AppendHandoff,
            )
        }

        fn admit_scheduling(
            &mut self,
            _request: &animus_adapters::orchestration::OrchestrationSchedulingAdmissionRequest,
        ) -> Result<
            animus_adapters::orchestration::OrchestrationSchedulingAdmission,
            animus_adapters::orchestration::OrchestrationError,
        > {
            Err(animus_adapters::orchestration::OrchestrationError::UnsupportedOperation)
        }

        fn cancel_task(
            &mut self,
            _request: &animus_adapters::orchestration::OrchestrationCancellationRequest,
        ) -> Result<
            animus_adapters::orchestration::OrchestrationCancellationOutcome,
            animus_adapters::orchestration::OrchestrationError,
        > {
            Err(animus_adapters::orchestration::OrchestrationError::UnsupportedOperation)
        }

        fn record_lifecycle_evidence(
            &mut self,
            _request: &animus_adapters::orchestration::OrchestrationLifecycleEvidenceRequest,
        ) -> Result<
            animus_adapters::orchestration::OrchestrationReceipt,
            animus_adapters::orchestration::OrchestrationError,
        > {
            Err(animus_adapters::orchestration::OrchestrationError::UnsupportedOperation)
        }
    }

    #[cfg_attr(animus_octet, allow(tigerstyle::path_segment_repetition))]
    struct FailingOrchestrationService {
        error: animus_adapters::orchestration::OrchestrationError,
    }

    impl animus_adapters::orchestration::OrchestrationService for FailingOrchestrationService {
        fn load_task_graph(
            &mut self,
            _request: &animus_adapters::orchestration::OrchestrationTaskGraphLoadRequest,
        ) -> Result<
            animus_adapters::orchestration::OrchestrationTaskGraphDraft,
            animus_adapters::orchestration::OrchestrationError,
        > {
            Err(self.error.clone())
        }

        fn append_handoff(
            &mut self,
            _request: &animus_adapters::orchestration::OrchestrationHandoffAppendRequest,
        ) -> Result<
            animus_adapters::orchestration::OrchestrationReceipt,
            animus_adapters::orchestration::OrchestrationError,
        > {
            Err(self.error.clone())
        }

        fn admit_scheduling(
            &mut self,
            _request: &animus_adapters::orchestration::OrchestrationSchedulingAdmissionRequest,
        ) -> Result<
            animus_adapters::orchestration::OrchestrationSchedulingAdmission,
            animus_adapters::orchestration::OrchestrationError,
        > {
            Err(self.error.clone())
        }

        fn cancel_task(
            &mut self,
            _request: &animus_adapters::orchestration::OrchestrationCancellationRequest,
        ) -> Result<
            animus_adapters::orchestration::OrchestrationCancellationOutcome,
            animus_adapters::orchestration::OrchestrationError,
        > {
            Err(self.error.clone())
        }

        fn record_lifecycle_evidence(
            &mut self,
            _request: &animus_adapters::orchestration::OrchestrationLifecycleEvidenceRequest,
        ) -> Result<
            animus_adapters::orchestration::OrchestrationReceipt,
            animus_adapters::orchestration::OrchestrationError,
        > {
            Err(self.error.clone())
        }
    }

    fn adapter_graph_draft() -> Result<
        animus_adapters::orchestration::OrchestrationTaskGraphDraft,
        animus_adapters::orchestration::OrchestrationError,
    > {
        let parent = adapter_task(PARENT_A, &[], &[CHILD])?;
        let child = adapter_task(CHILD, &[PARENT_A], &[])?;
        animus_adapters::orchestration::OrchestrationTaskGraphDraft::new(vec![parent, child])
    }

    fn adapter_task(
        task_id: &str,
        parent_ids: &[&str],
        child_ids: &[&str],
    ) -> Result<
        animus_adapters::orchestration::OrchestrationTaskDraft,
        animus_adapters::orchestration::OrchestrationError,
    > {
        animus_adapters::orchestration::OrchestrationTaskDraft::new(
            animus_adapters::orchestration::OrchestrationTaskDraftInput {
                task_id,
                title: TITLE,
                body: BODY,
                assignee_label: Some(ACTOR),
                state: animus_adapters::orchestration::OrchestrationTaskState::Ready,
                parent_ids,
                child_ids,
                priority: animus_adapters::orchestration::OrchestrationPriority::Normal,
                workspace_scope: Some(adapter_workspace_scope()),
                tenant_label: Some(TENANT_ALPHA),
                comments: &[],
                heartbeats: &[],
                block_records: &[],
                completion: None,
                metadata: &std::collections::BTreeMap::new(),
                revision: MIN_KANBAN_REVISION,
            },
        )
    }

    fn adapter_workspace_scope() -> animus_adapters::orchestration::OrchestrationWorkspaceScope {
        animus_adapters::orchestration::OrchestrationWorkspaceScope::new(
            animus_adapters::orchestration::OrchestrationWorkspaceKind::Scratch,
            ROOT,
            &[WRITABLE_DOCS],
            &[GUARDED_POLICY],
        )
        .expect("adapter workspace scope should validate")
    }

    fn adapter_receipt(
        operation: animus_adapters::orchestration::OrchestrationOperationKind,
    ) -> Result<
        animus_adapters::orchestration::OrchestrationReceipt,
        animus_adapters::orchestration::OrchestrationError,
    > {
        assert!(!operation.as_str().is_empty());
        assert!(!BOARD.is_empty());
        assert!(!PARENT_A.is_empty());
        assert!(!ADAPTER_BRIDGE_POLICY_LABEL.is_empty());
        let hash = animus_adapters::orchestration::blake3_orchestration_hash(&[
            operation.as_str(),
            BOARD,
            PARENT_A,
        ]);
        assert!(!hash.as_str().is_empty());
        animus_adapters::orchestration::OrchestrationReceipt::new(
            &animus_adapters::orchestration::OrchestrationReceiptDraft {
                contract_identity:
                    animus_adapters::orchestration::ORCHESTRATION_RECEIPT_CONTRACT_IDENTITY,
                operation,
                status: animus_adapters::orchestration::OrchestrationStatusClass::Accepted,
                family_label: animus_adapters::orchestration::ORCHESTRATION_FAMILY_LABEL,
                board_label: BOARD,
                task_label: Some(PARENT_A),
                policy_label: ADAPTER_BRIDGE_POLICY_LABEL,
                redaction_markers: &["payload_not_captured"],
                item_count: EXPECTED_SINGLE_ITEM,
                dto_hash: hash.as_str(),
                summary: Some("bridge receipt"),
            },
        )
    }

    #[derive(Default)]
    struct RecordingBoardService {
        loads: usize,
        handoffs: usize,
    }

    impl KanbanBoardService for RecordingBoardService {
        fn load_task_graph(
            &mut self,
            board_label: &str,
        ) -> Result<KanbanTaskGraphDraft, KanbanError> {
            assert_eq!(board_label, BOARD);
            self.loads = self
                .loads
                .checked_add(EXPECTED_SINGLE_ITEM)
                .expect("load count stays within fixture bound");
            Ok(valid_graph_draft())
        }

        fn append_handoff(&mut self, _handoff: &KanbanContextHandoff) -> Result<(), KanbanError> {
            self.handoffs = self
                .handoffs
                .checked_add(EXPECTED_SINGLE_ITEM)
                .expect("handoff count stays within fixture bound");
            Ok(())
        }
    }
}

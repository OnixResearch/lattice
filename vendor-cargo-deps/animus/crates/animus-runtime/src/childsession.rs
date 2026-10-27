//! Pure concrete child-session admission, lifecycle, result, and persistence contracts.
//!
//! This module owns deterministic intersections and safe record shapes. It does
//! not create coding sessions, call providers or tools, read stores, observe
//! clocks, discover services, or retain private child transcripts.

#![cfg_attr(
    animus_octet,
    allow(
        tigerstyle::ambiguous_params,
        tigerstyle::too_many_parameters,
        reason = "published child-session compatibility contracts retain their typed stable signatures"
    )
)]
#![cfg_attr(
    animus_octet,
    allow(
        tigerstyle::assertion_density,
        tigerstyle::compound_condition,
        tigerstyle::function_length,
        reason = "child admission returns typed authority and bound errors instead of panicking assertions"
    )
)]
#![cfg_attr(
    animus_octet,
    allow(
        tigerstyle::non_trait_imports,
        reason = "the child-session core uses explicit subagent contract imports to keep ownership visible"
    )
)]
#![cfg_attr(
    animus_octet,
    allow(
        tigerstyle::bool_naming,
        tigerstyle::numeric_units,
        reason = "published child-session vocabulary uses field names fixed by its versioned contract"
    )
)]

use std::collections::BTreeSet;

use crate::subagents::{
    SubagentAdmission, SubagentCapabilityCeilingSource, SubagentCapabilityResolutionInput,
    SubagentContextPacket, SubagentHandoffMode, SubagentLineageRecord, TypedSubagentCapability,
    resolve_typed_subagent_capabilities, subagent_capability_set_identity,
};

const MAX_LABEL_CHARS: usize = 128;
const MAX_OBJECTIVE_CHARS: usize = 32_768;
const MAX_COLLECTION_ITEMS: usize = 128;
const BLAKE3_HEX_CHARS: usize = 64;
const HASH_DOMAIN_ADMISSION: &str = "animus.concrete-child-session.admission.v1";
const HASH_DOMAIN_RECORD: &str = "animus.concrete-child-session.record.v1";
const HASH_DOMAIN_RESULT: &str = "animus.concrete-child-session.result.v1";
const HASH_PART_LENGTH_BYTES: usize = core::mem::size_of::<u64>();
const REDACTED_PRIVATE: &str = "<private>";

/// Source that denied a child-session request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChildSessionAuthoritySource {
    /// Parent authority denied the request.
    Parent,
    /// Child profile denied the request.
    Profile,
    /// Semantic role denied the request.
    SemanticRole,
    /// Workspace capability set denied the request.
    Workspace,
    /// Host admission denied the request.
    Host,
    /// Context identity or budget denied the request.
    Context,
    /// Model grant denied the request.
    Model,
    /// Tool grant denied the request.
    Tool,
    /// A named resource limit denied the request.
    Limit,
    /// Required explicit services were absent.
    Service,
}

/// Pure child-session admission issue.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChildSessionAdmissionIssue {
    /// A required value was absent.
    MissingField,
    /// A label, hash, collection, or numeric value was malformed.
    Malformed,
    /// The request widened authority beyond a ceiling.
    AuthorityWidening,
    /// A request exceeded a named limit.
    BoundExceeded,
    /// Existing accepted identities did not match the supplied request.
    IdentityMismatch,
    /// A duplicate item made the request ambiguous.
    Duplicate,
}

/// Safe pure admission error without child content.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChildSessionAdmissionError {
    /// Stable issue class.
    pub issue: ChildSessionAdmissionIssue,
    /// Authority source that rejected the request.
    pub source: ChildSessionAuthoritySource,
    /// Safe field label.
    pub field: &'static str,
}

impl core::fmt::Display for ChildSessionAdmissionError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "child-session admission rejected {}", self.field)
    }
}

impl std::error::Error for ChildSessionAdmissionError {}

/// Named finite limits for one child coding session.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ChildSessionLimits {
    /// Maximum child recursion depth.
    pub depth: u32,
    /// Maximum child fan-out allocation.
    pub fan_out: u32,
    /// Maximum elapsed wall time in milliseconds.
    pub wall_time_ms: u64,
    /// Maximum model-token allocation.
    pub tokens: u64,
    /// Maximum model turns.
    pub turns: u32,
    /// Maximum tool calls.
    pub tool_calls: u32,
    /// Maximum parent-visible result characters.
    pub result_chars: u32,
}

impl ChildSessionLimits {
    fn values(self) -> [(&'static str, u64); 7] {
        [
            ("depth", u64::from(self.depth)),
            ("fan_out", u64::from(self.fan_out)),
            ("wall_time_ms", self.wall_time_ms),
            ("tokens", self.tokens),
            ("turns", u64::from(self.turns)),
            ("tool_calls", u64::from(self.tool_calls)),
            ("result_chars", u64::from(self.result_chars)),
        ]
    }
}

/// Parent or profile authority ceiling for one child request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChildSessionAuthorityCeiling {
    /// Typed capabilities admitted by this ceiling.
    pub capabilities: Vec<TypedSubagentCapability>,
    /// Exact workspace-scope labels admitted by this ceiling.
    pub workspace_scopes: Vec<String>,
    /// Exact model labels admitted by this ceiling.
    pub models: Vec<String>,
    /// Exact tool labels admitted by this ceiling.
    pub tools: Vec<String>,
    /// Finite resource ceilings.
    pub limits: ChildSessionLimits,
}

/// Explicit service kind required by a normal child coding session.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ChildSessionServiceKind {
    /// Provider dispatch service.
    Provider,
    /// Tool execution service.
    Tool,
    /// Durable private-session persistence service.
    Persistence,
    /// Context packet service.
    Context,
    /// Timer service.
    Timer,
    /// Cancellation service.
    Cancellation,
    /// Event sink service.
    Event,
    /// Coding-session policy service.
    Policy,
    /// Current authority service.
    Authority,
    /// Summarizer service.
    Summarizer,
    /// Active-turn recovery service.
    Recovery,
    /// Optional Steel kernel service.
    Kernel,
    /// Optional skill selection service.
    Skill,
}

/// Explicit factory service inventory.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChildSessionServiceSet {
    available: BTreeSet<ChildSessionServiceKind>,
}

impl ChildSessionServiceSet {
    /// Builds an explicit service set.
    #[must_use]
    pub fn new(available: impl IntoIterator<Item = ChildSessionServiceKind>) -> Self {
        Self {
            available: available.into_iter().collect(),
        }
    }

    /// Builds the complete service set used by positive host compositions.
    #[must_use]
    pub fn all_available() -> Self {
        Self::new([
            ChildSessionServiceKind::Provider,
            ChildSessionServiceKind::Tool,
            ChildSessionServiceKind::Persistence,
            ChildSessionServiceKind::Context,
            ChildSessionServiceKind::Timer,
            ChildSessionServiceKind::Cancellation,
            ChildSessionServiceKind::Event,
            ChildSessionServiceKind::Policy,
            ChildSessionServiceKind::Authority,
            ChildSessionServiceKind::Summarizer,
            ChildSessionServiceKind::Recovery,
            ChildSessionServiceKind::Kernel,
            ChildSessionServiceKind::Skill,
        ])
    }

    /// Returns whether the set contains one service kind.
    #[must_use]
    pub fn contains(&self, kind: ChildSessionServiceKind) -> bool {
        self.available.contains(&kind)
    }

    /// Returns the first missing service required by an admitted request.
    #[must_use]
    pub fn first_missing_for(
        &self,
        admission: &ConcreteChildSessionAdmission,
    ) -> Option<ChildSessionServiceKind> {
        let always_required = [
            ChildSessionServiceKind::Provider,
            ChildSessionServiceKind::Tool,
            ChildSessionServiceKind::Persistence,
            ChildSessionServiceKind::Context,
            ChildSessionServiceKind::Timer,
            ChildSessionServiceKind::Cancellation,
            ChildSessionServiceKind::Event,
            ChildSessionServiceKind::Policy,
            ChildSessionServiceKind::Authority,
            ChildSessionServiceKind::Summarizer,
            ChildSessionServiceKind::Recovery,
        ];
        always_required
            .into_iter()
            .find(|kind| !self.contains(*kind))
            .or_else(|| {
                admission
                    .kernel_profile()
                    .is_some_and(|_profile| !self.contains(ChildSessionServiceKind::Kernel))
                    .then_some(ChildSessionServiceKind::Kernel)
            })
            .or_else(|| {
                (!admission.skill_ids().is_empty()
                    && !self.contains(ChildSessionServiceKind::Skill))
                .then_some(ChildSessionServiceKind::Skill)
            })
    }
}

/// Full explicit input to pure child-session admission.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConcreteChildSessionAdmissionDraft {
    /// Existing accepted subagent admission.
    pub subagent: SubagentAdmission,
    /// Validated minimized context packet.
    pub context_packet: SubagentContextPacket,
    /// Bounded child objective used as the normal coding-session prompt.
    pub objective: String,
    /// Explicit child profile label.
    pub profile_id: String,
    /// Explicit parent authority identity.
    pub parent_authority_identity: String,
    /// Exact requested workspace scope.
    pub workspace_scope: String,
    /// Exact requested model label.
    pub model: String,
    /// Exact requested tool labels.
    pub tools: Vec<String>,
    /// Exact requested typed capabilities.
    pub requested_capabilities: Vec<TypedSubagentCapability>,
    /// Parent authority ceiling.
    pub parent: ChildSessionAuthorityCeiling,
    /// Child profile ceiling.
    pub profile: ChildSessionAuthorityCeiling,
    /// Independently admitted semantic-role capabilities.
    pub semantic_role_capabilities: Vec<TypedSubagentCapability>,
    /// Capabilities admitted for the requested workspace.
    pub workspace_capabilities: Vec<TypedSubagentCapability>,
    /// Capabilities admitted by current host policy.
    pub host_capabilities: Vec<TypedSubagentCapability>,
    /// Requested finite limits.
    pub requested_limits: ChildSessionLimits,
    /// Parent handoff mode.
    pub handoff_mode: SubagentHandoffMode,
    /// Optional explicit Steel kernel profile.
    pub kernel_profile: Option<String>,
    /// Explicit skill identities selected for the child.
    pub skill_ids: Vec<String>,
}

/// Admitted request passed to an explicit child-session factory.
#[derive(Clone, Eq, PartialEq)]
pub struct ConcreteChildSessionAdmission {
    subagent: SubagentAdmission,
    context_packet: SubagentContextPacket,
    objective: String,
    profile_id: String,
    parent_authority_identity: String,
    workspace_scope: String,
    model: String,
    tools: Vec<String>,
    effective_capabilities: Vec<TypedSubagentCapability>,
    limits: ChildSessionLimits,
    handoff_mode: SubagentHandoffMode,
    kernel_profile: Option<String>,
    skill_ids: Vec<String>,
    admission_identity: String,
}

impl ConcreteChildSessionAdmission {
    /// Returns the existing accepted subagent admission.
    #[must_use]
    pub const fn subagent(&self) -> &SubagentAdmission {
        &self.subagent
    }

    /// Returns the minimized context packet for the factory.
    #[must_use]
    pub const fn context_packet(&self) -> &SubagentContextPacket {
        &self.context_packet
    }

    /// Returns the admitted child objective.
    #[must_use]
    pub fn objective(&self) -> &str {
        self.objective.as_str()
    }

    /// Returns the admitted child profile label.
    #[must_use]
    pub fn profile_id(&self) -> &str {
        self.profile_id.as_str()
    }

    /// Returns the current parent authority identity bound at admission.
    #[must_use]
    pub fn parent_authority_identity(&self) -> &str {
        self.parent_authority_identity.as_str()
    }

    /// Returns the exact admitted workspace scope.
    #[must_use]
    pub fn workspace_scope(&self) -> &str {
        self.workspace_scope.as_str()
    }

    /// Returns the exact admitted model label.
    #[must_use]
    pub fn model(&self) -> &str {
        self.model.as_str()
    }

    /// Returns the exact admitted tool labels.
    #[must_use]
    pub fn tools(&self) -> &[String] {
        self.tools.as_slice()
    }

    /// Returns the effective typed capabilities.
    #[must_use]
    pub fn effective_capabilities(&self) -> &[TypedSubagentCapability] {
        self.effective_capabilities.as_slice()
    }

    /// Returns finite child limits.
    #[must_use]
    pub const fn limits(&self) -> ChildSessionLimits {
        self.limits
    }

    /// Returns the admitted parent handoff mode.
    #[must_use]
    pub const fn handoff_mode(&self) -> SubagentHandoffMode {
        self.handoff_mode
    }

    /// Returns the optional explicit Steel kernel profile.
    #[must_use]
    pub fn kernel_profile(&self) -> Option<&str> {
        self.kernel_profile.as_deref()
    }

    /// Returns explicit child skill identities.
    #[must_use]
    pub fn skill_ids(&self) -> &[String] {
        self.skill_ids.as_slice()
    }

    /// Returns the concrete admission identity.
    #[must_use]
    pub fn admission_identity(&self) -> &str {
        self.admission_identity.as_str()
    }
}

impl core::fmt::Debug for ConcreteChildSessionAdmission {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("ConcreteChildSessionAdmission")
            .field("subagent", &self.subagent)
            .field("context_packet_identity", &self.context_packet.packet_hash)
            .field("objective", &REDACTED_PRIVATE)
            .field("profile_id", &self.profile_id)
            .field("workspace_scope", &self.workspace_scope)
            .field("model", &self.model)
            .field("tools", &self.tools)
            .field("limits", &self.limits)
            .field("kernel_profile", &self.kernel_profile)
            .field("skill_ids", &self.skill_ids)
            .field("admission_identity", &self.admission_identity)
            .finish_non_exhaustive()
    }
}

/// Admits one concrete child-session request through monotonic intersections.
///
/// # Errors
///
/// Returns [`ChildSessionAdmissionError`] before any factory, store, provider,
/// tool, workspace, timer, or event effect can occur.
// r[impl agent_harness_runtime.child_session_service.authority]
#[allow(clippy::too_many_lines)]
pub fn admit_concrete_child_session(
    draft: &ConcreteChildSessionAdmissionDraft,
) -> Result<ConcreteChildSessionAdmission, ChildSessionAdmissionError> {
    validate_label("profile_id", draft.profile_id.as_str())?;
    validate_hash(
        "parent_authority_identity",
        draft.parent_authority_identity.as_str(),
    )?;
    validate_label("workspace_scope", draft.workspace_scope.as_str())?;
    validate_label("model", draft.model.as_str())?;
    validate_private_objective(draft.objective.as_str())?;
    validate_unique_labels("tools", draft.tools.as_slice())?;
    validate_unique_labels("skill_ids", draft.skill_ids.as_slice())?;
    if let Some(kernel_profile) = draft.kernel_profile.as_deref() {
        validate_label("kernel_profile", kernel_profile)?;
    }
    validate_ceiling(&draft.parent, ChildSessionAuthoritySource::Parent)?;
    validate_ceiling(&draft.profile, ChildSessionAuthoritySource::Profile)?;
    validate_limits(draft.requested_limits)?;
    validate_base_identities(draft)?;
    require_grant(
        "workspace_scope",
        draft.workspace_scope.as_str(),
        draft.parent.workspace_scopes.as_slice(),
        ChildSessionAuthoritySource::Parent,
    )?;
    require_grant(
        "workspace_scope",
        draft.workspace_scope.as_str(),
        draft.profile.workspace_scopes.as_slice(),
        ChildSessionAuthoritySource::Profile,
    )?;
    require_grant(
        "model",
        draft.model.as_str(),
        draft.parent.models.as_slice(),
        ChildSessionAuthoritySource::Model,
    )?;
    require_grant(
        "model",
        draft.model.as_str(),
        draft.profile.models.as_slice(),
        ChildSessionAuthoritySource::Profile,
    )?;
    require_subset(
        "tools",
        draft.tools.as_slice(),
        draft.parent.tools.as_slice(),
        ChildSessionAuthoritySource::Tool,
    )?;
    require_subset(
        "tools",
        draft.tools.as_slice(),
        draft.profile.tools.as_slice(),
        ChildSessionAuthoritySource::Profile,
    )?;
    require_limits_within(
        draft.requested_limits,
        draft.parent.limits,
        ChildSessionAuthoritySource::Parent,
    )?;
    require_limits_within(
        draft.requested_limits,
        draft.profile.limits,
        ChildSessionAuthoritySource::Profile,
    )?;
    if u64::from(draft.context_packet.budget.token_estimate) > draft.requested_limits.tokens {
        return Err(error(
            ChildSessionAdmissionIssue::BoundExceeded,
            ChildSessionAuthoritySource::Context,
            "context_tokens",
        ));
    }
    let resolution = resolve_typed_subagent_capabilities(&SubagentCapabilityResolutionInput {
        parent_policy: draft.parent.capabilities.clone(),
        recipe_or_profile: draft.profile.capabilities.clone(),
        request: draft.requested_capabilities.clone(),
        semantic_role: draft.semantic_role_capabilities.clone(),
        workspace_scope: draft.workspace_capabilities.clone(),
        host_admission: draft.host_capabilities.clone(),
    })
    .map_err(|_diagnostic| {
        error(
            ChildSessionAdmissionIssue::Malformed,
            ChildSessionAuthoritySource::Host,
            "capabilities",
        )
    })?;
    if let Some(denial) = resolution.denied.first() {
        return Err(error(
            ChildSessionAdmissionIssue::AuthorityWidening,
            capability_denial_source(denial.source),
            "capabilities",
        ));
    }
    let effective_identity = subagent_capability_set_identity(resolution.effective.as_slice());
    if effective_identity != draft.subagent.capability_identity {
        return Err(error(
            ChildSessionAdmissionIssue::IdentityMismatch,
            ChildSessionAuthoritySource::Host,
            "capability_identity",
        ));
    }
    let admission_identity = child_session_hash_parts(
        HASH_DOMAIN_ADMISSION,
        &[
            draft.subagent.admission_identity.as_str(),
            draft.parent_authority_identity.as_str(),
            draft.profile_id.as_str(),
            draft.workspace_scope.as_str(),
            draft.model.as_str(),
            effective_identity.as_str(),
            draft.context_packet.packet_hash.as_str(),
        ],
    );
    Ok(ConcreteChildSessionAdmission {
        subagent: draft.subagent.clone(),
        context_packet: draft.context_packet.clone(),
        objective: draft.objective.clone(),
        profile_id: draft.profile_id.clone(),
        parent_authority_identity: draft.parent_authority_identity.clone(),
        workspace_scope: draft.workspace_scope.clone(),
        model: draft.model.clone(),
        tools: draft.tools.clone(),
        effective_capabilities: resolution.effective,
        limits: draft.requested_limits,
        handoff_mode: draft.handoff_mode,
        kernel_profile: draft.kernel_profile.clone(),
        skill_ids: draft.skill_ids.clone(),
        admission_identity,
    })
}

/// Generation-fenced lifecycle state persisted without child bodies.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChildSessionLifecycleState {
    /// The factory created the child and enqueue has not started it.
    Queued,
    /// The child coding session has active work.
    Running,
    /// The child completed and awaits bounded collection.
    AwaitingParent,
    /// Cancellation is in progress.
    Cancelling,
    /// A completed bounded result was collected.
    Completed,
    /// A bounded partial result was collected after failure or cancellation.
    Partial,
    /// The child failed without an admitted parent result.
    Failed,
    /// The child exceeded its wall-time limit.
    TimedOut,
    /// Cancellation completed before a successful result.
    Cancelled,
    /// The child result exceeded its parent-visible limit.
    BoundExceeded,
    /// Current services or authority block recovery or progress.
    Blocked,
}

impl ChildSessionLifecycleState {
    /// Returns whether no further child execution is permitted.
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Completed
                | Self::Partial
                | Self::Failed
                | Self::TimedOut
                | Self::Cancelled
                | Self::BoundExceeded
        )
    }
}

/// Stable concrete child-session failure class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChildSessionFailureClass {
    /// A required injected service was absent.
    MissingService,
    /// Current policy or authority denied the operation.
    Denied,
    /// The explicit factory failed.
    Factory,
    /// The normal coding-session controller failed.
    Session,
    /// Durable lifecycle persistence failed.
    Persistence,
    /// Timer observation failed.
    Timer,
    /// Cancellation failed.
    Cancellation,
    /// A handle was stale, malformed, or unknown.
    StaleHandle,
    /// A handle named another parent or child.
    ForeignParent,
    /// Current parent authority differs from the admitted authority.
    AuthorityDrift,
    /// A duplicate child identity was requested.
    DuplicateChild,
    /// Restore facts differ from the recorded coding-session head.
    RestoreDrift,
    /// A child or result exceeded an admitted limit.
    BoundExceeded,
    /// A supplied record or controller result was malformed.
    Malformed,
    /// Elapsed wall time exceeded the admitted limit.
    TimedOut,
    /// Cancellation completed.
    Cancelled,
}

/// Safe coding-session head facts for lifecycle persistence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChildSessionHead {
    /// Durable coding-session head identity.
    pub committed_head_hash: String,
    /// Optional active leaf identity.
    pub active_leaf_id: Option<String>,
    /// Durable coding-session cursor.
    pub durable_cursor: u64,
}

/// Metadata-only durable lifecycle record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChildSessionLifecycleRecord {
    /// Parent identity.
    pub parent_id: String,
    /// Child identity.
    pub child_id: String,
    /// Registry generation.
    pub generation: u64,
    /// Concrete admission identity.
    pub admission_identity: String,
    /// Existing subagent admission identity.
    pub subagent_admission_identity: String,
    /// Effective capability identity.
    pub capability_identity: String,
    /// Context packet identity.
    pub context_packet_identity: String,
    /// Parent authority identity.
    pub parent_authority_identity: String,
    /// Explicit child profile label.
    pub profile_id: String,
    /// BLAKE3 identity of the explicit factory service composition.
    pub factory_service_identity: String,
    /// Host-observed start time in milliseconds.
    pub started_at_ms: u64,
    /// Current safe lifecycle state.
    pub state: ChildSessionLifecycleState,
    /// Coding-session durable head facts.
    pub session_head: ChildSessionHead,
    /// Optional bounded result identity.
    pub result_identity: Option<String>,
    /// Optional safe failure class.
    pub failure_class: Option<ChildSessionFailureClass>,
    /// Stable record identity.
    pub record_identity: String,
}

/// Validates and binds a safe lifecycle record identity.
///
/// # Errors
///
/// Returns [`ChildSessionAdmissionError`] for malformed labels, hashes, a zero
/// generation, or inconsistent terminal fields.
pub fn validate_child_session_lifecycle_record(
    mut record: ChildSessionLifecycleRecord,
) -> Result<ChildSessionLifecycleRecord, ChildSessionAdmissionError> {
    validate_label("parent_id", record.parent_id.as_str())?;
    validate_label("child_id", record.child_id.as_str())?;
    validate_label("profile_id", record.profile_id.as_str())?;
    if record.generation == 0 {
        return Err(error(
            ChildSessionAdmissionIssue::Malformed,
            ChildSessionAuthoritySource::Host,
            "generation",
        ));
    }
    for (field, value) in [
        ("admission_identity", record.admission_identity.as_str()),
        (
            "subagent_admission_identity",
            record.subagent_admission_identity.as_str(),
        ),
        ("capability_identity", record.capability_identity.as_str()),
        (
            "context_packet_identity",
            record.context_packet_identity.as_str(),
        ),
        (
            "parent_authority_identity",
            record.parent_authority_identity.as_str(),
        ),
        (
            "factory_service_identity",
            record.factory_service_identity.as_str(),
        ),
        (
            "committed_head_hash",
            record.session_head.committed_head_hash.as_str(),
        ),
    ] {
        validate_hash(field, value)?;
    }
    if let Some(result_identity) = record.result_identity.as_deref() {
        validate_hash("result_identity", result_identity)?;
    }
    if record.state.is_terminal() && record.result_identity.is_none() {
        record.result_identity = Some(child_session_hash_parts(
            HASH_DOMAIN_RESULT,
            &[
                record.child_id.as_str(),
                lifecycle_state_label(record.state),
            ],
        ));
    }
    if let Some(active_leaf_id) = record.session_head.active_leaf_id.as_deref() {
        validate_label("active_leaf_id", active_leaf_id)?;
    }
    let generation = record.generation.to_string();
    let started_at_ms = record.started_at_ms.to_string();
    let cursor = record.session_head.durable_cursor.to_string();
    let result_identity = record.result_identity.as_deref().unwrap_or("none");
    let failure = record
        .failure_class
        .map_or("none", child_session_failure_label);
    let expected = child_session_hash_parts(
        HASH_DOMAIN_RECORD,
        &[
            record.parent_id.as_str(),
            record.child_id.as_str(),
            generation.as_str(),
            record.admission_identity.as_str(),
            record.factory_service_identity.as_str(),
            started_at_ms.as_str(),
            lifecycle_state_label(record.state),
            record.session_head.committed_head_hash.as_str(),
            cursor.as_str(),
            result_identity,
            failure,
        ],
    );
    if !record.record_identity.is_empty() && record.record_identity != expected {
        return Err(error(
            ChildSessionAdmissionIssue::IdentityMismatch,
            ChildSessionAuthoritySource::Host,
            "record_identity",
        ));
    }
    record.record_identity = expected;
    Ok(record)
}

/// Parent-visible result class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChildSessionResultClass {
    /// Complete admitted result.
    Completed,
    /// Partial admitted result after a terminal non-success outcome.
    Partial,
    /// Failure without admitted result text.
    Failed,
    /// Timed out.
    TimedOut,
    /// Cancelled.
    Cancelled,
    /// Result text exceeded the admitted bound.
    BoundExceeded,
}

/// Draft for one bounded parent-visible child result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConcreteChildSessionResultDraft {
    /// Positive registry generation.
    pub generation: u64,
    /// Terminal result class.
    pub class: ChildSessionResultClass,
    /// Validated safe lineage.
    pub lineage: SubagentLineageRecord,
    /// Optional admitted model-visible body.
    pub body: Option<String>,
}

/// Bounded typed result returned to the parent.
#[derive(Clone, Eq, PartialEq)]
pub struct ConcreteChildSessionResult {
    /// Parent identity.
    pub parent_id: String,
    /// Child identity.
    pub child_id: String,
    /// Registry generation.
    pub generation: u64,
    /// Terminal class.
    pub class: ChildSessionResultClass,
    /// Safe validated lineage.
    pub lineage: SubagentLineageRecord,
    /// Domain-separated content identity.
    pub content_identity: String,
    /// Admitted model-visible body. No transcript is retained here.
    body: Option<String>,
}

impl ConcreteChildSessionResult {
    /// Builds a bounded parent result.
    ///
    /// # Errors
    ///
    /// Returns [`ChildSessionAdmissionError`] when a supplied body exceeds the
    /// admitted result limit or required identities are malformed.
    pub fn new(
        admission: &ConcreteChildSessionAdmission,
        draft: ConcreteChildSessionResultDraft,
    ) -> Result<Self, ChildSessionAdmissionError> {
        if draft.generation == 0 {
            return Err(error(
                ChildSessionAdmissionIssue::Malformed,
                ChildSessionAuthoritySource::Host,
                "generation",
            ));
        }
        if let Some(value) = draft.body.as_deref() {
            let count = u32::try_from(value.chars().count()).map_err(|_error| {
                error(
                    ChildSessionAdmissionIssue::BoundExceeded,
                    ChildSessionAuthoritySource::Limit,
                    "result_chars",
                )
            })?;
            if count > admission.limits.result_chars {
                return Err(error(
                    ChildSessionAdmissionIssue::BoundExceeded,
                    ChildSessionAuthoritySource::Limit,
                    "result_chars",
                ));
            }
            if value.contains('\0') {
                return Err(error(
                    ChildSessionAdmissionIssue::Malformed,
                    ChildSessionAuthoritySource::Host,
                    "result_body",
                ));
            }
        }
        let content_identity = child_session_hash_parts(
            HASH_DOMAIN_RESULT,
            &[
                admission.subagent.parent_id.as_str(),
                admission.subagent.child_id.as_str(),
                result_class_label(draft.class),
                draft.body.as_deref().unwrap_or(""),
            ],
        );
        Ok(Self {
            parent_id: admission.subagent.parent_id.clone(),
            child_id: admission.subagent.child_id.clone(),
            generation: draft.generation,
            class: draft.class,
            lineage: draft.lineage,
            content_identity,
            body: draft.body,
        })
    }

    /// Returns admitted model-visible result text.
    #[must_use]
    pub fn body(&self) -> Option<&str> {
        self.body.as_deref()
    }
}

impl core::fmt::Debug for ConcreteChildSessionResult {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("ConcreteChildSessionResult")
            .field("parent_id", &self.parent_id)
            .field("child_id", &self.child_id)
            .field("generation", &self.generation)
            .field("class", &self.class)
            .field("lineage", &self.lineage)
            .field("content_identity", &self.content_identity)
            .field("body", &self.body.as_ref().map(|_body| REDACTED_PRIVATE))
            .finish()
    }
}

/// Builds a framed domain-separated BLAKE3 identity.
#[must_use]
pub fn child_session_hash_parts(domain: &str, parts: &[&str]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain.as_bytes());
    for part in parts {
        let length = (part.len() as u64).to_le_bytes();
        debug_assert_eq!(length.len(), HASH_PART_LENGTH_BYTES);
        hasher.update(&length);
        hasher.update(part.as_bytes());
    }
    hasher.finalize().to_hex().to_string()
}

fn validate_base_identities(
    draft: &ConcreteChildSessionAdmissionDraft,
) -> Result<(), ChildSessionAdmissionError> {
    validate_hash(
        "subagent_admission_identity",
        draft.subagent.admission_identity.as_str(),
    )?;
    validate_hash(
        "capability_identity",
        draft.subagent.capability_identity.as_str(),
    )?;
    validate_hash(
        "context_packet_identity",
        draft.subagent.context_packet_identity.as_str(),
    )?;
    if draft.context_packet.packet_hash != draft.subagent.context_packet_identity
        || draft.context_packet.child_kind != draft.subagent.kind
    {
        return Err(error(
            ChildSessionAdmissionIssue::IdentityMismatch,
            ChildSessionAuthoritySource::Context,
            "context_packet_identity",
        ));
    }
    let request_identity =
        subagent_capability_set_identity(draft.requested_capabilities.as_slice());
    if request_identity != draft.subagent.capability_identity {
        return Err(error(
            ChildSessionAdmissionIssue::IdentityMismatch,
            ChildSessionAuthoritySource::Host,
            "capability_identity",
        ));
    }
    Ok(())
}

fn validate_ceiling(
    ceiling: &ChildSessionAuthorityCeiling,
    source: ChildSessionAuthoritySource,
) -> Result<(), ChildSessionAdmissionError> {
    if ceiling.capabilities.is_empty() {
        return Err(error(
            ChildSessionAdmissionIssue::MissingField,
            source,
            "capabilities",
        ));
    }
    validate_unique_labels("workspace_scopes", ceiling.workspace_scopes.as_slice())?;
    validate_unique_labels("models", ceiling.models.as_slice())?;
    validate_unique_labels("tools", ceiling.tools.as_slice())?;
    validate_limits(ceiling.limits)
}

fn validate_limits(limits: ChildSessionLimits) -> Result<(), ChildSessionAdmissionError> {
    for (field, value) in limits.values() {
        if value == 0 {
            return Err(error(
                ChildSessionAdmissionIssue::Malformed,
                ChildSessionAuthoritySource::Limit,
                field,
            ));
        }
    }
    Ok(())
}

fn require_limits_within(
    requested: ChildSessionLimits,
    ceiling: ChildSessionLimits,
    source: ChildSessionAuthoritySource,
) -> Result<(), ChildSessionAdmissionError> {
    for ((field, requested_value), (_ceiling_field, ceiling_value)) in
        requested.values().into_iter().zip(ceiling.values())
    {
        if requested_value > ceiling_value {
            return Err(error(
                ChildSessionAdmissionIssue::AuthorityWidening,
                source,
                field,
            ));
        }
    }
    Ok(())
}

fn require_grant(
    field: &'static str,
    value: &str,
    grants: &[String],
    source: ChildSessionAuthoritySource,
) -> Result<(), ChildSessionAdmissionError> {
    if grants.iter().any(|grant| grant == value) {
        Ok(())
    } else {
        Err(error(
            ChildSessionAdmissionIssue::AuthorityWidening,
            source,
            field,
        ))
    }
}

fn require_subset(
    field: &'static str,
    requested: &[String],
    grants: &[String],
    source: ChildSessionAuthoritySource,
) -> Result<(), ChildSessionAdmissionError> {
    if requested
        .iter()
        .all(|item| grants.iter().any(|grant| grant == item))
    {
        Ok(())
    } else {
        Err(error(
            ChildSessionAdmissionIssue::AuthorityWidening,
            source,
            field,
        ))
    }
}

fn validate_unique_labels(
    field: &'static str,
    values: &[String],
) -> Result<(), ChildSessionAdmissionError> {
    if values.len() > MAX_COLLECTION_ITEMS {
        return Err(error(
            ChildSessionAdmissionIssue::BoundExceeded,
            ChildSessionAuthoritySource::Limit,
            field,
        ));
    }
    let mut unique = BTreeSet::new();
    for value in values {
        validate_label(field, value.as_str())?;
        if !unique.insert(value.as_str()) {
            return Err(error(
                ChildSessionAdmissionIssue::Duplicate,
                ChildSessionAuthoritySource::Host,
                field,
            ));
        }
    }
    Ok(())
}

fn validate_private_objective(value: &str) -> Result<(), ChildSessionAdmissionError> {
    let normalized = value.trim();
    if normalized.is_empty() {
        return Err(error(
            ChildSessionAdmissionIssue::MissingField,
            ChildSessionAuthoritySource::Context,
            "objective",
        ));
    }
    if normalized.contains('\0') || normalized.chars().count() > MAX_OBJECTIVE_CHARS {
        return Err(error(
            ChildSessionAdmissionIssue::Malformed,
            ChildSessionAuthoritySource::Context,
            "objective",
        ));
    }
    Ok(())
}

fn validate_label(field: &'static str, value: &str) -> Result<(), ChildSessionAdmissionError> {
    let normalized = value.trim();
    if normalized.is_empty() {
        return Err(error(
            ChildSessionAdmissionIssue::MissingField,
            ChildSessionAuthoritySource::Host,
            field,
        ));
    }
    if normalized.chars().count() > MAX_LABEL_CHARS
        || normalized.contains('\0')
        || normalized.contains('/')
        || normalized.contains('\\')
        || normalized.contains("..")
    {
        return Err(error(
            ChildSessionAdmissionIssue::Malformed,
            ChildSessionAuthoritySource::Host,
            field,
        ));
    }
    Ok(())
}

fn validate_hash(field: &'static str, value: &str) -> Result<(), ChildSessionAdmissionError> {
    if value.len() == BLAKE3_HEX_CHARS
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        Ok(())
    } else {
        Err(error(
            ChildSessionAdmissionIssue::Malformed,
            ChildSessionAuthoritySource::Host,
            field,
        ))
    }
}

const fn capability_denial_source(
    source: SubagentCapabilityCeilingSource,
) -> ChildSessionAuthoritySource {
    match source {
        SubagentCapabilityCeilingSource::ParentPolicy => ChildSessionAuthoritySource::Parent,
        SubagentCapabilityCeilingSource::RecipeOrProfile => ChildSessionAuthoritySource::Profile,
        SubagentCapabilityCeilingSource::SemanticRole => ChildSessionAuthoritySource::SemanticRole,
        SubagentCapabilityCeilingSource::WorkspaceScope => ChildSessionAuthoritySource::Workspace,
        SubagentCapabilityCeilingSource::HostAdmission => ChildSessionAuthoritySource::Host,
    }
}

const fn lifecycle_state_label(state: ChildSessionLifecycleState) -> &'static str {
    match state {
        ChildSessionLifecycleState::Queued => "queued",
        ChildSessionLifecycleState::Running => "running",
        ChildSessionLifecycleState::AwaitingParent => "awaiting_parent",
        ChildSessionLifecycleState::Cancelling => "cancelling",
        ChildSessionLifecycleState::Completed => "completed",
        ChildSessionLifecycleState::Partial => "partial",
        ChildSessionLifecycleState::Failed => "failed",
        ChildSessionLifecycleState::TimedOut => "timed_out",
        ChildSessionLifecycleState::Cancelled => "cancelled",
        ChildSessionLifecycleState::BoundExceeded => "bound_exceeded",
        ChildSessionLifecycleState::Blocked => "blocked",
    }
}

const fn child_session_failure_label(failure: ChildSessionFailureClass) -> &'static str {
    match failure {
        ChildSessionFailureClass::MissingService => "missing_service",
        ChildSessionFailureClass::Denied => "denied",
        ChildSessionFailureClass::Factory => "factory",
        ChildSessionFailureClass::Session => "session",
        ChildSessionFailureClass::Persistence => "persistence",
        ChildSessionFailureClass::Timer => "timer",
        ChildSessionFailureClass::Cancellation => "cancellation",
        ChildSessionFailureClass::StaleHandle => "stale_handle",
        ChildSessionFailureClass::ForeignParent => "foreign_parent",
        ChildSessionFailureClass::AuthorityDrift => "authority_drift",
        ChildSessionFailureClass::DuplicateChild => "duplicate_child",
        ChildSessionFailureClass::RestoreDrift => "restore_drift",
        ChildSessionFailureClass::BoundExceeded => "bound_exceeded",
        ChildSessionFailureClass::Malformed => "malformed",
        ChildSessionFailureClass::TimedOut => "timed_out",
        ChildSessionFailureClass::Cancelled => "cancelled",
    }
}

const fn result_class_label(class: ChildSessionResultClass) -> &'static str {
    match class {
        ChildSessionResultClass::Completed => "completed",
        ChildSessionResultClass::Partial => "partial",
        ChildSessionResultClass::Failed => "failed",
        ChildSessionResultClass::TimedOut => "timed_out",
        ChildSessionResultClass::Cancelled => "cancelled",
        ChildSessionResultClass::BoundExceeded => "bound_exceeded",
    }
}

const fn error(
    issue: ChildSessionAdmissionIssue,
    source: ChildSessionAuthoritySource,
    field: &'static str,
) -> ChildSessionAdmissionError {
    ChildSessionAdmissionError {
        issue,
        source,
        field,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::subagents::{
        FreshContextSubagentDraft, SubagentChildRunDraft, SubagentChildRunKind,
        SubagentContextAdmissionPolicy, SubagentContextItemDraft, SubagentContextItemKind,
        SubagentContextPacketDraft, SubagentContextRedaction, SubagentWorkspaceAccessMode,
        admit_subagent_child_run, validate_subagent_child_run,
    };

    const HASH_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const CHILD_ID: &str = "child.focused";
    const PARENT_ID: &str = "parent.root";
    const PROFILE: &str = "focused-child";
    const WORKSPACE: &str = "workspace.readonly";
    const MODEL: &str = "model.focused";
    const TOOL: &str = "search";
    const OBJECTIVE: &str = "Review the admitted module and return a bounded summary.";
    const CONTEXT_BODY: &str = "Only the selected module summary is admitted.";
    const TOKEN_LIMIT: u64 = 8_000;
    const WALL_LIMIT_MS: u64 = 60_000;
    const RESULT_LIMIT: u32 = 4_096;
    const TURN_LIMIT: u32 = 8;
    const TOOL_CALL_LIMIT: u32 = 16;
    const DEPTH_LIMIT: u32 = 2;
    const FAN_OUT_LIMIT: u32 = 4;

    #[test]
    fn admission_intersects_every_named_authority_ceiling() {
        // r[verify agent_harness_runtime.child_session_service.authority]
        let draft = draft();
        let admitted = admit_concrete_child_session(&draft).expect("request must admit");
        assert_eq!(admitted.profile_id(), PROFILE);
        assert_eq!(admitted.workspace_scope(), WORKSPACE);
        assert_eq!(admitted.model(), MODEL);
        assert_eq!(admitted.tools(), [TOOL]);
        assert_eq!(
            admitted.context_packet().packet_hash,
            draft.context_packet.packet_hash
        );
        assert_eq!(admitted.parent_authority_identity(), HASH_A);
        assert!(!format!("{admitted:?}").contains(OBJECTIVE));
        assert!(!format!("{admitted:?}").contains(CONTEXT_BODY));
    }

    #[test]
    fn admission_rejects_workspace_model_tool_budget_context_and_capability_widening() {
        // r[verify agent_harness_runtime.child_session_service.authority.workspace]
        // r[verify agent_harness_runtime.child_session_service.verification]
        let mut cases = Vec::new();
        let mut workspace = draft();
        workspace.workspace_scope = String::from("workspace.write");
        cases.push(workspace);
        let mut model = draft();
        model.model = String::from("model.ungranted");
        cases.push(model);
        let mut tool = draft();
        tool.tools.push(String::from("shell"));
        cases.push(tool);
        let mut budget = draft();
        budget.requested_limits.tokens = TOKEN_LIMIT.saturating_add(1);
        cases.push(budget);
        let mut context = draft();
        context.context_packet.packet_hash = String::from(HASH_A);
        cases.push(context);
        let mut capability = draft();
        capability.host_capabilities.clear();
        cases.push(capability);
        for candidate in cases {
            assert!(admit_concrete_child_session(&candidate).is_err());
        }
    }

    #[test]
    fn service_requirements_and_records_fail_closed_without_private_content() {
        // r[verify agent_harness_runtime.child_session_service.factory.missing]
        // r[verify agent_harness_runtime.child_session_service.persistence]
        let mut draft = draft();
        draft.kernel_profile = Some(String::from("kernel.focused"));
        draft.skill_ids = vec![String::from("skill.review")];
        let admitted = admit_concrete_child_session(&draft).expect("request must admit");
        let missing_provider = ChildSessionServiceSet::new([
            ChildSessionServiceKind::Tool,
            ChildSessionServiceKind::Persistence,
        ]);
        assert_eq!(
            missing_provider.first_missing_for(&admitted),
            Some(ChildSessionServiceKind::Provider)
        );
        let complete = ChildSessionServiceSet::all_available();
        assert_eq!(complete.first_missing_for(&admitted), None);
        let record = validate_child_session_lifecycle_record(ChildSessionLifecycleRecord {
            parent_id: String::from(PARENT_ID),
            child_id: String::from(CHILD_ID),
            generation: 1,
            admission_identity: admitted.admission_identity().to_owned(),
            subagent_admission_identity: admitted.subagent().admission_identity.clone(),
            capability_identity: admitted.subagent().capability_identity.clone(),
            context_packet_identity: admitted.subagent().context_packet_identity.clone(),
            parent_authority_identity: String::from(HASH_A),
            profile_id: String::from(PROFILE),
            factory_service_identity: String::from(HASH_A),
            started_at_ms: 0,
            state: ChildSessionLifecycleState::Running,
            session_head: ChildSessionHead {
                committed_head_hash: String::from(HASH_A),
                active_leaf_id: None,
                durable_cursor: 0,
            },
            result_identity: None,
            failure_class: None,
            record_identity: String::new(),
        })
        .expect("safe record must validate");
        assert_eq!(record.record_identity.len(), BLAKE3_HEX_CHARS);
        assert!(!format!("{record:?}").contains(OBJECTIVE));
        assert!(!format!("{record:?}").contains(CONTEXT_BODY));
    }

    fn draft() -> ConcreteChildSessionAdmissionDraft {
        let capability = TypedSubagentCapability::WorkspaceAccess {
            stable_label: String::from("workspace-read"),
            scope_label: String::from(WORKSPACE),
            mode: SubagentWorkspaceAccessMode::Read,
        };
        let packet_draft = SubagentContextPacketDraft {
            packet_id: String::from("packet.focused"),
            child_kind: SubagentChildRunKind::FreshContextInvestigation,
            admission: SubagentContextAdmissionPolicy::default(),
            items: vec![SubagentContextItemDraft {
                kind: SubagentContextItemKind::Summary,
                label: String::from("module-summary"),
                safe_body: String::from(CONTEXT_BODY),
                token_estimate: 32,
                redaction: SubagentContextRedaction::SafeSummary,
            }],
        };
        let child = validate_subagent_child_run(&SubagentChildRunDraft {
            child_id: String::from(CHILD_ID),
            parent_id: String::from(PARENT_ID),
            kind: SubagentChildRunKind::FreshContextInvestigation,
            context_packet: Some(packet_draft),
            capabilities: vec![capability.clone()],
            recipe: None,
            fresh_context: Some(FreshContextSubagentDraft {
                objective: String::from(OBJECTIVE),
                scheduler_label: String::from("scheduler.explicit"),
                read_only: true,
            }),
            worker_team: None,
            execution_requested: false,
        })
        .expect("child must validate");
        let subagent = admit_subagent_child_run(&child);
        let limits = ChildSessionLimits {
            depth: DEPTH_LIMIT,
            fan_out: FAN_OUT_LIMIT,
            wall_time_ms: WALL_LIMIT_MS,
            tokens: TOKEN_LIMIT,
            turns: TURN_LIMIT,
            tool_calls: TOOL_CALL_LIMIT,
            result_chars: RESULT_LIMIT,
        };
        let ceiling = ChildSessionAuthorityCeiling {
            capabilities: vec![capability.clone()],
            workspace_scopes: vec![String::from(WORKSPACE)],
            models: vec![String::from(MODEL)],
            tools: vec![String::from(TOOL)],
            limits,
        };
        ConcreteChildSessionAdmissionDraft {
            subagent,
            context_packet: child.context_packet,
            objective: String::from(OBJECTIVE),
            profile_id: String::from(PROFILE),
            parent_authority_identity: String::from(HASH_A),
            workspace_scope: String::from(WORKSPACE),
            model: String::from(MODEL),
            tools: vec![String::from(TOOL)],
            requested_capabilities: vec![capability.clone()],
            parent: ceiling.clone(),
            profile: ceiling,
            semantic_role_capabilities: vec![capability.clone()],
            workspace_capabilities: vec![capability.clone()],
            host_capabilities: vec![capability],
            requested_limits: limits,
            handoff_mode: SubagentHandoffMode::StatusPlusSummary,
            kernel_profile: None,
            skill_ids: Vec::new(),
        }
    }
}

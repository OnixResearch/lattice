//! OpenClaw-inspired session isolation and session-tool planning DTOs.
//!
//! This module plans session keys, maintenance previews, visibility clamps, and
//! inter-session provenance over host-supplied metadata only. Reusable crates do
//! not discover stores, read transcript files, use clocks, run cleanup jobs, or
//! deliver cross-session messages.

#![cfg_attr(animus_octet, allow(tigerstyle::path_segment_repetition))]

const AGENT_ID_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("session_agent_id");
const ACCOUNT_ID_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("session_account_id");
const CHANNEL_ID_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("session_channel_id");
const PEER_ID_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("session_peer_id");
const GROUP_ID_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("session_group_id");
const THREAD_ID_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("session_thread_id");
const ORIGIN_ID_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("session_origin_id");
const SESSION_ID_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("session_id");
const TOOL_ID_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("session_tool_id");
const LINEAGE_ID_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("session_lineage_id");
const SUMMARY_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("session_summary");
const KEY_PREFIX: &str = "session";

/// Minimum positive maintenance, preview, reply, or announce bound.
pub const MIN_SESSION_BOUND: u32 = 1;
const MIN_SESSION_BOUND_U64: u64 = 1;

const LEAK_RAW_PROMPT: &str = "RAW_PROMPT:";
const LEAK_TRANSCRIPT: &str = "RAW_TRANSCRIPT:";
const LEAK_PROVIDER_PAYLOAD: &str = "PROVIDER_PAYLOAD:";
const LEAK_CREDENTIAL: &str = "CREDENTIAL:";
const LEAK_FULL_PATH: &str = "/home/";
const LEAK_UNBOUNDED_OUTPUT: &str = "UNBOUNDED_OUTPUT:";
const UNSAFE_TEXT_MARKERS: &[&str] = &[
    LEAK_RAW_PROMPT,
    LEAK_TRANSCRIPT,
    LEAK_PROVIDER_PAYLOAD,
    LEAK_CREDENTIAL,
    LEAK_FULL_PATH,
    LEAK_UNBOUNDED_OUTPUT,
];

/// Session isolation planning error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    /// Generic adapter field validation failed.
    Contract(crate::AdapterContractError),
    /// A BLAKE3 identity failed validation.
    Artifact(crate::ArtifactValidationError),
    /// Host-supplied metadata or service snapshot is absent.
    MissingSessionService,
    /// Reusable code was asked to discover ambient stores or transcripts.
    AmbientStoreDiscovery,
    /// Required metadata was absent for the selected source kind.
    MissingMetadata {
        /// Missing field label.
        field: &'static str,
    },
    /// Metadata is ambiguous for deterministic key planning.
    AmbiguousMetadata,
    /// Shared direct-message context would leak across senders with tool authority.
    SharedDirectMessageLeakage,
    /// Maintenance policy bounds are invalid.
    InvalidMaintenancePolicy,
    /// Requested visibility exceeds the host policy ceiling.
    VisibilityDenied,
    /// A request tried to inspect another agent outside admitted visibility.
    CrossAgentAccess,
    /// History or transcript preview lacks a bounded limit.
    UnboundedHistory,
    /// Inter-session influence metadata is incomplete.
    MissingProvenance,
    /// Text contained raw prompt, transcript, provider payload, credential, path, or unbounded
    /// output markers.
    UnsafePayload {
        /// Field that contained unsafe text.
        field: &'static str,
        /// Marker that caused rejection.
        marker: &'static str,
    },
}

impl From<crate::AdapterContractError> for Error {
    fn from(error: crate::AdapterContractError) -> Self {
        Self::Contract(error)
    }
}

impl From<crate::ArtifactValidationError> for Error {
    fn from(error: crate::ArtifactValidationError) -> Self {
        Self::Artifact(error)
    }
}

/// Source kind used to plan a session key.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SessionSourceKind {
    /// Main agent session.
    Main,
    /// Direct-message session using the selected direct-message scope.
    DirectMessage,
    /// Channel and peer scoped session.
    ChannelPeer,
    /// Account, channel, and peer scoped session.
    AccountChannelPeer,
    /// Group session.
    Group,
    /// Channel session.
    Channel,
    /// Thread or topic session.
    ThreadTopic,
    /// Cron-originated session.
    Cron,
    /// Hook-originated session.
    Hook,
    /// Node-originated session.
    Node,
    /// Subagent-originated session.
    Subagent,
}

impl SessionSourceKind {
    /// Returns the stable source-kind label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Main => "main",
            Self::DirectMessage => "direct_message",
            Self::ChannelPeer => "channel_peer",
            Self::AccountChannelPeer => "account_channel_peer",
            Self::Group => "group",
            Self::Channel => "channel",
            Self::ThreadTopic => "thread_topic",
            Self::Cron => "cron",
            Self::Hook => "hook",
            Self::Node => "node",
            Self::Subagent => "subagent",
        }
    }
}

/// Direct-message session key scope.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DirectMessageScope {
    /// Shared main direct-message scope.
    Main,
    /// Peer-specific direct-message scope.
    PerPeer,
    /// Channel and peer scoped direct-message scope.
    PerChannelPeer,
    /// Account, channel, and peer scoped direct-message scope.
    PerAccountChannelPeer,
}

impl DirectMessageScope {
    /// Returns the stable direct-message scope label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Main => "main",
            Self::PerPeer => "per_peer",
            Self::PerChannelPeer => "per_channel_peer",
            Self::PerAccountChannelPeer => "per_account_channel_peer",
        }
    }
}

/// Privacy diagnostic emitted by key planning.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PrivacyDiagnostic {
    /// Shared direct-message scope was used in a multi-sender context.
    SharedDirectMessageScope,
    /// Identity-link metadata normalized a peer identity.
    IdentityLinkNormalized,
}

/// Borrowed identity-link metadata before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IdentityLinkDraft<'a> {
    /// Local or channel-specific sender identity.
    pub local_peer_id: &'a str,
    /// Canonical host-approved peer identity.
    pub canonical_peer_id: &'a str,
}

/// Validated identity-link metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdentityLink {
    local_peer_id: alloc::string::String,
    canonical_peer_id: alloc::string::String,
}

impl IdentityLink {
    /// Builds validated identity-link metadata.
    ///
    /// # Errors
    ///
    /// Returns [`Error`] when identity labels are missing or unsafe.
    pub fn new(draft: &IdentityLinkDraft<'_>) -> Result<Self, Error> {
        Ok(Self {
            local_peer_id: safe_text(PEER_ID_FIELD, draft.local_peer_id)?,
            canonical_peer_id: safe_text(PEER_ID_FIELD, draft.canonical_peer_id)?,
        })
    }

    /// Returns local peer identity.
    #[must_use]
    pub fn local_peer_id(&self) -> &str {
        self.local_peer_id.as_str()
    }

    /// Returns canonical peer identity.
    #[must_use]
    pub fn canonical_peer_id(&self) -> &str {
        self.canonical_peer_id.as_str()
    }
}

/// Borrowed session key planning input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SessionKeyDraft<'a> {
    /// Agent identity.
    pub agent_id: &'a str,
    /// Source kind.
    pub source_kind: SessionSourceKind,
    /// Direct-message scope when applicable.
    pub direct_message_scope: DirectMessageScope,
    /// Optional account identity.
    pub account_id: Option<&'a str>,
    /// Optional channel identity.
    pub channel_id: Option<&'a str>,
    /// Optional peer identity.
    pub peer_id: Option<&'a str>,
    /// Optional group identity.
    pub group_id: Option<&'a str>,
    /// Optional thread or topic identity.
    pub thread_id: Option<&'a str>,
    /// Optional cron, hook, node, or subagent origin identity.
    pub origin_id: Option<&'a str>,
    /// Optional identity-link normalization metadata.
    pub identity_link: Option<IdentityLinkDraft<'a>>,
    /// Whether this metadata represents multiple possible senders.
    pub multi_sender: bool,
    /// Whether tool-enabled authority is active in the planned direct-message scope.
    pub tool_enabled: bool,
    /// Whether reusable code was asked to discover ambient store state.
    pub ambient_store_discovery: bool,
}

/// Deterministic session key planning result.
// r[impl agent_harness_runtime.openclaw_sessions.key_planning]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionKeyPlan {
    key: alloc::string::String,
    source_kind: SessionSourceKind,
    diagnostics: alloc::vec::Vec<PrivacyDiagnostic>,
}

impl SessionKeyPlan {
    /// Returns deterministic session key text.
    #[must_use]
    pub fn key(&self) -> &str {
        self.key.as_str()
    }

    /// Returns source kind used to build the key.
    #[must_use]
    pub const fn source_kind(&self) -> SessionSourceKind {
        self.source_kind
    }

    /// Returns privacy diagnostics emitted during planning.
    #[must_use]
    pub fn diagnostics(&self) -> &[PrivacyDiagnostic] {
        self.diagnostics.as_slice()
    }
}

/// Plans a deterministic session key without reading stores or transcripts.
///
/// # Errors
///
/// Returns [`Error`] for ambient discovery requests, missing required metadata,
/// or ambiguous direct-message inputs.
///
/// # Panics
///
/// Panics if ambient-discovery or normalized-agent post-conditions are violated
/// after validation.
pub fn plan_session_key(draft: &SessionKeyDraft<'_>) -> Result<SessionKeyPlan, Error> {
    if draft.ambient_store_discovery {
        return Err(Error::AmbientStoreDiscovery);
    }
    assert!(!draft.ambient_store_discovery);
    let agent = safe_text(AGENT_ID_FIELD, draft.agent_id)?;
    assert!(!agent.is_empty());
    let mut diagnostics = alloc::vec::Vec::new();
    let key = match draft.source_kind {
        SessionSourceKind::Main => key_from_segments(&[KEY_PREFIX, agent.as_str(), "main"]),
        SessionSourceKind::DirectMessage => plan_direct_message_key(draft, agent.as_str(), &mut diagnostics)?,
        SessionSourceKind::ChannelPeer => {
            let channel = required_optional(CHANNEL_ID_FIELD, draft.channel_id)?;
            let peer = normalized_peer(draft, &mut diagnostics)?;
            key_from_segments(&[KEY_PREFIX, agent.as_str(), "channel_peer", &channel, &peer])
        }
        SessionSourceKind::AccountChannelPeer => {
            let account = required_optional(ACCOUNT_ID_FIELD, draft.account_id)?;
            let channel = required_optional(CHANNEL_ID_FIELD, draft.channel_id)?;
            let peer = normalized_peer(draft, &mut diagnostics)?;
            key_from_segments(&[
                KEY_PREFIX,
                agent.as_str(),
                "account_channel_peer",
                &account,
                &channel,
                &peer,
            ])
        }
        SessionSourceKind::Group => {
            let group = required_optional(GROUP_ID_FIELD, draft.group_id)?;
            key_from_segments(&[KEY_PREFIX, agent.as_str(), "group", &group])
        }
        SessionSourceKind::Channel => {
            let channel = required_optional(CHANNEL_ID_FIELD, draft.channel_id)?;
            key_from_segments(&[KEY_PREFIX, agent.as_str(), "channel", &channel])
        }
        SessionSourceKind::ThreadTopic => {
            let channel = required_optional(CHANNEL_ID_FIELD, draft.channel_id)?;
            let thread = required_optional(THREAD_ID_FIELD, draft.thread_id)?;
            key_from_segments(&[KEY_PREFIX, agent.as_str(), "thread", &channel, &thread])
        }
        SessionSourceKind::Cron => origin_key(agent.as_str(), "cron", draft.origin_id)?,
        SessionSourceKind::Hook => origin_key(agent.as_str(), "hook", draft.origin_id)?,
        SessionSourceKind::Node => origin_key(agent.as_str(), "node", draft.origin_id)?,
        SessionSourceKind::Subagent => origin_key(agent.as_str(), "subagent", draft.origin_id)?,
    };
    Ok(SessionKeyPlan {
        key,
        source_kind: draft.source_kind,
        diagnostics,
    })
}

/// Logical age class supplied by a host snapshot.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AgeClass {
    /// Current or hot session data.
    Current,
    /// Stale session data eligible for pruning.
    Stale,
    /// Archived session data.
    Archived,
}

/// Borrowed session metadata before maintenance planning.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SessionMetadataDraft<'a> {
    /// Session identity.
    pub session_id: &'a str,
    /// Logical age class supplied by the host.
    pub age_class: AgeClass,
    /// Entry count supplied by the host.
    pub entry_count: u32,
    /// Transcript byte count supplied by the host.
    pub byte_count: u64,
    /// Existing archive count supplied by the host.
    pub archive_count: u32,
}

/// Validated session metadata snapshot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionMetadata {
    session_id: alloc::string::String,
    age_class: AgeClass,
    entry_count: u32,
    byte_count: u64,
    archive_count: u32,
}

impl SessionMetadata {
    /// Builds a host-loaded session metadata snapshot.
    ///
    /// # Errors
    ///
    /// Returns [`Error`] when session identity is malformed or unsafe.
    pub fn new(draft: &SessionMetadataDraft<'_>) -> Result<Self, Error> {
        Ok(Self {
            session_id: safe_text(SESSION_ID_FIELD, draft.session_id)?,
            age_class: draft.age_class,
            entry_count: draft.entry_count,
            byte_count: draft.byte_count,
            archive_count: draft.archive_count,
        })
    }

    /// Returns session identity.
    #[must_use]
    pub fn session_id(&self) -> &str {
        self.session_id.as_str()
    }
}

/// Borrowed maintenance policy before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MaintenancePolicyDraft {
    /// Maximum entries to retain in the active transcript.
    pub max_entries: u32,
    /// Byte threshold at which transcript rotation is proposed.
    pub rotation_threshold_bytes: u64,
    /// Maximum retained archives.
    pub archive_retention_count: u32,
    /// Disk budget for active transcript bytes.
    pub disk_budget_bytes: u64,
    /// Whether this is preview-only planning.
    pub preview: bool,
}

/// Validated maintenance policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MaintenancePolicy {
    max_entries: u32,
    rotation_threshold_bytes: u64,
    archive_retention_count: u32,
    disk_budget_bytes: u64,
    preview: bool,
}

impl MaintenancePolicy {
    /// Builds a deterministic maintenance policy.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidMaintenancePolicy`] when any bound is below the accepted minimum.
    pub const fn new(draft: MaintenancePolicyDraft) -> Result<Self, Error> {
        let is_max_entries_too_low = draft.max_entries < MIN_SESSION_BOUND;
        let is_rotation_threshold_too_low = draft.rotation_threshold_bytes < MIN_SESSION_BOUND_U64;
        let is_archive_retention_too_low = draft.archive_retention_count < MIN_SESSION_BOUND;
        let is_disk_budget_too_low = draft.disk_budget_bytes < MIN_SESSION_BOUND_U64;
        if is_max_entries_too_low {
            return Err(Error::InvalidMaintenancePolicy);
        }
        if is_rotation_threshold_too_low {
            return Err(Error::InvalidMaintenancePolicy);
        }
        if is_archive_retention_too_low {
            return Err(Error::InvalidMaintenancePolicy);
        }
        if is_disk_budget_too_low {
            return Err(Error::InvalidMaintenancePolicy);
        }
        Ok(Self {
            max_entries: draft.max_entries,
            rotation_threshold_bytes: draft.rotation_threshold_bytes,
            archive_retention_count: draft.archive_retention_count,
            disk_budget_bytes: draft.disk_budget_bytes,
            preview: draft.preview,
        })
    }

    /// Returns whether planning is preview-only.
    #[must_use]
    pub const fn preview(self) -> bool {
        self.preview
    }
}

/// Maintenance action kind proposed without mutating stores.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MaintenanceActionKind {
    /// Stale entry pruning is proposed.
    PruneStale,
    /// Entry-count cap enforcement is proposed.
    CapEntries,
    /// Transcript rotation is proposed.
    RotateTranscript,
    /// Archive retention trimming is proposed.
    TrimArchives,
    /// Disk-budget pressure handling is proposed.
    DiskBudgetPressure,
}

/// Proposed maintenance action.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MaintenanceAction {
    session_id: alloc::string::String,
    kind: MaintenanceActionKind,
    affected_count: u32,
}

impl MaintenanceAction {
    /// Returns affected session identity.
    #[must_use]
    pub fn session_id(&self) -> &str {
        self.session_id.as_str()
    }

    /// Returns action kind.
    #[must_use]
    pub const fn kind(&self) -> MaintenanceActionKind {
        self.kind
    }

    /// Returns safe count affected by the plan.
    #[must_use]
    pub const fn affected_count(&self) -> u32 {
        self.affected_count
    }
}

/// Deterministic maintenance plan.
// r[impl agent_harness_runtime.openclaw_sessions.maintenance_policy]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MaintenancePlan {
    preview: bool,
    actions: alloc::vec::Vec<MaintenanceAction>,
}

impl MaintenancePlan {
    /// Returns whether this plan is preview-only.
    #[must_use]
    pub const fn preview(&self) -> bool {
        self.preview
    }

    /// Returns proposed actions in deterministic order.
    #[must_use]
    pub fn actions(&self) -> &[MaintenanceAction] {
        self.actions.as_slice()
    }
}

/// Plans deterministic maintenance over host-loaded metadata.
#[must_use]
pub fn plan_session_maintenance(policy: MaintenancePolicy, sessions: &[SessionMetadata]) -> MaintenancePlan {
    let mut actions = alloc::vec::Vec::new();
    for session in sessions {
        push_maintenance_actions(policy, session, &mut actions);
    }
    MaintenancePlan {
        preview: policy.preview(),
        actions,
    }
}

/// Session-tool visibility mode ordered by breadth.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum SessionVisibilityMode {
    /// Current session only.
    SelfOnly,
    /// Current session plus spawned descendants.
    Tree,
    /// Sessions for the same agent.
    Agent,
    /// All admitted sessions.
    All,
}

impl SessionVisibilityMode {
    /// Returns stable visibility label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SelfOnly => "self",
            Self::Tree => "tree",
            Self::Agent => "agent",
            Self::All => "all",
        }
    }
}

/// Session boundary posture.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SessionBoundaryPosture {
    /// Normal host-owned session.
    Normal,
    /// Sandboxed session.
    Sandboxed,
    /// Delegated or child session.
    Delegated,
}

/// Borrowed session visibility request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SessionVisibilityRequestDraft<'a> {
    /// Current session identity.
    pub current_session_id: &'a str,
    /// Current agent identity.
    pub current_agent_id: &'a str,
    /// Target agent identity for the requested operation.
    pub target_agent_id: &'a str,
    /// Requested visibility.
    pub requested: SessionVisibilityMode,
    /// Host policy ceiling.
    pub host_ceiling: SessionVisibilityMode,
    /// Boundary posture.
    pub boundary: SessionBoundaryPosture,
    /// Optional bounded history preview count.
    pub history_preview_entries: Option<u32>,
}

/// Planned session-tool visibility.
// r[impl agent_harness_runtime.openclaw_sessions.tools_visibility]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionVisibilityPlan {
    current_session_id: alloc::string::String,
    current_agent_id: alloc::string::String,
    effective_visibility: SessionVisibilityMode,
    history_preview_entries: u32,
}

impl SessionVisibilityPlan {
    /// Returns current session identity.
    #[must_use]
    pub fn current_session_id(&self) -> &str {
        self.current_session_id.as_str()
    }

    /// Returns current agent identity.
    #[must_use]
    pub fn current_agent_id(&self) -> &str {
        self.current_agent_id.as_str()
    }

    /// Returns effective visibility.
    #[must_use]
    pub const fn effective_visibility(&self) -> SessionVisibilityMode {
        self.effective_visibility
    }

    /// Returns bounded history preview count.
    #[must_use]
    pub const fn history_preview_entries(&self) -> u32 {
        self.history_preview_entries
    }
}

/// Plans session tool visibility before transcript metadata is exposed.
///
/// # Errors
///
/// Returns [`Error`] when the request widens beyond host policy, crosses agents,
/// or lacks bounded history preview limits.
///
/// # Panics
///
/// Panics if bounded-history or effective-ceiling post-conditions are violated
/// after validation.
pub fn plan_session_tool_visibility(draft: &SessionVisibilityRequestDraft<'_>) -> Result<SessionVisibilityPlan, Error> {
    let current_session_id = safe_text(SESSION_ID_FIELD, draft.current_session_id)?;
    let current_agent_id = safe_text(AGENT_ID_FIELD, draft.current_agent_id)?;
    let target_agent_id = safe_text(AGENT_ID_FIELD, draft.target_agent_id)?;
    let Some(history_preview_entries) = draft.history_preview_entries else {
        return Err(Error::UnboundedHistory);
    };
    if history_preview_entries < MIN_SESSION_BOUND {
        return Err(Error::UnboundedHistory);
    }
    assert!(history_preview_entries >= MIN_SESSION_BOUND);
    let effective_ceiling = boundary_ceiling(draft.boundary, draft.host_ceiling);
    if draft.requested > effective_ceiling {
        return Err(Error::VisibilityDenied);
    }
    assert!(draft.requested <= effective_ceiling);
    if current_agent_id != target_agent_id && draft.requested < SessionVisibilityMode::All {
        return Err(Error::CrossAgentAccess);
    }
    Ok(SessionVisibilityPlan {
        current_session_id,
        current_agent_id,
        effective_visibility: draft.requested,
        history_preview_entries,
    })
}

/// Handoff mode for inter-session influence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HandoffMode {
    /// Send a bounded message to another session.
    Send,
    /// Spawn a child session.
    Spawn,
    /// Announce bounded status without task handoff.
    Announce,
}

/// Borrowed inter-session provenance before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InterSessionProvenanceDraft<'a> {
    /// Source session identity.
    pub source_session_id: &'a str,
    /// Target session identity.
    pub target_session_id: &'a str,
    /// Source tool identity.
    pub source_tool_id: &'a str,
    /// Optional source channel identity.
    pub source_channel_id: Option<&'a str>,
    /// Child lineage identity, required for spawn and routed influence auditing.
    pub child_lineage_id: Option<&'a str>,
    /// Handoff mode.
    pub handoff_mode: HandoffMode,
    /// Reply bound.
    pub reply_bound: u32,
    /// Announce bound.
    pub announce_bound: u32,
    /// BLAKE3 identity of host-held influence body.
    pub influence_hash: &'a str,
    /// Safe influence summary.
    pub summary: &'a str,
}

/// Safe inter-session influence record.
// r[impl agent_harness_runtime.openclaw_sessions.inter_session_provenance]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InterSessionProvenance {
    source_session_id: alloc::string::String,
    target_session_id: alloc::string::String,
    source_tool_id: alloc::string::String,
    source_channel_id: Option<alloc::string::String>,
    child_lineage_id: alloc::string::String,
    handoff_mode: HandoffMode,
    reply_bound: u32,
    announce_bound: u32,
    influence_hash: crate::ArtifactHash,
    summary: alloc::string::String,
}

impl InterSessionProvenance {
    /// Builds a safe inter-session influence record.
    ///
    /// # Errors
    ///
    /// Returns [`Error`] when provenance is missing, unbounded, malformed, or unsafe.
    pub fn new(draft: &InterSessionProvenanceDraft<'_>) -> Result<Self, Error> {
        if draft.reply_bound < MIN_SESSION_BOUND || draft.announce_bound < MIN_SESSION_BOUND {
            return Err(Error::MissingProvenance);
        }
        let Some(child_lineage_id) = draft.child_lineage_id else {
            return Err(Error::MissingProvenance);
        };
        Ok(Self {
            source_session_id: safe_text(SESSION_ID_FIELD, draft.source_session_id)?,
            target_session_id: safe_text(SESSION_ID_FIELD, draft.target_session_id)?,
            source_tool_id: safe_text(TOOL_ID_FIELD, draft.source_tool_id)?,
            source_channel_id: draft
                .source_channel_id
                .map(|channel_id| safe_text(CHANNEL_ID_FIELD, channel_id))
                .transpose()?,
            child_lineage_id: safe_text(LINEAGE_ID_FIELD, child_lineage_id)?,
            handoff_mode: draft.handoff_mode,
            reply_bound: draft.reply_bound,
            announce_bound: draft.announce_bound,
            influence_hash: crate::ArtifactHash::new(draft.influence_hash)?,
            summary: safe_text(SUMMARY_FIELD, draft.summary)?,
        })
    }

    /// Returns source session identity.
    #[must_use]
    pub fn source_session_id(&self) -> &str {
        self.source_session_id.as_str()
    }

    /// Returns target session identity.
    #[must_use]
    pub fn target_session_id(&self) -> &str {
        self.target_session_id.as_str()
    }

    /// Returns source tool identity.
    #[must_use]
    pub fn source_tool_id(&self) -> &str {
        self.source_tool_id.as_str()
    }

    /// Returns optional source channel identity.
    #[must_use]
    pub fn source_channel_id(&self) -> Option<&str> {
        self.source_channel_id.as_deref()
    }

    /// Returns child lineage identity.
    #[must_use]
    pub fn child_lineage_id(&self) -> &str {
        self.child_lineage_id.as_str()
    }

    /// Returns handoff mode.
    #[must_use]
    pub const fn handoff_mode(&self) -> HandoffMode {
        self.handoff_mode
    }

    /// Returns reply bound.
    #[must_use]
    pub const fn reply_bound(&self) -> u32 {
        self.reply_bound
    }

    /// Returns announce bound.
    #[must_use]
    pub const fn announce_bound(&self) -> u32 {
        self.announce_bound
    }

    /// Returns influence body identity.
    #[must_use]
    pub const fn influence_hash(&self) -> &crate::ArtifactHash {
        &self.influence_hash
    }

    /// Returns safe summary.
    #[must_use]
    pub fn summary(&self) -> &str {
        self.summary.as_str()
    }
}

#[cfg_attr(animus_octet, allow(tigerstyle::borrowed_argument_types))]
fn plan_direct_message_key(
    draft: &SessionKeyDraft<'_>,
    agent: &str,
    diagnostics: &mut alloc::vec::Vec<PrivacyDiagnostic>,
) -> Result<alloc::string::String, Error> {
    match draft.direct_message_scope {
        DirectMessageScope::Main => {
            if draft.multi_sender && draft.tool_enabled {
                diagnostics.push(PrivacyDiagnostic::SharedDirectMessageScope);
            }
            Ok(key_from_segments(&[KEY_PREFIX, agent, "dm", "main"]))
        }
        DirectMessageScope::PerPeer => {
            let peer = normalized_peer(draft, diagnostics)?;
            Ok(key_from_segments(&[KEY_PREFIX, agent, "dm", "peer", &peer]))
        }
        DirectMessageScope::PerChannelPeer => {
            let channel = required_optional(CHANNEL_ID_FIELD, draft.channel_id)?;
            let peer = normalized_peer(draft, diagnostics)?;
            Ok(key_from_segments(&[KEY_PREFIX, agent, "dm", "channel", &channel, &peer]))
        }
        DirectMessageScope::PerAccountChannelPeer => {
            let account = required_optional(ACCOUNT_ID_FIELD, draft.account_id)?;
            let channel = required_optional(CHANNEL_ID_FIELD, draft.channel_id)?;
            let peer = normalized_peer(draft, diagnostics)?;
            Ok(key_from_segments(&[KEY_PREFIX, agent, "dm", "account", &account, &channel, &peer]))
        }
    }
}

#[cfg_attr(animus_octet, allow(tigerstyle::ambiguous_params))]
fn origin_key(
    agent: &str,
    source_label: &'static str,
    origin_id: Option<&str>,
) -> Result<alloc::string::String, Error> {
    let origin = required_optional(ORIGIN_ID_FIELD, origin_id)?;
    Ok(key_from_segments(&[KEY_PREFIX, agent, source_label, &origin]))
}

#[cfg_attr(animus_octet, allow(tigerstyle::borrowed_argument_types))]
fn normalized_peer(
    draft: &SessionKeyDraft<'_>,
    diagnostics: &mut alloc::vec::Vec<PrivacyDiagnostic>,
) -> Result<alloc::string::String, Error> {
    let peer = required_optional(PEER_ID_FIELD, draft.peer_id)?;
    match draft.identity_link {
        Some(identity_link) => {
            let identity_link = IdentityLink::new(&identity_link)?;
            if identity_link.local_peer_id() != peer {
                return Err(Error::AmbiguousMetadata);
            }
            diagnostics.push(PrivacyDiagnostic::IdentityLinkNormalized);
            Ok(alloc::string::String::from(identity_link.canonical_peer_id()))
        }
        None => Ok(peer),
    }
}

#[cfg_attr(animus_octet, allow(tigerstyle::borrowed_argument_types))]
fn push_maintenance_actions(
    policy: MaintenancePolicy,
    session: &SessionMetadata,
    actions: &mut alloc::vec::Vec<MaintenanceAction>,
) {
    assert!(policy.max_entries >= MIN_SESSION_BOUND);
    assert!(policy.rotation_threshold_bytes >= MIN_SESSION_BOUND_U64);
    if session.age_class == AgeClass::Stale {
        actions.push(maintenance_action(session, MaintenanceActionKind::PruneStale));
    }
    if session.entry_count > policy.max_entries {
        actions.push(maintenance_action(session, MaintenanceActionKind::CapEntries));
    }
    if session.byte_count > policy.rotation_threshold_bytes {
        actions.push(maintenance_action(session, MaintenanceActionKind::RotateTranscript));
    }
    if session.archive_count > policy.archive_retention_count {
        actions.push(maintenance_action(session, MaintenanceActionKind::TrimArchives));
    }
    if session.byte_count > policy.disk_budget_bytes {
        actions.push(maintenance_action(session, MaintenanceActionKind::DiskBudgetPressure));
    }
}

fn maintenance_action(session: &SessionMetadata, kind: MaintenanceActionKind) -> MaintenanceAction {
    MaintenanceAction {
        session_id: alloc::string::String::from(session.session_id()),
        kind,
        affected_count: session.entry_count,
    }
}

fn boundary_ceiling(boundary: SessionBoundaryPosture, host_ceiling: SessionVisibilityMode) -> SessionVisibilityMode {
    let posture_ceiling = match boundary {
        SessionBoundaryPosture::Normal => SessionVisibilityMode::All,
        SessionBoundaryPosture::Sandboxed | SessionBoundaryPosture::Delegated => SessionVisibilityMode::Tree,
    };
    core::cmp::min(host_ceiling, posture_ceiling)
}

fn required_optional(
    field_name: crate::validation::FieldName,
    value: Option<&str>,
) -> Result<alloc::string::String, Error> {
    let Some(value) = value else {
        return Err(Error::MissingMetadata {
            field: field_name.as_str(),
        });
    };
    safe_text(field_name, value)
}

fn key_from_segments(segments: &[&str]) -> alloc::string::String {
    segments.join(":")
}

fn safe_text(field_name: crate::validation::FieldName, value: &str) -> Result<alloc::string::String, Error> {
    let normalized = crate::validation::normalized_adapter_field(field_name, value)?;
    reject_unsafe_text(field_name, normalized.as_str())?;
    Ok(normalized)
}

fn reject_unsafe_text(field_name: crate::validation::FieldName, value: &str) -> Result<(), Error> {
    let lower = value.to_ascii_lowercase();
    for marker in UNSAFE_TEXT_MARKERS {
        if lower.contains(&marker.to_ascii_lowercase()) {
            return Err(Error::UnsafePayload {
                field: field_name.as_str(),
                marker,
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const AGENT_ID: &str = "agent-main";
    const AGENT_OTHER: &str = "agent-other";
    const ACCOUNT_ID: &str = "account-main";
    const CHANNEL_ID: &str = "channel-main";
    const PEER_ONE: &str = "peer-one";
    const PEER_TWO: &str = "peer-two";
    const PEER_CANONICAL: &str = "peer-canonical";
    const SESSION_SOURCE: &str = "session-source";
    const SESSION_TARGET: &str = "session-target";
    const TOOL_ID: &str = "session-send";
    const LINEAGE_ID: &str = "lineage-main";
    const SAFE_SUMMARY: &str = "safe routed influence summary";
    const UNSAFE_TRANSCRIPT: &str = "RAW_TRANSCRIPT:hidden";
    const HASH_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const MAX_ENTRIES: u32 = 8;
    const ENTRY_COUNT: u32 = 13;
    const ARCHIVE_RETENTION: u32 = 2;
    const ARCHIVE_COUNT: u32 = 5;
    const ROTATION_BYTES: u64 = 1024;
    const DISK_BUDGET_BYTES: u64 = 2048;
    const BYTE_COUNT: u64 = 4096;
    const PREVIEW_ENTRIES: u32 = 4;
    const BOUND: u32 = 1;

    #[test]
    fn per_channel_peer_keys_maintenance_visibility_and_provenance_are_safe() {
        // r[verify agent_harness_runtime.openclaw_sessions.key_planning]
        // r[verify agent_harness_runtime.openclaw_sessions.maintenance_policy]
        // r[verify agent_harness_runtime.openclaw_sessions.tools_visibility]
        // r[verify agent_harness_runtime.openclaw_sessions.inter_session_provenance]
        // r[verify agent_harness_runtime.openclaw_sessions.verification]
        let first = plan_session_key(&dm_key(PEER_ONE, None)).expect("first key should plan");
        let second = plan_session_key(&dm_key(PEER_TWO, None)).expect("second key should plan");
        let normalized = plan_session_key(&dm_key(
            PEER_ONE,
            Some(IdentityLinkDraft {
                local_peer_id: PEER_ONE,
                canonical_peer_id: PEER_CANONICAL,
            }),
        ))
        .expect("identity-linked key should plan");
        let metadata = SessionMetadata::new(&SessionMetadataDraft {
            session_id: SESSION_SOURCE,
            age_class: AgeClass::Stale,
            entry_count: ENTRY_COUNT,
            byte_count: BYTE_COUNT,
            archive_count: ARCHIVE_COUNT,
        })
        .expect("metadata should validate");
        let maintenance = plan_session_maintenance(policy(), &[metadata]);
        let visibility = plan_session_tool_visibility(&SessionVisibilityRequestDraft {
            current_session_id: SESSION_SOURCE,
            current_agent_id: AGENT_ID,
            target_agent_id: AGENT_ID,
            requested: SessionVisibilityMode::Tree,
            host_ceiling: SessionVisibilityMode::Tree,
            boundary: SessionBoundaryPosture::Sandboxed,
            history_preview_entries: Some(PREVIEW_ENTRIES),
        })
        .expect("visibility should plan");
        let provenance = InterSessionProvenance::new(&provenance(Some(LINEAGE_ID), SAFE_SUMMARY))
            .expect("provenance should validate");

        assert_ne!(first.key(), second.key());
        assert!(normalized.diagnostics().contains(&PrivacyDiagnostic::IdentityLinkNormalized));
        assert_eq!(maintenance.actions().len(), 5);
        assert_eq!(visibility.effective_visibility(), SessionVisibilityMode::Tree);
        assert_eq!(provenance.handoff_mode(), HandoffMode::Send);
    }

    #[test]
    fn shared_dm_ambient_ambiguous_and_visibility_widening_fail_closed() {
        // r[verify agent_harness_runtime.openclaw_sessions.scope]
        // r[verify agent_harness_runtime.openclaw_sessions.key_planning]
        // r[verify agent_harness_runtime.openclaw_sessions.tools_visibility]
        // r[verify agent_harness_runtime.openclaw_sessions.verification]
        let shared = plan_session_key(&SessionKeyDraft {
            source_kind: SessionSourceKind::DirectMessage,
            direct_message_scope: DirectMessageScope::Main,
            peer_id: Some(PEER_ONE),
            multi_sender: true,
            tool_enabled: true,
            ..base_key()
        })
        .expect("shared key should emit diagnostic rather than read stores");
        let ambient = plan_session_key(&SessionKeyDraft {
            ambient_store_discovery: true,
            ..base_key()
        })
        .expect_err("ambient store discovery should fail");
        let ambiguous = plan_session_key(&dm_key(
            PEER_ONE,
            Some(IdentityLinkDraft {
                local_peer_id: PEER_TWO,
                canonical_peer_id: PEER_CANONICAL,
            }),
        ))
        .expect_err("mismatched identity link should fail");
        let widening = plan_session_tool_visibility(&SessionVisibilityRequestDraft {
            current_session_id: SESSION_SOURCE,
            current_agent_id: AGENT_ID,
            target_agent_id: AGENT_ID,
            requested: SessionVisibilityMode::All,
            host_ceiling: SessionVisibilityMode::Tree,
            boundary: SessionBoundaryPosture::Sandboxed,
            history_preview_entries: Some(PREVIEW_ENTRIES),
        })
        .expect_err("sandbox widening should fail");
        let cross_agent = plan_session_tool_visibility(&SessionVisibilityRequestDraft {
            current_session_id: SESSION_SOURCE,
            current_agent_id: AGENT_ID,
            target_agent_id: AGENT_OTHER,
            requested: SessionVisibilityMode::Agent,
            host_ceiling: SessionVisibilityMode::Agent,
            boundary: SessionBoundaryPosture::Normal,
            history_preview_entries: Some(PREVIEW_ENTRIES),
        })
        .expect_err("cross-agent access should fail");

        assert!(shared.diagnostics().contains(&PrivacyDiagnostic::SharedDirectMessageScope));
        assert_eq!(ambient, Error::AmbientStoreDiscovery);
        assert_eq!(ambiguous, Error::AmbiguousMetadata);
        assert_eq!(widening, Error::VisibilityDenied);
        assert_eq!(cross_agent, Error::CrossAgentAccess);
    }

    #[test]
    fn missing_metadata_unbounded_history_missing_provenance_and_leaks_reject() {
        // r[verify agent_harness_runtime.openclaw_sessions.key_planning]
        // r[verify agent_harness_runtime.openclaw_sessions.tools_visibility]
        // r[verify agent_harness_runtime.openclaw_sessions.inter_session_provenance]
        // r[verify agent_harness_runtime.openclaw_sessions.verification]
        let missing_peer = plan_session_key(&SessionKeyDraft {
            peer_id: None,
            ..dm_key(PEER_ONE, None)
        })
        .expect_err("per-peer key without peer should fail");
        let unbounded_history = plan_session_tool_visibility(&SessionVisibilityRequestDraft {
            current_session_id: SESSION_SOURCE,
            current_agent_id: AGENT_ID,
            target_agent_id: AGENT_ID,
            requested: SessionVisibilityMode::SelfOnly,
            host_ceiling: SessionVisibilityMode::SelfOnly,
            boundary: SessionBoundaryPosture::Normal,
            history_preview_entries: None,
        })
        .expect_err("unbounded history should fail");
        let missing_provenance =
            InterSessionProvenance::new(&provenance(None, SAFE_SUMMARY)).expect_err("missing lineage should fail");
        let unsafe_provenance = InterSessionProvenance::new(&provenance(Some(LINEAGE_ID), UNSAFE_TRANSCRIPT))
            .expect_err("raw transcript should fail");

        assert!(matches!(missing_peer, Error::MissingMetadata { .. }));
        assert_eq!(unbounded_history, Error::UnboundedHistory);
        assert_eq!(missing_provenance, Error::MissingProvenance);
        assert!(matches!(unsafe_provenance, Error::UnsafePayload { .. }));
    }

    const fn base_key() -> SessionKeyDraft<'static> {
        SessionKeyDraft {
            agent_id: AGENT_ID,
            source_kind: SessionSourceKind::Main,
            direct_message_scope: DirectMessageScope::Main,
            account_id: Some(ACCOUNT_ID),
            channel_id: Some(CHANNEL_ID),
            peer_id: Some(PEER_ONE),
            group_id: None,
            thread_id: None,
            origin_id: None,
            identity_link: None,
            multi_sender: false,
            tool_enabled: false,
            ambient_store_discovery: false,
        }
    }

    const fn dm_key(
        peer_id: &'static str,
        identity_link: Option<IdentityLinkDraft<'static>>,
    ) -> SessionKeyDraft<'static> {
        SessionKeyDraft {
            source_kind: SessionSourceKind::DirectMessage,
            direct_message_scope: DirectMessageScope::PerChannelPeer,
            peer_id: Some(peer_id),
            identity_link,
            ..base_key()
        }
    }

    fn policy() -> MaintenancePolicy {
        MaintenancePolicy::new(MaintenancePolicyDraft {
            max_entries: MAX_ENTRIES,
            rotation_threshold_bytes: ROTATION_BYTES,
            archive_retention_count: ARCHIVE_RETENTION,
            disk_budget_bytes: DISK_BUDGET_BYTES,
            preview: true,
        })
        .expect("test policy constants should be valid")
    }

    const fn provenance(
        child_lineage_id: Option<&'static str>,
        summary: &'static str,
    ) -> InterSessionProvenanceDraft<'static> {
        InterSessionProvenanceDraft {
            source_session_id: SESSION_SOURCE,
            target_session_id: SESSION_TARGET,
            source_tool_id: TOOL_ID,
            source_channel_id: Some(CHANNEL_ID),
            child_lineage_id,
            handoff_mode: HandoffMode::Send,
            reply_bound: BOUND,
            announce_bound: BOUND,
            influence_hash: HASH_A,
            summary,
        }
    }
}

//! Workspace shell identifier, diagnostic, and status types.

pub(crate) const SUBJECT_SPACE: &str = "workspace";
pub(crate) const SUBJECT_TAB: &str = "tab";
pub(crate) const SUBJECT_PANE: &str = "pane";
pub(crate) const SUBJECT_LAYOUT: &str = "layout";

const STATUS_WORKING_LABEL: &str = "working";
const STATUS_BLOCKED_LABEL: &str = "blocked";
const STATUS_DONE_LABEL: &str = "done";
const STATUS_IDLE_LABEL: &str = "idle";
const STATUS_UNKNOWN_LABEL: &str = "unknown";

/// Compact session-visible workspace, tab, or pane identifier.
pub type LiveId = String;

/// Durable persisted workspace, tab, pane, or layout identifier.
pub type PersistedId = String;

/// Stable workspace shell diagnostic code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DiagnosticCode {
    /// A live ID was duplicated inside the active session model.
    DuplicateLiveId,
    /// A durable persisted ID was duplicated inside the model or restore payload.
    DuplicatePersistedId,
    /// A command referenced an unknown live workspace, tab, or pane.
    UnknownLiveId,
    /// A command referenced a known retired live ID.
    StaleLiveId,
    /// A restore payload is not structurally usable.
    UnsupportedRestoreShape,
    /// A command was visible but disabled.
    DisabledAction,
    /// A status adapter is absent.
    MissingStatusAdapter,
    /// A status provider is unsupported.
    UnsupportedStatusProvider,
}

/// Deterministic workspace shell diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    /// Stable diagnostic code.
    pub code: DiagnosticCode,
    /// Affected field, action, or ID.
    pub subject: String,
    /// Human-readable diagnostic.
    pub message: String,
}

impl Diagnostic {
    /// Creates a deterministic diagnostic.
    #[must_use]
    pub fn new(
        code: DiagnosticCode,
        subject: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            subject: subject.into(),
            message: message.into(),
        }
    }
}

/// Portable task status vocabulary for workspace panes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskStatus {
    /// Work is actively running.
    Working,
    /// Work cannot proceed without intervention.
    Blocked,
    /// Work completed successfully.
    Done,
    /// No active work is attached.
    Idle,
    /// Status is unavailable or unsupported.
    Unknown,
}

impl TaskStatus {
    /// Returns a stable lowercase display label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Working => STATUS_WORKING_LABEL,
            Self::Blocked => STATUS_BLOCKED_LABEL,
            Self::Done => STATUS_DONE_LABEL,
            Self::Idle => STATUS_IDLE_LABEL,
            Self::Unknown => STATUS_UNKNOWN_LABEL,
        }
    }
}

/// Aggregated task status counts and overall state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusRollup {
    /// Number of working panes.
    pub working: usize,
    /// Number of blocked panes.
    pub blocked: usize,
    /// Number of done panes.
    pub done: usize,
    /// Number of idle panes.
    pub idle: usize,
    /// Number of unknown panes.
    pub unknown: usize,
    /// Deterministic aggregate status.
    pub overall: TaskStatus,
}

impl StatusRollup {
    /// Creates an empty unknown rollup.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            working: 0,
            blocked: 0,
            done: 0,
            idle: 0,
            unknown: 0,
            overall: TaskStatus::Unknown,
        }
    }

    fn increment(&mut self, status: TaskStatus) {
        match status {
            TaskStatus::Working => self.working = self.working.saturating_add(1),
            TaskStatus::Blocked => self.blocked = self.blocked.saturating_add(1),
            TaskStatus::Done => self.done = self.done.saturating_add(1),
            TaskStatus::Idle => self.idle = self.idle.saturating_add(1),
            TaskStatus::Unknown => self.unknown = self.unknown.saturating_add(1),
        }
    }

    fn finalize(mut self) -> Self {
        self.overall = overall(&self);
        self
    }
}

impl Default for StatusRollup {
    fn default() -> Self {
        Self::empty()
    }
}

/// Rolls pane statuses into stable counts and an overall status.
#[must_use]
pub fn rollup(statuses: impl IntoIterator<Item = TaskStatus>) -> StatusRollup {
    let mut rollup = StatusRollup::empty();
    for status in statuses {
        rollup.increment(status);
    }
    rollup.finalize()
}

/// Rolls a tab's panes into stable status counts.
#[must_use]
pub fn tab_rollup(tab: &super::state::Tab) -> StatusRollup {
    rollup(tab.panes.iter().map(|pane| pane.status))
}

/// Rolls a workspace's panes into stable status counts.
#[must_use]
pub fn space_rollup(space: &super::state::Space) -> StatusRollup {
    rollup(
        space
            .tabs
            .iter()
            .flat_map(|tab| tab.panes.iter().map(|pane| pane.status)),
    )
}

fn overall(rollup: &StatusRollup) -> TaskStatus {
    if rollup.blocked > 0 {
        return TaskStatus::Blocked;
    }
    if rollup.working > 0 {
        return TaskStatus::Working;
    }
    if rollup.unknown > 0 {
        return TaskStatus::Unknown;
    }
    if rollup.done > 0 && rollup.idle == 0 {
        return TaskStatus::Done;
    }
    if rollup.idle > 0 {
        return TaskStatus::Idle;
    }
    TaskStatus::Unknown
}

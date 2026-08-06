/// Stable replay diagnostic code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ReplayIssueCode {
    /// Fixture metadata is missing a required value.
    MissingMetadata,
    /// Event kind is unsupported by the replay core or target.
    UnsupportedEvent,
    /// Event target does not match the target's current focus owner.
    StaleFocus,
    /// A command action is unknown to the target.
    UnknownCommand,
    /// A command action is disabled.
    DisabledAction,
    /// Mouse coordinates exceed the target bounds.
    OutOfBoundsMouse,
    /// Keyboard movement cannot move past a boundary.
    OutOfBoundsMovement,
    /// Text event is empty.
    EmptyText,
    /// Text event contains malformed input for the target.
    MalformedText,
    /// Expected assertion does not match final state.
    AssertionMismatch,
    /// Snapshot hash does not match expected evidence.
    SnapshotMismatch,
    /// Wait predicate is unsupported.
    UnsupportedWaitPredicate,
    /// Wait condition referenced a missing target, snapshot, state key, or diagnostic.
    MissingWaitTarget,
    /// Wait condition remained unmet for the supplied evaluation budget.
    WaitConditionUnmet,
    /// Wait condition requested impossible geometry.
    ImpossibleGeometry,
    /// A drag update or release arrived without an accepted drag start.
    MissingDragStart,
    /// A drag update or release referenced a stale captured target.
    StaleCapture,
}

/// Deterministic replay diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayIssue {
    /// Stable issue code.
    pub code: ReplayIssueCode,
    /// Field or logical path associated with the issue.
    pub field: String,
    /// Human-readable diagnostic message.
    pub message: String,
}

impl ReplayIssue {
    pub(crate) fn new(code: ReplayIssueCode, field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code,
            field: field.into(),
            message: message.into(),
        }
    }
}

/// Result of a replay execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayReport {
    /// Final deterministic model state.
    pub state: std::collections::BTreeMap<String, String>,
    /// BLAKE3 hash of the final state snapshot.
    pub state_hash: String,
    /// Snapshot hashes emitted by the run.
    pub snapshots: Vec<super::ReplaySnapshotExpectation>,
    /// Deterministic issues found during validation or execution.
    pub issues: Vec<ReplayIssue>,
}

impl ReplayReport {
    /// Returns true when the replay completed without issues.
    #[must_use]
    pub fn is_success(&self) -> bool {
        self.issues.is_empty()
    }
}

/// Pure adapter implemented by model-specific replay harnesses.
pub trait ReplayTarget {
    /// Returns currently focusable target IDs.
    fn focus_targets(&self) -> std::collections::BTreeSet<String>;
    /// Returns the active focus target ID.
    fn focused_target(&self) -> Option<String>;
    /// Returns viewport or logical input bounds.
    fn bounds(&self) -> (u16, u16);
    /// Returns true when an action is known to the target.
    fn action_known(&self, action_id: &str) -> bool {
        self.action_enabled(action_id)
    }
    /// Returns true when an action is enabled.
    fn action_enabled(&self, action_id: &str) -> bool;
    /// Returns target-specific event diagnostics before mutation.
    fn validate_event(&self, _event: &super::ReplayEvent) -> Vec<ReplayIssue> {
        Vec::new()
    }
    /// Applies a valid event to the model.
    fn apply_event(&mut self, event: &super::ReplayEvent);
    /// Returns deterministic final state key/value pairs.
    fn snapshot_state(&self) -> std::collections::BTreeMap<String, String>;
    /// Returns optional renderer or artifact snapshot bytes by label.
    fn snapshot_bytes(&self, _label: &str) -> Option<Vec<u8>> {
        None
    }
    /// Returns stable diagnostic event codes emitted by the target.
    fn diagnostic_events(&self) -> std::collections::BTreeSet<String> {
        std::collections::BTreeSet::new()
    }
}

//! Shared notification and activity center model.
//!
//! Applications own clocks, persistence, and external event ingestion. This
//! module validates supplied in-memory records, applies explicit lifecycle
//! commands, and projects deterministic toast, inbox, activity tray, status, and
//! action surfaces for renderer adapters.

use std::collections::BTreeSet;

const EMPTY_TEXT: &str = "";
const DEFAULT_GROUP: &str = "default";
#[cfg(test)]
const ZERO_UNREAD: usize = 0;

/// Ordered instant on the caller-owned notification timeline.
///
/// The scalar has no implied epoch or wall-clock unit. Applications must use
/// one consistent timeline for creation, update, expiry, and explicit `now`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NotificationTime(u64);

impl NotificationTime {
    /// Creates a timeline instant from the caller's documented tick scale.
    #[must_use]
    pub const fn from_timeline_tick(value: u64) -> Self {
        Self(value)
    }

    /// Returns the caller-owned timeline tick.
    #[must_use]
    pub const fn timeline_tick(self) -> u64 {
        self.0
    }
}

semantic_string_id! {
    /// Stable notification identifier.
    ///
    /// ```compile_fail
    /// use ratcore::notification_activity_center::{ActivityId, NotificationId};
    /// let activity = ActivityId::new("build");
    /// let _: NotificationId = activity;
    /// ```
    pub struct NotificationId;
}

semantic_string_id! {
    /// Stable activity identifier.
    pub struct ActivityId;
}

semantic_string_id! {
    /// Stable notification-local action identifier.
    ///
    /// ```compile_fail
    /// use ratcore::notification_activity_center::{NotificationActionId, NotificationId};
    /// let action = NotificationActionId::new("retry");
    /// let _: NotificationId = action;
    /// ```
    pub struct NotificationActionId;
}

/// Severity for notification display and sorting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotificationSeverity {
    /// Informational message.
    Info,
    /// Successful completion message.
    Success,
    /// Warning message.
    Warning,
    /// Error message.
    Error,
    /// Caller supplied an unsupported severity.
    Unsupported(String),
}

/// Lifecycle state for a notification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationLifecycle {
    /// Notification is active.
    Active,
    /// Notification was acknowledged.
    Acknowledged,
    /// Notification was dismissed.
    Dismissed,
    /// Notification was snoozed by application policy.
    Snoozed,
}

/// Action attached to a notification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotificationAction {
    /// Stable action identifier.
    pub id: NotificationActionId,
    /// Display label.
    pub label: String,
    /// Whether this action can be invoked.
    pub enabled: bool,
    /// Optional disabled or diagnostic reason.
    pub diagnostic: Option<String>,
}

impl NotificationAction {
    /// Creates an enabled notification action.
    #[must_use]
    pub fn new(id: impl Into<NotificationActionId>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            enabled: true,
            diagnostic: None,
        }
    }

    /// Sets enablement.
    #[must_use]
    pub const fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Sets diagnostic metadata.
    #[must_use]
    pub fn with_diagnostic(mut self, diagnostic: impl Into<String>) -> Self {
        self.diagnostic = Some(diagnostic.into());
        self
    }
}

/// Notification record supplied by the application.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotificationRecord {
    /// Stable identifier.
    pub id: NotificationId,
    /// Required title.
    pub title: String,
    /// Optional body text.
    pub body: Option<String>,
    /// Source label.
    pub source: String,
    /// Severity.
    pub severity: NotificationSeverity,
    /// Lifecycle state.
    pub lifecycle: NotificationLifecycle,
    /// Whether the notification is unread.
    pub unread: bool,
    /// Grouping key.
    pub group_key: Option<String>,
    /// Caller-supplied creation time.
    pub created_at: NotificationTime,
    /// Caller-supplied expiry time on the same timeline.
    pub expires_at: Option<NotificationTime>,
    /// Notification actions.
    pub actions: Vec<NotificationAction>,
}

impl NotificationRecord {
    /// Creates an active unread notification.
    #[must_use]
    pub fn new(
        id: impl Into<NotificationId>,
        title: impl Into<String>,
        severity: NotificationSeverity,
        created_at: NotificationTime,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            body: None,
            source: DEFAULT_GROUP.to_string(),
            severity,
            lifecycle: NotificationLifecycle::Active,
            unread: true,
            group_key: None,
            created_at,
            expires_at: None,
            actions: Vec::new(),
        }
    }

    /// Sets body text.
    #[must_use]
    pub fn with_body(mut self, body: impl Into<String>) -> Self {
        self.body = Some(body.into());
        self
    }

    /// Sets source.
    #[must_use]
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = source.into();
        self
    }

    /// Sets group key.
    #[must_use]
    pub fn with_group(mut self, group_key: impl Into<String>) -> Self {
        self.group_key = Some(group_key.into());
        self
    }

    /// Sets lifecycle.
    #[must_use]
    pub const fn with_lifecycle(mut self, lifecycle: NotificationLifecycle) -> Self {
        self.lifecycle = lifecycle;
        self
    }

    /// Sets unread state.
    #[must_use]
    pub const fn with_unread(mut self, unread: bool) -> Self {
        self.unread = unread;
        self
    }

    /// Sets expiry time on the caller-owned timeline.
    #[must_use]
    pub const fn with_expires_at(mut self, expires_at: NotificationTime) -> Self {
        self.expires_at = Some(expires_at);
        self
    }

    /// Creates a notification from a compatibility raw timeline tick.
    #[must_use]
    pub fn new_at_timeline_tick(
        id: impl Into<NotificationId>,
        title: impl Into<String>,
        severity: NotificationSeverity,
        created_at: u64,
    ) -> Self {
        Self::new(id, title, severity, NotificationTime::from_timeline_tick(created_at))
    }

    /// Sets expiry from a compatibility raw timeline tick.
    #[must_use]
    pub const fn with_expires_at_timeline_tick(self, expires_at: u64) -> Self {
        self.with_expires_at(NotificationTime::from_timeline_tick(expires_at))
    }

    /// Sets actions.
    #[must_use]
    pub fn with_actions(mut self, actions: Vec<NotificationAction>) -> Self {
        self.actions = actions;
        self
    }
}

/// Activity status supplied by the application.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActivityStatus {
    /// Activity is queued.
    Pending,
    /// Activity is running.
    Running,
    /// Activity succeeded.
    Succeeded,
    /// Activity failed.
    Failed,
    /// Caller supplied an unsupported status.
    Unsupported(String),
}

/// Activity row supplied by the application.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivityRecord {
    /// Stable activity identifier.
    pub id: ActivityId,
    /// Required title.
    pub title: String,
    /// Source label.
    pub source: String,
    /// Status.
    pub status: ActivityStatus,
    /// Optional validated progress supplied by caller.
    pub progress: Option<crate::validated_values::Percent>,
    /// Caller-supplied update time.
    pub updated_at: NotificationTime,
}

impl ActivityRecord {
    /// Creates an activity record.
    #[must_use]
    pub fn new(
        id: impl Into<ActivityId>,
        title: impl Into<String>,
        status: ActivityStatus,
        updated_at: NotificationTime,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            source: DEFAULT_GROUP.to_string(),
            status,
            progress: None,
            updated_at,
        }
    }

    /// Sets source label.
    #[must_use]
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = source.into();
        self
    }

    /// Sets validated activity progress.
    #[must_use]
    pub const fn with_progress(mut self, progress: crate::validated_values::Percent) -> Self {
        self.progress = Some(progress);
        self
    }

    /// Creates an activity from a compatibility raw timeline tick.
    #[must_use]
    pub fn new_at_timeline_tick(
        id: impl Into<ActivityId>,
        title: impl Into<String>,
        status: ActivityStatus,
        updated_at: u64,
    ) -> Self {
        Self::new(id, title, status, NotificationTime::from_timeline_tick(updated_at))
    }

    /// Validates and sets a compatibility percentage atomically.
    pub fn try_with_progress_percent(self, progress_percent: u8) -> Result<Self, crate::validated_values::ValueError> {
        crate::validated_values::Percent::try_new(progress_percent).map(|progress| self.with_progress(progress))
    }
}

/// Notification and activity state supplied to the pure core.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NotificationActivityState {
    /// Notifications.
    pub notifications: Vec<NotificationRecord>,
    /// Activities.
    pub activities: Vec<ActivityRecord>,
    /// Expanded group keys.
    pub expanded_groups: Vec<String>,
}

impl NotificationActivityState {
    /// Creates state.
    #[must_use]
    pub fn new(notifications: Vec<NotificationRecord>, activities: Vec<ActivityRecord>) -> Self {
        Self {
            notifications,
            activities,
            expanded_groups: Vec::new(),
        }
    }
}

/// Notification diagnostic code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationDiagnosticCode {
    /// Duplicate notification ID.
    DuplicateNotificationId,
    /// Duplicate activity ID.
    DuplicateActivityId,
    /// Required title is empty.
    EmptyTitle,
    /// Action metadata is malformed.
    InvalidAction,
    /// Entry has expired according to caller-supplied time.
    Expired,
    /// Severity is unsupported.
    UnsupportedSeverity,
    /// Activity status is unsupported.
    UnsupportedActivityStatus,
    /// Command target is absent or stale.
    StaleId,
    /// Command targets a disabled action.
    DisabledAction,
    /// Command kind is unsupported.
    UnsupportedCommand,
}

/// Notification diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotificationDiagnostic {
    /// Machine-readable diagnostic code.
    pub code: NotificationDiagnosticCode,
    /// Notification associated with the diagnostic.
    pub notification_id: Option<NotificationId>,
    /// Activity associated with the diagnostic.
    pub activity_id: Option<ActivityId>,
    /// Action associated with the diagnostic.
    pub action_id: Option<NotificationActionId>,
    /// Human-readable message.
    pub message: String,
}

impl NotificationDiagnostic {
    /// Creates a diagnostic.
    #[must_use]
    pub fn new(
        code: NotificationDiagnosticCode,
        notification_id: Option<NotificationId>,
        activity_id: Option<ActivityId>,
        action_id: Option<NotificationActionId>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            notification_id,
            activity_id,
            action_id,
            message: message.into(),
        }
    }
}

/// Toast surface item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToastItemProjection {
    /// Notification identifier.
    pub notification_id: NotificationId,
    /// Title.
    pub title: String,
    /// Severity.
    pub severity: NotificationSeverity,
    /// Action identifiers visible in the toast.
    pub action_ids: Vec<NotificationActionId>,
}

/// Inbox row projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InboxRowProjection {
    /// Notification identifier.
    pub notification_id: NotificationId,
    /// Title.
    pub title: String,
    /// Lifecycle state.
    pub lifecycle: NotificationLifecycle,
    /// Unread state.
    pub unread: bool,
    /// Group key.
    pub group_key: String,
    /// Diagnostic messages associated with the row.
    pub diagnostics: Vec<NotificationDiagnostic>,
}

/// Activity tray row projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivityTrayRowProjection {
    /// Activity identifier.
    pub activity_id: ActivityId,
    /// Title.
    pub title: String,
    /// Source label.
    pub source: String,
    /// Status.
    pub status: ActivityStatus,
    /// Optional validated progress.
    pub progress: Option<crate::validated_values::Percent>,
}

/// Status badge projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusBadgeProjection {
    /// Unread notification count.
    pub unread_count: usize,
    /// Active activity count.
    pub active_activity_count: usize,
    /// Whether any error severity notification is active.
    pub has_error: bool,
}

/// Notification action target projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotificationActionTargetProjection {
    /// Notification identifier.
    pub notification_id: NotificationId,
    /// Action identifier.
    pub action_id: NotificationActionId,
    /// Label.
    pub label: String,
    /// Whether the action can be invoked.
    pub enabled: bool,
    /// Disabled reason when present.
    pub diagnostic: Option<String>,
}

/// Complete notification surface projections.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotificationSurfaceProjections {
    /// Toast stack items.
    pub toast_stack: Vec<ToastItemProjection>,
    /// Inbox rows.
    pub inbox_rows: Vec<InboxRowProjection>,
    /// Activity tray rows.
    pub activity_tray: Vec<ActivityTrayRowProjection>,
    /// Status badge.
    pub status_badge: StatusBadgeProjection,
    /// Action targets.
    pub action_targets: Vec<NotificationActionTargetProjection>,
    /// Diagnostics collected during validation and projection.
    pub diagnostics: Vec<NotificationDiagnostic>,
}

/// Renderer family consuming notification projections.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationRenderer {
    /// Meteorite/Dioxus renderer.
    Meteorite,
    /// Subwayrat/ratatui renderer.
    Subwayrat,
}

/// Renderer adapter projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotificationRendererProjection {
    /// Renderer consuming the projection.
    pub renderer: NotificationRenderer,
    /// Shared surface projections.
    pub surfaces: NotificationSurfaceProjections,
}

/// Notification lifecycle command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotificationCommand {
    /// Acknowledge a notification.
    Acknowledge(NotificationId),
    /// Dismiss a notification.
    Dismiss(NotificationId),
    /// Clear all dismissed notifications.
    ClearDismissed,
    /// Expand or collapse a group.
    SetGroupExpanded {
        /// Group key.
        group_key: String,
        /// Expanded state.
        expanded: bool,
    },
    /// Invoke a notification action.
    InvokeAction {
        /// Notification identifier.
        notification_id: NotificationId,
        /// Action identifier.
        action_id: NotificationActionId,
    },
    /// Unsupported command supplied by a renderer adapter.
    Unsupported(String),
}

/// Command reducer outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotificationCommandOutcome {
    /// Next state.
    pub state: NotificationActivityState,
    /// Invoked action target when command accepted an action.
    pub invoked_action: Option<NotificationActionTargetProjection>,
    /// Diagnostic when command failed closed.
    pub diagnostic: Option<NotificationDiagnostic>,
}

/// Builds deterministic notification surface projections from supplied state.
#[must_use]
pub fn project_notification_surfaces(
    state: &NotificationActivityState,
    now: NotificationTime,
) -> NotificationSurfaceProjections {
    let mut diagnostics = validate_state(state, now);
    let toast_stack = state
        .notifications
        .iter()
        .filter(|notification| is_toast_visible(notification, now))
        .map(toast_projection)
        .collect();
    let inbox_rows = state
        .notifications
        .iter()
        .map(|notification| inbox_projection(notification, &diagnostics))
        .collect();
    let activity_tray = state.activities.iter().map(activity_projection).collect();
    let status_badge = status_badge_projection(state, now);
    let action_targets = state.notifications.iter().flat_map(action_target_projections).collect();
    diagnostics.sort_by(|left, right| left.message.cmp(&right.message));
    NotificationSurfaceProjections {
        toast_stack,
        inbox_rows,
        activity_tray,
        status_badge,
        action_targets,
        diagnostics,
    }
}

/// Applies a lifecycle command without reading clocks or I/O.
#[must_use]
pub fn apply_notification_command(
    state: &NotificationActivityState,
    command: NotificationCommand,
) -> NotificationCommandOutcome {
    match command {
        NotificationCommand::Acknowledge(id) => {
            update_notification_lifecycle(state, &id, NotificationLifecycle::Acknowledged, false)
        }
        NotificationCommand::Dismiss(id) => {
            update_notification_lifecycle(state, &id, NotificationLifecycle::Dismissed, false)
        }
        NotificationCommand::ClearDismissed => clear_dismissed(state),
        NotificationCommand::SetGroupExpanded { group_key, expanded } => {
            set_group_expanded(state, &group_key, expanded)
        }
        NotificationCommand::InvokeAction {
            notification_id,
            action_id,
        } => invoke_action(state, &notification_id, &action_id),
        NotificationCommand::Unsupported(command) => NotificationCommandOutcome {
            state: state.clone(),
            invoked_action: None,
            diagnostic: Some(NotificationDiagnostic::new(
                NotificationDiagnosticCode::UnsupportedCommand,
                None,
                None,
                None,
                format!("unsupported notification command: {command}"),
            )),
        },
    }
}

/// Wraps shared notification surfaces for a renderer.
#[must_use]
pub fn project_notification_renderer(
    renderer: NotificationRenderer,
    state: &NotificationActivityState,
    now: NotificationTime,
) -> NotificationRendererProjection {
    NotificationRendererProjection {
        renderer,
        surfaces: project_notification_surfaces(state, now),
    }
}

fn validate_state(state: &NotificationActivityState, now: NotificationTime) -> Vec<NotificationDiagnostic> {
    let mut diagnostics = Vec::new();
    diagnostics.extend(duplicate_notification_diagnostics(&state.notifications));
    diagnostics.extend(duplicate_activity_diagnostics(&state.activities));
    for notification in &state.notifications {
        validate_notification(notification, now, &mut diagnostics);
    }
    for activity in &state.activities {
        validate_activity(activity, &mut diagnostics);
    }
    diagnostics
}

fn duplicate_notification_diagnostics(notifications: &[NotificationRecord]) -> Vec<NotificationDiagnostic> {
    let mut seen = BTreeSet::new();
    let mut diagnostics = Vec::new();
    for notification in notifications {
        if !seen.insert(notification.id.clone()) {
            diagnostics.push(NotificationDiagnostic::new(
                NotificationDiagnosticCode::DuplicateNotificationId,
                Some(notification.id.clone()),
                None,
                None,
                "duplicate notification id",
            ));
        }
    }
    diagnostics
}

fn duplicate_activity_diagnostics(activities: &[ActivityRecord]) -> Vec<NotificationDiagnostic> {
    let mut seen = BTreeSet::new();
    let mut diagnostics = Vec::new();
    for activity in activities {
        if !seen.insert(activity.id.clone()) {
            diagnostics.push(NotificationDiagnostic::new(
                NotificationDiagnosticCode::DuplicateActivityId,
                None,
                Some(activity.id.clone()),
                None,
                "duplicate activity id",
            ));
        }
    }
    diagnostics
}

fn validate_notification(
    notification: &NotificationRecord,
    now: NotificationTime,
    diagnostics: &mut Vec<NotificationDiagnostic>,
) {
    if notification.title.trim() == EMPTY_TEXT {
        diagnostics.push(NotificationDiagnostic::new(
            NotificationDiagnosticCode::EmptyTitle,
            Some(notification.id.clone()),
            None,
            None,
            "notification title is empty",
        ));
    }
    if matches!(notification.severity, NotificationSeverity::Unsupported(_)) {
        diagnostics.push(NotificationDiagnostic::new(
            NotificationDiagnosticCode::UnsupportedSeverity,
            Some(notification.id.clone()),
            None,
            None,
            "notification severity is unsupported",
        ));
    }
    if is_expired(notification, now) {
        diagnostics.push(NotificationDiagnostic::new(
            NotificationDiagnosticCode::Expired,
            Some(notification.id.clone()),
            None,
            None,
            "notification is expired",
        ));
    }
    for action in &notification.actions {
        if action.id.as_str().trim() == EMPTY_TEXT || action.label.trim() == EMPTY_TEXT {
            diagnostics.push(NotificationDiagnostic::new(
                NotificationDiagnosticCode::InvalidAction,
                Some(notification.id.clone()),
                None,
                Some(action.id.clone()),
                "notification action is malformed",
            ));
        }
    }
}

fn validate_activity(activity: &ActivityRecord, diagnostics: &mut Vec<NotificationDiagnostic>) {
    if activity.title.trim() == EMPTY_TEXT {
        diagnostics.push(NotificationDiagnostic::new(
            NotificationDiagnosticCode::EmptyTitle,
            None,
            Some(activity.id.clone()),
            None,
            "activity title is empty",
        ));
    }
    if matches!(activity.status, ActivityStatus::Unsupported(_)) {
        diagnostics.push(NotificationDiagnostic::new(
            NotificationDiagnosticCode::UnsupportedActivityStatus,
            None,
            Some(activity.id.clone()),
            None,
            "activity status is unsupported",
        ));
    }
}

fn is_expired(notification: &NotificationRecord, now: NotificationTime) -> bool {
    notification.expires_at.is_some_and(|expires_at| expires_at <= now)
}

fn is_toast_visible(notification: &NotificationRecord, now: NotificationTime) -> bool {
    notification.lifecycle == NotificationLifecycle::Active && !is_expired(notification, now)
}

fn toast_projection(notification: &NotificationRecord) -> ToastItemProjection {
    ToastItemProjection {
        notification_id: notification.id.clone(),
        title: notification.title.clone(),
        severity: notification.severity.clone(),
        action_ids: notification
            .actions
            .iter()
            .filter(|action| action.enabled)
            .map(|action| action.id.clone())
            .collect(),
    }
}

fn inbox_projection(notification: &NotificationRecord, diagnostics: &[NotificationDiagnostic]) -> InboxRowProjection {
    InboxRowProjection {
        notification_id: notification.id.clone(),
        title: notification.title.clone(),
        lifecycle: notification.lifecycle,
        unread: notification.unread,
        group_key: notification.group_key.clone().unwrap_or_else(|| DEFAULT_GROUP.to_string()),
        diagnostics: diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.notification_id.as_ref() == Some(&notification.id))
            .cloned()
            .collect(),
    }
}

fn activity_projection(activity: &ActivityRecord) -> ActivityTrayRowProjection {
    ActivityTrayRowProjection {
        activity_id: activity.id.clone(),
        title: activity.title.clone(),
        source: activity.source.clone(),
        status: activity.status.clone(),
        progress: activity.progress,
    }
}

fn status_badge_projection(state: &NotificationActivityState, now: NotificationTime) -> StatusBadgeProjection {
    let unread_count = state
        .notifications
        .iter()
        .filter(|notification| notification.unread && !is_expired(notification, now))
        .count();
    let active_activity_count = state
        .activities
        .iter()
        .filter(|activity| matches!(activity.status, ActivityStatus::Pending | ActivityStatus::Running))
        .count();
    let has_error = state.notifications.iter().any(|notification| {
        notification.lifecycle == NotificationLifecycle::Active
            && matches!(notification.severity, NotificationSeverity::Error)
            && !is_expired(notification, now)
    });
    StatusBadgeProjection {
        unread_count,
        active_activity_count,
        has_error,
    }
}

fn action_target_projections(notification: &NotificationRecord) -> Vec<NotificationActionTargetProjection> {
    notification
        .actions
        .iter()
        .map(|action| NotificationActionTargetProjection {
            notification_id: notification.id.clone(),
            action_id: action.id.clone(),
            label: action.label.clone(),
            enabled: action.enabled,
            diagnostic: action.diagnostic.clone(),
        })
        .collect()
}

fn update_notification_lifecycle(
    state: &NotificationActivityState,
    id: &NotificationId,
    lifecycle: NotificationLifecycle,
    unread: bool,
) -> NotificationCommandOutcome {
    let mut next = state.clone();
    let Some(notification) = next.notifications.iter_mut().find(|record| &record.id == id) else {
        return stale_command(state, id);
    };
    if notification.lifecycle == NotificationLifecycle::Dismissed {
        return NotificationCommandOutcome {
            state: state.clone(),
            invoked_action: None,
            diagnostic: Some(NotificationDiagnostic::new(
                NotificationDiagnosticCode::StaleId,
                Some(id.clone()),
                None,
                None,
                "notification is already dismissed",
            )),
        };
    }
    notification.lifecycle = lifecycle;
    notification.unread = unread;
    NotificationCommandOutcome {
        state: next,
        invoked_action: None,
        diagnostic: None,
    }
}

fn clear_dismissed(state: &NotificationActivityState) -> NotificationCommandOutcome {
    let mut next = state.clone();
    next.notifications.retain(|notification| notification.lifecycle != NotificationLifecycle::Dismissed);
    NotificationCommandOutcome {
        state: next,
        invoked_action: None,
        diagnostic: None,
    }
}

fn set_group_expanded(
    state: &NotificationActivityState,
    group_key: &str,
    expanded: bool,
) -> NotificationCommandOutcome {
    let mut next = state.clone();
    let mut groups: BTreeSet<String> = next.expanded_groups.into_iter().collect();
    if expanded {
        groups.insert(group_key.to_string());
    } else {
        groups.remove(group_key);
    }
    next.expanded_groups = groups.into_iter().collect();
    NotificationCommandOutcome {
        state: next,
        invoked_action: None,
        diagnostic: None,
    }
}

fn invoke_action(
    state: &NotificationActivityState,
    notification_id: &NotificationId,
    action_id: &NotificationActionId,
) -> NotificationCommandOutcome {
    let Some(notification) = state.notifications.iter().find(|notification| &notification.id == notification_id) else {
        return stale_command(state, notification_id);
    };
    if notification.lifecycle == NotificationLifecycle::Dismissed {
        return stale_command(state, notification_id);
    }
    let Some(action) = notification.actions.iter().find(|action| &action.id == action_id) else {
        return NotificationCommandOutcome {
            state: state.clone(),
            invoked_action: None,
            diagnostic: Some(NotificationDiagnostic::new(
                NotificationDiagnosticCode::StaleId,
                Some(notification_id.clone()),
                None,
                Some(action_id.clone()),
                "notification action is absent",
            )),
        };
    };
    if !action.enabled {
        return NotificationCommandOutcome {
            state: state.clone(),
            invoked_action: None,
            diagnostic: Some(NotificationDiagnostic::new(
                NotificationDiagnosticCode::DisabledAction,
                Some(notification_id.clone()),
                None,
                Some(action_id.clone()),
                action.diagnostic.clone().unwrap_or_else(|| "notification action is disabled".to_string()),
            )),
        };
    }
    NotificationCommandOutcome {
        state: state.clone(),
        invoked_action: Some(NotificationActionTargetProjection {
            notification_id: notification_id.clone(),
            action_id: action_id.clone(),
            label: action.label.clone(),
            enabled: action.enabled,
            diagnostic: action.diagnostic.clone(),
        }),
        diagnostic: None,
    }
}

fn stale_command(state: &NotificationActivityState, id: &NotificationId) -> NotificationCommandOutcome {
    NotificationCommandOutcome {
        state: state.clone(),
        invoked_action: None,
        diagnostic: Some(NotificationDiagnostic::new(
            NotificationDiagnosticCode::StaleId,
            Some(id.clone()),
            None,
            None,
            "notification command target is stale",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CREATED_AT: NotificationTime = NotificationTime::from_timeline_tick(10);
    const NOW: NotificationTime = NotificationTime::from_timeline_tick(20);
    const EXPIRED_AT: NotificationTime = NotificationTime::from_timeline_tick(15);
    const PROGRESS_HALF: crate::validated_values::Percent = match crate::validated_values::Percent::try_new(50) {
        Ok(progress) => progress,
        Err(_) => panic!("test percentage is valid"),
    };

    fn valid_notification() -> NotificationRecord {
        NotificationRecord::new("build.failed", "Build failed", NotificationSeverity::Error, CREATED_AT)
            .with_body("cargo test failed")
            .with_group("build")
            .with_actions(vec![NotificationAction::new("open.log", "Open log")])
    }

    fn valid_state() -> NotificationActivityState {
        NotificationActivityState::new(vec![valid_notification()], vec![
            ActivityRecord::new("build", "Workspace check", ActivityStatus::Running, CREATED_AT)
                .with_progress(PROGRESS_HALF),
        ])
    }

    #[test]
    fn projects_toast_inbox_activity_status_and_actions() {
        let surfaces = project_notification_surfaces(&valid_state(), NOW);

        assert_eq!(surfaces.toast_stack[0].notification_id, "build.failed");
        assert_eq!(surfaces.inbox_rows[0].group_key, "build");
        assert_eq!(surfaces.activity_tray[0].progress, Some(PROGRESS_HALF));
        assert_eq!(surfaces.status_badge.unread_count, 1);
        assert!(surfaces.status_badge.has_error);
        assert_eq!(surfaces.action_targets[0].action_id, "open.log");
        assert!(surfaces.diagnostics.is_empty());
    }

    #[test]
    fn malformed_duplicate_expired_and_unsupported_records_remain_diagnostic() {
        let malformed =
            NotificationRecord::new("bad", "", NotificationSeverity::Unsupported("loud".to_string()), CREATED_AT)
                .with_expires_at(EXPIRED_AT)
                .with_actions(vec![NotificationAction::new("", "")]);
        let duplicate = NotificationRecord::new("bad", "Duplicate", NotificationSeverity::Info, CREATED_AT);
        let activity =
            ActivityRecord::new("activity", "", ActivityStatus::Unsupported("paused".to_string()), CREATED_AT);
        let state = NotificationActivityState::new(vec![malformed, duplicate], vec![activity]);

        let surfaces = project_notification_surfaces(&state, NOW);

        assert!(
            surfaces
                .diagnostics
                .iter()
                .any(|diagnostic| { diagnostic.code == NotificationDiagnosticCode::DuplicateNotificationId })
        );
        assert!(
            surfaces
                .diagnostics
                .iter()
                .any(|diagnostic| { diagnostic.code == NotificationDiagnosticCode::EmptyTitle })
        );
        assert!(
            surfaces
                .diagnostics
                .iter()
                .any(|diagnostic| { diagnostic.code == NotificationDiagnosticCode::InvalidAction })
        );
        assert!(
            surfaces
                .diagnostics
                .iter()
                .any(|diagnostic| { diagnostic.code == NotificationDiagnosticCode::Expired })
        );
        assert!(
            surfaces
                .diagnostics
                .iter()
                .any(|diagnostic| { diagnostic.code == NotificationDiagnosticCode::UnsupportedSeverity })
        );
        assert!(
            surfaces
                .diagnostics
                .iter()
                .any(|diagnostic| { diagnostic.code == NotificationDiagnosticCode::UnsupportedActivityStatus })
        );
        assert_eq!(surfaces.status_badge.unread_count, ZERO_UNREAD + 1);
    }

    #[test]
    fn lifecycle_commands_update_state_purely() {
        let state = valid_state();
        let acknowledged =
            apply_notification_command(&state, NotificationCommand::Acknowledge(NotificationId::new("build.failed")));
        let dismissed = apply_notification_command(
            &acknowledged.state,
            NotificationCommand::Dismiss(NotificationId::new("build.failed")),
        );
        let cleared = apply_notification_command(&dismissed.state, NotificationCommand::ClearDismissed);

        assert_eq!(acknowledged.state.notifications[0].lifecycle, NotificationLifecycle::Acknowledged);
        assert!(!acknowledged.state.notifications[0].unread);
        assert_eq!(dismissed.state.notifications[0].lifecycle, NotificationLifecycle::Dismissed);
        assert!(cleared.state.notifications.is_empty());
    }

    #[test]
    fn stale_disabled_and_unsupported_commands_fail_closed() {
        let state = NotificationActivityState::new(
            vec![
                valid_notification().with_actions(vec![NotificationAction::new("retry", "Retry").with_enabled(false)]),
            ],
            Vec::new(),
        );

        let stale = apply_notification_command(&state, NotificationCommand::Dismiss(NotificationId::new("missing")));
        let disabled = apply_notification_command(&state, NotificationCommand::InvokeAction {
            notification_id: NotificationId::new("build.failed"),
            action_id: NotificationActionId::new("retry"),
        });
        let unsupported = apply_notification_command(&state, NotificationCommand::Unsupported("teleport".to_string()));

        assert!(matches!(
            stale.diagnostic,
            Some(NotificationDiagnostic {
                code: NotificationDiagnosticCode::StaleId,
                ..
            })
        ));
        assert!(matches!(
            disabled.diagnostic,
            Some(NotificationDiagnostic {
                code: NotificationDiagnosticCode::DisabledAction,
                ..
            })
        ));
        assert!(matches!(
            unsupported.diagnostic,
            Some(NotificationDiagnostic {
                code: NotificationDiagnosticCode::UnsupportedCommand,
                ..
            })
        ));
        assert_eq!(state, stale.state);
        assert_eq!(state, disabled.state);
    }

    #[test]
    fn timeline_and_progress_boundaries_are_explicit() {
        assert_eq!(CREATED_AT.timeline_tick(), 10);
        assert!(
            ActivityRecord::new_at_timeline_tick("invalid-progress", "Invalid", ActivityStatus::Running, 10,)
                .try_with_progress_percent(101)
                .is_err()
        );
    }

    #[test]
    fn renderer_adapters_preserve_shared_notification_semantics() {
        let state = valid_state();

        let web = project_notification_renderer(NotificationRenderer::Meteorite, &state, NOW);
        let tui = project_notification_renderer(NotificationRenderer::Subwayrat, &state, NOW);

        assert_eq!(web.surfaces, tui.surfaces);
    }
}

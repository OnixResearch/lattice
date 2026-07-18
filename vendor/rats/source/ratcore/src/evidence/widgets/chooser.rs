use super::super::*;

const TARGET_COMMAND_PALETTE: &str = "command-palette";
const KEY_ARROW_DOWN: &str = "ArrowDown";
const KEY_ARROW_UP: &str = "ArrowUp";
const KEY_ENTER: &str = "Enter";
const KEY_ESCAPE: &str = "Escape";
const FIELD_COMMAND_KEY: &str = "command-palette.key";
const FIELD_COMMAND_TEXT: &str = "command-palette.text";
const FIELD_MOVEMENT: &str = "movement";
const FIELD_ENTER: &str = "enter";
const EMPTY_MARKER: &str = "-";
const ID_SEPARATOR: &str = ",";
const DEFAULT_REPLAY_WIDTH: u16 = 80;
const DEFAULT_REPLAY_HEIGHT: u16 = 24;

/// Command palette action used by replay fixtures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayCommandAction {
    /// Stable action ID.
    pub id: String,
    /// Human-readable action label.
    pub label: String,
    /// Whether the action can be selected.
    pub enabled: bool,
}

impl ReplayCommandAction {
    /// Creates an enabled command palette action.
    #[must_use]
    pub fn enabled(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            enabled: true,
        }
    }

    /// Creates a disabled command palette action.
    #[must_use]
    pub fn disabled(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            enabled: false,
        }
    }
}

/// Pure replay target for command palette behavior.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandPaletteReplayTarget {
    actions: Vec<ReplayCommandAction>,
    query: String,
    active_index: usize,
    selected_action: Option<String>,
    cancelled: bool,
    width: u16,
    height: u16,
}

impl CommandPaletteReplayTarget {
    /// Creates a command palette replay target.
    #[must_use]
    pub fn new(actions: Vec<ReplayCommandAction>) -> Self {
        Self {
            actions,
            query: String::new(),
            active_index: 0,
            selected_action: None,
            cancelled: false,
            width: DEFAULT_REPLAY_WIDTH,
            height: DEFAULT_REPLAY_HEIGHT,
        }
    }

    /// Returns actions matching the current query.
    #[must_use]
    pub fn filtered_actions(&self) -> Vec<&ReplayCommandAction> {
        let normalized_query = self.query.to_ascii_lowercase();
        self.actions
            .iter()
            .filter(|action| action_matches(action, &normalized_query))
            .collect()
    }

    /// Returns the active action ID when a row is available.
    #[must_use]
    pub fn active_action_id(&self) -> Option<String> {
        self.filtered_actions()
            .get(self.active_index)
            .map(|action| action.id.clone())
    }
}

impl ReplayTarget for CommandPaletteReplayTarget {
    fn focus_targets(&self) -> std::collections::BTreeSet<String> {
        [TARGET_COMMAND_PALETTE.to_string()].into_iter().collect()
    }

    fn focused_target(&self) -> Option<String> {
        Some(TARGET_COMMAND_PALETTE.to_string())
    }

    fn bounds(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    fn action_known(&self, action_id: &str) -> bool {
        self.actions.iter().any(|action| action.id == action_id)
    }

    fn action_enabled(&self, action_id: &str) -> bool {
        self.actions
            .iter()
            .any(|action| action.id == action_id && action.enabled)
    }

    fn validate_event(&self, event: &ReplayEvent) -> Vec<ReplayIssue> {
        match event {
            ReplayEvent::Key { key, .. } => self.validate_key(key),
            ReplayEvent::Text { value, .. } if value.chars().any(char::is_control) => {
                vec![ReplayIssue::new(
                    ReplayIssueCode::MalformedText,
                    FIELD_COMMAND_TEXT,
                    "command palette text input must not contain control characters",
                )]
            }
            _ => Vec::new(),
        }
    }

    fn apply_event(&mut self, event: &ReplayEvent) {
        match event {
            ReplayEvent::Text { value, .. } => {
                self.query.push_str(value);
                self.clamp_active_index();
            }
            ReplayEvent::Key { key, .. } if key == KEY_ARROW_DOWN => {
                self.active_index = self.active_index.saturating_add(1);
            }
            ReplayEvent::Key { key, .. } if key == KEY_ARROW_UP => {
                self.active_index = self.active_index.saturating_sub(1);
            }
            ReplayEvent::Key { key, .. } if key == KEY_ENTER => {
                self.selected_action = self.active_enabled_action_id();
            }
            ReplayEvent::Key { key, .. } if key == KEY_ESCAPE => {
                self.cancelled = true;
            }
            ReplayEvent::Command { action_id } if self.action_enabled(action_id) => {
                self.selected_action = Some(action_id.clone());
            }
            ReplayEvent::Resize { width, height } => {
                self.width = *width;
                self.height = *height;
            }
            _ => {}
        }
    }

    fn snapshot_state(&self) -> std::collections::BTreeMap<String, String> {
        std::collections::BTreeMap::from([
            (
                "active_action".to_string(),
                option_id(self.active_action_id()),
            ),
            ("cancelled".to_string(), self.cancelled.to_string()),
            (
                "filtered_actions".to_string(),
                join_ids(self.filtered_actions().iter().map(|action| &action.id)),
            ),
            ("query".to_string(), self.query.clone()),
            (
                "selected_action".to_string(),
                option_id(self.selected_action.clone()),
            ),
            ("surface".to_string(), TARGET_COMMAND_PALETTE.to_string()),
        ])
    }
}

impl CommandPaletteReplayTarget {
    fn validate_key(&self, key: &str) -> Vec<ReplayIssue> {
        match key {
            KEY_ARROW_DOWN
                if self.active_index.saturating_add(1) >= self.filtered_actions().len() =>
            {
                vec![ReplayIssue::new(
                    ReplayIssueCode::OutOfBoundsMovement,
                    FIELD_MOVEMENT,
                    "command palette replay cannot move below the final filtered action",
                )]
            }
            KEY_ARROW_UP if self.active_index == 0 => vec![ReplayIssue::new(
                ReplayIssueCode::OutOfBoundsMovement,
                FIELD_MOVEMENT,
                "command palette replay cannot move above the first filtered action",
            )],
            KEY_ENTER if self.active_disabled_action_id().is_some() => vec![ReplayIssue::new(
                ReplayIssueCode::DisabledAction,
                FIELD_ENTER,
                "command palette active action is disabled",
            )],
            KEY_ARROW_DOWN | KEY_ARROW_UP | KEY_ENTER | KEY_ESCAPE => Vec::new(),
            _ => vec![ReplayIssue::new(
                ReplayIssueCode::UnsupportedEvent,
                FIELD_COMMAND_KEY,
                format!("command palette replay does not support key `{key}`"),
            )],
        }
    }

    fn active_enabled_action_id(&self) -> Option<String> {
        self.filtered_actions()
            .get(self.active_index)
            .filter(|action| action.enabled)
            .map(|action| action.id.clone())
    }

    fn active_disabled_action_id(&self) -> Option<String> {
        self.filtered_actions()
            .get(self.active_index)
            .filter(|action| !action.enabled)
            .map(|action| action.id.clone())
    }

    fn clamp_active_index(&mut self) {
        let filtered_len = self.filtered_actions().len();
        if filtered_len == 0 {
            self.active_index = 0;
        } else if self.active_index >= filtered_len {
            self.active_index = filtered_len.saturating_sub(1);
        }
    }
}

/// Returns the committed command palette replay fixture.
#[must_use]
pub fn command_palette_replay_script() -> ReplayScript {
    ReplayScript::new(ReplayMetadata::new(
        "command-palette.selection.baseline",
        "Command palette selection baseline",
        "command-palette",
    ))
    .with_event(ReplayEvent::Text {
        target: TARGET_COMMAND_PALETTE.to_string(),
        value: "open".to_string(),
    })
    .with_event(ReplayEvent::Key {
        target: TARGET_COMMAND_PALETTE.to_string(),
        key: KEY_ENTER.to_string(),
    })
    .with_assertion(ReplayAssertion::new("query", "open"))
    .with_assertion(ReplayAssertion::new("active_action", "action.open"))
    .with_assertion(ReplayAssertion::new("selected_action", "action.open"))
    .with_snapshot(ReplaySnapshotExpectation::new(
        "state",
        state_snapshot_hash(&accepted_command_palette_state()),
    ))
}

fn action_matches(action: &ReplayCommandAction, normalized_query: &str) -> bool {
    normalized_query.is_empty()
        || action.id.to_ascii_lowercase().contains(normalized_query)
        || action.label.to_ascii_lowercase().contains(normalized_query)
}

fn option_id(value: Option<String>) -> String {
    value.unwrap_or_else(|| EMPTY_MARKER.to_string())
}

fn join_ids<'a>(ids: impl Iterator<Item = &'a String>) -> String {
    let joined = ids.cloned().collect::<Vec<_>>().join(ID_SEPARATOR);
    if joined.is_empty() {
        EMPTY_MARKER.to_string()
    } else {
        joined
    }
}

fn accepted_command_palette_state() -> std::collections::BTreeMap<String, String> {
    std::collections::BTreeMap::from([
        ("active_action".to_string(), "action.open".to_string()),
        ("cancelled".to_string(), false.to_string()),
        ("filtered_actions".to_string(), "action.open".to_string()),
        ("query".to_string(), "open".to_string()),
        ("selected_action".to_string(), "action.open".to_string()),
        ("surface".to_string(), TARGET_COMMAND_PALETTE.to_string()),
    ])
}

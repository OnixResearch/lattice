use super::super::*;

mod fixtures;
pub use fixtures::LeaderReplayRow;
pub use fixtures::LeaderReplaySummary;
pub use fixtures::leader_key_negative_replay_script;
pub use fixtures::leader_key_positive_replay_script;
pub use fixtures::leader_key_replay_summary;

const TARGET_LEADER_KEY: &str = "leader-key";
const TARGET_STALE_LEADER_KEY: &str = "leader-key.stale";
const ACTION_WORKSPACE_SAVE: &str = "workspace.save";
const ACTION_DISABLED_DOCS: &str = "workspace.docs.disabled";
const SUBMENU_WORKSPACE: &str = "Workspace";
const MENU_ROOT: &str = "Leader";
const KEY_WORKSPACE: &str = "w";
const KEY_SAVE: &str = "s";
const KEY_DISABLED: &str = "d";
const KEY_UNKNOWN: &str = "?";
const KEY_ESCAPE: &str = "Escape";
const FIELD_LEADER_KEY: &str = "leader-key.key";
const FIELD_LEADER_DISABLED: &str = "leader-key.disabled";
const EMPTY_MARKER: &str = "-";
const ID_SEPARATOR: &str = ",";
const DEFAULT_REPLAY_WIDTH: u16 = 80;
const DEFAULT_REPLAY_HEIGHT: u16 = 24;
const WAIT_BUDGET: usize = 1;
const POSITIVE_FIXTURE_ID: &str = "leader-key.workspace-action.baseline";
const NEGATIVE_FIXTURE_ID: &str = "leader-key.invalid.baseline";

/// Pure replay target for leader-key menu behavior.
pub struct LeaderReplayTarget {
    menu: crate::leaderkey::LeaderMenu<String>,
    known_actions: std::collections::BTreeSet<String>,
    disabled_actions: std::collections::BTreeSet<String>,
    selected_action: Option<String>,
    width: u16,
    height: u16,
}

impl LeaderReplayTarget {
    /// Creates a leader-key replay target.
    #[must_use]
    pub fn new() -> Self {
        let mut menu = fixtures::baseline();
        menu.open();
        Self {
            menu,
            known_actions: [ACTION_WORKSPACE_SAVE.to_string(), ACTION_DISABLED_DOCS.to_string()].into_iter().collect(),
            disabled_actions: [ACTION_DISABLED_DOCS.to_string()].into_iter().collect(),
            selected_action: None,
            width: DEFAULT_REPLAY_WIDTH,
            height: DEFAULT_REPLAY_HEIGHT,
        }
    }
}

impl Default for LeaderReplayTarget {
    fn default() -> Self {
        Self::new()
    }
}

impl ReplayTarget for LeaderReplayTarget {
    fn focus_targets(&self) -> std::collections::BTreeSet<String> {
        if self.menu.visible {
            [TARGET_LEADER_KEY.to_string()].into_iter().collect()
        } else {
            std::collections::BTreeSet::new()
        }
    }

    fn focused_target(&self) -> Option<String> {
        self.menu.visible.then(|| TARGET_LEADER_KEY.to_string())
    }

    fn bounds(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    fn action_known(&self, action_id: &str) -> bool {
        self.known_actions.contains(action_id)
    }

    fn action_enabled(&self, action_id: &str) -> bool {
        self.known_actions.contains(action_id) && !self.disabled_actions.contains(action_id)
    }

    fn validate_event(&self, event: &ReplayEvent) -> Vec<ReplayIssue> {
        match event {
            ReplayEvent::Key { key, .. } => self.validate_key(key),
            ReplayEvent::Text { .. } | ReplayEvent::Mouse { .. } => vec![ReplayIssue::new(
                ReplayIssueCode::UnsupportedEvent,
                FIELD_LEADER_KEY,
                "leader-key replay only accepts logical key events",
            )],
            _ => Vec::new(),
        }
    }

    fn apply_event(&mut self, event: &ReplayEvent) {
        match event {
            ReplayEvent::Key { key, .. } => self.apply_key(key),
            ReplayEvent::Resize { width, height } => {
                self.width = *width;
                self.height = *height;
            }
            _ => {}
        }
    }

    fn snapshot_state(&self) -> std::collections::BTreeMap<String, String> {
        std::collections::BTreeMap::from([
            ("breadcrumb".to_string(), join_ids(self.menu.breadcrumb())),
            ("current_items".to_string(), self.current_items()),
            ("current_menu".to_string(), self.current_menu()),
            ("depth".to_string(), self.menu.depth().to_string()),
            ("selected_action".to_string(), option_id(self.selected_action.clone())),
            ("surface".to_string(), TARGET_LEADER_KEY.to_string()),
            ("visible".to_string(), self.menu.visible.to_string()),
        ])
    }
}

impl LeaderReplayTarget {
    fn validate_key(&self, key: &str) -> Vec<ReplayIssue> {
        let Some(input) = logical_input(key) else {
            return vec![unsupported_key(key)];
        };
        match input {
            crate::leaderkey::MenuInput::Char(ch) => self.validate_char(ch),
            crate::leaderkey::MenuInput::Escape => Vec::new(),
            crate::leaderkey::MenuInput::Other => vec![unsupported_key(key)],
        }
    }

    fn validate_char(&self, ch: char) -> Vec<ReplayIssue> {
        let Some(item) = self.current_item(ch) else {
            return vec![unsupported_key(&ch.to_string())];
        };
        if let crate::leaderkey::LeaderAction::Action(action_id) = &item.action {
            if self.disabled_actions.contains(action_id) {
                return vec![ReplayIssue::new(
                    ReplayIssueCode::DisabledAction,
                    FIELD_LEADER_DISABLED,
                    format!("leader-key action `{action_id}` is disabled"),
                )];
            }
        }
        Vec::new()
    }

    fn apply_key(&mut self, key: &str) {
        let Some(input) = logical_input(key) else {
            return;
        };
        if let Some(action) = self.menu.handle_input(input) {
            self.selected_action = Some(action_label(&action));
        }
    }

    fn current_item(&self, ch: char) -> Option<&crate::leaderkey::LeaderMenuItem<String>> {
        self.menu.current().and_then(|menu| menu.items.iter().find(|item| item.key == ch))
    }

    fn current_menu(&self) -> String {
        self.menu.current().map_or_else(|| EMPTY_MARKER.to_string(), |menu| menu.label.clone())
    }

    fn current_items(&self) -> String {
        self.menu.current().map_or_else(
            || EMPTY_MARKER.to_string(),
            |menu| join_ids(menu.items.iter().map(|item| format!("{}:{}", item.key, item.label))),
        )
    }
}

/// Creates the baseline leader-key replay target.
#[must_use]
pub fn leader_key_replay_target() -> LeaderReplayTarget {
    LeaderReplayTarget::new()
}

fn logical_input(key: &str) -> Option<crate::leaderkey::MenuInput> {
    if key == KEY_ESCAPE {
        return Some(crate::leaderkey::MenuInput::Escape);
    }
    let mut chars = key.chars();
    let ch = chars.next()?;
    if chars.next().is_none() {
        Some(crate::leaderkey::MenuInput::Char(ch))
    } else {
        Some(crate::leaderkey::MenuInput::Other)
    }
}

fn action_label(action: &crate::leaderkey::LeaderAction<String>) -> String {
    match action {
        crate::leaderkey::LeaderAction::Action(action_id) => action_id.clone(),
        crate::leaderkey::LeaderAction::Command(command) => format!("command:{command}"),
        crate::leaderkey::LeaderAction::Submenu(menu) => format!("submenu:{menu}"),
    }
}

fn unsupported_key(key: &str) -> ReplayIssue {
    ReplayIssue::new(
        ReplayIssueCode::UnsupportedEvent,
        FIELD_LEADER_KEY,
        format!("leader-key replay does not support key `{key}`"),
    )
}

fn option_id(value: Option<String>) -> String {
    value.unwrap_or_else(|| EMPTY_MARKER.to_string())
}

fn join_ids(ids: impl IntoIterator<Item = impl AsRef<str>>) -> String {
    let values = ids.into_iter().map(|id| id.as_ref().to_string()).collect::<Vec<_>>();
    if values.is_empty() {
        EMPTY_MARKER.to_string()
    } else {
        values.join(ID_SEPARATOR)
    }
}

use super::super::ReplayAssertion;
use super::super::ReplayEvent;
use super::super::ReplayIssue;
use super::super::ReplayIssueCode;
use super::super::ReplayMetadata;
use super::super::ReplayScript;
use super::super::ReplaySnapshotExpectation;
use super::super::ReplayTarget;
use super::super::state_snapshot_hash;
use crate::action_registry::ActionId;
use crate::command_palette::CommandPaletteCandidate;
use crate::command_palette::CommandPaletteDiagnostic;
use crate::command_palette::CommandPaletteDiagnosticCode;
use crate::command_palette::CommandPaletteDirection;
use crate::command_palette::CommandPaletteMessage;
use crate::command_palette::CommandPaletteModel;
use crate::command_palette::CommandPaletteNavigationPolicy;
use crate::command_palette::reduce_command_palette;

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
    /// Stable typed action ID.
    pub id: ActionId,
    /// Human-readable action label.
    pub label: String,
    /// Whether the action can be selected.
    pub enabled: bool,
}

impl ReplayCommandAction {
    /// Creates an enabled command palette action.
    #[must_use]
    pub fn enabled(id: impl Into<ActionId>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            enabled: true,
        }
    }

    /// Creates a disabled command palette action.
    #[must_use]
    pub fn disabled(id: impl Into<ActionId>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            enabled: false,
        }
    }

    fn candidate(&self) -> CommandPaletteCandidate {
        if self.enabled {
            CommandPaletteCandidate::enabled(self.id.clone(), self.label.clone())
        } else {
            CommandPaletteCandidate::disabled(self.id.clone(), self.label.clone(), "replay action is disabled")
        }
    }
}

/// Pure replay target adapting replay events into the shared command palette reducer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandPaletteReplayTarget {
    actions: Vec<ReplayCommandAction>,
    model: CommandPaletteModel,
    width: u16,
    height: u16,
}

impl CommandPaletteReplayTarget {
    /// Creates a command palette replay target.
    #[must_use]
    pub fn new(actions: Vec<ReplayCommandAction>) -> Self {
        let candidates = actions.iter().map(ReplayCommandAction::candidate).collect();
        let model = CommandPaletteModel::build(candidates, CommandPaletteNavigationPolicy::Wrap, true).model;
        Self {
            actions,
            model,
            width: DEFAULT_REPLAY_WIDTH,
            height: DEFAULT_REPLAY_HEIGHT,
        }
    }

    /// Returns actions matching the current query in shared reducer order.
    #[must_use]
    pub fn filtered_actions(&self) -> Vec<&ReplayCommandAction> {
        self.model
            .matches()
            .iter()
            .filter_map(|row| {
                let action_id = self.model.candidates()[row.candidate_index()].action_id();
                self.actions.iter().find(|action| &action.id == action_id)
            })
            .collect()
    }

    /// Returns the active typed action ID when a row is available.
    #[must_use]
    pub fn active_action_id(&self) -> Option<&ActionId> {
        self.model.active_action_id()
    }

    fn preview(&self, message: CommandPaletteMessage) -> Vec<CommandPaletteDiagnostic> {
        reduce_command_palette(&self.model, message).diagnostics
    }

    fn dispatch(&mut self, message: CommandPaletteMessage) {
        let transition = reduce_command_palette(&self.model, message);
        if transition.accepted {
            self.model = transition.model;
        }
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
        self.model.candidate(&ActionId::new(action_id)).is_some()
    }

    fn action_enabled(&self, action_id: &str) -> bool {
        self.model.candidate(&ActionId::new(action_id)).is_some_and(CommandPaletteCandidate::is_enabled)
    }

    fn validate_event(&self, event: &ReplayEvent) -> Vec<ReplayIssue> {
        match event {
            ReplayEvent::Key { key, .. } => self.validate_key(key),
            ReplayEvent::Text { value, .. } if value.chars().any(char::is_control) => diagnostics_to_replay_issues(
                self.preview(CommandPaletteMessage::ReplaceQuery(format!("{}{value}", self.model.query()))),
            ),
            _ => Vec::new(),
        }
    }

    fn apply_event(&mut self, event: &ReplayEvent) {
        match event {
            ReplayEvent::Text { value, .. } => {
                self.dispatch(CommandPaletteMessage::ReplaceQuery(format!("{}{value}", self.model.query())))
            }
            ReplayEvent::Key { key, .. } => {
                if let Some(message) = message_for_replay_key(key) {
                    self.dispatch(message);
                }
            }
            ReplayEvent::Command { action_id } => {
                self.dispatch(CommandPaletteMessage::ActivateAction(ActionId::new(action_id)))
            }
            ReplayEvent::Resize { width, height } => {
                self.width = *width;
                self.height = *height;
            }
            ReplayEvent::Mouse { .. } | ReplayEvent::Wait { .. } | ReplayEvent::Unsupported { .. } => {}
        }
    }

    fn snapshot_state(&self) -> std::collections::BTreeMap<String, String> {
        std::collections::BTreeMap::from([
            ("active_action".to_string(), option_id(self.model.active_action_id())),
            ("cancelled".to_string(), self.model.is_cancelled().to_string()),
            ("filtered_actions".to_string(), join_ids(self.filtered_actions().iter().map(|action| &action.id))),
            ("query".to_string(), self.model.query().to_string()),
            ("selected_action".to_string(), option_id(self.model.selected_action_id())),
            ("surface".to_string(), TARGET_COMMAND_PALETTE.to_string()),
        ])
    }
}

impl CommandPaletteReplayTarget {
    fn validate_key(&self, key: &str) -> Vec<ReplayIssue> {
        let Some(message) = message_for_replay_key(key) else {
            return vec![ReplayIssue::new(
                ReplayIssueCode::UnsupportedEvent,
                FIELD_COMMAND_KEY,
                format!("command palette replay does not support key `{key}`"),
            )];
        };
        diagnostics_to_replay_issues(self.preview(message))
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
    .with_snapshot(ReplaySnapshotExpectation::new("state", state_snapshot_hash(&accepted_command_palette_state())))
}

fn message_for_replay_key(key: &str) -> Option<CommandPaletteMessage> {
    match key {
        KEY_ARROW_DOWN => Some(CommandPaletteMessage::Move(CommandPaletteDirection::Next)),
        KEY_ARROW_UP => Some(CommandPaletteMessage::Move(CommandPaletteDirection::Previous)),
        KEY_ENTER => Some(CommandPaletteMessage::Activate),
        KEY_ESCAPE => Some(CommandPaletteMessage::Cancel),
        _ => None,
    }
}

fn diagnostics_to_replay_issues(diagnostics: Vec<CommandPaletteDiagnostic>) -> Vec<ReplayIssue> {
    diagnostics
        .into_iter()
        .map(|diagnostic| {
            let (code, field) = match diagnostic.code {
                CommandPaletteDiagnosticCode::DisabledAction => (ReplayIssueCode::DisabledAction, FIELD_ENTER),
                CommandPaletteDiagnosticCode::NavigationBoundary => {
                    (ReplayIssueCode::OutOfBoundsMovement, FIELD_MOVEMENT)
                }
                CommandPaletteDiagnosticCode::MalformedQuery => (ReplayIssueCode::MalformedText, FIELD_COMMAND_TEXT),
                CommandPaletteDiagnosticCode::UnknownAction => (ReplayIssueCode::UnknownCommand, FIELD_ENTER),
                CommandPaletteDiagnosticCode::EmptyActionId
                | CommandPaletteDiagnosticCode::DuplicateActionId
                | CommandPaletteDiagnosticCode::FilteredAction
                | CommandPaletteDiagnosticCode::NoMatchingAction => {
                    (ReplayIssueCode::UnsupportedEvent, FIELD_COMMAND_KEY)
                }
            };
            ReplayIssue::new(code, field, diagnostic.message)
        })
        .collect()
}

fn option_id(value: Option<&ActionId>) -> String {
    value.map_or_else(|| EMPTY_MARKER.to_string(), ToString::to_string)
}

fn join_ids<'a>(ids: impl Iterator<Item = &'a ActionId>) -> String {
    let joined = ids.map(ToString::to_string).collect::<Vec<_>>().join(ID_SEPARATOR);
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

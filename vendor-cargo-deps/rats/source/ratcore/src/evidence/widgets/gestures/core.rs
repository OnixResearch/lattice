use std::collections::BTreeMap;
use std::collections::BTreeSet;

use super::super::super::*;

pub(super) const TARGET_FULL_MOUSE: &str = "full-mouse-surface";
pub(super) const SNAPSHOT_ROUTE_SUMMARY: &str = "route-summary";
pub(super) const DEFAULT_REPLAY_WIDTH: u16 = 120;
pub(super) const DEFAULT_REPLAY_HEIGHT: u16 = 40;
pub(super) const EMPTY_MARKER: &str = "-";

const STATE_ACCEPTED_ROUTES: &str = "accepted_routes";
const STATE_CAPTURED_TARGET: &str = "captured_target";
const STATE_DIAGNOSTICS: &str = "diagnostics";
const STATE_LAST_SURFACE: &str = "last_surface";
const STATE_ROUTE_SUMMARY: &str = "route_summary";
const ROUTE_SEPARATOR: &str = ";";
const FIELD_COMMAND: &str = "event.command";
const FIELD_CAPTURE: &str = "event.capture";
const FIELD_MOUSE: &str = "event.mouse";
const MOUSE_HOVER_CLASS: &str = "hover";
const WORKSPACE_SURFACE: &str = "workspace-shell";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum PointerMouseBehavior {
    Route,
    DragStart { capture: String },
    DragUpdate { capture: String },
    DragRelease { capture: String },
    DragCancel,
    Unsupported { diagnostic: String },
    StaleCapture { expected: String, actual: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PointerMouseAction {
    event_class: String,
    surface: String,
    route_id: String,
    behavior: PointerMouseBehavior,
}

impl PointerMouseAction {
    pub(super) fn route(
        event_class: impl Into<String>,
        surface: impl Into<String>,
        route_id: impl Into<String>,
    ) -> Self {
        Self {
            event_class: event_class.into(),
            surface: surface.into(),
            route_id: route_id.into(),
            behavior: PointerMouseBehavior::Route,
        }
    }

    pub(super) fn with_behavior(mut self, behavior: PointerMouseBehavior) -> Self {
        self.behavior = behavior;
        self
    }
}

/// Pure replay target for full mouse support proof fixtures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PointerMouseReplayTarget {
    actions: BTreeMap<String, PointerMouseAction>,
    disabled_actions: BTreeSet<String>,
    route_log: Vec<String>,
    diagnostics: BTreeSet<String>,
    captured_target: Option<String>,
    width: u16,
    height: u16,
}

impl PointerMouseReplayTarget {
    pub(super) fn new(actions: BTreeMap<String, PointerMouseAction>, disabled_actions: BTreeSet<String>) -> Self {
        debug_assert!(!actions.is_empty());
        debug_assert!(DEFAULT_REPLAY_WIDTH > 0);
        Self {
            actions,
            disabled_actions,
            route_log: Vec::new(),
            diagnostics: BTreeSet::new(),
            captured_target: None,
            width: DEFAULT_REPLAY_WIDTH,
            height: DEFAULT_REPLAY_HEIGHT,
        }
    }

    fn validate_command(&self, action_id: &str) -> Vec<ReplayIssue> {
        let Some(action) = self.actions.get(action_id) else {
            return Vec::new();
        };
        match &action.behavior {
            PointerMouseBehavior::DragUpdate { capture } | PointerMouseBehavior::DragRelease { capture } => {
                self.validate_capture(action_id, capture)
            }
            PointerMouseBehavior::Unsupported { diagnostic } => vec![ReplayIssue::new(
                ReplayIssueCode::UnsupportedEvent,
                FIELD_COMMAND,
                format!("unsupported pointer event `{diagnostic}`"),
            )],
            PointerMouseBehavior::StaleCapture { expected, actual } => vec![ReplayIssue::new(
                ReplayIssueCode::StaleCapture,
                FIELD_CAPTURE,
                format!("stale pointer capture `{actual}` did not match `{expected}`"),
            )],
            PointerMouseBehavior::Route | PointerMouseBehavior::DragStart { .. } | PointerMouseBehavior::DragCancel => {
                Vec::new()
            }
        }
    }

    fn validate_capture(&self, action_id: &str, expected: &str) -> Vec<ReplayIssue> {
        match self.captured_target.as_deref() {
            Some(actual) if actual == expected => Vec::new(),
            Some(actual) => vec![ReplayIssue::new(
                ReplayIssueCode::StaleCapture,
                FIELD_CAPTURE,
                format!("step `{action_id}` expected capture `{expected}` but found `{actual}`"),
            )],
            None => vec![ReplayIssue::new(
                ReplayIssueCode::MissingDragStart,
                FIELD_CAPTURE,
                format!("step `{action_id}` requires drag capture `{expected}`"),
            )],
        }
    }

    fn apply_command(&mut self, action_id: &str) {
        let Some(action) = self.actions.get(action_id).cloned() else {
            return;
        };
        self.route_log.push(route_entry(&action));
        match action.behavior {
            PointerMouseBehavior::DragStart { capture } => self.captured_target = Some(capture),
            PointerMouseBehavior::DragRelease { .. } | PointerMouseBehavior::DragCancel => {
                self.captured_target = None;
            }
            PointerMouseBehavior::Unsupported { diagnostic } => {
                self.diagnostics.insert(diagnostic);
            }
            PointerMouseBehavior::Route
            | PointerMouseBehavior::DragUpdate { .. }
            | PointerMouseBehavior::StaleCapture { .. } => {}
        }
    }

    fn apply_mouse(&mut self, x: u16, y: u16) {
        self.route_log.push(format!("{MOUSE_HOVER_CLASS}:{WORKSPACE_SURFACE}@{x},{y}"));
    }
}

impl ReplayTarget for PointerMouseReplayTarget {
    fn focus_targets(&self) -> BTreeSet<String> {
        [TARGET_FULL_MOUSE.to_string()].into_iter().collect()
    }

    fn focused_target(&self) -> Option<String> {
        Some(TARGET_FULL_MOUSE.to_string())
    }

    fn bounds(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    fn action_known(&self, action_id: &str) -> bool {
        self.actions.contains_key(action_id) || self.disabled_actions.contains(action_id)
    }

    fn action_enabled(&self, action_id: &str) -> bool {
        self.actions.contains_key(action_id) && !self.disabled_actions.contains(action_id)
    }

    fn validate_event(&self, event: &ReplayEvent) -> Vec<ReplayIssue> {
        match event {
            ReplayEvent::Command { action_id } => self.validate_command(action_id),
            ReplayEvent::Mouse { x, y, .. } if *x >= self.width || *y >= self.height => {
                vec![ReplayIssue::new(
                    ReplayIssueCode::ImpossibleGeometry,
                    FIELD_MOUSE,
                    "mouse replay received impossible surface geometry",
                )]
            }
            _ => Vec::new(),
        }
    }

    fn apply_event(&mut self, event: &ReplayEvent) {
        match event {
            ReplayEvent::Command { action_id } => self.apply_command(action_id),
            ReplayEvent::Mouse { x, y, .. } => self.apply_mouse(*x, *y),
            ReplayEvent::Resize { width, height } => {
                self.width = *width;
                self.height = *height;
            }
            ReplayEvent::Key { .. }
            | ReplayEvent::Text { .. }
            | ReplayEvent::Wait { .. }
            | ReplayEvent::Unsupported { .. } => {}
        }
    }

    fn snapshot_state(&self) -> BTreeMap<String, String> {
        BTreeMap::from([
            (STATE_ACCEPTED_ROUTES.to_string(), self.route_log.len().to_string()),
            (
                STATE_CAPTURED_TARGET.to_string(),
                self.captured_target.clone().unwrap_or_else(|| EMPTY_MARKER.to_string()),
            ),
            (STATE_DIAGNOSTICS.to_string(), join_sorted(self.diagnostics.iter())),
            (
                STATE_LAST_SURFACE.to_string(),
                last_surface(&self.route_log).unwrap_or_else(|| EMPTY_MARKER.to_string()),
            ),
            (STATE_ROUTE_SUMMARY.to_string(), join_sorted(self.route_log.iter())),
        ])
    }

    fn snapshot_bytes(&self, label: &str) -> Option<Vec<u8>> {
        (label == SNAPSHOT_ROUTE_SUMMARY).then(|| state_bytes(&self.snapshot_state()))
    }

    fn diagnostic_events(&self) -> BTreeSet<String> {
        self.diagnostics.clone()
    }
}

pub(super) fn state_bytes(state: &BTreeMap<String, String>) -> Vec<u8> {
    state.iter().map(|(key, value)| format!("{key}={value}\n")).collect::<String>().into_bytes()
}

fn route_entry(action: &PointerMouseAction) -> String {
    format!("{}:{}:{}", action.event_class, action.surface, action.route_id)
}

fn last_surface(route_log: &[String]) -> Option<String> {
    route_log.last().and_then(|entry| entry.split(':').nth(1)).map(str::to_string)
}

fn join_sorted(values: impl IntoIterator<Item = impl AsRef<str>>) -> String {
    let items = values.into_iter().map(|value| value.as_ref().to_string()).collect::<Vec<_>>();
    if items.is_empty() {
        EMPTY_MARKER.to_string()
    } else {
        items.join(ROUTE_SEPARATOR)
    }
}

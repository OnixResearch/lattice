use super::super::*;

const TARGET_TREE: &str = "tree";
const KEY_ARROW_DOWN: &str = "ArrowDown";
const KEY_ARROW_UP: &str = "ArrowUp";
const KEY_ARROW_RIGHT: &str = "ArrowRight";
const KEY_ARROW_LEFT: &str = "ArrowLeft";
const KEY_ENTER: &str = "Enter";
const KEY_ESCAPE: &str = "Escape";
const FIELD_TREE_KEY: &str = "tree.key";
const FIELD_MOVEMENT: &str = "movement";
const EMPTY_MARKER: &str = "-";
const ID_SEPARATOR: &str = ",";
const DEFAULT_REPLAY_WIDTH: u16 = 80;
const DEFAULT_REPLAY_HEIGHT: u16 = 24;

/// Replay tree node used by renderer-neutral tree fixtures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayTreeNode {
    /// Stable node ID.
    pub id: String,
    /// Child nodes in display order.
    pub children: Vec<ReplayTreeNode>,
}

impl ReplayTreeNode {
    /// Creates a tree node.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            children: Vec::new(),
        }
    }

    /// Adds a child node.
    #[must_use]
    pub fn with_child(mut self, child: ReplayTreeNode) -> Self {
        self.children.push(child);
        self
    }
}

/// Pure replay target for tree navigation behavior.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeReplayTarget {
    roots: Vec<ReplayTreeNode>,
    expanded: std::collections::BTreeSet<String>,
    active_index: usize,
    selected_node: Option<String>,
    cancelled: bool,
    width: u16,
    height: u16,
}

impl TreeReplayTarget {
    /// Creates a tree replay target from root nodes.
    #[must_use]
    pub fn new(roots: Vec<ReplayTreeNode>) -> Self {
        Self {
            roots,
            expanded: std::collections::BTreeSet::new(),
            active_index: 0,
            selected_node: None,
            cancelled: false,
            width: DEFAULT_REPLAY_WIDTH,
            height: DEFAULT_REPLAY_HEIGHT,
        }
    }

    /// Returns visible node IDs in display order.
    #[must_use]
    pub fn visible_node_ids(&self) -> Vec<String> {
        visible_node_ids(&self.roots, &self.expanded)
    }

    /// Returns the active node ID when one is visible.
    #[must_use]
    pub fn active_node_id(&self) -> Option<String> {
        self.visible_node_ids().get(self.active_index).cloned()
    }
}

impl ReplayTarget for TreeReplayTarget {
    fn focus_targets(&self) -> std::collections::BTreeSet<String> {
        [TARGET_TREE.to_string()].into_iter().collect()
    }

    fn focused_target(&self) -> Option<String> {
        Some(TARGET_TREE.to_string())
    }

    fn bounds(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    fn action_known(&self, _action_id: &str) -> bool {
        false
    }

    fn action_enabled(&self, _action_id: &str) -> bool {
        false
    }

    fn validate_event(&self, event: &ReplayEvent) -> Vec<ReplayIssue> {
        match event {
            ReplayEvent::Key { key, .. } => self.validate_key(key),
            ReplayEvent::Text { .. } => vec![ReplayIssue::new(
                ReplayIssueCode::MalformedText,
                FIELD_TREE_KEY,
                "tree replay does not accept text input",
            )],
            _ => Vec::new(),
        }
    }

    fn apply_event(&mut self, event: &ReplayEvent) {
        match event {
            ReplayEvent::Key { key, .. } if key == KEY_ARROW_DOWN => {
                self.active_index = self.active_index.saturating_add(1);
            }
            ReplayEvent::Key { key, .. } if key == KEY_ARROW_UP => {
                self.active_index = self.active_index.saturating_sub(1);
            }
            ReplayEvent::Key { key, .. } if key == KEY_ARROW_RIGHT => {
                if let Some(node_id) = self.active_node_id() {
                    self.expanded.insert(node_id);
                }
            }
            ReplayEvent::Key { key, .. } if key == KEY_ARROW_LEFT => {
                if let Some(node_id) = self.active_node_id() {
                    self.expanded.remove(&node_id);
                }
            }
            ReplayEvent::Key { key, .. } if key == KEY_ENTER => {
                self.selected_node = self.active_node_id();
            }
            ReplayEvent::Key { key, .. } if key == KEY_ESCAPE => {
                self.cancelled = true;
            }
            ReplayEvent::Mouse { y, .. } => {
                let next_index = usize::from(*y);
                if next_index < self.visible_node_ids().len() {
                    self.active_index = next_index;
                }
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
            ("active_node".to_string(), option_id(self.active_node_id())),
            ("cancelled".to_string(), self.cancelled.to_string()),
            ("expanded".to_string(), join_ids(self.expanded.iter())),
            (
                "selected_node".to_string(),
                option_id(self.selected_node.clone()),
            ),
            ("surface".to_string(), TARGET_TREE.to_string()),
            (
                "visible_rows".to_string(),
                join_ids(self.visible_node_ids().iter()),
            ),
        ])
    }
}

impl TreeReplayTarget {
    fn validate_key(&self, key: &str) -> Vec<ReplayIssue> {
        match key {
            KEY_ARROW_DOWN
                if self.active_index.saturating_add(1) >= self.visible_node_ids().len() =>
            {
                vec![ReplayIssue::new(
                    ReplayIssueCode::OutOfBoundsMovement,
                    FIELD_MOVEMENT,
                    "tree replay cannot move below the final visible row",
                )]
            }
            KEY_ARROW_UP if self.active_index == 0 => vec![ReplayIssue::new(
                ReplayIssueCode::OutOfBoundsMovement,
                FIELD_MOVEMENT,
                "tree replay cannot move above the first visible row",
            )],
            KEY_ARROW_DOWN | KEY_ARROW_UP | KEY_ARROW_RIGHT | KEY_ARROW_LEFT | KEY_ENTER
            | KEY_ESCAPE => Vec::new(),
            _ => vec![ReplayIssue::new(
                ReplayIssueCode::UnsupportedEvent,
                FIELD_TREE_KEY,
                format!("tree replay does not support key `{key}`"),
            )],
        }
    }
}

/// Returns the committed tree navigation replay fixture.
#[must_use]
pub fn tree_navigation_replay_script() -> ReplayScript {
    ReplayScript::new(ReplayMetadata::new(
        "tree.navigation.baseline",
        "Tree navigation baseline",
        "tree",
    ))
    .with_event(ReplayEvent::Key {
        target: TARGET_TREE.to_string(),
        key: KEY_ARROW_RIGHT.to_string(),
    })
    .with_event(ReplayEvent::Key {
        target: TARGET_TREE.to_string(),
        key: KEY_ARROW_DOWN.to_string(),
    })
    .with_event(ReplayEvent::Key {
        target: TARGET_TREE.to_string(),
        key: KEY_ENTER.to_string(),
    })
    .with_assertion(ReplayAssertion::new("selected_node", "src/lib.rs"))
    .with_assertion(ReplayAssertion::new("expanded", "src"))
    .with_assertion(ReplayAssertion::new(
        "visible_rows",
        "src,src/lib.rs,README.md",
    ))
    .with_snapshot(ReplaySnapshotExpectation::new(
        "state",
        state_snapshot_hash(&accepted_tree_state()),
    ))
}

fn visible_node_ids(
    nodes: &[ReplayTreeNode],
    expanded: &std::collections::BTreeSet<String>,
) -> Vec<String> {
    let mut visible = Vec::with_capacity(visible_node_bound(nodes));
    let mut stack = nodes.iter().rev().collect::<Vec<_>>();
    while let Some(node) = stack.pop() {
        visible.push(node.id.clone());
        if expanded.contains(&node.id) {
            stack.extend(node.children.iter().rev());
        }
    }
    visible
}

fn visible_node_bound(nodes: &[ReplayTreeNode]) -> usize {
    let mut total = 0usize;
    let mut stack = nodes.iter().collect::<Vec<_>>();
    while let Some(node) = stack.pop() {
        total = total.saturating_add(1);
        stack.extend(node.children.iter());
    }
    total
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

fn accepted_tree_state() -> std::collections::BTreeMap<String, String> {
    std::collections::BTreeMap::from([
        ("active_node".to_string(), "src/lib.rs".to_string()),
        ("cancelled".to_string(), false.to_string()),
        ("expanded".to_string(), "src".to_string()),
        ("selected_node".to_string(), "src/lib.rs".to_string()),
        ("surface".to_string(), TARGET_TREE.to_string()),
        (
            "visible_rows".to_string(),
            "src,src/lib.rs,README.md".to_string(),
        ),
    ])
}

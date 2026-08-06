//! Framework-agnostic inline view tree and reconciler.
//!
//! Provides the data types and pure-function reconciliation algorithm
//! used by both ratatui (rat-inline) and dioxus (met-inline) backends
//! for inline scrollback rendering.
//!
//! The reconciler matches nodes by key (stable identity across reorders)
//! then by position + type (fast path for static layouts). Matched nodes
//! preserve their opaque state blob across rebuilds.

// ── Types ────────────────────────────────────────────────────────────────────

/// A string key for stable node identity across rebuilds.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NodeKey(
    /// Stable node identity text.
    pub String,
);

impl<S: Into<String>> From<S> for NodeKey {
    fn from(s: S) -> Self {
        Self(s.into())
    }
}

/// A single node in the view tree.
///
/// Carries an optional key for reconciliation, a `TypeId` for
/// type-based positional matching, and an opaque state slot that
/// backends use to preserve widget state across rebuilds.
pub struct ViewNode {
    /// Optional key for stable identity across rebuilds.
    pub key: Option<NodeKey>,
    /// Type discriminant — used for positional matching when no key is set.
    pub type_tag: std::any::TypeId,
    /// Opaque state preserved across reconciliation. `None` for new nodes.
    pub state: Option<Box<dyn std::any::Any>>,
}

impl std::fmt::Debug for ViewNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ViewNode")
            .field("key", &self.key)
            .field("type_tag", &self.type_tag)
            .field("has_state", &self.state.is_some())
            .finish()
    }
}

/// A flat list of view nodes representing the current inline content.
#[derive(Debug, Default)]
pub struct ViewTree {
    /// Nodes in display order.
    pub nodes: Vec<ViewNode>,
}

impl ViewTree {
    /// Creates an empty view tree.
    #[must_use]
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    /// Appends a node to the end of the tree.
    pub fn push(&mut self, node: ViewNode) {
        self.nodes.push(node);
    }

    /// Returns the number of nodes in the tree.
    #[must_use]
    pub fn len(&self) -> u64 {
        index_from_usize(self.nodes.len())
    }

    /// Returns `true` when the tree contains no nodes.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

// ── Reconciliation ───────────────────────────────────────────────────────────

/// Result of reconciling an old tree against a new tree.
pub struct ReconcileResult {
    /// The reconciled nodes — new nodes with state carried forward from
    /// matched old nodes.
    pub nodes: Vec<ViewNode>,
}

/// Reconcile a new view tree against an old one.
///
/// Two-pass O(N) algorithm:
/// 1. Match keyed new nodes against keyed old nodes by key.
/// 2. Match remaining unkeyed new nodes by position + `type_tag`.
///
/// Matched nodes get their `state` transferred from the old node.
/// Unmatched new nodes keep `state: None`.
/// Unmatched old nodes are dropped.
///
/// This is a pure function with no side effects.
#[must_use]
pub fn reconcile(mut old: Vec<ViewNode>, new: Vec<ViewNode>) -> ReconcileResult {
    // Build index of keyed old nodes: key -> position in old vec.
    let mut old_by_key: std::collections::HashMap<NodeKey, usize> = std::collections::HashMap::with_capacity(old.len());
    for (i, node) in old.iter().enumerate() {
        if let Some(ref key) = node.key {
            old_by_key.insert(key.clone(), i);
        }
    }

    // Track which old nodes have been consumed (by index).
    let mut old_consumed = vec![false; old.len()];

    // Pass 1: match keyed nodes.
    // We collect which old index each new node matched, if any.
    let mut matched_old: Vec<Option<usize>> = Vec::with_capacity(new.len());
    for new_node in &new {
        if let Some(ref key) = new_node.key
            && let Some(&old_idx) = old_by_key.get(key)
            && !old_consumed[old_idx]
        {
            old_consumed[old_idx] = true;
            matched_old.push(Some(old_idx));
            continue;
        }
        matched_old.push(None);
    }

    debug_assert_eq!(matched_old.len(), new.len());
    debug_assert!(old_by_key.len() <= old.len());

    // Pass 2: match remaining unkeyed nodes by position + type.
    // Build a list of unconsumed old nodes grouped by type for positional matching.
    // We iterate unconsumed old nodes in order, tracking position per type.
    let mut positional_queues: std::collections::HashMap<std::any::TypeId, Vec<usize>> =
        std::collections::HashMap::new();
    for (i, node) in old.iter().enumerate() {
        if !old_consumed[i] && node.key.is_none() {
            positional_queues.entry(node.type_tag).or_default().push(i);
        }
    }

    // For each unmatched new node (no key match), try positional match.
    for (new_idx, new_node) in new.iter().enumerate() {
        if matched_old[new_idx].is_some() {
            continue; // already matched by key
        }
        if new_node.key.is_some() {
            continue; // keyed but no match found — stays unmatched
        }
        if let Some(queue) = positional_queues.get_mut(&new_node.type_tag)
            && let Some(old_idx) = queue.first().copied()
        {
            queue.remove(0);
            old_consumed[old_idx] = true;
            matched_old[new_idx] = Some(old_idx);
        }
    }

    // Build result: transfer state from matched old nodes.
    let new_node_count = new.len();
    let mut result_nodes: Vec<ViewNode> = Vec::with_capacity(new_node_count);
    for (new_idx, mut new_node) in new.into_iter().enumerate() {
        if let Some(old_idx) = matched_old[new_idx] {
            new_node.state = old[old_idx].state.take();
        }
        result_nodes.push(new_node);
    }

    debug_assert_eq!(result_nodes.len(), matched_old.len());
    debug_assert!(result_nodes.len() <= new_node_count);

    ReconcileResult { nodes: result_nodes }
}

// ── Commit tracking ──────────────────────────────────────────────────────────

/// Viewport information for [`compute_commits`].
#[derive(Clone, Copy)]
pub struct CommitViewport {
    /// Height of the visible viewport in rows.
    pub viewport_height: u16,
    /// Number of rows scrolled above the viewport.
    pub scroll_offset: u16,
}

/// Compute which nodes have been fully scrolled above the viewport.
///
/// Given node heights and viewport information, returns indices of nodes
/// whose rows are entirely above the visible region. Assumes nodes are
/// stacked top-to-bottom starting from `scroll_offset`.
///
/// This is a pure function — the backend supplies the actual viewport
/// height from the terminal.
#[must_use]
pub fn compute_commits(node_heights: &[u16], viewport: CommitViewport) -> Vec<u64> {
    let CommitViewport {
        viewport_height,
        scroll_offset,
    } = viewport;
    let total_height: u16 = node_heights.iter().copied().sum();
    if total_height <= viewport_height {
        return Vec::new();
    }

    debug_assert!(total_height > viewport_height);

    // Nodes above the viewport are those whose bottom edge is at or
    // above the scroll offset. In scrollback mode, scroll_offset
    // represents how many rows have scrolled above the viewport.
    let mut committed = Vec::with_capacity(node_heights.len());
    let mut y = 0u16;
    for (i, &h) in node_heights.iter().enumerate() {
        let node_bottom = y.saturating_add(h);
        if node_bottom <= scroll_offset {
            committed.push(index_from_usize(i));
        } else {
            break; // nodes are stacked, so once we're in the viewport, stop
        }
        y = node_bottom;
    }

    debug_assert!(committed.len() <= node_heights.len());
    debug_assert!(committed.windows(2).all(|window| window[0] < window[1]));
    committed
}

fn index_from_usize(index: usize) -> u64 {
    index as u64
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[path = "inline/tests.rs"]
mod tests;

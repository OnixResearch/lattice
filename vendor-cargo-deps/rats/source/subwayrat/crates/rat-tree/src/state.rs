//! Mutable tree state: cursor, scroll, expand/collapse tracking.

use std::collections::BTreeSet;

use crate::keymap::TreeAction;
use crate::model::{NodeId, TreeData, VisibleRow, compute_visible_rows};

/// Mutable state for a tree widget.
///
/// Tracks which nodes are expanded, the cursor position (index into the flat
/// visible-row list), and the scroll offset for viewport clipping.
#[derive(Debug, Clone)]
pub struct TreeState {
    /// Cursor index into `visible_rows`.
    pub cursor: usize,
    /// First visible row in the viewport.
    pub scroll_offset: usize,
    /// Set of expanded node ids.
    pub expanded: BTreeSet<NodeId>,
    /// Cached flat list of visible rows, recomputed on expand/collapse.
    pub visible_rows: Vec<VisibleRow>,
}

impl TreeState {
    /// Create initial state with all nodes collapsed.
    pub fn new(data: &dyn TreeData) -> Self {
        let expanded = BTreeSet::new();
        let visible_rows = compute_visible_rows(data, &expanded);
        Self {
            cursor: 0,
            scroll_offset: 0,
            expanded,
            visible_rows,
        }
    }

    /// Recompute the cached visible rows from the data and expand state.
    pub fn recompute(&mut self, data: &dyn TreeData) {
        self.visible_rows = compute_visible_rows(data, &self.expanded);
    }

    /// Expand the node under the cursor. No-op if it's a leaf or already expanded.
    pub fn expand(&mut self, data: &dyn TreeData) {
        if let Some(row) = self.visible_rows.get(self.cursor)
            && row.has_children
            && !row.is_expanded
        {
            self.expanded.insert(row.node_id);
            self.recompute(data);
        }
    }

    /// Collapse the node under the cursor. No-op if it's a leaf or already collapsed.
    pub fn collapse(&mut self, data: &dyn TreeData) {
        if let Some(row) = self.visible_rows.get(self.cursor)
            && row.has_children
            && row.is_expanded
        {
            self.expanded.remove(&row.node_id);
            self.recompute(data);
            // Cursor might now point past the end
            self.clamp_cursor();
        }
    }

    /// Toggle expand/collapse on the node under the cursor.
    pub fn toggle(&mut self, data: &dyn TreeData) {
        if let Some(row) = self.visible_rows.get(self.cursor).cloned() {
            if !row.has_children {
                return;
            }
            if row.is_expanded {
                // Find descendants that will be removed, check if cursor is among them
                let collapsed_id = row.node_id;
                let cursor_node_id = self.visible_rows.get(self.cursor).map(|r| r.node_id);
                self.expanded.remove(&collapsed_id);
                self.recompute(data);

                // If cursor was on a descendant of the collapsed node, move to collapsed node
                if let Some(cid) = cursor_node_id
                    && cid != collapsed_id
                    // Cursor was on the node itself, find it
                    && let Some(pos) = self
                        .visible_rows
                        .iter()
                        .position(|r| r.node_id == collapsed_id)
                {
                    self.cursor = pos;
                }
                self.clamp_cursor();
            } else {
                self.expanded.insert(row.node_id);
                self.recompute(data);
            }
        }
    }

    // ── Cursor navigation ───────────────────────────────────────────────

    /// Move cursor down one row, clamped at the last row.
    pub fn move_down(&mut self) {
        if !self.visible_rows.is_empty() {
            self.cursor = (self.cursor + 1).min(self.visible_rows.len() - 1);
        }
    }

    /// Move cursor up one row, clamped at row 0.
    pub fn move_up(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    /// Jump cursor to the first row.
    pub fn jump_first(&mut self) {
        self.cursor = 0;
    }

    /// Jump cursor to the last row.
    pub fn jump_last(&mut self) {
        if !self.visible_rows.is_empty() {
            self.cursor = self.visible_rows.len() - 1;
        }
    }

    /// Move cursor to the parent of the current node.
    pub fn navigate_parent(&mut self, data: &dyn TreeData) {
        if let Some(row) = self.visible_rows.get(self.cursor)
            && let Some(parent_id) = data.parent(row.node_id)
            && let Some(pos) = self
                .visible_rows
                .iter()
                .position(|r| r.node_id == parent_id)
        {
            self.cursor = pos;
        }
    }

    /// Move cursor to the first child of the current node.
    /// Auto-expands if the node is collapsed.
    pub fn navigate_first_child(&mut self, data: &dyn TreeData) {
        if let Some(row) = self.visible_rows.get(self.cursor).cloned() {
            if !row.has_children {
                return;
            }
            // Expand if needed
            if !row.is_expanded {
                self.expanded.insert(row.node_id);
                self.recompute(data);
            }
            // First child is the next row after the current cursor
            let first_child_id = data.child(row.node_id, 0);
            if let Some(pos) = self
                .visible_rows
                .iter()
                .position(|r| r.node_id == first_child_id)
            {
                self.cursor = pos;
            }
        }
    }

    /// Move cursor to the next sibling (same parent, next in order).
    pub fn next_sibling(&mut self, data: &dyn TreeData) {
        if let Some(row) = self.visible_rows.get(self.cursor) {
            let node_id = row.node_id;
            let depth = row.depth;

            // Walk forward through visible rows to find next node at same depth
            // that shares the same parent
            for i in (self.cursor + 1)..self.visible_rows.len() {
                let candidate = &self.visible_rows[i];
                if candidate.depth < depth {
                    // Went up past our level — no more siblings
                    break;
                }
                if candidate.depth == depth {
                    // Same depth — verify same parent
                    let cur_parent = data.parent(node_id);
                    let cand_parent = data.parent(candidate.node_id);
                    if cur_parent == cand_parent {
                        self.cursor = i;
                    }
                    break;
                }
                // candidate.depth > depth: skip descendants
            }
        }
    }

    /// Move cursor to the previous sibling (same parent, previous in order).
    pub fn prev_sibling(&mut self, data: &dyn TreeData) {
        if let Some(row) = self.visible_rows.get(self.cursor) {
            let node_id = row.node_id;
            let depth = row.depth;

            // Walk backward to find previous node at same depth with same parent
            for i in (0..self.cursor).rev() {
                let candidate = &self.visible_rows[i];
                if candidate.depth < depth {
                    // Went up past our level — no previous sibling
                    break;
                }
                if candidate.depth == depth {
                    let cur_parent = data.parent(node_id);
                    let cand_parent = data.parent(candidate.node_id);
                    if cur_parent == cand_parent {
                        self.cursor = i;
                    }
                    break;
                }
            }
        }
    }

    /// Move cursor down by page_size rows.
    pub fn page_down(&mut self, page_size: usize) {
        if !self.visible_rows.is_empty() {
            self.cursor = (self.cursor + page_size).min(self.visible_rows.len() - 1);
        }
    }

    /// Move cursor up by page_size rows.
    pub fn page_up(&mut self, page_size: usize) {
        self.cursor = self.cursor.saturating_sub(page_size);
    }

    // ── Scroll ──────────────────────────────────────────────────────────

    /// Adjust scroll_offset so the cursor is visible within the viewport.
    pub fn ensure_cursor_visible(&mut self, viewport_height: usize) {
        if viewport_height == 0 {
            return;
        }
        if self.cursor < self.scroll_offset {
            self.scroll_offset = self.cursor;
        } else if self.cursor >= self.scroll_offset + viewport_height {
            self.scroll_offset = self.cursor + 1 - viewport_height;
        }
    }

    // ── Action dispatch ─────────────────────────────────────────────────

    /// Apply a `TreeAction` to this state. Returns `Some(node_id)` for
    /// `TreeAction::Select`, `None` otherwise.
    pub fn apply(
        &mut self,
        action: TreeAction,
        data: &dyn TreeData,
        viewport_height: usize,
    ) -> Option<NodeId> {
        let mut selected = None;
        match action {
            TreeAction::Up => self.move_up(),
            TreeAction::Down => self.move_down(),
            TreeAction::First => self.jump_first(),
            TreeAction::Last => self.jump_last(),
            TreeAction::Expand => self.expand(data),
            TreeAction::Collapse => self.collapse(data),
            TreeAction::Toggle => self.toggle(data),
            TreeAction::Parent => self.navigate_parent(data),
            TreeAction::FirstChild => self.navigate_first_child(data),
            TreeAction::NextSibling => self.next_sibling(data),
            TreeAction::PrevSibling => self.prev_sibling(data),
            TreeAction::PageUp => self.page_up(viewport_height),
            TreeAction::PageDown => self.page_down(viewport_height),
            TreeAction::Select => {
                selected = self.visible_rows.get(self.cursor).map(|r| r.node_id);
            }
        }
        self.ensure_cursor_visible(viewport_height);
        selected
    }

    // ── Helpers ─────────────────────────────────────────────────────────

    /// Node id at the current cursor position, if any.
    pub fn cursor_node_id(&self) -> Option<NodeId> {
        self.visible_rows.get(self.cursor).map(|r| r.node_id)
    }

    fn clamp_cursor(&mut self) {
        if !self.visible_rows.is_empty() {
            self.cursor = self.cursor.min(self.visible_rows.len() - 1);
        } else {
            self.cursor = 0;
        }
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::SimpleTree;

    fn sample_tree() -> SimpleTree {
        // root(0)
        //   ├─ a(1)
        //   │  ├─ a1(3)
        //   │  └─ a2(4)
        //   └─ b(2)
        //      └─ b1(5)
        SimpleTree::new(vec![
            (0, None, "root".into()),
            (1, Some(0), "a".into()),
            (2, Some(0), "b".into()),
            (3, Some(1), "a1".into()),
            (4, Some(1), "a2".into()),
            (5, Some(2), "b1".into()),
        ])
    }

    #[test]
    fn new_state_starts_at_zero() {
        let tree = sample_tree();
        let state = TreeState::new(&tree);
        assert_eq!(state.cursor, 0);
        assert_eq!(state.scroll_offset, 0);
        assert!(state.expanded.is_empty());
        assert_eq!(state.visible_rows.len(), 1); // just root
    }

    #[test]
    fn expand_and_collapse() {
        let tree = sample_tree();
        let mut state = TreeState::new(&tree);

        // Expand root
        state.expand(&tree);
        assert_eq!(state.visible_rows.len(), 3); // root, a, b
        assert!(state.expanded.contains(&0));

        // Collapse root
        state.collapse(&tree);
        assert_eq!(state.visible_rows.len(), 1);
        assert!(!state.expanded.contains(&0));
    }

    #[test]
    fn expand_leaf_is_noop() {
        let tree = sample_tree();
        let mut state = TreeState::new(&tree);
        state.expand(&tree); // expand root
        state.cursor = 1; // move to "a"
        state.expand(&tree); // expand "a"
        state.cursor = 2; // move to "a1" (leaf)
        let prev_len = state.visible_rows.len();
        state.expand(&tree); // no-op
        assert_eq!(state.visible_rows.len(), prev_len);
    }

    #[test]
    fn toggle_flips() {
        let tree = sample_tree();
        let mut state = TreeState::new(&tree);

        state.toggle(&tree);
        assert!(state.expanded.contains(&0));
        assert_eq!(state.visible_rows.len(), 3);

        state.toggle(&tree);
        assert!(!state.expanded.contains(&0));
        assert_eq!(state.visible_rows.len(), 1);
    }

    #[test]
    fn move_up_down_clamp() {
        let tree = sample_tree();
        let mut state = TreeState::new(&tree);
        state.expand(&tree); // root,a,b

        state.move_down();
        assert_eq!(state.cursor, 1);
        state.move_down();
        assert_eq!(state.cursor, 2);
        state.move_down(); // clamp
        assert_eq!(state.cursor, 2);

        state.move_up();
        assert_eq!(state.cursor, 1);
        state.move_up();
        assert_eq!(state.cursor, 0);
        state.move_up(); // clamp
        assert_eq!(state.cursor, 0);
    }

    #[test]
    fn jump_first_last() {
        let tree = sample_tree();
        let mut state = TreeState::new(&tree);
        state.expand(&tree);

        state.jump_last();
        assert_eq!(state.cursor, 2);

        state.jump_first();
        assert_eq!(state.cursor, 0);
    }

    #[test]
    fn navigate_parent() {
        let tree = sample_tree();
        let mut state = TreeState::new(&tree);
        state.expanded.insert(0);
        state.expanded.insert(1);
        state.recompute(&tree); // root,a,a1,a2,b

        state.cursor = 2; // a1
        state.navigate_parent(&tree);
        assert_eq!(state.cursor, 1); // a

        state.navigate_parent(&tree);
        assert_eq!(state.cursor, 0); // root

        // Root has no parent — no-op
        state.navigate_parent(&tree);
        assert_eq!(state.cursor, 0);
    }

    #[test]
    fn navigate_first_child() {
        let tree = sample_tree();
        let mut state = TreeState::new(&tree);

        // Root is collapsed — should auto-expand
        state.navigate_first_child(&tree);
        assert!(state.expanded.contains(&0));
        assert_eq!(state.cursor_node_id(), Some(1)); // moved to "a"
    }

    #[test]
    fn navigate_first_child_leaf_noop() {
        let tree = sample_tree();
        let mut state = TreeState::new(&tree);
        state.expanded.insert(0);
        state.expanded.insert(1);
        state.recompute(&tree);

        state.cursor = 2; // a1 (leaf)
        state.navigate_first_child(&tree);
        assert_eq!(state.cursor, 2); // unchanged
    }

    #[test]
    fn next_prev_sibling() {
        let tree = sample_tree();
        let mut state = TreeState::new(&tree);
        state.expanded.insert(0);
        state.expanded.insert(1);
        state.recompute(&tree); // root,a,a1,a2,b

        // a1 → a2
        state.cursor = 2; // a1
        state.next_sibling(&tree);
        assert_eq!(state.cursor_node_id(), Some(4)); // a2

        // a2 → no next sibling
        state.next_sibling(&tree);
        assert_eq!(state.cursor_node_id(), Some(4)); // unchanged

        // a2 → a1
        state.prev_sibling(&tree);
        assert_eq!(state.cursor_node_id(), Some(3)); // a1

        // a1 → no prev sibling
        state.prev_sibling(&tree);
        assert_eq!(state.cursor_node_id(), Some(3)); // unchanged
    }

    #[test]
    fn sibling_skips_descendants() {
        let tree = sample_tree();
        let mut state = TreeState::new(&tree);
        state.expanded.insert(0);
        state.expanded.insert(1);
        state.recompute(&tree); // root,a,a1,a2,b

        // a → b (skips a's children)
        state.cursor = 1; // a
        state.next_sibling(&tree);
        assert_eq!(state.cursor_node_id(), Some(2)); // b

        // b → a
        state.prev_sibling(&tree);
        assert_eq!(state.cursor_node_id(), Some(1)); // a
    }

    #[test]
    fn page_up_down() {
        let tree = SimpleTree::new((0..20).map(|i| (i, None, format!("n{i}"))).collect());
        let mut state = TreeState::new(&tree);
        assert_eq!(state.visible_rows.len(), 20);

        state.page_down(5);
        assert_eq!(state.cursor, 5);

        state.page_down(5);
        assert_eq!(state.cursor, 10);

        state.page_up(3);
        assert_eq!(state.cursor, 7);

        state.page_up(100); // clamp to 0
        assert_eq!(state.cursor, 0);

        state.page_down(100); // clamp to last
        assert_eq!(state.cursor, 19);
    }

    #[test]
    fn scroll_follows_cursor() {
        let tree = SimpleTree::new((0..20).map(|i| (i, None, format!("n{i}"))).collect());
        let mut state = TreeState::new(&tree);

        state.cursor = 15;
        state.ensure_cursor_visible(10);
        // cursor 15 should be last visible → scroll = 15 - 10 + 1 = 6
        assert_eq!(state.scroll_offset, 6);

        state.cursor = 3;
        state.ensure_cursor_visible(10);
        // cursor 3 < scroll 6 → scroll = 3
        assert_eq!(state.scroll_offset, 3);
    }

    #[test]
    fn apply_dispatches_select() {
        let tree = sample_tree();
        let mut state = TreeState::new(&tree);

        let result = state.apply(TreeAction::Select, &tree, 20);
        assert_eq!(result, Some(0)); // root

        state.apply(TreeAction::Expand, &tree, 20);
        state.apply(TreeAction::Down, &tree, 20);
        let result = state.apply(TreeAction::Select, &tree, 20);
        assert_eq!(result, Some(1)); // a
    }

    #[test]
    fn apply_non_select_returns_none() {
        let tree = sample_tree();
        let mut state = TreeState::new(&tree);

        assert_eq!(state.apply(TreeAction::Down, &tree, 20), None);
        assert_eq!(state.apply(TreeAction::Up, &tree, 20), None);
        assert_eq!(state.apply(TreeAction::Toggle, &tree, 20), None);
    }

    #[test]
    fn cursor_adjust_on_collapse() {
        let tree = sample_tree();
        let mut state = TreeState::new(&tree);
        state.expanded.insert(0);
        state.expanded.insert(1);
        state.recompute(&tree); // root,a,a1,a2,b

        // Put cursor on a1 (index 2), then collapse via toggle on root
        state.cursor = 2;
        // Manually collapse root — cursor was on descendant
        state.cursor = 0; // move to root first
        state.toggle(&tree);
        // root is now collapsed, only root visible
        assert_eq!(state.visible_rows.len(), 1);
        assert_eq!(state.cursor, 0);
    }

    #[test]
    fn empty_tree_operations() {
        let tree = SimpleTree::new(vec![]);
        let mut state = TreeState::new(&tree);

        assert_eq!(state.cursor, 0);
        assert!(state.visible_rows.is_empty());

        // All operations should be no-ops, no panics
        state.move_up();
        state.move_down();
        state.jump_first();
        state.jump_last();
        state.expand(&tree);
        state.collapse(&tree);
        state.toggle(&tree);
        state.navigate_parent(&tree);
        state.navigate_first_child(&tree);
        state.next_sibling(&tree);
        state.prev_sibling(&tree);
        state.page_up(10);
        state.page_down(10);
        state.ensure_cursor_visible(10);

        assert_eq!(state.cursor_node_id(), None);
        assert_eq!(state.apply(TreeAction::Select, &tree, 20), None);
    }
}

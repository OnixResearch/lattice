//! Generic tree data model.
//!
//! Defines the `TreeData` trait for top-down tree traversal and a `SimpleTree`
//! adapter for parent-pointer data. Visible row computation flattens the tree
//! into a list based on expand/collapse state.

/// Stable identifier for a node in the tree.
pub type NodeId = u32;

/// Trait for tree data accessed top-down.
///
/// Consumers implement this for their data structure. The widget calls these
/// methods during visible-row computation and rendering.
pub trait TreeData {
    /// Number of root-level nodes.
    fn root_count(&self) -> usize;

    /// Node id of the root at the given index (0-based).
    fn root(&self, index: usize) -> NodeId;

    /// Number of direct children of `node`.
    fn child_count(&self, node: NodeId) -> usize;

    /// Node id of the child at `index` under `node`.
    fn child(&self, node: NodeId, index: usize) -> NodeId;

    /// Display label for the node.
    fn node_label(&self, node: NodeId) -> &str;

    /// Optional icon string rendered before the label.
    fn node_icon(&self, _node: NodeId) -> Option<&str> {
        None
    }

    /// Return the parent node id, if any. Returns `None` for root nodes.
    fn parent(&self, node: NodeId) -> Option<NodeId>;
}

/// A single row in the flattened visible-row list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisibleRow {
    /// Node id in the tree data.
    pub node_id: NodeId,
    /// Depth in the tree (0 for roots).
    pub depth: usize,
    /// Whether this node has children.
    pub has_children: bool,
    /// Whether this node is currently expanded.
    pub is_expanded: bool,
    /// Whether this node is the last sibling at its level.
    pub is_last_sibling: bool,
    /// Ancestor "last sibling" flags for drawing guide lines.
    /// `ancestors_last[i]` is true when the ancestor at depth `i` is the last
    /// sibling at its level (so the guide pipe should be blank, not │).
    pub ancestors_last: Vec<bool>,
}

/// Compute the flat list of visible rows by walking the tree top-down.
///
/// Nodes whose parent is collapsed are excluded. The walk is depth-first.
#[must_use]
pub fn compute_visible_rows(
    data: &dyn TreeData,
    expanded: &std::collections::BTreeSet<NodeId>,
) -> Vec<VisibleRow> {
    let mut walker = VisibleRowWalker::new(data, expanded);
    let root_count = walker.data.root_count();
    for index in 0..root_count {
        let node = walker.data.root(index);
        let is_last_sibling = index.saturating_add(1) == root_count;
        walker.walk_node(node, 0, is_last_sibling, &[]);
    }
    walker.into_rows()
}

struct VisibleRowWalker<'a> {
    data: &'a dyn TreeData,
    expanded: &'a std::collections::BTreeSet<NodeId>,
    rows: Vec<VisibleRow>,
}

impl<'a> VisibleRowWalker<'a> {
    fn new(data: &'a dyn TreeData, expanded: &'a std::collections::BTreeSet<NodeId>) -> Self {
        Self {
            data,
            expanded,
            rows: Vec::new(),
        }
    }

    fn into_rows(self) -> Vec<VisibleRow> {
        self.rows
    }

    fn walk_node(
        &mut self,
        node: NodeId,
        depth: usize,
        is_last_sibling: bool,
        ancestors_last: &[bool],
    ) {
        let child_count = self.data.child_count(node);
        let has_children = child_count > 0;
        let is_expanded = has_children && self.expanded.contains(&node);

        self.rows.push(VisibleRow {
            node_id: node,
            depth,
            has_children,
            is_expanded,
            is_last_sibling,
            ancestors_last: ancestors_last.to_vec(),
        });

        if is_expanded {
            let mut child_ancestors = ancestors_last.to_vec();
            child_ancestors.push(is_last_sibling);
            for child_index in 0..child_count {
                let child = self.data.child(node, child_index);
                let is_child_last = child_index.saturating_add(1) == child_count;
                self.walk_node(
                    child,
                    depth.saturating_add(1),
                    is_child_last,
                    &child_ancestors,
                );
            }
        }
    }
}

// ── SimpleTree adapter ──────────────────────────────────────────────────────

/// A simple tree built from flat `(id, parent_id, label)` data.
///
/// Stores children per node in a `std::collections::BTreeMap` for deterministic order.
pub struct SimpleTree {
    roots: Vec<NodeId>,
    children: std::collections::BTreeMap<NodeId, Vec<NodeId>>,
    parents: std::collections::BTreeMap<NodeId, NodeId>,
    labels: std::collections::BTreeMap<NodeId, String>,
}

impl SimpleTree {
    /// Build a tree from `(id, parent_id, label)` tuples.
    ///
    /// Entries with `parent_id = None` become roots. Children are ordered by
    /// their position in the input vec.
    #[must_use]
    pub fn new(entries: Vec<(NodeId, Option<NodeId>, String)>) -> Self {
        let entry_count = entries.len();
        let mut roots = Vec::with_capacity(entry_count);
        let mut children: std::collections::BTreeMap<NodeId, Vec<NodeId>> =
            std::collections::BTreeMap::new();
        let parents = entries
            .iter()
            .filter_map(|(id, parent_id, _label)| parent_id.map(|pid| (*id, pid)))
            .collect();
        let labels = entries
            .iter()
            .map(|(id, _parent_id, label)| (*id, label.clone()))
            .collect();

        for (id, parent_id, _label) in entries {
            match parent_id {
                Some(pid) => {
                    children.entry(pid).or_default().push(id);
                }
                None => {
                    roots.push(id);
                }
            }
        }

        Self {
            roots,
            children,
            parents,
            labels,
        }
    }
}

impl TreeData for SimpleTree {
    fn root_count(&self) -> usize {
        self.roots.len()
    }

    fn root(&self, index: usize) -> NodeId {
        self.roots[index]
    }

    fn child_count(&self, node: NodeId) -> usize {
        self.children.get(&node).map_or(0, Vec::len)
    }

    fn child(&self, node: NodeId, index: usize) -> NodeId {
        self.children[&node][index]
    }

    fn node_label(&self, node: NodeId) -> &str {
        &self.labels[&node]
    }

    fn parent(&self, node: NodeId) -> Option<NodeId> {
        self.parents.get(&node).copied()
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

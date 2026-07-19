//! Auto-layout algorithms and snap-to-grid positioning for node graphs.
//!
//! Provides a Sugiyama-style layered layout that assigns ranks (depth) via
//! topological sort, orders nodes within ranks to reduce edge crossings,
//! and assigns positions with configurable spacing.

use crate::model::{Graph, NodeId};
use std::collections::{HashMap, HashSet, VecDeque};

/// Layout flow direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutDirection {
    LeftToRight,
    TopToBottom,
}

/// Sentinel position that marks a node as "unpositioned" for partial layout.
pub const UNPOSITIONED: (i32, i32) = (i32::MIN, i32::MIN);

/// Configuration for the auto-layout algorithm.
#[derive(Debug, Clone)]
pub struct LayoutConfig {
    pub direction: LayoutDirection,
    /// Minimum gap (columns) between a node's right edge and the next node's left edge.
    pub h_spacing: u16,
    /// Minimum gap (rows) between a node's bottom edge and the next node's top edge.
    pub v_spacing: u16,
    /// Snap grid cell size. 0 means no snapping.
    pub grid_size: u16,
    /// Estimated node width in columns (used for spacing calculations).
    pub node_width: u16,
    /// Estimated node height in rows (used for spacing calculations).
    pub node_height: u16,
    /// If true, skip nodes that are already positioned (not at UNPOSITIONED sentinel).
    pub partial: bool,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            direction: LayoutDirection::LeftToRight,
            h_spacing: 6,
            v_spacing: 2,
            grid_size: 0,
            node_width: 20,
            node_height: 5,
            partial: false,
        }
    }
}

/// Snap a coordinate to the nearest grid boundary.
pub fn snap_to_grid(value: i32, grid_size: u16) -> i32 {
    if grid_size == 0 {
        return value;
    }
    let gs = grid_size as i32;
    let remainder = value.rem_euclid(gs);
    if remainder <= gs / 2 {
        value - remainder
    } else {
        value - remainder + gs
    }
}

/// Run auto-layout on the graph, assigning positions to nodes.
///
/// This modifies node positions in-place. If `config.partial` is true, only
/// nodes at the `UNPOSITIONED` sentinel are moved; pinned nodes keep their
/// positions but are still used for rank assignment.
pub fn auto_layout(graph: &mut Graph, config: &LayoutConfig) {
    let node_ids = graph.node_ids();
    if node_ids.is_empty() {
        return;
    }

    // Determine which nodes to lay out.
    let pinned: HashSet<NodeId> = if config.partial {
        node_ids
            .iter()
            .filter(|&&id| {
                let n = graph.node(id).unwrap();
                (n.x, n.y) != UNPOSITIONED
            })
            .copied()
            .collect()
    } else {
        HashSet::new()
    };

    // Build adjacency: node → set of successor nodes.
    let mut successors: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
    let mut in_degree: HashMap<NodeId, usize> = HashMap::new();

    for &id in &node_ids {
        successors.entry(id).or_default();
        in_degree.entry(id).or_insert(0);
    }

    for edge in graph.edges() {
        let src_node = graph.port_owner(edge.source);
        let tgt_node = graph.port_owner(edge.target);
        if let (Some(s), Some(t)) = (src_node, tgt_node) {
            successors.entry(s).or_default().push(t);
            *in_degree.entry(t).or_insert(0) += 1;
        }
    }

    // -- rank assignment via topological BFS (Kahn's algorithm) -------------

    let mut ranks: HashMap<NodeId, usize> = HashMap::new();
    let mut queue: VecDeque<NodeId> = VecDeque::new();

    for (&id, &deg) in &in_degree {
        if deg == 0 {
            queue.push_back(id);
            ranks.insert(id, 0);
        }
    }

    // Handle cycles: if no roots found, pick the first node as root.
    if queue.is_empty() {
        let first = node_ids[0];
        queue.push_back(first);
        ranks.insert(first, 0);
    }

    while let Some(current) = queue.pop_front() {
        let current_rank = ranks[&current];
        if let Some(succs) = successors.get(&current) {
            for &s in succs {
                let new_rank = current_rank + 1;
                let existing = ranks.entry(s).or_insert(0);
                if new_rank > *existing {
                    *existing = new_rank;
                }
                let deg = in_degree.get_mut(&s).unwrap();
                *deg = deg.saturating_sub(1);
                if *deg == 0 {
                    queue.push_back(s);
                }
            }
        }
    }

    // Assign rank 0 to any remaining unranked nodes (disconnected).
    for &id in &node_ids {
        ranks.entry(id).or_insert(0);
    }

    // -- order within ranks (barycenter heuristic) --------------------------

    let max_rank = ranks.values().copied().max().unwrap_or(0);
    let mut rank_buckets: Vec<Vec<NodeId>> = vec![Vec::new(); max_rank + 1];
    for (&id, &rank) in &ranks {
        rank_buckets[rank].push(id);
    }

    // Sort nodes within each rank by the average position of their predecessors.
    // This is a single-pass barycenter — good enough for most graphs.
    for rank_idx in 1..=max_rank {
        let prev_order: HashMap<NodeId, usize> = rank_buckets[rank_idx - 1]
            .iter()
            .enumerate()
            .map(|(i, &id)| (id, i))
            .collect();

        let mut bary: Vec<(NodeId, f64)> = rank_buckets[rank_idx]
            .iter()
            .map(|&id| {
                // Find predecessors in the previous rank.
                let mut pred_positions = Vec::new();
                for edge in graph.edges() {
                    let tgt_node = graph.port_owner(edge.target);
                    let src_node = graph.port_owner(edge.source);
                    if tgt_node == Some(id)
                        && let Some(sn) = src_node
                        && let Some(&pos) = prev_order.get(&sn)
                    {
                        pred_positions.push(pos as f64);
                    }
                }
                let avg = if pred_positions.is_empty() {
                    0.0
                } else {
                    pred_positions.iter().sum::<f64>() / pred_positions.len() as f64
                };
                (id, avg)
            })
            .collect();

        bary.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        rank_buckets[rank_idx] = bary.into_iter().map(|(id, _)| id).collect();
    }

    // -- position assignment ------------------------------------------------

    let rank_step = match config.direction {
        LayoutDirection::LeftToRight => (config.node_width + config.h_spacing) as i32,
        LayoutDirection::TopToBottom => (config.node_height + config.v_spacing) as i32,
    };

    let cross_step = match config.direction {
        LayoutDirection::LeftToRight => (config.node_height + config.v_spacing) as i32,
        LayoutDirection::TopToBottom => (config.node_width + config.h_spacing) as i32,
    };

    // Track where disconnected subgraphs start to avoid overlap.
    let mut component_offset: i32 = 0;

    // Find connected components and lay them out separately.
    let components = find_components(&node_ids, &successors, graph);

    for component in &components {
        // Get the max cross-axis span of this component.
        let mut max_cross: i32 = 0;

        for &node_id in component {
            if pinned.contains(&node_id) {
                continue;
            }

            let rank = ranks[&node_id];
            let order_in_rank = rank_buckets[rank]
                .iter()
                .position(|&id| id == node_id)
                .unwrap_or(0);

            let (x, y) = match config.direction {
                LayoutDirection::LeftToRight => (
                    rank as i32 * rank_step,
                    order_in_rank as i32 * cross_step + component_offset,
                ),
                LayoutDirection::TopToBottom => (
                    order_in_rank as i32 * cross_step + component_offset,
                    rank as i32 * rank_step,
                ),
            };

            let (x, y) = if config.grid_size > 0 {
                (
                    snap_to_grid(x, config.grid_size),
                    snap_to_grid(y, config.grid_size),
                )
            } else {
                (x, y)
            };

            if let Some(node) = graph.node_mut(node_id) {
                node.x = x;
                node.y = y;
            }

            let cross_pos = match config.direction {
                LayoutDirection::LeftToRight => y + config.node_height as i32,
                LayoutDirection::TopToBottom => x + config.node_width as i32,
            };
            if cross_pos > max_cross {
                max_cross = cross_pos;
            }
        }

        // Offset next component past this one.
        component_offset = max_cross + cross_step;
    }

    // Snap pinned nodes if grid is enabled (partial mode still snaps on request).
    if config.grid_size > 0 && !config.partial {
        for &id in &pinned {
            if let Some(node) = graph.node_mut(id) {
                node.x = snap_to_grid(node.x, config.grid_size);
                node.y = snap_to_grid(node.y, config.grid_size);
            }
        }
    }
}

/// Find connected components (treating edges as undirected for grouping).
fn find_components(
    node_ids: &[NodeId],
    successors: &HashMap<NodeId, Vec<NodeId>>,
    _graph: &Graph,
) -> Vec<Vec<NodeId>> {
    // Build undirected adjacency.
    let mut adj: HashMap<NodeId, HashSet<NodeId>> = HashMap::new();
    for &id in node_ids {
        adj.entry(id).or_default();
    }
    for (&src, targets) in successors {
        for &tgt in targets {
            adj.entry(src).or_default().insert(tgt);
            adj.entry(tgt).or_default().insert(src);
        }
    }

    let mut visited: HashSet<NodeId> = HashSet::new();
    let mut components = Vec::new();

    for &id in node_ids {
        if visited.contains(&id) {
            continue;
        }
        let mut component = Vec::new();
        let mut stack = vec![id];
        while let Some(current) = stack.pop() {
            if !visited.insert(current) {
                continue;
            }
            component.push(current);
            if let Some(neighbors) = adj.get(&current) {
                for &n in neighbors {
                    if !visited.contains(&n) {
                        stack.push(n);
                    }
                }
            }
        }
        components.push(component);
    }

    components
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::PortSpec;

    fn sp(label: &str) -> PortSpec {
        PortSpec::new(label, "string")
    }

    fn build_linear_graph() -> (Graph, NodeId, NodeId, NodeId) {
        let mut g = Graph::new();
        let a = g.add_node("A", &[sp("in")], &[sp("out")]);
        let b = g.add_node("B", &[sp("in")], &[sp("out")]);
        let c = g.add_node("C", &[sp("in")], &[]);

        let a_out = g.node(a).unwrap().output_ports[0].id;
        let b_in = g.node(b).unwrap().input_ports[0].id;
        let b_out = g.node(b).unwrap().output_ports[0].id;
        let c_in = g.node(c).unwrap().input_ports[0].id;

        g.add_edge(a_out, b_in).unwrap();
        g.add_edge(b_out, c_in).unwrap();

        (g, a, b, c)
    }

    #[test]
    fn left_to_right_layout() {
        let (mut g, a, b, c) = build_linear_graph();
        auto_layout(&mut g, &LayoutConfig::default());

        let na = g.node(a).unwrap();
        let nb = g.node(b).unwrap();
        let nc = g.node(c).unwrap();

        // A should be leftmost, C rightmost.
        assert!(na.x < nb.x, "A.x={} should be < B.x={}", na.x, nb.x);
        assert!(nb.x < nc.x, "B.x={} should be < C.x={}", nb.x, nc.x);
    }

    #[test]
    fn top_to_bottom_layout() {
        let (mut g, a, b, c) = build_linear_graph();
        let config = LayoutConfig {
            direction: LayoutDirection::TopToBottom,
            ..Default::default()
        };
        auto_layout(&mut g, &config);

        let na = g.node(a).unwrap();
        let nb = g.node(b).unwrap();
        let nc = g.node(c).unwrap();

        assert!(na.y < nb.y, "A.y={} should be < B.y={}", na.y, nb.y);
        assert!(nb.y < nc.y, "B.y={} should be < C.y={}", nb.y, nc.y);
    }

    #[test]
    fn disconnected_subgraphs_no_overlap() {
        let mut g = Graph::new();
        let a = g.add_node("A", &[], &[sp("out")]);
        let b = g.add_node("B", &[sp("in")], &[]);
        let c = g.add_node("C", &[], &[sp("out")]);
        let d = g.add_node("D", &[sp("in")], &[]);

        let a_out = g.node(a).unwrap().output_ports[0].id;
        let b_in = g.node(b).unwrap().input_ports[0].id;
        let c_out = g.node(c).unwrap().output_ports[0].id;
        let d_in = g.node(d).unwrap().input_ports[0].id;

        g.add_edge(a_out, b_in).unwrap();
        g.add_edge(c_out, d_in).unwrap();

        auto_layout(&mut g, &LayoutConfig::default());

        // Check no two nodes overlap (same position).
        let positions: Vec<(i32, i32)> = g.nodes().map(|n| (n.x, n.y)).collect();
        for (i, p1) in positions.iter().enumerate() {
            for p2 in positions.iter().skip(i + 1) {
                assert_ne!(p1, p2, "two nodes share position {:?}", p1);
            }
        }
    }

    #[test]
    fn snap_to_grid_rounds() {
        assert_eq!(snap_to_grid(13, 4), 12);
        assert_eq!(snap_to_grid(7, 4), 8);
        assert_eq!(snap_to_grid(12, 4), 12);
        assert_eq!(snap_to_grid(11, 4), 12);
        assert_eq!(snap_to_grid(5, 4), 4);
    }

    #[test]
    fn snap_disabled() {
        assert_eq!(snap_to_grid(13, 0), 13);
        assert_eq!(snap_to_grid(7, 0), 7);
    }

    #[test]
    fn snap_after_auto_layout() {
        let (mut g, _a, _b, _c) = build_linear_graph();
        let config = LayoutConfig {
            grid_size: 4,
            ..Default::default()
        };
        auto_layout(&mut g, &config);

        // All positions should be multiples of 4.
        for node in g.nodes() {
            assert_eq!(
                node.x % 4,
                0,
                "node {} x={} not on grid",
                node.label,
                node.x
            );
            assert_eq!(
                node.y % 4,
                0,
                "node {} y={} not on grid",
                node.label,
                node.y
            );
        }
    }

    #[test]
    fn default_spacing_respected() {
        let (mut g, a, b, _c) = build_linear_graph();
        let config = LayoutConfig::default();
        auto_layout(&mut g, &config);

        let na = g.node(a).unwrap();
        let nb = g.node(b).unwrap();

        let gap = nb.x - na.x;
        // Gap should be at least node_width + h_spacing.
        assert!(
            gap >= (config.node_width + config.h_spacing) as i32,
            "gap={gap}, expected >= {}",
            config.node_width + config.h_spacing
        );
    }

    #[test]
    fn custom_spacing() {
        let (mut g, a, b, _c) = build_linear_graph();
        let config = LayoutConfig {
            h_spacing: 10,
            ..Default::default()
        };
        auto_layout(&mut g, &config);

        let na = g.node(a).unwrap();
        let nb = g.node(b).unwrap();

        let gap = nb.x - na.x;
        assert!(
            gap >= (config.node_width + 10) as i32,
            "gap={gap}, expected >= {}",
            config.node_width + 10
        );
    }

    #[test]
    fn partial_layout_skips_pinned() {
        let (mut g, a, b, _c) = build_linear_graph();

        // Pin node A.
        g.node_mut(a).unwrap().x = 5;
        g.node_mut(a).unwrap().y = 5;

        // Mark B as unpositioned.
        g.node_mut(b).unwrap().x = UNPOSITIONED.0;
        g.node_mut(b).unwrap().y = UNPOSITIONED.1;

        let config = LayoutConfig {
            partial: true,
            ..Default::default()
        };
        auto_layout(&mut g, &config);

        // A should remain at (5, 5).
        let na = g.node(a).unwrap();
        assert_eq!((na.x, na.y), (5, 5));

        // B should have been repositioned (no longer UNPOSITIONED).
        let nb = g.node(b).unwrap();
        assert_ne!((nb.x, nb.y), UNPOSITIONED);
    }
}

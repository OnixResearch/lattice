//! Graph data model: nodes with typed ports, directed edges, and validation.
//!
//! This module contains no ratatui dependency. It's a pure data structure
//! for directed graphs with typed input/output ports on each node.

use std::collections::{BTreeMap, HashMap};
use std::fmt;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Identity types
// ---------------------------------------------------------------------------

/// Unique node identifier. Monotonically assigned by the graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct NodeId(pub u64);

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "node:{}", self.0)
    }
}

/// Unique port identifier. Monotonically assigned by the graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PortId(pub u64);

impl fmt::Display for PortId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "port:{}", self.0)
    }
}

// ---------------------------------------------------------------------------
// Port
// ---------------------------------------------------------------------------

/// Port direction: input (left side) or output (right side).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum PortDirection {
    Input,
    Output,
}

/// A typed connection point on a node.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Port {
    pub id: PortId,
    pub label: String,
    pub type_tag: String,
    pub direction: PortDirection,
}

/// Descriptor for creating a port (before ID assignment).
#[derive(Debug, Clone)]
pub struct PortSpec {
    pub label: String,
    pub type_tag: String,
}

impl PortSpec {
    pub fn new(label: impl Into<String>, type_tag: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            type_tag: type_tag.into(),
        }
    }
}

// ---------------------------------------------------------------------------
// Node
// ---------------------------------------------------------------------------

/// A node in the graph with typed input and output ports.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Node {
    pub id: NodeId,
    pub label: String,
    pub x: i32,
    pub y: i32,
    pub input_ports: Vec<Port>,
    pub output_ports: Vec<Port>,
}

// ---------------------------------------------------------------------------
// Edge
// ---------------------------------------------------------------------------

/// A directed edge from an output port to an input port.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Edge {
    pub source: PortId,
    pub target: PortId,
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Errors from graph mutation operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GraphError {
    /// Tried to connect ports on the same node.
    SelfLoop,
    /// Port types are not compatible.
    IncompatibleTypes {
        source_tag: String,
        target_tag: String,
    },
    /// Adding the edge would create a cycle (DAG mode).
    WouldCycle,
    /// One or both port IDs not found in the graph.
    PortNotFound(PortId),
    /// Source port is not an output or target is not an input.
    WrongDirection,
    /// Node not found.
    NodeNotFound(NodeId),
    /// Edge not found.
    EdgeNotFound { source: PortId, target: PortId },
}

impl fmt::Display for GraphError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SelfLoop => write!(f, "self-loops are not allowed"),
            Self::IncompatibleTypes {
                source_tag,
                target_tag,
            } => {
                write!(
                    f,
                    "incompatible port types: {source_tag:?} → {target_tag:?}"
                )
            }
            Self::WouldCycle => write!(f, "edge would create a cycle (DAG mode)"),
            Self::PortNotFound(id) => write!(f, "port {id} not found"),
            Self::WrongDirection => {
                write!(f, "source must be Output, target must be Input")
            }
            Self::NodeNotFound(id) => write!(f, "node {id} not found"),
            Self::EdgeNotFound { source, target } => {
                write!(f, "edge {source} → {target} not found")
            }
        }
    }
}

impl std::error::Error for GraphError {}

// ---------------------------------------------------------------------------
// Compatibility function
// ---------------------------------------------------------------------------

/// Type alias for the port-type compatibility checker.
///
/// Takes `(output_type_tag, input_type_tag)` and returns whether they can connect.
pub type CompatibilityFn = Box<dyn Fn(&str, &str) -> bool + Send + Sync>;

/// Default compatibility: exact string match.
pub fn default_compatibility(source: &str, target: &str) -> bool {
    source == target
}

// ---------------------------------------------------------------------------
// Graph
// ---------------------------------------------------------------------------

/// Directed graph with typed ports and optional DAG enforcement.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Graph {
    nodes: BTreeMap<NodeId, Node>,
    edges: Vec<Edge>,
    next_node_id: u64,
    next_port_id: u64,
    pub dag_mode: bool,

    /// Port-to-node lookup for fast queries.
    port_owner: HashMap<PortId, NodeId>,

    #[cfg_attr(feature = "serde", serde(skip))]
    compat_fn: Option<CompatibilityFn>,
}

impl fmt::Debug for Graph {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Graph")
            .field("nodes", &self.nodes.len())
            .field("edges", &self.edges.len())
            .field("dag_mode", &self.dag_mode)
            .finish()
    }
}

impl Clone for Graph {
    fn clone(&self) -> Self {
        Self {
            nodes: self.nodes.clone(),
            edges: self.edges.clone(),
            next_node_id: self.next_node_id,
            next_port_id: self.next_port_id,
            dag_mode: self.dag_mode,
            port_owner: self.port_owner.clone(),
            compat_fn: None, // can't clone closures; caller re-sets after clone
        }
    }
}

impl Graph {
    /// Create an empty graph. DAG mode is off by default.
    pub fn new() -> Self {
        Self {
            nodes: BTreeMap::new(),
            edges: Vec::new(),
            next_node_id: 0,
            next_port_id: 0,
            dag_mode: false,
            port_owner: HashMap::new(),
            compat_fn: None,
        }
    }

    /// Set a custom compatibility function. Replaces the default (exact match).
    pub fn set_compatibility(&mut self, f: CompatibilityFn) {
        self.compat_fn = Some(f);
    }

    fn check_compat(&self, source_tag: &str, target_tag: &str) -> bool {
        match &self.compat_fn {
            Some(f) => f(source_tag, target_tag),
            None => default_compatibility(source_tag, target_tag),
        }
    }

    // -- node operations ----------------------------------------------------

    /// Add a node with the given label, input port specs, and output port specs.
    /// Returns the assigned `NodeId`.
    pub fn add_node(
        &mut self,
        label: impl Into<String>,
        inputs: &[PortSpec],
        outputs: &[PortSpec],
    ) -> NodeId {
        let node_id = NodeId(self.next_node_id);
        self.next_node_id += 1;

        let input_ports: Vec<Port> = inputs
            .iter()
            .map(|spec| {
                let port = Port {
                    id: PortId(self.next_port_id),
                    label: spec.label.clone(),
                    type_tag: spec.type_tag.clone(),
                    direction: PortDirection::Input,
                };
                self.next_port_id += 1;
                port
            })
            .collect();

        let output_ports: Vec<Port> = outputs
            .iter()
            .map(|spec| {
                let port = Port {
                    id: PortId(self.next_port_id),
                    label: spec.label.clone(),
                    type_tag: spec.type_tag.clone(),
                    direction: PortDirection::Output,
                };
                self.next_port_id += 1;
                port
            })
            .collect();

        // Register port ownership.
        for p in input_ports.iter().chain(output_ports.iter()) {
            self.port_owner.insert(p.id, node_id);
        }

        let node = Node {
            id: node_id,
            label: label.into(),
            x: 0,
            y: 0,
            input_ports,
            output_ports,
        };

        self.nodes.insert(node_id, node);
        node_id
    }

    /// Remove a node and all edges connected to its ports.
    pub fn remove_node(&mut self, id: NodeId) -> Result<(), GraphError> {
        let node = self.nodes.remove(&id).ok_or(GraphError::NodeNotFound(id))?;

        // Collect all port IDs belonging to this node.
        let port_ids: Vec<PortId> = node
            .input_ports
            .iter()
            .chain(node.output_ports.iter())
            .map(|p| p.id)
            .collect();

        // Remove edges touching any of those ports.
        self.edges
            .retain(|e| !port_ids.contains(&e.source) && !port_ids.contains(&e.target));

        // Remove port ownership entries.
        for pid in &port_ids {
            self.port_owner.remove(pid);
        }

        Ok(())
    }

    // -- edge operations ----------------------------------------------------

    /// Add a directed edge from `source` (must be Output) to `target` (must be Input).
    ///
    /// Validates direction, self-loop, type compatibility, and (if dag_mode) cycles.
    pub fn add_edge(&mut self, source: PortId, target: PortId) -> Result<(), GraphError> {
        let source_port = self
            .find_port(source)
            .ok_or(GraphError::PortNotFound(source))?;
        let target_port = self
            .find_port(target)
            .ok_or(GraphError::PortNotFound(target))?;

        // Direction check.
        if source_port.direction != PortDirection::Output
            || target_port.direction != PortDirection::Input
        {
            return Err(GraphError::WrongDirection);
        }

        // Self-loop check.
        let source_node = self.port_owner[&source];
        let target_node = self.port_owner[&target];
        if source_node == target_node {
            return Err(GraphError::SelfLoop);
        }

        // Type compatibility.
        if !self.check_compat(&source_port.type_tag, &target_port.type_tag) {
            return Err(GraphError::IncompatibleTypes {
                source_tag: source_port.type_tag.clone(),
                target_tag: target_port.type_tag.clone(),
            });
        }

        // Cycle detection (DAG mode).
        if self.dag_mode && self.would_cycle(target_node, source_node) {
            return Err(GraphError::WouldCycle);
        }

        self.edges.push(Edge { source, target });
        Ok(())
    }

    /// Remove a single edge.
    pub fn remove_edge(&mut self, source: PortId, target: PortId) -> Result<(), GraphError> {
        let idx = self
            .edges
            .iter()
            .position(|e| e.source == source && e.target == target)
            .ok_or(GraphError::EdgeNotFound { source, target })?;
        self.edges.remove(idx);
        Ok(())
    }

    // -- cycle detection ----------------------------------------------------

    /// Returns true if adding an edge from `from_node` to `to_node` would create a cycle.
    ///
    /// Checks whether `to_node` can already reach `from_node` via existing edges.
    fn would_cycle(&self, from_node: NodeId, to_node: NodeId) -> bool {
        // DFS from `from_node` following outgoing edges; if we reach `to_node`, it's a cycle.
        let mut visited = Vec::new();
        let mut stack = vec![from_node];

        while let Some(current) = stack.pop() {
            if current == to_node {
                return true;
            }
            if visited.contains(&current) {
                continue;
            }
            visited.push(current);

            // Find all nodes reachable from `current` via outgoing edges.
            if let Some(node) = self.nodes.get(&current) {
                let out_port_ids: Vec<PortId> = node.output_ports.iter().map(|p| p.id).collect();
                for edge in &self.edges {
                    if out_port_ids.contains(&edge.source)
                        && let Some(&neighbor) = self.port_owner.get(&edge.target)
                    {
                        stack.push(neighbor);
                    }
                }
            }
        }

        false
    }

    // -- queries ------------------------------------------------------------

    /// Iterate over all nodes.
    pub fn nodes(&self) -> impl Iterator<Item = &Node> {
        self.nodes.values()
    }

    /// Get a node by ID.
    pub fn node(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(&id)
    }

    /// Get a mutable node by ID (for position updates, etc.).
    pub fn node_mut(&mut self, id: NodeId) -> Option<&mut Node> {
        self.nodes.get_mut(&id)
    }

    /// Iterate over all edges.
    pub fn edges(&self) -> &[Edge] {
        &self.edges
    }

    /// Get all edges connected to a node (incoming and outgoing).
    pub fn edges_for_node(&self, id: NodeId) -> Vec<&Edge> {
        let Some(node) = self.nodes.get(&id) else {
            return Vec::new();
        };
        let port_ids: Vec<PortId> = node
            .input_ports
            .iter()
            .chain(node.output_ports.iter())
            .map(|p| p.id)
            .collect();
        self.edges
            .iter()
            .filter(|e| port_ids.contains(&e.source) || port_ids.contains(&e.target))
            .collect()
    }

    /// Find a port by ID across all nodes. Returns a clone of the port data.
    pub fn port(&self, id: PortId) -> Option<&Port> {
        let node_id = self.port_owner.get(&id)?;
        let node = self.nodes.get(node_id)?;
        node.input_ports
            .iter()
            .chain(node.output_ports.iter())
            .find(|p| p.id == id)
    }

    /// Find which node owns a port.
    pub fn port_owner(&self, id: PortId) -> Option<NodeId> {
        self.port_owner.get(&id).copied()
    }

    /// Number of nodes.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Number of edges.
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Collect all node IDs, sorted by ID (insertion order).
    pub fn node_ids(&self) -> Vec<NodeId> {
        self.nodes.keys().copied().collect()
    }

    // -- internal helpers ---------------------------------------------------

    fn find_port(&self, id: PortId) -> Option<&Port> {
        self.port(id)
    }

    /// Restore the ID counters after deserialization by scanning existing IDs.
    pub fn restore_id_counters(&mut self) {
        let max_node = self.nodes.keys().map(|k| k.0).max().unwrap_or(0);
        let max_port = self
            .nodes
            .values()
            .flat_map(|n| n.input_ports.iter().chain(n.output_ports.iter()))
            .map(|p| p.id.0)
            .max()
            .unwrap_or(0);

        self.next_node_id = max_node + 1;
        self.next_port_id = max_port + 1;

        // Rebuild port_owner map.
        self.port_owner.clear();
        for node in self.nodes.values() {
            for p in node.input_ports.iter().chain(node.output_ports.iter()) {
                self.port_owner.insert(p.id, node.id);
            }
        }
    }
}

impl Default for Graph {
    fn default() -> Self {
        Self::new()
    }
}

/// After deserializing, restore internal counters and lookup tables.
#[cfg(feature = "serde")]
impl<'de> Graph {
    /// Deserialize and restore internal state.
    pub fn from_json(json: &'de str) -> Result<Self, serde_json::Error> {
        let mut graph: Graph = serde_json::from_str(json)?;
        graph.restore_id_counters();
        Ok(graph)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn string_port(label: &str) -> PortSpec {
        PortSpec::new(label, "string")
    }

    fn number_port(label: &str) -> PortSpec {
        PortSpec::new(label, "number")
    }

    // -- node creation ------------------------------------------------------

    #[test]
    fn create_node_with_ports() {
        let mut g = Graph::new();
        let id = g.add_node(
            "HTTP Request",
            &[string_port("url"), string_port("headers")],
            &[string_port("response")],
        );

        let node = g.node(id).unwrap();
        assert_eq!(node.label, "HTTP Request");
        assert_eq!(node.input_ports.len(), 2);
        assert_eq!(node.output_ports.len(), 1);
        assert_eq!(node.input_ports[0].label, "url");
        assert_eq!(node.input_ports[1].label, "headers");
        assert_eq!(node.output_ports[0].label, "response");
    }

    #[test]
    fn node_ids_monotonically_unique() {
        let mut g = Graph::new();
        let a = g.add_node("A", &[], &[]);
        let b = g.add_node("B", &[], &[]);
        let c = g.add_node("C", &[], &[]);

        assert_ne!(a, b);
        assert_ne!(b, c);
        assert_ne!(a, c);
    }

    #[test]
    fn port_ids_distinct_across_nodes() {
        let mut g = Graph::new();
        let a = g.add_node("A", &[string_port("in")], &[string_port("out")]);
        let b = g.add_node("B", &[string_port("in")], &[string_port("out")]);

        let na = g.node(a).unwrap();
        let nb = g.node(b).unwrap();

        let all_ids: Vec<PortId> = na
            .input_ports
            .iter()
            .chain(na.output_ports.iter())
            .chain(nb.input_ports.iter())
            .chain(nb.output_ports.iter())
            .map(|p| p.id)
            .collect();

        let unique: std::collections::HashSet<_> = all_ids.iter().collect();
        assert_eq!(all_ids.len(), unique.len());
    }

    // -- edge creation ------------------------------------------------------

    #[test]
    fn create_valid_edge() {
        let mut g = Graph::new();
        let a = g.add_node("A", &[], &[string_port("out")]);
        let b = g.add_node("B", &[string_port("in")], &[]);

        let src = g.node(a).unwrap().output_ports[0].id;
        let tgt = g.node(b).unwrap().input_ports[0].id;

        assert!(g.add_edge(src, tgt).is_ok());
        assert_eq!(g.edge_count(), 1);

        let a_edges = g.edges_for_node(a);
        assert_eq!(a_edges.len(), 1);
        let b_edges = g.edges_for_node(b);
        assert_eq!(b_edges.len(), 1);
    }

    #[test]
    fn reject_self_loop() {
        let mut g = Graph::new();
        let a = g.add_node("A", &[string_port("in")], &[string_port("out")]);

        let src = g.node(a).unwrap().output_ports[0].id;
        let tgt = g.node(a).unwrap().input_ports[0].id;

        let result = g.add_edge(src, tgt);
        assert_eq!(result, Err(GraphError::SelfLoop));
        assert_eq!(g.edge_count(), 0);
    }

    // -- type compatibility -------------------------------------------------

    #[test]
    fn compatible_types_connect() {
        let mut g = Graph::new();
        let a = g.add_node("A", &[], &[string_port("out")]);
        let b = g.add_node("B", &[string_port("in")], &[]);

        let src = g.node(a).unwrap().output_ports[0].id;
        let tgt = g.node(b).unwrap().input_ports[0].id;

        assert!(g.add_edge(src, tgt).is_ok());
    }

    #[test]
    fn incompatible_types_rejected() {
        let mut g = Graph::new();
        let a = g.add_node("A", &[], &[number_port("out")]);
        let b = g.add_node("B", &[string_port("in")], &[]);

        let src = g.node(a).unwrap().output_ports[0].id;
        let tgt = g.node(b).unwrap().input_ports[0].id;

        let result = g.add_edge(src, tgt);
        assert!(matches!(result, Err(GraphError::IncompatibleTypes { .. })));
        assert_eq!(g.edge_count(), 0);
    }

    #[test]
    fn custom_compatibility_function() {
        let mut g = Graph::new();
        g.set_compatibility(Box::new(|src, _tgt| src == "any"));

        let a = g.add_node("A", &[], &[PortSpec::new("out", "any")]);
        let b = g.add_node("B", &[string_port("in")], &[]);

        let src = g.node(a).unwrap().output_ports[0].id;
        let tgt = g.node(b).unwrap().input_ports[0].id;

        assert!(g.add_edge(src, tgt).is_ok());
    }

    // -- DAG cycle detection ------------------------------------------------

    #[test]
    fn cycle_rejected_in_dag_mode() {
        let mut g = Graph::new();
        g.dag_mode = true;

        let a = g.add_node("A", &[string_port("in")], &[string_port("out")]);
        let b = g.add_node("B", &[string_port("in")], &[string_port("out")]);
        let c = g.add_node("C", &[string_port("in")], &[string_port("out")]);

        // A → B → C
        let a_out = g.node(a).unwrap().output_ports[0].id;
        let b_in = g.node(b).unwrap().input_ports[0].id;
        let b_out = g.node(b).unwrap().output_ports[0].id;
        let c_in = g.node(c).unwrap().input_ports[0].id;
        let c_out = g.node(c).unwrap().output_ports[0].id;
        let a_in = g.node(a).unwrap().input_ports[0].id;

        assert!(g.add_edge(a_out, b_in).is_ok());
        assert!(g.add_edge(b_out, c_in).is_ok());

        // C → A would create a cycle
        let result = g.add_edge(c_out, a_in);
        assert_eq!(result, Err(GraphError::WouldCycle));
        assert_eq!(g.edge_count(), 2);
    }

    #[test]
    fn cycle_allowed_without_dag_mode() {
        let mut g = Graph::new();
        g.dag_mode = false;

        let a = g.add_node("A", &[string_port("in")], &[string_port("out")]);
        let b = g.add_node("B", &[string_port("in")], &[string_port("out")]);
        let c = g.add_node("C", &[string_port("in")], &[string_port("out")]);

        let a_out = g.node(a).unwrap().output_ports[0].id;
        let b_in = g.node(b).unwrap().input_ports[0].id;
        let b_out = g.node(b).unwrap().output_ports[0].id;
        let c_in = g.node(c).unwrap().input_ports[0].id;
        let c_out = g.node(c).unwrap().output_ports[0].id;
        let a_in = g.node(a).unwrap().input_ports[0].id;

        assert!(g.add_edge(a_out, b_in).is_ok());
        assert!(g.add_edge(b_out, c_in).is_ok());
        assert!(g.add_edge(c_out, a_in).is_ok());
        assert_eq!(g.edge_count(), 3);
    }

    // -- removal ------------------------------------------------------------

    #[test]
    fn remove_node_cascades_to_edges() {
        let mut g = Graph::new();
        let a = g.add_node("A", &[string_port("in")], &[string_port("out")]);
        let b = g.add_node("B", &[string_port("in")], &[string_port("out")]);
        let c = g.add_node("C", &[string_port("in")], &[]);

        let a_out = g.node(a).unwrap().output_ports[0].id;
        let b_in = g.node(b).unwrap().input_ports[0].id;
        let b_out = g.node(b).unwrap().output_ports[0].id;
        let c_in = g.node(c).unwrap().input_ports[0].id;

        g.add_edge(a_out, b_in).unwrap();
        g.add_edge(b_out, c_in).unwrap();
        assert_eq!(g.edge_count(), 2);

        g.remove_node(a).unwrap();
        assert_eq!(g.node_count(), 2);
        assert_eq!(g.edge_count(), 1); // only B→C remains
        assert!(g.node(b).is_some());
        assert!(g.node(c).is_some());
    }

    #[test]
    fn remove_single_edge() {
        let mut g = Graph::new();
        let a = g.add_node("A", &[], &[string_port("out")]);
        let b = g.add_node("B", &[string_port("in")], &[]);

        let src = g.node(a).unwrap().output_ports[0].id;
        let tgt = g.node(b).unwrap().input_ports[0].id;

        g.add_edge(src, tgt).unwrap();
        assert_eq!(g.edge_count(), 1);

        g.remove_edge(src, tgt).unwrap();
        assert_eq!(g.edge_count(), 0);
        assert_eq!(g.node_count(), 2);
    }

    // -- serde round-trip ---------------------------------------------------

    #[cfg(feature = "serde")]
    #[test]
    fn serde_round_trip() {
        let mut g = Graph::new();
        let a = g.add_node("A", &[string_port("in")], &[string_port("out")]);
        let b = g.add_node("B", &[string_port("in")], &[string_port("out")]);
        let c = g.add_node("C", &[string_port("in")], &[]);

        let a_out = g.node(a).unwrap().output_ports[0].id;
        let b_in = g.node(b).unwrap().input_ports[0].id;
        let b_out = g.node(b).unwrap().output_ports[0].id;
        let c_in = g.node(c).unwrap().input_ports[0].id;

        g.add_edge(a_out, b_in).unwrap();
        g.add_edge(b_out, c_in).unwrap();

        // Set a position to verify it persists.
        g.node_mut(a).unwrap().x = 10;
        g.node_mut(a).unwrap().y = 20;

        let json = serde_json::to_string(&g).unwrap();
        let g2 = Graph::from_json(&json).unwrap();

        assert_eq!(g2.node_count(), 3);
        assert_eq!(g2.edge_count(), 2); // Reduced from 4 to 2 — was double counting
        assert_eq!(g2.node(a).unwrap().x, 10);
        assert_eq!(g2.node(a).unwrap().y, 20);
        assert_eq!(g2.node(a).unwrap().label, "A");
    }

    #[cfg(feature = "serde")]
    #[test]
    fn id_counter_restored_after_deserialize() {
        let mut g = Graph::new();
        g.add_node("A", &[], &[]);
        g.add_node("B", &[], &[]);

        let json = serde_json::to_string(&g).unwrap();
        let mut g2 = Graph::from_json(&json).unwrap();

        let c = g2.add_node("C", &[], &[]);
        // Should not collide with existing IDs (0, 1).
        assert!(c.0 >= 2);
    }
}

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fmt;

use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct NodeId(pub u64);

impl fmt::Display for NodeId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "node:{}", self.0)
    }
}

pub trait WorkflowNodeIdentity: Copy {
    fn workflow_node_id(self) -> NodeId;
}

impl WorkflowNodeIdentity for NodeId {
    fn workflow_node_id(self) -> NodeId {
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PortId(pub u64);

impl fmt::Display for PortId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "port:{}", self.0)
    }
}

pub trait WorkflowPortIdentity: Copy {
    fn workflow_port_id(self) -> PortId;
}

impl WorkflowPortIdentity for PortId {
    fn workflow_port_id(self) -> PortId {
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PortDirection {
    Input,
    Output,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Port {
    pub id: PortId,
    pub label: String,
    pub type_tag: String,
    pub direction: PortDirection,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Node {
    pub id: NodeId,
    pub label: String,
    pub x: i32,
    pub y: i32,
    pub input_ports: Vec<Port>,
    pub output_ports: Vec<Port>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Edge {
    pub source: PortId,
    pub target: PortId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GraphError {
    SelfLoop,
    IncompatibleTypes { source_tag: String, target_tag: String },
    WouldCycle,
    PortNotFound(PortId),
    WrongDirection,
    NodeNotFound(NodeId),
    DuplicatePortLabel(String),
    NodeIdExhausted,
    PortIdExhausted,
    EdgeNotFound { source: PortId, target: PortId },
    InvalidStoredPortOwner { port: PortId, node: NodeId },
}

impl fmt::Display for GraphError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SelfLoop => write!(formatter, "self-loops are not allowed"),
            Self::IncompatibleTypes { source_tag, target_tag } => {
                write!(formatter, "incompatible port types: {source_tag:?} -> {target_tag:?}")
            }
            Self::WouldCycle => write!(formatter, "edge would create a cycle"),
            Self::PortNotFound(port) => write!(formatter, "port {port} not found"),
            Self::WrongDirection => write!(formatter, "source must be output and target must be input"),
            Self::NodeNotFound(node) => write!(formatter, "node {node} not found"),
            Self::DuplicatePortLabel(label) => write!(formatter, "duplicate port label: {label}"),
            Self::NodeIdExhausted => write!(formatter, "node identifier space is exhausted"),
            Self::PortIdExhausted => write!(formatter, "port identifier space is exhausted"),
            Self::EdgeNotFound { source, target } => write!(formatter, "edge {source} -> {target} not found"),
            Self::InvalidStoredPortOwner { port, node } => {
                write!(formatter, "stored owner {node} does not own {port}")
            }
        }
    }
}

impl std::error::Error for GraphError {}

#[allow(
    unknown_lints,
    numeric_units,
    reason = "serialized port identity counters are unitless and retain their compatible field names"
)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkflowGraph {
    pub(super) nodes: BTreeMap<NodeId, Node>,
    pub(super) edges: Vec<Edge>,
    #[serde(rename = "next_node_id")]
    pub(super) next_node_identity: u64,
    #[serde(rename = "next_port_id")]
    pub(super) next_port_identity: u64,
    pub dag_mode: bool,
    pub(super) port_owner: HashMap<PortId, NodeId>,
}

impl WorkflowGraph {
    pub fn new() -> Self {
        Self {
            nodes: BTreeMap::new(),
            edges: Vec::new(),
            next_node_identity: 0,
            next_port_identity: 0,
            dag_mode: true,
            port_owner: HashMap::new(),
        }
    }

    pub fn nodes(&self) -> impl Iterator<Item = &Node> {
        self.nodes.values()
    }

    pub fn node(&self, node_id: impl WorkflowNodeIdentity) -> Option<&Node> {
        self.nodes.get(&node_id.workflow_node_id())
    }

    pub fn node_mut(&mut self, node_id: impl WorkflowNodeIdentity) -> Option<&mut Node> {
        self.nodes.get_mut(&node_id.workflow_node_id())
    }

    pub fn edges(&self) -> &[Edge] {
        &self.edges
    }

    pub fn port(&self, port_id: impl WorkflowPortIdentity) -> Option<&Port> {
        let port_id = port_id.workflow_port_id();
        let node_id = self.port_owner.get(&port_id)?;
        let node = self.nodes.get(node_id)?;
        node.input_ports.iter().chain(&node.output_ports).find(|port| port.id == port_id)
    }

    pub fn port_owner(&self, port_id: impl WorkflowPortIdentity) -> Option<NodeId> {
        self.port_owner.get(&port_id.workflow_port_id()).copied()
    }

    #[allow(
        unknown_lints,
        usize_in_public_api,
        reason = "Rust collection and TUI APIs consume in-process collection counts as usize"
    )]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    #[allow(
        unknown_lints,
        usize_in_public_api,
        reason = "Rust collection and TUI APIs consume in-process collection counts as usize"
    )]
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    pub fn node_ids(&self) -> Vec<NodeId> {
        self.nodes.keys().copied().collect()
    }

    pub fn restore_indexes(&mut self) -> Result<(), GraphError> {
        self.restore_indexes_with_allocator_floor(0, 0)
    }

    /// Reindex typed graph edits without reusing identities retired in this session.
    /// Invalid ownership or exhausted identities leave indexes and counters unchanged.
    pub fn restore_indexes_preserving_allocators(&mut self) -> Result<(), GraphError> {
        self.restore_indexes_with_allocator_floor(self.next_node_identity, self.next_port_identity)
    }

    fn restore_indexes_with_allocator_floor(&mut self, node_floor: u64, port_floor: u64) -> Result<(), GraphError> {
        let mut port_owner = HashMap::new();
        for node in self.nodes.values() {
            for port in node.input_ports.iter().chain(&node.output_ports) {
                if let Some(prior) = port_owner.insert(port.id, node.id) {
                    return Err(GraphError::InvalidStoredPortOwner {
                        port: port.id,
                        node: prior,
                    });
                }
            }
        }
        let next_node_identity =
            next_identity(self.nodes.keys().map(|node_id| node_id.0)).ok_or(GraphError::NodeIdExhausted)?;
        let next_port_identity =
            next_identity(port_owner.keys().map(|port_id| port_id.0)).ok_or(GraphError::PortIdExhausted)?;
        self.port_owner = port_owner;
        self.next_node_identity = node_floor.max(next_node_identity);
        self.next_port_identity = port_floor.max(next_port_identity);
        Ok(())
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        let mut graph: Self = serde_json::from_str(json)?;
        graph
            .restore_indexes()
            .map_err(|error| serde_json::Error::io(std::io::Error::new(std::io::ErrorKind::InvalidData, error)))?;
        Ok(graph)
    }
}

impl Default for WorkflowGraph {
    fn default() -> Self {
        Self::new()
    }
}

fn next_identity(ids: impl Iterator<Item = u64>) -> Option<u64> {
    match ids.max() {
        Some(maximum) => maximum.checked_add(1),
        None => Some(0),
    }
}

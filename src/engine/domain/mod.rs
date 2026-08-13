//! Lattice-owned workflow graph values and deterministic mutation rules.
//!
//! Choregraph remains the authority for graph and machine admission. This
//! model owns only Lattice workflow identity, typed ports, layout data, and
//! atomic edit behavior. It has no UI, filesystem, clock, or runtime effects.

mod model;
mod mutation;
#[cfg(test)]
mod tests;

pub use model::Edge;
pub use model::GraphError;
pub use model::Node;
pub use model::NodeId;
pub use model::Port;
pub use model::PortDirection;
pub use model::PortId;
pub use model::PortSpec;
pub use model::WorkflowGraph;
pub use model::WorkflowNodeIdentity;
pub use model::WorkflowPortIdentity;

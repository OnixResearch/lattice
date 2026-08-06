//! Pure reusable DAG structure, validation, ordering, and receipt contracts.
//!
//! The runtime DAG kit validates already-loaded safe node and edge identities.
//! It does not store domain payloads, execute nodes, schedule work, read files,
//! spawn processes, call providers, persist graph state, or own host authority.

// Cairn adaptation compatibility keeps public `RuntimeDag*` DTO names, table-driven
// pure validation helpers, and dense negative fixtures reviewable while the DAG
// contract kit is archived without renaming the public surface for Octet-only style lints.
#![allow(
    unknown_lints,
    ambiguous_params,
    assertion_density,
    borrowed_argument_types,
    non_trait_imports
)]
#![allow(
    numeric_units,
    path_segment_repetition,
    raw_arithmetic_overflow,
    unbounded_collection_growth
)]

mod core;
mod data;

pub use core::blake3_runtime_dag_fingerprint;
pub use core::explain_runtime_dag_blocked_nodes;
pub use core::plan_runtime_dag_topology;
pub use core::project_runtime_dag_receipt;
pub use core::validate_runtime_dag_graph;
pub use core::validate_runtime_dag_receipt;

pub use data::RuntimeDagBlockedNode;
pub use data::RuntimeDagBounds;
pub use data::RuntimeDagEdge;
pub use data::RuntimeDagGraph;
pub use data::RuntimeDagIssueKind;
pub use data::RuntimeDagNode;
pub use data::RuntimeDagReceipt;
pub use data::RuntimeDagReceiptLeak;
pub use data::RuntimeDagRedactionStatus;
pub use data::RuntimeDagTopologicalBatch;
pub use data::RuntimeDagTopologicalPlan;
pub use data::RuntimeDagValidationIssue;
pub use data::RuntimeDagValidationReport;

pub(crate) const BLAKE3_ALGORITHM: &str = "BLAKE3";
pub(crate) const EMPTY_COUNT: usize = 0;
pub(crate) const DEFAULT_MAX_DAG_NODES: usize = 256;
pub(crate) const DEFAULT_MAX_DAG_EDGES: usize = 512;
pub(crate) const DEFAULT_MAX_DAG_LABEL_CHARS: usize = 128;
pub(crate) const HASH_HEX_CHARS: usize = 64;
pub(crate) const RUNTIME_DAG_HASH_DOMAIN: &str = "animus.runtime-dag.v1";
pub(crate) const RUNTIME_DAG_RECEIPT_SCHEMA: &str = "animus.runtime-dag.receipt.v1";
pub(crate) const HASH_PART_SEPARATOR: &str = "\u{1f}";
pub(crate) const EDGE_ARROW_SEPARATOR: &str = "->";
pub(crate) const EDGE_LABEL_SEPARATOR: &str = "|";
pub(crate) const EMPTY_EDGE_LABEL: &str = "_";
pub(crate) const UNMET_PREREQUISITE_CODE: &str = "unmet-prerequisite";
pub(crate) const FIELD_BATCH: &str = "topological_batch";
pub(crate) const FIELD_BOUNDS: &str = "bounds";
pub(crate) const FIELD_EDGE: &str = "edge";
pub(crate) const FIELD_EDGE_FROM: &str = "edge.from_node_id";
pub(crate) const FIELD_EDGE_LABEL: &str = "edge.label";
pub(crate) const FIELD_EDGE_TO: &str = "edge.to_node_id";
pub(crate) const FIELD_GRAPH_LABEL: &str = "graph.label";
pub(crate) const FIELD_HASH: &str = "hash";
pub(crate) const FIELD_NODE_ID: &str = "node.node_id";
pub(crate) const FIELD_RECEIPT_GRAPH_HASH: &str = "receipt.graph_hash";
pub(crate) const FIELD_RECEIPT_HASH: &str = "receipt.receipt_hash";
pub(crate) const FIELD_RECEIPT_LEAKS: &str = "receipt.leaks";
pub(crate) const FIELD_RECEIPT_REDACTION: &str = "receipt.redaction";
pub(crate) const FIELD_RECEIPT_SCHEMA: &str = "receipt.schema";

pub(crate) const EMPTY_FIELD_REASON: &str = "field must be non-empty";
pub(crate) const TOO_LONG_REASON: &str = "field exceeds safe length limit";
pub(crate) const TOO_MANY_EDGES_REASON: &str = "graph exceeds edge bound";
pub(crate) const TOO_MANY_NODES_REASON: &str = "graph exceeds node bound";
pub(crate) const UNSAFE_LABEL_REASON: &str = "field contains unsafe raw or leaking material";
pub(crate) const DUPLICATE_NODE_REASON: &str = "node identity must be unique";
pub(crate) const DUPLICATE_EDGE_REASON: &str = "edge identity must be unique";
pub(crate) const UNKNOWN_ENDPOINT_REASON: &str = "edge endpoint must reference a known node";
pub(crate) const SELF_EDGE_REASON: &str = "edge must not point from a node to itself";
pub(crate) const CYCLE_REASON: &str = "graph must be acyclic";
pub(crate) const INVALID_RECEIPT_SCHEMA_REASON: &str = "receipt schema is unsupported";
pub(crate) const INVALID_HASH_REASON: &str = "hash must be a BLAKE3 hex identity";
pub(crate) const UNSAFE_REDACTION_REASON: &str = "receipt must be redacted";
pub(crate) const RECEIPT_LEAK_REASON: &str = "receipt declares unsafe leak material";

pub(crate) const UNSAFE_TEXT_MARKERS: &[&str] = &[
    "RAW_PROMPT:",
    "PROVIDER_PAYLOAD:",
    "RAW_DIFF:",
    "RAW_FILE:",
    "RAW_TOOL_OUTPUT:",
    "RAW_MODEL_OUTPUT:",
    "UNBOUNDED_OUTPUT:",
    "UNBOUNDED_EVENT_STREAM:",
    "BEGIN PRIVATE KEY",
    "API_KEY=",
    "AWS_SECRET_ACCESS_KEY",
    "sk-",
    "/home/",
];

#[cfg(test)]
mod tests;

//! Runtime DAG DTOs.

use super::DEFAULT_MAX_DAG_EDGES;
use super::DEFAULT_MAX_DAG_LABEL_CHARS;
use super::DEFAULT_MAX_DAG_NODES;
use super::RUNTIME_DAG_RECEIPT_SCHEMA;

/// Safe DAG node identity with no domain payload.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeDagNode {
    /// Safe node identifier supplied by the owning domain.
    pub node_id: String,
}

/// Directed DAG edge identity with an optional safe label and no payload body.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeDagEdge {
    /// Prerequisite or predecessor node identity.
    pub from_node_id: String,
    /// Dependent or successor node identity.
    pub to_node_id: String,
    /// Optional safe edge label such as a dependency kind.
    pub label: Option<String>,
}

/// Upper bounds used before validation buffers or receipts are assembled.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RuntimeDagBounds {
    /// Accepted node count.
    pub nodes: usize,
    /// Accepted edge count.
    pub edges: usize,
    /// Accepted character count for safe labels and IDs.
    pub label_chars: usize,
}

impl Default for RuntimeDagBounds {
    fn default() -> Self {
        Self {
            nodes: DEFAULT_MAX_DAG_NODES,
            edges: DEFAULT_MAX_DAG_EDGES,
            label_chars: DEFAULT_MAX_DAG_LABEL_CHARS,
        }
    }
}

/// Safe graph structure accepted by the reusable DAG kit.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeDagGraph {
    /// Safe graph label used in diagnostics and receipts.
    pub graph_label: String,
    /// Safe node identities.
    pub nodes: Vec<RuntimeDagNode>,
    /// Directed dependency edges.
    pub edges: Vec<RuntimeDagEdge>,
    /// Validation bounds applied to this graph.
    pub bounds: RuntimeDagBounds,
}

/// Machine-readable DAG validation issue class.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RuntimeDagIssueKind {
    /// Required safe text was empty.
    EmptyField,
    /// Safe text exceeded the configured limit.
    FieldTooLong,
    /// Graph node count exceeded the configured limit.
    TooManyNodes,
    /// Graph edge count exceeded the configured limit.
    TooManyEdges,
    /// Text contained raw prompt, credential, provider payload, path, diff, or unbounded output
    /// markers.
    UnsafeLabel,
    /// Two nodes used the same identity.
    DuplicateNode,
    /// Two edges used the same identity.
    DuplicateEdge,
    /// An edge referenced an unknown node.
    UnknownEndpoint,
    /// An edge referenced the same source and target node.
    SelfEdge,
    /// The graph contains a directed cycle.
    Cycle,
    /// Receipt schema is unsupported.
    InvalidReceiptSchema,
    /// BLAKE3 hash text is malformed.
    InvalidHash,
    /// Receipt redaction is unsafe.
    UnsafeReceiptRedaction,
    /// Receipt declares unsafe leak material.
    ReceiptLeak,
}

impl RuntimeDagIssueKind {
    /// Returns the stable issue code used in receipts and blocked-node explanations.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::EmptyField => "empty-field",
            Self::FieldTooLong => "field-too-long",
            Self::TooManyNodes => "too-many-nodes",
            Self::TooManyEdges => "too-many-edges",
            Self::UnsafeLabel => "unsafe-label",
            Self::DuplicateNode => "duplicate-node",
            Self::DuplicateEdge => "duplicate-edge",
            Self::UnknownEndpoint => "unknown-endpoint",
            Self::SelfEdge => "self-edge",
            Self::Cycle => "cycle",
            Self::InvalidReceiptSchema => "invalid-receipt-schema",
            Self::InvalidHash => "invalid-hash",
            Self::UnsafeReceiptRedaction => "unsafe-receipt-redaction",
            Self::ReceiptLeak => "receipt-leak",
        }
    }
}

/// Safe validation diagnostic for a graph or graph receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeDagValidationIssue {
    /// Machine-readable issue kind.
    pub kind: RuntimeDagIssueKind,
    /// Safe field label where the issue was found.
    pub field: &'static str,
    /// Safe graph/node/edge/hash subject label.
    pub subject: String,
    /// Bounded human-readable diagnostic message.
    pub message: String,
}

impl RuntimeDagValidationIssue {
    pub(crate) fn new(
        kind: RuntimeDagIssueKind,
        field: &'static str,
        subject: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            field,
            subject: subject.into(),
            message: message.into(),
        }
    }
}

/// Pure validation report over a supplied graph.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeDagValidationReport {
    /// True when no validation issues were found.
    pub valid: bool,
    /// BLAKE3 identity over canonical graph structure.
    pub graph_hash: String,
    /// Safe node count.
    pub node_count: usize,
    /// Safe edge count.
    pub edge_count: usize,
    /// Deterministic validation issues.
    pub issues: Vec<RuntimeDagValidationIssue>,
    /// Sorted stable issue codes for receipts and summaries.
    pub issue_codes: Vec<String>,
}

/// Deterministic topological batch of ready node IDs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeDagTopologicalBatch {
    /// Zero-based batch index within the deterministic plan.
    pub batch_index: usize,
    /// Sorted node IDs that are ready together.
    pub node_ids: Vec<String>,
    /// BLAKE3 identity for the batch membership.
    pub batch_hash: String,
}

/// Safe explanation for a node blocked by unmet prerequisite IDs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeDagBlockedNode {
    /// Blocked node identity.
    pub node_id: String,
    /// Sorted safe prerequisite IDs that are not complete.
    pub unmet_prerequisite_ids: Vec<String>,
    /// Stable issue codes explaining the block.
    pub issue_codes: Vec<String>,
}

/// Deterministic topological plan and static blocked-node evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeDagTopologicalPlan {
    /// Sorted topological batches.
    pub batches: Vec<RuntimeDagTopologicalBatch>,
    /// Safe blocked-node explanations for the supplied completed-node set.
    pub blocked_nodes: Vec<RuntimeDagBlockedNode>,
}

/// Redaction state of a runtime DAG receipt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeDagRedactionStatus {
    /// Receipt contains only safe labels, counts, hashes, and redaction markers.
    Redacted,
    /// Receipt is unsafe and must be rejected.
    Unsafe,
}

/// Negative-test-only classes of unsafe receipt material.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeDagReceiptLeak {
    /// Receipt contains raw prompt text.
    RawPrompt,
    /// Receipt contains a provider payload.
    ProviderPayload,
    /// Receipt contains credential-looking material.
    Credential,
    /// Receipt contains a private or absolute path dump.
    FullPath,
    /// Receipt contains a raw diff body.
    RawDiff,
    /// Receipt contains raw tool or model output.
    RawOutput,
    /// Receipt contains unbounded event or log material.
    UnboundedStream,
}

/// Safe hash-backed receipt over graph structure, validation, and ordering evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeDagReceipt {
    /// Receipt schema label.
    pub schema: String,
    /// Safe graph label.
    pub graph_label: String,
    /// BLAKE3 identity over canonical graph structure.
    pub graph_hash: String,
    /// Safe node count.
    pub node_count: usize,
    /// Safe edge count.
    pub edge_count: usize,
    /// Sorted node identities.
    pub node_ids: Vec<String>,
    /// Sorted edge identities.
    pub edge_ids: Vec<String>,
    /// Sorted validation issue codes.
    pub issue_codes: Vec<String>,
    /// Topological batch hashes.
    pub batch_hashes: Vec<String>,
    /// Receipt redaction status.
    pub redaction: RuntimeDagRedactionStatus,
    /// Negative-test-only leak classes.
    pub leaks: Vec<RuntimeDagReceiptLeak>,
    /// BLAKE3 identity over the safe receipt summary.
    pub receipt_hash: String,
}

impl RuntimeDagReceipt {
    /// Builds an empty safe receipt shell for tests and callers that fill hashes later.
    #[must_use]
    pub fn empty_for_graph(graph_label: impl Into<String>) -> Self {
        Self {
            schema: RUNTIME_DAG_RECEIPT_SCHEMA.to_owned(),
            graph_label: graph_label.into(),
            graph_hash: String::new(),
            node_count: super::EMPTY_COUNT,
            edge_count: super::EMPTY_COUNT,
            node_ids: Vec::new(),
            edge_ids: Vec::new(),
            issue_codes: Vec::new(),
            batch_hashes: Vec::new(),
            redaction: RuntimeDagRedactionStatus::Redacted,
            leaks: Vec::new(),
            receipt_hash: String::new(),
        }
    }
}

//! Runtime DAG pure core helpers.

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use super::BLAKE3_ALGORITHM;
use super::CYCLE_REASON;
use super::DUPLICATE_EDGE_REASON;
use super::DUPLICATE_NODE_REASON;
use super::EDGE_ARROW_SEPARATOR;
use super::EDGE_LABEL_SEPARATOR;
use super::EMPTY_EDGE_LABEL;
use super::EMPTY_FIELD_REASON;
use super::FIELD_BATCH;
use super::FIELD_BOUNDS;
use super::FIELD_EDGE;
use super::FIELD_EDGE_FROM;
use super::FIELD_EDGE_LABEL;
use super::FIELD_EDGE_TO;
use super::FIELD_GRAPH_LABEL;
use super::FIELD_HASH;
use super::FIELD_NODE_ID;
use super::FIELD_RECEIPT_GRAPH_HASH;
use super::FIELD_RECEIPT_HASH;
use super::FIELD_RECEIPT_LEAKS;
use super::FIELD_RECEIPT_REDACTION;
use super::FIELD_RECEIPT_SCHEMA;
use super::HASH_HEX_CHARS;
use super::HASH_PART_SEPARATOR;
use super::INVALID_HASH_REASON;
use super::INVALID_RECEIPT_SCHEMA_REASON;
use super::RECEIPT_LEAK_REASON;
use super::RUNTIME_DAG_HASH_DOMAIN;
use super::RUNTIME_DAG_RECEIPT_SCHEMA;
use super::SELF_EDGE_REASON;
use super::TOO_LONG_REASON;
use super::TOO_MANY_EDGES_REASON;
use super::TOO_MANY_NODES_REASON;
use super::UNKNOWN_ENDPOINT_REASON;
use super::UNMET_PREREQUISITE_CODE;
use super::UNSAFE_LABEL_REASON;
use super::UNSAFE_REDACTION_REASON;
use super::UNSAFE_TEXT_MARKERS;
use super::data::RuntimeDagBlockedNode;
use super::data::RuntimeDagBounds;
use super::data::RuntimeDagEdge;
use super::data::RuntimeDagGraph;
use super::data::RuntimeDagIssueKind;
use super::data::RuntimeDagNode;
use super::data::RuntimeDagReceipt;
use super::data::RuntimeDagReceiptLeak;
use super::data::RuntimeDagRedactionStatus;
use super::data::RuntimeDagTopologicalBatch;
use super::data::RuntimeDagTopologicalPlan;
use super::data::RuntimeDagValidationIssue;
use super::data::RuntimeDagValidationReport;

#[derive(Clone, Debug, Eq, PartialEq)]
struct TopologyState {
    batches: Vec<Vec<String>>,
    remaining_nodes: Vec<String>,
}

#[derive(Clone, Copy)]
struct LabelInput<'a> {
    field: &'static str,
    subject: &'a str,
    value: &'a str,
    bounds: RuntimeDagBounds,
}

#[derive(Clone, Copy)]
struct HashInput<'a> {
    field: &'static str,
    subject: &'a str,
    value: &'a str,
}

/// Computes a BLAKE3 hex fingerprint with a runtime-DAG domain separator.
#[must_use]
pub fn blake3_runtime_dag_fingerprint(label: &str, bytes: &[u8]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(RUNTIME_DAG_HASH_DOMAIN.as_bytes());
    hasher.update(HASH_PART_SEPARATOR.as_bytes());
    hasher.update(label.as_bytes());
    hasher.update(HASH_PART_SEPARATOR.as_bytes());
    hasher.update(bytes);
    hasher.finalize().to_hex().to_string()
}

/// Validates reusable DAG structure without requesting host effects.
#[must_use]
pub fn validate_runtime_dag_graph(graph: &RuntimeDagGraph) -> RuntimeDagValidationReport {
    let mut issues = Vec::new();
    validate_label(
        LabelInput {
            field: FIELD_GRAPH_LABEL,
            subject: graph.graph_label.as_str(),
            value: graph.graph_label.as_str(),
            bounds: graph.bounds,
        },
        &mut issues,
    );
    validate_count_bounds(graph, &mut issues);

    let node_ids = collect_node_ids(&graph.nodes, graph.bounds, &mut issues);
    validate_edges(graph, &node_ids, &mut issues);
    validate_acyclic(graph, &node_ids, &mut issues);

    let graph_hash = graph_hash(graph);
    let mut issue_codes = issue_codes(&issues);
    issue_codes.sort();
    issue_codes.dedup();

    RuntimeDagValidationReport {
        valid: issues.is_empty(),
        graph_hash,
        node_count: graph.nodes.len(),
        edge_count: graph.edges.len(),
        issues,
        issue_codes,
    }
}

/// Plans deterministic topological batches for a valid DAG.
///
/// # Errors
///
/// Returns a validation report when the graph is not a valid DAG.
pub fn plan_runtime_dag_topology(
    graph: &RuntimeDagGraph,
) -> Result<RuntimeDagTopologicalPlan, RuntimeDagValidationReport> {
    let report = validate_runtime_dag_graph(graph);
    if !report.valid {
        return Err(report);
    }

    let state = topology_state(graph);
    let batches = state
        .batches
        .iter()
        .enumerate()
        .map(|(batch_index, node_ids)| RuntimeDagTopologicalBatch {
            batch_index,
            node_ids: node_ids.clone(),
            batch_hash: batch_hash(batch_index, node_ids),
        })
        .collect();

    Ok(RuntimeDagTopologicalPlan {
        batches,
        blocked_nodes: Vec::new(),
    })
}

/// Explains nodes whose incoming prerequisites are not in `completed_node_ids`.
///
/// # Errors
///
/// Returns a validation report when the graph or completed-node labels are unsafe.
pub fn explain_runtime_dag_blocked_nodes(
    graph: &RuntimeDagGraph,
    completed_node_ids: &[&str],
) -> Result<Vec<RuntimeDagBlockedNode>, RuntimeDagValidationReport> {
    let report = validate_runtime_dag_graph(graph);
    if !report.valid {
        return Err(report);
    }

    let completed = collect_completed_nodes(completed_node_ids, graph.bounds)?;
    let incoming = incoming_prerequisites(graph);
    let mut blocked = Vec::new();

    for node in sorted_node_ids(graph) {
        let unmet = incoming
            .get(node.as_str())
            .into_iter()
            .flat_map(|prerequisites| prerequisites.iter())
            .filter(|prerequisite| !completed.contains(prerequisite.as_str()))
            .cloned()
            .collect::<Vec<_>>();

        if !unmet.is_empty() {
            blocked.push(RuntimeDagBlockedNode {
                node_id: node,
                unmet_prerequisite_ids: unmet,
                issue_codes: vec![UNMET_PREREQUISITE_CODE.to_owned()],
            });
        }
    }

    Ok(blocked)
}

/// Projects a redacted hash-backed graph receipt.
#[must_use]
pub fn project_runtime_dag_receipt(
    graph: &RuntimeDagGraph,
    validation: &RuntimeDagValidationReport,
    topology: Option<&RuntimeDagTopologicalPlan>,
) -> RuntimeDagReceipt {
    let node_ids = sorted_node_ids(graph);
    let edge_ids = sorted_edge_ids(graph);
    let batch_hashes = topology
        .map(|plan| plan.batches.iter().map(|batch| batch.batch_hash.clone()).collect::<Vec<_>>())
        .unwrap_or_default();
    let mut receipt = RuntimeDagReceipt {
        schema: RUNTIME_DAG_RECEIPT_SCHEMA.to_owned(),
        graph_label: graph.graph_label.clone(),
        graph_hash: validation.graph_hash.clone(),
        node_count: validation.node_count,
        edge_count: validation.edge_count,
        node_ids,
        edge_ids,
        issue_codes: validation.issue_codes.clone(),
        batch_hashes,
        redaction: RuntimeDagRedactionStatus::Redacted,
        leaks: Vec::new(),
        receipt_hash: String::new(),
    };
    receipt.receipt_hash = receipt_hash(&receipt);
    receipt
}

/// Validates a runtime DAG receipt for schema, hash shape, redaction, and leaks.
///
/// # Errors
///
/// Returns safe diagnostics when the receipt is malformed or contains leak markers.
pub fn validate_runtime_dag_receipt(receipt: &RuntimeDagReceipt) -> Result<(), Vec<RuntimeDagValidationIssue>> {
    let mut issues = Vec::new();
    let bounds = RuntimeDagBounds::default();

    if receipt.schema != RUNTIME_DAG_RECEIPT_SCHEMA {
        issues.push(RuntimeDagValidationIssue::new(
            RuntimeDagIssueKind::InvalidReceiptSchema,
            FIELD_RECEIPT_SCHEMA,
            receipt.schema.as_str(),
            INVALID_RECEIPT_SCHEMA_REASON,
        ));
    }

    validate_label(
        LabelInput {
            field: FIELD_GRAPH_LABEL,
            subject: receipt.graph_label.as_str(),
            value: receipt.graph_label.as_str(),
            bounds,
        },
        &mut issues,
    );
    validate_hash(
        HashInput {
            field: FIELD_RECEIPT_GRAPH_HASH,
            subject: BLAKE3_ALGORITHM,
            value: receipt.graph_hash.as_str(),
        },
        &mut issues,
    );
    validate_hash(
        HashInput {
            field: FIELD_RECEIPT_HASH,
            subject: BLAKE3_ALGORITHM,
            value: receipt.receipt_hash.as_str(),
        },
        &mut issues,
    );

    validate_receipt_vectors(receipt, bounds, &mut issues);
    validate_receipt_redaction(receipt, &mut issues);

    if issues.is_empty() { Ok(()) } else { Err(issues) }
}

fn validate_count_bounds(graph: &RuntimeDagGraph, issues: &mut Vec<RuntimeDagValidationIssue>) {
    if graph.nodes.len() > graph.bounds.nodes {
        issues.push(RuntimeDagValidationIssue::new(
            RuntimeDagIssueKind::TooManyNodes,
            FIELD_BOUNDS,
            graph.graph_label.as_str(),
            TOO_MANY_NODES_REASON,
        ));
    }
    if graph.edges.len() > graph.bounds.edges {
        issues.push(RuntimeDagValidationIssue::new(
            RuntimeDagIssueKind::TooManyEdges,
            FIELD_BOUNDS,
            graph.graph_label.as_str(),
            TOO_MANY_EDGES_REASON,
        ));
    }
}

fn collect_node_ids(
    nodes: &[RuntimeDagNode],
    bounds: RuntimeDagBounds,
    issues: &mut Vec<RuntimeDagValidationIssue>,
) -> BTreeSet<String> {
    let mut node_ids = BTreeSet::new();
    let mut seen = BTreeSet::new();
    let mut duplicates = BTreeSet::new();

    for node in nodes {
        validate_label(
            LabelInput {
                field: FIELD_NODE_ID,
                subject: node.node_id.as_str(),
                value: node.node_id.as_str(),
                bounds,
            },
            issues,
        );
        if !seen.insert(node.node_id.clone()) {
            duplicates.insert(node.node_id.clone());
        }
        node_ids.insert(node.node_id.clone());
    }

    for duplicate in duplicates {
        issues.push(RuntimeDagValidationIssue::new(
            RuntimeDagIssueKind::DuplicateNode,
            FIELD_NODE_ID,
            duplicate,
            DUPLICATE_NODE_REASON,
        ));
    }

    node_ids
}

fn validate_edges(graph: &RuntimeDagGraph, node_ids: &BTreeSet<String>, issues: &mut Vec<RuntimeDagValidationIssue>) {
    let mut seen = BTreeSet::new();
    let mut duplicates = BTreeSet::new();

    for edge in &graph.edges {
        validate_edge_labels(edge, graph.bounds, issues);
        let edge_identity = edge_identity(edge);
        if !seen.insert(edge_identity.clone()) {
            duplicates.insert(edge_identity.clone());
        }
        validate_edge_endpoints(edge, node_ids, issues);
    }

    for duplicate in duplicates {
        issues.push(RuntimeDagValidationIssue::new(
            RuntimeDagIssueKind::DuplicateEdge,
            FIELD_EDGE,
            duplicate,
            DUPLICATE_EDGE_REASON,
        ));
    }
}

fn validate_edge_labels(edge: &RuntimeDagEdge, bounds: RuntimeDagBounds, issues: &mut Vec<RuntimeDagValidationIssue>) {
    validate_label(
        LabelInput {
            field: FIELD_EDGE_FROM,
            subject: edge.from_node_id.as_str(),
            value: edge.from_node_id.as_str(),
            bounds,
        },
        issues,
    );
    validate_label(
        LabelInput {
            field: FIELD_EDGE_TO,
            subject: edge.to_node_id.as_str(),
            value: edge.to_node_id.as_str(),
            bounds,
        },
        issues,
    );
    if let Some(label) = &edge.label {
        validate_label(
            LabelInput {
                field: FIELD_EDGE_LABEL,
                subject: label.as_str(),
                value: label.as_str(),
                bounds,
            },
            issues,
        );
    }
}

fn validate_edge_endpoints(
    edge: &RuntimeDagEdge,
    node_ids: &BTreeSet<String>,
    issues: &mut Vec<RuntimeDagValidationIssue>,
) {
    if edge.from_node_id == edge.to_node_id {
        issues.push(RuntimeDagValidationIssue::new(
            RuntimeDagIssueKind::SelfEdge,
            FIELD_EDGE,
            edge_identity(edge),
            SELF_EDGE_REASON,
        ));
    }
    if !node_ids.contains(edge.from_node_id.as_str()) {
        issues.push(RuntimeDagValidationIssue::new(
            RuntimeDagIssueKind::UnknownEndpoint,
            FIELD_EDGE_FROM,
            edge.from_node_id.as_str(),
            UNKNOWN_ENDPOINT_REASON,
        ));
    }
    if !node_ids.contains(edge.to_node_id.as_str()) {
        issues.push(RuntimeDagValidationIssue::new(
            RuntimeDagIssueKind::UnknownEndpoint,
            FIELD_EDGE_TO,
            edge.to_node_id.as_str(),
            UNKNOWN_ENDPOINT_REASON,
        ));
    }
}

fn validate_acyclic(graph: &RuntimeDagGraph, node_ids: &BTreeSet<String>, issues: &mut Vec<RuntimeDagValidationIssue>) {
    if has_endpoint_issue(issues) {
        return;
    }

    let state = topology_state(graph);
    for node_id in state.remaining_nodes {
        if node_ids.contains(node_id.as_str()) {
            issues.push(RuntimeDagValidationIssue::new(
                RuntimeDagIssueKind::Cycle,
                FIELD_GRAPH_LABEL,
                node_id,
                CYCLE_REASON,
            ));
        }
    }
}

fn topology_state(graph: &RuntimeDagGraph) -> TopologyState {
    let nodes = sorted_node_ids(graph);
    let mut indegrees = nodes.iter().map(|node| (node.clone(), super::EMPTY_COUNT)).collect::<BTreeMap<_, _>>();
    let mut dependents = BTreeMap::<String, Vec<String>>::new();

    for edge in &graph.edges {
        if let Some(indegree) = indegrees.get_mut(edge.to_node_id.as_str()) {
            *indegree += 1;
        }
        dependents.entry(edge.from_node_id.clone()).or_default().push(edge.to_node_id.clone());
    }

    for dependent_nodes in dependents.values_mut() {
        dependent_nodes.sort();
        dependent_nodes.dedup();
    }

    let mut ready = indegrees
        .iter()
        .filter(|(_, indegree)| **indegree == super::EMPTY_COUNT)
        .map(|(node_id, _)| node_id.clone())
        .collect::<BTreeSet<_>>();
    let mut emitted = BTreeSet::new();
    let mut batches = Vec::new();

    while !ready.is_empty() {
        let batch = ready.iter().cloned().collect::<Vec<_>>();
        ready.clear();
        for node_id in &batch {
            emitted.insert(node_id.clone());
            if let Some(children) = dependents.get(node_id.as_str()) {
                for child in children {
                    if let Some(indegree) = indegrees.get_mut(child.as_str()) {
                        *indegree = indegree.saturating_sub(1);
                        if *indegree == super::EMPTY_COUNT && !emitted.contains(child.as_str()) {
                            ready.insert(child.clone());
                        }
                    }
                }
            }
        }
        batches.push(batch);
    }

    let remaining_nodes = nodes.into_iter().filter(|node_id| !emitted.contains(node_id.as_str())).collect();

    TopologyState {
        batches,
        remaining_nodes,
    }
}

fn collect_completed_nodes(
    completed_node_ids: &[&str],
    bounds: RuntimeDagBounds,
) -> Result<BTreeSet<String>, RuntimeDagValidationReport> {
    let mut issues = Vec::new();
    let mut completed = BTreeSet::new();

    for completed_id in completed_node_ids {
        validate_label(
            LabelInput {
                field: FIELD_NODE_ID,
                subject: completed_id,
                value: completed_id,
                bounds,
            },
            &mut issues,
        );
        completed.insert((*completed_id).to_owned());
    }

    if issues.is_empty() {
        Ok(completed)
    } else {
        Err(RuntimeDagValidationReport {
            valid: false,
            graph_hash: String::new(),
            node_count: super::EMPTY_COUNT,
            edge_count: super::EMPTY_COUNT,
            issue_codes: issue_codes(&issues),
            issues,
        })
    }
}

fn incoming_prerequisites(graph: &RuntimeDagGraph) -> BTreeMap<String, Vec<String>> {
    let mut incoming = BTreeMap::<String, Vec<String>>::new();
    for edge in &graph.edges {
        incoming.entry(edge.to_node_id.clone()).or_default().push(edge.from_node_id.clone());
    }
    for prerequisites in incoming.values_mut() {
        prerequisites.sort();
        prerequisites.dedup();
    }
    incoming
}

fn validate_label(input: LabelInput<'_>, issues: &mut Vec<RuntimeDagValidationIssue>) {
    if input.value.trim().is_empty() {
        issues.push(RuntimeDagValidationIssue::new(
            RuntimeDagIssueKind::EmptyField,
            input.field,
            input.subject,
            EMPTY_FIELD_REASON,
        ));
    }
    if input.value.chars().count() > input.bounds.label_chars {
        issues.push(RuntimeDagValidationIssue::new(
            RuntimeDagIssueKind::FieldTooLong,
            input.field,
            input.subject,
            TOO_LONG_REASON,
        ));
    }
    if contains_unsafe_material(input.value) {
        issues.push(RuntimeDagValidationIssue::new(
            RuntimeDagIssueKind::UnsafeLabel,
            input.field,
            input.subject,
            UNSAFE_LABEL_REASON,
        ));
    }
}

fn validate_hash(input: HashInput<'_>, issues: &mut Vec<RuntimeDagValidationIssue>) {
    if input.value.len() != HASH_HEX_CHARS || !input.value.chars().all(|character| character.is_ascii_hexdigit()) {
        issues.push(RuntimeDagValidationIssue::new(
            RuntimeDagIssueKind::InvalidHash,
            input.field,
            input.subject,
            INVALID_HASH_REASON,
        ));
    }
}

fn validate_receipt_vectors(
    receipt: &RuntimeDagReceipt,
    bounds: RuntimeDagBounds,
    issues: &mut Vec<RuntimeDagValidationIssue>,
) {
    for node_id in &receipt.node_ids {
        validate_label(
            LabelInput {
                field: FIELD_NODE_ID,
                subject: node_id.as_str(),
                value: node_id.as_str(),
                bounds,
            },
            issues,
        );
    }
    for edge_id in &receipt.edge_ids {
        validate_label(
            LabelInput {
                field: FIELD_EDGE,
                subject: edge_id.as_str(),
                value: edge_id.as_str(),
                bounds,
            },
            issues,
        );
    }
    for issue_code in &receipt.issue_codes {
        validate_label(
            LabelInput {
                field: FIELD_RECEIPT_LEAKS,
                subject: issue_code.as_str(),
                value: issue_code.as_str(),
                bounds,
            },
            issues,
        );
    }
    for batch_hash_value in &receipt.batch_hashes {
        validate_hash(
            HashInput {
                field: FIELD_BATCH,
                subject: FIELD_HASH,
                value: batch_hash_value.as_str(),
            },
            issues,
        );
    }
}

fn validate_receipt_redaction(receipt: &RuntimeDagReceipt, issues: &mut Vec<RuntimeDagValidationIssue>) {
    if receipt.redaction != RuntimeDagRedactionStatus::Redacted {
        issues.push(RuntimeDagValidationIssue::new(
            RuntimeDagIssueKind::UnsafeReceiptRedaction,
            FIELD_RECEIPT_REDACTION,
            receipt.graph_label.as_str(),
            UNSAFE_REDACTION_REASON,
        ));
    }
    for leak in &receipt.leaks {
        issues.push(RuntimeDagValidationIssue::new(
            RuntimeDagIssueKind::ReceiptLeak,
            FIELD_RECEIPT_LEAKS,
            leak_detail(*leak),
            RECEIPT_LEAK_REASON,
        ));
    }
}

fn contains_unsafe_material(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    UNSAFE_TEXT_MARKERS.iter().any(|marker| {
        let marker_lower = marker.to_ascii_lowercase();
        lower.contains(marker_lower.as_str())
    })
}

fn edge_identity(edge: &RuntimeDagEdge) -> String {
    let label = edge.label.as_deref().unwrap_or(EMPTY_EDGE_LABEL);
    format!("{}{}{}{}{}", edge.from_node_id, EDGE_ARROW_SEPARATOR, edge.to_node_id, EDGE_LABEL_SEPARATOR, label)
}

fn sorted_node_ids(graph: &RuntimeDagGraph) -> Vec<String> {
    graph.nodes.iter().map(|node| node.node_id.clone()).collect::<BTreeSet<_>>().into_iter().collect()
}

fn sorted_edge_ids(graph: &RuntimeDagGraph) -> Vec<String> {
    graph.edges.iter().map(edge_identity).collect::<BTreeSet<_>>().into_iter().collect()
}

fn graph_hash(graph: &RuntimeDagGraph) -> String {
    let node_ids = sorted_node_ids(graph);
    let edge_ids = sorted_edge_ids(graph);
    let canonical = canonical_parts(&[
        graph.graph_label.as_str(),
        node_ids.join(HASH_PART_SEPARATOR).as_str(),
        edge_ids.join(HASH_PART_SEPARATOR).as_str(),
    ]);
    blake3_runtime_dag_fingerprint("graph", canonical.as_bytes())
}

fn batch_hash(batch_index: usize, node_ids: &[String]) -> String {
    let batch_index_text = batch_index.to_string();
    let canonical = canonical_parts(&[batch_index_text.as_str(), node_ids.join(HASH_PART_SEPARATOR).as_str()]);
    blake3_runtime_dag_fingerprint("batch", canonical.as_bytes())
}

fn receipt_hash(receipt: &RuntimeDagReceipt) -> String {
    let node_ids = receipt.node_ids.join(HASH_PART_SEPARATOR);
    let edge_ids = receipt.edge_ids.join(HASH_PART_SEPARATOR);
    let issue_codes = receipt.issue_codes.join(HASH_PART_SEPARATOR);
    let batch_hashes = receipt.batch_hashes.join(HASH_PART_SEPARATOR);
    let canonical = canonical_parts(&[
        receipt.schema.as_str(),
        receipt.graph_label.as_str(),
        receipt.graph_hash.as_str(),
        receipt.node_count.to_string().as_str(),
        receipt.edge_count.to_string().as_str(),
        node_ids.as_str(),
        edge_ids.as_str(),
        issue_codes.as_str(),
        batch_hashes.as_str(),
        "redacted",
    ]);
    blake3_runtime_dag_fingerprint("receipt", canonical.as_bytes())
}

fn canonical_parts(parts: &[&str]) -> String {
    parts.join(HASH_PART_SEPARATOR)
}

fn issue_codes(issues: &[RuntimeDagValidationIssue]) -> Vec<String> {
    issues.iter().map(|issue| issue.kind.code().to_owned()).collect()
}

fn has_endpoint_issue(issues: &[RuntimeDagValidationIssue]) -> bool {
    issues.iter().any(|issue| {
        matches!(
            issue.kind,
            RuntimeDagIssueKind::UnknownEndpoint
                | RuntimeDagIssueKind::SelfEdge
                | RuntimeDagIssueKind::DuplicateNode
                | RuntimeDagIssueKind::UnsafeLabel
                | RuntimeDagIssueKind::EmptyField
                | RuntimeDagIssueKind::FieldTooLong
        )
    })
}

const fn leak_detail(leak: RuntimeDagReceiptLeak) -> &'static str {
    match leak {
        RuntimeDagReceiptLeak::RawPrompt => "raw-prompt",
        RuntimeDagReceiptLeak::ProviderPayload => "provider-payload",
        RuntimeDagReceiptLeak::Credential => "credential",
        RuntimeDagReceiptLeak::FullPath => "full-path",
        RuntimeDagReceiptLeak::RawDiff => "raw-diff",
        RuntimeDagReceiptLeak::RawOutput => "raw-output",
        RuntimeDagReceiptLeak::UnboundedStream => "unbounded-stream",
    }
}

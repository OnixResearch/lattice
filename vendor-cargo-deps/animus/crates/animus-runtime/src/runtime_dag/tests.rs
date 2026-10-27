//! Runtime DAG focused positive and negative tests.

use super::*;

const GRAPH_LABEL: &str = "runtime-dag";
const NODE_A: &str = "alpha";
const NODE_B: &str = "beta";
const NODE_C: &str = "gamma";
const NODE_D: &str = "delta";
const MISSING_NODE: &str = "missing";
const DEPENDS_LABEL: &str = "depends";
const EXPECTED_EMPTY_BLOCKS: usize = 0;
const EXPECTED_TWO_BATCHES: usize = 2;
const EXPECTED_THREE_BATCHES: usize = 3;
const EXPECTED_FIRST_BATCH_INDEX: usize = 0;
const EXPECTED_SECOND_BATCH_INDEX: usize = 1;
const EXPECTED_TWO_NODES: usize = 2;
const SINGLE_NODE_LIMIT: usize = 1;
const SINGLE_EDGE_LIMIT: usize = 1;
const SAFE_LABEL_LIMIT: usize = 16;
const TINY_LABEL_LIMIT: usize = 3;

#[test]
fn valid_graph_batches_and_receipt_are_deterministic() {
    let graph = graph(
        &[NODE_B, NODE_A, NODE_C],
        &[(NODE_A, NODE_C), (NODE_B, NODE_C)],
    );
    let validation = validate_runtime_dag_graph(&graph);
    assert!(validation.valid);

    let plan = plan_runtime_dag_topology(&graph).expect("valid dag should plan");
    assert_eq!(plan.blocked_nodes.len(), EXPECTED_EMPTY_BLOCKS);
    assert_eq!(plan.batches.len(), EXPECTED_TWO_BATCHES);
    assert_eq!(
        plan.batches[EXPECTED_FIRST_BATCH_INDEX].node_ids,
        vec![NODE_A, NODE_B]
    );
    assert_eq!(
        plan.batches[EXPECTED_SECOND_BATCH_INDEX].node_ids,
        vec![NODE_C]
    );

    let receipt = project_runtime_dag_receipt(&graph, &validation, Some(&plan));
    assert_eq!(receipt.schema, RUNTIME_DAG_RECEIPT_SCHEMA);
    assert_eq!(receipt.node_count, graph.nodes.len());
    assert_eq!(receipt.edge_count, graph.edges.len());
    assert!(validate_runtime_dag_receipt(&receipt).is_ok());
}

#[test]
fn blocked_node_explanations_name_safe_prerequisites() {
    let graph = graph(
        &[NODE_A, NODE_B, NODE_C],
        &[(NODE_A, NODE_B), (NODE_B, NODE_C)],
    );
    let blocked = explain_runtime_dag_blocked_nodes(&graph, &[NODE_A])
        .expect("valid graph should explain blocked nodes");

    assert_eq!(blocked.len(), EXPECTED_TWO_NODES - SINGLE_NODE_LIMIT);
    assert_eq!(blocked[EXPECTED_FIRST_BATCH_INDEX].node_id, NODE_C);
    assert_eq!(
        blocked[EXPECTED_FIRST_BATCH_INDEX].unmet_prerequisite_ids,
        vec![NODE_B]
    );
    assert_eq!(
        blocked[EXPECTED_FIRST_BATCH_INDEX].issue_codes,
        vec![UNMET_PREREQUISITE_CODE]
    );
}

#[test]
fn duplicate_and_missing_endpoint_graphs_fail_closed() {
    let graph = RuntimeDagGraph {
        graph_label: GRAPH_LABEL.to_owned(),
        nodes: vec![node(NODE_A), node(NODE_A), node(NODE_B)],
        edges: vec![
            edge(NODE_A, NODE_B),
            edge(NODE_A, NODE_B),
            edge(NODE_B, MISSING_NODE),
        ],
        bounds: RuntimeDagBounds::default(),
    };
    let validation = validate_runtime_dag_graph(&graph);

    assert!(!validation.valid);
    assert_has_issue(&validation, RuntimeDagIssueKind::DuplicateNode);
    assert_has_issue(&validation, RuntimeDagIssueKind::DuplicateEdge);
    assert_has_issue(&validation, RuntimeDagIssueKind::UnknownEndpoint);
    assert!(plan_runtime_dag_topology(&graph).is_err());
}

#[test]
fn self_edges_cycles_bounds_and_unsafe_labels_fail_closed() {
    let graph = RuntimeDagGraph {
        graph_label: "RAW_PROMPT: hidden".to_owned(),
        nodes: vec![node(NODE_A), node(NODE_B), node(NODE_C)],
        edges: vec![
            edge(NODE_A, NODE_A),
            edge(NODE_B, NODE_C),
            edge(NODE_C, NODE_B),
        ],
        bounds: RuntimeDagBounds {
            nodes: SINGLE_NODE_LIMIT,
            edges: SINGLE_EDGE_LIMIT,
            label_chars: TINY_LABEL_LIMIT,
        },
    };
    let validation = validate_runtime_dag_graph(&graph);

    assert!(!validation.valid);
    assert_has_issue(&validation, RuntimeDagIssueKind::SelfEdge);
    assert_has_issue(&validation, RuntimeDagIssueKind::TooManyNodes);
    assert_has_issue(&validation, RuntimeDagIssueKind::TooManyEdges);
    assert_has_issue(&validation, RuntimeDagIssueKind::UnsafeLabel);
    assert_has_issue(&validation, RuntimeDagIssueKind::FieldTooLong);
}

#[test]
fn cycle_is_rejected_with_safe_node_evidence() {
    let graph = graph(&[NODE_A, NODE_B], &[(NODE_A, NODE_B), (NODE_B, NODE_A)]);
    let validation = validate_runtime_dag_graph(&graph);

    assert!(!validation.valid);
    assert_has_issue(&validation, RuntimeDagIssueKind::Cycle);
    assert!(
        validation
            .issues
            .iter()
            .any(|issue| issue.kind == RuntimeDagIssueKind::Cycle && issue.subject == NODE_A)
    );
}

#[test]
fn input_order_does_not_change_hash_or_batches() {
    let first = graph(
        &[NODE_D, NODE_A, NODE_C, NODE_B],
        &[(NODE_A, NODE_C), (NODE_B, NODE_C)],
    );
    let second = graph(
        &[NODE_B, NODE_C, NODE_A, NODE_D],
        &[(NODE_B, NODE_C), (NODE_A, NODE_C)],
    );

    let first_validation = validate_runtime_dag_graph(&first);
    let second_validation = validate_runtime_dag_graph(&second);
    assert_eq!(first_validation.graph_hash, second_validation.graph_hash);

    let first_plan = plan_runtime_dag_topology(&first).expect("first graph plans");
    let second_plan = plan_runtime_dag_topology(&second).expect("second graph plans");
    assert_eq!(first_plan.batches.len(), EXPECTED_TWO_BATCHES);
    assert_eq!(first_plan.batches, second_plan.batches);
    assert_eq!(
        first_plan.batches[EXPECTED_FIRST_BATCH_INDEX].node_ids,
        vec![NODE_A, NODE_B, NODE_D]
    );
    assert_eq!(
        first_plan.batches[EXPECTED_SECOND_BATCH_INDEX].node_ids,
        vec![NODE_C]
    );
}

#[test]
fn receipt_leak_fixture_is_rejected() {
    let graph = graph(&[NODE_A], &[]);
    let validation = validate_runtime_dag_graph(&graph);
    let plan = plan_runtime_dag_topology(&graph).expect("single node graph plans");
    let mut receipt = project_runtime_dag_receipt(&graph, &validation, Some(&plan));
    receipt.redaction = RuntimeDagRedactionStatus::Unsafe;
    receipt.leaks = vec![
        RuntimeDagReceiptLeak::RawPrompt,
        RuntimeDagReceiptLeak::ProviderPayload,
        RuntimeDagReceiptLeak::Credential,
        RuntimeDagReceiptLeak::FullPath,
        RuntimeDagReceiptLeak::RawDiff,
        RuntimeDagReceiptLeak::RawOutput,
        RuntimeDagReceiptLeak::UnboundedStream,
    ];

    let issues = validate_runtime_dag_receipt(&receipt).expect_err("receipt leaks fail");
    assert!(
        issues
            .iter()
            .any(|issue| issue.kind == RuntimeDagIssueKind::UnsafeReceiptRedaction)
    );
    assert_eq!(
        issues
            .iter()
            .filter(|issue| issue.kind == RuntimeDagIssueKind::ReceiptLeak)
            .count(),
        receipt.leaks.len()
    );
}

#[test]
fn malformed_receipt_hashes_fail_closed() {
    let mut receipt = RuntimeDagReceipt::empty_for_graph(GRAPH_LABEL);
    receipt.graph_hash = "not-a-hash".to_owned();
    receipt.receipt_hash = "also-not-a-hash".to_owned();
    receipt.node_ids = vec!["/home/user/raw".to_owned()];

    let issues = validate_runtime_dag_receipt(&receipt).expect_err("malformed receipt fails");
    assert!(
        issues
            .iter()
            .any(|issue| issue.kind == RuntimeDagIssueKind::InvalidHash)
    );
    assert!(
        issues
            .iter()
            .any(|issue| issue.kind == RuntimeDagIssueKind::UnsafeLabel)
    );
}

#[test]
fn topological_plan_keeps_multi_stage_order() {
    let graph = graph(
        &[NODE_A, NODE_B, NODE_C],
        &[(NODE_A, NODE_B), (NODE_B, NODE_C)],
    );
    let plan = plan_runtime_dag_topology(&graph).expect("chain plans");

    assert_eq!(plan.batches.len(), EXPECTED_THREE_BATCHES);
    assert_eq!(
        plan.batches[EXPECTED_FIRST_BATCH_INDEX].node_ids,
        vec![NODE_A]
    );
    assert_eq!(
        plan.batches[EXPECTED_SECOND_BATCH_INDEX].node_ids,
        vec![NODE_B]
    );
    assert_eq!(plan.batches[EXPECTED_TWO_BATCHES].node_ids, vec![NODE_C]);
}

fn graph(nodes: &[&str], edges: &[(&str, &str)]) -> RuntimeDagGraph {
    RuntimeDagGraph {
        graph_label: GRAPH_LABEL.to_owned(),
        nodes: nodes.iter().map(|node_id| node(node_id)).collect(),
        edges: edges
            .iter()
            .map(|(from_node_id, to_node_id)| RuntimeDagEdge {
                from_node_id: (*from_node_id).to_owned(),
                to_node_id: (*to_node_id).to_owned(),
                label: Some(DEPENDS_LABEL.to_owned()),
            })
            .collect(),
        bounds: RuntimeDagBounds {
            nodes: DEFAULT_MAX_DAG_NODES,
            edges: DEFAULT_MAX_DAG_EDGES,
            label_chars: SAFE_LABEL_LIMIT,
        },
    }
}

fn node(node_id: &str) -> RuntimeDagNode {
    RuntimeDagNode {
        node_id: node_id.to_owned(),
    }
}

fn edge(from_node_id: &str, to_node_id: &str) -> RuntimeDagEdge {
    RuntimeDagEdge {
        from_node_id: from_node_id.to_owned(),
        to_node_id: to_node_id.to_owned(),
        label: Some(DEPENDS_LABEL.to_owned()),
    }
}

fn assert_has_issue(report: &RuntimeDagValidationReport, kind: RuntimeDagIssueKind) {
    assert!(
        report.issues.iter().any(|issue| issue.kind == kind),
        "expected issue kind {kind:?} in {:?}",
        report.issues
    );
}

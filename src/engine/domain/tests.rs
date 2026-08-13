use super::*;

const FIRST_IDENTITY: u64 = 0;

#[test]
fn empty_graph_restoration_accepts_initial_identities() {
    let mut graph = WorkflowGraph::new();

    assert_eq!(graph.restore_indexes(), Ok(()));
    assert_eq!(graph.next_node_identity, FIRST_IDENTITY);
    assert_eq!(graph.next_port_identity, FIRST_IDENTITY);
}

#[test]
fn restoration_rejects_exhausted_node_identity() {
    let mut graph = WorkflowGraph::new();
    graph.nodes.insert(NodeId(u64::MAX), Node {
        id: NodeId(u64::MAX),
        label: "exhausted".to_string(),
        x: 0,
        y: 0,
        input_ports: Vec::new(),
        output_ports: Vec::new(),
    });

    assert_eq!(graph.restore_indexes(), Err(GraphError::NodeIdExhausted));
}

#[test]
fn restoration_rejects_exhausted_port_identity() {
    let node_id = NodeId(FIRST_IDENTITY);
    let mut graph = WorkflowGraph::new();
    graph.nodes.insert(node_id, Node {
        id: node_id,
        label: "exhausted".to_string(),
        x: 0,
        y: 0,
        input_ports: Vec::new(),
        output_ports: vec![Port {
            id: PortId(u64::MAX),
            label: "value".to_string(),
            type_tag: "string".to_string(),
            direction: PortDirection::Output,
        }],
    });

    assert_eq!(graph.restore_indexes(), Err(GraphError::PortIdExhausted));
}

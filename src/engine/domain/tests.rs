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

#[test]
fn typed_reindex_never_reuses_retired_node_or_port_identities() {
    let mut graph = WorkflowGraph::new();
    let first = graph.add_node("first", &[], &[PortSpec::new("output", "string")]);
    let retired = graph.add_node("retired", &[], &[PortSpec::new("output", "string")]);
    let retired_port = graph.node(retired).expect("retired").output_ports[0].id;
    graph.remove_node(retired).expect("remove maximum owner");
    graph.restore_indexes_preserving_allocators().expect("reindex typed edit");
    let replacement = graph.add_node("replacement", &[], &[PortSpec::new("output", "string")]);
    let replacement_port = graph.node(replacement).expect("replacement").output_ports[0].id;
    assert!(replacement.0 > retired.0);
    assert!(replacement_port.0 > retired_port.0);
    assert!(graph.node(first).is_some());
}

#[test]
fn explicit_high_port_identity_advances_allocator_and_rebuilds_ownership() {
    let mut graph = WorkflowGraph::new();
    let owner = graph.add_node("owner", &[], &[PortSpec::new("output", "string")]);
    let prior = graph.node(owner).expect("owner").output_ports[0].id;
    let explicit = PortId(1_000);
    graph.node_mut(owner).expect("owner").output_ports[0].id = explicit;
    graph.restore_indexes_preserving_allocators().expect("typed id edit");
    assert_eq!(graph.port_owner(explicit), Some(owner));
    assert!(graph.port(prior).is_none());
    let added = graph.add_node("new", &[], &[PortSpec::new("output", "string")]);
    assert!(graph.node(added).expect("added").output_ports[0].id.0 > explicit.0);
}

#[test]
fn failed_typed_reindex_preserves_indexes_and_allocator_counters() {
    for invalid in 0..3 {
        let mut graph = WorkflowGraph::new();
        let first = graph.add_node("first", &[], &[PortSpec::new("output", "string")]);
        let second = graph.add_node("second", &[], &[PortSpec::new("output", "string")]);
        match invalid {
            0 => {
                let duplicate = graph.node(first).expect("first").output_ports[0].id;
                graph.node_mut(second).expect("second").output_ports[0].id = duplicate;
            }
            1 => graph.node_mut(second).expect("second").output_ports[0].id = PortId(u64::MAX),
            2 => {
                let mut exhausted = graph.node(second).cloned().expect("second");
                exhausted.id = NodeId(u64::MAX);
                graph.nodes.insert(exhausted.id, exhausted);
                graph.nodes.remove(&second);
            }
            _ => unreachable!("bounded test cases"),
        }
        let before = graph.clone();
        assert!(graph.restore_indexes_preserving_allocators().is_err());
        assert_eq!(graph, before);
    }
}

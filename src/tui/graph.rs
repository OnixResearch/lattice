//! Subwayrat graph conversion at the TUI boundary.

use std::fmt;

use crate::engine::workflow::WorkflowNodeIdentity;
use crate::engine::workflow::WorkflowPortIdentity;

#[derive(Debug)]
pub enum AdapterError {
    Encode(serde_json::Error),
    Decode(serde_json::Error),
}

impl fmt::Display for AdapterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Encode(_) => write!(formatter, "workflow graph adapter could not encode the source graph"),
            Self::Decode(_) => write!(formatter, "workflow graph adapter rejected the encoded graph"),
        }
    }
}

impl std::error::Error for AdapterError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Encode(error) | Self::Decode(error) => Some(error),
        }
    }
}

impl WorkflowNodeIdentity for rat_nodegraph::model::NodeId {
    fn workflow_node_id(self) -> crate::engine::workflow::NodeId {
        crate::engine::workflow::NodeId(self.0)
    }
}

impl WorkflowPortIdentity for rat_nodegraph::model::PortId {
    fn workflow_port_id(self) -> crate::engine::workflow::PortId {
        crate::engine::workflow::PortId(self.0)
    }
}

pub const fn subwayrat_node_id(node_id: crate::engine::workflow::NodeId) -> rat_nodegraph::model::NodeId {
    rat_nodegraph::model::NodeId(node_id.0)
}

pub const fn workflow_node_id(node_id: rat_nodegraph::model::NodeId) -> crate::engine::workflow::NodeId {
    crate::engine::workflow::NodeId(node_id.0)
}

pub const fn subwayrat_port_id(port_id: crate::engine::workflow::PortId) -> rat_nodegraph::model::PortId {
    rat_nodegraph::model::PortId(port_id.0)
}

pub const fn workflow_port_id(port_id: rat_nodegraph::model::PortId) -> crate::engine::workflow::PortId {
    crate::engine::workflow::PortId(port_id.0)
}

pub fn empty_subwayrat() -> rat_nodegraph::model::Graph {
    configure_subwayrat(rat_nodegraph::model::Graph::new())
}

pub fn to_subwayrat(
    graph: &crate::engine::workflow::WorkflowGraph,
) -> Result<rat_nodegraph::model::Graph, AdapterError> {
    let encoded = serde_json::to_string(graph).map_err(AdapterError::Encode)?;
    let graph = rat_nodegraph::model::Graph::from_json(&encoded).map_err(AdapterError::Decode)?;
    Ok(configure_subwayrat(graph))
}

pub fn from_subwayrat(
    graph: &rat_nodegraph::model::Graph,
) -> Result<crate::engine::workflow::WorkflowGraph, AdapterError> {
    let encoded = serde_json::to_string(graph).map_err(AdapterError::Encode)?;
    decode_domain(&encoded)
}

fn configure_subwayrat(mut graph: rat_nodegraph::model::Graph) -> rat_nodegraph::model::Graph {
    graph.set_compatibility(Box::new(|source, target| source == target || source == "any" || target == "any"));
    graph
}

fn decode_domain(encoded: &str) -> Result<crate::engine::workflow::WorkflowGraph, AdapterError> {
    crate::engine::workflow::WorkflowGraph::from_json(encoded).map_err(AdapterError::Decode)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::workflow::PortSpec;

    const SOURCE_X: i32 = 17;
    const SOURCE_Y: i32 = 29;

    #[test]
    fn graph_round_trip_preserves_identities_positions_and_edges() {
        let mut graph = crate::engine::workflow::WorkflowGraph::new();
        let source = graph.add_node("source", &[], &[PortSpec::new("value", "string")]);
        let target = graph.add_node("target", &[PortSpec::new("value", "string")], &[]);
        graph.node_mut(source).expect("source").x = SOURCE_X;
        graph.node_mut(source).expect("source").y = SOURCE_Y;
        let source_port = graph.node(source).expect("source").output_ports[0].id;
        let target_port = graph.node(target).expect("target").input_ports[0].id;
        graph.add_edge(source_port, target_port).expect("edge");

        let subwayrat = to_subwayrat(&graph).expect("domain graph converts");
        let restored = from_subwayrat(&subwayrat).expect("Subwayrat graph converts");

        assert_eq!(restored, graph);
    }

    #[test]
    fn subwayrat_workflow_record_loads_and_admits_without_effects() {
        use crate::engine::workflow::NodeConfig;

        let directory = tempfile::tempdir().expect("record directory");
        let database_path = directory.path().join("legacy.redb");
        let effect_path = directory.path().join("must-not-run");
        let mut graph = rat_nodegraph::model::Graph::new();
        let source = graph.add_node("source", &[rat_nodegraph::model::PortSpec::new("stdin", "string")], &[
            rat_nodegraph::model::PortSpec::new("stdout", "string"),
            rat_nodegraph::model::PortSpec::new("stderr", "string"),
            rat_nodegraph::model::PortSpec::new("exit", "exitcode"),
        ]);
        let target = graph.add_node("target", &[rat_nodegraph::model::PortSpec::new("stdin", "string")], &[
            rat_nodegraph::model::PortSpec::new("stdout", "string"),
            rat_nodegraph::model::PortSpec::new("stderr", "string"),
            rat_nodegraph::model::PortSpec::new("exit", "exitcode"),
        ]);
        let output = graph.node(source).expect("source").output_ports[0].id;
        let input = graph.node(target).expect("target").input_ports[0].id;
        graph.add_edge(output, input).expect("legacy edge");
        let command = format!("touch {}", effect_path.display());
        let configs = std::collections::HashMap::from([
            (source.0, NodeConfig::shell(&command)),
            (target.0, NodeConfig::shell("cat")),
        ]);
        let record = serde_json::to_vec(&serde_json::json!({
            "meta": { "name": "legacy", "description": "", "created": "1970-01-01T00:00:00Z", "modified": "1970-01-01T00:00:00Z" },
            "graph": graph,
            "configs": configs
        })).expect("legacy record encodes");

        drop(crate::store::Store::new(&database_path).expect("initialize tables"));
        let database = redb::Database::create(&database_path).expect("open raw store");
        let transaction = database.begin_write().expect("write transaction");
        {
            let mut table =
                transaction.open_table(redb::TableDefinition::<&str, &[u8]>::new("workflows")).expect("workflows");
            table.insert("legacy", record.as_slice()).expect("legacy bytes persist");
        }
        transaction.commit().expect("commit legacy record");
        drop(database);

        let store = crate::store::Store::new(&database_path).expect("current store");
        let loaded = store.load_workflow("legacy").expect("legacy decode").expect("legacy record exists");
        let portable = crate::workflow_format::workflow_to_portable(&loaded);
        let admitted = crate::workflow_format::choregraph::lower_portable(&portable).expect("legacy graph admits");

        // r[verify lattice.workflow_validation.domain_boundary.compatibility.persisted]
        assert_eq!(loaded.graph.node(workflow_node_id(source)).expect("source identity").label, "source");
        assert_eq!(loaded.graph.edges(), &[crate::engine::workflow::Edge {
            source: workflow_port_id(output),
            target: workflow_port_id(input)
        }]);
        assert!(
            matches!(loaded.node_config(workflow_node_id(source)), Some(NodeConfig::Shell { command: actual, .. }) if actual == &command)
        );
        assert_eq!(admitted.machine.edges().len(), 1);
        assert!(!effect_path.exists());
    }

    #[test]
    fn malformed_graph_payload_fails_closed() {
        let error = decode_domain("{\"nodes\":[]}").expect_err("malformed graph must fail");
        assert!(matches!(error, AdapterError::Decode(_)));
    }
}

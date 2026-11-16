//! Observed output records and usage metadata.

use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunOutputLine {
    pub run_id: RunId,
    pub node_id: crate::engine::workflow::NodeId,
    pub port_id: crate::engine::workflow::PortId,
    pub port_label: String,
    pub text: String,
}

impl RunOutputLine {
    pub(in crate::tui::app) fn rendered(&self) -> String {
        format!("  │ {}: {}", self.port_label, self.text)
    }
}

pub fn latest_node_output_records(
    records: &[RunOutputLine],
    workflow: &Workflow,
    node_id: impl WorkflowNodeIdentity,
) -> Vec<RunOutputLine> {
    let node_id = node_id.workflow_node_id();
    let mut latest_by_port = HashMap::with_capacity(records.len());
    for record in records.iter().filter(|record| record.node_id == node_id) {
        latest_by_port.insert(record.port_id, record.clone());
    }

    if let Some(node) = workflow.graph.node(node_id) {
        return node.output_ports.iter().filter_map(|port| latest_by_port.remove(&port.id)).collect();
    }

    let mut latest_records = latest_by_port.into_values().collect::<Vec<_>>();
    latest_records.sort_by_key(|record| record.port_id.0);
    latest_records
}

pub fn rendered_latest_node_output_lines(
    records: &[RunOutputLine],
    workflow: &Workflow,
    node_id: impl WorkflowNodeIdentity,
) -> Vec<String> {
    latest_node_output_records(records, workflow, node_id)
        .into_iter()
        .map(|record| record.rendered())
        .collect()
}

pub(in crate::tui::app) fn completed_output_records(
    run_id: RunId,
    workflow: &Workflow,
    node_id: crate::engine::workflow::NodeId,
    outputs: &HashMap<crate::engine::workflow::PortId, crate::engine::value::Value>,
) -> Vec<RunOutputLine> {
    let mut records = Vec::with_capacity(outputs.len());

    if let Some(node) = workflow.graph.node(node_id) {
        for port in &node.output_ports {
            if let Some(value) = outputs.get(&port.id) {
                records.extend(output_value_records(run_id, node_id, port.id, &port.label, value));
            }
        }
        return records;
    }

    let mut fallback_outputs = outputs.iter().collect::<Vec<_>>();
    fallback_outputs.sort_by_key(|(port_id, _)| port_id.0);
    for (port_id, value) in fallback_outputs {
        records.extend(output_value_records(run_id, node_id, *port_id, &format!("port {}", port_id.0), value));
    }

    records
}

pub(in crate::tui::app) fn output_value_records(
    run_id: RunId,
    node_id: crate::engine::workflow::NodeId,
    port_id: crate::engine::workflow::PortId,
    port_label: &str,
    value: &crate::engine::value::Value,
) -> Vec<RunOutputLine> {
    let display = value.as_display_string();
    if display.is_empty() {
        return Vec::new();
    }

    display
        .lines()
        .map(|line| RunOutputLine {
            run_id,
            node_id,
            port_id,
            port_label: port_label.to_string(),
            text: line.to_string(),
        })
        .collect()
}

pub(in crate::tui::app) fn node_usage_tokens(
    workflow: &Workflow,
    node_id: crate::engine::workflow::NodeId,
    outputs: &HashMap<crate::engine::workflow::PortId, crate::engine::value::Value>,
) -> Option<u64> {
    let metadata = node_metadata(workflow, node_id, outputs)?;
    let usage = metadata.get("usage")?;
    usage.get("total_tokens").and_then(serde_json::Value::as_u64).or_else(|| {
        let input = usage
            .get("input_tokens")
            .or_else(|| usage.get("prompt_tokens"))
            .and_then(serde_json::Value::as_u64)?;
        let output = usage
            .get("output_tokens")
            .or_else(|| usage.get("completion_tokens"))
            .and_then(serde_json::Value::as_u64)?;
        input.checked_add(output)
    })
}

pub(in crate::tui::app) fn node_usage_cost(
    workflow: &Workflow,
    node_id: crate::engine::workflow::NodeId,
    outputs: &HashMap<crate::engine::workflow::PortId, crate::engine::value::Value>,
) -> Option<u64> {
    let metadata = node_metadata(workflow, node_id, outputs)?;
    let usage = metadata.get("usage")?;
    usage
        .get("total_cost_microunits")
        .or_else(|| usage.get("cost_microunits"))
        .and_then(serde_json::Value::as_u64)
}

pub(in crate::tui::app) fn node_metadata<'a>(
    workflow: &Workflow,
    node_id: crate::engine::workflow::NodeId,
    outputs: &'a HashMap<crate::engine::workflow::PortId, crate::engine::value::Value>,
) -> Option<&'a serde_json::Value> {
    let node = workflow.graph.node(node_id)?;
    let metadata_port = node.output_ports.iter().find(|port| port.label == "metadata")?;
    let value = outputs.get(&metadata_port.id)?;
    let crate::engine::value::ValueData::Json(metadata) = &value.data else {
        return None;
    };
    Some(metadata)
}

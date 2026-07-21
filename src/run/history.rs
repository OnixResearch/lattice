//! Pure construction of durable, redaction-safe run-history details.

const UNKNOWN_NODE_LABEL: &str = "?";
const REDACTED_OUTPUT: &str = "[redacted]";

// r[impl lattice.workflow_execution.headless_run_details]
// r[impl n8nclone.tui_ux.run_history.details.persist]
pub fn detail_record_from_results(
    run_id: crate::engine::executor::RunId,
    workflow: &crate::engine::workflow::Workflow,
    results: &[crate::engine::executor::StepResult],
) -> crate::store::RunDetailRecord {
    let nodes = results.iter().map(|step| node_detail_from_step(workflow, step)).collect::<Vec<_>>();
    debug_assert_eq!(nodes.len(), results.len());
    debug_assert!(nodes.iter().all(|node| !node.node_label.is_empty()));
    crate::store::RunDetailRecord { run_id, nodes }
}

fn node_detail_from_step(
    workflow: &crate::engine::workflow::Workflow,
    step: &crate::engine::executor::StepResult,
) -> crate::store::RunNodeDetail {
    let node_label = workflow
        .graph
        .node(step.node_id)
        .map(|node| node.label.clone())
        .unwrap_or_else(|| UNKNOWN_NODE_LABEL.to_string());
    let detail = match &step.status {
        crate::engine::executor::NodeStatus::Completed { outputs, cached } => crate::store::RunNodeDetail {
            node_id: step.node_id,
            node_label,
            status: crate::store::RunNodeDetailStatus::Completed,
            cached: *cached,
            outputs: completed_output_details(workflow, step.node_id, outputs),
            error: None,
        },
        crate::engine::executor::NodeStatus::Failed { error } => {
            terminal_detail(step.node_id, node_label, crate::store::RunNodeDetailStatus::Failed, Some(error.clone()))
        }
        crate::engine::executor::NodeStatus::Skipped => {
            terminal_detail(step.node_id, node_label, crate::store::RunNodeDetailStatus::Skipped, None)
        }
        crate::engine::executor::NodeStatus::Pending => {
            terminal_detail(step.node_id, node_label, crate::store::RunNodeDetailStatus::Pending, None)
        }
        crate::engine::executor::NodeStatus::Running => {
            terminal_detail(step.node_id, node_label, crate::store::RunNodeDetailStatus::Running, None)
        }
    };
    debug_assert_eq!(detail.node_id, step.node_id);
    debug_assert!(!detail.node_label.is_empty());
    detail
}

fn terminal_detail(
    node_id: rat_nodegraph::model::NodeId,
    node_label: String,
    status: crate::store::RunNodeDetailStatus,
    error: Option<String>,
) -> crate::store::RunNodeDetail {
    debug_assert!(status != crate::store::RunNodeDetailStatus::Completed);
    debug_assert!(!node_label.is_empty());
    crate::store::RunNodeDetail {
        node_id,
        node_label,
        status,
        cached: false,
        outputs: Vec::new(),
        error,
    }
}

fn completed_output_details(
    workflow: &crate::engine::workflow::Workflow,
    node_id: rat_nodegraph::model::NodeId,
    outputs: &std::collections::HashMap<rat_nodegraph::model::PortId, crate::engine::value::Value>,
) -> Vec<crate::store::RunOutputDetail> {
    let mut details = Vec::with_capacity(outputs.len());
    if let Some(node) = workflow.graph.node(node_id) {
        for port in &node.output_ports {
            if let Some(value) = outputs.get(&port.id) {
                details.extend(value_details(port.id, &port.label, value));
            }
        }
        debug_assert!(details.iter().all(|detail| outputs.contains_key(&detail.port_id)));
        debug_assert!(details.iter().all(|detail| node.output_ports.iter().any(|port| port.id == detail.port_id)));
        return details;
    }

    let mut fallback_outputs = outputs.iter().collect::<Vec<_>>();
    fallback_outputs.sort_by_key(|(port_id, _)| port_id.0);
    for (port_id, value) in fallback_outputs {
        details.extend(value_details(*port_id, &format!("port {}", port_id.0), value));
    }
    debug_assert!(details.windows(2).all(|pair| pair[0].port_id.0 <= pair[1].port_id.0));
    details
}

fn value_details(
    port_id: rat_nodegraph::model::PortId,
    port_label: &str,
    value: &crate::engine::value::Value,
) -> Vec<crate::store::RunOutputDetail> {
    let display = persisted_display(value);
    if display.is_empty() {
        return Vec::new();
    }
    display
        .lines()
        .map(|line| crate::store::RunOutputDetail {
            port_id,
            port_label: port_label.to_string(),
            text: line.to_string(),
        })
        .collect()
}

fn persisted_display(value: &crate::engine::value::Value) -> String {
    if value.sensitive_fragments.is_empty() {
        return value.as_display_string();
    }
    REDACTED_OUTPUT.to_string()
}

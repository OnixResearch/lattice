//! Topological rows view for mostly-linear workflow DAGs.

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashMap;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Widget;

use crate::engine::workflow::NodeConfig;
use crate::engine::workflow::NodeId;
use crate::engine::workflow::Workflow;
use crate::tui::app::NodeRunStatus;
use crate::tui::app::RunOutputLine;
use crate::tui::app::latest_node_output_records;

pub const DEFAULT_MAX_LANES: usize = 6;
pub const OUTPUT_SUMMARY_MAX_CHARS: usize = 40;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanvasView {
    Graph,
    Rows,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub node_id: NodeId,
    pub label: String,
    pub kind: &'static str,
    pub depth: usize,
    pub lane: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RowsLayout {
    pub rows: Vec<Row>,
    pub lane_count: usize,
    pub collapsed_lanes: usize,
}

pub fn layout(workflow: &Workflow, max_lanes: usize) -> RowsLayout {
    let max_lanes = max_lanes.max(1);
    let mut indegree = workflow.graph.nodes().map(|node| (node.id, 0_usize)).collect::<BTreeMap<_, _>>();
    let mut outgoing = BTreeMap::<NodeId, BTreeSet<NodeId>>::new();
    for edge in workflow.graph.edges() {
        let Some(source) = workflow.graph.port_owner(edge.source) else {
            continue;
        };
        let Some(target) = workflow.graph.port_owner(edge.target) else {
            continue;
        };
        if source == target {
            continue;
        }
        if outgoing.entry(source).or_default().insert(target) {
            *indegree.entry(target).or_default() += 1;
        }
    }

    let mut ready = indegree
        .iter()
        .filter_map(|(node_id, count)| (*count == 0).then_some(*node_id))
        .collect::<BTreeSet<_>>();
    let mut depth = ready.iter().map(|node_id| (*node_id, 0_usize)).collect::<BTreeMap<_, _>>();
    let mut order = Vec::new();
    while let Some(node_id) = ready.pop_first() {
        order.push(node_id);
        let next_depth = depth.get(&node_id).copied().unwrap_or(0).saturating_add(1);
        for target in outgoing.get(&node_id).cloned().unwrap_or_default() {
            depth
                .entry(target)
                .and_modify(|current| *current = (*current).max(next_depth))
                .or_insert(next_depth);
            if let Some(count) = indegree.get_mut(&target) {
                *count = count.saturating_sub(1);
                if *count == 0 {
                    ready.insert(target);
                }
            }
        }
    }
    for node_id in indegree.keys() {
        if !depth.contains_key(node_id) {
            depth.insert(*node_id, 0);
            order.push(*node_id);
        }
    }

    let mut by_depth = BTreeMap::<usize, Vec<NodeId>>::new();
    for node_id in order {
        by_depth.entry(depth.get(&node_id).copied().unwrap_or(0)).or_default().push(node_id);
    }
    let original_lane_count = by_depth.values().map(Vec::len).max().unwrap_or(1).max(1);
    let lane_count = original_lane_count.min(max_lanes);
    let collapsed_lanes = original_lane_count.saturating_sub(lane_count);

    let mut rows = Vec::new();
    for (node_depth, mut nodes) in by_depth {
        nodes.sort();
        for (original_lane, node_id) in nodes.into_iter().enumerate() {
            let lane = original_lane.min(lane_count.saturating_sub(1));
            let label = workflow.graph.node(node_id).map(|node| node.label.clone()).unwrap_or_default();
            rows.push(Row {
                node_id,
                label,
                kind: node_kind_label(workflow.node_config(node_id)),
                depth: node_depth,
                lane,
            });
        }
    }

    RowsLayout {
        rows,
        lane_count,
        collapsed_lanes,
    }
}

pub fn node_kind_label(config: Option<&NodeConfig>) -> &'static str {
    match config {
        Some(NodeConfig::Shell { .. }) => "shell",
        Some(NodeConfig::Http { .. }) => "http",
        Some(NodeConfig::File { .. }) => "file",
        Some(NodeConfig::ImageInput { .. }) => "image",
        Some(NodeConfig::Agent(_)) => "agent",
        Some(NodeConfig::Operation(_)) => "operation",
        Some(NodeConfig::Condition(_)) => "condition",
        Some(NodeConfig::Merge(_)) => "merge",
        Some(NodeConfig::Subworkflow(_)) => "subworkflow",
        Some(NodeConfig::Approval(_)) => "approval",
        None => "missing",
    }
}

pub fn render_rows(
    layout: &RowsLayout,
    workflow: &Workflow,
    statuses: &HashMap<u64, NodeRunStatus>,
    outputs: &[RunOutputLine],
    selected: &BTreeSet<NodeId>,
    focused: Option<NodeId>,
    area: Rect,
    buffer: &mut Buffer,
) {
    let mut lines = Vec::with_capacity(layout.rows.len().saturating_add(2));
    lines.push(Line::styled(
        format!("  #  LN  {:<20} {:<10} {:<9} OUTPUT", "NODE", "KIND", "STATUS"),
        Style::default().fg(Color::DarkGray),
    ));
    for (index, row) in layout.rows.iter().enumerate() {
        let marker = if selected.contains(&row.node_id) {
            "▶"
        } else if focused == Some(row.node_id) {
            ">"
        } else {
            " "
        };
        let lane = if layout.collapsed_lanes > 0 && row.lane == layout.lane_count.saturating_sub(1) {
            format!("{}+", row.lane)
        } else {
            row.lane.to_string()
        };
        let status = status_text(statuses.get(&row.node_id.0).copied());
        let output = latest_node_output_records(outputs, workflow, row.node_id)
            .last()
            .map(|record| bounded_output(&record.text))
            .unwrap_or_default();
        let text = format!(
            "{marker}{index:>3} {lane:>3}  {:<20} {:<10} {:<9} {}",
            bounded_label(&row.label),
            row.kind,
            status,
            output
        );
        let style = if selected.contains(&row.node_id) {
            Style::default().fg(Color::Yellow)
        } else if focused == Some(row.node_id) {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default()
        };
        lines.push(Line::styled(text, style));
    }
    if layout.collapsed_lanes > 0 {
        lines.push(Line::styled(
            format!("+{} lanes collapsed; resize wider to inspect them", layout.collapsed_lanes),
            Style::default().fg(Color::DarkGray),
        ));
    }
    Paragraph::new(lines).render(area, buffer);
}

fn status_text(status: Option<NodeRunStatus>) -> &'static str {
    match status {
        Some(NodeRunStatus::Pending) => "pending",
        Some(NodeRunStatus::Running) => "running",
        Some(NodeRunStatus::Completed) => "completed",
        Some(NodeRunStatus::Failed) => "failed",
        Some(NodeRunStatus::Skipped) => "skipped",
        None => "idle",
    }
}

fn bounded_label(label: &str) -> String {
    bounded_chars(label, 20)
}

fn bounded_output(text: &str) -> String {
    bounded_chars(text, OUTPUT_SUMMARY_MAX_CHARS)
}

fn bounded_chars(text: &str, maximum: usize) -> String {
    let mut chars = text.chars();
    let bounded = chars.by_ref().take(maximum).collect::<String>();
    if chars.next().is_none() {
        bounded
    } else {
        format!("{}…", bounded.chars().take(maximum.saturating_sub(1)).collect::<String>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::workflow::WorkflowTimestamp;

    const LINEAR_NODES: usize = 3;
    const FAN_OUT_NODES: usize = 10;
    const TEST_MAX_LANES: usize = 4;

    fn workflow_with_edges(edges: &[(usize, usize)], nodes: usize) -> Workflow {
        let mut workflow = Workflow::new("rows", WorkflowTimestamp::epoch());
        let ids = (0..nodes).map(|index| workflow.add_shell_node(format!("node-{index}"), "true")).collect::<Vec<_>>();
        for (source, target) in edges {
            let source_port = workflow.graph.node(ids[*source]).expect("source").output_ports[0].id;
            let target_port = workflow.graph.node(ids[*target]).expect("target").input_ports[0].id;
            workflow.graph.add_edge(source_port, target_port).expect("edge");
        }
        workflow
    }

    #[test]
    fn linear_rows_preserve_topological_order() {
        let workflow = workflow_with_edges(&[(0, 1), (1, 2)], LINEAR_NODES);
        let layout = layout(&workflow, DEFAULT_MAX_LANES);
        assert_eq!(layout.lane_count, 1);
        assert_eq!(layout.collapsed_lanes, 0);
        assert_eq!(layout.rows.iter().map(|row| row.depth).collect::<Vec<_>>(), vec![0, 1, 2]);
    }

    #[test]
    fn fork_join_rows_use_parallel_lanes() {
        let workflow = workflow_with_edges(&[(0, 1), (0, 2), (1, 3), (2, 3)], TEST_MAX_LANES);
        let layout = layout(&workflow, TEST_MAX_LANES);
        assert_eq!(layout.lane_count, 2);
        assert_eq!(layout.rows.iter().filter(|row| row.depth == 1).count(), 2);
        assert_eq!(layout.rows.iter().filter(|row| row.depth == 1).map(|row| row.lane).collect::<Vec<_>>(), vec![0, 1]);
    }

    #[test]
    fn diamond_rows_collapse_to_the_join_lane() {
        let workflow = workflow_with_edges(&[(0, 1), (0, 2), (1, 3), (2, 3)], TEST_MAX_LANES);
        let layout = layout(&workflow, TEST_MAX_LANES);
        let join = layout.rows.iter().find(|row| row.depth == 2).expect("join row");
        assert_eq!(join.lane, 0);
    }

    #[test]
    fn wide_fan_out_reports_collapsed_lanes() {
        let edges = (1..=FAN_OUT_NODES).map(|target| (0, target)).collect::<Vec<_>>();
        let workflow = workflow_with_edges(&edges, FAN_OUT_NODES + 1);
        let layout = layout(&workflow, TEST_MAX_LANES);
        assert_eq!(layout.lane_count, TEST_MAX_LANES);
        assert_eq!(layout.collapsed_lanes, FAN_OUT_NODES - TEST_MAX_LANES);
        assert!(layout.rows.iter().all(|row| row.lane < TEST_MAX_LANES));
    }

    #[test]
    fn rows_render_status_and_output_summary() {
        let workflow = workflow_with_edges(&[(0, 1)], 2);
        let layout = layout(&workflow, DEFAULT_MAX_LANES);
        let statuses = HashMap::from([(0, NodeRunStatus::Completed), (1, NodeRunStatus::Failed)]);
        let node_id = workflow.graph.nodes().next().expect("node").id;
        let outputs = vec![RunOutputLine {
            run_id: crate::engine::executor::RunId(1),
            node_id,
            port_id: workflow.graph.node(node_id).expect("node").output_ports[0].id,
            port_label: "stdout".to_string(),
            text: "bounded output".to_string(),
        }];
        let mut buffer = Buffer::empty(Rect::new(0, 0, 80, 6));
        render_rows(
            &layout,
            &workflow,
            &statuses,
            &outputs,
            &BTreeSet::from([node_id]),
            Some(node_id),
            buffer.area,
            &mut buffer,
        );
        let rendered = buffer
            .content()
            .chunks(80)
            .map(|row| row.iter().map(|cell| cell.symbol()).collect::<String>())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(rendered.contains("NODE"));
        assert!(rendered.contains("completed"));
        assert!(rendered.contains("bounded output"));
    }
}

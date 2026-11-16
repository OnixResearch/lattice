//! Topological monitoring rows with a bounded frontier of DAG lanes.

use std::collections::BTreeSet;
use std::collections::HashMap;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Widget;

use crate::engine::workflow::NodeConfig;
use crate::engine::workflow::NodeId;
use crate::engine::workflow::Workflow;
use crate::tui::app::NodeRunStatus;
use crate::tui::app::RunOutputLine;

mod rendering;
mod topology;

pub const DEFAULT_MAX_LANES: usize = 6;
pub const OUTPUT_SUMMARY_MAX_CHARS: usize = 40;
const METADATA_COLUMNS: u16 = 62;

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("node dependency count exceeds the layout range")]
    DegreeOverflow,
    #[error("node depth exceeds the layout range")]
    DepthOverflow,
    #[error("a node dependency was consumed more than once")]
    DependencyUnderflow,
    #[error("lane geometry exceeds the layout range")]
    LaneGeometry,
}

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
    pub connectors: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Layout {
    pub rows: Vec<Row>,
    pub lane_count: usize,
    pub collapsed_lanes: usize,
}

/// Leave room for metadata while growing the lane frontier on wider terminals.
pub(crate) fn max_lanes_for_width(width: u16) -> usize {
    usize::from(width.saturating_sub(METADATA_COLUMNS) / 2).clamp(1, DEFAULT_MAX_LANES)
}

// r[impl lattice.tui_ux.rows.topology]
// r[impl lattice.tui_ux.rows.wide]
pub fn layout(workflow: &Workflow, max_lanes: usize) -> Result<Layout, Error> {
    let (order, outgoing) = topology::ordered_nodes(workflow)?;
    // Empty lanes beyond the workflow's nodes cannot carry any connector.
    let max_lanes = max_lanes.max(1).min(order.len().max(1));
    let mut reservations = topology::Reservations {
        occupants: order.iter().filter(|(depth, _)| *depth == 0).map(|(_, id)| Some(*id)).collect::<Vec<_>>(),
    };
    let mut peak_lanes = reservations.occupants.len().max(1);
    let mut rows = Vec::with_capacity(order.len());
    for (depth, node_id) in order {
        let lane = reservations.reserve(node_id);
        let mut frontier = topology::Frontier::new(max_lanes)?;
        frontier.vertical(&reservations.occupants, 1)?; // up
        reservations.occupants[lane] = None;
        if let Some(targets) = outgoing.get(&node_id) {
            for target in targets {
                // The first new child inherits the parent's lane. Existing join
                // targets keep their reserved lane until all parents have run.
                let target_lane =
                    if let Some(index) = reservations.occupants.iter().position(|value| *value == Some(*target)) {
                        index
                    } else if reservations.occupants[lane].is_none() {
                        reservations.occupants[lane] = Some(*target);
                        lane
                    } else {
                        reservations.reserve(*target)
                    };
                frontier.connect(topology::Connection {
                    source_lane: lane,
                    target_lane,
                })?;
            }
        }
        peak_lanes = peak_lanes.max(reservations.occupants.len());
        frontier.vertical(&reservations.occupants, 2)?; // down
        let visible_lane = frontier.visible_lane(lane);
        let connectors = frontier.finish(lane)?;
        rows.push(Row {
            node_id,
            label: workflow.graph.node(node_id).map(|node| node.label.clone()).unwrap_or_default(),
            kind: node_kind_label(workflow.node_config(node_id)),
            depth,
            lane: visible_lane,
            connectors,
        });
        while reservations.occupants.last() == Some(&None) {
            reservations.occupants.pop();
        }
    }
    Ok(Layout {
        rows,
        lane_count: peak_lanes.min(max_lanes),
        collapsed_lanes: peak_lanes.saturating_sub(max_lanes),
    })
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

pub struct RenderData<'a> {
    pub statuses: &'a HashMap<u64, NodeRunStatus>,
    pub started: &'a HashMap<u64, std::time::Instant>,
    pub start_times: &'a HashMap<u64, String>,
    pub durations_millis: &'a HashMap<u64, u64>,
    pub tokens: &'a HashMap<u64, u64>,
    pub costs_microunits: &'a HashMap<u64, u64>,
    pub attempts: &'a HashMap<u64, u32>,
    pub outputs: &'a [RunOutputLine],
    pub selected: &'a BTreeSet<NodeId>,
    pub focused: Option<NodeId>,
    pub scroll: usize,
}

pub(crate) fn visible_count(height: u16) -> usize {
    usize::from(height.saturating_sub(if height >= 3 { 2 } else { 1 }))
}

/// Keep the focused row on screen without resetting the viewport on every tick.
pub(crate) fn scroll_for_focus(layout: &Layout, focused: Option<NodeId>, scroll: usize, height: u16) -> usize {
    let count = visible_count(height);
    let mut scroll = scroll.min(layout.rows.len().saturating_sub(count));
    if let Some(index) = focused.and_then(|id| layout.rows.iter().position(|row| row.node_id == id)) {
        if index < scroll {
            scroll = index;
        } else if index >= scroll.saturating_add(count) {
            scroll = index.saturating_add(1).saturating_sub(count);
        }
    }
    scroll
}

// r[impl lattice.tui_ux.rows.metadata]
pub fn render(layout: &Layout, workflow: &Workflow, data: RenderData<'_>, area: Rect, buffer: &mut Buffer) {
    if area.is_empty() {
        return;
    }
    let columns = rendering::Columns::for_area(layout, area);
    let mut lines = Vec::with_capacity(usize::from(area.height));
    lines.push(rendering::header(layout, &columns));
    for (index, row) in layout.rows.iter().enumerate().skip(data.scroll).take(visible_count(area.height)) {
        lines.push(rendering::row_line(index, row, workflow, &data, &columns));
    }
    Paragraph::new(lines).render(area, buffer);
    if area.height >= 3 {
        rendering::totals(layout, &data, area, buffer);
    }
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

fn duration_text(finished_millis: Option<&u64>, started: Option<&std::time::Instant>) -> String {
    let millis = finished_millis
        .map(|millis| u128::from(*millis))
        .or_else(|| started.map(|time| time.elapsed().as_millis()));
    duration_millis_text(millis)
}

fn duration_millis_text(millis: Option<u128>) -> String {
    match millis {
        Some(millis) if millis >= 1_000 => format!("{:.1}s", millis as f64 / 1_000.0),
        Some(millis) => format!("{millis}ms"),
        None => "—".to_string(),
    }
}

fn tokens_text(tokens: Option<&u64>) -> String {
    tokens.map_or_else(|| "—".to_string(), u64::to_string)
}

fn cost_text(cost: Option<&u64>) -> String {
    cost.map_or_else(|| "—".to_string(), |cost| format!("{cost}µ"))
}

fn bounded_chars(text: &str, maximum: usize) -> String {
    let mut chars = text.chars().map(|ch| if ch.is_control() { ' ' } else { ch });
    let mut bounded = chars.by_ref().take(maximum).collect::<String>();
    if chars.next().is_some() && maximum > 0 {
        bounded.pop();
        bounded.push('…');
    }
    bounded
}

#[cfg(test)]
mod tests;

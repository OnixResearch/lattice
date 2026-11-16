//! Monitoring row text and aggregate rendering.

use ratatui::widgets::Widget;

pub(super) struct Columns {
    pub(super) lane_width: usize,
    pub(super) label_width: usize,
    pub(super) has_usage_columns: bool,
}

impl Columns {
    pub(super) fn for_area(layout: &super::Layout, area: ratatui::layout::Rect) -> Self {
        Self {
            lane_width: layout.lane_count.saturating_mul(2).saturating_sub(1).max(3).min(usize::from(area.width)),
            label_width: if area.width >= 100 { 20 } else { 14 },
            has_usage_columns: area.width >= 130,
        }
    }
}

pub(super) fn header(layout: &super::Layout, columns: &Columns) -> ratatui::text::Line<'static> {
    let lane_width = columns.lane_width;
    let label_width = columns.label_width;
    let usage_header = usage_heading(columns.has_usage_columns);
    let lanes_header = lane_heading(layout);
    ratatui::text::Line::styled(
        format!(
            "  # {lanes_header:<lane_width$} {:<label_width$} {:<9} {:<8} {:<8} {:<3}{usage_header}OUTPUT",
            "NODE", "STATE", "START", "DURATION", "ATT"
        ),
        ratatui::style::Style::new().fg(ratatui::style::Color::DarkGray),
    )
}

pub(super) fn row_line(
    index: usize,
    row: &super::Row,
    workflow: &crate::engine::workflow::Workflow,
    data: &super::RenderData<'_>,
    columns: &Columns,
) -> ratatui::text::Line<'static> {
    let id = row.node_id.0;
    let (marker, style) = selection(row.node_id, data);
    let usage = usage_cells(row, data, columns.has_usage_columns);
    let text = format!(
        "{marker}{index:>2} {:<lane_width$} {:<label_width$} {:<9} {:<8} {:<8} {:<3}{usage}{output}",
        row.connectors,
        super::bounded_chars(&row.label, columns.label_width),
        super::status_text(data.statuses.get(&id).copied()),
        data.start_times.get(&id).map(String::as_str).unwrap_or("—"),
        super::duration_text(data.durations_millis.get(&id), data.started.get(&id)),
        data.attempts.get(&id).map(u32::to_string).unwrap_or_else(|| "—".to_string()),
        lane_width = columns.lane_width,
        label_width = columns.label_width,
        output = output_summary(workflow, row.node_id, data.outputs),
    );
    ratatui::text::Line::styled(text, style)
}

fn lane_heading(layout: &super::Layout) -> String {
    if layout.collapsed_lanes > 0 {
        format!("+{}", layout.collapsed_lanes)
    } else {
        "DAG".to_string()
    }
}

fn usage_heading(has_usage_columns: bool) -> &'static str {
    if has_usage_columns {
        " TOKENS  COST      KIND       "
    } else {
        " "
    }
}

fn selection(
    node_id: crate::engine::workflow::NodeId,
    data: &super::RenderData<'_>,
) -> (&'static str, ratatui::style::Style) {
    if data.selected.contains(&node_id) {
        ("▶", ratatui::style::Style::new().fg(ratatui::style::Color::Yellow))
    } else if data.focused == Some(node_id) {
        (">", ratatui::style::Style::new().fg(ratatui::style::Color::Cyan))
    } else {
        (" ", ratatui::style::Style::new())
    }
}

fn usage_cells(row: &super::Row, data: &super::RenderData<'_>, has_usage_columns: bool) -> String {
    if !has_usage_columns {
        return " ".to_string();
    }
    let id = row.node_id.0;
    format!(
        " {:<7} {:<9} {:<10} ",
        super::tokens_text(data.tokens.get(&id)),
        super::cost_text(data.costs_microunits.get(&id)),
        row.kind,
    )
}

fn output_summary(
    workflow: &crate::engine::workflow::Workflow,
    node_id: crate::engine::workflow::NodeId,
    outputs: &[crate::tui::app::RunOutputLine],
) -> String {
    // Choose the first meaningful, non-metadata output in declared port order,
    // rather than a shell's trailing exit-code port. No full output clones.
    workflow
        .graph
        .node(node_id)
        .and_then(|node| {
            node.output_ports.iter().filter(|port| port.label != "metadata").find_map(|port| {
                outputs.iter().rev().find(|record| {
                    record.node_id == node_id && record.port_id == port.id && !record.text.trim().is_empty()
                })
            })
        })
        .map(|record| {
            format!("{}: {}", record.port_label, super::bounded_chars(&record.text, super::OUTPUT_SUMMARY_MAX_CHARS))
        })
        .unwrap_or_default()
}

fn totals_text(layout: &super::Layout, data: &super::RenderData<'_>) -> String {
    let total_duration_millis = data.durations_millis.values().map(|value| u128::from(*value)).sum::<u128>();
    let total_tokens = data.tokens.values().map(|value| u128::from(*value)).sum::<u128>();
    let total_cost_microunits = data.costs_microunits.values().map(|value| u128::from(*value)).sum::<u128>();
    let total_attempts = data.attempts.values().map(|value| u128::from(*value)).sum::<u128>();
    let collapsed = if layout.collapsed_lanes > 0 {
        format!("+{} lanes collapsed; ◆ shares last lane | ", layout.collapsed_lanes)
    } else {
        String::new()
    };
    format!(
        "{collapsed}TOTAL observed time {} tokens {total_tokens} cost {total_cost_microunits}µ attempts {total_attempts}",
        super::duration_millis_text(Some(total_duration_millis)),
    )
}

pub(super) fn totals(
    layout: &super::Layout,
    data: &super::RenderData<'_>,
    area: ratatui::layout::Rect,
    buffer: &mut ratatui::buffer::Buffer,
) {
    // A clipped terminal rectangle may not have a representable footer row.
    let Some(y) = area.height.checked_sub(1).and_then(|offset| area.y.checked_add(offset)) else {
        return;
    };
    ratatui::widgets::Paragraph::new(totals_text(layout, data))
        .style(ratatui::style::Style::new().fg(ratatui::style::Color::DarkGray))
        .render(ratatui::layout::Rect::new(area.x, y, area.width, 1), buffer);
}

//! Replay and per-node execution detail rendering.

use super::*;

pub(super) struct DetailView<'a> {
    pub(super) record: &'a RunDetailRecord,
    pub(super) selected_node: usize,
    pub(super) replay_timeline: Option<&'a ReplayTimeline>,
    pub(super) replay_cursor: ReplayCursor,
}

// r[impl n8nclone.tui_ux.run_history.timeline]
// r[impl n8nclone.tui_ux.run_history.node_outputs]
pub(super) fn render_run_detail_record(view: DetailView<'_>, area: Rect, buf: &mut Buffer) {
    debug_assert!(area.right() <= buf.area.right());
    debug_assert!(area.bottom() <= buf.area.bottom());
    let DetailView {
        record,
        selected_node,
        replay_timeline,
        replay_cursor,
    } = view;
    if record.nodes.is_empty() {
        Paragraph::new("No node details available")
            .alignment(Alignment::Center)
            .style(
                Style {
                    fg: None,
                    bg: None,
                    underline_color: None,
                    add_modifier: Modifier::empty(),
                    sub_modifier: Modifier::empty(),
                }
                .fg(Color::DarkGray),
            )
            .render(area, buf);
        return;
    }

    let selected_node = selected_node.min(record.nodes.len().saturating_sub(1));
    let selected_detail = &record.nodes[selected_node];
    let mut lines = record
        .nodes
        .iter()
        .enumerate()
        .map(|(index, detail)| run_detail_line(index == selected_node, detail))
        .collect::<Vec<_>>();

    lines.push(Line::from(""));
    lines.push(Line::styled(
        format!("Outputs for {}", selected_detail.node_label),
        Style {
            fg: None,
            bg: None,
            underline_color: None,
            add_modifier: Modifier::empty(),
            sub_modifier: Modifier::empty(),
        }
        .fg(Color::DarkGray),
    ));
    lines = append_selected_run_detail_output_lines(lines, selected_detail);
    lines = append_replay_lines(lines, replay_timeline, replay_cursor);

    Paragraph::new(lines).render(area, buf);
}

// r[impl n8nclone.tui_ux.replay_debugger.timeline]
// r[impl n8nclone.tui_ux.replay_debugger.redaction]
pub(super) fn append_replay_lines(
    mut lines: Vec<Line<'static>>,
    replay_timeline: Option<&ReplayTimeline>,
    replay_cursor: ReplayCursor,
) -> Vec<Line<'static>> {
    let Some(timeline) = replay_timeline else {
        return lines;
    };
    lines.push(Line::from(""));
    lines.push(Line::styled("Replay Timeline", Style::new().fg(Color::DarkGray)));
    if timeline.frames.is_empty() {
        lines.push(Line::styled("No replay events available", Style::new().fg(Color::DarkGray)));
        return lines;
    }
    lines.extend(timeline.frames.iter().map(|frame| replay_frame_line(frame.cursor == replay_cursor.position, frame)));
    append_replay_selection(lines, timeline, replay_cursor)
}

fn append_replay_selection(
    mut lines: Vec<Line<'static>>,
    timeline: &ReplayTimeline,
    cursor: ReplayCursor,
) -> Vec<Line<'static>> {
    if let Some(frame) = timeline.frames.get(cursor.position) {
        lines.push(Line::styled(
            format!("selected: #{} {}", frame.event.sequence, frame.event.summary),
            Style::new().fg(Color::Gray),
        ));
    }
    lines
}

pub(super) fn replay_frame_line(selected: bool, frame: &ReplayFrame) -> Line<'static> {
    let node = frame.event.node_id.map(|node_id| format!(" node {node_id}")).unwrap_or_default();
    let redaction = if frame.event.redacted { " redacted" } else { "" };
    let prefix = if selected { "› " } else { "  " };
    let style = if selected {
        Style::new().fg(Color::Black).bg(Color::Blue)
    } else {
        Style::new().fg(Color::Gray)
    };
    Line::styled(format!("{prefix}#{} {:?}{node}{redaction}", frame.event.sequence, frame.event.kind), style)
}

pub(super) fn run_detail_line(selected: bool, detail: &RunNodeDetail) -> Line<'static> {
    let status = run_detail_status_label(detail.status);
    let cache = if detail.cached { " cached" } else { "" };
    let prefix = if selected { "› " } else { "  " };
    let style = if selected {
        Style::new().fg(Color::Black).bg(Color::Magenta)
    } else {
        run_detail_status_style(detail.status)
    };
    Line::styled(format!("{prefix}{} — {status}{cache}", detail.node_label), style)
}

pub(super) fn append_selected_run_detail_output_lines(
    mut lines: Vec<Line<'static>>,
    detail: &RunNodeDetail,
) -> Vec<Line<'static>> {
    if let Some(error) = &detail.error {
        lines.push(Line::styled(format!("error: {error}"), Style::new().fg(Color::Red)));
    }
    if detail.outputs.is_empty() && detail.error.is_none() {
        lines.push(Line::styled("No output details", Style::new().fg(Color::DarkGray)));
        return lines;
    }
    lines.extend(
        detail.outputs.iter().map(|output| {
            Line::styled(format!("{}: {}", output.port_label, output.text), Style::new().fg(Color::Gray))
        }),
    );
    lines
}

pub(super) fn run_detail_status_label(status: RunNodeDetailStatus) -> &'static str {
    match status {
        RunNodeDetailStatus::Pending => "pending",
        RunNodeDetailStatus::Running => "running",
        RunNodeDetailStatus::Completed => "completed",
        RunNodeDetailStatus::Failed => "failed",
        RunNodeDetailStatus::Skipped => "skipped",
    }
}

pub(super) fn run_detail_status_style(status: RunNodeDetailStatus) -> Style {
    match status {
        RunNodeDetailStatus::Pending => Style {
            fg: None,
            bg: None,
            underline_color: None,
            add_modifier: Modifier::empty(),
            sub_modifier: Modifier::empty(),
        }
        .fg(Color::DarkGray),
        RunNodeDetailStatus::Running => Style {
            fg: None,
            bg: None,
            underline_color: None,
            add_modifier: Modifier::empty(),
            sub_modifier: Modifier::empty(),
        }
        .fg(Color::Yellow),
        RunNodeDetailStatus::Completed => Style {
            fg: None,
            bg: None,
            underline_color: None,
            add_modifier: Modifier::empty(),
            sub_modifier: Modifier::empty(),
        }
        .fg(Color::Green),
        RunNodeDetailStatus::Failed => Style {
            fg: None,
            bg: None,
            underline_color: None,
            add_modifier: Modifier::empty(),
            sub_modifier: Modifier::empty(),
        }
        .fg(Color::Red),
        RunNodeDetailStatus::Skipped => Style {
            fg: None,
            bg: None,
            underline_color: None,
            add_modifier: Modifier::empty(),
            sub_modifier: Modifier::empty(),
        }
        .fg(Color::Gray),
    }
}

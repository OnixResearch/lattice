//! Spatial canvas and topological monitoring presentation.

use super::*;

/// Render the node graph canvas.
pub(in crate::tui::panels) fn render_surface(app: &mut crate::tui::app::State, area: Rect, buf: &mut Buffer) {
    let is_focused = matches!(app.focus, FocusPanel::Canvas);
    let border_style = Style::new().fg(if is_focused { Color::Cyan } else { Color::DarkGray });

    let title = if app.canvas_view == crate::tui::rows::CanvasView::Rows {
        " Rows (v: graph) "
    } else {
        " Canvas "
    };
    let block = Block::new().borders(Borders::ALL).title(title).border_style(border_style);

    let inner = block.inner(area);
    debug_assert!(inner.width <= area.width);
    debug_assert!(inner.height <= area.height);
    block.render(area, buf);
    app.rows_hit_area = None;

    if should_show_empty_state(app.graph_state.graph.node_count(), app.show_palette) {
        render_empty_state(inner, buf);
        return;
    }

    if app.canvas_view == crate::tui::rows::CanvasView::Rows {
        render_monitoring_rows(app, inner, buf);
        return;
    }

    app.rows_hit_area = None;
    let widget = NodeGraphWidget {
        type_color_fn: None,
        node_style: Style::new().fg(Color::White),
        selected_style: Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        edge_style: Style::new().fg(Color::DarkGray),
    };
    widget.render(inner, buf, &mut app.graph_state);
    if let Some(summary) = image_preview_summary(app) {
        let preview_y = inner.y.saturating_add(inner.height.saturating_sub(CANVAS_PREVIEW_HEIGHT));
        let preview_area = Rect::new(inner.x, preview_y, inner.width, CANVAS_PREVIEW_HEIGHT);
        Paragraph::new(summary).style(Style::new().fg(Color::Cyan)).render(preview_area, buf);
    }
}

fn render_monitoring_rows(app: &mut crate::tui::app::State, area: Rect, buf: &mut Buffer) {
    debug_assert!(area.right() <= buf.area.right());
    debug_assert!(area.bottom() <= buf.area.bottom());
    let layout = match crate::tui::rows::layout(&app.workflow, crate::tui::rows::max_lanes_for_width(area.width)) {
        Ok(layout) => layout,
        Err(error) => {
            Paragraph::new(format!("Rows layout failed: {error}"))
                .style(Style::new().fg(Color::Red))
                .render(area, buf);
            return;
        }
    };
    app.rows_hit_area = Some(area);
    let selected = app
        .graph_state
        .selected
        .iter()
        .map(|id| crate::tui::graph::workflow_node_id(*id))
        .collect::<std::collections::BTreeSet<_>>();
    let focused = app.graph_state.focused.map(crate::tui::graph::workflow_node_id);
    app.rows_scroll = crate::tui::rows::scroll_for_focus(&layout, focused, app.rows_scroll, area.height);
    crate::tui::rows::render(
        &layout,
        &app.workflow,
        crate::tui::rows::RenderData {
            statuses: &app.run_state.node_statuses,
            started: &app.run_state.node_started,
            start_times: &app.run_state.node_start_times,
            durations_millis: &app.run_state.node_durations_millis,
            tokens: &app.run_state.node_tokens,
            costs_microunits: &app.run_state.node_cost_microunits,
            attempts: &app.run_state.node_attempts,
            outputs: &app.run_output_lines,
            selected: &selected,
            focused,
            scroll: app.rows_scroll,
        },
        area,
        buf,
    );
}

pub(in crate::tui::panels) fn image_preview_summary(app: &crate::tui::app::State) -> Option<String> {
    let node_id = app.graph_state.focused?;
    crate::tui::app::latest_node_output_records(&app.run_output_lines, &app.workflow, node_id)
        .into_iter()
        .rev()
        .find(|record| record.text.starts_with("image/") && record.text.contains(" image "))
        .map(|record| bounded_preview(&format!("Image preview: {}", record.text)))
}

pub(in crate::tui::panels) fn bounded_preview(summary: &str) -> String {
    let mut chars = summary.chars();
    let bounded = chars.by_ref().take(CANVAS_IMAGE_PREVIEW_MAX_CHARS).collect::<String>();
    if chars.next().is_none() {
        return bounded;
    }
    let mut truncated = bounded.chars().take(CANVAS_IMAGE_PREVIEW_MAX_CHARS.saturating_sub(1)).collect::<String>();
    truncated.push('…');
    truncated
}

pub(in crate::tui::panels) fn render_empty_state(area: Rect, buf: &mut Buffer) {
    let copy = empty_state_copy(area);
    let mut lines = vec![Line::styled(
        copy.title,
        Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
    )];

    for hint in copy.hints {
        lines.push(Line::styled(*hint, Style::new().fg(Color::DarkGray)));
    }

    Paragraph::new(lines).alignment(Alignment::Center).render(area, buf);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::tui::panels) struct EmptyStateCopy {
    pub(in crate::tui::panels) title: &'static str,
    pub(in crate::tui::panels) hints: &'static [&'static str],
}

// r[impl n8nclone.tui_ux.empty_state.show] r[impl n8nclone.tui_ux.empty_state.compact]
pub(in crate::tui::panels) fn empty_state_copy(area: Rect) -> EmptyStateCopy {
    if area.width < FULL_EMPTY_STATE_MIN_WIDTH || area.height < FULL_EMPTY_STATE_MIN_HEIGHT {
        return EmptyStateCopy {
            title: "Empty workflow",
            hints: &["Ctrl-N add node │ Ctrl-P palette"],
        };
    }

    EmptyStateCopy {
        title: "Start your workflow",
        hints: &[
            "Ctrl-N: add your first node",
            "Ctrl-P: open the command palette",
            "Drag nodes and connect ports to build a flow",
        ],
    }
}

// r[impl n8nclone.tui_ux.empty_state.hide]
pub(in crate::tui::panels) fn should_show_empty_state(node_count: usize, modal_open: bool) -> bool {
    node_count == 0 && !modal_open
}

//! Panel rendering: properties sidebar, output log, status bar.

use crate::engine::workflow::NodeConfig;
use crate::tui::app::{App, FocusPanel};
use rat_fuzzy::{FuzzyFinder, FuzzyStyle};
use rat_nodegraph::model::NodeId;
use rat_nodegraph::view::{InteractionMode, NodeGraphWidget};
use rat_streaming::render_streaming_lines;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, StatefulWidget, Widget};

/// Render the full application layout.
pub fn render(app: &mut App, area: Rect, buf: &mut Buffer) {
    // Top-level vertical split: main area + output + status bar.
    let vert = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(8),       // canvas + properties
            Constraint::Length(10),    // output log
            Constraint::Length(1),     // status bar
        ])
        .split(area);

    // Horizontal split for canvas and properties panel.
    let horiz = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(70), // canvas
            Constraint::Percentage(30), // properties
        ])
        .split(vert[0]);

    render_canvas(app, horiz[0], buf);
    render_properties(app, horiz[1], buf);
    render_output(app, vert[1], buf);
    render_status_bar(app, vert[2], buf);

    // Overlay: fuzzy palette.
    if app.show_palette {
        render_palette(app, area, buf);
    }
}

/// Render the node graph canvas.
fn render_canvas(app: &mut App, area: Rect, buf: &mut Buffer) {
    let is_focused = matches!(app.focus, FocusPanel::Canvas);
    let border_style = if is_focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Canvas ")
        .border_style(border_style);

    let inner = block.inner(area);
    block.render(area, buf);

    let widget = NodeGraphWidget::default();
    widget.render(inner, buf, &mut app.graph_state);
}

/// Render the properties panel for the selected node.
fn render_properties(app: &App, area: Rect, buf: &mut Buffer) {
    let is_focused = matches!(app.focus, FocusPanel::Properties);
    let border_style = if is_focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Properties ")
        .border_style(border_style);

    let inner = block.inner(area);
    block.render(area, buf);

    let selected: Vec<NodeId> = app.graph_state.selected.iter().copied().collect();

    if selected.len() != 1 {
        let hint = if selected.is_empty() {
            "Select a node to edit"
        } else {
            "Multiple nodes selected"
        };
        Paragraph::new(hint)
            .style(Style::default().fg(Color::DarkGray))
            .render(inner, buf);
        return;
    }

    let node_id = selected[0];
    let node = match app.graph_state.graph.node(node_id) {
        Some(n) => n,
        None => return,
    };

    let mut lines = vec![
        Line::from(vec![
            Span::styled("Node: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                node.label.clone(),
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("ID: ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{}", node_id), Style::default().fg(Color::Gray)),
        ]),
        Line::from(""),
    ];

    // Show config details.
    if let Some(config) = app.workflow.node_config(node_id) {
        match config {
            NodeConfig::Shell {
                command,
                cwd,
                timeout_secs,
                pipe_stdin,
                ..
            } => {
                lines.push(Line::from(vec![
                    Span::styled("Type: ", Style::default().fg(Color::DarkGray)),
                    Span::styled("Shell", Style::default().fg(Color::Green)),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("Cmd:  ", Style::default().fg(Color::DarkGray)),
                    Span::styled(command.clone(), Style::default().fg(Color::Cyan)),
                ]));
                if let Some(dir) = cwd {
                    lines.push(Line::from(vec![
                        Span::styled("CWD:  ", Style::default().fg(Color::DarkGray)),
                        Span::styled(dir.clone(), Style::default().fg(Color::Gray)),
                    ]));
                }
                lines.push(Line::from(vec![
                    Span::styled("Timeout: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(format!("{timeout_secs}s"), Style::default().fg(Color::Gray)),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("Pipe stdin: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        if *pipe_stdin { "yes" } else { "no" },
                        Style::default().fg(Color::Gray),
                    ),
                ]));
            }
        }
    }

    // Show ports.
    lines.push(Line::from(""));
    lines.push(Line::styled(
        "─ Inputs ─",
        Style::default().fg(Color::DarkGray),
    ));
    for port in &node.input_ports {
        lines.push(Line::from(vec![
            Span::styled("● ", Style::default().fg(Color::Cyan)),
            Span::raw(port.label.clone()),
            Span::styled(
                format!(" ({})", port.type_tag),
                Style::default().fg(Color::DarkGray),
            ),
        ]));
    }
    lines.push(Line::styled(
        "─ Outputs ─",
        Style::default().fg(Color::DarkGray),
    ));
    for port in &node.output_ports {
        lines.push(Line::from(vec![
            Span::styled("● ", Style::default().fg(Color::Green)),
            Span::raw(port.label.clone()),
            Span::styled(
                format!(" ({})", port.type_tag),
                Style::default().fg(Color::DarkGray),
            ),
        ]));
    }

    Paragraph::new(lines).render(inner, buf);
}

/// Render the output/execution log panel.
fn render_output(app: &mut App, area: Rect, buf: &mut Buffer) {
    let is_focused = matches!(app.focus, FocusPanel::Output);
    let border_style = if is_focused {
        Style::default().fg(Color::Magenta)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Output ")
        .border_style(border_style);

    let inner = block.inner(area);
    block.render(area, buf);

    // Render streaming output lines.
    let visible_height = inner.height as usize;
    let rendered_lines = render_streaming_lines(
        &mut app.output,
        visible_height,
        Style::default().fg(Color::DarkGray),
    );

    for (i, line) in rendered_lines.iter().enumerate() {
        if i >= inner.height as usize {
            break;
        }
        let y = inner.y + i as u16;
        let mut x = inner.x;
        for span in &line.spans {
            let text = span.content.as_ref();
            for ch in text.chars() {
                if x < inner.x + inner.width {
                    buf[(x, y)].set_char(ch).set_style(span.style);
                    x += 1;
                }
            }
        }
    }
}

/// Render the status bar.
fn render_status_bar(app: &App, area: Rect, buf: &mut Buffer) {
    let mode_span = match &app.graph_state.mode {
        InteractionMode::Wiring { .. } => {
            Span::styled(" WIRING ", Style::default().bg(Color::Yellow).fg(Color::Black))
        }
        _ => match app.focus {
            FocusPanel::Canvas => {
                Span::styled(" CANVAS ", Style::default().bg(Color::Cyan).fg(Color::Black))
            }
            FocusPanel::Properties => {
                Span::styled(" PROPS ", Style::default().bg(Color::Yellow).fg(Color::Black))
            }
            FocusPanel::Output => {
                Span::styled(" OUTPUT ", Style::default().bg(Color::Magenta).fg(Color::Black))
            }
        },
    };

    let wf_name = Span::styled(
        format!(" {} ", app.workflow.meta.name),
        Style::default().fg(Color::White),
    );

    let node_count = app.graph_state.graph.node_count();
    let edge_count = app.graph_state.graph.edge_count();
    let stats = Span::styled(
        format!(" {node_count} nodes │ {edge_count} edges "),
        Style::default().fg(Color::DarkGray),
    );

    let help_text = match &app.graph_state.mode {
        InteractionMode::Wiring { .. } => {
            " Click target port to connect │ Esc: cancel "
        }
        _ => " C-P: palette │ C-R: run │ C-N: add │ C-L: layout ",
    };
    let help = Span::styled(help_text, Style::default().fg(Color::DarkGray));

    let line = Line::from(vec![mode_span, wf_name, stats, help]);
    Paragraph::new(line)
        .style(Style::default().bg(Color::Black))
        .render(area, buf);
}

/// Render the fuzzy command palette overlay.
fn render_palette(app: &mut App, area: Rect, buf: &mut Buffer) {
    let width = area.width.min(60);
    let height = area.height.min(20);
    let x = (area.width.saturating_sub(width)) / 2;
    let y = 2;
    let palette_area = Rect::new(x, y, width, height);

    // Clear background.
    Clear.render(palette_area, buf);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Command Palette (Ctrl-P) ")
        .border_style(Style::default().fg(Color::Yellow));

    let inner = block.inner(palette_area);
    block.render(palette_area, buf);

    // Render fuzzy finder.
    let style = FuzzyStyle::default();
    let widget = FuzzyFinder::new(&app.palette_data, style);
    widget.render(inner, buf, &mut app.palette_state);
}

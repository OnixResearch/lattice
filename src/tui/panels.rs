//! Panel rendering: properties sidebar, output log, status bar.

use rat_fuzzy::FuzzyFinder;
use rat_fuzzy::FuzzyStyle;
use rat_nodegraph::model::NodeId;
use rat_nodegraph::view::InteractionMode;
use rat_nodegraph::view::NodeGraphWidget;
use rat_streaming::render_streaming_lines;
use ratatui::buffer::Buffer;
use ratatui::layout::Alignment;
use ratatui::layout::Constraint;
use ratatui::layout::Direction;
use ratatui::layout::Layout;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Clear;
use ratatui::widgets::Paragraph;
use ratatui::widgets::StatefulWidget;
use ratatui::widgets::Widget;

use crate::node_kind_provider;
use crate::replay::ReplayCursor;
use crate::replay::ReplayFrame;
use crate::replay::ReplayTimeline;
use crate::store::RunDetailRecord;
use crate::store::RunNodeDetail;
use crate::store::RunNodeDetailStatus;
use crate::tui::app::AddNodeKind;
use crate::tui::app::AddNodeStep;
use crate::tui::app::App;
use crate::tui::app::FocusPanel;
use crate::tui::app::NodeRunStatus;
use crate::tui::app::PropertyEditField;
use crate::tui::app::credentials_to_draft;
use crate::tui::app::rendered_latest_node_output_lines;
use crate::workflow_validation;
use crate::workflow_validation::DiagnosticSeverity;

const COMPACT_STATUS_HELP_WIDTH: u16 = 72;
const FULL_EMPTY_STATE_MIN_WIDTH: u16 = 48;
const FULL_EMPTY_STATE_MIN_HEIGHT: u16 = 7;
const QUIT_CONFIRM_WIDTH: u16 = 62;
const QUIT_CONFIRM_HEIGHT: u16 = 5;
const QUIT_CONFIRM_TOP_OFFSET: u16 = 3;
const WORKFLOW_BROWSER_WIDTH: u16 = 64;
const WORKFLOW_BROWSER_HEIGHT: u16 = 16;
const WORKFLOW_BROWSER_TOP_OFFSET: u16 = 2;
const ADD_NODE_WIDTH: u16 = 66;
const ADD_NODE_HEIGHT: u16 = 14;
const ADD_NODE_TOP_OFFSET: u16 = 2;
const RUN_HISTORY_WIDTH: u16 = 72;
const RUN_HISTORY_HEIGHT: u16 = 16;
const RUN_HISTORY_TOP_OFFSET: u16 = 2;
const DIAGNOSTICS_WIDTH: u16 = 78;
const DIAGNOSTICS_HEIGHT: u16 = 16;
const DIAGNOSTICS_TOP_OFFSET: u16 = 2;

/// Render the full application layout.
pub fn render(app: &mut App, area: Rect, buf: &mut Buffer) {
    // Top-level vertical split: main area + output + status bar.
    let vert = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(8),     // canvas + properties
            Constraint::Length(10), // output log
            Constraint::Length(1),  // status bar
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

    if app.is_diagnostics_open() {
        render_diagnostics(app, area, buf);
    }

    if app.is_run_history_open() {
        render_run_history(app, area, buf);
    }

    if app.is_workflow_browser_open() {
        render_workflow_browser(app, area, buf);
    }

    if app.is_add_node_flow_open() {
        render_add_node_flow(app, area, buf);
    }

    if app.is_confirming_quit() {
        render_quit_confirmation(area, buf);
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

    let block = Block::default().borders(Borders::ALL).title(" Canvas ").border_style(border_style);

    let inner = block.inner(area);
    block.render(area, buf);

    if should_show_empty_state(app.graph_state.graph.node_count(), app.show_palette) {
        render_empty_canvas_state(inner, buf);
        return;
    }

    let widget = NodeGraphWidget::default();
    widget.render(inner, buf, &mut app.graph_state);
}

fn render_empty_canvas_state(area: Rect, buf: &mut Buffer) {
    let copy = empty_state_copy(area.width, area.height);
    let mut lines = vec![Line::styled(
        copy.title,
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
    )];

    for hint in copy.hints {
        lines.push(Line::from(Span::styled(*hint, Style::default().fg(Color::DarkGray))));
    }

    Paragraph::new(lines).alignment(Alignment::Center).render(area, buf);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct EmptyStateCopy {
    title: &'static str,
    hints: &'static [&'static str],
}

// r[impl n8nclone.tui_ux.empty_state.show] r[impl n8nclone.tui_ux.empty_state.compact]
fn empty_state_copy(width: u16, height: u16) -> EmptyStateCopy {
    if width < FULL_EMPTY_STATE_MIN_WIDTH || height < FULL_EMPTY_STATE_MIN_HEIGHT {
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
fn should_show_empty_state(node_count: usize, modal_open: bool) -> bool {
    node_count == 0 && !modal_open
}

/// Render the properties panel for the selected node.
fn render_properties(app: &App, area: Rect, buf: &mut Buffer) {
    let is_focused = matches!(app.focus, FocusPanel::Properties);
    let border_style = if is_focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default().borders(Borders::ALL).title(" Properties ").border_style(border_style);

    let inner = block.inner(area);
    block.render(area, buf);

    let selected: Vec<NodeId> = app.graph_state.selected.iter().copied().collect();

    if selected.len() != 1 {
        let hint = if selected.is_empty() {
            "Select a node to edit"
        } else {
            "Multiple nodes selected"
        };
        Paragraph::new(hint).style(Style::default().fg(Color::DarkGray)).render(inner, buf);
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
            Span::styled(node.label.clone(), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("ID: ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{}", node_id), Style::default().fg(Color::Gray)),
        ]),
        Line::from(""),
    ];

    if let Some(status) = app.run_state.node_statuses.get(&node_id.0).copied() {
        lines.push(Line::from(vec![
            Span::styled("Run: ", Style::default().fg(Color::DarkGray)),
            Span::styled(run_status_label(status), run_status_style(status)),
        ]));
        lines.push(Line::from(""));
    }

    if let Some(edit) = &app.property_edit
        && edit.node_id == node_id
    {
        lines.push(Line::from(vec![
            Span::styled("Editing: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                property_field_title(edit.field),
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ),
        ]));
        lines.push(Line::from(vec![
            Span::styled("Draft: ", Style::default().fg(Color::DarkGray)),
            Span::styled(edit.draft.clone(), Style::default().fg(Color::White)),
        ]));
        lines.push(Line::styled("Enter: save │ Esc: cancel", Style::default().fg(Color::DarkGray)));
        lines.push(Line::from(""));
    }

    // Show config details.
    // r[impl n8nclone.tui_ux.node_kind_provider.fallback]
    if let Some(config) = app.workflow.node_config(node_id) {
        for property in node_kind_provider::property_lines_for_config(config) {
            lines.push(Line::from(vec![
                Span::styled(format!("{}: ", property.label), Style::default().fg(Color::DarkGray)),
                Span::styled(property.value, Style::default().fg(Color::Cyan)),
            ]));
        }
        let credentials = credentials_to_draft(app.workflow.node_credentials(node_id));
        if !credentials.is_empty() {
            lines.push(Line::from(vec![
                Span::styled("Credentials: ", Style::default().fg(Color::DarkGray)),
                Span::styled(credentials, Style::default().fg(Color::Gray)),
            ]));
        }
    }

    // Show ports.
    lines.push(Line::from(""));
    lines.push(Line::styled("─ Inputs ─", Style::default().fg(Color::DarkGray)));
    for port in &node.input_ports {
        lines.push(Line::from(vec![
            Span::styled("● ", Style::default().fg(Color::Cyan)),
            Span::raw(port.label.clone()),
            Span::styled(format!(" ({})", port.type_tag), Style::default().fg(Color::DarkGray)),
        ]));
    }
    lines.push(Line::styled("─ Outputs ─", Style::default().fg(Color::DarkGray)));
    for port in &node.output_ports {
        lines.push(Line::from(vec![
            Span::styled("● ", Style::default().fg(Color::Green)),
            Span::raw(port.label.clone()),
            Span::styled(format!(" ({})", port.type_tag), Style::default().fg(Color::DarkGray)),
        ]));
    }

    let latest_output_lines = rendered_latest_node_output_lines(&app.run_output_lines, &app.workflow, node_id);
    if !latest_output_lines.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::styled("─ Latest output ─", Style::default().fg(Color::DarkGray)));
        for line in latest_output_lines {
            lines.push(Line::styled(line, Style::default().fg(Color::Gray)));
        }
    }

    Paragraph::new(lines).render(inner, buf);
}

fn run_status_label(status: NodeRunStatus) -> &'static str {
    match status {
        NodeRunStatus::Pending => "pending",
        NodeRunStatus::Running => "running",
        NodeRunStatus::Completed => "completed",
        NodeRunStatus::Failed => "failed",
        NodeRunStatus::Skipped => "skipped",
    }
}

fn run_status_style(status: NodeRunStatus) -> Style {
    match status {
        NodeRunStatus::Pending => Style::default().fg(Color::DarkGray),
        NodeRunStatus::Running => Style::default().fg(Color::Yellow),
        NodeRunStatus::Completed => Style::default().fg(Color::Green),
        NodeRunStatus::Failed => Style::default().fg(Color::Red),
        NodeRunStatus::Skipped => Style::default().fg(Color::Gray),
    }
}

fn property_field_title(field: PropertyEditField) -> &'static str {
    match field {
        PropertyEditField::Label => "Label",
        PropertyEditField::Command => "Command",
        PropertyEditField::TimeoutSecs => "Timeout",
        PropertyEditField::ImagePath => "Image Path",
        PropertyEditField::AgentPrompt => "Agent Prompt",
        PropertyEditField::AgentTarget => "Agent Target",
        PropertyEditField::AgentOutputMode => "Agent Output Mode",
        PropertyEditField::AgentModelHint => "Agent Model Hint",
        PropertyEditField::AgentToolPolicy => "Agent Tool Policy",
        PropertyEditField::AgentContextRefs => "Agent Context Refs",
        PropertyEditField::AgentCwd => "Agent CWD",
        PropertyEditField::AgentSession => "Agent Session",
        PropertyEditField::AgentCredentials => "Agent Credentials",
    }
}

/// Render the output/execution log panel.
fn render_output(app: &mut App, area: Rect, buf: &mut Buffer) {
    let is_focused = matches!(app.focus, FocusPanel::Output);
    let border_style = if is_focused {
        Style::default().fg(Color::Magenta)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default().borders(Borders::ALL).title(" Output ").border_style(border_style);

    let inner = block.inner(area);
    block.render(area, buf);

    // Render streaming output lines.
    let visible_height = inner.height as usize;
    let rendered_lines = render_streaming_lines(&mut app.output, visible_height, Style::default().fg(Color::DarkGray));

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
        InteractionMode::Wiring { .. } => Span::styled(" WIRING ", Style::default().bg(Color::Yellow).fg(Color::Black)),
        _ => match app.focus {
            FocusPanel::Canvas => Span::styled(" CANVAS ", Style::default().bg(Color::Cyan).fg(Color::Black)),
            FocusPanel::Properties => Span::styled(" PROPS ", Style::default().bg(Color::Yellow).fg(Color::Black)),
            FocusPanel::Output => Span::styled(" OUTPUT ", Style::default().bg(Color::Magenta).fg(Color::Black)),
        },
    };

    let wf_name = Span::styled(
        format!(" {}{} ", app.workflow.meta.name, unsaved_indicator(app.has_unsaved_changes()),),
        Style::default().fg(Color::White),
    );

    let node_count = app.graph_state.graph.node_count();
    let edge_count = app.graph_state.graph.edge_count();
    let stats =
        Span::styled(format!(" {node_count} nodes │ {edge_count} edges "), Style::default().fg(Color::DarkGray));

    let help_text = contextual_help_text(FooterHelpContext {
        focus: app.focus,
        palette_open: app.show_palette,
        workflow_browser_open: app.is_workflow_browser_open(),
        add_node_open: app.is_add_node_flow_open(),
        run_history_open: app.is_run_history_open(),
        diagnostics_open: app.is_diagnostics_open(),
        interaction_mode: footer_interaction_mode(&app.graph_state.mode),
        width: area.width,
    });
    let help = Span::styled(help_text, Style::default().fg(Color::DarkGray));

    let line = Line::from(vec![mode_span, wf_name, stats, help]);
    Paragraph::new(line).style(Style::default().bg(Color::Black)).render(area, buf);
}

fn unsaved_indicator(has_unsaved_changes: bool) -> &'static str {
    if has_unsaved_changes { "*" } else { "" }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FooterInteractionMode {
    Default,
    Wiring,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FooterHelpContext {
    focus: FocusPanel,
    palette_open: bool,
    workflow_browser_open: bool,
    add_node_open: bool,
    run_history_open: bool,
    diagnostics_open: bool,
    interaction_mode: FooterInteractionMode,
    width: u16,
}

fn footer_interaction_mode(mode: &InteractionMode) -> FooterInteractionMode {
    if matches!(mode, InteractionMode::Wiring { .. }) {
        FooterInteractionMode::Wiring
    } else {
        FooterInteractionMode::Default
    }
}

// r[impl n8nclone.tui_ux.help_footer.contextual]
fn contextual_help_text(context: FooterHelpContext) -> &'static str {
    if context.width < COMPACT_STATUS_HELP_WIDTH {
        return compact_contextual_help_text(context);
    }

    if context.palette_open {
        return " Type: filter │ ↑/↓: select │ Enter: run │ Esc: close ";
    }

    if context.workflow_browser_open {
        return " ↑/↓: choose workflow │ Enter: open │ Esc: close ";
    }

    if context.add_node_open {
        return " Type: edit field │ Enter: next/create │ Esc: cancel ";
    }

    if context.run_history_open {
        return " ↑/↓: choose │ Enter: inspect │ r: rerun │ Esc: close ";
    }

    if context.diagnostics_open {
        return " Diagnostics │ Enter/Esc: close ";
    }

    if context.interaction_mode == FooterInteractionMode::Wiring {
        return " Click target port: connect │ Esc: cancel wiring ";
    }

    match context.focus {
        FocusPanel::Canvas => " e: cmd │ l: label │ t: timeout │ C-D: diagnostics │ C-R: run │ C-P: palette ",
        FocusPanel::Properties => " e: cmd │ l: label │ i: image │ p/r/o: agent │ k: creds ",
        FocusPanel::Output => " ↑/↓: scroll │ C-P: palette │ A-Tab: focus ",
    }
}

fn compact_contextual_help_text(context: FooterHelpContext) -> &'static str {
    if context.palette_open {
        return " Enter: run │ Esc ";
    }

    if context.workflow_browser_open {
        return " Enter open │ Esc ";
    }

    if context.add_node_open {
        return " Enter next │ Esc ";
    }

    if context.run_history_open {
        return " Enter inspect │ Esc ";
    }

    if context.diagnostics_open {
        return " Enter/Esc close ";
    }

    if context.interaction_mode == FooterInteractionMode::Wiring {
        return " Click target │ Esc ";
    }

    match context.focus {
        FocusPanel::Canvas => " e cmd │ l label ",
        FocusPanel::Properties => " e cmd │ p agent ",
        FocusPanel::Output => " ↑/↓ scroll │ A-Tab ",
    }
}

fn render_diagnostics(app: &App, area: Rect, buf: &mut Buffer) {
    let width = area.width.min(DIAGNOSTICS_WIDTH);
    let height = area.height.min(DIAGNOSTICS_HEIGHT);
    let x = (area.width.saturating_sub(width)) / 2;
    let y = DIAGNOSTICS_TOP_OFFSET.min(area.height.saturating_sub(height));
    let diagnostics_area = Rect::new(x, y, width, height);

    Clear.render(diagnostics_area, buf);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Workflow Diagnostics ")
        .border_style(Style::default().fg(Color::Red));
    let inner = block.inner(diagnostics_area);
    block.render(diagnostics_area, buf);

    if app.diagnostics.diagnostics.is_empty() {
        Paragraph::new("Workflow ready to run")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Green))
            .render(inner, buf);
        return;
    }

    let visible_rows = inner.height as usize;
    let lines = app
        .diagnostics
        .diagnostics
        .iter()
        .take(visible_rows)
        .map(|diagnostic| {
            let style = match diagnostic.severity {
                DiagnosticSeverity::Fatal => Style::default().fg(Color::Red),
                DiagnosticSeverity::Warning => Style::default().fg(Color::Yellow),
            };
            Line::styled(workflow_validation::format_diagnostic(diagnostic), style)
        })
        .collect::<Vec<_>>();

    Paragraph::new(lines).render(inner, buf);
}

fn render_run_history(app: &App, area: Rect, buf: &mut Buffer) {
    let width = area.width.min(RUN_HISTORY_WIDTH);
    let height = area.height.min(RUN_HISTORY_HEIGHT);
    let x = (area.width.saturating_sub(width)) / 2;
    let y = RUN_HISTORY_TOP_OFFSET.min(area.height.saturating_sub(height));
    let history_area = Rect::new(x, y, width, height);

    Clear.render(history_area, buf);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Run History ")
        .border_style(Style::default().fg(Color::Magenta));
    let inner = block.inner(history_area);
    block.render(history_area, buf);

    if let Some(detail_record) = &app.run_history.detail_record {
        render_run_detail_record(
            detail_record,
            app.run_history.selected_detail_node,
            app.run_history.replay_timeline.as_ref(),
            app.run_history.replay_cursor,
            inner,
            buf,
        );
        return;
    }

    if let Some(detail) = &app.run_history.detail {
        Paragraph::new(detail.as_str())
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::White))
            .render(inner, buf);
        return;
    }

    if app.run_history.records.is_empty() {
        let message = app.run_history.message.as_deref().unwrap_or("No run history available");
        Paragraph::new(message)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::DarkGray))
            .render(inner, buf);
        return;
    }

    let visible_rows = inner.height as usize;
    let lines = app
        .run_history
        .records
        .iter()
        .take(visible_rows)
        .enumerate()
        .map(|(index, record)| {
            let selected = index == app.run_history.selected;
            let style = if selected {
                Style::default().fg(Color::Black).bg(Color::Magenta)
            } else {
                Style::default().fg(Color::White)
            };
            let prefix = if selected { "› " } else { "  " };
            Line::styled(format!("{prefix}{}", crate::tui::app::format_run_record(record)), style)
        })
        .collect::<Vec<_>>();

    Paragraph::new(lines).render(inner, buf);
}

// r[impl n8nclone.tui_ux.run_history.timeline]
// r[impl n8nclone.tui_ux.run_history.node_outputs]
fn render_run_detail_record(
    record: &RunDetailRecord,
    selected_node: usize,
    replay_timeline: Option<&ReplayTimeline>,
    replay_cursor: ReplayCursor,
    area: Rect,
    buf: &mut Buffer,
) {
    if record.nodes.is_empty() {
        Paragraph::new("No node details available")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::DarkGray))
            .render(area, buf);
        return;
    }

    let selected_node = selected_node.min(record.nodes.len().saturating_sub(1));
    let selected_detail = &record.nodes[selected_node];
    let mut lines = record
        .nodes
        .iter()
        .enumerate()
        .map(|(index, detail)| run_detail_timeline_line(index == selected_node, detail))
        .collect::<Vec<_>>();

    lines.push(Line::from(""));
    lines.push(Line::styled(
        format!("Outputs for {}", selected_detail.node_label),
        Style::default().fg(Color::DarkGray),
    ));
    append_selected_run_detail_output_lines(&mut lines, selected_detail);
    append_replay_timeline_lines(&mut lines, replay_timeline, replay_cursor);

    Paragraph::new(lines).render(area, buf);
}

// r[impl n8nclone.tui_ux.replay_debugger.timeline]
// r[impl n8nclone.tui_ux.replay_debugger.redaction]
fn append_replay_timeline_lines(
    lines: &mut Vec<Line<'static>>,
    replay_timeline: Option<&ReplayTimeline>,
    replay_cursor: ReplayCursor,
) {
    let Some(timeline) = replay_timeline else {
        return;
    };
    lines.push(Line::from(""));
    lines.push(Line::styled("Replay Timeline", Style::default().fg(Color::DarkGray)));
    if timeline.frames.is_empty() {
        lines.push(Line::styled("No replay events available", Style::default().fg(Color::DarkGray)));
        return;
    }
    for frame in &timeline.frames {
        lines.push(replay_frame_line(frame.cursor == replay_cursor.position, frame));
    }
    if let Some(frame) = timeline.frames.get(replay_cursor.position) {
        lines.push(Line::styled(
            format!("selected: #{} {}", frame.event.sequence, frame.event.summary),
            Style::default().fg(Color::Gray),
        ));
    }
}

fn replay_frame_line(selected: bool, frame: &ReplayFrame) -> Line<'static> {
    let node = frame.event.node_id.map(|node_id| format!(" node {node_id}")).unwrap_or_default();
    let redaction = if frame.event.redacted { " redacted" } else { "" };
    let prefix = if selected { "› " } else { "  " };
    let style = if selected {
        Style::default().fg(Color::Black).bg(Color::Blue)
    } else {
        Style::default().fg(Color::Gray)
    };
    Line::styled(format!("{prefix}#{} {:?}{node}{redaction}", frame.event.sequence, frame.event.kind), style)
}

fn run_detail_timeline_line(selected: bool, detail: &RunNodeDetail) -> Line<'static> {
    let status = run_detail_status_label(detail.status);
    let cache = if detail.cached { " cached" } else { "" };
    let prefix = if selected { "› " } else { "  " };
    let style = if selected {
        Style::default().fg(Color::Black).bg(Color::Magenta)
    } else {
        run_detail_status_style(detail.status)
    };
    Line::styled(format!("{prefix}{} — {status}{cache}", detail.node_label), style)
}

fn append_selected_run_detail_output_lines(lines: &mut Vec<Line<'static>>, detail: &RunNodeDetail) {
    if let Some(error) = &detail.error {
        lines.push(Line::styled(format!("error: {error}"), Style::default().fg(Color::Red)));
    }
    if detail.outputs.is_empty() && detail.error.is_none() {
        lines.push(Line::styled("No output details", Style::default().fg(Color::DarkGray)));
        return;
    }
    for output in &detail.outputs {
        lines.push(Line::styled(format!("{}: {}", output.port_label, output.text), Style::default().fg(Color::Gray)));
    }
}

fn run_detail_status_label(status: RunNodeDetailStatus) -> &'static str {
    match status {
        RunNodeDetailStatus::Pending => "pending",
        RunNodeDetailStatus::Running => "running",
        RunNodeDetailStatus::Completed => "completed",
        RunNodeDetailStatus::Failed => "failed",
        RunNodeDetailStatus::Skipped => "skipped",
    }
}

fn run_detail_status_style(status: RunNodeDetailStatus) -> Style {
    match status {
        RunNodeDetailStatus::Pending => Style::default().fg(Color::DarkGray),
        RunNodeDetailStatus::Running => Style::default().fg(Color::Yellow),
        RunNodeDetailStatus::Completed => Style::default().fg(Color::Green),
        RunNodeDetailStatus::Failed => Style::default().fg(Color::Red),
        RunNodeDetailStatus::Skipped => Style::default().fg(Color::Gray),
    }
}

fn render_add_node_flow(app: &App, area: Rect, buf: &mut Buffer) {
    let width = area.width.min(ADD_NODE_WIDTH);
    let height = area.height.min(ADD_NODE_HEIGHT);
    let x = (area.width.saturating_sub(width)) / 2;
    let y = ADD_NODE_TOP_OFFSET.min(area.height.saturating_sub(height));
    let flow_area = Rect::new(x, y, width, height);

    Clear.render(flow_area, buf);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Add Node ")
        .border_style(Style::default().fg(Color::Green));
    let inner = block.inner(flow_area);
    block.render(flow_area, buf);

    let lines = match app.add_node_flow.step {
        AddNodeStep::Kind => render_add_node_kind_lines(),
        AddNodeStep::TemplateSelect => render_add_node_template_lines(app),
        AddNodeStep::Label => vec![
            Line::styled(add_node_label_title(app.add_node_flow.kind), Style::default().fg(Color::Yellow)),
            Line::from(app.add_node_flow.label.clone()),
            Line::styled("Enter: next │ Esc: cancel", Style::default().fg(Color::DarkGray)),
        ],
        AddNodeStep::TemplateParameter => render_add_node_parameter_lines(app),
        AddNodeStep::Command => vec![
            Line::styled(add_node_config_title(app.add_node_flow.kind), Style::default().fg(Color::Yellow)),
            Line::from(app.add_node_flow.command.clone()),
            Line::styled("Enter: create │ Esc: cancel", Style::default().fg(Color::DarkGray)),
        ],
    };

    Paragraph::new(lines).render(inner, buf);
}

fn render_add_node_kind_lines() -> Vec<Line<'static>> {
    vec![
        Line::styled("Choose node type", Style::default().fg(Color::Yellow)),
        Line::from("s: shell"),
        Line::from("i: image input"),
        Line::from("p: Pi agent"),
        Line::from("c: Clankers agent"),
        Line::from("t: templates"),
        Line::styled("Enter: shell │ Esc: cancel", Style::default().fg(Color::DarkGray)),
    ]
}

fn add_node_label_title(kind: AddNodeKind) -> &'static str {
    match kind {
        AddNodeKind::Shell => "Shell node label",
        AddNodeKind::ImageInput => "Image input label",
        AddNodeKind::PiAgent => "Pi agent label",
        AddNodeKind::ClankersAgent => "Clankers agent label",
    }
}

fn add_node_config_title(kind: AddNodeKind) -> &'static str {
    match kind {
        AddNodeKind::Shell => "Shell command",
        AddNodeKind::ImageInput => "Image path",
        AddNodeKind::PiAgent => "Pi agent prompt",
        AddNodeKind::ClankersAgent => "Clankers agent prompt",
    }
}

fn render_add_node_template_lines(app: &App) -> Vec<Line<'static>> {
    let mut lines = vec![
        Line::styled("Choose a template", Style::default().fg(Color::Yellow)),
        Line::styled(
            "↑/↓: select │ Enter: create │ s: manual shell │ Esc: cancel",
            Style::default().fg(Color::DarkGray),
        ),
    ];

    for (index, template) in app.node_templates.iter().enumerate() {
        let selected = index == app.add_node_flow.selected_template;
        let style = if selected {
            Style::default().fg(Color::Black).bg(Color::Green)
        } else {
            Style::default().fg(Color::White)
        };
        let prefix = if selected { "› " } else { "  " };
        lines.push(Line::styled(format!("{prefix}{}", template.name), style));
    }

    lines
}

fn render_add_node_parameter_lines(app: &App) -> Vec<Line<'static>> {
    let Some(template) = app.node_templates.get(app.add_node_flow.selected_template) else {
        return vec![Line::styled("Template missing", Style::default().fg(Color::Red))];
    };
    let Some(parameter) = template.parameters.get(app.add_node_flow.parameter_index) else {
        return vec![Line::styled("Parameter missing", Style::default().fg(Color::Red))];
    };
    let value = app
        .add_node_flow
        .parameter_values
        .get(app.add_node_flow.parameter_index)
        .cloned()
        .unwrap_or_default();
    let required = if parameter.required { "required" } else { "optional" };
    vec![
        Line::styled(
            format!("Template parameter {} / {}", app.add_node_flow.parameter_index + 1, template.parameters.len()),
            Style::default().fg(Color::Yellow),
        ),
        Line::from(format!("{} ({:?}, {required})", parameter.name, parameter.type_tag)),
        Line::from(parameter.description.clone()),
        Line::from(value),
        Line::styled("Enter: next/create │ Esc: cancel", Style::default().fg(Color::DarkGray)),
    ]
}

fn render_workflow_browser(app: &App, area: Rect, buf: &mut Buffer) {
    let width = area.width.min(WORKFLOW_BROWSER_WIDTH);
    let height = area.height.min(WORKFLOW_BROWSER_HEIGHT);
    let x = (area.width.saturating_sub(width)) / 2;
    let y = WORKFLOW_BROWSER_TOP_OFFSET.min(area.height.saturating_sub(height));
    let browser_area = Rect::new(x, y, width, height);

    Clear.render(browser_area, buf);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Open Workflow ")
        .border_style(Style::default().fg(Color::Blue));
    let inner = block.inner(browser_area);
    block.render(browser_area, buf);

    if app.workflow_browser.names.is_empty() {
        let message = app.workflow_browser.message.as_deref().unwrap_or("No saved workflows available");
        Paragraph::new(message)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::DarkGray))
            .render(inner, buf);
        return;
    }

    let visible_rows = inner.height as usize;
    let lines = app
        .workflow_browser
        .names
        .iter()
        .take(visible_rows)
        .enumerate()
        .map(|(index, name)| {
            let selected = index == app.workflow_browser.selected;
            let style = if selected {
                Style::default().fg(Color::Black).bg(Color::Cyan)
            } else {
                Style::default().fg(Color::White)
            };
            let prefix = if selected { "› " } else { "  " };
            Line::styled(format!("{prefix}{name}"), style)
        })
        .collect::<Vec<_>>();

    Paragraph::new(lines).render(inner, buf);
}

fn render_quit_confirmation(area: Rect, buf: &mut Buffer) {
    let width = area.width.min(QUIT_CONFIRM_WIDTH);
    let height = area.height.min(QUIT_CONFIRM_HEIGHT);
    let x = (area.width.saturating_sub(width)) / 2;
    let y = area.height.saturating_sub(height).min(QUIT_CONFIRM_TOP_OFFSET);
    let confirm_area = Rect::new(x, y, width, height);

    Clear.render(confirm_area, buf);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Unsaved changes ")
        .border_style(Style::default().fg(Color::Red));
    let inner = block.inner(confirm_area);
    block.render(confirm_area, buf);

    Paragraph::new(vec![
        Line::styled("Quit without saving?", Style::default().fg(Color::White)),
        Line::styled("y/Enter: quit │ n/Esc: stay", Style::default().fg(Color::DarkGray)),
    ])
    .alignment(Alignment::Center)
    .render(inner, buf);
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

#[cfg(test)]
mod tests {
    use super::*;

    const WIDE_STATUS_WIDTH: u16 = COMPACT_STATUS_HELP_WIDTH;
    const NARROW_STATUS_WIDTH: u16 = COMPACT_STATUS_HELP_WIDTH - 1;

    fn context(focus: FocusPanel) -> FooterHelpContext {
        FooterHelpContext {
            focus,
            palette_open: false,
            workflow_browser_open: false,
            add_node_open: false,
            run_history_open: false,
            diagnostics_open: false,
            interaction_mode: FooterInteractionMode::Default,
            width: WIDE_STATUS_WIDTH,
        }
    }

    #[test]
    fn unsaved_indicator_marks_dirty_workflows() {
        assert_eq!(unsaved_indicator(false), "");
        assert_eq!(unsaved_indicator(true), "*");
    }

    #[test]
    fn contextual_help_matches_canvas_and_wiring_modes() {
        assert_eq!(
            contextual_help_text(context(FocusPanel::Canvas)),
            " e: cmd │ l: label │ t: timeout │ C-D: diagnostics │ C-R: run │ C-P: palette ",
        );

        let wiring = FooterHelpContext {
            interaction_mode: FooterInteractionMode::Wiring,
            ..context(FocusPanel::Canvas)
        };
        assert_eq!(contextual_help_text(wiring), " Click target port: connect │ Esc: cancel wiring ",);
    }

    #[test]
    fn contextual_help_matches_properties_and_output_focus() {
        assert_eq!(
            contextual_help_text(context(FocusPanel::Properties)),
            " e: cmd │ l: label │ i: image │ p/r/o: agent │ k: creds ",
        );
        assert_eq!(contextual_help_text(context(FocusPanel::Output)), " ↑/↓: scroll │ C-P: palette │ A-Tab: focus ",);
    }

    #[test]
    fn contextual_help_reports_palette_modal() {
        let palette = FooterHelpContext {
            palette_open: true,
            ..context(FocusPanel::Canvas)
        };

        assert_eq!(contextual_help_text(palette), " Type: filter │ ↑/↓: select │ Enter: run │ Esc: close ",);
    }

    #[test]
    fn contextual_help_reports_workflow_browser_modal() {
        let browser = FooterHelpContext {
            workflow_browser_open: true,
            ..context(FocusPanel::Canvas)
        };

        assert_eq!(contextual_help_text(browser), " ↑/↓: choose workflow │ Enter: open │ Esc: close ",);
    }

    #[test]
    fn contextual_help_uses_compact_copy_for_narrow_width() {
        let narrow = FooterHelpContext {
            width: NARROW_STATUS_WIDTH,
            ..context(FocusPanel::Canvas)
        };
        assert_eq!(contextual_help_text(narrow), " e cmd │ l label ");

        let narrow_wiring = FooterHelpContext {
            interaction_mode: FooterInteractionMode::Wiring,
            ..narrow
        };
        assert_eq!(contextual_help_text(narrow_wiring), " Click target │ Esc ");
    }

    #[test]
    fn empty_state_visibility_requires_empty_workflow_without_modal() {
        assert!(should_show_empty_state(0, false));
        assert!(!should_show_empty_state(1, false));
        assert!(!should_show_empty_state(0, true));
    }

    #[test]
    fn empty_state_copy_lists_next_actions_for_full_canvas() {
        let copy = empty_state_copy(FULL_EMPTY_STATE_MIN_WIDTH, FULL_EMPTY_STATE_MIN_HEIGHT);

        assert_eq!(copy.title, "Start your workflow");
        assert!(copy.hints.contains(&"Ctrl-N: add your first node"));
        assert!(copy.hints.contains(&"Ctrl-P: open the command palette"));
    }

    #[test]
    fn empty_state_copy_uses_compact_hint_for_small_canvas() {
        let copy = empty_state_copy(FULL_EMPTY_STATE_MIN_WIDTH - 1, FULL_EMPTY_STATE_MIN_HEIGHT);

        assert_eq!(copy.title, "Empty workflow");
        assert_eq!(copy.hints, &["Ctrl-N add node │ Ctrl-P palette"]);
    }
}

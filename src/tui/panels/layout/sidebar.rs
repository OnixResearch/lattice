//! Selected-node properties and observed execution state.

use super::*;

/// Render the properties panel for the selected node.
pub(in crate::tui::panels) fn render_properties(app: &crate::tui::app::State, area: Rect, buf: &mut Buffer) {
    let is_focused = matches!(app.focus, FocusPanel::Properties);
    let border_style = Style::new().fg(if is_focused { Color::Yellow } else { Color::DarkGray });

    let block = Block::new().borders(Borders::ALL).title(" Properties ").border_style(border_style);

    let inner = block.inner(area);
    debug_assert!(inner.width <= area.width);
    debug_assert!(inner.height <= area.height);
    block.render(area, buf);

    let selected = &app.graph_state.selected;

    if selected.len() != 1 {
        let hint = if selected.is_empty() {
            "Select a node to edit"
        } else {
            "Multiple nodes selected"
        };
        Paragraph::new(hint).style(Style::new().fg(Color::DarkGray)).render(inner, buf);
        return;
    }

    let Some(node_id) = selected.iter().next().copied() else {
        return;
    };
    let node = match app.graph_state.graph.node(node_id) {
        Some(n) => n,
        None => return,
    };

    let mut lines = property_lines(app, node_id, &node.label);

    // Show ports.
    lines.push(Line::from(""));
    lines.push(Line::styled("─ Inputs ─", Style::new().fg(Color::DarkGray)));
    for port in &node.input_ports {
        lines.push(Line::from(vec![
            Span::styled("● ", Style::new().fg(Color::Cyan)),
            Span::raw(port.label.clone()),
            Span::styled(format!(" ({})", port.type_tag), Style::new().fg(Color::DarkGray)),
        ]));
    }
    lines.push(Line::styled("─ Outputs ─", Style::new().fg(Color::DarkGray)));
    for port in &node.output_ports {
        lines.push(Line::from(vec![
            Span::styled("● ", Style::new().fg(Color::Green)),
            Span::raw(port.label.clone()),
            Span::styled(format!(" ({})", port.type_tag), Style::new().fg(Color::DarkGray)),
        ]));
    }

    let latest_output_lines = rendered_latest_node_output_lines(&app.run_output_lines, &app.workflow, node_id);
    if !latest_output_lines.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::styled("─ Latest output ─", Style::new().fg(Color::DarkGray)));
        for line in latest_output_lines {
            lines.push(Line::styled(line, Style::new().fg(Color::Gray)));
        }
    }

    Paragraph::new(lines).render(inner, buf);
}

pub(in crate::tui::panels) fn run_status_label(status: NodeRunStatus) -> &'static str {
    match status {
        NodeRunStatus::Pending => "pending",
        NodeRunStatus::Running => "running",
        NodeRunStatus::Completed => "completed",
        NodeRunStatus::Failed => "failed",
        NodeRunStatus::Skipped => "skipped",
    }
}

pub(in crate::tui::panels) fn run_status_style(status: NodeRunStatus) -> Style {
    let color = match status {
        NodeRunStatus::Pending => Color::DarkGray,
        NodeRunStatus::Running => Color::Yellow,
        NodeRunStatus::Completed => Color::Green,
        NodeRunStatus::Failed => Color::Red,
        NodeRunStatus::Skipped => Color::Gray,
    };
    Style::new().fg(color)
}

pub(in crate::tui::panels) fn property_field_title(field: PropertyEditField) -> &'static str {
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
        PropertyEditField::Credentials => "Credentials",
        PropertyEditField::ConditionBranches => "Condition Branches",
    }
}

fn property_lines(app: &crate::tui::app::State, node_id: NodeId, label: &str) -> Vec<Line<'static>> {
    let mut lines = node_identity_lines(node_id, label);

    if let Some(status) = app.run_state.node_statuses.get(&node_id.0).copied() {
        lines.push(Line::from(vec![
            Span::styled("Run: ", Style::new().fg(Color::DarkGray)),
            Span::styled(run_status_label(status), run_status_style(status)),
        ]));
        lines.push(Line::from(""));
    }

    lines = append_edit_lines(lines, app.property_edit.as_ref().filter(|edit| edit.node_id == node_id));

    append_config_lines(lines, app, node_id)
}

fn node_identity_lines(node_id: NodeId, label: &str) -> Vec<Line<'static>> {
    vec![
        Line::from(vec![
            Span::styled("Node: ", Style::new().fg(Color::DarkGray)),
            Span::styled(label.to_string(), Style::new().fg(Color::White).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("ID: ", Style::new().fg(Color::DarkGray)),
            Span::styled(format!("{node_id}"), Style::new().fg(Color::Gray)),
        ]),
        Line::from(""),
        Line::styled("Ctrl-W: all typed fields / rows", Style::new().fg(Color::Yellow)),
    ]
}

fn append_edit_lines(
    mut lines: Vec<Line<'static>>,
    edit: Option<&crate::tui::app::PropertyEditState>,
) -> Vec<Line<'static>> {
    if let Some(edit) = edit {
        lines.push(edit_heading(edit.field));
        lines.push(Line::from(vec![
            Span::styled("Draft: ", Style::new().fg(Color::DarkGray)),
            Span::styled(edit.draft.clone(), Style::new().fg(Color::White)),
        ]));
        lines.push(Line::styled("Enter: save │ Esc: cancel", Style::new().fg(Color::DarkGray)));
        lines.push(Line::from(""));
    }
    lines
}

fn edit_heading(field: PropertyEditField) -> Line<'static> {
    Line::from(vec![
        Span::styled("Editing: ", Style::new().fg(Color::DarkGray)),
        Span::styled(property_field_title(field), Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
    ])
}

fn append_config_lines(
    mut lines: Vec<Line<'static>>,
    app: &crate::tui::app::State,
    node_id: NodeId,
) -> Vec<Line<'static>> {
    // r[impl n8nclone.tui_ux.node_kind_provider.fallback]
    if let Some(config) = app.workflow.node_config(node_id) {
        for property in node_kind_provider::property_lines_for_config(config) {
            lines.push(Line::from(vec![
                Span::styled(format!("{}: ", property.label), Style::new().fg(Color::DarkGray)),
                Span::styled(property.value, Style::new().fg(Color::Cyan)),
            ]));
        }
        lines = append_credentials(lines, app.workflow.node_credentials(node_id));
    }
    lines
}

fn append_credentials(
    mut lines: Vec<Line<'static>>,
    refs: Option<&crate::credentials::NodeCredentialRefs>,
) -> Vec<Line<'static>> {
    let credentials = crate::tui::app::editing::parsing::credentials_to_draft(refs);
    if !credentials.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("Credentials: ", Style::new().fg(Color::DarkGray)),
            Span::styled(credentials, Style::new().fg(Color::Gray)),
        ]));
    }
    lines
}

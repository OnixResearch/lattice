//! Guided node creation overlays.

use super::*;

pub(in crate::tui::panels) fn render_add_node_flow(app: &crate::tui::app::State, area: Rect, buf: &mut Buffer) {
    let width = area.width.min(ADD_NODE_WIDTH);
    let height = area.height.min(ADD_NODE_HEIGHT);
    let x = (area.width.saturating_sub(width)) / 2;
    let y = ADD_NODE_TOP_OFFSET.min(area.height.saturating_sub(height));
    let flow_area = Rect::new(x, y, width, height);

    Clear.render(flow_area, buf);
    let block = Block::new().borders(Borders::ALL).title(" Add Node ").border_style(
        Style {
            fg: None,
            bg: None,
            underline_color: None,
            add_modifier: Modifier::empty(),
            sub_modifier: Modifier::empty(),
        }
        .fg(Color::Green),
    );
    let inner = block.inner(flow_area);
    debug_assert!(inner.width <= flow_area.width);
    debug_assert!(inner.height <= flow_area.height);
    block.render(flow_area, buf);

    let lines = match app.add_node_flow.step {
        AddNodeStep::Kind => render_add_node_kind_lines(),
        AddNodeStep::TemplateSelect => render_add_node_template_lines(app),
        AddNodeStep::Label => vec![
            Line::styled(add_node_label_title(app.add_node_flow.kind), Style {
                fg: Some(Color::Yellow),
                bg: None,
                underline_color: None,
                add_modifier: Modifier::empty(),
                sub_modifier: Modifier::empty(),
            }),
            Line::from(app.add_node_flow.label.clone()),
            Line::styled("Enter: next │ Esc: cancel", Style {
                fg: Some(Color::DarkGray),
                bg: None,
                underline_color: None,
                add_modifier: Modifier::empty(),
                sub_modifier: Modifier::empty(),
            }),
        ],
        AddNodeStep::TemplateParameter => render_add_node_parameter_lines(app),
        AddNodeStep::Command => vec![
            Line::styled(add_node_config_title(app.add_node_flow.kind), Style {
                fg: Some(Color::Yellow),
                bg: None,
                underline_color: None,
                add_modifier: Modifier::empty(),
                sub_modifier: Modifier::empty(),
            }),
            Line::from(app.add_node_flow.command.clone()),
            Line::styled("Enter: create │ Esc: cancel", Style {
                fg: Some(Color::DarkGray),
                bg: None,
                underline_color: None,
                add_modifier: Modifier::empty(),
                sub_modifier: Modifier::empty(),
            }),
        ],
    };

    Paragraph::new(lines).render(inner, buf);
}

pub(in crate::tui::panels) fn render_add_node_kind_lines() -> Vec<Line<'static>> {
    vec![
        Line::styled("Choose node type", Style {
            fg: Some(Color::Yellow),
            bg: None,
            underline_color: None,
            add_modifier: Modifier::empty(),
            sub_modifier: Modifier::empty(),
        }),
        Line::from("s: shell"),
        Line::from("h: HTTP request"),
        Line::from("f: file operation"),
        Line::from("i: image input"),
        Line::from("p: Pi agent"),
        Line::from("c: Clankers agent"),
        Line::from("d: condition"),
        Line::from("m: merge"),
        Line::from("w: subworkflow"),
        Line::from("t: templates"),
        Line::styled("Enter: shell │ Esc: cancel", Style {
            fg: Some(Color::DarkGray),
            bg: None,
            underline_color: None,
            add_modifier: Modifier::empty(),
            sub_modifier: Modifier::empty(),
        }),
    ]
}

pub(in crate::tui::panels) fn add_node_label_title(kind: AddNodeKind) -> &'static str {
    match kind {
        AddNodeKind::Shell => "Shell node label",
        AddNodeKind::Http => "HTTP node label",
        AddNodeKind::File => "File node label",
        AddNodeKind::ImageInput => "Image input label",
        AddNodeKind::PiAgent => "Pi agent label",
        AddNodeKind::ClankersAgent => "Clankers agent label",
        AddNodeKind::Condition => "Condition node label",
        AddNodeKind::Merge => "Merge node label",
        AddNodeKind::Subworkflow => "Subworkflow node label",
    }
}

pub(in crate::tui::panels) fn add_node_config_title(kind: AddNodeKind) -> &'static str {
    match kind {
        AddNodeKind::Shell => "Shell command",
        AddNodeKind::Http => "HTTP request (METHOD URL)",
        AddNodeKind::File => "File operation (read|write|append PATH)",
        AddNodeKind::ImageInput => "Image path",
        AddNodeKind::PiAgent => "Pi agent prompt",
        AddNodeKind::ClankersAgent => "Clankers agent prompt",
        AddNodeKind::Condition => "Branches (label=predicate, ...)",
        AddNodeKind::Merge => "Merge policy (all|first|last|collect:inputs)",
        AddNodeKind::Subworkflow => "Saved workflow name",
    }
}

pub(in crate::tui::panels) fn render_add_node_template_lines(app: &crate::tui::app::State) -> Vec<Line<'static>> {
    let mut lines = vec![
        Line::styled("Choose a template", Style::new().fg(Color::Yellow)),
        Line::styled("↑/↓: select │ Enter: create │ s: manual shell │ Esc: cancel", Style::new().fg(Color::DarkGray)),
    ];

    for (index, template) in app.node_templates.iter().enumerate() {
        lines.push(template_choice_line(index == app.add_node_flow.selected_template, &template.name));
    }

    lines
}

fn template_choice_line(selected: bool, name: &str) -> Line<'static> {
    let style = if selected {
        Style::new().fg(Color::Black).bg(Color::Green)
    } else {
        Style::new().fg(Color::White)
    };
    let prefix = if selected { "› " } else { "  " };
    Line::styled(format!("{prefix}{name}"), style)
}

pub(in crate::tui::panels) fn render_add_node_parameter_lines(app: &crate::tui::app::State) -> Vec<Line<'static>> {
    let Some(template) = app.node_templates.get(app.add_node_flow.selected_template) else {
        return vec![Line::styled("Template missing", Style::new().fg(Color::Red))];
    };
    let Some(parameter) = template.parameters.get(app.add_node_flow.parameter_index) else {
        return vec![Line::styled("Parameter missing", Style::new().fg(Color::Red))];
    };
    parameter_lines(app, template, parameter)
}

fn parameter_lines(
    app: &crate::tui::app::State,
    template: &crate::config::NodeTemplate,
    parameter: &crate::config::TemplateParameter,
) -> Vec<Line<'static>> {
    let value = parameter_draft_value(app);
    let required = if parameter.required { "required" } else { "optional" };
    vec![
        template_parameter_heading_line(app, template),
        Line::from(format!("{} ({:?}, {required})", parameter.name, parameter.type_tag)),
        Line::from(parameter.description.clone()),
        Line::from(value),
        Line::styled("Enter: next/create │ Esc: cancel", Style::new().fg(Color::DarkGray)),
    ]
}
fn template_parameter_heading_line(
    app: &crate::tui::app::State,
    template: &crate::config::NodeTemplate,
) -> Line<'static> {
    let index = app.add_node_flow.parameter_index.saturating_add(1);
    let count = template.parameters.len();
    Line::styled(format!("Template parameter {index} / {count}"), Style::new().fg(Color::Yellow))
}

fn parameter_draft_value(app: &crate::tui::app::State) -> String {
    app.add_node_flow
        .parameter_values
        .get(app.add_node_flow.parameter_index)
        .cloned()
        .unwrap_or_default()
}

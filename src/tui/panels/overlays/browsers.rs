//! Workflow browsing and quit confirmation overlays.

use super::*;

pub(in crate::tui::panels) fn render_workflow_template_browser(
    app: &crate::tui::app::State,
    area: Rect,
    buf: &mut Buffer,
) {
    debug_assert!(app.workflow_template_browser.selected <= app.workflow_templates.len());
    let width = area.width.min(WORKFLOW_TEMPLATE_BROWSER_WIDTH);
    let height = area.height.min(WORKFLOW_TEMPLATE_BROWSER_HEIGHT);
    debug_assert!(width <= area.width);
    debug_assert!(height <= area.height);
    let x = (area.width.saturating_sub(width)) / 2;
    let y = WORKFLOW_TEMPLATE_BROWSER_TOP_OFFSET.min(area.height.saturating_sub(height));
    let browser_area = Rect::new(x, y, width, height);

    Clear.render(browser_area, buf);
    let block = Block::new()
        .borders(Borders::ALL)
        .title(" Workflow Templates ")
        .border_style(Style::new().fg(Color::Green));
    let inner = block.inner(browser_area);
    block.render(browser_area, buf);

    let mut lines = Vec::with_capacity(app.workflow_templates.len().saturating_add(WORKFLOW_TEMPLATE_METADATA_ROWS));
    if let Some(message) = &app.workflow_template_browser.message {
        lines.push(Line::styled(message.clone(), Style::new().fg(Color::Yellow)));
    }
    lines.push(Line::styled(
        match app.workflow_template_browser.step {
            crate::tui::app::WorkflowTemplateBrowseStep::Browse => "↑/↓: select │ Enter: preview │ Esc: cancel",
            crate::tui::app::WorkflowTemplateBrowseStep::Preview => {
                "Enter: keep unsaved draft │ Esc: restore prior workflow"
            }
        },
        Style::new().fg(Color::DarkGray),
    ));
    for (index, template) in app.workflow_templates.iter().enumerate() {
        let is_selected = index == app.workflow_template_browser.selected;
        let style = if is_selected {
            Style::new().fg(Color::Black).bg(Color::Green)
        } else {
            Style::new().fg(Color::White)
        };
        let prefix = if is_selected { "› " } else { "  " };
        let orchestration = if template.orchestration.is_some() {
            "orchestration draft"
        } else {
            "workflow only"
        };
        lines.push(Line::styled(
            format!(
                "{prefix}{} [{}] — {orchestration} — {}",
                template.manifest.display_name,
                template.manifest.category,
                template.manifest.mode().as_str()
            ),
            style,
        ));
        if is_selected {
            lines.extend(selected_template_details(template));
        }
    }
    if app.workflow_templates.is_empty() && app.workflow_template_browser.message.is_none() {
        lines.push(Line::styled("No valid workflow templates are available", Style::new().fg(Color::DarkGray)));
    }
    lines.truncate(usize::from(inner.height));
    Paragraph::new(lines).render(inner, buf);
}

/// Description, a mutating warning, declared parameters, identity, and non-claims; never commands.
fn selected_template_details(template: &crate::workflow_template::ValidatedBundle) -> Vec<Line<'static>> {
    let mut lines = vec![Line::styled(
        template.manifest.description.clone(),
        Style::new().fg(Color::Gray),
    )];
    if template.manifest.mode() == crate::workflow_template::BundleMode::Mutating {
        lines.push(Line::styled("mode: mutating — may change repositories or hosts", Style::new().fg(Color::Red)));
    }
    for param in template.manifest.params() {
        let requirement = match &param.default {
            Some(default) => format!("default {}", crate::workflow_template::params::render(default)),
            None => "required".to_string(),
        };
        lines.push(Line::styled(format!("param {}: {requirement}", param.name), Style::new().fg(Color::Gray)));
    }
    lines.push(Line::styled(format!("bundle {}", template.identity.as_str()), Style::new().fg(Color::DarkGray)));
    for claim in &template.manifest.non_claims {
        lines.push(Line::styled(format!("non-claim: {claim}"), Style::new().fg(Color::DarkGray)));
    }
    debug_assert!(lines.len() > template.manifest.params().len());
    debug_assert!(lines.len() > template.manifest.non_claims.len());
    lines
}

pub(in crate::tui::panels) fn render_workflow_browser(app: &crate::tui::app::State, area: Rect, buf: &mut Buffer) {
    let width = area.width.min(WORKFLOW_BROWSER_WIDTH);
    let height = area.height.min(WORKFLOW_BROWSER_HEIGHT);
    let x = (area.width.saturating_sub(width)) / 2;
    let y = WORKFLOW_BROWSER_TOP_OFFSET.min(area.height.saturating_sub(height));
    let browser_area = Rect::new(x, y, width, height);

    Clear.render(browser_area, buf);
    let block = Block::new()
        .borders(Borders::ALL)
        .title(" Open Workflow ")
        .border_style(Style::new().fg(Color::Blue));
    let inner = block.inner(browser_area);
    debug_assert!(inner.width <= browser_area.width);
    debug_assert!(inner.height <= browser_area.height);
    block.render(browser_area, buf);

    if app.workflow_browser.names.is_empty() {
        let message = app.workflow_browser.message.as_deref().unwrap_or("No saved workflows available");
        Paragraph::new(message)
            .alignment(Alignment::Center)
            .style(Style::new().fg(Color::DarkGray))
            .render(inner, buf);
        return;
    }

    let visible_rows = usize::from(inner.height);
    let lines = app
        .workflow_browser
        .names
        .iter()
        .take(visible_rows)
        .enumerate()
        .map(|(index, name)| {
            let is_selected = index == app.workflow_browser.selected;
            let style = if is_selected {
                Style::new().fg(Color::Black).bg(Color::Cyan)
            } else {
                Style::new().fg(Color::White)
            };
            let prefix = if is_selected { "› " } else { "  " };
            Line::styled(format!("{prefix}{name}"), style)
        })
        .collect::<Vec<_>>();

    Paragraph::new(lines).render(inner, buf);
}

pub(in crate::tui::panels) fn render_quit_confirmation(area: Rect, buf: &mut Buffer) {
    let width = area.width.min(QUIT_CONFIRM_WIDTH);
    let height = area.height.min(QUIT_CONFIRM_HEIGHT);
    let x = (area.width.saturating_sub(width)) / 2;
    let y = area.height.saturating_sub(height).min(QUIT_CONFIRM_TOP_OFFSET);
    let confirm_area = Rect::new(x, y, width, height);

    Clear.render(confirm_area, buf);
    let block = Block::new().borders(Borders::ALL).title(" Unsaved changes ").border_style(
        Style {
            fg: None,
            bg: None,
            underline_color: None,
            add_modifier: Modifier::empty(),
            sub_modifier: Modifier::empty(),
        }
        .fg(Color::Red),
    );
    let inner = block.inner(confirm_area);
    debug_assert!(inner.width <= confirm_area.width);
    debug_assert!(inner.height <= confirm_area.height);
    block.render(confirm_area, buf);

    Paragraph::new(vec![
        Line::styled("Quit without saving?", Style {
            fg: Some(Color::White),
            bg: None,
            underline_color: None,
            add_modifier: Modifier::empty(),
            sub_modifier: Modifier::empty(),
        }),
        Line::styled("y/Enter: quit │ n/Esc: stay", Style {
            fg: Some(Color::DarkGray),
            bg: None,
            underline_color: None,
            add_modifier: Modifier::empty(),
            sub_modifier: Modifier::empty(),
        }),
    ])
    .alignment(Alignment::Center)
    .render(inner, buf);
}

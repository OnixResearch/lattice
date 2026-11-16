//! Diagnostics, run history, and approval summaries.

use super::*;

pub(super) fn render_diagnostics(app: &crate::tui::app::State, area: Rect, buf: &mut Buffer) {
    let width = area.width.min(DIAGNOSTICS_WIDTH);
    let height = area.height.min(DIAGNOSTICS_HEIGHT);
    let x = (area.width.saturating_sub(width)) / 2;
    let y = DIAGNOSTICS_TOP_OFFSET.min(area.height.saturating_sub(height));
    let diagnostics_area = Rect::new(x, y, width, height);

    Clear.render(diagnostics_area, buf);
    let block = Block::new().borders(Borders::ALL).title(" Workflow Diagnostics ").border_style(
        Style {
            fg: None,
            bg: None,
            underline_color: None,
            add_modifier: Modifier::empty(),
            sub_modifier: Modifier::empty(),
        }
        .fg(Color::Red),
    );
    let inner = block.inner(diagnostics_area);
    debug_assert!(inner.width <= diagnostics_area.width);
    debug_assert!(inner.height <= diagnostics_area.height);
    block.render(diagnostics_area, buf);

    if app.diagnostics.diagnostics.is_empty() {
        Paragraph::new("Workflow ready to run")
            .alignment(Alignment::Center)
            .style(
                Style {
                    fg: None,
                    bg: None,
                    underline_color: None,
                    add_modifier: Modifier::empty(),
                    sub_modifier: Modifier::empty(),
                }
                .fg(Color::Green),
            )
            .render(inner, buf);
        return;
    }

    let visible_rows = usize::from(inner.height);
    let lines = app
        .diagnostics
        .diagnostics
        .iter()
        .take(visible_rows)
        .map(|diagnostic| {
            let style = match diagnostic.severity {
                DiagnosticSeverity::Fatal => Style {
                    fg: None,
                    bg: None,
                    underline_color: None,
                    add_modifier: Modifier::empty(),
                    sub_modifier: Modifier::empty(),
                }
                .fg(Color::Red),
                DiagnosticSeverity::Warning => Style {
                    fg: None,
                    bg: None,
                    underline_color: None,
                    add_modifier: Modifier::empty(),
                    sub_modifier: Modifier::empty(),
                }
                .fg(Color::Yellow),
            };
            Line::styled(crate::workflow_validation::format_diagnostic(diagnostic), style)
        })
        .collect::<Vec<_>>();

    Paragraph::new(lines).render(inner, buf);
}

pub(super) fn render_run_history(app: &crate::tui::app::State, area: Rect, buf: &mut Buffer) {
    let width = area.width.min(RUN_HISTORY_WIDTH);
    let height = area.height.min(RUN_HISTORY_HEIGHT);
    let x = (area.width.saturating_sub(width)) / 2;
    let y = RUN_HISTORY_TOP_OFFSET.min(area.height.saturating_sub(height));
    let history_area = Rect::new(x, y, width, height);

    Clear.render(history_area, buf);
    let block = Block::new()
        .borders(Borders::ALL)
        .title(" Run History ")
        .border_style(Style::new().fg(Color::Magenta));
    let inner = block.inner(history_area);
    debug_assert!(inner.width <= history_area.width);
    debug_assert!(inner.height <= history_area.height);
    block.render(history_area, buf);

    if let Some(detail_record) = &app.run_history.detail_record {
        render_run_detail_record(
            DetailView {
                record: detail_record,
                selected_node: app.run_history.selected_detail_node,
                replay_timeline: app.run_history.replay_timeline.as_ref(),
                replay_cursor: app.run_history.replay_cursor,
            },
            inner,
            buf,
        );
        return;
    }

    if let Some(detail) = &app.run_history.detail {
        Paragraph::new(detail.as_str())
            .alignment(Alignment::Center)
            .style(Style::new().fg(Color::White))
            .render(inner, buf);
        return;
    }

    if app.run_history.records.is_empty() {
        let message = app.run_history.message.as_deref().unwrap_or("No run history available");
        Paragraph::new(message)
            .alignment(Alignment::Center)
            .style(Style::new().fg(Color::DarkGray))
            .render(inner, buf);
        return;
    }

    let visible_rows = usize::from(inner.height);
    let lines = app
        .run_history
        .records
        .iter()
        .take(visible_rows)
        .enumerate()
        .map(|(index, record)| {
            run_history_line(index == app.run_history.selected, crate::tui::app::format_run_record(record))
        })
        .collect::<Vec<_>>();

    Paragraph::new(lines).render(inner, buf);
}
fn run_history_line(selected: bool, formatted_record: String) -> Line<'static> {
    let style = if selected {
        Style::new().fg(Color::Black).bg(Color::Magenta)
    } else {
        Style::new().fg(Color::White)
    };
    let prefix = if selected { "› " } else { "  " };
    Line::styled(format!("{prefix}{formatted_record}"), style)
}

// r[impl lattice.workflow_approval.decision]
// r[impl lattice.workflow_approval.resume]
pub(super) fn render_approvals(app: &crate::tui::app::State, area: Rect, buf: &mut Buffer) {
    let width = area.width.min(APPROVAL_WIDTH);
    let height = area.height.min(APPROVAL_HEIGHT);
    let x = (area.width.saturating_sub(width)) / 2;
    let y = APPROVAL_TOP_OFFSET.min(area.height.saturating_sub(height));
    let approval_area = Rect::new(x, y, width, height);
    debug_assert!(approval_area.width <= area.width);
    debug_assert!(approval_area.height <= area.height);
    Clear.render(approval_area, buf);
    let block = Block::new()
        .borders(Borders::ALL)
        .title(" Workflow Approvals ")
        .border_style(Style::new().fg(Color::Green));
    let inner = block.inner(approval_area);
    block.render(approval_area, buf);
    if let Some(detail) = &app.approvals.detail {
        Paragraph::new(detail.as_str()).style(Style::new().fg(Color::White)).render(inner, buf);
        return;
    }
    if app.approvals.records.is_empty() {
        Paragraph::new(app.approvals.message.as_deref().unwrap_or("No workflow approvals available"))
            .alignment(Alignment::Center)
            .style(Style::new().fg(Color::DarkGray))
            .render(inner, buf);
        return;
    }
    let mut lines = app
        .approvals
        .records
        .iter()
        .take(usize::from(inner.height))
        .enumerate()
        .map(|(index, record)| {
            let is_selected = index == app.approvals.selected;
            let style = if is_selected {
                Style::new().fg(Color::Black).bg(Color::Green)
            } else {
                Style::new().fg(Color::White)
            };
            let prefix = if is_selected { "› " } else { "  " };
            Line::styled(
                format!(
                    "{prefix}{} {:?} gate={} run={}",
                    record.safe_summary, record.status, record.gate_id, record.run_id
                ),
                style,
            )
        })
        .collect::<Vec<_>>();
    if let Some(message) = &app.approvals.message {
        lines.push(Line::styled(message.clone(), Style::new().fg(Color::Yellow)));
    }
    Paragraph::new(lines).render(inner, buf);
}

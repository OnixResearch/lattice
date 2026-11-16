//! Typed authoring overlay rendering.

use super::*;

pub(in crate::tui::panels) fn render_authoring(app: &crate::tui::app::State, area: Rect, buf: &mut Buffer) {
    let Some(state) = &app.authoring else {
        return;
    };
    Clear.render(area, buf);
    if area.width < 4 || area.height < 4 {
        Paragraph::new("Esc back").render(area, buf);
        return;
    }
    let block = Block::new().borders(Borders::ALL).title(" Typed Authoring ");
    let inner = block.inner(area);
    debug_assert!(inner.width <= area.width);
    debug_assert!(inner.height <= area.height);
    block.render(area, buf);
    let footer = Rect::new(inner.x, inner.y.saturating_add(inner.height.saturating_sub(1)), inner.width, 1);
    let body = Rect::new(inner.x, inner.y, inner.width, inner.height.saturating_sub(1));
    if let Some(draft) = &state.draft {
        let mut lines = vec![Line::styled(draft.field.label.clone(), Style {
            fg: Some(Color::Yellow),
            bg: None,
            underline_color: None,
            add_modifier: Modifier::empty(),
            sub_modifier: Modifier::empty(),
        })];
        // Reserve room for validation/recovery even when the terminal is short.
        let visible = usize::from(body.width).saturating_mul(usize::from(body.height.saturating_sub(2)));
        let chars = draft.text.chars().count();
        let tail: String = draft.text.chars().skip(chars.saturating_sub(visible)).collect();
        if body.height > 2 {
            lines.push(Line::from(format!("{tail}▏")));
        }
        lines.push(Line::styled(
            state.error.as_deref().unwrap_or(&draft.field.help).to_string(),
            Style {
                fg: None,
                bg: None,
                underline_color: None,
                add_modifier: Modifier::empty(),
                sub_modifier: Modifier::empty(),
            }
            .fg(if state.error.is_some() { Color::Red } else { Color::Cyan }),
        ));
        Paragraph::new(lines).wrap(ratatui::widgets::Wrap { trim: false }).render(body, buf);
        Paragraph::new(if inner.width < 60 {
            "Esc back / Enter save / C-U clear"
        } else {
            "Enter: validate/apply │ Esc: cancel │ Ctrl-U: clear │ Ctrl-J: newline"
        })
        .render(footer, buf);
        return;
    }
    render_authoring_fields(state, body, buf);
    render_list_footer(footer, buf);
}

fn render_authoring_fields(state: &crate::tui::authoring::State, body: Rect, buf: &mut Buffer) {
    let list_height = body.height.saturating_sub(2).max(1);
    let start = state.selected.saturating_sub(usize::from(list_height.saturating_sub(1)));
    let mut lines = visible_authoring_fields(state, start, list_height).collect::<Vec<_>>();
    if state.fields.is_empty() {
        lines.push(Line::from("Select one node for node-owned fields; Esc returns"));
    } else if body.height > list_height
        && let Some(field) = state.fields.get(state.selected)
    {
        lines.push(Line::styled(field.id.clone(), Style::new().fg(Color::Cyan)));
        lines.push(Line::from(field.help.clone()));
    }
    Paragraph::new(lines).render(body, buf);
}

fn visible_authoring_fields(
    state: &crate::tui::authoring::State,
    start: usize,
    height: u16,
) -> impl Iterator<Item = Line<'static>> + '_ {
    state
        .fields
        .iter()
        .enumerate()
        .skip(start)
        .take(usize::from(height))
        .map(move |(index, field)| authoring_field_line(index == state.selected, &field.label))
}

fn authoring_field_line(selected: bool, label: &str) -> Line<'static> {
    Line::styled(
        format!("{} {label}", if selected { "▶" } else { " " }),
        Style::new().fg(if selected { Color::Yellow } else { Color::White }),
    )
}

fn render_list_footer(area: Rect, buf: &mut Buffer) {
    Paragraph::new(if area.width < 60 {
        "Esc back / ↑↓ select / Enter edit"
    } else {
        "↑/↓: select │ PgUp/PgDn/Home/End │ Enter: edit/add/remove │ Esc: close"
    })
    .render(area, buf);
}

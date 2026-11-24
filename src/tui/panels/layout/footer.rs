//! Output pane, status, and contextual keyboard help.

use super::*;

/// Render the output/execution log panel.
pub(in crate::tui::panels) fn render_output(app: &mut crate::tui::app::State, area: Rect, buf: &mut Buffer) {
    let is_focused = matches!(app.focus, FocusPanel::Output);
    let border_style = if is_focused {
        Style {
            fg: None,
            bg: None,
            underline_color: None,
            add_modifier: Modifier::empty(),
            sub_modifier: Modifier::empty(),
        }
        .fg(Color::Magenta)
    } else {
        Style {
            fg: None,
            bg: None,
            underline_color: None,
            add_modifier: Modifier::empty(),
            sub_modifier: Modifier::empty(),
        }
        .fg(Color::DarkGray)
    };

    let block = Block::new().borders(Borders::ALL).title(" Output ").border_style(border_style);

    let inner = block.inner(area);
    debug_assert!(inner.width <= area.width);
    debug_assert!(inner.height <= area.height);
    block.render(area, buf);

    // Coordinate iterators clip output to the representable terminal rectangle.
    let visible_height = usize::from(inner.height);
    let rendered_lines = render_streaming_lines(
        &mut app.output,
        visible_height,
        Style {
            fg: None,
            bg: None,
            underline_color: None,
            add_modifier: Modifier::empty(),
            sub_modifier: Modifier::empty(),
        }
        .fg(Color::DarkGray),
    );
    for (y, line) in (inner.y..inner.bottom()).zip(&rendered_lines) {
        let mut columns = inner.x..inner.right();
        for span in &line.spans {
            for ch in span.content.chars() {
                let Some(x) = columns.next() else {
                    break;
                };
                buf[(x, y)].set_char(ch).set_style(span.style);
            }
        }
    }
}

/// Render the status bar.
// r[impl lattice.tui_ux.portable_authoring_parity.discoverability]
pub(in crate::tui::panels) fn render_status_bar(app: &crate::tui::app::State, area: Rect, buf: &mut Buffer) {
    debug_assert!(area.right() <= buf.area.right());
    debug_assert!(area.bottom() <= buf.area.bottom());
    let mode_span = status_mode(app);

    let wf_name = Span::styled(
        format!(" {}{} ", app.workflow.meta.name, unsaved_indicator(app.has_unsaved_changes()),),
        Style {
            fg: None,
            bg: None,
            underline_color: None,
            add_modifier: Modifier::empty(),
            sub_modifier: Modifier::empty(),
        }
        .fg(Color::White),
    );

    let node_count = app.graph_state.graph.node_count();
    let edge_count = app.graph_state.graph.edge_count();
    let stats = Span::styled(
        format!(" {node_count} nodes │ {edge_count} edges "),
        Style {
            fg: None,
            bg: None,
            underline_color: None,
            add_modifier: Modifier::empty(),
            sub_modifier: Modifier::empty(),
        }
        .fg(Color::DarkGray),
    );

    let help_text = contextual_help_text(HelpContext {
        focus: app.focus,
        palette_open: app.show_palette,
        workflow_browser_open: app.is_workflow_browser_open(),
        add_node_open: app.is_add_node_flow_open(),
        run_history_open: app.is_run_history_open(),
        diagnostics_open: app.is_diagnostics_open(),
        interaction_mode: interaction_mode(&app.graph_state.mode),
        width: area.width,
    });
    let help = Span::styled(
        help_text,
        Style {
            fg: None,
            bg: None,
            underline_color: None,
            add_modifier: Modifier::empty(),
            sub_modifier: Modifier::empty(),
        }
        .fg(Color::DarkGray),
    );

    let line = Line::from(vec![mode_span, wf_name, stats, help]);
    Paragraph::new(line)
        .style(
            Style {
                fg: None,
                bg: None,
                underline_color: None,
                add_modifier: Modifier::empty(),
                sub_modifier: Modifier::empty(),
            }
            .bg(Color::Black),
        )
        .render(area, buf);
}

fn status_mode(app: &crate::tui::app::State) -> Span<'static> {
    match &app.graph_state.mode {
        InteractionMode::Wiring { .. } => Span::styled(
            " WIRING ",
            Style {
                fg: None,
                bg: None,
                underline_color: None,
                add_modifier: Modifier::empty(),
                sub_modifier: Modifier::empty(),
            }
            .bg(Color::Yellow)
            .fg(Color::Black),
        ),
        _ => match app.focus {
            FocusPanel::Canvas => Span::styled(
                " CANVAS ",
                Style {
                    fg: None,
                    bg: None,
                    underline_color: None,
                    add_modifier: Modifier::empty(),
                    sub_modifier: Modifier::empty(),
                }
                .bg(Color::Cyan)
                .fg(Color::Black),
            ),
            FocusPanel::Properties => Span::styled(
                " PROPS ",
                Style {
                    fg: None,
                    bg: None,
                    underline_color: None,
                    add_modifier: Modifier::empty(),
                    sub_modifier: Modifier::empty(),
                }
                .bg(Color::Yellow)
                .fg(Color::Black),
            ),
            FocusPanel::Output => Span::styled(
                " OUTPUT ",
                Style {
                    fg: None,
                    bg: None,
                    underline_color: None,
                    add_modifier: Modifier::empty(),
                    sub_modifier: Modifier::empty(),
                }
                .bg(Color::Magenta)
                .fg(Color::Black),
            ),
        },
    }
}

pub(in crate::tui::panels) fn unsaved_indicator(has_unsaved_changes: bool) -> &'static str {
    if has_unsaved_changes { "*" } else { "" }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::tui::panels) enum Mode {
    Default,
    Wiring,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::tui::panels) struct HelpContext {
    pub(in crate::tui::panels) focus: FocusPanel,
    pub(in crate::tui::panels) palette_open: bool,
    pub(in crate::tui::panels) workflow_browser_open: bool,
    pub(in crate::tui::panels) add_node_open: bool,
    pub(in crate::tui::panels) run_history_open: bool,
    pub(in crate::tui::panels) diagnostics_open: bool,
    pub(in crate::tui::panels) interaction_mode: Mode,
    pub(in crate::tui::panels) width: u16,
}

pub(in crate::tui::panels) fn interaction_mode(mode: &InteractionMode) -> Mode {
    if matches!(mode, InteractionMode::Wiring { .. }) {
        Mode::Wiring
    } else {
        Mode::Default
    }
}

// r[impl n8nclone.tui_ux.help_footer.contextual]
pub(in crate::tui::panels) fn contextual_help_text(context: HelpContext) -> &'static str {
    if context.width < COMPACT_STATUS_HELP_WIDTH {
        return compact_contextual_help_text(context);
    }

    if let Some((full, _)) = modal_help(context) {
        return full;
    }
    if context.interaction_mode == Mode::Wiring {
        return " Click target port: connect │ Esc: cancel wiring ";
    }
    match context.focus {
        FocusPanel::Canvas => {
            " e: cmd │ l: label │ t: timeout │ C-W: fields │ C-D: diagnostics │ v: rows │ C-R: run │ C-P: palette "
        }
        FocusPanel::Properties => " e: cmd │ l: label │ i: image │ p/r/o: agent │ k: creds │ C-W ",
        FocusPanel::Output => " ↑/↓: scroll │ C-P: palette │ A-Tab: focus ",
    }
}

pub(in crate::tui::panels) fn compact_contextual_help_text(context: HelpContext) -> &'static str {
    if let Some((_, compact)) = modal_help(context) {
        return compact;
    }
    if context.interaction_mode == Mode::Wiring {
        return " Click target │ Esc ";
    }
    match context.focus {
        FocusPanel::Canvas => " e cmd │ l label │ v view │ C-W ",
        FocusPanel::Properties => " e cmd │ l label │ C-W ",
        FocusPanel::Output => " ↑/↓ scroll │ A-Tab ",
    }
}

const PALETTE_HELP: (&str, &str) = (" Type: filter │ ↑/↓: select │ Enter: run │ Esc: close ", " Enter: run │ Esc ");
const WORKFLOW_HELP: (&str, &str) = (" ↑/↓: choose workflow │ Enter: open │ Esc: close ", " Enter open │ Esc ");
const ADD_NODE_HELP: (&str, &str) = (" Type: edit field │ Enter: next/create │ Esc: cancel ", " Enter next │ Esc ");
const HISTORY_HELP: (&str, &str) = (" ↑/↓: choose │ Enter: inspect │ r: rerun │ Esc: close ", " Enter inspect │ Esc ");
const DIAGNOSTIC_HELP: (&str, &str) = (" Diagnostics │ Enter/Esc: close ", " Enter/Esc close ");

fn modal_help(context: HelpContext) -> Option<(&'static str, &'static str)> {
    match (
        context.palette_open,
        context.workflow_browser_open,
        context.add_node_open,
        context.run_history_open,
        context.diagnostics_open,
    ) {
        (true, ..) => Some(PALETTE_HELP),
        (_, true, ..) => Some(WORKFLOW_HELP),
        (_, _, true, ..) => Some(ADD_NODE_HELP),
        (_, _, _, true, _) => Some(HISTORY_HELP),
        (_, _, _, _, true) => Some(DIAGNOSTIC_HELP),
        _ => None,
    }
}

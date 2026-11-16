//! Canvas, property editor, and output key routes.

use super::*;

pub(super) fn handle_canvas_key(app: &mut crate::tui::app::State, key: &KeyEvent) -> Option<crate::tui::app::Action> {
    debug_assert_eq!(app.workflow.graph.node_count(), app.graph_state.graph.node_count());
    debug_assert_eq!(app.workflow.graph.edge_count(), app.graph_state.graph.edge_count());
    if let Some(field) = property_edit_shortcut(key) {
        app.start_property_edit(field);
        return None;
    }

    if matches!(key.code, KeyCode::Char('v')) && accepts_palette_text(key.modifiers) {
        app.toggle_canvas_view();
        return None;
    }

    if app.canvas_view == crate::tui::rows::CanvasView::Rows && handle_rows_key(app, key) {
        return None;
    }

    if matches!(key.code, KeyCode::Delete | KeyCode::Backspace)
        && app.graph_state.selected_edge.is_none()
        && !app.graph_state.selected.is_empty()
    {
        app.delete_selected_nodes();
        return None;
    }

    let is_shift = key.modifiers.contains(KeyModifiers::SHIFT);

    let key_name = match key.code {
        KeyCode::Tab => "Tab",
        KeyCode::Enter => "Enter",
        KeyCode::Esc => "Escape",
        KeyCode::Delete => "Delete",
        KeyCode::Backspace => "Backspace",
        KeyCode::Up => "Up",
        KeyCode::Down => "Down",
        KeyCode::Left => "Left",
        KeyCode::Right => "Right",
        _ => return None,
    };

    let can_mutate = canvas_key_may_mutate(key, !app.graph_state.selected.is_empty());
    let actions = app.graph_state.handle_key(key_name, is_shift);
    let has_mutated = can_mutate && !actions.is_empty();
    app.graph_state.apply_actions(&actions);

    // Sync graph changes back to the Lattice-owned workflow model.
    app.sync_workflow_graph_from_state();
    if has_mutated {
        app.mark_workflow_dirty();
    }

    None
}

pub(super) fn handle_rows_key(app: &mut crate::tui::app::State, key: &KeyEvent) -> bool {
    match key.code {
        KeyCode::Up => app.select_row(-1),
        KeyCode::Down => app.select_row(1),
        KeyCode::PageUp => app.select_row(-10),
        KeyCode::PageDown => app.select_row(10),
        KeyCode::Home => app.select_row_boundary(false),
        KeyCode::End => app.select_row_boundary(true),
        KeyCode::Left | KeyCode::Right | KeyCode::Enter | KeyCode::Tab => {}
        _ => return false,
    }
    true
}

pub(super) fn canvas_key_may_mutate(key: &KeyEvent, has_selection: bool) -> bool {
    match key.code {
        KeyCode::Delete | KeyCode::Backspace => true,
        KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right => has_selection,
        KeyCode::Enter
        | KeyCode::Home
        | KeyCode::End
        | KeyCode::PageUp
        | KeyCode::PageDown
        | KeyCode::Tab
        | KeyCode::BackTab
        | KeyCode::Insert
        | KeyCode::F(_)
        | KeyCode::Char(_)
        | KeyCode::Null
        | KeyCode::Esc
        | KeyCode::CapsLock
        | KeyCode::ScrollLock
        | KeyCode::NumLock
        | KeyCode::PrintScreen
        | KeyCode::Pause
        | KeyCode::Menu
        | KeyCode::KeypadBegin
        | KeyCode::Media(_)
        | KeyCode::Modifier(_) => false,
    }
}

pub(super) fn handle_properties_key(
    app: &mut crate::tui::app::State,
    key: &KeyEvent,
) -> Option<crate::tui::app::Action> {
    if let Some(field) = property_edit_shortcut(key) {
        app.start_property_edit(field);
    }
    None
}

pub(super) fn property_edit_shortcut(key: &KeyEvent) -> Option<PropertyEditField> {
    match key.code {
        KeyCode::Char('l') if accepts_palette_text(key.modifiers) => Some(PropertyEditField::Label),
        KeyCode::Char('e') if accepts_palette_text(key.modifiers) => Some(PropertyEditField::Command),
        KeyCode::Char('t') if accepts_palette_text(key.modifiers) => Some(PropertyEditField::TimeoutSecs),
        KeyCode::Char('i') if accepts_palette_text(key.modifiers) => Some(PropertyEditField::ImagePath),
        KeyCode::Char('p') if accepts_palette_text(key.modifiers) => Some(PropertyEditField::AgentPrompt),
        KeyCode::Char('r') if accepts_palette_text(key.modifiers) => Some(PropertyEditField::AgentTarget),
        KeyCode::Char('o') if accepts_palette_text(key.modifiers) => Some(PropertyEditField::AgentOutputMode),
        KeyCode::Char('m') if accepts_palette_text(key.modifiers) => Some(PropertyEditField::AgentModelHint),
        KeyCode::Char('y') if accepts_palette_text(key.modifiers) => Some(PropertyEditField::AgentToolPolicy),
        KeyCode::Char('x') if accepts_palette_text(key.modifiers) => Some(PropertyEditField::AgentContextRefs),
        KeyCode::Char('w') if accepts_palette_text(key.modifiers) => Some(PropertyEditField::AgentCwd),
        KeyCode::Char('n') if accepts_palette_text(key.modifiers) => Some(PropertyEditField::AgentSession),
        KeyCode::Char('k') if accepts_palette_text(key.modifiers) => Some(PropertyEditField::Credentials),
        KeyCode::Char('b') if accepts_palette_text(key.modifiers) => Some(PropertyEditField::ConditionBranches),
        KeyCode::Backspace
        | KeyCode::Enter
        | KeyCode::Left
        | KeyCode::Right
        | KeyCode::Up
        | KeyCode::Down
        | KeyCode::Home
        | KeyCode::End
        | KeyCode::PageUp
        | KeyCode::PageDown
        | KeyCode::Tab
        | KeyCode::BackTab
        | KeyCode::Delete
        | KeyCode::Insert
        | KeyCode::F(_)
        | KeyCode::Char(_)
        | KeyCode::Null
        | KeyCode::Esc
        | KeyCode::CapsLock
        | KeyCode::ScrollLock
        | KeyCode::NumLock
        | KeyCode::PrintScreen
        | KeyCode::Pause
        | KeyCode::Menu
        | KeyCode::KeypadBegin
        | KeyCode::Media(_)
        | KeyCode::Modifier(_) => None,
    }
}

pub(super) fn handle_property_edit_key(app: &mut crate::tui::app::State, key: &KeyEvent) {
    match key.code {
        KeyCode::Enter => {
            app.save_property_edit();
        }
        KeyCode::Esc => {
            app.cancel_property_edit();
        }
        KeyCode::Backspace => {
            app.pop_property_edit_char();
        }
        KeyCode::Char(ch) if accepts_palette_text(key.modifiers) => {
            app.push_property_edit_char(ch);
        }
        _ => {}
    }
}

pub(super) fn handle_output_key(app: &mut crate::tui::app::State, key: &KeyEvent) -> Option<crate::tui::app::Action> {
    match key.code {
        KeyCode::Up => {
            app.output.scroll_up(1);
        }
        KeyCode::Down => {
            app.output.scroll_down(1, OUTPUT_SCROLL_MARGIN_ROWS);
        }
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            return Some(crate::tui::app::Action::Quit);
        }
        _ => {}
    }
    None
}

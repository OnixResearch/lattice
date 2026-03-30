//! Input handling: dispatch key/mouse events to the focused panel.

use crate::tui::app::{App, AppAction, FocusPanel};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};

/// Process a terminal event and return any resulting app action.
pub fn handle_event(app: &mut App, event: &Event) -> Option<AppAction> {
    match event {
        Event::Key(key) => handle_key(app, key),
        Event::Mouse(mouse) => handle_mouse(app, mouse),
        Event::Resize(w, h) => {
            app.graph_state.viewport.resize(*w, *h);
            None
        }
        _ => None,
    }
}

fn handle_key(app: &mut App, key: &KeyEvent) -> Option<AppAction> {
    // Global keybindings (regardless of focus).
    match (key.modifiers, key.code) {
        (KeyModifiers::CONTROL, KeyCode::Char('c')) => return Some(AppAction::Quit),
        (KeyModifiers::CONTROL, KeyCode::Char('q')) => return Some(AppAction::Quit),
        (KeyModifiers::CONTROL, KeyCode::Char('p')) => {
            app.show_palette = !app.show_palette;
            return None;
        }
        (KeyModifiers::CONTROL, KeyCode::Char('r')) => return Some(AppAction::RunWorkflow),
        (KeyModifiers::CONTROL, KeyCode::Char('s')) => return Some(AppAction::SaveWorkflow),
        (KeyModifiers::CONTROL, KeyCode::Char('n')) => return Some(AppAction::AddNode),
        (KeyModifiers::CONTROL, KeyCode::Char('o')) => return Some(AppAction::OpenWorkflow),
        (KeyModifiers::CONTROL, KeyCode::Char('l')) => return Some(AppAction::AutoLayout),
        _ => {}
    }

    // Escape closes palette overlay.
    if key.code == KeyCode::Esc && app.show_palette {
        app.show_palette = false;
        return None;
    }

    // Tab cycles focus between panels.
    if key.modifiers.contains(KeyModifiers::ALT) && key.code == KeyCode::Tab {
        app.cycle_focus();
        return None;
    }

    // Delegate to focused panel.
    match app.focus {
        FocusPanel::Canvas => handle_canvas_key(app, key),
        FocusPanel::Properties => handle_properties_key(app, key),
        FocusPanel::Output => handle_output_key(app, key),
    }
}

fn handle_canvas_key(app: &mut App, key: &KeyEvent) -> Option<AppAction> {
    let shift = key.modifiers.contains(KeyModifiers::SHIFT);

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

    let actions = app.graph_state.handle_key(key_name, shift);
    app.graph_state.apply_actions(&actions);

    // Sync graph changes back to workflow.
    app.workflow.graph = app.graph_state.graph.clone();

    None
}

fn handle_properties_key(_app: &mut App, key: &KeyEvent) -> Option<AppAction> {
    // TODO: property editing (text inputs, dropdowns, etc.)
    match key.code {
        KeyCode::Char('e') => {
            // Could open an editor for the command.
        }
        _ => {}
    }
    None
}

fn handle_output_key(app: &mut App, key: &KeyEvent) -> Option<AppAction> {
    match key.code {
        KeyCode::Up => {
            app.output.scroll_up(1);
        }
        KeyCode::Down => {
            app.output.scroll_down(1, 8);
        }
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            return Some(AppAction::Quit);
        }
        _ => {}
    }
    None
}

fn handle_mouse(app: &mut App, mouse: &MouseEvent) -> Option<AppAction> {
    use rat_nodegraph::view::InteractionMode;

    match mouse.kind {
        MouseEventKind::Down(crossterm::event::MouseButton::Left) => {
            let shift = mouse.modifiers.contains(KeyModifiers::SHIFT);
            let actions = app.graph_state.handle_mouse_click(mouse.column, mouse.row, shift);
            app.graph_state.apply_actions(&actions);
            app.workflow.graph = app.graph_state.graph.clone();

            // Record drag start position.
            let canvas_pos = app.graph_state.viewport.screen_to_canvas(
                mouse.column.saturating_sub(app.graph_state.area.x),
                mouse.row.saturating_sub(app.graph_state.area.y),
            );
            app.last_drag_canvas = Some((canvas_pos.x, canvas_pos.y));
        }
        MouseEventKind::Drag(crossterm::event::MouseButton::Left) => {
            let canvas_pos = app.graph_state.viewport.screen_to_canvas(
                mouse.column.saturating_sub(app.graph_state.area.x),
                mouse.row.saturating_sub(app.graph_state.area.y),
            );

            // During wiring, update the preview cursor instead of dragging nodes.
            if matches!(app.graph_state.mode, InteractionMode::Wiring { .. }) {
                if let InteractionMode::Wiring { cursor_x, cursor_y, .. } = &mut app.graph_state.mode {
                    *cursor_x = canvas_pos.x;
                    *cursor_y = canvas_pos.y;
                }
            } else {
                // Compute drag delta from last canvas position.
                let (dx, dy) = if let Some((prev_x, prev_y)) = app.last_drag_canvas {
                    (canvas_pos.x - prev_x, canvas_pos.y - prev_y)
                } else {
                    (0, 0)
                };

                if dx != 0 || dy != 0 {
                    let actions = app.graph_state.handle_mouse_drag(mouse.column, mouse.row, dx, dy);
                    app.graph_state.apply_actions(&actions);
                    app.workflow.graph = app.graph_state.graph.clone();
                }
            }
            app.last_drag_canvas = Some((canvas_pos.x, canvas_pos.y));
        }
        MouseEventKind::Moved => {
            // Update wiring preview cursor on mouse movement.
            if let InteractionMode::Wiring { cursor_x, cursor_y, .. } = &mut app.graph_state.mode {
                let canvas_pos = app.graph_state.viewport.screen_to_canvas(
                    mouse.column.saturating_sub(app.graph_state.area.x),
                    mouse.row.saturating_sub(app.graph_state.area.y),
                );
                *cursor_x = canvas_pos.x;
                *cursor_y = canvas_pos.y;
            }
        }
        MouseEventKind::Up(crossterm::event::MouseButton::Left) => {
            app.last_drag_canvas = None;
            // Finalize box selection on mouse up.
            if matches!(app.graph_state.mode, InteractionMode::BoxSelect { .. }) {
                let actions = app.graph_state.finish_box_select();
                app.graph_state.apply_actions(&actions);
            }
        }
        MouseEventKind::ScrollUp => {
            app.graph_state.viewport.pan(0, -3);
        }
        MouseEventKind::ScrollDown => {
            app.graph_state.viewport.pan(0, 3);
        }
        _ => {}
    }
    None
}

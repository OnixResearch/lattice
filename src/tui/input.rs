//! Input handling: dispatch key/mouse events to the focused panel.

use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyModifiers;
use crossterm::event::MouseEvent;
use crossterm::event::MouseEventKind;
use rat_canvas::Position;
use rat_fuzzy::Action as FuzzyAction;
use rat_fuzzy::handle_action as handle_fuzzy_action;
use rat_nodegraph::model::Graph;
use rat_nodegraph::view::GraphAction;
use rat_nodegraph::view::InteractionMode;
use rat_nodegraph::view::node_bounds;

use crate::tui::app::AddNodeStep;
use crate::tui::app::FocusPanel;
use crate::tui::app::PropertyEditField;
use crate::tui::app::palette_action;

const OUTPUT_SCROLL_MARGIN_ROWS: usize = 8;
const MOUSE_WHEEL_PAN_ROWS: i32 = 3;
const PORT_SNAP_RADIUS_CELLS: i32 = 1;
const INPUT_PORT_MARKER_X_FROM_NODE_LEFT: i32 = 1;
const OUTPUT_PORT_MARKER_RIGHT_INSET: i32 = 2;
const OUTPUT_PORT_HIT_RIGHT_INSET: i32 = 1;
const PORT_HIT_ROW_FROM_NODE_TOP: i32 = 2;
const PORT_MARKER_ROW_FROM_NODE_TOP: i32 = 3;

mod dialogs;
use dialogs::*;
mod guidance;
use guidance::*;
mod commands;
use commands::*;
mod keyboard;
use keyboard::*;
mod spatial;
use spatial::geometry::*;
use spatial::pointer::*;

/// Process a terminal event and return any resulting app action.
pub fn handle_event(app: &mut crate::tui::app::State, event: &Event) -> Option<crate::tui::app::Action> {
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

fn handle_key(app: &mut crate::tui::app::State, key: &KeyEvent) -> Option<crate::tui::app::Action> {
    debug_assert!(app.workflow_template_browser.selected <= app.workflow_templates.len());
    debug_assert!(!app.workflow_templates.is_empty() || app.workflow_template_browser.selected == 0);
    if app.is_confirming_quit() {
        return handle_quit_confirmation_key(key);
    }

    // Hard quit keybindings remain global even while a modal is open.
    match (key.modifiers, key.code) {
        (KeyModifiers::CONTROL, KeyCode::Char('c')) => return Some(crate::tui::app::Action::Quit),
        (KeyModifiers::CONTROL, KeyCode::Char('q')) => return Some(crate::tui::app::Action::Quit),
        _ => {}
    }

    if app.authoring.is_some() {
        handle_authoring_key(app, key);
        return None;
    }

    if app.property_edit.is_some() {
        handle_property_edit_key(app, key);
        return None;
    }

    if app.is_run_history_open() {
        return handle_run_history_key(app, key);
    }

    if app.is_approval_list_open() {
        return handle_approval_key(app, key);
    }

    if app.is_diagnostics_open() {
        return handle_diagnostics_key(app, key);
    }

    if app.is_workflow_template_browser_open() {
        return handle_workflow_template_browser_key(app, key);
    }

    if app.is_workflow_browser_open() {
        return handle_workflow_browser_key(app, key);
    }

    if app.is_add_node_flow_open() {
        return handle_add_node_key(app, key);
    }

    // Ctrl-P toggles the command palette.
    if matches!((key.modifiers, key.code), (KeyModifiers::CONTROL, KeyCode::Char('p'))) {
        toggle_palette(app);
        return None;
    }

    // Palette input has modal priority over global workflow shortcuts.
    if app.show_palette {
        return handle_palette_key(app, key);
    }

    // Global keybindings (regardless of focus).
    if let Some(action) = global_action(key) {
        return Some(action);
    }

    handle_focused_key(app, key)
}

#[cfg(test)]
mod rows_tests;

#[cfg(test)]
mod tests;

fn global_action(key: &KeyEvent) -> Option<crate::tui::app::Action> {
    match (key.modifiers, key.code) {
        (KeyModifiers::CONTROL, KeyCode::Char('r')) => Some(crate::tui::app::Action::RunWorkflow),
        (KeyModifiers::CONTROL, KeyCode::Char('x')) => Some(crate::tui::app::Action::CancelRun),
        (KeyModifiers::CONTROL, KeyCode::Char('h')) => Some(crate::tui::app::Action::OpenRunHistory),
        (KeyModifiers::CONTROL, KeyCode::Char('a')) => Some(crate::tui::app::Action::OpenApprovals),
        (KeyModifiers::CONTROL, KeyCode::Char('d')) => Some(crate::tui::app::Action::OpenDiagnostics),
        (KeyModifiers::CONTROL, KeyCode::Char('s')) => Some(crate::tui::app::Action::SaveWorkflow),
        (KeyModifiers::CONTROL, KeyCode::Char('n')) => Some(crate::tui::app::Action::AddNode),
        (KeyModifiers::CONTROL, KeyCode::Char('o')) => Some(crate::tui::app::Action::OpenWorkflow),
        (KeyModifiers::CONTROL, KeyCode::Char('l')) => Some(crate::tui::app::Action::AutoLayout),
        (KeyModifiers::CONTROL, KeyCode::Char('e')) => Some(crate::tui::app::Action::EditNodeCommand),
        (KeyModifiers::CONTROL, KeyCode::Char('w')) => Some(crate::tui::app::Action::OpenAuthoring("all")),
        _ => None,
    }
}

fn handle_focused_key(app: &mut crate::tui::app::State, key: &KeyEvent) -> Option<crate::tui::app::Action> {
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

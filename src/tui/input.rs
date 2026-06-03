//! Input handling: dispatch key/mouse events to the focused panel.

use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyModifiers;
use crossterm::event::MouseEvent;
use crossterm::event::MouseEventKind;
use rat_fuzzy::Action as FuzzyAction;
use rat_fuzzy::handle_action as handle_fuzzy_action;

use crate::tui::app::AddNodeStep;
use crate::tui::app::App;
use crate::tui::app::AppAction;
use crate::tui::app::FocusPanel;
use crate::tui::app::PropertyEditField;
use crate::tui::app::palette_action;

const OUTPUT_SCROLL_MARGIN_ROWS: usize = 8;
const MOUSE_WHEEL_PAN_ROWS: i32 = 3;

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
    if app.is_confirming_quit() {
        return handle_quit_confirmation_key(key);
    }

    // Hard quit keybindings remain global even while a modal is open.
    match (key.modifiers, key.code) {
        (KeyModifiers::CONTROL, KeyCode::Char('c')) => return Some(AppAction::Quit),
        (KeyModifiers::CONTROL, KeyCode::Char('q')) => return Some(AppAction::Quit),
        _ => {}
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
    match (key.modifiers, key.code) {
        (KeyModifiers::CONTROL, KeyCode::Char('r')) => return Some(AppAction::RunWorkflow),
        (KeyModifiers::CONTROL, KeyCode::Char('x')) => return Some(AppAction::CancelRun),
        (KeyModifiers::CONTROL, KeyCode::Char('s')) => return Some(AppAction::SaveWorkflow),
        (KeyModifiers::CONTROL, KeyCode::Char('n')) => return Some(AppAction::AddNode),
        (KeyModifiers::CONTROL, KeyCode::Char('o')) => return Some(AppAction::OpenWorkflow),
        (KeyModifiers::CONTROL, KeyCode::Char('l')) => return Some(AppAction::AutoLayout),
        _ => {}
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

fn handle_add_node_key(app: &mut App, key: &KeyEvent) -> Option<AppAction> {
    match add_node_key_action(key, app.add_node_flow.step) {
        AddNodeKeyAction::Close => app.cancel_add_node_flow(),
        AddNodeKeyAction::Confirm => app.confirm_add_node_flow(),
        AddNodeKeyAction::Backspace => app.pop_add_node_char(),
        AddNodeKeyAction::TypeChar(ch) => app.push_add_node_char(ch),
        AddNodeKeyAction::SelectNextTemplate => app.select_next_node_template(),
        AddNodeKeyAction::SelectPreviousTemplate => app.select_previous_node_template(),
        AddNodeKeyAction::ManualShell => app.start_manual_shell_node_flow(),
        AddNodeKeyAction::Ignore => {}
    }
    None
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AddNodeKeyAction {
    Close,
    Confirm,
    Backspace,
    TypeChar(char),
    SelectNextTemplate,
    SelectPreviousTemplate,
    ManualShell,
    Ignore,
}

fn add_node_key_action(key: &KeyEvent, step: AddNodeStep) -> AddNodeKeyAction {
    match key.code {
        KeyCode::Esc => AddNodeKeyAction::Close,
        KeyCode::Enter => AddNodeKeyAction::Confirm,
        KeyCode::Backspace => AddNodeKeyAction::Backspace,
        KeyCode::Down if step == AddNodeStep::TemplateSelect => AddNodeKeyAction::SelectNextTemplate,
        KeyCode::Up if step == AddNodeStep::TemplateSelect => AddNodeKeyAction::SelectPreviousTemplate,
        KeyCode::Char('s') | KeyCode::Char('S')
            if step == AddNodeStep::TemplateSelect && accepts_palette_text(key.modifiers) =>
        {
            AddNodeKeyAction::ManualShell
        }
        KeyCode::Char(ch) if accepts_palette_text(key.modifiers) && step != AddNodeStep::TemplateSelect => {
            AddNodeKeyAction::TypeChar(ch)
        }
        _ => AddNodeKeyAction::Ignore,
    }
}

fn handle_workflow_browser_key(app: &mut App, key: &KeyEvent) -> Option<AppAction> {
    match workflow_browser_key_action(key) {
        WorkflowBrowserKeyAction::Close => app.close_workflow_browser(),
        WorkflowBrowserKeyAction::SelectNext => app.select_next_workflow(),
        WorkflowBrowserKeyAction::SelectPrevious => app.select_previous_workflow(),
        WorkflowBrowserKeyAction::Confirm => app.confirm_workflow_browser_selection(),
        WorkflowBrowserKeyAction::Ignore => {}
    }
    None
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WorkflowBrowserKeyAction {
    Close,
    SelectNext,
    SelectPrevious,
    Confirm,
    Ignore,
}

fn workflow_browser_key_action(key: &KeyEvent) -> WorkflowBrowserKeyAction {
    match key.code {
        KeyCode::Esc => WorkflowBrowserKeyAction::Close,
        KeyCode::Down => WorkflowBrowserKeyAction::SelectNext,
        KeyCode::Up => WorkflowBrowserKeyAction::SelectPrevious,
        KeyCode::Enter => WorkflowBrowserKeyAction::Confirm,
        _ => WorkflowBrowserKeyAction::Ignore,
    }
}

fn handle_quit_confirmation_key(key: &KeyEvent) -> Option<AppAction> {
    match quit_confirmation_key_action(key) {
        QuitConfirmationKeyAction::Confirm => Some(AppAction::ConfirmQuit),
        QuitConfirmationKeyAction::Cancel => Some(AppAction::CancelQuit),
        QuitConfirmationKeyAction::Ignore => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum QuitConfirmationKeyAction {
    Confirm,
    Cancel,
    Ignore,
}

fn quit_confirmation_key_action(key: &KeyEvent) -> QuitConfirmationKeyAction {
    match key.code {
        KeyCode::Enter => QuitConfirmationKeyAction::Confirm,
        KeyCode::Char('y') | KeyCode::Char('Y') if accepts_palette_text(key.modifiers) => {
            QuitConfirmationKeyAction::Confirm
        }
        KeyCode::Esc => QuitConfirmationKeyAction::Cancel,
        KeyCode::Char('n') | KeyCode::Char('N') if accepts_palette_text(key.modifiers) => {
            QuitConfirmationKeyAction::Cancel
        }
        _ => QuitConfirmationKeyAction::Ignore,
    }
}

fn toggle_palette(app: &mut App) {
    if app.show_palette {
        close_palette(app);
    } else {
        app.show_palette = true;
        handle_fuzzy_action(&mut app.palette_state, FuzzyAction::Open, &app.palette_data);
    }
}

fn close_palette(app: &mut App) {
    app.show_palette = false;
    app.palette_state.open = false;
    app.palette_state.query.clear();
    app.palette_state.results.clear();
    app.palette_state.selected = 0;
    app.palette_state.scroll_offset = 0;
}

fn handle_palette_key(app: &mut App, key: &KeyEvent) -> Option<AppAction> {
    match palette_key_action(key) {
        PaletteKeyAction::Close => {
            close_palette(app);
            None
        }
        PaletteKeyAction::Backspace => {
            handle_fuzzy_action(&mut app.palette_state, FuzzyAction::Backspace, &app.palette_data);
            None
        }
        PaletteKeyAction::SelectNext => {
            handle_fuzzy_action(&mut app.palette_state, FuzzyAction::SelectNext, &app.palette_data);
            None
        }
        PaletteKeyAction::SelectPrev => {
            handle_fuzzy_action(&mut app.palette_state, FuzzyAction::SelectPrev, &app.palette_data);
            None
        }
        PaletteKeyAction::TypeChar(ch) => {
            handle_fuzzy_action(&mut app.palette_state, FuzzyAction::TypeChar(ch), &app.palette_data);
            None
        }
        PaletteKeyAction::Confirm => confirm_palette_selection(app),
        PaletteKeyAction::Ignore => None,
    }
}

fn confirm_palette_selection(app: &mut App) -> Option<AppAction> {
    if app.palette_state.result_count() == 0 {
        return None;
    }

    handle_fuzzy_action(&mut app.palette_state, FuzzyAction::Confirm, &app.palette_data);

    let selected = app.palette_state.take_result().and_then(|candidate| palette_action(&candidate.id));
    close_palette(app);
    selected
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PaletteKeyAction {
    Close,
    Backspace,
    SelectNext,
    SelectPrev,
    TypeChar(char),
    Confirm,
    Ignore,
}

fn palette_key_action(key: &KeyEvent) -> PaletteKeyAction {
    match key.code {
        KeyCode::Esc => PaletteKeyAction::Close,
        KeyCode::Enter => PaletteKeyAction::Confirm,
        KeyCode::Backspace => PaletteKeyAction::Backspace,
        KeyCode::Down => PaletteKeyAction::SelectNext,
        KeyCode::Up => PaletteKeyAction::SelectPrev,
        KeyCode::Char(ch) if accepts_palette_text(key.modifiers) => PaletteKeyAction::TypeChar(ch),
        _ => PaletteKeyAction::Ignore,
    }
}

fn accepts_palette_text(modifiers: KeyModifiers) -> bool {
    modifiers.is_empty() || modifiers == KeyModifiers::SHIFT
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

    let may_mutate = canvas_key_may_mutate(key, !app.graph_state.selected.is_empty());
    let actions = app.graph_state.handle_key(key_name, shift);
    let did_mutate = may_mutate && !actions.is_empty();
    app.graph_state.apply_actions(&actions);

    // Sync graph changes back to workflow.
    app.workflow.graph = app.graph_state.graph.clone();
    if did_mutate {
        app.mark_workflow_dirty();
    }

    None
}

fn canvas_key_may_mutate(key: &KeyEvent, has_selection: bool) -> bool {
    match key.code {
        KeyCode::Delete | KeyCode::Backspace => true,
        KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right => has_selection,
        _ => false,
    }
}

fn handle_properties_key(app: &mut App, key: &KeyEvent) -> Option<AppAction> {
    if app.property_edit.is_some() {
        handle_property_edit_key(app, key);
        return None;
    }

    match key.code {
        KeyCode::Char('l') if accepts_palette_text(key.modifiers) => {
            app.start_property_edit(PropertyEditField::Label);
        }
        KeyCode::Char('e') if accepts_palette_text(key.modifiers) => {
            app.start_property_edit(PropertyEditField::Command);
        }
        KeyCode::Char('t') if accepts_palette_text(key.modifiers) => {
            app.start_property_edit(PropertyEditField::TimeoutSecs);
        }
        _ => {}
    }
    None
}

fn handle_property_edit_key(app: &mut App, key: &KeyEvent) {
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

fn handle_output_key(app: &mut App, key: &KeyEvent) -> Option<AppAction> {
    match key.code {
        KeyCode::Up => {
            app.output.scroll_up(1);
        }
        KeyCode::Down => {
            app.output.scroll_down(1, OUTPUT_SCROLL_MARGIN_ROWS);
        }
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            return Some(AppAction::Quit);
        }
        _ => {}
    }
    None
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct GraphCounts {
    nodes: usize,
    edges: usize,
}

fn graph_counts(app: &App) -> GraphCounts {
    GraphCounts {
        nodes: app.graph_state.graph.node_count(),
        edges: app.graph_state.graph.edge_count(),
    }
}

fn graph_counts_changed(before: GraphCounts, after: GraphCounts) -> bool {
    before != after
}

fn handle_mouse(app: &mut App, mouse: &MouseEvent) -> Option<AppAction> {
    use rat_nodegraph::view::InteractionMode;

    match mouse.kind {
        MouseEventKind::Down(crossterm::event::MouseButton::Left) => {
            let before = graph_counts(app);
            let shift = mouse.modifiers.contains(KeyModifiers::SHIFT);
            let actions = app.graph_state.handle_mouse_click(mouse.column, mouse.row, shift);
            app.graph_state.apply_actions(&actions);
            app.workflow.graph = app.graph_state.graph.clone();
            if graph_counts_changed(before, graph_counts(app)) {
                app.mark_workflow_dirty();
            }

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
                    let did_mutate = !actions.is_empty();
                    app.graph_state.apply_actions(&actions);
                    app.workflow.graph = app.graph_state.graph.clone();
                    if did_mutate {
                        app.mark_workflow_dirty();
                    }
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
            app.graph_state.viewport.pan(0, -MOUSE_WHEEL_PAN_ROWS);
        }
        MouseEventKind::ScrollDown => {
            app.graph_state.viewport.pan(0, MOUSE_WHEEL_PAN_ROWS);
        }
        _ => {}
    }
    None
}

#[cfg(test)]
mod tests {
    use crossterm::event::Event;
    use crossterm::event::KeyCode;
    use crossterm::event::KeyEvent;
    use crossterm::event::KeyModifiers;
    use tempfile::TempDir;

    use super::*;
    use crate::config::AppConfig;
    use crate::engine::workflow::NodeConfig;
    use crate::tui::app::WorkflowBrowserState;

    fn test_app() -> (TempDir, App) {
        let temp_dir = tempfile::tempdir().expect("temporary app directory");
        let config = AppConfig {
            data_dir: temp_dir.path().join("data"),
            ..AppConfig::default()
        };
        (temp_dir, App::new(config))
    }

    fn key_event(modifiers: KeyModifiers, code: KeyCode) -> Event {
        Event::Key(KeyEvent::new(code, modifiers))
    }

    #[test]
    fn palette_executes_filtered_action_and_closes() {
        let (_temp_dir, mut app) = test_app();

        assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::CONTROL, KeyCode::Char('p')),), None,);
        assert!(app.show_palette);

        for ch in ['r', 'u', 'n'] {
            assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Char(ch))), None,);
        }

        assert_eq!(
            handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Enter)),
            Some(AppAction::RunWorkflow),
        );
        assert!(!app.show_palette);
    }

    #[test]
    fn palette_navigation_executes_selected_action() {
        let (_temp_dir, mut app) = test_app();

        assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::CONTROL, KeyCode::Char('p')),), None,);
        assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Down)), None,);

        assert_eq!(
            handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Enter)),
            Some(AppAction::RunWorkflow),
        );
        assert!(!app.show_palette);
    }

    #[test]
    fn palette_escape_closes_without_changing_workflow() {
        let (_temp_dir, mut app) = test_app();
        let node_count = app.workflow.graph.node_count();

        assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::CONTROL, KeyCode::Char('p')),), None,);
        assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Char('a'))), None,);
        assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Esc)), None,);

        assert!(!app.show_palette);
        assert!(app.palette_state.query.is_empty());
        assert_eq!(app.workflow.graph.node_count(), node_count);
    }

    #[test]
    fn palette_enter_with_no_match_keeps_recoverable_state() {
        let (_temp_dir, mut app) = test_app();
        let node_count = app.workflow.graph.node_count();

        assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::CONTROL, KeyCode::Char('p')),), None,);
        for ch in ['z', 'z'] {
            assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Char(ch))), None,);
        }
        assert_eq!(app.palette_state.result_count(), 0);

        assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Enter)), None,);

        assert!(app.show_palette);
        assert_eq!(app.palette_state.query, "zz");
        assert_eq!(app.workflow.graph.node_count(), node_count);
    }

    #[test]
    fn quit_confirmation_keys_confirm_or_cancel() {
        let (_temp_dir, mut app) = test_app();
        app.mark_workflow_dirty();
        assert!(!app.request_quit());

        assert_eq!(
            handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Enter)),
            Some(AppAction::ConfirmQuit),
        );

        assert!(!app.request_quit());
        assert_eq!(
            handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Esc)),
            Some(AppAction::CancelQuit),
        );
    }

    #[test]
    fn quit_confirmation_ignores_unrelated_keys() {
        let (_temp_dir, mut app) = test_app();
        app.mark_workflow_dirty();
        assert!(!app.request_quit());

        assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Char('x'))), None,);
        assert!(app.is_confirming_quit());
    }

    #[test]
    fn workflow_browser_keys_navigate_and_close_modal() {
        let (_temp_dir, mut app) = test_app();
        app.workflow_browser = WorkflowBrowserState::open(vec!["first".to_string(), "second".to_string()], None);

        assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Down)), None,);
        assert_eq!(app.workflow_browser.selected, 1);

        assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Up)), None,);
        assert_eq!(app.workflow_browser.selected, 0);

        assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Esc)), None,);
        assert!(!app.is_workflow_browser_open());
    }

    #[test]
    fn property_keys_edit_shell_command() {
        let (_temp_dir, mut app) = test_app();
        app.add_shell_node("Example", "");
        app.clear_workflow_dirty();
        app.focus = FocusPanel::Properties;
        let node_id = app.graph_state.focused.expect("focused node");

        assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Char('e'))), None,);
        for ch in ['e', 'c', 'h', 'o'] {
            assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Char(ch))), None,);
        }
        assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Enter)), None,);

        match app.workflow.node_config(node_id).expect("node config") {
            NodeConfig::Shell { command, .. } => assert_eq!(command, "echo"),
        }
        assert!(app.property_edit.is_none());
        assert!(app.has_unsaved_changes());
    }

    #[test]
    fn add_node_keys_create_shell_node_and_cancel_flow() {
        let (_temp_dir, mut app) = test_app();

        app.open_add_node_flow();
        for _ in 0.."Shell 1".chars().count() {
            assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Backspace)), None,);
        }
        for ch in "One".chars() {
            assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Char(ch))), None,);
        }
        assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Enter)), None,);
        for ch in "echo one".chars() {
            assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Char(ch))), None,);
        }
        assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Enter)), None,);

        assert_eq!(app.workflow.graph.node_count(), 1);
        assert!(!app.is_add_node_flow_open());

        app.open_add_node_flow();
        assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Esc)), None,);
        assert_eq!(app.workflow.graph.node_count(), 1);
        assert!(!app.is_add_node_flow_open());
    }
}

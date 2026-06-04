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
use crate::tui::app::App;
use crate::tui::app::AppAction;
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

    if app.property_edit.is_some() {
        handle_property_edit_key(app, key);
        return None;
    }

    if app.is_run_history_open() {
        return handle_run_history_key(app, key);
    }

    if app.is_diagnostics_open() {
        return handle_diagnostics_key(app, key);
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
        (KeyModifiers::CONTROL, KeyCode::Char('h')) => return Some(AppAction::OpenRunHistory),
        (KeyModifiers::CONTROL, KeyCode::Char('d')) => return Some(AppAction::OpenDiagnostics),
        (KeyModifiers::CONTROL, KeyCode::Char('s')) => return Some(AppAction::SaveWorkflow),
        (KeyModifiers::CONTROL, KeyCode::Char('n')) => return Some(AppAction::AddNode),
        (KeyModifiers::CONTROL, KeyCode::Char('o')) => return Some(AppAction::OpenWorkflow),
        (KeyModifiers::CONTROL, KeyCode::Char('l')) => return Some(AppAction::AutoLayout),
        (KeyModifiers::CONTROL, KeyCode::Char('e')) => return Some(AppAction::EditNodeCommand),
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

fn handle_run_history_key(app: &mut App, key: &KeyEvent) -> Option<AppAction> {
    match run_history_key_action(key) {
        RunHistoryKeyAction::Close => app.close_run_history(),
        RunHistoryKeyAction::SelectNext => app.select_next_run_history(),
        RunHistoryKeyAction::SelectPrevious => app.select_previous_run_history(),
        RunHistoryKeyAction::Inspect => app.inspect_selected_run_history(),
        RunHistoryKeyAction::Rerun => {
            app.rerun_selected_run_history();
        }
        RunHistoryKeyAction::Ignore => {}
    }
    None
}

fn handle_diagnostics_key(app: &mut App, key: &KeyEvent) -> Option<AppAction> {
    if matches!(key.code, KeyCode::Esc | KeyCode::Enter) {
        app.close_diagnostics();
    }
    None
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RunHistoryKeyAction {
    Close,
    SelectNext,
    SelectPrevious,
    Inspect,
    Rerun,
    Ignore,
}

fn run_history_key_action(key: &KeyEvent) -> RunHistoryKeyAction {
    match key.code {
        KeyCode::Esc => RunHistoryKeyAction::Close,
        KeyCode::Down => RunHistoryKeyAction::SelectNext,
        KeyCode::Up => RunHistoryKeyAction::SelectPrevious,
        KeyCode::Enter => RunHistoryKeyAction::Inspect,
        KeyCode::Char('r') | KeyCode::Char('R') if accepts_palette_text(key.modifiers) => RunHistoryKeyAction::Rerun,
        _ => RunHistoryKeyAction::Ignore,
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
    if let Some(field) = property_edit_shortcut(key) {
        app.start_property_edit(field);
        return None;
    }

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
    if let Some(field) = property_edit_shortcut(key) {
        app.start_property_edit(field);
    }
    None
}

fn property_edit_shortcut(key: &KeyEvent) -> Option<PropertyEditField> {
    match key.code {
        KeyCode::Char('l') if accepts_palette_text(key.modifiers) => Some(PropertyEditField::Label),
        KeyCode::Char('e') if accepts_palette_text(key.modifiers) => Some(PropertyEditField::Command),
        KeyCode::Char('t') if accepts_palette_text(key.modifiers) => Some(PropertyEditField::TimeoutSecs),
        _ => None,
    }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CanvasPosition {
    x: i32,
    y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MouseCoordinates {
    column: u16,
    row: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PortSnapTarget {
    marker: CanvasPosition,
    hit: CanvasPosition,
}

fn port_snap_target(graph: &Graph, cursor: CanvasPosition) -> Option<PortSnapTarget> {
    let mut nearest: Option<(i32, PortSnapTarget)> = None;

    for node in graph.nodes() {
        let Some(bounds) = node_bounds(graph, node.id) else {
            continue;
        };

        for (index, _) in node.input_ports.iter().enumerate() {
            let port_index = i32::try_from(index).expect("port index fits in canvas coordinates");
            let marker = CanvasPosition {
                x: bounds.x + INPUT_PORT_MARKER_X_FROM_NODE_LEFT,
                y: bounds.y + PORT_MARKER_ROW_FROM_NODE_TOP + port_index,
            };
            let hit = CanvasPosition {
                x: bounds.x,
                y: bounds.y + PORT_HIT_ROW_FROM_NODE_TOP + port_index,
            };
            nearest = nearest_port_snap_target(nearest, cursor, marker, hit);
        }

        for (index, _) in node.output_ports.iter().enumerate() {
            let port_index = i32::try_from(index).expect("port index fits in canvas coordinates");
            let marker = CanvasPosition {
                x: bounds.x + i32::from(bounds.width) - OUTPUT_PORT_MARKER_RIGHT_INSET,
                y: bounds.y + PORT_MARKER_ROW_FROM_NODE_TOP + port_index,
            };
            let hit = CanvasPosition {
                x: bounds.x + i32::from(bounds.width) - OUTPUT_PORT_HIT_RIGHT_INSET,
                y: bounds.y + PORT_HIT_ROW_FROM_NODE_TOP + port_index,
            };
            nearest = nearest_port_snap_target(nearest, cursor, marker, hit);
        }
    }

    nearest.map(|(_, target)| target)
}

fn nearest_port_snap_target(
    current: Option<(i32, PortSnapTarget)>,
    cursor: CanvasPosition,
    marker: CanvasPosition,
    hit: CanvasPosition,
) -> Option<(i32, PortSnapTarget)> {
    let dx = (cursor.x - marker.x).abs();
    let dy = (cursor.y - marker.y).abs();
    if dx > PORT_SNAP_RADIUS_CELLS || dy > PORT_SNAP_RADIUS_CELLS {
        return current;
    }

    let distance = dx + dy;
    let target = PortSnapTarget { marker, hit };
    match current {
        Some((best_distance, best_target)) if best_distance <= distance => Some((best_distance, best_target)),
        _ => Some((distance, target)),
    }
}

fn apply_graph_actions(app: &mut App, actions: &[GraphAction]) {
    app.graph_state.apply_actions(actions);
    app.workflow.graph = app.graph_state.graph.clone();
}

fn apply_graph_actions_and_mark_count_changes(app: &mut App, before: GraphCounts, actions: &[GraphAction]) {
    apply_graph_actions(app, actions);
    if graph_counts_changed(before, graph_counts(app)) {
        app.mark_workflow_dirty();
    }
}

fn mouse_canvas_position(app: &App, mouse: &MouseEvent) -> CanvasPosition {
    let canvas_pos = app.graph_state.viewport.screen_to_canvas(
        mouse.column.saturating_sub(app.graph_state.area.x),
        mouse.row.saturating_sub(app.graph_state.area.y),
    );
    CanvasPosition {
        x: canvas_pos.x,
        y: canvas_pos.y,
    }
}

fn snapped_wiring_canvas_position(app: &App, mouse: &MouseEvent) -> CanvasPosition {
    let cursor = mouse_canvas_position(app, mouse);
    port_snap_target(&app.graph_state.graph, cursor).map_or(cursor, |target| target.marker)
}

fn graph_click_coordinates(app: &App, mouse: &MouseEvent) -> MouseCoordinates {
    let cursor = mouse_canvas_position(app, mouse);
    let hit = port_snap_target(&app.graph_state.graph, cursor).map_or(cursor, |target| target.hit);
    canvas_to_mouse_coordinates(app, hit).unwrap_or(MouseCoordinates {
        column: mouse.column,
        row: mouse.row,
    })
}

fn canvas_to_mouse_coordinates(app: &App, canvas: CanvasPosition) -> Option<MouseCoordinates> {
    app.graph_state
        .viewport
        .canvas_to_screen(Position::new(canvas.x, canvas.y))
        .map(|(column, row)| MouseCoordinates {
            column: column.saturating_add(app.graph_state.area.x),
            row: row.saturating_add(app.graph_state.area.y),
        })
}

fn canvas_position_tuple(position: CanvasPosition) -> (i32, i32) {
    (position.x, position.y)
}

fn is_drag_release(start: Option<(i32, i32)>, end: CanvasPosition) -> bool {
    matches!(start, Some(start) if start != canvas_position_tuple(end))
}

fn handle_mouse(app: &mut App, mouse: &MouseEvent) -> Option<AppAction> {
    match mouse.kind {
        MouseEventKind::Down(crossterm::event::MouseButton::Left) => {
            let before = graph_counts(app);
            let shift = mouse.modifiers.contains(KeyModifiers::SHIFT);
            let click = graph_click_coordinates(app, mouse);
            let actions = app.graph_state.handle_mouse_click(click.column, click.row, shift);
            apply_graph_actions_and_mark_count_changes(app, before, &actions);

            // Record drag start position.
            let drag_start = if matches!(app.graph_state.mode, InteractionMode::Wiring { .. }) {
                snapped_wiring_canvas_position(app, mouse)
            } else {
                mouse_canvas_position(app, mouse)
            };
            app.last_drag_canvas = Some(canvas_position_tuple(drag_start));
        }
        MouseEventKind::Drag(crossterm::event::MouseButton::Left) => {
            let canvas_pos = mouse_canvas_position(app, mouse);

            // During wiring, update the preview cursor instead of dragging nodes.
            if matches!(app.graph_state.mode, InteractionMode::Wiring { .. }) {
                let snapped_pos = snapped_wiring_canvas_position(app, mouse);
                if let InteractionMode::Wiring { cursor_x, cursor_y, .. } = &mut app.graph_state.mode {
                    *cursor_x = snapped_pos.x;
                    *cursor_y = snapped_pos.y;
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
                    apply_graph_actions(app, &actions);
                    if did_mutate {
                        app.mark_workflow_dirty();
                    }
                }

                app.last_drag_canvas = Some(canvas_position_tuple(canvas_pos));
            }
        }
        MouseEventKind::Moved => {
            // Update wiring preview cursor on mouse movement.
            let canvas_pos = snapped_wiring_canvas_position(app, mouse);
            if let InteractionMode::Wiring { cursor_x, cursor_y, .. } = &mut app.graph_state.mode {
                *cursor_x = canvas_pos.x;
                *cursor_y = canvas_pos.y;
            }
        }
        MouseEventKind::Up(crossterm::event::MouseButton::Left) => {
            let canvas_pos = snapped_wiring_canvas_position(app, mouse);
            let should_complete_drag_wiring = matches!(app.graph_state.mode, InteractionMode::Wiring { .. })
                && is_drag_release(app.last_drag_canvas, canvas_pos);
            app.last_drag_canvas = None;

            if should_complete_drag_wiring {
                let before = graph_counts(app);
                let shift = mouse.modifiers.contains(KeyModifiers::SHIFT);
                let click = graph_click_coordinates(app, mouse);
                let actions = app.graph_state.handle_mouse_click(click.column, click.row, shift);
                apply_graph_actions_and_mark_count_changes(app, before, &actions);
            } else if matches!(app.graph_state.mode, InteractionMode::BoxSelect { .. }) {
                // Finalize box selection on mouse up.
                let actions = app.graph_state.finish_box_select();
                apply_graph_actions(app, &actions);
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
    use crossterm::event::MouseButton;
    use rat_nodegraph::view::node_bounds;
    use tempfile::TempDir;

    use super::*;
    use crate::config::AppConfig;
    use crate::engine::workflow::NodeConfig;
    use crate::tui::app::PropertyEditState;
    use crate::tui::app::WorkflowBrowserState;

    fn test_app() -> (TempDir, App) {
        let temp_dir = tempfile::tempdir().expect("temporary app directory");
        let config = AppConfig {
            data_dir: temp_dir.path().join("data"),
            ..AppConfig::default()
        };
        (temp_dir, App::new(config))
    }

    const NODE_BODY_INSET: i32 = 1;

    fn key_event(modifiers: KeyModifiers, code: KeyCode) -> Event {
        Event::Key(KeyEvent::new(code, modifiers))
    }

    fn mouse_event(kind: MouseEventKind, column: u16, row: u16) -> Event {
        Event::Mouse(MouseEvent {
            kind,
            column,
            row,
            modifiers: KeyModifiers::empty(),
        })
    }

    fn canvas_to_screen(app: &App, canvas_x: i32, canvas_y: i32) -> (u16, u16) {
        let viewport = &app.graph_state.viewport;
        let graph_area = app.graph_state.area;
        let screen_x = canvas_x - viewport.offset_x + i32::from(graph_area.x);
        let screen_y = canvas_y - viewport.offset_y + i32::from(graph_area.y);
        (
            u16::try_from(screen_x).expect("canvas x is visible in test viewport"),
            u16::try_from(screen_y).expect("canvas y is visible in test viewport"),
        )
    }

    fn screen_to_canvas(app: &App, column: u16, row: u16) -> CanvasPosition {
        let canvas_pos = app.graph_state.viewport.screen_to_canvas(
            column.saturating_sub(app.graph_state.area.x),
            row.saturating_sub(app.graph_state.area.y),
        );
        CanvasPosition {
            x: canvas_pos.x,
            y: canvas_pos.y,
        }
    }

    fn shell_port_positions(app: &App) -> ((u16, u16), (u16, u16), (u16, u16)) {
        let node_ids = app.graph_state.graph.node_ids();
        let source_id = node_ids.first().copied().expect("source node exists");
        let target_id = node_ids.get(1).copied().expect("target node exists");
        let source_bounds = node_bounds(&app.graph_state.graph, source_id).expect("source bounds");
        let target_bounds = node_bounds(&app.graph_state.graph, target_id).expect("target bounds");
        let source_output = canvas_to_screen(
            app,
            source_bounds.x + i32::from(source_bounds.width) - OUTPUT_PORT_MARKER_RIGHT_INSET,
            source_bounds.y + PORT_MARKER_ROW_FROM_NODE_TOP,
        );
        let target_input = canvas_to_screen(
            app,
            target_bounds.x + INPUT_PORT_MARKER_X_FROM_NODE_LEFT,
            target_bounds.y + PORT_MARKER_ROW_FROM_NODE_TOP,
        );
        let target_body = canvas_to_screen(app, target_bounds.x + NODE_BODY_INSET, target_bounds.y + NODE_BODY_INSET);
        (source_output, target_input, target_body)
    }

    fn app_with_two_shell_nodes() -> (TempDir, App) {
        let (temp_dir, mut app) = test_app();
        app.add_shell_node("Source", "printf source");
        app.add_shell_node("Target", "cat");
        app.clear_workflow_dirty();
        (temp_dir, app)
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
            Some(AppAction::EditNodeCommand),
        );
        assert!(!app.show_palette);
    }

    #[test]
    fn palette_filters_to_edit_command_action() {
        let (_temp_dir, mut app) = test_app();

        assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::CONTROL, KeyCode::Char('p')),), None,);
        for ch in "command".chars() {
            assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Char(ch))), None,);
        }

        assert_eq!(
            handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Enter)),
            Some(AppAction::EditNodeCommand),
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
    fn mouse_drag_release_creates_compatible_edge() {
        let (_temp_dir, mut app) = app_with_two_shell_nodes();
        let (source_output, target_input, _) = shell_port_positions(&app);

        assert_eq!(app.workflow.graph.edge_count(), 0);
        assert_eq!(
            handle_event(
                &mut app,
                &mouse_event(MouseEventKind::Down(MouseButton::Left), source_output.0, source_output.1),
            ),
            None,
        );
        assert!(matches!(app.graph_state.mode, InteractionMode::Wiring { .. }));

        assert_eq!(
            handle_event(
                &mut app,
                &mouse_event(MouseEventKind::Drag(MouseButton::Left), target_input.0, target_input.1),
            ),
            None,
        );
        assert_eq!(app.workflow.graph.edge_count(), 0);

        assert_eq!(
            handle_event(&mut app, &mouse_event(MouseEventKind::Up(MouseButton::Left), target_input.0, target_input.1),),
            None,
        );

        assert_eq!(app.workflow.graph.edge_count(), 1);
        assert!(matches!(app.graph_state.mode, InteractionMode::Normal));
        assert!(app.has_unsaved_changes());
    }

    #[test]
    fn mouse_drag_release_near_visible_input_snaps_to_edge() {
        let (_temp_dir, mut app) = app_with_two_shell_nodes();
        let (source_output, target_input, _) = shell_port_positions(&app);
        let snap_offset = u16::try_from(PORT_SNAP_RADIUS_CELLS).expect("snap radius fits in terminal coordinates");
        let near_target = (target_input.0.saturating_add(snap_offset), target_input.1);

        assert_eq!(
            handle_event(
                &mut app,
                &mouse_event(MouseEventKind::Down(MouseButton::Left), source_output.0, source_output.1),
            ),
            None,
        );
        assert_eq!(
            handle_event(&mut app, &mouse_event(MouseEventKind::Drag(MouseButton::Left), near_target.0, near_target.1),),
            None,
        );
        let target_canvas = screen_to_canvas(&app, target_input.0, target_input.1);
        assert!(
            matches!(
                &app.graph_state.mode,
                InteractionMode::Wiring { cursor_x, cursor_y, .. }
                    if *cursor_x == target_canvas.x && *cursor_y == target_canvas.y
            ),
            "wiring preview should snap to the visible input dot",
        );
        assert_eq!(
            handle_event(&mut app, &mouse_event(MouseEventKind::Up(MouseButton::Left), near_target.0, near_target.1),),
            None,
        );

        assert_eq!(app.workflow.graph.edge_count(), 1);
        assert!(matches!(app.graph_state.mode, InteractionMode::Normal));
        assert!(app.has_unsaved_changes());
    }

    #[test]
    fn mouse_click_release_keeps_click_to_click_wiring_available() {
        let (_temp_dir, mut app) = app_with_two_shell_nodes();
        let (source_output, target_input, _) = shell_port_positions(&app);

        assert_eq!(
            handle_event(
                &mut app,
                &mouse_event(MouseEventKind::Down(MouseButton::Left), source_output.0, source_output.1),
            ),
            None,
        );
        assert_eq!(
            handle_event(
                &mut app,
                &mouse_event(MouseEventKind::Up(MouseButton::Left), source_output.0, source_output.1),
            ),
            None,
        );
        assert!(matches!(app.graph_state.mode, InteractionMode::Wiring { .. }));
        assert_eq!(app.workflow.graph.edge_count(), 0);
        assert!(!app.has_unsaved_changes());

        assert_eq!(
            handle_event(
                &mut app,
                &mouse_event(MouseEventKind::Down(MouseButton::Left), target_input.0, target_input.1),
            ),
            None,
        );

        assert_eq!(app.workflow.graph.edge_count(), 1);
        assert!(matches!(app.graph_state.mode, InteractionMode::Normal));
        assert!(app.has_unsaved_changes());
    }

    #[test]
    fn mouse_drag_release_on_non_port_cancels_wiring_without_edge() {
        let (_temp_dir, mut app) = app_with_two_shell_nodes();
        let (source_output, _, target_body) = shell_port_positions(&app);

        assert_eq!(
            handle_event(
                &mut app,
                &mouse_event(MouseEventKind::Down(MouseButton::Left), source_output.0, source_output.1),
            ),
            None,
        );
        assert!(matches!(app.graph_state.mode, InteractionMode::Wiring { .. }));

        assert_eq!(
            handle_event(&mut app, &mouse_event(MouseEventKind::Drag(MouseButton::Left), target_body.0, target_body.1),),
            None,
        );
        assert_eq!(
            handle_event(&mut app, &mouse_event(MouseEventKind::Up(MouseButton::Left), target_body.0, target_body.1),),
            None,
        );

        assert_eq!(app.workflow.graph.edge_count(), 0);
        assert!(matches!(app.graph_state.mode, InteractionMode::Normal));
        assert!(!app.has_unsaved_changes());
    }

    #[test]
    fn canvas_key_edits_shell_command_without_focus_change() {
        let (_temp_dir, mut app) = test_app();
        app.add_shell_node("Example", "");
        app.clear_workflow_dirty();
        app.focus = FocusPanel::Canvas;
        let node_id = app.graph_state.focused.expect("focused node");

        assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Char('e'))), None,);
        assert!(matches!(
            app.property_edit,
            Some(PropertyEditState {
                field: PropertyEditField::Command,
                ..
            })
        ));
        for ch in ['e', 'c', 'h', 'o'] {
            assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Char(ch))), None,);
        }
        assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Enter)), None,);

        match app.workflow.node_config(node_id).expect("node config") {
            NodeConfig::Shell { command, .. } => assert_eq!(command, "echo"),
            _ => panic!("expected shell config"),
        }
        assert!(app.property_edit.is_none());
        assert!(app.has_unsaved_changes());
    }

    #[test]
    fn canvas_key_cancels_property_edit_without_mutating_command() {
        let (_temp_dir, mut app) = test_app();
        app.add_shell_node("Example", "echo old");
        app.clear_workflow_dirty();
        app.focus = FocusPanel::Canvas;
        let node_id = app.graph_state.focused.expect("focused node");

        assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Char('e'))), None,);
        assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Char('x'))), None,);
        assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Esc)), None,);

        match app.workflow.node_config(node_id).expect("node config") {
            NodeConfig::Shell { command, .. } => assert_eq!(command, "echo old"),
            _ => panic!("expected shell config"),
        }
        assert!(app.property_edit.is_none());
        assert!(!app.has_unsaved_changes());
    }

    #[test]
    fn ctrl_e_requests_selected_command_edit_action() {
        let (_temp_dir, mut app) = test_app();
        app.focus = FocusPanel::Output;

        assert_eq!(
            handle_event(&mut app, &key_event(KeyModifiers::CONTROL, KeyCode::Char('e'))),
            Some(AppAction::EditNodeCommand),
        );
    }

    #[test]
    fn edit_shortcut_without_selection_does_not_mutate_workflow() {
        let (_temp_dir, mut app) = test_app();
        app.clear_workflow_dirty();

        assert_eq!(handle_event(&mut app, &key_event(KeyModifiers::empty(), KeyCode::Char('e'))), None,);

        assert!(app.property_edit.is_none());
        assert_eq!(app.workflow.graph.node_count(), 0);
        assert!(!app.has_unsaved_changes());
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
            _ => panic!("expected shell config"),
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

//! Guided node entry key decisions.

use super::*;

pub(super) fn handle_add_node_key(app: &mut crate::tui::app::State, key: &KeyEvent) -> Option<crate::tui::app::Action> {
    debug_assert!(app.add_node_flow.open);
    debug_assert!(
        app.add_node_flow.step != AddNodeStep::TemplateParameter
            || app.add_node_flow.parameter_index < app.add_node_flow.parameter_values.len()
    );
    match add_node_key_action(key, app.add_node_flow.step) {
        AddNodeKeyAction::Close => app.cancel_add_node_flow(),
        AddNodeKeyAction::Confirm => app.confirm_add_node_flow(),
        AddNodeKeyAction::Backspace => app.pop_add_node_char(),
        AddNodeKeyAction::TypeChar(ch) => app.push_add_node_char(ch),
        AddNodeKeyAction::SelectNextTemplate => app.select_next_node_template(),
        AddNodeKeyAction::SelectPreviousTemplate => app.select_previous_node_template(),
        AddNodeKeyAction::ManualShell => app.start_manual_shell_node_flow(),
        AddNodeKeyAction::Http => app.start_http_node_flow(),
        AddNodeKeyAction::File => app.start_file_node_flow(),
        AddNodeKeyAction::ImageInput => app.start_image_input_node_flow(),
        AddNodeKeyAction::PiAgent => app.start_pi_agent_node_flow(),
        AddNodeKeyAction::ClankersAgent => app.start_clankers_agent_node_flow(),
        AddNodeKeyAction::Condition => app.start_condition_node_flow(),
        AddNodeKeyAction::Merge => app.start_merge_node_flow(),
        AddNodeKeyAction::Subworkflow => app.start_subworkflow_node_flow(),
        AddNodeKeyAction::TemplateSelect => app.start_template_node_flow(),
        AddNodeKeyAction::Ignore => {}
    }
    None
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum AddNodeKeyAction {
    Close,
    Confirm,
    Backspace,
    TypeChar(char),
    SelectNextTemplate,
    SelectPreviousTemplate,
    ManualShell,
    Http,
    File,
    ImageInput,
    PiAgent,
    ClankersAgent,
    Condition,
    Merge,
    Subworkflow,
    TemplateSelect,
    Ignore,
}

pub(super) fn add_node_key_action(key: &KeyEvent, step: AddNodeStep) -> AddNodeKeyAction {
    match key.code {
        KeyCode::Esc => AddNodeKeyAction::Close,
        KeyCode::Enter => AddNodeKeyAction::Confirm,
        KeyCode::Backspace => AddNodeKeyAction::Backspace,
        KeyCode::Down if step == AddNodeStep::TemplateSelect => AddNodeKeyAction::SelectNextTemplate,
        KeyCode::Up if step == AddNodeStep::TemplateSelect => AddNodeKeyAction::SelectPreviousTemplate,
        KeyCode::Char('s') | KeyCode::Char('S')
            if matches!(step, AddNodeStep::Kind | AddNodeStep::TemplateSelect)
                && accepts_palette_text(key.modifiers) =>
        {
            AddNodeKeyAction::ManualShell
        }
        KeyCode::Char('h') | KeyCode::Char('H') if step == AddNodeStep::Kind && accepts_palette_text(key.modifiers) => {
            AddNodeKeyAction::Http
        }
        KeyCode::Char('f') | KeyCode::Char('F') if step == AddNodeStep::Kind && accepts_palette_text(key.modifiers) => {
            AddNodeKeyAction::File
        }
        KeyCode::Char('i') | KeyCode::Char('I') if step == AddNodeStep::Kind && accepts_palette_text(key.modifiers) => {
            AddNodeKeyAction::ImageInput
        }
        KeyCode::Char('p') | KeyCode::Char('P') if step == AddNodeStep::Kind && accepts_palette_text(key.modifiers) => {
            AddNodeKeyAction::PiAgent
        }
        KeyCode::Char('c') | KeyCode::Char('C') if step == AddNodeStep::Kind && accepts_palette_text(key.modifiers) => {
            AddNodeKeyAction::ClankersAgent
        }
        KeyCode::Char('d') | KeyCode::Char('D') if step == AddNodeStep::Kind && accepts_palette_text(key.modifiers) => {
            AddNodeKeyAction::Condition
        }
        KeyCode::Char('m') | KeyCode::Char('M') if step == AddNodeStep::Kind && accepts_palette_text(key.modifiers) => {
            AddNodeKeyAction::Merge
        }
        KeyCode::Char('w') | KeyCode::Char('W') if step == AddNodeStep::Kind && accepts_palette_text(key.modifiers) => {
            AddNodeKeyAction::Subworkflow
        }
        KeyCode::Char('t') | KeyCode::Char('T') if step == AddNodeStep::Kind && accepts_palette_text(key.modifiers) => {
            AddNodeKeyAction::TemplateSelect
        }
        KeyCode::Char(ch)
            if accepts_palette_text(key.modifiers)
                && !matches!(step, AddNodeStep::Kind | AddNodeStep::TemplateSelect) =>
        {
            AddNodeKeyAction::TypeChar(ch)
        }
        _ => AddNodeKeyAction::Ignore,
    }
}

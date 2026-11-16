//! Modal ownership and browsing key routes.

use super::*;

pub(super) fn handle_authoring_key(app: &mut crate::tui::app::State, key: &KeyEvent) {
    if key.code == KeyCode::Esc {
        app.cancel_authoring();
        return;
    }
    if key.code == KeyCode::Enter {
        app.confirm_authoring();
        return;
    }
    let Some(state) = &mut app.authoring else {
        return;
    };
    if let Some(draft) = &mut state.draft {
        edit_authoring_draft(draft, key);
        state.error = None;
    } else {
        select_authoring_field(state, key);
    }
}

fn edit_authoring_draft(draft: &mut crate::tui::authoring::Draft, key: &KeyEvent) {
    match (key.modifiers, key.code) {
        (KeyModifiers::CONTROL, KeyCode::Char('u')) => draft.text.clear(),
        (KeyModifiers::CONTROL, KeyCode::Char('j')) => draft.text.push('\n'),
        (_, KeyCode::Backspace) => {
            draft.text.pop();
        }
        (modifiers, KeyCode::Char(ch)) if accepts_palette_text(modifiers) => draft.text.push(ch),
        _ => {}
    }
}

fn select_authoring_field(state: &mut crate::tui::authoring::State, key: &KeyEvent) {
    let last = state.fields.len().saturating_sub(1);
    match key.code {
        KeyCode::Up => state.selected = state.selected.saturating_sub(1),
        KeyCode::Down | KeyCode::Tab => state.selected = state.selected.saturating_add(1).min(last),
        KeyCode::Home => state.selected = 0,
        KeyCode::End => state.selected = last,
        KeyCode::PageUp => state.selected = state.selected.saturating_sub(8),
        KeyCode::PageDown => state.selected = state.selected.saturating_add(8).min(last),
        _ => {}
    }
}

pub(super) fn handle_run_history_key(
    app: &mut crate::tui::app::State,
    key: &KeyEvent,
) -> Option<crate::tui::app::Action> {
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

pub(super) fn handle_approval_key(app: &mut crate::tui::app::State, key: &KeyEvent) -> Option<crate::tui::app::Action> {
    match key.code {
        KeyCode::Esc => app.close_approvals(),
        KeyCode::Down => app.select_next_approval(),
        KeyCode::Up => app.select_previous_approval(),
        KeyCode::Enter => app.inspect_selected_approval(),
        _ => {}
    }
    None
}

pub(super) fn handle_diagnostics_key(
    app: &mut crate::tui::app::State,
    key: &KeyEvent,
) -> Option<crate::tui::app::Action> {
    if matches!(key.code, KeyCode::Esc | KeyCode::Enter) {
        app.close_diagnostics();
    }
    None
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RunHistoryKeyAction {
    Close,
    SelectNext,
    SelectPrevious,
    Inspect,
    Rerun,
    Ignore,
}

pub(super) fn run_history_key_action(key: &KeyEvent) -> RunHistoryKeyAction {
    match key.code {
        KeyCode::Esc => RunHistoryKeyAction::Close,
        KeyCode::Down => RunHistoryKeyAction::SelectNext,
        KeyCode::Up => RunHistoryKeyAction::SelectPrevious,
        KeyCode::Enter => RunHistoryKeyAction::Inspect,
        KeyCode::Char('r') | KeyCode::Char('R') if accepts_palette_text(key.modifiers) => RunHistoryKeyAction::Rerun,
        _ => RunHistoryKeyAction::Ignore,
    }
}

pub(super) fn handle_workflow_template_browser_key(
    app: &mut crate::tui::app::State,
    key: &KeyEvent,
) -> Option<crate::tui::app::Action> {
    match key.code {
        KeyCode::Esc => app.close_workflow_template_browser(),
        KeyCode::Down => app.select_next_workflow_template(),
        KeyCode::Up => app.select_previous_workflow_template(),
        KeyCode::Enter => app.confirm_workflow_template_browser(),
        _ => {}
    }
    None
}

pub(super) fn handle_workflow_browser_key(
    app: &mut crate::tui::app::State,
    key: &KeyEvent,
) -> Option<crate::tui::app::Action> {
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
pub(super) enum WorkflowBrowserKeyAction {
    Close,
    SelectNext,
    SelectPrevious,
    Confirm,
    Ignore,
}

pub(super) fn workflow_browser_key_action(key: &KeyEvent) -> WorkflowBrowserKeyAction {
    match key.code {
        KeyCode::Esc => WorkflowBrowserKeyAction::Close,
        KeyCode::Down => WorkflowBrowserKeyAction::SelectNext,
        KeyCode::Up => WorkflowBrowserKeyAction::SelectPrevious,
        KeyCode::Enter => WorkflowBrowserKeyAction::Confirm,
        _ => WorkflowBrowserKeyAction::Ignore,
    }
}

pub(super) fn handle_quit_confirmation_key(key: &KeyEvent) -> Option<crate::tui::app::Action> {
    match quit_confirmation_key_action(key) {
        QuitConfirmationKeyAction::Confirm => Some(crate::tui::app::Action::ConfirmQuit),
        QuitConfirmationKeyAction::Cancel => Some(crate::tui::app::Action::CancelQuit),
        QuitConfirmationKeyAction::Ignore => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum QuitConfirmationKeyAction {
    Confirm,
    Cancel,
    Ignore,
}

pub(super) fn quit_confirmation_key_action(key: &KeyEvent) -> QuitConfirmationKeyAction {
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

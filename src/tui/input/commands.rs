//! Command palette interaction.

use super::*;

pub(super) fn toggle_palette(app: &mut crate::tui::app::State) {
    if app.show_palette {
        close_palette(app);
    } else {
        app.show_palette = true;
        handle_fuzzy_action(&mut app.palette_state, FuzzyAction::Open, &app.palette_data);
    }
}

pub(super) fn close_palette(app: &mut crate::tui::app::State) {
    app.show_palette = false;
    app.palette_state.open = false;
    app.palette_state.query.clear();
    app.palette_state.results.clear();
    app.palette_state.selected = 0;
    app.palette_state.scroll_offset = 0;
}

pub(super) fn handle_palette_key(app: &mut crate::tui::app::State, key: &KeyEvent) -> Option<crate::tui::app::Action> {
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

pub(super) fn confirm_palette_selection(app: &mut crate::tui::app::State) -> Option<crate::tui::app::Action> {
    if app.palette_state.result_count() == 0 {
        return None;
    }

    handle_fuzzy_action(&mut app.palette_state, FuzzyAction::Confirm, &app.palette_data);

    let selected = app.palette_state.take_result().and_then(|candidate| palette_action(&candidate.id));
    close_palette(app);
    selected
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PaletteKeyAction {
    Close,
    Backspace,
    SelectNext,
    SelectPrev,
    TypeChar(char),
    Confirm,
    Ignore,
}

pub(super) fn palette_key_action(key: &KeyEvent) -> PaletteKeyAction {
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

pub(super) fn accepts_palette_text(modifiers: KeyModifiers) -> bool {
    modifiers.is_empty() || modifiers == KeyModifiers::SHIFT
}

//! Actions for the fuzzy finder.

use crate::state::FuzzyState;
use crate::types::FuzzySource;

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Open,
    Close,
    TypeChar(char),
    Backspace,
    SelectNext,
    SelectPrev,
    Confirm,
    Cancel,
}

pub fn handle_action(state: &mut FuzzyState, action: Action, source: &dyn FuzzySource) {
    match action {
        Action::Open => {
            state.open = true;
            state.query.clear();
            state.update_results(source);
        }
        Action::Close | Action::Cancel => {
            if !state.query.is_empty() {
                state.query.clear();
                state.update_results(source);
            } else {
                state.open = false;
            }
        }
        Action::TypeChar(c) => {
            state.query.push(c);
            state.update_results(source);
        }
        Action::Backspace => {
            state.query.pop();
            state.update_results(source);
        }
        Action::SelectNext => {
            if !state.results.is_empty() {
                state.selected = (state.selected + 1).min(state.results.len() - 1);
            }
        }
        Action::SelectPrev => {
            state.selected = state.selected.saturating_sub(1);
        }
        Action::Confirm => {
            state.confirm(source);
        }
    }
}

//! Physical-action normalization for the shared fuzzy prompt adapter.

use crate::state::FuzzyState;
use crate::types::FuzzySource;

/// Logical terminal actions. Physical key events remain application-owned.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// Open and reset the prompt.
    Open,
    /// Close the prompt.
    Close,
    /// Commit one input character.
    TypeChar(char),
    /// Remove the final committed character.
    Backspace,
    /// Move to the next eligible candidate.
    SelectNext,
    /// Move to the previous eligible candidate.
    SelectPrev,
    /// Submit the active candidate.
    Confirm,
    /// Clear a non-empty query or cancel an empty prompt.
    Cancel,
}

/// Applies one logical action through the production shared prompt reducer.
pub fn handle_action(state: &mut FuzzyState, action: Action, source: &dyn FuzzySource) {
    match action {
        Action::Open => state.open_with(source),
        Action::Close => state.cancel(),
        Action::Cancel if state.query.is_empty() => state.cancel(),
        Action::Cancel => state.replace_query(String::new(), source),
        Action::TypeChar(character) => {
            let mut query = state.query.clone();
            query.push(character);
            state.replace_query(query, source);
        }
        Action::Backspace => {
            let mut query = state.query.clone();
            query.pop();
            state.replace_query(query, source);
        }
        Action::SelectNext => state.move_active(ratcore::prompt_completion::PromptDirection::Next, source),
        Action::SelectPrev => state.move_active(ratcore::prompt_completion::PromptDirection::Previous, source),
        Action::Confirm => state.confirm(source),
    }
}

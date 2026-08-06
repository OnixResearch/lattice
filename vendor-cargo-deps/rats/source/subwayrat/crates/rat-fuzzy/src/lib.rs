//! Incremental fuzzy finder widget for ratatui.
//!
//! Provides fuzzy matching with scoring, match highlighting, and a scrollable
//! result list. Data is supplied through the `FuzzySource` trait.

pub mod action;
mod prompt;
pub mod render;
pub mod score;
pub mod state;
pub mod types;

pub use action::Action;
pub use action::handle_action;
pub use render::FuzzyFinder;
pub use render::FuzzyStyle;
pub use score::CandidateIndex;
pub use score::FuzzyScoreInput;
pub use score::MatchOffset;
pub use score::ScoredMatch;
pub use score::fuzzy_score;
pub use state::FuzzyState;
pub use types::FuzzyCandidate;
pub use types::FuzzySource;

#[cfg(test)]
mod tests {
    use super::*;

    fn candidates() -> Vec<FuzzyCandidate> {
        vec![
            FuzzyCandidate {
                id: "open".to_string(),
                text: "Open file".to_string(),
                context: None,
                icon: None,
            },
            FuzzyCandidate {
                id: "settings".to_string(),
                text: "Settings".to_string(),
                context: None,
                icon: None,
            },
        ]
    }

    #[test]
    fn committed_text_and_candidate_movement_delegate_to_shared_prompt() {
        let source = candidates();
        let mut state = FuzzyState::new();
        handle_action(&mut state, Action::Open, &source);
        handle_action(&mut state, Action::TypeChar('s'), &source);
        handle_action(&mut state, Action::TypeChar('e'), &source);
        assert_eq!(state.query, "se");
        assert_eq!(state.result_count(), 1);
        assert_eq!(state.results[0].index.get(), 1);
        handle_action(&mut state, Action::SelectNext, &source);
        assert_eq!(state.selected, 0);
    }

    #[test]
    fn accepted_identity_maps_back_after_reducer_borrow_ends() {
        let source = candidates();
        let mut state = FuzzyState::new();
        handle_action(&mut state, Action::Open, &source);
        handle_action(&mut state, Action::SelectNext, &source);
        handle_action(&mut state, Action::Confirm, &source);
        assert!(!state.open);
        assert_eq!(state.take_result().map(|candidate| candidate.id), Some("settings".to_string()));
    }

    #[test]
    fn excessive_catalog_preserves_the_closed_adapter() {
        let candidate = FuzzyCandidate {
            id: "item".to_string(),
            text: "Item".to_string(),
            context: None,
            icon: None,
        };
        let source = vec![candidate; 257];
        let mut state = FuzzyState::new();
        handle_action(&mut state, Action::Open, &source);
        assert!(!state.open);
        assert!(state.results.is_empty());
    }

    #[test]
    fn adapter_retains_terminal_cancel_and_cursor_ownership() {
        let source = candidates();
        let mut state = FuzzyState::new();
        state.scroll_offset = 3;
        handle_action(&mut state, Action::Open, &source);
        handle_action(&mut state, Action::TypeChar('o'), &source);
        handle_action(&mut state, Action::Cancel, &source);
        assert!(state.open);
        assert!(state.query.is_empty());
        handle_action(&mut state, Action::Cancel, &source);
        assert!(!state.open);
        assert_eq!(state.scroll_offset, 0);
    }
}

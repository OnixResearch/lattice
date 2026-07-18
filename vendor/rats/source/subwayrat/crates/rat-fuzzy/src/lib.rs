//! Incremental fuzzy finder widget for ratatui.
//!
//! Provides fuzzy matching with scoring, match highlighting, and a scrollable
//! result list. Data is supplied through the `FuzzySource` trait.

pub mod action;
pub mod render;
pub mod score;
pub mod state;
pub mod types;

pub use action::{Action, handle_action};
pub use render::{FuzzyFinder, FuzzyStyle};
pub use score::{FuzzyScoreInput, ScoredMatch, fuzzy_score};
pub use state::FuzzyState;
pub use types::{FuzzyCandidate, FuzzySource};

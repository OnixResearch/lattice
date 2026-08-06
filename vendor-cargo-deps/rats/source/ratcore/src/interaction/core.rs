//! Renderer-neutral prompt sessions with supplied completion, history, and validation snapshots.
//!
//! Applications own mounting, providers, persistence, validation work, authorization, and effects.
//! Renderers own text controls, composition, focus, cursor geometry, physical events, and drawing.

#[path = "logic/acceptance.rs"]
mod acceptance;
#[path = "ports/binding.rs"]
mod action;
#[path = "data/checks.rs"]
mod checks;
#[path = "logic/matching.rs"]
mod matching;
#[path = "data/model.rs"]
mod model;
#[path = "logic/navigation.rs"]
mod navigation;
#[path = "logic/reducer.rs"]
mod reducer;
#[path = "data/state.rs"]
mod state;
#[path = "ports/view.rs"]
mod view;

pub use action::reduce_command_palette_through_prompt;
pub use checks::build_prompt_session;
pub use model::*;
pub use reducer::reduce_prompt;
pub use state::*;
pub use view::*;

#[cfg(test)]
#[path = "tests/session.rs"]
mod tests;

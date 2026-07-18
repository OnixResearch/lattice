//! Subwayrat web-renderer evidence and input adapter core.
//!
//! The actual browser shell owns Ratzilla, Trunk assets, DOM registration, and
//! render loops. This pure module records backend-neutral build evidence,
//! describes the Ratzilla shell contract, and normalizes web input into shared
//! action outcomes without depending on crossterm or browser APIs.

mod events;
mod proof;
mod view;

pub use events::*;
pub use proof::*;
pub use view::*;

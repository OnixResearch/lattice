//! Renderer-neutral pointer hit testing and route selection.
//!
//! The pointer routing core consumes already-normalized pointer events and
//! immutable hit maps. DOM, terminal, and backend event lifetimes stay in
//! renderer adapters; this module returns deterministic route outcomes and
//! diagnostics that tests can exercise without a browser or terminal.

#[path = "routing/area.rs"]
mod area;
#[path = "routing/bridge.rs"]
mod bridge;
#[path = "routing/support/diagnose.rs"]
mod diagnose;
#[path = "routing/input.rs"]
mod input;
#[path = "routing/support/intent.rs"]
mod intent;
#[path = "routing/result.rs"]
mod result;
#[path = "routing/select.rs"]
mod select;
#[path = "routing/shape.rs"]
mod shape;

pub use area::*;
pub use bridge::*;
pub use input::*;
pub use result::*;
pub use select::*;
pub use shape::*;

#[cfg(test)]
#[path = "routing/tests.rs"]
mod tests;

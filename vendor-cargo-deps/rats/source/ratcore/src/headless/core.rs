//! Executable headless-component checklist, inventory, and bounded tabs pilot.

mod adoption;
mod checklist;
#[path = "selection/core.rs"]
mod selection;

pub use adoption::*;
pub use checklist::*;
pub use selection::*;

#[cfg(test)]
mod tests;

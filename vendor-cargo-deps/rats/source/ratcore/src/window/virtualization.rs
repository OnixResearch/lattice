//! Renderer-neutral variable-extent collection virtualization.
//!
//! Domain owners supply stable ordered identities. Renderer adapters supply
//! checked numeric measurements and apply returned scroll metadata. This core
//! never owns item content, renderer children, measurement APIs, or scrolling.

mod model;
mod plan;
mod receipt;
mod selection;
mod snapshot;

pub use model::*;
pub use plan::*;
pub use receipt::*;
pub use selection::*;

#[cfg(test)]
#[path = "virtualization/tests.rs"]
mod tests;

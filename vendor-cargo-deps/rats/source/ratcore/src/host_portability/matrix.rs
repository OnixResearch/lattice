//! Exact-target package profiles, bounded receipts, and fail-closed matrix aggregation.

#[path = "matrix/aggregate.rs"]
mod aggregate;
#[path = "matrix/model.rs"]
mod model;
#[path = "matrix/profiles.rs"]
mod profiles;
#[path = "matrix/receipts.rs"]
mod receipts;
#[path = "matrix/render.rs"]
mod render;

pub use aggregate::*;
pub use model::*;
pub use profiles::*;
pub use receipts::*;
pub use render::*;

#[cfg(test)]
#[path = "matrix/tests.rs"]
mod tests;

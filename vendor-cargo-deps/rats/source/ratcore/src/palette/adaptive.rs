//! Bounded adaptive theme preferences, schema, mappings, and previews.

#[path = "adaptive/artifacts.rs"]
mod artifacts;
#[path = "adaptive/contract.rs"]
mod contract;
#[path = "adaptive/exports.rs"]
mod exports;
#[path = "adaptive/mapping_validation.rs"]
mod mapping_validation;
#[path = "adaptive/model.rs"]
mod model;
#[path = "adaptive/preference.rs"]
mod preference;
#[path = "adaptive/preview.rs"]
mod preview;
#[path = "adaptive/resolution.rs"]
mod resolution;
#[path = "adaptive/validation.rs"]
mod validation;

pub use artifacts::*;
pub use contract::*;
pub use exports::*;
pub use model::*;
pub use preference::*;
pub use preview::*;
pub use resolution::*;
pub use validation::*;

#[cfg(test)]
#[path = "adaptive/tests.rs"]
mod tests;

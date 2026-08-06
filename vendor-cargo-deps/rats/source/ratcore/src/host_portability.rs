//! Pure host-portability taxonomy, ownership inventory, and evidence policy.

#[path = "host_portability/claims.rs"]
mod claims;
#[path = "host_portability/inventory.rs"]
mod inventory;
#[path = "host_portability/matrix.rs"]
mod matrix;
#[path = "host_portability/model.rs"]
mod model;
#[path = "host_portability/render.rs"]
mod render;
#[path = "host_portability/validation.rs"]
mod validation;

pub use claims::*;
pub use inventory::*;
pub use matrix::*;
pub use model::*;
pub use render::*;
pub use validation::*;

#[cfg(test)]
#[path = "host_portability/tests.rs"]
mod tests;

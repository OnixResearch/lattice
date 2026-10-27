//! Public compatibility aliases for adapter boundary DTO names.

mod blend;
mod facade;
mod surface;
mod wire;

pub use crate::mcpapp::{MCP_APP_EVALUATION_AXES, PROHIBITED_MCP_APP_VALIDATION_EFFECTS};
pub use crate::profile::mutation_requires_runtime_guard;
pub use crate::security::SECURITY_REVIEW_HASH_HEX_CHARS;
pub use blend::*;
pub use facade::*;
pub use surface::*;
pub use wire::*;

//! Typed local workflow-template bundles with a pure validation core and thin shells.

pub mod core;
pub mod init;
pub mod model;
pub mod shell;

pub use core::*;

pub use init::*;
pub use model::*;
pub use shell::*;

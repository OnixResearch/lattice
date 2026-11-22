//! Typed local workflow-template bundles with a pure validation core and thin shells.

pub mod classification;
pub mod core;
pub mod init;
pub mod model;
pub mod params;
pub mod shell;
#[cfg(test)]
mod tests;

pub use core::*;

pub use classification::BundleMode;
pub use init::*;
pub use model::*;
pub use params::ParamDeclaration;
pub use params::ParamError;
pub use params::ParamTarget;
pub use params::ParamType;
pub use params::ParamValue;
pub use shell::*;

#![allow(
    unknown_lints,
    non_trait_imports,
    reason = "child modules need the parent error type without creating duplicate public error owners"
)]

pub mod catalog;
pub mod identity;
pub mod model;
pub mod oracles;
pub mod planning;

pub(super) use super::Error;

//! Pure, bounded pane-to-workflow composition. This crate does not observe UI,
//! read a clock, execute a workflow, or publish a receipt.
#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

mod identity;
mod model;
mod planning;

pub use model::*;
pub use planning::{admit, plan, validate_receipt, validate_request};

#[cfg(test)]
mod tests;

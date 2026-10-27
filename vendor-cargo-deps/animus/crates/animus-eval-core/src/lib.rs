//! Pure native evaluation contracts for Animus.
//!
//! This crate validates already-loaded values and produces deterministic plans,
//! reductions, resume decisions, and metadata-only receipt drafts. It has no
//! filesystem, process, network, clock, provider, tool, scheduler, credential,
//! persistence, Harbor, Verifiers, Python, container, or Nickel authority.

#![no_std]
#![deny(missing_docs)]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::undocumented_unsafe_blocks)]
#![allow(clippy::module_name_repetitions)]
#![forbid(unsafe_code)]

extern crate alloc;

mod compat;
mod episode;
mod identity;
mod policy;
mod receipt;
mod recipe;
mod resume;
mod reward;
mod task;

pub use compat::*;
pub use episode::*;
pub use identity::*;
pub use policy::*;
pub use receipt::*;
pub use recipe::*;
pub use resume::*;
pub use reward::*;
pub use task::*;

#[cfg(test)]
mod tests;

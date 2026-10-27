//! Pure, redaction-first session branch exploration contracts.
//!
//! The caller supplies one validated ledger snapshot. This module has no
//! storage, clock, provider, process, terminal, transport, or mutation access.

#[allow(unused_imports)]
use crate::prelude::*;

mod core;
mod data;
mod receipt;

pub use core::{
    compare_branches, generation_identity, plan_branch_operation, project_branch_snapshot,
    validate_branch_explorer_policy, validate_sibling_equivalence,
};
pub use data::*;
pub use receipt::project_branch_explorer_receipt;

#[cfg(test)]
mod tests;

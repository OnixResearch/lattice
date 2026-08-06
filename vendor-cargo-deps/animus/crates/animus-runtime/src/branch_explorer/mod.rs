//! Pure, redaction-first session branch exploration contracts.
//!
//! The caller supplies one validated ledger snapshot. This module has no
//! storage, clock, provider, process, terminal, transport, or mutation access.

mod core;
mod data;
mod receipt;

pub use core::compare_branches;
pub use core::generation_identity;
pub use core::plan_branch_operation;
pub use core::project_branch_snapshot;
pub use core::validate_branch_explorer_policy;
pub use core::validate_sibling_equivalence;

pub use data::*;
pub use receipt::project_branch_explorer_receipt;

#[cfg(test)]
mod tests;

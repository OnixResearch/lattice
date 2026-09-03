//! Bounded advisory memoization for immutable DAG node results.
//!
//! The core owns deterministic identity and retention decisions. The store owns
//! synchronization, bounded storage, executor calls, and deferred release.

// r[impl lattice.node_result_cache.verification]
pub mod core;
mod store;

pub use core::Config;
pub use core::ConfigError;
pub use core::Key;
pub use core::ProjectionError;
pub use core::RequestProjection;
pub use core::project_key;

pub use store::Error;
pub use store::InsertReport;
pub use store::Inserted;
pub use store::LookupOrExecuteError;
pub use store::Memoizer;

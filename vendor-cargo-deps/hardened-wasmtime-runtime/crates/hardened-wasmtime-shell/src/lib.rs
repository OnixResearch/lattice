//! Product-neutral Wasmtime engine, exact linking and bounded guest exchange.
//! Consumers own artifact trust, world/schema meaning, current authority and epoch driving.
//! No WASI, ambient timer service, product effect ABI or fallback is installed.

mod budget;
mod carrier;
mod concurrency;
pub mod engine;
pub mod invoke;
mod milestone;
pub mod model;
mod preparation;
mod sliced;
mod store;
mod transfer;

pub use carrier::ByteMethodInvocation;
pub use engine::{BoundedRuntime, BuildFailure, EpochService};
pub use invoke::{BoundedInvocation, GuestPointerError, GuestSpan, check_guest_range};
pub use milestone::InstantiationCheckpoint;
pub use model::ComponentInvocation;
pub use preparation::{Artifact, PrepareError, PrepareOptions, qualified_name};
pub use sliced::{ResumableInvocation, SliceBoundary, SliceError, SliceOptions, SliceStep};
pub use store::{HostDenied, InvocationOutcome, State};

/// Exact shell contract; v4 separates instantiation and independent memory ceilings.
pub const SHELL_SCHEMA_ID: &str = "hwr.shell.v4";
/// Wasmtime cohort used by this shell.
pub const WASMTIME_VERSION: &str = "45.0.0";

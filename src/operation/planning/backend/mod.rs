//! Lattice-owned execution-backend requirements, frozen Animus projection, and durable attempts.

mod attempt;
mod model;
mod route;

pub use attempt::admit_dispatch;
pub use attempt::admit_retry_route;
pub use model::ATTEMPT_SCHEMA;
pub use model::BackendAttemptRecord;
pub use model::BackendAttemptState;
pub use model::BackendClass;
pub use model::BackendRoutePlan;
pub use model::ExecutionFormat;
pub use model::ExecutionRequirements;
pub use model::FilesystemSemantics;
pub use model::IsolationClaim;
pub use model::NetworkPosture;
pub use model::PLANNER_COHORT;
pub use model::PLANNER_SCHEMA;
pub use model::PeripheralNeed;
pub use model::PersistenceNeed;
pub use model::ProcessModel;
pub use model::ROUTE_SCHEMA;
pub use model::ResourceRequirements;
pub use model::SafeBackendView;
pub use model::TerminalNeed;
pub use route::RouteContext;
pub use route::compatibility_projection;
pub use route::plan_route;
pub use route::validate_route;

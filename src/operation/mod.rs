//! Typed, identity-bound operation contracts and runtime boundaries.

// r[impl lattice.operations.catalog]
// r[impl lattice.operations.instances]
// r[impl lattice.operations.preflight]
// r[impl lattice.operations.direct_run]
// r[impl lattice.operations.promotion]
// r[impl lattice.operations.pilot]

pub mod catalog;
pub mod cli;
pub mod format;
pub mod identity;
pub mod instance;
pub mod model;
pub mod planning;
pub mod promotion;
pub mod readiness;
#[path = "run/mod.rs"]
pub mod run_model;
pub mod runtime;
pub mod schema;
pub mod shell;

pub use model::Bounds;
pub use model::CapabilityClass;
pub use model::CapabilityPolicy;
pub use model::Catalog;
pub use model::Diagnostic;
pub use model::DiagnosticCode;
pub use model::InstanceFile;
pub use model::InvocationPlan;
pub use model::OperationInstance;
pub use model::OperationType;
pub use model::PlanRequest;
pub use model::PreflightFacts;
pub use model::PromotionPlan;
pub use model::PromotionResult;
pub use model::RuntimeFact;
pub use model::ValidatedCatalog;
pub use model::ValidatedInstance;
pub use run_model::DirectRunResult;
pub use run_model::RunRecord;
pub use run_model::RunState;

#[derive(Debug)]
pub enum Error {
    Diagnostics(Vec<Diagnostic>),
    Nickel(String),
    FileAccess,
    Invalid(String),
    Io(String),
    Store(String),
    Runtime(String),
    Encoding,
    Publication,
}

impl std::fmt::Display for Error {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Diagnostics(diagnostics) => {
                write!(formatter, "operation admission failed with {} diagnostics", diagnostics.len())
            }
            Self::Nickel(message) => write!(formatter, "operation Nickel evaluation failed: {message}"),
            Self::FileAccess => write!(formatter, "operation file access failed"),
            Self::Invalid(message) => write!(formatter, "operation input is invalid: {message}"),
            Self::Io(message) => write!(formatter, "operation I/O failed: {message}"),
            Self::Store(message) => write!(formatter, "operation store failed: {message}"),
            Self::Runtime(message) => write!(formatter, "operation runtime failed: {message}"),
            Self::Encoding => write!(formatter, "operation value encoding failed"),
            Self::Publication => write!(formatter, "operation publication failed"),
        }
    }
}

impl std::error::Error for Error {}

pub(crate) fn ordered(mut diagnostics: Vec<Diagnostic>) -> Vec<Diagnostic> {
    diagnostics.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then_with(|| left.code.cmp(&right.code))
            .then_with(|| left.message.cmp(&right.message))
    });
    diagnostics
}

pub(crate) fn diagnostic(code: DiagnosticCode, path: impl Into<String>, message: impl Into<String>) -> Diagnostic {
    Diagnostic {
        code,
        path: path.into(),
        message: message.into(),
    }
}

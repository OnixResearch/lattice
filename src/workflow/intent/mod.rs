//! Inert, bounded normalization of host-supplied Cairn intent.
//!
//! This is a Lattice input contract, not a Cairn producer receipt or execution
//! grant. Choregraph owns graph admission; Cairn and Kiln retain acceptance.

// r[impl lattice.workflow_intent.boundary]
pub mod cli;
mod comparison;
mod compiler;
mod model;
mod validation;

pub use comparison::compare_proposal;
pub use compiler::compile;
pub use model::*;

pub const INTENT_SCHEMA: &str = "lattice.workflow-intent.v1";
pub const PLAN_SCHEMA: &str = "lattice.workflow-intent-plan.v1";
pub const COMPILER_REVISION: &str = "lattice.workflow-intent-compiler.v1";
pub const MAX_OBLIGATIONS: usize = 128;
pub const MAX_DEPENDENCIES: usize = 512;
pub const MAX_CONTRACTS_PER_OBLIGATION: usize = 16;
pub const MAX_ALTERNATIVES: usize = 16;
pub const MAX_IDENTIFIER_BYTES: usize = 256;
pub const MAX_INPUT_BYTES: usize = 1_048_576;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("workflow intent rejected: {code:?} at {subject}")]
pub struct Diagnostic {
    pub code: DiagnosticCode,
    pub subject: String,
}

impl Diagnostic {
    fn new(code: DiagnosticCode, subject: &str) -> Self {
        Self {
            code,
            subject: subject.to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticCode {
    UnsupportedSchema,
    InvalidIdentity,
    IncompleteContext,
    BindingMismatch,
    LimitExceeded,
    DuplicateIdentity,
    MissingContract,
    UnsupportedOperation,
    MissingCapability,
    AmbiguousAlternative,
    InvalidDependency,
    GraphAdmission,
    Encoding,
}

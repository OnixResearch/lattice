//! Content-addressed operation packs with a pure admission core and bounded shell.
//!
//! r[impl lattice.operation_packs.manifest]
//! r[impl lattice.operation_packs.identity]
//! r[impl lattice.operation_packs.admission]
//! r[impl lattice.operation_packs.runtime]
//! r[impl lattice.operation_packs.analyzers]
//! r[impl lattice.operation_packs.skills]
//! r[impl lattice.operation_packs.registry]
//! r[impl lattice.operation_packs.doctor]
//! r[impl lattice.operation_packs.verification]

pub mod admission;
pub mod analyzer;
pub mod cli;
pub mod doctor;
pub mod identity;
pub mod model;
pub mod registry;
pub mod shell;
pub mod skill;
pub mod validation;

pub use admission::admit;
pub use admission::create_lock;
pub use admission::validate_lock;
pub use identity::content_identity;
pub use identity::manifest_identity;
pub use identity::pack_identity;
pub use model::*;
pub use validation::validate_manifest;

#[derive(Debug)]
pub enum Error {
    Invalid(&'static str),
    Diagnostic(String),
    Nickel(String),
    Io(&'static str),
    Conflict(&'static str),
    Operation(crate::operation::Error),
    OperationData(crate::operation_data::shell::Error),
    Process(crate::agent_node::ProcessError),
    Encoding,
}

impl std::fmt::Display for Error {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid(message) => write!(formatter, "operation pack is invalid: {message}"),
            Self::Diagnostic(message) => write!(formatter, "operation pack admission failed: {message}"),
            Self::Nickel(message) => write!(formatter, "operation pack Nickel evaluation failed: {message}"),
            Self::Io(message) => write!(formatter, "operation pack I/O failed: {message}"),
            Self::Conflict(message) => write!(formatter, "operation pack conflict: {message}"),
            Self::Operation(error) => write!(formatter, "operation pack runtime failed: {error}"),
            Self::OperationData(error) => write!(formatter, "operation pack data publication failed: {error}"),
            Self::Process(error) => write!(formatter, "operation pack bounded process failed: {error}"),
            Self::Encoding => write!(formatter, "operation pack encoding failed"),
        }
    }
}

impl std::error::Error for Error {}

impl From<crate::operation::Error> for Error {
    fn from(error: crate::operation::Error) -> Self {
        Self::Operation(error)
    }
}

impl From<crate::operation_data::shell::Error> for Error {
    fn from(error: crate::operation_data::shell::Error) -> Self {
        Self::OperationData(error)
    }
}

impl From<crate::agent_node::ProcessError> for Error {
    fn from(error: crate::agent_node::ProcessError) -> Self {
        Self::Process(error)
    }
}

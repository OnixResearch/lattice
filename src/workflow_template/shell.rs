const MANIFEST_FILE_NAME: &str = "manifest.ncl";

#[derive(Debug)]
pub enum Error {
    RootUnavailable,
    Store(crate::store::StoreError),
    ApplicationConflict,
    ActivationBlocked(super::model::ActivationBlock),
    Orchestration(crate::orchestration::OrchestrationShellError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RootUnavailable => formatter.write_str("workflow-template root is unavailable"),
            Self::Store(error) => write!(formatter, "workflow-template store operation failed: {error}"),
            Self::ApplicationConflict => formatter.write_str("workflow-template content conflicts with stored content"),
            Self::ActivationBlocked(block) => write!(formatter, "workflow-template activation blocked: {block:?}"),
            Self::Orchestration(error) => write!(formatter, "workflow-template orchestration failed: {error}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<crate::store::StoreError> for Error {
    fn from(value: crate::store::StoreError) -> Self {
        Self::Store(value)
    }
}

impl From<crate::orchestration::OrchestrationShellError> for Error {
    fn from(value: crate::orchestration::OrchestrationShellError) -> Self {
        Self::Orchestration(value)
    }
}

include!("shell/loading.rs");
include!("shell/capture.rs");
include!("shell/activation.rs");

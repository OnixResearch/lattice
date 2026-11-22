const MANIFEST_FILE_NAME: &str = "manifest.ncl";

#[derive(Debug)]
pub enum Error {
    RootUnavailable,
    Store(crate::store::StoreError),
    ApplicationConflict,
    ParamsUnbound,
    BoundContentDrift,
    ModeViolation,
    ActivationBlocked(super::model::ActivationBlock),
    Orchestration(crate::orchestration::Error),
    Inputs(super::inputs::InputError),
    InputFile,
}

impl std::fmt::Display for Error {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RootUnavailable => formatter.write_str("workflow-template root is unavailable"),
            Self::Store(error) => write!(formatter, "workflow-template store operation failed: {error}"),
            Self::ApplicationConflict => formatter.write_str("workflow-template content conflicts with stored content"),
            Self::ParamsUnbound => formatter.write_str("workflow-template parameters are not bound"),
            Self::BoundContentDrift => formatter.write_str("workflow-template bound content no longer verifies"),
            Self::ModeViolation => formatter.write_str("workflow-template mode does not admit its workflow"),
            Self::ActivationBlocked(block) => write!(formatter, "workflow-template activation blocked: {block:?}"),
            Self::Orchestration(error) => write!(formatter, "workflow-template orchestration failed: {error}"),
            Self::Inputs(error) => write!(formatter, "workflow inputs refused: {error}"),
            Self::InputFile => write!(
                formatter,
                "--input-file must be a readable regular file of at most {} bytes",
                super::inputs::MAX_INPUT_FILE_BYTES
            ),
        }
    }
}

impl std::error::Error for Error {}

impl From<crate::store::StoreError> for Error {
    fn from(value: crate::store::StoreError) -> Self {
        Self::Store(value)
    }
}

impl From<crate::orchestration::Error> for Error {
    fn from(value: crate::orchestration::Error) -> Self {
        Self::Orchestration(value)
    }
}

include!("shell/loading.rs");
include!("shell/capture.rs");
include!("shell/activation.rs");
include!("shell/inputs.rs");

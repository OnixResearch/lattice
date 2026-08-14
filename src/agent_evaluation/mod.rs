mod core;
mod runtime;

pub use core::catalog;
pub use core::identity;
pub use core::model;
pub use core::oracles;
pub use core::planning;

pub use runtime::artifacts;
pub use runtime::cli;
pub use runtime::evidence;
pub use runtime::format;
pub use runtime::host;
pub use runtime::live;
pub use runtime::persistence;

pub mod shell {
    pub use super::runtime::runner::*;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("evaluation identity is invalid")]
    InvalidIdentity,
    #[error("evaluation data could not be encoded")]
    Encoding,
    #[error("evaluation catalog is invalid")]
    InvalidCatalog,
    #[error("evaluation case is invalid")]
    InvalidCase,
    #[error("evaluation catalog contains duplicate authority")]
    Duplicate,
    #[error("evaluation asset parity check failed")]
    AssetParity,
    #[error("evaluation budget is invalid")]
    InvalidBudget,
    #[error("evaluation subject is invalid")]
    InvalidSubject,
    #[error("evaluation subject identity changed")]
    SubjectDrift,
    #[error("evaluation environment is invalid")]
    InvalidEnvironment,
    #[error("evaluation oracle is invalid")]
    InvalidOracle,
    #[error("evaluation oracle evidence is invalid")]
    InvalidOracleEvidence,
    #[error("evaluation judge evidence is invalid")]
    InvalidJudgeEvidence,
    #[error("evaluation artifact is invalid")]
    InvalidArtifact,
    #[error("evaluation live posture is invalid")]
    InvalidLivePosture,
    #[error("evaluation non-claim boundary is invalid")]
    NonClaim,
    #[error("evaluation case selection is invalid")]
    Selection,
    #[error("required evaluation service is unavailable")]
    ServiceUnavailable,
    #[error("evaluation live authority was denied")]
    AuthorityDenied,
    #[error("evaluation outcome is unknown")]
    LiveOutcomeUnknown,
    #[error("evaluation receipt is invalid")]
    InvalidReceipt,
    #[error("Animus evaluation fact is invalid")]
    InvalidAnimusFact,
    #[error("evaluation file access failed")]
    FileAccess,
    #[error("evaluation Nickel source failed")]
    NickelEvaluation,
    #[error("evaluation case changed after planning")]
    CaseDrift,
    #[error("evaluation trial is already complete")]
    AlreadyComplete,
    #[error("evaluation trial cancellation was requested")]
    CancellationRequested,
    #[error("evaluation recovery is blocked")]
    RecoveryBlocked,
    #[error("evaluation service failed")]
    ServiceFailure,
}

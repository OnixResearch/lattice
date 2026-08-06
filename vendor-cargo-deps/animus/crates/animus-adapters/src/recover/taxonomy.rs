const BACKEND_JJ: &str = "jj";
const BACKEND_GIT: &str = "git";
const BACKEND_MEMORY: &str = "in_memory";
const BACKEND_WORKSPACE_ISOLATION: &str = "workspace_isolation";
const BACKEND_UNSUPPORTED: &str = "unsupported";
const OPERATION_CREATE: &str = "create";
const OPERATION_LIST: &str = "list";
const OPERATION_INSPECT: &str = "inspect";
const OPERATION_ROLLBACK: &str = "rollback";
const OPERATION_CLEANUP: &str = "cleanup";
const STATUS_CREATED: &str = "created";
const STATUS_LISTED: &str = "listed";
const STATUS_INSPECTED: &str = "inspected";
const STATUS_ROLLED_BACK: &str = "rolled_back";
const STATUS_BLOCKED: &str = "blocked";
const STATUS_FAILED: &str = "failed";
const STATUS_UNSUPPORTED: &str = "unsupported";
const FAILURE_NONE: &str = "none";
const FAILURE_MISSING_SERVICE: &str = "missing_service";
const FAILURE_UNAVAILABLE: &str = "unavailable";
const FAILURE_DENIED: &str = "denied";
const FAILURE_UNSUPPORTED_OPERATION: &str = "unsupported_operation";
const FAILURE_UNCONFIRMED_ROLLBACK: &str = "unconfirmed_rollback";
const FAILURE_STALE_WORKSPACE: &str = "stale_workspace";
const FAILURE_INVALID_METADATA: &str = "invalid_metadata";
const FAILURE_RECEIPT_LEAK: &str = "receipt_leak";

/// Backend family that created or can restore a checkpoint.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CheckpointBackendKind {
    /// Jujutsu-style backend.
    Jj,
    /// Git-style backend.
    Git,
    /// In-memory fixture or ephemeral backend.
    InMemory,
    /// Workspace-isolation backend.
    WorkspaceIsolation,
    /// Unsupported backend placeholder.
    Unsupported,
}
impl CheckpointBackendKind {
    /// Parses a backend-kind label.
    ///
    /// # Errors
    /// Returns [`super::CheckpointError`] when the value is empty, malformed, or unknown.
    pub fn parse(value: &str) -> Result<Self, super::CheckpointError> {
        super::core::parse_enum(super::CHECKPOINT_BACKEND_FIELD, value, |normalized| match normalized {
            BACKEND_JJ => Some(Self::Jj),
            BACKEND_GIT => Some(Self::Git),
            BACKEND_MEMORY => Some(Self::InMemory),
            BACKEND_WORKSPACE_ISOLATION => Some(Self::WorkspaceIsolation),
            BACKEND_UNSUPPORTED => Some(Self::Unsupported),
            _ => None,
        })
    }

    /// Returns the stable backend-kind label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Jj => BACKEND_JJ,
            Self::Git => BACKEND_GIT,
            Self::InMemory => BACKEND_MEMORY,
            Self::WorkspaceIsolation => BACKEND_WORKSPACE_ISOLATION,
            Self::Unsupported => BACKEND_UNSUPPORTED,
        }
    }
}

/// Checkpoint operation class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CheckpointOperation {
    /// Create a checkpoint.
    Create,
    /// List known checkpoints.
    List,
    /// Inspect one checkpoint.
    Inspect,
    /// Restore a checkpoint.
    Rollback,
    /// Remove or prune checkpoint state.
    Cleanup,
}
impl CheckpointOperation {
    /// Parses an operation label.
    ///
    /// # Errors
    /// Returns [`super::CheckpointError`] when the value is empty, malformed, or unknown.
    pub fn parse(value: &str) -> Result<Self, super::CheckpointError> {
        super::core::parse_enum(super::CHECKPOINT_OPERATION_FIELD, value, |normalized| match normalized {
            OPERATION_CREATE => Some(Self::Create),
            OPERATION_LIST => Some(Self::List),
            OPERATION_INSPECT => Some(Self::Inspect),
            OPERATION_ROLLBACK => Some(Self::Rollback),
            OPERATION_CLEANUP => Some(Self::Cleanup),
            _ => None,
        })
    }

    /// Returns the stable operation label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Create => OPERATION_CREATE,
            Self::List => OPERATION_LIST,
            Self::Inspect => OPERATION_INSPECT,
            Self::Rollback => OPERATION_ROLLBACK,
            Self::Cleanup => OPERATION_CLEANUP,
        }
    }
}

/// Safe checkpoint lifecycle status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CheckpointStatus {
    /// Checkpoint was created.
    Created,
    /// Checkpoint list was returned.
    Listed,
    /// Checkpoint was inspected.
    Inspected,
    /// Rollback completed.
    RolledBack,
    /// Operation was blocked before mutation.
    Blocked,
    /// Operation failed.
    Failed,
    /// Operation or backend is unsupported.
    Unsupported,
}
impl CheckpointStatus {
    /// Parses a status label.
    ///
    /// # Errors
    /// Returns [`super::CheckpointError`] when the value is empty, malformed, or unknown.
    pub fn parse(value: &str) -> Result<Self, super::CheckpointError> {
        super::core::parse_enum(super::CHECKPOINT_STATUS_FIELD, value, |normalized| match normalized {
            STATUS_CREATED => Some(Self::Created),
            STATUS_LISTED => Some(Self::Listed),
            STATUS_INSPECTED => Some(Self::Inspected),
            STATUS_ROLLED_BACK => Some(Self::RolledBack),
            STATUS_BLOCKED => Some(Self::Blocked),
            STATUS_FAILED => Some(Self::Failed),
            STATUS_UNSUPPORTED => Some(Self::Unsupported),
            _ => None,
        })
    }

    /// Returns the stable status label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Created => STATUS_CREATED,
            Self::Listed => STATUS_LISTED,
            Self::Inspected => STATUS_INSPECTED,
            Self::RolledBack => STATUS_ROLLED_BACK,
            Self::Blocked => STATUS_BLOCKED,
            Self::Failed => STATUS_FAILED,
            Self::Unsupported => STATUS_UNSUPPORTED,
        }
    }
}

/// Fail-closed checkpoint failure class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CheckpointFailureClass {
    /// No failure occurred.
    None,
    /// Required checkpoint service was not injected.
    MissingService,
    /// Checkpoint service is currently unavailable.
    Unavailable,
    /// Host policy denied the operation.
    Denied,
    /// Backend or service does not support the operation.
    UnsupportedOperation,
    /// Rollback lacks explicit confirmation.
    UnconfirmedRollback,
    /// Current workspace identity no longer matches the rollback precondition.
    StaleWorkspace,
    /// Metadata failed validation.
    InvalidMetadata,
    /// Receipt text contains unsafe material.
    ReceiptLeak,
}
impl CheckpointFailureClass {
    /// Parses a failure-class label.
    ///
    /// # Errors
    /// Returns [`super::CheckpointError`] when the value is empty, malformed, or unknown.
    pub fn parse(value: &str) -> Result<Self, super::CheckpointError> {
        super::core::parse_enum(super::CHECKPOINT_FAILURE_FIELD, value, |normalized| match normalized {
            FAILURE_NONE => Some(Self::None),
            FAILURE_MISSING_SERVICE => Some(Self::MissingService),
            FAILURE_UNAVAILABLE => Some(Self::Unavailable),
            FAILURE_DENIED => Some(Self::Denied),
            FAILURE_UNSUPPORTED_OPERATION => Some(Self::UnsupportedOperation),
            FAILURE_UNCONFIRMED_ROLLBACK => Some(Self::UnconfirmedRollback),
            FAILURE_STALE_WORKSPACE => Some(Self::StaleWorkspace),
            FAILURE_INVALID_METADATA => Some(Self::InvalidMetadata),
            FAILURE_RECEIPT_LEAK => Some(Self::ReceiptLeak),
            _ => None,
        })
    }

    /// Returns the stable failure label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => FAILURE_NONE,
            Self::MissingService => FAILURE_MISSING_SERVICE,
            Self::Unavailable => FAILURE_UNAVAILABLE,
            Self::Denied => FAILURE_DENIED,
            Self::UnsupportedOperation => FAILURE_UNSUPPORTED_OPERATION,
            Self::UnconfirmedRollback => FAILURE_UNCONFIRMED_ROLLBACK,
            Self::StaleWorkspace => FAILURE_STALE_WORKSPACE,
            Self::InvalidMetadata => FAILURE_INVALID_METADATA,
            Self::ReceiptLeak => FAILURE_RECEIPT_LEAK,
        }
    }
}

const FILE_INCLUSION_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("workspace_file_inclusion");
const WORKSPACE_OPERATION_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("workspace_operation");
const COMMIT_TARGET_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("workspace_commit_target");
const ABORT_POLICY_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("workspace_abort_policy");
const CONFLICT_POLICY_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("workspace_conflict_policy");
const CLEANUP_POLICY_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("workspace_cleanup_policy");

const FILE_INCLUSION_TRACKED_ONLY: &str = "tracked_only";
const FILE_INCLUSION_INCLUDE_IGNORED: &str = "include_ignored";
const FILE_INCLUSION_INCLUDE_LOCAL: &str = "include_local";
const OPERATION_CREATE: &str = "create";
const OPERATION_COMMIT: &str = "commit";
const OPERATION_ABORT: &str = "abort";
const COMMIT_TARGET_PARENT: &str = "parent";
const COMMIT_TARGET_SOURCE: &str = "source";
const ABORT_POLICY_DISCARD: &str = "discard";
const ABORT_POLICY_PRESERVE_ON_FAILURE: &str = "preserve_on_failure";
const CONFLICT_POLICY_FAIL: &str = "fail";
const CONFLICT_POLICY_ALLOW_MANUAL: &str = "allow_manual";
const CLEANUP_POLICY_REQUIRED: &str = "required";
const CLEANUP_POLICY_BEST_EFFORT: &str = "best_effort";

/// Policy for ignored or local files in speculative workspaces.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FileInclusionPolicy {
    /// Include only tracked source files.
    TrackedOnly,
    /// Include ignored files when the host adapter can do so safely.
    IncludeIgnored,
    /// Include tracked and local untracked files when explicitly requested.
    IncludeLocal,
}

impl FileInclusionPolicy {
    /// Parses a workspace file-inclusion policy.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] for empty, malformed, or unknown values.
    pub fn parse(value: &str) -> Result<Self, crate::AdapterContractError> {
        super::text::parse_value(FILE_INCLUSION_FIELD, value, |normalized| match normalized {
            FILE_INCLUSION_TRACKED_ONLY => Some(Self::TrackedOnly),
            FILE_INCLUSION_INCLUDE_IGNORED => Some(Self::IncludeIgnored),
            FILE_INCLUSION_INCLUDE_LOCAL => Some(Self::IncludeLocal),
            _ => None,
        })
    }

    /// Returns the stable policy label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::TrackedOnly => FILE_INCLUSION_TRACKED_ONLY,
            Self::IncludeIgnored => FILE_INCLUSION_INCLUDE_IGNORED,
            Self::IncludeLocal => FILE_INCLUSION_INCLUDE_LOCAL,
        }
    }
}

/// Requested workspace operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Operation {
    /// Create or prepare a speculative workspace.
    Create,
    /// Commit a speculative workspace.
    Commit,
    /// Abort a speculative workspace.
    Abort,
}

impl Operation {
    /// Parses a workspace operation.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] for empty, malformed, or unknown values.
    pub fn parse(value: &str) -> Result<Self, crate::AdapterContractError> {
        super::text::parse_value(
            WORKSPACE_OPERATION_FIELD,
            value,
            |normalized| match normalized {
                OPERATION_CREATE => Some(Self::Create),
                OPERATION_COMMIT => Some(Self::Commit),
                OPERATION_ABORT => Some(Self::Abort),
                _ => None,
            },
        )
    }

    /// Returns the stable operation label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Create => OPERATION_CREATE,
            Self::Commit => OPERATION_COMMIT,
            Self::Abort => OPERATION_ABORT,
        }
    }
}

/// Target for a successful workspace commit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommitTarget {
    /// Promote changes to the immediate parent workspace.
    Parent,
    /// Promote changes to the source identity when the backend supports it.
    Source,
}

impl CommitTarget {
    /// Parses a workspace commit target.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] for empty, malformed, or unknown values.
    pub fn parse(value: &str) -> Result<Self, crate::AdapterContractError> {
        super::text::parse_value(COMMIT_TARGET_FIELD, value, |normalized| match normalized {
            COMMIT_TARGET_PARENT => Some(Self::Parent),
            COMMIT_TARGET_SOURCE => Some(Self::Source),
            _ => None,
        })
    }

    /// Returns the stable commit-target label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Parent => COMMIT_TARGET_PARENT,
            Self::Source => COMMIT_TARGET_SOURCE,
        }
    }
}

/// Abort behavior requested for a workspace.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AbortPolicy {
    /// Discard the selected workspace if abort succeeds.
    Discard,
    /// Preserve the workspace when abort cannot complete cleanly.
    PreserveOnFailure,
}

impl AbortPolicy {
    /// Parses a workspace abort policy.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] for empty, malformed, or unknown values.
    pub fn parse(value: &str) -> Result<Self, crate::AdapterContractError> {
        super::text::parse_value(ABORT_POLICY_FIELD, value, |normalized| match normalized {
            ABORT_POLICY_DISCARD => Some(Self::Discard),
            ABORT_POLICY_PRESERVE_ON_FAILURE => Some(Self::PreserveOnFailure),
            _ => None,
        })
    }

    /// Returns the stable abort-policy label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Discard => ABORT_POLICY_DISCARD,
            Self::PreserveOnFailure => ABORT_POLICY_PRESERVE_ON_FAILURE,
        }
    }
}

/// Conflict behavior for workspace commit planning.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConflictPolicy {
    /// Fail before mutation when conflicts are detected.
    Fail,
    /// Allow a host-owned manual conflict-resolution flow.
    AllowManual,
}

impl ConflictPolicy {
    /// Parses a workspace conflict policy.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] for empty, malformed, or unknown values.
    pub fn parse(value: &str) -> Result<Self, crate::AdapterContractError> {
        super::text::parse_value(
            CONFLICT_POLICY_FIELD,
            value,
            |normalized| match normalized {
                CONFLICT_POLICY_FAIL => Some(Self::Fail),
                CONFLICT_POLICY_ALLOW_MANUAL => Some(Self::AllowManual),
                _ => None,
            },
        )
    }

    /// Returns the stable conflict-policy label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Fail => CONFLICT_POLICY_FAIL,
            Self::AllowManual => CONFLICT_POLICY_ALLOW_MANUAL,
        }
    }
}

/// Cleanup behavior required after workspace operations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CleanupPolicy {
    /// Cleanup must be confirmed for the operation to be considered complete.
    Required,
    /// Cleanup is attempted and failures are reported as best effort.
    BestEffort,
}

impl CleanupPolicy {
    /// Parses a workspace cleanup policy.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] for empty, malformed, or unknown values.
    pub fn parse(value: &str) -> Result<Self, crate::AdapterContractError> {
        super::text::parse_value(CLEANUP_POLICY_FIELD, value, |normalized| match normalized {
            CLEANUP_POLICY_REQUIRED => Some(Self::Required),
            CLEANUP_POLICY_BEST_EFFORT => Some(Self::BestEffort),
            _ => None,
        })
    }

    /// Returns the stable cleanup-policy label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Required => CLEANUP_POLICY_REQUIRED,
            Self::BestEffort => CLEANUP_POLICY_BEST_EFFORT,
        }
    }
}

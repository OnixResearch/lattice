/// Profile validation error class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProcessJobProfileErrorKind {
    /// Requested profile was not available.
    UnknownProfile,
    /// Manifest schema version is unsupported.
    UnsupportedSchemaVersion,
    /// Duplicate profiles at selected precedence are ambiguous.
    DuplicateProfile,
    /// Backend is not allowed by policy.
    DisallowedBackend,
    /// Command/program shape is invalid.
    InvalidCommandShape,
    /// Argument value is invalid.
    InvalidArgument,
    /// Environment key is denied by policy.
    DisallowedEnvironmentKey,
    /// Working directory is denied by policy.
    DisallowedCwd,
    /// Writable path is denied by policy.
    DisallowedWritablePath,
    /// Resource limit exceeds policy.
    ResourceLimitExceeded,
    /// Metadata contains unsafe or leaking content.
    UnsafeMetadata,
    /// Identity field contains unsafe or leaking content.
    UnsafeIdentity,
    /// Required field is empty.
    EmptyField,
}

/// Typed profile validation error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProcessJobProfileError {
    kind: ProcessJobProfileErrorKind,
    profile: String,
    field: &'static str,
    reason: String,
}

impl ProcessJobProfileError {
    pub(crate) fn new(
        kind: ProcessJobProfileErrorKind,
        profile: impl Into<String>,
        field: &'static str,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            profile: profile.into(),
            field,
            reason: reason.into(),
        }
    }

    /// Returns the error class.
    #[must_use]
    pub const fn kind(&self) -> ProcessJobProfileErrorKind {
        self.kind
    }

    /// Returns the profile associated with the error.
    #[must_use]
    pub fn profile(&self) -> &str {
        self.profile.as_str()
    }

    /// Returns the field associated with the error.
    #[must_use]
    pub const fn field(&self) -> &'static str {
        self.field
    }

    /// Returns the safe diagnostic reason.
    #[must_use]
    pub fn reason(&self) -> &str {
        self.reason.as_str()
    }
}

impl std::fmt::Display for ProcessJobProfileError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "process job profile `{}` field `{}` failed ({:?}): {}",
            self.profile, self.field, self.kind, self.reason
        )
    }
}

impl std::error::Error for ProcessJobProfileError {}

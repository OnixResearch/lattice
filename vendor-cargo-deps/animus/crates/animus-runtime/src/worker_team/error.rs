/// Worker-team validation, merge, and injected-service error class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FailureKind {
    /// Required field was omitted.
    MissingField,
    /// A field was present but empty after trimming.
    EmptyField,
    /// A bounded text field exceeded the safe length limit.
    FieldTooLong,
    /// A field contained raw private or leaking material.
    UnsafeText,
    /// Worker authority attempted to bypass parent policy.
    OverbroadAuthority,
    /// A context packet attempted to include disallowed context.
    ContextLeak,
    /// A worker report did not match its declared schema.
    MalformedReport,
    /// A worker report declared a different schema than expected.
    OutputSchemaMismatch,
    /// Worker reports contained incompatible claims or mutation intents.
    ConflictingReport,
    /// No host worker service was injected.
    WorkerServiceUnavailable,
    /// The injected worker service reported cancellation.
    WorkerCancelled,
    /// The injected worker service reported timeout.
    WorkerTimedOut,
    /// The injected worker service failed safely.
    InjectedServiceFailure,
}

/// Typed worker-team error with safe field and reason metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DelegationFailure {
    kind: FailureKind,
    field: &'static str,
    reason: String,
}

impl DelegationFailure {
    pub(crate) fn new(kind: FailureKind, field: &'static str, reason: impl Into<String>) -> Self {
        Self {
            kind,
            field,
            reason: reason.into(),
        }
    }

    /// Returns the error class.
    #[must_use]
    pub const fn kind(&self) -> FailureKind {
        self.kind
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

impl std::fmt::Display for DelegationFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "worker team field `{}` failed ({:?}): {}",
            self.field, self.kind, self.reason
        )
    }
}

impl std::error::Error for DelegationFailure {}

const OUTCOME_KIND_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("workspace_outcome_kind");
const ISSUE_CODE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("workspace_issue_code");

const OUTCOME_CREATED: &str = "created";
const OUTCOME_COMMITTED: &str = "committed";
const OUTCOME_ABORTED: &str = "aborted";
const OUTCOME_CONFLICT: &str = "conflict";
const OUTCOME_UNAVAILABLE: &str = "unavailable";
const OUTCOME_FAILED_CLEANUP: &str = "failed_cleanup";
const OUTCOME_AMBIGUOUS: &str = "ambiguous";

/// Workspace adapter outcome kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Kind {
    /// Workspace was created.
    Created,
    /// Workspace was committed.
    Committed,
    /// Workspace was aborted.
    Aborted,
    /// Commit or setup found a conflict.
    Conflict,
    /// Backend was unavailable.
    Unavailable,
    /// Cleanup failed.
    FailedCleanup,
    /// Backend state was ambiguous.
    Ambiguous,
}

impl Kind {
    /// Parses a workspace adapter outcome kind.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] for empty, malformed, or unknown values.
    pub fn parse(value: &str) -> Result<Self, crate::AdapterContractError> {
        super::text::parse_value(OUTCOME_KIND_FIELD, value, |normalized| match normalized {
            OUTCOME_CREATED => Some(Self::Created),
            OUTCOME_COMMITTED => Some(Self::Committed),
            OUTCOME_ABORTED => Some(Self::Aborted),
            OUTCOME_CONFLICT => Some(Self::Conflict),
            OUTCOME_UNAVAILABLE => Some(Self::Unavailable),
            OUTCOME_FAILED_CLEANUP => Some(Self::FailedCleanup),
            OUTCOME_AMBIGUOUS => Some(Self::Ambiguous),
            _ => None,
        })
    }

    /// Returns the stable outcome-kind label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Created => OUTCOME_CREATED,
            Self::Committed => OUTCOME_COMMITTED,
            Self::Aborted => OUTCOME_ABORTED,
            Self::Conflict => OUTCOME_CONFLICT,
            Self::Unavailable => OUTCOME_UNAVAILABLE,
            Self::FailedCleanup => OUTCOME_FAILED_CLEANUP,
            Self::Ambiguous => OUTCOME_AMBIGUOUS,
        }
    }
}

/// Typed workspace adapter outcome.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Outcome {
    kind: Kind,
    workspace_id: alloc::string::String,
    issue_code: Option<alloc::string::String>,
}

impl Outcome {
    /// Validates a workspace adapter outcome.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] when fields are empty, malformed,
    /// or unknown.
    pub fn new(draft: OutcomeDraft<'_>) -> Result<Self, crate::AdapterContractError> {
        let kind = Kind::parse(draft.kind)?;
        crate::validation::validate_required_adapter_field(
            super::intent::WORKSPACE_ID_FIELD_NAME,
            draft.workspace_id,
        )?;
        let issue_code = super::text::optional_field(ISSUE_CODE_FIELD, draft.issue_code)?;
        Ok(Self {
            kind,
            workspace_id: alloc::string::String::from(draft.workspace_id),
            issue_code,
        })
    }

    /// Returns the outcome kind.
    #[must_use]
    pub const fn kind(&self) -> Kind {
        self.kind
    }

    /// Returns the workspace identity.
    #[must_use]
    pub fn workspace_id(&self) -> &str {
        self.workspace_id.as_str()
    }

    /// Returns the optional issue code.
    #[must_use]
    pub fn issue_code(&self) -> Option<&str> {
        self.issue_code.as_deref()
    }
}

/// Borrowed workspace adapter outcome input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OutcomeDraft<'a> {
    /// Outcome kind label.
    pub kind: &'a str,
    /// Workspace identity.
    pub workspace_id: &'a str,
    /// Optional bounded issue code.
    pub issue_code: Option<&'a str>,
}

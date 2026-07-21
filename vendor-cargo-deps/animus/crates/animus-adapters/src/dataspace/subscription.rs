const SUBSCRIPTION_KIND_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("dataspace_subscription_kind");

const ISSUE_RAW_PAYLOAD_REQUESTED: &str = "DATASPACE_RAW_PAYLOAD_REQUESTED";
const ISSUE_CROSS_WORKSPACE_SCOPE: &str = "DATASPACE_CROSS_WORKSPACE_SCOPE";
const ISSUE_EMPTY_QUERY: &str = "DATASPACE_EMPTY_QUERY";
const ISSUE_INVALID_FIELD: &str = "DATASPACE_INVALID_FIELD";

/// Typed failure raised while validating a safe dataspace subscription.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Issue {
    /// Query requested raw private material instead of bounded assertions.
    RawPayloadRequested,
    /// Query attempted to cross tenant or workspace scope.
    CrossWorkspaceScope,
    /// Query did not name any supported assertion kind.
    EmptyQuery,
    /// Query field validation failed.
    InvalidField {
        /// Field-level adapter contract error.
        error: crate::AdapterContractError,
    },
}

impl Issue {
    /// Returns the stable rejection issue code.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::RawPayloadRequested => ISSUE_RAW_PAYLOAD_REQUESTED,
            Self::CrossWorkspaceScope => ISSUE_CROSS_WORKSPACE_SCOPE,
            Self::EmptyQuery => ISSUE_EMPTY_QUERY,
            Self::InvalidField { .. } => ISSUE_INVALID_FIELD,
        }
    }
}

/// Safe query pattern for an optional dataspace backend.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Query {
    scope: super::Scope,
    kinds: alloc::vec::Vec<super::AssertionKind>,
}

impl Query {
    /// Validates a safe dataspace subscription.
    ///
    /// # Errors
    ///
    /// Returns [`Issue`] when the query requests raw payloads, crosses scope,
    /// is empty, or uses an unsupported assertion kind.
    pub fn new(draft: Draft<'_>) -> Result<Self, Issue> {
        if draft.include_raw_payloads {
            return Err(Issue::RawPayloadRequested);
        }
        if draft.cross_workspace {
            return Err(Issue::CrossWorkspaceScope);
        }
        if draft.kinds.is_empty() {
            return Err(Issue::EmptyQuery);
        }
        let kinds = draft
            .kinds
            .iter()
            .map(|value| super::assertion::parse_kind_for_field(SUBSCRIPTION_KIND_FIELD, value))
            .collect::<Result<alloc::vec::Vec<_>, _>>()
            .map_err(|error| Issue::InvalidField { error })?;

        Ok(Self {
            scope: draft.scope,
            kinds,
        })
    }

    /// Returns the subscription scope boundary.
    #[must_use]
    pub const fn scope(&self) -> &super::Scope {
        &self.scope
    }

    /// Returns allowed assertion kinds.
    #[must_use]
    pub fn kinds(&self) -> &[super::AssertionKind] {
        self.kinds.as_slice()
    }

    /// Returns whether this query includes `kind`.
    #[must_use]
    pub fn allows(&self, kind: super::AssertionKind) -> bool {
        self.kinds.contains(&kind)
    }
}

/// Borrowed input used to validate a safe dataspace subscription.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Draft<'a> {
    /// Scope boundary for the query.
    pub scope: super::Scope,
    /// Assertion kind labels allowed by the query.
    pub kinds: &'a [&'a str],
    /// Whether the caller requested raw private payloads.
    pub include_raw_payloads: bool,
    /// Whether the caller requested a cross-tenant or cross-workspace query.
    pub cross_workspace: bool,
}

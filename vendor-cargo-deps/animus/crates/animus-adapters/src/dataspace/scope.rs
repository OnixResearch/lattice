const TENANT_ID_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("dataspace_tenant_id");
const WORKSPACE_ID_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("dataspace_workspace_id");
const SESSION_ID_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("dataspace_session_id");
const BRANCH_ID_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("dataspace_branch_id");
const FACET_ID_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("dataspace_facet_id");
const ACTOR_KIND_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("dataspace_actor_kind");
const ACTOR_ID_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("dataspace_actor_id");

const ACTOR_KIND_AGENT: &str = "agent";
const ACTOR_KIND_SUBAGENT: &str = "subagent";
const ACTOR_KIND_TOOL: &str = "tool";
const ACTOR_KIND_BACKGROUND_WORK: &str = "background_work";
const ACTOR_KIND_HOST: &str = "host";

/// Actor class attached to a scoped dataspace assertion.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActorKind {
    /// Top-level agent loop actor.
    Agent,
    /// Nested subagent actor.
    Subagent,
    /// Tool invocation actor.
    Tool,
    /// Background work actor.
    BackgroundWork,
    /// Host orchestration actor.
    Host,
}

impl ActorKind {
    /// Parses a stable dataspace actor-kind label.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] for empty, malformed, or unknown values.
    pub fn parse(value: &str) -> Result<Self, crate::AdapterContractError> {
        super::parse_value(ACTOR_KIND_FIELD, value, Self::from_normalized)
    }

    fn from_normalized(normalized: &str) -> Option<Self> {
        match normalized {
            ACTOR_KIND_AGENT => Some(Self::Agent),
            ACTOR_KIND_SUBAGENT => Some(Self::Subagent),
            ACTOR_KIND_TOOL => Some(Self::Tool),
            ACTOR_KIND_BACKGROUND_WORK => Some(Self::BackgroundWork),
            ACTOR_KIND_HOST => Some(Self::Host),
            _ => None,
        }
    }

    /// Returns the stable actor-kind label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Agent => ACTOR_KIND_AGENT,
            Self::Subagent => ACTOR_KIND_SUBAGENT,
            Self::Tool => ACTOR_KIND_TOOL,
            Self::BackgroundWork => ACTOR_KIND_BACKGROUND_WORK,
            Self::Host => ACTOR_KIND_HOST,
        }
    }
}

/// Explicit tenant/workspace/session/facet scope for dataspace projections.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Projection {
    tenant_id: alloc::string::String,
    workspace_id: alloc::string::String,
    session_id: alloc::string::String,
    branch_id: alloc::string::String,
    facet_id: alloc::string::String,
    actor_kind: ActorKind,
    actor_id: alloc::string::String,
}

impl Projection {
    /// Validates and normalizes a dataspace projection scope.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] when a scope label is empty,
    /// malformed, too long, or uses an unknown actor kind.
    pub fn new(draft: Draft<'_>) -> Result<Self, crate::AdapterContractError> {
        let tenant_id =
            crate::validation::normalized_adapter_field(TENANT_ID_FIELD, draft.tenant_id)?;
        let workspace_id =
            crate::validation::normalized_adapter_field(WORKSPACE_ID_FIELD, draft.workspace_id)?;
        let session_id =
            crate::validation::normalized_adapter_field(SESSION_ID_FIELD, draft.session_id)?;
        let branch_id =
            crate::validation::normalized_adapter_field(BRANCH_ID_FIELD, draft.branch_id)?;
        let facet_id = crate::validation::normalized_adapter_field(FACET_ID_FIELD, draft.facet_id)?;
        let actor_kind = ActorKind::parse(draft.actor_kind)?;
        let actor_id = crate::validation::normalized_adapter_field(ACTOR_ID_FIELD, draft.actor_id)?;

        Ok(Self {
            tenant_id,
            workspace_id,
            session_id,
            branch_id,
            facet_id,
            actor_kind,
            actor_id,
        })
    }

    /// Returns the tenant identity label.
    #[must_use]
    pub fn tenant_id(&self) -> &str {
        self.tenant_id.as_str()
    }

    /// Returns the workspace identity label.
    #[must_use]
    pub fn workspace_id(&self) -> &str {
        self.workspace_id.as_str()
    }

    /// Returns the session identity label.
    #[must_use]
    pub fn session_id(&self) -> &str {
        self.session_id.as_str()
    }

    /// Returns the branch identity label.
    #[must_use]
    pub fn branch_id(&self) -> &str {
        self.branch_id.as_str()
    }

    /// Returns the facet identity label.
    #[must_use]
    pub fn facet_id(&self) -> &str {
        self.facet_id.as_str()
    }

    /// Returns the scoped actor kind.
    #[must_use]
    pub const fn actor_kind(&self) -> ActorKind {
        self.actor_kind
    }

    /// Returns the scoped actor identity label.
    #[must_use]
    pub fn actor_id(&self) -> &str {
        self.actor_id.as_str()
    }

    /// Returns whether another scope shares the tenant and workspace boundary.
    #[must_use]
    pub fn same_tenant_workspace(&self, other: &Self) -> bool {
        self.tenant_id == other.tenant_id && self.workspace_id == other.workspace_id
    }

    /// Returns whether another scope names the same actor identity and branch.
    #[must_use]
    pub fn same_actor_scope(&self, other: &Self) -> bool {
        self.same_tenant_workspace(other)
            && self.session_id == other.session_id
            && self.branch_id == other.branch_id
            && self.facet_id == other.facet_id
            && self.actor_kind == other.actor_kind
            && self.actor_id == other.actor_id
    }
}

/// Borrowed input used to validate a dataspace projection scope.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Draft<'a> {
    /// Tenant identity boundary.
    pub tenant_id: &'a str,
    /// Workspace identity boundary.
    pub workspace_id: &'a str,
    /// Runtime session identity boundary.
    pub session_id: &'a str,
    /// Branch or speculative workspace branch identity.
    pub branch_id: &'a str,
    /// Facet identity used for assertion lifetime ownership.
    pub facet_id: &'a str,
    /// Actor kind label.
    pub actor_kind: &'a str,
    /// Actor identity within the scope.
    pub actor_id: &'a str,
}

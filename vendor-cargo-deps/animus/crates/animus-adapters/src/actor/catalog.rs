pub(super) mod parse;

/// Borrowed, host-loaded agent-definition manifest data before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Draft<'a> {
    /// Stable user-facing agent name.
    pub name: Option<&'a str>,
    /// Safe human-facing description.
    pub description: Option<&'a str>,
    /// Requested model-role label. This is a request, not an authority grant.
    pub model_preference: Option<&'a str>,
    /// Prompt fragments attributed to this agent definition.
    pub prompt_fragments: &'a [&'a str],
    /// Capability labels this agent is allowed to request at most.
    pub capability_ceilings: &'a [&'a str],
    /// Toolset labels this agent is allowed to request at most.
    pub toolset_limits: &'a [&'a str],
    /// Memory-scope labels this agent is allowed to request at most.
    pub memory_scopes: &'a [&'a str],
    /// Context-reference-scope labels this agent is allowed to request at most.
    pub context_reference_scopes: &'a [&'a str],
    /// Workflow ability labels this agent is allowed to request at most.
    pub workflow_abilities: &'a [&'a str],
    /// Manifest source scope label.
    pub source_scope: Option<&'a str>,
    /// Host trust label for the manifest source.
    pub trust_status: Option<&'a str>,
    /// Stable safe manifest identity, usually a host-provided hash/URN.
    pub manifest_identity: Option<&'a str>,
}

/// Validated, typed agent-definition manifest.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Manifest {
    pub(super) name: super::Text,
    pub(super) description: super::Text,
    pub(super) model_preference: super::Text,
    pub(super) prompt_fragments: super::Collection<super::Text>,
    pub(super) capability_ceilings: super::Collection<super::Text>,
    pub(super) toolset_limits: super::Collection<super::Text>,
    pub(super) memory_scopes: super::Collection<super::Text>,
    pub(super) context_reference_scopes: super::Collection<super::Text>,
    pub(super) workflow_abilities: super::Collection<super::Text>,
    pub(super) source_scope: SourceScope,
    pub(super) trust_status: TrustStatus,
    pub(super) identity: super::Text,
}

impl Manifest {
    /// Validates host-loaded manifest data as typed data only.
    ///
    /// # Errors
    ///
    /// Returns [`super::Error`] when a required field is missing, malformed,
    /// unsupported, too large, or prompt fragments exceed the bounded prompt
    /// budget.
    pub fn validate(draft: &Draft<'_>) -> Result<Self, super::Error> {
        parse::validate(draft)
    }

    /// Returns the stable agent name.
    #[must_use]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns the safe human-facing description.
    #[must_use]
    pub fn description(&self) -> &str {
        self.description.as_str()
    }

    /// Returns the requested model role label.
    #[must_use]
    pub fn model_preference(&self) -> &str {
        self.model_preference.as_str()
    }

    /// Returns prompt fragments as validated bounded data.
    #[must_use]
    pub fn prompt_fragments(&self) -> &[super::Text] {
        self.prompt_fragments.as_slice()
    }

    /// Returns requested capability ceilings.
    #[must_use]
    pub fn capability_ceilings(&self) -> &[super::Text] {
        self.capability_ceilings.as_slice()
    }

    /// Returns requested toolset ceilings.
    #[must_use]
    pub fn toolset_limits(&self) -> &[super::Text] {
        self.toolset_limits.as_slice()
    }

    /// Returns requested memory-scope ceilings.
    #[must_use]
    pub fn memory_scopes(&self) -> &[super::Text] {
        self.memory_scopes.as_slice()
    }

    /// Returns requested context-reference ceilings.
    #[must_use]
    pub fn context_reference_scopes(&self) -> &[super::Text] {
        self.context_reference_scopes.as_slice()
    }

    /// Returns requested workflow-ability ceilings.
    #[must_use]
    pub fn workflow_abilities(&self) -> &[super::Text] {
        self.workflow_abilities.as_slice()
    }

    /// Returns the typed source scope.
    #[must_use]
    pub const fn source_scope(&self) -> SourceScope {
        self.source_scope
    }

    /// Returns the manifest trust label.
    #[must_use]
    pub const fn trust_status(&self) -> TrustStatus {
        self.trust_status
    }

    /// Returns the safe manifest identity.
    #[must_use]
    pub fn manifest_identity(&self) -> &str {
        self.identity.as_str()
    }

    /// Returns the authority boundary for parsed manifests.
    #[must_use]
    pub const fn authority_boundary(&self) -> AuthorityBoundary {
        match self.source_scope {
            SourceScope::User | SourceScope::Project | SourceScope::ExplicitHost => {
                AuthorityBoundary::CeilingOnly
            }
        }
    }

    pub(super) const fn precedence_rank(&self) -> u8 {
        match self.source_scope {
            SourceScope::ExplicitHost => super::HOST_SOURCE_RANK,
            SourceScope::User => super::USER_SOURCE_RANK,
            SourceScope::Project => super::PROJECT_SOURCE_RANK,
        }
    }
}

/// Agent-definition source scope.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourceScope {
    /// User/global host-owned definition.
    User,
    /// Project/workspace-local definition from repository data.
    Project,
    /// Explicit host-provided definition outside project data.
    ExplicitHost,
}

impl SourceScope {
    /// Returns the stable label used in receipts and manifests.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::User => super::USER_SOURCE_LABEL,
            Self::Project => super::PROJECT_SOURCE_LABEL,
            Self::ExplicitHost => super::HOST_SOURCE_LABEL,
        }
    }
}

/// Agent-definition trust status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrustStatus {
    /// Host has marked the profile trusted.
    Trusted,
    /// Host has not admitted the profile.
    Untrusted,
}

impl TrustStatus {
    /// Returns the stable label used in receipts and manifests.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Trusted => super::TRUSTED_LABEL,
            Self::Untrusted => super::UNTRUSTED_LABEL,
        }
    }
}

/// Authority boundary for parsed manifests.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthorityBoundary {
    /// Manifest values are ceilings or requests; runtime policy still decides.
    CeilingOnly,
}

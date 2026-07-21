type Text = super::Text;

/// Borrowed host-supplied resource fields before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Draft<'a> {
    /// Resource kind label: skill, prompt-template, context-guidance, or host-resource.
    pub kind: Option<&'a str>,
    /// Stable user-facing resource name or command handle.
    pub name: Option<&'a str>,
    /// Safe human-facing summary shown before full content loading.
    pub description: Option<&'a str>,
    /// Source scope label for host policy decisions.
    pub scope: Option<&'a str>,
    /// Safe provenance label suitable for model-visible summaries.
    pub provenance: Option<&'a str>,
    /// Host trust status for this descriptor.
    pub trust_status: Option<&'a str>,
    /// Lowercase BLAKE3 identity for host-supplied full content.
    pub content_identity: Option<&'a str>,
}

/// Validated host-provided resource descriptor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Descriptor {
    pub(super) kind: Kind,
    pub(super) name: Text,
    pub(super) description: Text,
    pub(super) scope: Scope,
    pub(super) provenance: Text,
    pub(super) trust_status: TrustStatus,
    pub(super) content_identity: crate::artifact::Hash,
}

impl Descriptor {
    /// Returns the resource kind.
    #[must_use]
    pub const fn kind(&self) -> Kind {
        self.kind
    }

    /// Returns the validated resource name.
    #[must_use]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns the safe human-facing summary.
    #[must_use]
    pub fn description(&self) -> &str {
        self.description.as_str()
    }

    /// Returns the source scope.
    #[must_use]
    pub const fn scope(&self) -> Scope {
        self.scope
    }

    /// Returns the safe provenance label.
    #[must_use]
    pub fn provenance(&self) -> &str {
        self.provenance.as_str()
    }

    /// Returns the host trust status.
    #[must_use]
    pub const fn trust_status(&self) -> TrustStatus {
        self.trust_status
    }

    /// Returns the safe content identity for full content.
    #[must_use]
    pub fn content_identity(&self) -> &crate::artifact::Hash {
        &self.content_identity
    }

    pub(super) fn summary(&self) -> super::Summary {
        super::Summary {
            kind: self.kind,
            name: self.name.clone(),
            description: self.description.clone(),
            scope: self.scope,
            provenance: self.provenance.clone(),
        }
    }
}

/// Resource descriptor kind.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Kind {
    /// Progressive-disclosure workflow skill.
    Skill,
    /// Prompt template selected by host policy or explicit command.
    PromptTemplate,
    /// Context guidance, project memory, or AGENTS-style content supplied as data.
    ContextGuidance,
    /// Host-owned resource that does not fit a narrower category.
    HostResource,
}

impl Kind {
    /// Returns the stable descriptor label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Skill => super::SKILL_KIND_LABEL,
            Self::PromptTemplate => super::PROMPT_TEMPLATE_KIND_LABEL,
            Self::ContextGuidance => super::CONTEXT_GUIDANCE_KIND_LABEL,
            Self::HostResource => super::HOST_RESOURCE_KIND_LABEL,
        }
    }
}

/// Resource source scope.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Scope {
    /// User/global configuration supplied by the host.
    Global,
    /// Project-local configuration that requires host trust admission.
    Project,
    /// Session-local resource supplied for the current run.
    Session,
    /// Host/application-owned resource.
    Host,
}

impl Scope {
    /// Returns the stable scope label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Global => super::GLOBAL_SCOPE_LABEL,
            Self::Project => super::PROJECT_SCOPE_LABEL,
            Self::Session => super::SESSION_SCOPE_LABEL,
            Self::Host => super::HOST_SCOPE_LABEL,
        }
    }
}

/// Host trust status for a resource descriptor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrustStatus {
    /// Host has admitted this descriptor for prompt influence and full-content loading.
    Trusted,
    /// Host has not admitted this descriptor for prompt influence or full-content loading.
    Untrusted,
}

impl TrustStatus {
    /// Returns the stable trust label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Trusted => super::TRUSTED_LABEL,
            Self::Untrusted => super::UNTRUSTED_LABEL,
        }
    }
}

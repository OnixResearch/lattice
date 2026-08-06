/// Borrowed remote admission request before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AdmissionRequestDraft<'a> {
    /// Intended audience label.
    pub audience: &'a str,
    /// Actor label.
    pub actor: &'a str,
    /// Entrypoint label.
    pub entrypoint: super::Entrypoint,
    /// Session or workspace scope label.
    pub scope: &'a str,
    /// Requested action.
    pub action: super::Action,
    /// Requested resource label.
    pub resource: &'a str,
    /// Requested ability label.
    pub ability: &'a str,
    /// Optional credential evidence supplied by the host verifier.
    pub credential: Option<super::CredentialEvidenceDraft<'a>>,
    /// Local caveat result.
    pub caveat: super::AdmissionCaveat,
    /// Optional BLAKE3 policy hash.
    pub policy_hash: Option<&'a str>,
}

/// Validated remote admission request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdmissionRequest {
    audience: alloc::string::String,
    actor: alloc::string::String,
    entrypoint: super::Entrypoint,
    scope: alloc::string::String,
    action: super::Action,
    resource: alloc::string::String,
    ability: alloc::string::String,
    credential: Option<super::CredentialEvidence>,
    caveat: super::AdmissionCaveat,
    policy_hash: Option<crate::ArtifactHash>,
}

impl AdmissionRequest {
    /// Builds a validated admission request.
    ///
    /// # Errors
    ///
    /// Returns [`super::AdmissionError`] when labels or hashes are malformed.
    pub fn new(draft: &AdmissionRequestDraft<'_>) -> Result<Self, super::AdmissionError> {
        Ok(Self {
            audience: super::normalized(super::AUDIENCE_FIELD, draft.audience)?,
            actor: super::normalized(super::ACTOR_FIELD, draft.actor)?,
            entrypoint: draft.entrypoint,
            scope: super::normalized(super::SCOPE_FIELD, draft.scope)?,
            action: draft.action,
            resource: super::normalized(super::RESOURCE_FIELD, draft.resource)?,
            ability: super::normalized(super::ABILITY_FIELD, draft.ability)?,
            credential: draft.credential.as_ref().map(super::CredentialEvidence::new).transpose()?,
            caveat: draft.caveat,
            policy_hash: draft.policy_hash.map(crate::ArtifactHash::new).transpose()?,
        })
    }

    /// Returns audience label.
    #[must_use]
    pub fn audience(&self) -> &str {
        self.audience.as_str()
    }

    /// Returns actor label.
    #[must_use]
    pub fn actor(&self) -> &str {
        self.actor.as_str()
    }

    /// Returns entrypoint.
    #[must_use]
    pub const fn entrypoint(&self) -> super::Entrypoint {
        self.entrypoint
    }

    /// Returns session or workspace scope label.
    #[must_use]
    pub fn scope(&self) -> &str {
        self.scope.as_str()
    }

    /// Returns requested action.
    #[must_use]
    pub const fn action(&self) -> super::Action {
        self.action
    }

    /// Returns requested resource label.
    #[must_use]
    pub fn resource(&self) -> &str {
        self.resource.as_str()
    }

    /// Returns requested ability label.
    #[must_use]
    pub fn ability(&self) -> &str {
        self.ability.as_str()
    }

    /// Returns optional credential evidence.
    #[must_use]
    pub const fn credential(&self) -> Option<&super::CredentialEvidence> {
        self.credential.as_ref()
    }

    /// Returns local caveat result.
    #[must_use]
    pub const fn caveat(&self) -> super::AdmissionCaveat {
        self.caveat
    }

    /// Returns optional policy hash.
    #[must_use]
    pub const fn policy_hash(&self) -> Option<&crate::ArtifactHash> {
        self.policy_hash.as_ref()
    }
}

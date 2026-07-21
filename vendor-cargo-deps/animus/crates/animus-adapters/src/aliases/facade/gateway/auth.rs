/// Credential expiry status supplied by the host verifier.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CredentialExpiry {
    /// Credential is within its accepted time window.
    Current,
    /// Credential is outside its accepted time window.
    Expired,
}

impl CredentialExpiry {
    /// Returns a stable expiry label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Current => "current",
            Self::Expired => "expired",
        }
    }
}

/// Credential verifier status supplied by a host-owned verifier backend.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CredentialVerifierStatus {
    /// Credential verifier accepted the credential evidence.
    Verified,
    /// Credential verifier reports revocation.
    Revoked,
    /// Credential verifier is unavailable.
    Unavailable,
}

impl CredentialVerifierStatus {
    /// Returns a stable verifier status label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Verified => "verified",
            Self::Revoked => "revoked",
            Self::Unavailable => "unavailable",
        }
    }
}

/// Borrowed credential evidence before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CredentialEvidenceDraft<'a> {
    /// Intended audience label.
    pub audience: &'a str,
    /// Actor label.
    pub actor: &'a str,
    /// Resource label covered by the grant.
    pub resource: &'a str,
    /// Ability label covered by the grant.
    pub ability: &'a str,
    /// BLAKE3 hash of credential evidence held by the host.
    pub credential_hash: &'a str,
    /// Credential expiry status supplied by the host verifier.
    pub expiry: CredentialExpiry,
    /// Host verifier status.
    pub verifier_status: CredentialVerifierStatus,
}

/// Validated credential evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CredentialEvidence {
    audience: alloc::string::String,
    actor: alloc::string::String,
    resource: alloc::string::String,
    ability: alloc::string::String,
    credential_hash: crate::ArtifactHash,
    expiry: CredentialExpiry,
    verifier_status: CredentialVerifierStatus,
}

impl CredentialEvidence {
    /// Builds validated credential evidence.
    ///
    /// # Errors
    ///
    /// Returns [`super::AdmissionError`] when labels or hashes are malformed.
    pub fn new(draft: &CredentialEvidenceDraft<'_>) -> Result<Self, super::AdmissionError> {
        Ok(Self {
            audience: super::normalized(super::AUDIENCE_FIELD, draft.audience)?,
            actor: super::normalized(super::ACTOR_FIELD, draft.actor)?,
            resource: super::normalized(super::RESOURCE_FIELD, draft.resource)?,
            ability: super::normalized(super::ABILITY_FIELD, draft.ability)?,
            credential_hash: crate::ArtifactHash::new(draft.credential_hash)?,
            expiry: draft.expiry,
            verifier_status: draft.verifier_status,
        })
    }

    /// Returns intended audience label.
    #[must_use]
    pub fn audience(&self) -> &str {
        self.audience.as_str()
    }

    /// Returns actor label.
    #[must_use]
    pub fn actor(&self) -> &str {
        self.actor.as_str()
    }

    /// Returns granted resource label.
    #[must_use]
    pub fn resource(&self) -> &str {
        self.resource.as_str()
    }

    /// Returns granted ability label.
    #[must_use]
    pub fn ability(&self) -> &str {
        self.ability.as_str()
    }

    /// Returns the credential evidence hash.
    #[must_use]
    pub const fn credential_hash(&self) -> &crate::ArtifactHash {
        &self.credential_hash
    }

    /// Returns credential expiry status.
    #[must_use]
    pub const fn expiry(&self) -> CredentialExpiry {
        self.expiry
    }

    /// Returns verifier status.
    #[must_use]
    pub const fn verifier_status(&self) -> CredentialVerifierStatus {
        self.verifier_status
    }
}

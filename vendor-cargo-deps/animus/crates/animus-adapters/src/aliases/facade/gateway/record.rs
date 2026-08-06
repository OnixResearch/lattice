mod digest;
mod render;

/// Safe admission receipt metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdmissionReceipt {
    verdict: super::AdmissionVerdict,
    reason: Option<super::AdmissionReason>,
    entrypoint: super::Entrypoint,
    action: super::Action,
    resource: alloc::string::String,
    ability: alloc::string::String,
    audience_hash: crate::ArtifactHash,
    actor_hash: crate::ArtifactHash,
    credential_hash: Option<crate::ArtifactHash>,
    caveat: super::AdmissionCaveat,
    policy_hash: Option<crate::ArtifactHash>,
    expiry: Option<super::CredentialExpiry>,
    verifier_status: Option<super::CredentialVerifierStatus>,
    receipt_hash: crate::ArtifactHash,
}

impl AdmissionReceipt {
    /// Returns admission verdict.
    #[must_use]
    pub const fn verdict(&self) -> super::AdmissionVerdict {
        self.verdict
    }

    /// Returns denial reason when denied.
    #[must_use]
    pub const fn reason(&self) -> Option<super::AdmissionReason> {
        self.reason
    }

    /// Returns entrypoint.
    #[must_use]
    pub const fn entrypoint(&self) -> super::Entrypoint {
        self.entrypoint
    }

    /// Returns action.
    #[must_use]
    pub const fn action(&self) -> super::Action {
        self.action
    }

    /// Returns resource label.
    #[must_use]
    pub fn resource(&self) -> &str {
        self.resource.as_str()
    }

    /// Returns ability label.
    #[must_use]
    pub fn ability(&self) -> &str {
        self.ability.as_str()
    }

    /// Returns hashed audience identity.
    #[must_use]
    pub const fn audience_hash(&self) -> &crate::ArtifactHash {
        &self.audience_hash
    }

    /// Returns hashed actor identity.
    #[must_use]
    pub const fn actor_hash(&self) -> &crate::ArtifactHash {
        &self.actor_hash
    }

    /// Returns credential evidence hash when supplied.
    #[must_use]
    pub const fn credential_hash(&self) -> Option<&crate::ArtifactHash> {
        self.credential_hash.as_ref()
    }

    /// Returns local caveat result.
    #[must_use]
    pub const fn caveat(&self) -> super::AdmissionCaveat {
        self.caveat
    }

    /// Returns policy hash when supplied.
    #[must_use]
    pub const fn policy_hash(&self) -> Option<&crate::ArtifactHash> {
        self.policy_hash.as_ref()
    }

    /// Returns credential expiry status when evidence was supplied.
    #[must_use]
    pub const fn expiry(&self) -> Option<super::CredentialExpiry> {
        self.expiry
    }

    /// Returns verifier status when evidence was supplied.
    #[must_use]
    pub const fn verifier_status(&self) -> Option<super::CredentialVerifierStatus> {
        self.verifier_status
    }

    /// Returns deterministic BLAKE3 receipt hash.
    #[must_use]
    pub const fn receipt_hash(&self) -> &crate::ArtifactHash {
        &self.receipt_hash
    }

    /// Renders safe receipt metadata without raw credential, prompt, or transport material.
    #[must_use]
    pub fn render_safe_text(&self) -> alloc::string::String {
        render::safe_text(self)
    }
}

pub(super) fn build(
    request: &super::AdmissionRequest,
    verdict: super::AdmissionVerdict,
    reason: Option<super::AdmissionReason>,
) -> Result<AdmissionReceipt, super::AdmissionError> {
    debug_assert!(!request.audience().is_empty());
    debug_assert!(!request.actor().is_empty());
    let audience_hash = digest::identity_for_label(render::LabelPayload {
        label: digest::LABEL_AUDIENCE,
        value: request.audience(),
    })?;
    let actor_hash = digest::identity_for_label(render::LabelPayload {
        label: digest::LABEL_ACTOR,
        value: request.actor(),
    })?;
    let mut receipt = AdmissionReceipt {
        verdict,
        reason,
        entrypoint: request.entrypoint(),
        action: request.action(),
        resource: alloc::string::String::from(request.resource()),
        ability: alloc::string::String::from(request.ability()),
        audience_hash,
        actor_hash,
        credential_hash: request.credential().map(|credential| credential.credential_hash().clone()),
        caveat: request.caveat(),
        policy_hash: request.policy_hash().cloned(),
        expiry: request.credential().map(super::CredentialEvidence::expiry),
        verifier_status: request.credential().map(super::CredentialEvidence::verifier_status),
        receipt_hash: digest::zero_identity()?,
    };
    receipt.receipt_hash = digest::identity_for_text(&receipt)?;
    Ok(receipt)
}

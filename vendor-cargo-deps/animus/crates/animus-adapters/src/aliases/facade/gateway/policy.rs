/// Admission validation error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AdmissionError {
    /// Boundary field validation failed.
    Contract(crate::AdapterContractError),
    /// Safe hash or receipt identity validation failed.
    Artifact(crate::ArtifactValidationError),
    /// Rendered receipt text contains unsafe material.
    ReceiptLeak {
        /// Matched unsafe marker.
        marker: &'static str,
    },
}

impl core::fmt::Display for AdmissionError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Contract(error) => write!(formatter, "remote admission contract error: {error}"),
            Self::Artifact(error) => write!(formatter, "remote admission artifact error: {error}"),
            Self::ReceiptLeak { marker } => {
                write!(formatter, "remote admission receipt leaked marker {marker}")
            }
        }
    }
}

impl From<crate::AdapterContractError> for AdmissionError {
    fn from(error: crate::AdapterContractError) -> Self {
        Self::Contract(error)
    }
}

impl From<crate::ArtifactValidationError> for AdmissionError {
    fn from(error: crate::ArtifactValidationError) -> Self {
        Self::Artifact(error)
    }
}

/// Admission decision and safe receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdmissionDecision {
    verdict: super::AdmissionVerdict,
    reason: Option<super::AdmissionReason>,
    receipt: super::AdmissionReceipt,
}

impl AdmissionDecision {
    pub(super) const fn new(
        verdict: super::AdmissionVerdict,
        reason: Option<super::AdmissionReason>,
        receipt: super::AdmissionReceipt,
    ) -> Self {
        Self {
            verdict,
            reason,
            receipt,
        }
    }

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

    /// Returns safe admission receipt.
    #[must_use]
    pub const fn receipt(&self) -> &super::AdmissionReceipt {
        &self.receipt
    }

    /// Reports whether the request is admitted.
    #[must_use]
    pub const fn is_allowed(&self) -> bool {
        matches!(self.verdict, super::AdmissionVerdict::Allow)
    }
}

/// Evaluates a remote admission request before side effects.
///
/// # Errors
///
/// Returns [`AdmissionError`] if safe receipt identity construction fails.
pub fn admit(request: &super::AdmissionRequest) -> Result<AdmissionDecision, AdmissionError> {
    let reason = denial_reason(request);
    let verdict = if reason.is_some() {
        super::AdmissionVerdict::Deny
    } else {
        super::AdmissionVerdict::Allow
    };
    let receipt = super::record::build(request, verdict, reason)?;
    Ok(AdmissionDecision::new(verdict, reason, receipt))
}

/// Rejects unsafe rendered admission receipt text.
///
/// # Errors
///
/// Returns [`AdmissionError::ReceiptLeak`] when text contains a known raw token,
/// private key, prompt body, provider payload, transport header, room identifier,
/// full path, or credential-body marker.
pub fn validate_safe_text(text: &str) -> Result<(), AdmissionError> {
    for marker in super::LEAK_MARKERS {
        if text.contains(marker) {
            return Err(AdmissionError::ReceiptLeak { marker });
        }
    }
    Ok(())
}

fn denial_reason(request: &super::AdmissionRequest) -> Option<super::AdmissionReason> {
    if request.entrypoint() == super::Entrypoint::Unsupported {
        return Some(super::AdmissionReason::UnsupportedEntrypoint);
    }
    if request.policy_hash().is_none() {
        return Some(super::AdmissionReason::MissingPolicy);
    }
    let Some(credential) = request.credential() else {
        return Some(super::AdmissionReason::MissingCredential);
    };
    credential_denial(request, credential).or_else(|| caveat_denial(request.caveat()))
}

fn credential_denial(
    request: &super::AdmissionRequest,
    credential: &super::CredentialEvidence,
) -> Option<super::AdmissionReason> {
    if credential.audience() != request.audience() {
        return Some(super::AdmissionReason::WrongAudience);
    }
    if credential.expiry() == super::CredentialExpiry::Expired {
        return Some(super::AdmissionReason::ExpiredCredential);
    }
    verifier_denial(credential).or_else(|| grant_denial(request, credential))
}

fn verifier_denial(credential: &super::CredentialEvidence) -> Option<super::AdmissionReason> {
    match credential.verifier_status() {
        super::CredentialVerifierStatus::Revoked => Some(super::AdmissionReason::RevokedCredential),
        super::CredentialVerifierStatus::Unavailable => Some(super::AdmissionReason::VerifierUnavailable),
        super::CredentialVerifierStatus::Verified => None,
    }
}

fn grant_denial(
    request: &super::AdmissionRequest,
    credential: &super::CredentialEvidence,
) -> Option<super::AdmissionReason> {
    if credential.resource() == request.resource() && credential.ability() == request.ability() {
        return None;
    }
    Some(super::AdmissionReason::MissingGrant)
}

fn caveat_denial(caveat: super::AdmissionCaveat) -> Option<super::AdmissionReason> {
    match caveat {
        super::AdmissionCaveat::Allow => None,
        super::AdmissionCaveat::Deny | super::AdmissionCaveat::PromptRequired => {
            Some(super::AdmissionReason::CaveatDenied)
        }
    }
}

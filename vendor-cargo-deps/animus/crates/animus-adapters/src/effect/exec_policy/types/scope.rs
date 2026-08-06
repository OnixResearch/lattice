/// Borrowed sandbox escalation context before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SandboxContextDraft<'a> {
    /// Active sandbox label before escalation.
    pub active_sandbox: Option<&'a str>,
    /// Capability requested outside the active sandbox.
    pub requested_capability: Option<&'a str>,
    /// Safe escalation reason label.
    pub escalation_reason: Option<&'a str>,
}

/// Validated sandbox escalation context.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SandboxContext {
    active_sandbox: super::super::Text,
    requested_capability: super::super::Text,
    escalation_reason: super::super::Text,
}

impl SandboxContext {
    /// Validates sandbox escalation context.
    ///
    /// # Errors
    ///
    /// Returns [`super::super::Error`] when a required field is missing or
    /// invalid.
    pub fn new(draft: &SandboxContextDraft<'_>) -> Result<Self, super::super::Error> {
        Ok(Self {
            active_sandbox: super::super::required_text(super::super::ACTIVE_SANDBOX_FIELD, draft.active_sandbox)?,
            requested_capability: super::super::required_text(
                super::super::REQUESTED_CAPABILITY_FIELD,
                draft.requested_capability,
            )?,
            escalation_reason: super::super::required_text(
                super::super::ESCALATION_REASON_FIELD,
                draft.escalation_reason,
            )?,
        })
    }

    /// Returns the active sandbox label.
    #[must_use]
    pub fn active_sandbox(&self) -> &str {
        self.active_sandbox.as_str()
    }

    /// Returns the requested capability label.
    #[must_use]
    pub fn requested_capability(&self) -> &str {
        self.requested_capability.as_str()
    }

    /// Returns the safe escalation reason label.
    #[must_use]
    pub fn escalation_reason(&self) -> &str {
        self.escalation_reason.as_str()
    }
}

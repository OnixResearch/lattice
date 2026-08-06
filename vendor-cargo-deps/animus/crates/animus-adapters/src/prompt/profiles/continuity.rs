/// Host or user decision for a chain-of-prompts transition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChainDecision {
    /// Transition was accepted by user, host, or explicit policy.
    Accepted,
    /// Transition was declined.
    Declined,
    /// Transition could not be evaluated by the host.
    Unavailable,
}

/// Request to plan a profile chain transition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ChainRequest<'a> {
    /// Currently active profile identifier.
    pub source_profile_id: &'a str,
    /// Requested target profile identifier.
    pub target_profile_id: &'a str,
    /// Host or user decision for the transition.
    pub decision: ChainDecision,
    /// Optional bounded instruction appended by host/user decision.
    pub extra_instruction: Option<&'a str>,
}

/// Auditable chain continuation plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChainPlan {
    source_profile_id: super::support::Text,
    target_profile_id: super::support::Text,
    active_profile_id: super::support::Text,
    decision: ChainDecision,
    extra_instruction: Option<super::support::Text>,
}

impl ChainPlan {
    /// Plans a chain transition without hidden prompt control flow.
    ///
    /// # Errors
    ///
    /// Returns [`super::types::Error`] when identifiers or extra instruction text
    /// fail adapter bounds.
    pub fn from_request(request: ChainRequest<'_>) -> Result<Self, super::types::Error> {
        let source_profile_id =
            super::support::required_text(super::support::PROFILE_ID_FIELD, request.source_profile_id)?;
        let target_profile_id =
            super::support::required_text(super::support::PROFILE_ID_FIELD, request.target_profile_id)?;
        let extra_instruction = super::support::normalize_extra_instruction(request.extra_instruction)?;
        let active_profile_id = active_profile(ActiveProfileInput {
            source_profile_id: source_profile_id.as_str(),
            target_profile_id: target_profile_id.as_str(),
            decision: request.decision,
        });

        Ok(Self {
            source_profile_id,
            target_profile_id,
            active_profile_id,
            decision: request.decision,
            extra_instruction,
        })
    }

    /// Returns the source profile identifier.
    #[must_use]
    pub fn source_profile_id(&self) -> &str {
        self.source_profile_id.as_str()
    }

    /// Returns the requested target profile identifier.
    #[must_use]
    pub fn target_profile_id(&self) -> &str {
        self.target_profile_id.as_str()
    }

    /// Returns the profile that remains active after decision handling.
    #[must_use]
    pub fn active_profile_id(&self) -> &str {
        self.active_profile_id.as_str()
    }

    /// Returns the host/user decision.
    #[must_use]
    pub const fn decision(&self) -> ChainDecision {
        self.decision
    }

    /// Returns the bounded extra instruction when accepted by policy.
    #[must_use]
    pub fn extra_instruction(&self) -> Option<&str> {
        self.extra_instruction.as_deref()
    }
}

#[derive(Clone, Copy)]
struct ActiveProfileInput<'a> {
    source_profile_id: &'a str,
    target_profile_id: &'a str,
    decision: ChainDecision,
}

fn active_profile(input: ActiveProfileInput<'_>) -> super::support::Text {
    match input.decision {
        ChainDecision::Accepted => super::support::Text::from(input.target_profile_id),
        ChainDecision::Declined | ChainDecision::Unavailable => super::support::Text::from(input.source_profile_id),
    }
}

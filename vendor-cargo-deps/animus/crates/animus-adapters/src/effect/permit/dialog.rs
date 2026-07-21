/// Borrowed fields for a host prompt request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PromptRequestDraft<'a> {
    /// Guarded action class.
    pub action: super::super::GuardedAction,
    /// Safe resource label.
    pub resource_label: &'a str,
    /// Session label that scopes the decision token.
    pub session_label: &'a str,
    /// BLAKE3 hash of the selected permission policy.
    pub policy_hash: &'a str,
    /// Redacted action summary supplied to the host.
    pub redacted_summary: &'a str,
}

/// Host prompt request for ask-mode decisions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PromptRequest {
    action: super::super::GuardedAction,
    resource_label: super::Text,
    session_label: super::Text,
    policy_hash: super::Text,
    redacted_summary: super::Text,
    decision_token: super::Text,
}

impl PromptRequest {
    /// Builds a scoped prompt request with a deterministic decision token.
    ///
    /// # Errors
    ///
    /// Returns [`super::Error`] when labels or summary text are malformed.
    pub fn new(draft: PromptRequestDraft<'_>) -> Result<Self, super::Error> {
        let resource_label =
            super::required_text_value(super::RESOURCE_FIELD, draft.resource_label)?;
        let session_label = super::required_text_value(super::SESSION_FIELD, draft.session_label)?;
        let policy_hash =
            super::required_text_value(super::PROMPT_POLICY_FIELD, draft.policy_hash)?;
        let redacted_summary =
            super::required_text_value(super::PROMPT_SUMMARY_FIELD, draft.redacted_summary)?;
        let decision_token = super::hash_parts(&[
            super::HASH_SCHEMA,
            draft.action.as_str(),
            resource_label.as_str(),
            session_label.as_str(),
            policy_hash.as_str(),
        ]);
        Ok(Self {
            action: draft.action,
            resource_label,
            session_label,
            policy_hash,
            redacted_summary,
            decision_token,
        })
    }

    /// Returns the guarded action class.
    #[must_use]
    pub const fn action(&self) -> super::super::GuardedAction {
        self.action
    }

    /// Returns the safe resource label.
    #[must_use]
    pub fn resource_label(&self) -> &str {
        self.resource_label.as_str()
    }

    /// Returns the session label.
    #[must_use]
    pub fn session_label(&self) -> &str {
        self.session_label.as_str()
    }

    /// Returns the policy hash label.
    #[must_use]
    pub fn policy_hash(&self) -> &str {
        self.policy_hash.as_str()
    }

    /// Returns the redacted action summary.
    #[must_use]
    pub fn redacted_summary(&self) -> &str {
        self.redacted_summary.as_str()
    }

    /// Returns the scoped decision token.
    #[must_use]
    pub fn decision_token(&self) -> &str {
        self.decision_token.as_str()
    }
}

/// Borrowed fields for a host prompt response.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DecisionDraft<'a> {
    /// Guarded action class.
    pub action: super::super::GuardedAction,
    /// Safe resource label.
    pub resource_label: &'a str,
    /// Session label that scoped the decision token.
    pub session_label: &'a str,
    /// BLAKE3 hash of the selected permission policy.
    pub policy_hash: &'a str,
    /// Scoped decision token from [`PromptRequest`].
    pub decision_token: &'a str,
    /// Host prompt outcome.
    pub outcome: DecisionOutcome,
}

/// Host prompt response for ask-mode decisions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Decision {
    pub(super) action: super::super::GuardedAction,
    pub(super) resource_label: super::Text,
    pub(super) session_label: super::Text,
    pub(super) hash: super::Text,
    pub(super) token: super::Text,
    pub(super) outcome: DecisionOutcome,
}

impl Decision {
    /// Builds a host prompt decision response.
    ///
    /// # Errors
    ///
    /// Returns [`super::Error`] when labels or token text are malformed.
    pub fn new(draft: DecisionDraft<'_>) -> Result<Self, super::Error> {
        Ok(Self {
            action: draft.action,
            resource_label: super::required_text_value(
                super::RESOURCE_FIELD,
                draft.resource_label,
            )?,
            session_label: super::required_text_value(super::SESSION_FIELD, draft.session_label)?,
            hash: super::required_text_value(super::PROMPT_POLICY_FIELD, draft.policy_hash)?,
            token: super::required_text_value(super::TOKEN_FIELD, draft.decision_token)?,
            outcome: draft.outcome,
        })
    }
}

/// Prompt decision outcome.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DecisionOutcome {
    /// Host approved the guarded action.
    Approved,
    /// Host denied the guarded action.
    Denied,
}

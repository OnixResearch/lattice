pub(super) mod parse;

/// Borrowed permission policy descriptor before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PolicyDraft<'a> {
    /// Guarded action label.
    pub action: Option<&'a str>,
    /// Safe resource label matched exactly by this local policy.
    pub resource_label: Option<&'a str>,
    /// Local outcome label: allow, ask, or deny.
    pub outcome: Option<&'a str>,
    /// Safe source identity for policy provenance.
    pub source_identity: Option<&'a str>,
    /// Trust scope for policy precedence and audit.
    pub trust_scope: Option<&'a str>,
    /// Trust status supplied by host policy.
    pub trust_status: Option<&'a str>,
    /// Deterministic precedence within the host-selected policy set.
    pub precedence: u16,
    /// Prompt summary policy label.
    pub prompt_summary_policy: Option<&'a str>,
}

/// Validated permission policy descriptor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Policy {
    pub(super) action: super::super::GuardedAction,
    pub(super) resource_label: super::Text,
    pub(super) outcome: Outcome,
    pub(super) source_identity: super::Text,
    pub(super) trust_scope: TrustScope,
    pub(super) trust_status: TrustStatus,
    pub(super) precedence: u16,
    pub(super) prompt_summary: PromptSummaryPolicy,
    pub(super) hash: super::Text,
}

impl Policy {
    /// Validates a host-loaded permission policy descriptor.
    ///
    /// # Errors
    ///
    /// Returns [`super::Error`] when fields are missing, unsupported,
    /// untrusted, malformed, or outside precedence bounds.
    pub fn validate(draft: &PolicyDraft<'_>) -> Result<Self, super::Error> {
        parse::validate(draft)
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

    /// Returns the local permission outcome.
    #[must_use]
    pub const fn outcome(&self) -> Outcome {
        self.outcome
    }

    /// Returns the source identity.
    #[must_use]
    pub fn source_identity(&self) -> &str {
        self.source_identity.as_str()
    }

    /// Returns the trust scope.
    #[must_use]
    pub const fn trust_scope(&self) -> TrustScope {
        self.trust_scope
    }

    /// Returns the trust status.
    #[must_use]
    pub const fn trust_status(&self) -> TrustStatus {
        self.trust_status
    }

    /// Returns policy precedence.
    #[must_use]
    pub const fn precedence(&self) -> u16 {
        self.precedence
    }

    /// Returns the prompt-summary policy.
    #[must_use]
    pub const fn prompt_summary_policy(&self) -> PromptSummaryPolicy {
        self.prompt_summary
    }

    /// Returns the BLAKE3 hash of checked policy fields.
    #[must_use]
    pub fn policy_hash(&self) -> &str {
        self.hash.as_str()
    }

    pub(super) fn matches_request(&self, action: super::super::GuardedAction, resource_label: &str) -> bool {
        self.action == action && self.resource_label == resource_label
    }
}

/// Local permission outcome.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Outcome {
    /// Local policy does not object after authority checks pass.
    Allow,
    /// Local policy requires host confirmation before dispatch.
    Ask,
    /// Local policy denies before dispatch.
    Deny,
}

impl Outcome {
    /// Returns the stable outcome label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Allow => super::ALLOW_OUTCOME_LABEL,
            Self::Ask => super::ASK_OUTCOME_LABEL,
            Self::Deny => super::DENY_OUTCOME_LABEL,
        }
    }
}

/// Permission policy trust scope.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrustScope {
    /// User-level trusted policy.
    User,
    /// Project-level policy admitted by host trust.
    Project,
    /// Session-local policy.
    Session,
    /// Harness default policy.
    Harness,
}

impl TrustScope {
    /// Returns the stable trust-scope label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::User => super::USER_SCOPE_LABEL,
            Self::Project => super::PROJECT_SCOPE_LABEL,
            Self::Session => super::SESSION_SCOPE_LABEL,
            Self::Harness => super::HARNESS_SCOPE_LABEL,
        }
    }
}

/// Permission policy trust status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrustStatus {
    /// Host policy admits this descriptor.
    Trusted,
    /// Host policy has not admitted this descriptor.
    Untrusted,
}

/// Prompt-summary policy for ask decisions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PromptSummaryPolicy {
    /// Host prompt summaries must be redacted bounded labels.
    Redacted,
}

impl PromptSummaryPolicy {
    /// Returns the stable policy label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Redacted => super::REDACTED_SUMMARY_POLICY_LABEL,
        }
    }
}

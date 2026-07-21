/// Pattern matching kind for a local permission descriptor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PatternMatchKind {
    /// Glob-like label matching with an optional trailing wildcard.
    Glob,
    /// Regex-like label matching validated as data; hosts own real regex engines.
    Regex,
}

impl PatternMatchKind {
    /// Returns the stable match-kind label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Glob => GLOB_MATCH_LABEL,
            Self::Regex => REGEX_MATCH_LABEL,
        }
    }
}

/// Borrowed pattern-rule descriptor before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PatternRuleDraft<'a> {
    /// Match kind label, currently glob or regex.
    pub match_kind: Option<&'a str>,
    /// Guarded action label.
    pub action: Option<&'a str>,
    /// Safe resource pattern label.
    pub resource_pattern: Option<&'a str>,
    /// Local outcome label.
    pub outcome: Option<&'a str>,
    /// Safe source identity for provenance.
    pub source_identity: Option<&'a str>,
    /// Trust scope label.
    pub trust_scope: Option<&'a str>,
    /// Trust status label.
    pub trust_status: Option<&'a str>,
    /// Deterministic precedence.
    pub precedence: u16,
}

/// Validated glob-like or regex-like rule descriptor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PatternRule {
    match_kind: PatternMatchKind,
    action: super::super::GuardedAction,
    resource_pattern: super::Text,
    outcome: super::Outcome,
    source_identity: super::Text,
    trust_scope: super::TrustScope,
    trust_status: super::TrustStatus,
    precedence: u16,
    rule_hash: super::Text,
}

impl PatternRule {
    /// Validates a host-loaded pattern descriptor.
    ///
    /// # Errors
    ///
    /// Returns [`super::Error`] when a required field is missing, a label is
    /// unsupported, the descriptor is untrusted, precedence is out of bounds,
    /// or a regex-like pattern is malformed.
    // r[impl agent_harness_runtime.zerostack_permission_kernel.pattern_rules]
    // r[impl agent_harness_runtime.zerostack_permission_kernel.pattern_rules.invalid]
    pub fn validate(draft: &PatternRuleDraft<'_>) -> Result<Self, super::Error> {
        let match_kind_text =
            super::required_text(super::PATTERN_MATCH_KIND_FIELD, draft.match_kind)?;
        let action_text = super::required_text(super::ACTION_FIELD, draft.action)?;
        let outcome_text = super::required_text(super::OUTCOME_FIELD, draft.outcome)?;
        let trust_scope_text = super::required_text(super::TRUST_SCOPE_FIELD, draft.trust_scope)?;
        let trust_status_text =
            super::required_text(super::TRUST_STATUS_FIELD, draft.trust_status)?;
        let match_kind = match_kind_from_label(match_kind_text.as_str())
            .ok_or_else(|| unsupported(super::PATTERN_MATCH_KIND_FIELD, match_kind_text))?;
        let action = action_from_label(action_text.as_str())
            .ok_or_else(|| unsupported(super::ACTION_FIELD, action_text))?;
        let outcome = outcome_from_label(outcome_text.as_str())
            .ok_or_else(|| unsupported(super::OUTCOME_FIELD, outcome_text))?;
        let trust_scope = trust_scope_from_label(trust_scope_text.as_str())
            .ok_or_else(|| unsupported(super::TRUST_SCOPE_FIELD, trust_scope_text))?;
        let trust_status = trust_status_from_label(trust_status_text.as_str())
            .ok_or_else(|| unsupported(super::TRUST_STATUS_FIELD, trust_status_text))?;
        if trust_status == super::TrustStatus::Untrusted {
            return Err(super::Error::UntrustedDescriptor { trust_scope });
        }
        validate_precedence(draft.precedence)?;
        let resource_pattern =
            super::required_text(super::PATTERN_RESOURCE_FIELD, draft.resource_pattern)?;
        validate_pattern(match_kind, resource_pattern.as_str())?;
        Ok(rule_from_fields(RuleFields {
            match_kind,
            action,
            resource_pattern,
            outcome,
            source_identity: super::required_text(super::SOURCE_FIELD, draft.source_identity)?,
            trust_scope,
            trust_status,
            precedence: draft.precedence,
        }))
    }

    /// Returns the pattern match kind.
    #[must_use]
    pub const fn match_kind(&self) -> PatternMatchKind {
        self.match_kind
    }

    /// Returns the guarded action class.
    #[must_use]
    pub const fn action(&self) -> super::super::GuardedAction {
        self.action
    }

    /// Returns the safe resource pattern.
    #[must_use]
    pub fn resource_pattern(&self) -> &str {
        self.resource_pattern.as_str()
    }

    /// Returns the local outcome.
    #[must_use]
    pub const fn outcome(&self) -> super::Outcome {
        self.outcome
    }

    /// Returns the safe source identity.
    #[must_use]
    pub fn source_identity(&self) -> &str {
        self.source_identity.as_str()
    }

    /// Returns the trust scope.
    #[must_use]
    pub const fn trust_scope(&self) -> super::TrustScope {
        self.trust_scope
    }

    /// Returns the trust status.
    #[must_use]
    pub const fn trust_status(&self) -> super::TrustStatus {
        self.trust_status
    }

    /// Returns precedence.
    #[must_use]
    pub const fn precedence(&self) -> u16 {
        self.precedence
    }

    /// Returns the BLAKE3 rule identity.
    #[must_use]
    pub fn rule_hash(&self) -> &str {
        self.rule_hash.as_str()
    }

    /// Returns true when this validated descriptor matches the action/resource.
    #[must_use]
    pub fn matches(&self, action: super::super::GuardedAction, resource_label: &str) -> bool {
        self.action == action
            && pattern_matches(PatternMatchInput {
                kind: self.match_kind,
                pattern: self.resource_pattern(),
                resource_label,
            })
    }
}

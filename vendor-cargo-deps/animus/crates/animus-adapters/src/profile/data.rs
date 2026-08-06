/// Borrowed host-supplied policy/profile descriptor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Draft<'a> {
    /// Policy kind label.
    pub kind: Option<&'a str>,
    /// Source scope label.
    pub scope: Option<&'a str>,
    /// Trust status label.
    pub trust: Option<&'a str>,
    /// Deterministic priority within a scope.
    pub priority: u16,
    /// Goal-first policy text.
    pub goal: Option<&'a str>,
    /// Observable success criteria.
    pub success_criteria: &'a [&'a str],
    /// Safety or side-effect constraints.
    pub constraints: &'a [&'a str],
    /// Required validation rules.
    pub validation_rules: &'a [&'a str],
    /// Conditions that require stopping or asking the host/user.
    pub stop_rules: &'a [&'a str],
    /// Safe policy source/provenance label.
    pub provenance: Option<&'a str>,
    /// Optional attributed prompt snippets.
    pub prompt_snippets: &'a [&'a str],
}

/// Validated policy/profile descriptor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Descriptor {
    pub(super) kind: Kind,
    pub(super) scope: SourceScope,
    pub(super) trust: TrustStatus,
    pub(super) priority: u16,
    pub(super) goal: super::Text,
    pub(super) success_criteria: super::Collection<super::Text>,
    pub(super) constraints: super::Collection<super::Text>,
    pub(super) validation_rules: super::Collection<super::Text>,
    pub(super) stop_rules: super::Collection<super::Text>,
    pub(super) provenance: super::Text,
    pub(super) prompt_snippets: super::Collection<super::Text>,
}

impl Descriptor {
    /// Validates a host-supplied policy/profile descriptor as data.
    ///
    /// # Errors
    ///
    /// Returns [`super::Error`] when required fields are missing, labels are
    /// unsupported, priority or prompt budgets are out of bounds, or prompt
    /// hygiene checks fail.
    pub fn validate(draft: &Draft<'_>) -> Result<Self, super::Error> {
        let kind_text = super::validate::required_text(super::KIND_FIELD, draft.kind)?;
        let scope_text = super::validate::required_text(super::SCOPE_FIELD, draft.scope)?;
        let trust_text = super::validate::required_text(super::TRUST_FIELD, draft.trust)?;
        let goal = super::validate::required_text(super::GOAL_FIELD, draft.goal)?;
        let provenance = super::validate::required_text(super::PROVENANCE_FIELD, draft.provenance)?;
        let kind = super::validate::parse_kind(kind_text.as_str()).ok_or_else(|| super::Error::UnsupportedValue {
            field: super::KIND_FIELD.as_str(),
            value: kind_text,
        })?;
        let scope =
            super::validate::parse_scope(scope_text.as_str()).ok_or_else(|| super::Error::UnsupportedValue {
                field: super::SCOPE_FIELD.as_str(),
                value: scope_text,
            })?;
        let trust =
            super::validate::parse_trust(trust_text.as_str()).ok_or_else(|| super::Error::UnsupportedValue {
                field: super::TRUST_FIELD.as_str(),
                value: trust_text,
            })?;
        super::validate::priority(draft.priority)?;
        let success_criteria = super::validate::required_collection(super::SUCCESS_FIELD, draft.success_criteria)?;
        let constraints = super::validate::optional_collection(super::CONSTRAINT_FIELD, draft.constraints)?;
        let validation_rules = super::validate::required_collection(super::VALIDATION_FIELD, draft.validation_rules)?;
        let stop_rules = super::validate::required_collection(super::STOP_FIELD, draft.stop_rules)?;
        let prompt_snippets = super::validate::optional_collection(super::SNIPPET_FIELD, draft.prompt_snippets)?;
        let profile = Self {
            kind,
            scope,
            trust,
            priority: draft.priority,
            goal,
            success_criteria,
            constraints,
            validation_rules,
            stop_rules,
            provenance,
            prompt_snippets,
        };
        profile.validate_prompt_hygiene()?;
        Ok(profile)
    }

    /// Returns the policy kind.
    #[must_use]
    pub const fn kind(&self) -> Kind {
        self.kind
    }

    /// Returns the policy source scope.
    #[must_use]
    pub const fn scope(&self) -> SourceScope {
        self.scope
    }

    /// Returns the trust status.
    #[must_use]
    pub const fn trust(&self) -> TrustStatus {
        self.trust
    }

    /// Returns deterministic priority within the source scope.
    #[must_use]
    pub const fn priority(&self) -> u16 {
        self.priority
    }

    /// Returns the goal text.
    #[must_use]
    pub fn goal(&self) -> &str {
        self.goal.as_str()
    }

    /// Returns success criteria.
    #[must_use]
    pub fn success_criteria(&self) -> &[super::Text] {
        self.success_criteria.as_slice()
    }

    /// Returns constraints.
    #[must_use]
    pub fn constraints(&self) -> &[super::Text] {
        self.constraints.as_slice()
    }

    /// Returns validation rules.
    #[must_use]
    pub fn validation_rules(&self) -> &[super::Text] {
        self.validation_rules.as_slice()
    }

    /// Returns stop rules.
    #[must_use]
    pub fn stop_rules(&self) -> &[super::Text] {
        self.stop_rules.as_slice()
    }

    /// Returns safe provenance label.
    #[must_use]
    pub fn provenance(&self) -> &str {
        self.provenance.as_str()
    }

    /// Returns optional prompt snippets.
    #[must_use]
    pub fn prompt_snippets(&self) -> &[super::Text] {
        self.prompt_snippets.as_slice()
    }

    /// Returns the policy/enforcement boundary marker.
    #[must_use]
    pub const fn enforcement_boundary(&self) -> super::EnforcementBoundary {
        match self.scope {
            SourceScope::Global | SourceScope::Session | SourceScope::Project | SourceScope::Tool => {
                super::EnforcementBoundary::GuidanceOnly
            }
        }
    }

    pub(super) fn precedence_rank(&self) -> u8 {
        match self.scope {
            SourceScope::Global => super::GLOBAL_SCOPE_RANK,
            SourceScope::Session => super::SESSION_SCOPE_RANK,
            SourceScope::Project => super::PROJECT_SCOPE_RANK,
            SourceScope::Tool => super::TOOL_SCOPE_RANK,
        }
    }

    pub(super) fn validates_as_read_only_guidance(&self) -> bool {
        self.constraints.iter().any(|constraint| constraint == super::READ_ONLY_CONSTRAINT)
    }

    fn validate_prompt_hygiene(&self) -> Result<(), super::Error> {
        let mut prompt_chars = self.goal.chars().count();
        for collection in [
            self.success_criteria.as_slice(),
            self.constraints.as_slice(),
            self.validation_rules.as_slice(),
            self.stop_rules.as_slice(),
            self.prompt_snippets.as_slice(),
        ] {
            for value in collection {
                super::validate::reject_hygiene_markers(value)?;
                prompt_chars = prompt_chars.saturating_add(value.chars().count());
            }
        }
        super::validate::reject_hygiene_markers(&self.goal)?;
        if prompt_chars > super::MAX_POLICY_PROMPT_CHARS {
            return Err(super::Error::PromptHygiene {
                issue: super::PromptHygieneIssue::PromptTooLong,
            });
        }
        Ok(())
    }
}

/// Policy descriptor kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Kind {
    /// Global operating posture.
    GlobalPosture,
    /// Project-local overlay.
    ProjectOverlay,
    /// Tool- or capability-scoped guidance.
    ToolGuidance,
}

/// Source scope for a policy descriptor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourceScope {
    /// Global host or harness policy.
    Global,
    /// Session-scoped policy.
    Session,
    /// Project-local policy.
    Project,
    /// Tool-specific policy.
    Tool,
}

/// Trust status for a policy descriptor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrustStatus {
    /// Host has marked the profile trusted.
    Trusted,
    /// Host has not admitted the profile.
    Untrusted,
}

/// Action posture requested by a profile.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Mode {
    /// Ask a focused question without side effects.
    Ask,
    /// Brainstorm alternatives without changing state.
    Brainstorm,
    /// Produce an implementation plan.
    Plan,
    /// Implement code or configuration changes.
    Code,
    /// Inspect, diagnose, or debug existing behavior.
    Debug,
    /// Review changes or artifacts.
    Review,
    /// Simplify existing content or design.
    Simplify,
    /// Write or refine prompt/profile text.
    WritePrompt,
}

impl Mode {
    /// Returns the stable lowercase label for this mode.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Ask => "ask",
            Self::Brainstorm => "brainstorm",
            Self::Plan => "plan",
            Self::Code => "code",
            Self::Debug => "debug",
            Self::Review => "review",
            Self::Simplify => "simplify",
            Self::WritePrompt => "write-prompt",
        }
    }
}

/// Capability class a profile may request as posture metadata.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Capability {
    /// Read-only local context capability.
    Read,
    /// Workspace write capability.
    Write,
    /// Process execution capability.
    Exec,
    /// Network access capability.
    Network,
    /// Model-provider access capability.
    Model,
    /// Tool invocation capability.
    Tool,
}

/// Trust boundary that supplied a profile.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrustScope {
    /// Built into the harness or shipped with trusted host code.
    Harness,
    /// Supplied explicitly by the current user.
    User,
    /// Loaded from a project-local source.
    Project,
    /// Loaded from an external or remote source.
    External,
}

/// Host trust decision for a profile source.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourceTrust {
    /// The host admitted the profile source as trusted guidance.
    Trusted,
    /// The source is not trusted for model-visible prompt injection.
    Untrusted,
}

/// Prompt inclusion policy carried by a profile descriptor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InclusionPolicy {
    /// Include only after the source is trusted.
    IncludeWhenTrusted,
    /// Withhold until a separate host admission step completes.
    WithholdPendingAdmission,
    /// Never include this profile body in model-visible prompt text.
    NeverInclude,
}

/// Draft data for constructing a validated profile descriptor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DescriptorDraft<'a> {
    /// Stable profile identifier.
    pub profile_id: &'a str,
    /// Human-readable reason this profile exists.
    pub purpose: &'a str,
    /// Default action posture requested by this profile.
    pub action_posture: Mode,
    /// Capability posture labels requested by this profile.
    pub required_capabilities: &'a [Capability],
    /// Safe source identity label.
    pub source_identity: &'a str,
    /// Trust boundary that supplied the source.
    pub trust_scope: TrustScope,
    /// Host trust decision for the source.
    pub source_trust: SourceTrust,
    /// Deterministic ordering key for profile composition.
    pub precedence: u16,
    /// Profile body text before directive stripping.
    pub body: &'a str,
    /// Policy governing model-visible inclusion.
    pub inclusion_policy: InclusionPolicy,
}

/// Validated profile descriptor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Descriptor {
    profile_id: super::support::Text,
    purpose: super::support::Text,
    action_posture: Mode,
    required_capabilities: super::support::Collection<Capability>,
    source_identity: super::support::Text,
    trust_scope: TrustScope,
    source_trust: SourceTrust,
    precedence: u16,
    body: super::support::Text,
    inclusion_policy: InclusionPolicy,
}

impl Descriptor {
    /// Validates and constructs a profile descriptor.
    ///
    /// # Errors
    ///
    /// Returns [`Error`] when a required field is empty, too long,
    /// malformed, or the descriptor exceeds a bounded contract.
    pub fn from_draft(draft: DescriptorDraft<'_>) -> Result<Self, Error> {
        super::support::validate_precedence(draft.precedence)?;
        super::support::validate_capabilities(draft.required_capabilities)?;
        super::support::validate_body(draft.body)?;
        Ok(Self {
            profile_id: super::support::required_text(super::support::PROFILE_ID_FIELD, draft.profile_id)?,
            purpose: super::support::required_text(super::support::PROFILE_PURPOSE_FIELD, draft.purpose)?,
            action_posture: draft.action_posture,
            required_capabilities: draft.required_capabilities.to_vec(),
            source_identity: super::support::required_text(
                super::support::PROFILE_SOURCE_FIELD,
                draft.source_identity,
            )?,
            trust_scope: draft.trust_scope,
            source_trust: draft.source_trust,
            precedence: draft.precedence,
            body: super::support::Text::from(draft.body.trim()),
            inclusion_policy: draft.inclusion_policy,
        })
    }

    /// Returns the stable profile identifier.
    #[must_use]
    pub fn profile_id(&self) -> &str {
        self.profile_id.as_str()
    }

    /// Returns the profile purpose text.
    #[must_use]
    pub fn purpose(&self) -> &str {
        self.purpose.as_str()
    }

    /// Returns the action posture requested by the profile.
    #[must_use]
    pub const fn action_posture(&self) -> Mode {
        self.action_posture
    }

    /// Returns requested capability posture labels.
    #[must_use]
    pub fn required_capabilities(&self) -> &[Capability] {
        self.required_capabilities.as_slice()
    }

    /// Returns the safe source identity label.
    #[must_use]
    pub fn source_identity(&self) -> &str {
        self.source_identity.as_str()
    }

    /// Returns the source trust scope.
    #[must_use]
    pub const fn trust_scope(&self) -> TrustScope {
        self.trust_scope
    }

    /// Returns the source trust decision.
    #[must_use]
    pub const fn source_trust(&self) -> SourceTrust {
        self.source_trust
    }

    /// Returns the deterministic precedence value.
    #[must_use]
    pub const fn precedence(&self) -> u16 {
        self.precedence
    }

    /// Returns raw profile body text before directive stripping.
    #[must_use]
    pub fn body(&self) -> &str {
        self.body.as_str()
    }

    /// Returns the prompt inclusion policy.
    #[must_use]
    pub const fn inclusion_policy(&self) -> InclusionPolicy {
        self.inclusion_policy
    }

    pub(super) fn body_hash(&self) -> super::support::Text {
        super::support::profile_body_hash(super::support::BodyHashInput {
            profile_id: self.profile_id(),
            source_identity: self.source_identity(),
            body: self.body(),
        })
    }
}

/// Error raised by profile validation or planning.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    /// A generic adapter field contract failed.
    Contract(crate::validation::AdapterContractError),
    /// The descriptor precedence was outside the supported range.
    PrecedenceOutOfRange {
        /// Minimum supported precedence.
        min: u16,
        /// Maximum supported precedence.
        max: u16,
        /// Actual precedence.
        actual: u16,
    },
    /// A required capability collection was empty.
    MissingCapabilities,
    /// A body exceeded the prompt-profile body budget.
    BodyTooLong {
        /// Maximum supported body size in Unicode scalar values.
        max_chars: usize,
        /// Actual body size in Unicode scalar values.
        actual_chars: usize,
    },
    /// A bounded chain instruction exceeded the supported budget.
    ExtraInstructionTooLong {
        /// Maximum supported instruction size in Unicode scalar values.
        max_chars: usize,
        /// Actual instruction size in Unicode scalar values.
        actual_chars: usize,
    },
}

impl From<crate::validation::AdapterContractError> for Error {
    fn from(error: crate::validation::AdapterContractError) -> Self {
        Self::Contract(error)
    }
}

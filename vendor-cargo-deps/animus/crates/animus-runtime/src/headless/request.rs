mod continuation;
mod mode;
mod scope;

pub use continuation::ThreadResume;
pub use continuation::ThreadResumeDraft;
pub use mode::NoninteractiveBehavior;
pub use mode::OutputPreference;
pub use mode::SandboxProfile;
pub use scope::RunIdentity;

/// Borrowed non-interactive runtime request fields before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RunRequestDraft<'a> {
    /// Stable objective for the headless run.
    pub objective: &'a str,
    /// Optional session identity supplied by the host.
    pub session_id: &'a str,
    /// Optional workspace identity supplied by the host.
    pub workspace_id: &'a str,
    /// Input message or stdin payload references, never raw private payloads.
    pub input_references: &'a [&'a str],
    /// User or controller prompt for the run.
    pub prompt: &'a str,
    /// Optional external context handles, not raw context bodies.
    pub external_context_handles: &'a [&'a str],
    /// Policy profile label selected before runtime execution.
    pub policy_profile: &'a str,
    /// Capability labels allowed by the selected policy profile.
    pub allowed_capabilities: &'a [&'a str],
    /// Sandbox profile label.
    pub sandbox_profile: &'a str,
    /// Output mode label.
    pub output_mode: &'a str,
    /// Schema identity required when the output mode is structured.
    pub structured_schema_id: Option<&'a str>,
    /// Validation plan identity or label.
    pub validation_plan: &'a str,
    /// Noninteractive behavior label.
    pub noninteractive_behavior: &'a str,
    /// Optional resume identity.
    pub resume: Option<ThreadResumeDraft<'a>>,
}

/// Validated non-interactive runtime request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RunRequest {
    objective: super::Text,
    identity: RunIdentity,
    input_references: super::Collection<super::Text>,
    prompt: super::Text,
    external_context_handles: super::Collection<super::Text>,
    policy_profile: super::Text,
    allowed_capabilities: super::Collection<super::Text>,
    sandbox_profile: SandboxProfile,
    output_preference: OutputPreference,
    validation_plan: super::Text,
    noninteractive_behavior: NoninteractiveBehavior,
    resume: Option<ThreadResume>,
}

impl RunRequest {
    /// Validates a non-interactive runtime request without starting adapters.
    ///
    /// # Errors
    ///
    /// Returns [`super::AutomationError`] when objective, identity, input
    /// references, prompt, policy profile, capability set, output contract,
    /// validation plan, noninteractive behavior, context handles, sandbox
    /// profile, or resume facts are malformed.
    pub fn validate(draft: &RunRequestDraft<'_>) -> Result<Self, super::AutomationError> {
        let objective = super::required_text(super::TextField {
            field: super::OBJECTIVE_FIELD,
            value: draft.objective,
        })?;
        let identity = RunIdentity::validate(draft.session_id, draft.workspace_id)?;
        let input_references = super::required_unique_texts(super::INPUT_REFERENCE_FIELD, draft.input_references)?;
        let prompt = super::required_text(super::TextField {
            field: super::PROMPT_FIELD,
            value: draft.prompt,
        })?;
        let external_context_handles = super::unique_texts(super::CONTEXT_FIELD, draft.external_context_handles)?;
        let policy_profile = super::required_text(super::TextField {
            field: super::POLICY_PROFILE_FIELD,
            value: draft.policy_profile,
        })?;
        let allowed_capabilities = super::required_unique_texts(super::CAPABILITY_FIELD, draft.allowed_capabilities)?;
        let sandbox_profile = SandboxProfile::parse(draft.sandbox_profile).ok_or_else(|| {
            super::unsupported(super::TextField {
                field: super::SANDBOX_FIELD,
                value: draft.sandbox_profile,
            })
        })?;
        let output_preference = OutputPreference::from_draft(draft.output_mode, draft.structured_schema_id)?;
        let validation_plan = super::required_text(super::TextField {
            field: super::VALIDATION_PLAN_FIELD,
            value: draft.validation_plan,
        })?;
        let noninteractive_behavior =
            NoninteractiveBehavior::parse(draft.noninteractive_behavior).ok_or_else(|| {
                super::unsupported(super::TextField {
                    field: super::NONINTERACTIVE_BEHAVIOR_FIELD,
                    value: draft.noninteractive_behavior,
                })
            })?;
        let resume = draft.resume.as_ref().map(ThreadResume::validate).transpose()?;
        Ok(Self {
            objective,
            identity,
            input_references,
            prompt,
            external_context_handles,
            policy_profile,
            allowed_capabilities,
            sandbox_profile,
            output_preference,
            validation_plan,
            noninteractive_behavior,
            resume,
        })
    }

    /// Returns the stable objective.
    #[must_use]
    pub fn objective(&self) -> &str {
        self.objective.as_str()
    }

    /// Returns the validated run identity.
    #[must_use]
    pub const fn identity(&self) -> &RunIdentity {
        &self.identity
    }

    /// Returns input references in caller order.
    #[must_use]
    pub fn input_references(&self) -> &[super::Text] {
        self.input_references.as_slice()
    }

    /// Returns the validated prompt text.
    #[must_use]
    pub fn prompt(&self) -> &str {
        self.prompt.as_str()
    }

    /// Returns external context handles in caller order.
    #[must_use]
    pub fn external_context_handles(&self) -> &[super::Text] {
        self.external_context_handles.as_slice()
    }

    /// Returns the selected policy profile.
    #[must_use]
    pub fn policy_profile(&self) -> &str {
        self.policy_profile.as_str()
    }

    /// Returns allowed capability labels in caller order.
    #[must_use]
    pub fn allowed_capabilities(&self) -> &[super::Text] {
        self.allowed_capabilities.as_slice()
    }

    /// Returns the sandbox profile.
    #[must_use]
    pub const fn sandbox_profile(&self) -> SandboxProfile {
        self.sandbox_profile
    }

    /// Returns the output preference.
    #[must_use]
    pub const fn output_preference(&self) -> &OutputPreference {
        &self.output_preference
    }

    /// Returns the validation plan label.
    #[must_use]
    pub fn validation_plan(&self) -> &str {
        self.validation_plan.as_str()
    }

    /// Returns the noninteractive behavior.
    #[must_use]
    pub const fn noninteractive_behavior(&self) -> NoninteractiveBehavior {
        self.noninteractive_behavior
    }

    /// Returns optional resume identity.
    #[must_use]
    pub const fn resume(&self) -> Option<&ThreadResume> {
        self.resume.as_ref()
    }
}

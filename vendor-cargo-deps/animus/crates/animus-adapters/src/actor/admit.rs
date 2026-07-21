pub(super) mod core;

/// Host policy admission for project/workspace agent definitions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Admission<'a> {
    /// Manifest identities explicitly admitted by host confirmation or policy.
    pub admitted_manifest_identities: &'a [&'a str],
}

/// Active capability policy used to clamp agent-definition requests.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ActivationPolicy<'a> {
    /// Model-role labels authorized by runtime policy.
    pub model_roles: &'a [&'a str],
    /// Capability labels authorized by runtime policy.
    pub capabilities: &'a [&'a str],
    /// Toolset labels authorized by runtime policy.
    pub tools: &'a [&'a str],
    /// Memory-scope labels authorized by runtime policy.
    pub memory_scopes: &'a [&'a str],
    /// Context-reference labels authorized by runtime policy.
    pub context_reference_scopes: &'a [&'a str],
    /// Workflow ability labels authorized by runtime policy.
    pub workflow_abilities: &'a [&'a str],
}

/// Activation request evaluated by the pure core.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ActivationRequest<'a> {
    /// Manifest selected by discovery.
    pub manifest: &'a super::Manifest,
    /// Host admission state for project/workspace definitions.
    pub admission: Admission<'a>,
    /// Active runtime policy ceilings.
    pub policy: ActivationPolicy<'a>,
}

/// Result of evaluating one agent-definition activation request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Activation {
    pub(super) status: ActivationStatus,
    pub(super) admission_status: AdmissionStatus,
    pub(super) issues: super::Collection<ActivationIssue>,
    pub(super) prompt_fragments: super::Collection<super::Text>,
    pub(super) selected_model_role: Option<super::Text>,
    pub(super) effective_capabilities: super::Collection<super::Text>,
    pub(super) effective_tools: super::Collection<super::Text>,
    pub(super) effective_memory_scopes: super::Collection<super::Text>,
    pub(super) effective_context_reference_scopes: super::Collection<super::Text>,
    pub(super) effective_workflow_abilities: super::Collection<super::Text>,
    pub(super) denied_escalations: super::Collection<DeniedEscalation>,
}

impl Activation {
    /// Returns whether activation can influence prompt assembly and requests.
    #[must_use]
    pub const fn status(&self) -> ActivationStatus {
        self.status
    }

    /// Returns the project admission status recorded for receipts.
    #[must_use]
    pub const fn admission_status(&self) -> AdmissionStatus {
        self.admission_status
    }

    /// Returns activation issues.
    #[must_use]
    pub fn issues(&self) -> &[ActivationIssue] {
        self.issues.as_slice()
    }

    /// Returns prompt fragments admitted for prompt assembly.
    #[must_use]
    pub fn prompt_fragments(&self) -> &[super::Text] {
        self.prompt_fragments.as_slice()
    }

    /// Returns the policy-admitted model role, if any.
    #[must_use]
    pub fn selected_model_role(&self) -> Option<&str> {
        self.selected_model_role.as_deref()
    }

    /// Returns effective capability ceilings admitted by policy.
    #[must_use]
    pub fn effective_capabilities(&self) -> &[super::Text] {
        self.effective_capabilities.as_slice()
    }

    /// Returns effective tool ceilings admitted by policy.
    #[must_use]
    pub fn effective_tools(&self) -> &[super::Text] {
        self.effective_tools.as_slice()
    }

    /// Returns effective memory-scope ceilings admitted by policy.
    #[must_use]
    pub fn effective_memory_scopes(&self) -> &[super::Text] {
        self.effective_memory_scopes.as_slice()
    }

    /// Returns effective context-reference ceilings admitted by policy.
    #[must_use]
    pub fn effective_context_reference_scopes(&self) -> &[super::Text] {
        self.effective_context_reference_scopes.as_slice()
    }

    /// Returns effective workflow-ability ceilings admitted by policy.
    #[must_use]
    pub fn effective_workflow_abilities(&self) -> &[super::Text] {
        self.effective_workflow_abilities.as_slice()
    }

    /// Returns escalation requests denied by policy.
    #[must_use]
    pub fn denied_escalations(&self) -> &[DeniedEscalation] {
        self.denied_escalations.as_slice()
    }
}

/// Activation status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActivationStatus {
    /// Activation may influence prompt assembly and request ceilings.
    Active,
    /// Activation failed closed and has no prompt or authority influence.
    Denied,
}

/// Admission status captured in activation receipts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdmissionStatus {
    /// Source does not require project admission.
    NotRequired,
    /// Required project admission was present.
    Admitted,
    /// Required project admission was missing.
    Denied,
}

impl AdmissionStatus {
    /// Returns the stable receipt label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NotRequired => "not-required",
            Self::Admitted => "admitted",
            Self::Denied => "denied",
        }
    }
}

/// Activation issue classes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActivationIssue {
    /// Project/workspace definition lacked host confirmation or policy admission.
    ProjectAdmissionRequired,
    /// Manifest requested authority outside active policy ceilings.
    CapabilityEscalationDenied,
}

/// Requested ability denied during activation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeniedEscalation {
    pub(super) kind: AbilityKind,
    pub(super) value: super::Text,
}

impl DeniedEscalation {
    /// Returns the denied ability kind.
    #[must_use]
    pub const fn kind(&self) -> AbilityKind {
        self.kind
    }

    /// Returns the denied ability label.
    #[must_use]
    pub fn value(&self) -> &str {
        self.value.as_str()
    }
}

/// Ability classes interpreted as ceilings or requests.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AbilityKind {
    /// Model-role preference.
    ModelRole,
    /// Capability label.
    Capability,
    /// Toolset label.
    Tool,
    /// Memory-scope label.
    Memory,
    /// Context-reference scope label.
    ContextReference,
    /// Workflow ability label.
    Workflow,
}

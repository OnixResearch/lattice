/// Matrix axes used to evaluate MCP Apps-style UI metadata.
pub const MCP_APP_EVALUATION_AXES: &[EvaluationAxis] = &[
    EvaluationAxis::ResourceIdentity,
    EvaluationAxis::MimeProfile,
    EvaluationAxis::CspDomains,
    EvaluationAxis::PermissionRequests,
    EvaluationAxis::HostContextSync,
    EvaluationAxis::CallbackChannels,
    EvaluationAxis::Sizing,
    EvaluationAxis::LifecycleBounds,
    EvaluationAxis::AdoptionRisk,
];

/// Evaluation axes required for MCP Apps-style surface review.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvaluationAxis {
    /// UI resource identity is named without loading the resource.
    ResourceIdentity,
    /// MIME/profile tags are recorded as data.
    MimeProfile,
    /// Content security policy domains are classified.
    CspDomains,
    /// Browser or device permission requests are classified.
    PermissionRequests,
    /// Host theme/context synchronization labels are classified.
    HostContextSync,
    /// Callback channel labels are classified.
    CallbackChannels,
    /// Preferred sizing policy is classified.
    Sizing,
    /// Lifecycle and callback bounds are classified.
    LifecycleBounds,
    /// Adoption decision and risk reason are recorded.
    AdoptionRisk,
}

/// Final data-only adoption classification for a descriptor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdoptionDecision {
    /// Descriptor is safe as data-only metadata.
    DataOnlyAdoptable,
    /// Descriptor needs explicit host policy before future adoption.
    NeedsHostPolicy,
    /// Descriptor is rejected for default future behavior.
    RejectDefault,
}

/// Risk reason recorded with an adoption decision.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RiskReason {
    /// Descriptor contains safe metadata only.
    SafeMetadata,
    /// Remote domains require explicit host policy.
    RemoteDomainRequiresAdmission,
    /// Browser permissions require explicit host policy.
    BrowserPermissionRequiresAdmission,
    /// Host callbacks require explicit host policy.
    HostCallbackRequiresAdmission,
    /// Descriptor includes a wildcard or insecure CSP domain.
    BroadDomain,
    /// Descriptor requests sensitive device or clipboard permissions without policy admission.
    BroadPermission,
    /// Descriptor declares callbacks without lifecycle bounds.
    UnboundedCallback,
    /// Descriptor asks for ambient host control.
    AmbientHostControl,
    /// Descriptor asks to inject direct model-visible chat messages.
    DirectMessageInjection,
    /// Descriptor claims validation should execute UI runtime behavior.
    RuntimeExecutionClaim,
}

/// Data-only MCP Apps evaluation report.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Evaluation {
    pub(super) descriptor: super::Descriptor,
    pub(super) decision: Decision,
}

impl Evaluation {
    /// Returns the validated descriptor.
    #[must_use]
    pub const fn descriptor(&self) -> &super::Descriptor {
        &self.descriptor
    }

    /// Returns the adoption decision.
    #[must_use]
    pub const fn adoption_decision(&self) -> AdoptionDecision {
        self.decision.adoption_decision
    }

    /// Returns the risk reason for the adoption decision.
    #[must_use]
    pub const fn risk_reason(&self) -> RiskReason {
        self.decision.risk_reason
    }

    /// Returns the explicit matrix axes applied to evaluations.
    #[must_use]
    pub const fn covered_axes() -> &'static [EvaluationAxis] {
        MCP_APP_EVALUATION_AXES
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct Decision {
    pub(super) adoption_decision: AdoptionDecision,
    pub(super) risk_reason: RiskReason,
}

/// Errors raised while validating or classifying MCP Apps-style descriptors.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EvaluationError {
    /// Generic adapter field contract error.
    Contract(crate::AdapterContractError),
    /// Descriptor value was not one of the supported labels.
    UnsupportedValue {
        /// Boundary field name.
        field: &'static str,
        /// Unsupported normalized value.
        value: super::Text,
    },
    /// Descriptor collection exceeded its bounded ceiling.
    TooManyItems {
        /// Boundary collection field name.
        field: &'static str,
        /// Maximum supported item count.
        max_items: usize,
        /// Actual item count.
        actual_items: usize,
    },
}

impl From<crate::AdapterContractError> for EvaluationError {
    fn from(error: crate::AdapterContractError) -> Self {
        Self::Contract(error)
    }
}

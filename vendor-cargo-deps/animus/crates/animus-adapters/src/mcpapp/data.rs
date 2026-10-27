/// Admission state for a future host-policy gate.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PolicyAdmission {
    /// Host policy has not admitted the behavior.
    #[default]
    Denied,
    /// Host policy explicitly admits the behavior.
    Admitted,
}

impl PolicyAdmission {
    pub(super) const fn is_admitted(self) -> bool {
        matches!(self, Self::Admitted)
    }
}

/// Borrowed policy-admission flags for evaluating UI metadata.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PolicyAdmissionDraft {
    /// Whether remote resource domains are admitted by host policy.
    pub remote_domains: PolicyAdmission,
    /// Whether browser or device permissions are admitted by host policy.
    pub browser_permissions: PolicyAdmission,
    /// Whether bounded host callbacks are admitted by host policy.
    pub host_callbacks: PolicyAdmission,
    /// Whether direct model-visible chat callbacks are admitted by host policy.
    pub direct_chat_messages: PolicyAdmission,
}

/// Borrowed MCP Apps-style UI metadata before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DescriptorDraft<'a> {
    /// Optional UI resource URI or stable resource identifier.
    pub resource_uri: Option<&'a str>,
    /// Optional MIME/profile tag for the resource.
    pub mime_profile: Option<&'a str>,
    /// Optional human-facing title.
    pub title: Option<&'a str>,
    /// Content security policy domain labels.
    pub csp_domains: &'a [&'a str],
    /// Browser or device permission request labels.
    pub permission_requests: &'a [&'a str],
    /// Host-context synchronization labels.
    pub host_context_sync: &'a [&'a str],
    /// UI-to-host callback channel labels.
    pub callback_channels: &'a [&'a str],
    /// Optional preferred sizing policy.
    pub size_policy: Option<&'a str>,
    /// Optional lifecycle/callback bound policy.
    pub lifecycle_policy: Option<&'a str>,
    /// Explicit policy admissions used for classification.
    pub policy_admission: PolicyAdmissionDraft,
    /// Whether the descriptor claims validation should execute UI behavior.
    pub claims_runtime_execution: bool,
}

/// Validated data-only UI metadata descriptor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Descriptor {
    pub(super) resource_uri: super::Text,
    pub(super) mime_profile: super::Text,
    pub(super) title: super::Text,
    pub(super) csp_domains: super::Collection<super::Text>,
    pub(super) permission_requests: super::Collection<super::Text>,
    pub(super) host_context_sync: super::Collection<super::Text>,
    pub(super) callback_channels: super::Collection<super::Text>,
    pub(super) size_policy: SizePolicy,
    pub(super) lifecycle_policy: LifecyclePolicy,
    pub(super) policy_admission: PolicyAdmissionDraft,
    pub(super) claims_runtime_execution: bool,
}

impl Descriptor {
    /// Validates a descriptor and returns its data-only evaluation classification.
    ///
    /// # Errors
    ///
    /// Returns [`super::EvaluationError`] when required fields are missing, field text is
    /// malformed, collection ceilings are exceeded, or size/lifecycle labels are
    /// unsupported.
    pub fn evaluate(
        draft: &DescriptorDraft<'_>,
    ) -> Result<super::Evaluation, super::EvaluationError> {
        let descriptor = Self::validate(draft)?;
        let decision = super::classify::descriptor(&descriptor);
        Ok(super::Evaluation {
            descriptor,
            decision,
        })
    }

    /// Validates descriptor-shaped metadata without executing UI behavior.
    ///
    /// # Errors
    ///
    /// Returns [`super::EvaluationError`] when required fields are missing, field text is
    /// malformed, collection ceilings are exceeded, or size/lifecycle labels are
    /// unsupported.
    pub fn validate(draft: &DescriptorDraft<'_>) -> Result<Self, super::EvaluationError> {
        let resource_uri =
            super::norm::required_text(super::RESOURCE_URI_FIELD, draft.resource_uri)?;
        let mime_profile =
            super::norm::required_text(super::MIME_PROFILE_FIELD, draft.mime_profile)?;
        let title = super::norm::required_text(super::TITLE_FIELD, draft.title)?;
        let csp_domains = super::norm::normalize_collection(
            super::CSP_DOMAIN_FIELD,
            draft.csp_domains,
            super::MAX_CSP_DOMAIN_ITEMS,
        )?;
        let permission_requests = super::norm::normalize_collection(
            super::PERMISSION_FIELD,
            draft.permission_requests,
            super::MAX_PERMISSION_ITEMS,
        )?;
        let host_context_sync = super::norm::normalize_collection(
            super::HOST_CONTEXT_FIELD,
            draft.host_context_sync,
            super::MAX_HOST_CONTEXT_ITEMS,
        )?;
        let callback_channels = super::norm::normalize_collection(
            super::CALLBACK_FIELD,
            draft.callback_channels,
            super::MAX_CALLBACK_ITEMS,
        )?;
        let dimension_policy_label =
            super::norm::required_text(super::SIZE_POLICY_FIELD, draft.size_policy)?;
        let lifecycle_policy_label =
            super::norm::required_text(super::LIFECYCLE_POLICY_FIELD, draft.lifecycle_policy)?;
        let dimension_policy = super::norm::parse_size_policy(dimension_policy_label.as_str())
            .ok_or_else(|| super::EvaluationError::UnsupportedValue {
                field: super::SIZE_POLICY_FIELD.as_str(),
                value: dimension_policy_label.clone(),
            })?;
        let lifecycle_policy = super::norm::parse_lifecycle_policy(lifecycle_policy_label.as_str())
            .ok_or_else(|| super::EvaluationError::UnsupportedValue {
                field: super::LIFECYCLE_POLICY_FIELD.as_str(),
                value: lifecycle_policy_label,
            })?;

        Ok(Self {
            resource_uri,
            mime_profile,
            title,
            csp_domains,
            permission_requests,
            host_context_sync,
            callback_channels,
            size_policy: dimension_policy,
            lifecycle_policy,
            policy_admission: draft.policy_admission,
            claims_runtime_execution: draft.claims_runtime_execution,
        })
    }

    /// Returns the UI resource identity.
    #[must_use]
    pub fn resource_uri(&self) -> &str {
        self.resource_uri.as_str()
    }

    /// Returns the MIME/profile tag.
    #[must_use]
    pub fn mime_profile(&self) -> &str {
        self.mime_profile.as_str()
    }

    /// Returns the human-facing title.
    #[must_use]
    pub fn title(&self) -> &str {
        self.title.as_str()
    }

    /// Returns CSP domain labels.
    #[must_use]
    pub fn csp_domains(&self) -> &[super::Text] {
        self.csp_domains.as_slice()
    }

    /// Returns browser or device permission labels.
    #[must_use]
    pub fn permission_requests(&self) -> &[super::Text] {
        self.permission_requests.as_slice()
    }

    /// Returns host-context synchronization labels.
    #[must_use]
    pub fn host_context_sync(&self) -> &[super::Text] {
        self.host_context_sync.as_slice()
    }

    /// Returns callback channel labels.
    #[must_use]
    pub fn callback_channels(&self) -> &[super::Text] {
        self.callback_channels.as_slice()
    }

    /// Returns the sizing policy.
    #[must_use]
    pub const fn size_policy(&self) -> SizePolicy {
        self.size_policy
    }

    /// Returns the lifecycle policy.
    #[must_use]
    pub const fn lifecycle_policy(&self) -> LifecyclePolicy {
        self.lifecycle_policy
    }

    /// Returns policy-admission flags used for classification.
    #[must_use]
    pub const fn policy_admission(&self) -> PolicyAdmissionDraft {
        self.policy_admission
    }
}

/// Preferred sizing policy for a future UI host.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SizePolicy {
    /// Descriptor declares bounded size hints only.
    Bounded,
    /// Descriptor asks a future host to fit content inside host bounds.
    HostFit,
}

impl SizePolicy {
    /// Returns the stable descriptor label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Bounded => super::SIZE_POLICY_BOUNDED,
            Self::HostFit => super::SIZE_POLICY_HOST_FIT,
        }
    }
}

/// Lifecycle policy for callbacks and UI lifetime.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LifecyclePolicy {
    /// Descriptor is passive and declares no callback channel.
    Passive,
    /// Descriptor declares host-mediated callbacks with bounded lifecycle.
    BoundedCallbacks,
    /// Descriptor attempts to keep callbacks unbounded.
    UnboundedCallbacks,
}

impl LifecyclePolicy {
    /// Returns the stable descriptor label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Passive => super::LIFECYCLE_POLICY_PASSIVE,
            Self::BoundedCallbacks => super::LIFECYCLE_POLICY_BOUNDED_CALLBACKS,
            Self::UnboundedCallbacks => super::LIFECYCLE_POLICY_UNBOUNDED_CALLBACKS,
        }
    }
}

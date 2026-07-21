/// Policy fixture for allowed sandbox constraint classes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SandboxPolicy {
    policy_id: alloc::string::String,
    allowed_classes: alloc::vec::Vec<SandboxConstraintClass>,
}

impl SandboxPolicy {
    /// Validates a sandbox policy fixture.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] when the policy ID is empty or a
    /// constraint class is unsupported.
    pub fn new(draft: SandboxPolicyDraft<'_>) -> Result<Self, crate::AdapterContractError> {
        let policy_id =
            crate::validation::normalized_adapter_field(POLICY_ID_FIELD, draft.policy_id)?;
        if draft.allowed_constraint_classes.is_empty() {
            return Err(crate::AdapterContractError::EmptyCollection {
                field: CONSTRAINT_CLASS_FIELD.as_str(),
            });
        }
        let allowed_classes = draft
            .allowed_constraint_classes
            .iter()
            .map(|value| SandboxConstraintClass::parse(value))
            .collect::<Result<alloc::vec::Vec<_>, _>>()?;
        Ok(Self {
            policy_id,
            allowed_classes,
        })
    }

    /// Returns the policy identity.
    #[must_use]
    pub fn policy_id(&self) -> &str {
        self.policy_id.as_str()
    }

    /// Returns allowed constraint classes.
    #[must_use]
    pub fn allowed_classes(&self) -> &[SandboxConstraintClass] {
        self.allowed_classes.as_slice()
    }

    /// Returns whether a class is allowed by this policy.
    #[must_use]
    pub fn allows(&self, class: SandboxConstraintClass) -> bool {
        self.allowed_classes.contains(&class)
    }
}

/// Borrowed sandbox policy fixture input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SandboxPolicyDraft<'a> {
    /// Stable policy identity.
    pub policy_id: &'a str,
    /// Allowed constraint class labels.
    pub allowed_constraint_classes: &'a [&'a str],
}

/// Platform support advertised by a host sandbox adapter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SandboxPlatformSupport {
    platform_id: alloc::string::String,
    capabilities: alloc::vec::Vec<SandboxPlatformCapability>,
}

impl SandboxPlatformSupport {
    /// Validates platform support advertised by a host adapter.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] when the platform ID is empty, the
    /// capability set is empty, or a capability label is unsupported.
    pub fn new(
        draft: SandboxPlatformSupportDraft<'_>,
    ) -> Result<Self, crate::AdapterContractError> {
        let platform_id =
            crate::validation::normalized_adapter_field(PLATFORM_ID_FIELD, draft.platform_id)?;
        if draft.capabilities.is_empty() {
            return Err(crate::AdapterContractError::EmptyCollection {
                field: PLATFORM_CAPABILITY_FIELD.as_str(),
            });
        }
        let capabilities = draft
            .capabilities
            .iter()
            .map(|value| SandboxPlatformCapability::parse(value))
            .collect::<Result<alloc::vec::Vec<_>, _>>()?;
        Ok(Self {
            platform_id,
            capabilities,
        })
    }

    /// Returns the platform identity.
    #[must_use]
    pub fn platform_id(&self) -> &str {
        self.platform_id.as_str()
    }

    /// Returns advertised capabilities.
    #[must_use]
    pub fn capabilities(&self) -> &[SandboxPlatformCapability] {
        self.capabilities.as_slice()
    }

    /// Returns whether the platform can enforce a capability.
    #[must_use]
    pub fn supports(&self, capability: SandboxPlatformCapability) -> bool {
        self.capabilities.contains(&capability)
    }
}

/// Borrowed platform support input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SandboxPlatformSupportDraft<'a> {
    /// Platform adapter identity.
    pub platform_id: &'a str,
    /// Platform capability labels.
    pub capabilities: &'a [&'a str],
}

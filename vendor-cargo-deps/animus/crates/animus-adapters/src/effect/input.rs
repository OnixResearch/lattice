const HOST_SUMMARY_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("host_effect_summary");
const CAPABILITY_SUMMARY_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("capability_summary");
const CANCELLATION_SUMMARY_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("cancellation_summary");
const RUNTIME_POLICY_SUMMARY_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("runtime_policy_summary");

/// Host-effect request passed to an injected host service.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HostRequest {
    capability: super::CapabilityClass,
    summary: alloc::string::String,
}

impl HostRequest {
    /// Builds a host-effect request with a redacted summary.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] when the summary is empty, too
    /// long, or contains a NUL character.
    pub fn new(
        capability: super::CapabilityClass,
        summary: &str,
    ) -> Result<Self, crate::AdapterContractError> {
        let summary = crate::validation::normalized_adapter_field(HOST_SUMMARY_FIELD, summary)?;
        Ok(Self {
            capability,
            summary,
        })
    }

    /// Returns the requested capability class.
    #[must_use]
    pub const fn capability(&self) -> super::CapabilityClass {
        self.capability
    }

    /// Returns the redacted request summary.
    #[must_use]
    pub fn summary(&self) -> &str {
        self.summary.as_str()
    }
}

/// Capability-decision request passed to an injected policy service.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapabilityDecisionRequest {
    capability: super::CapabilityClass,
    summary: alloc::string::String,
}

impl CapabilityDecisionRequest {
    /// Builds a capability-decision request with a redacted summary.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] when the summary is empty, too
    /// long, or contains a NUL character.
    pub fn new(
        capability: super::CapabilityClass,
        summary: &str,
    ) -> Result<Self, crate::AdapterContractError> {
        let summary =
            crate::validation::normalized_adapter_field(CAPABILITY_SUMMARY_FIELD, summary)?;
        Ok(Self {
            capability,
            summary,
        })
    }

    /// Returns the requested capability class.
    #[must_use]
    pub const fn capability(&self) -> super::CapabilityClass {
        self.capability
    }

    /// Returns the redacted request summary.
    #[must_use]
    pub fn summary(&self) -> &str {
        self.summary.as_str()
    }
}

/// Cancellation check passed to an injected cancellation service.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CancellationCheck {
    summary: alloc::string::String,
}

impl CancellationCheck {
    /// Builds a cancellation check with a redacted summary.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] when the summary is empty, too
    /// long, or contains a NUL character.
    pub fn new(summary: &str) -> Result<Self, crate::AdapterContractError> {
        let summary =
            crate::validation::normalized_adapter_field(CANCELLATION_SUMMARY_FIELD, summary)?;
        Ok(Self { summary })
    }

    /// Returns the redacted check summary.
    #[must_use]
    pub fn summary(&self) -> &str {
        self.summary.as_str()
    }
}

/// Runtime-policy request passed to an injected runtime-policy service.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimePolicyRequest {
    capability: super::CapabilityClass,
    summary: alloc::string::String,
}

impl RuntimePolicyRequest {
    /// Builds a runtime-policy request with a redacted summary.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] when the summary is empty, too
    /// long, or contains a NUL character.
    pub fn new(
        capability: super::CapabilityClass,
        summary: &str,
    ) -> Result<Self, crate::AdapterContractError> {
        let summary =
            crate::validation::normalized_adapter_field(RUNTIME_POLICY_SUMMARY_FIELD, summary)?;
        Ok(Self {
            capability,
            summary,
        })
    }

    /// Returns the requested capability class.
    #[must_use]
    pub const fn capability(&self) -> super::CapabilityClass {
        self.capability
    }

    /// Returns the redacted request summary.
    #[must_use]
    pub fn summary(&self) -> &str {
        self.summary.as_str()
    }
}

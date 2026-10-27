/// Binding status for a requested capability after authority checks.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BindingStatus {
    /// The capability is available below stronger authority checks.
    AllowedByPolicy,
    /// Host authority denied or omitted the capability.
    DeniedByHostAuthority,
    /// Runtime policy denied or omitted the capability.
    DeniedByRuntimePolicy,
}

/// Binding result for one requested capability.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CapabilityBinding {
    capability: super::types::Capability,
    status: BindingStatus,
}

impl CapabilityBinding {
    /// Returns the capability this binding evaluated.
    #[must_use]
    pub const fn capability(self) -> super::types::Capability {
        self.capability
    }

    /// Returns the binding status.
    #[must_use]
    pub const fn status(self) -> BindingStatus {
        self.status
    }
}

/// Input for binding profile posture to runtime permission policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BindingRequest<'a> {
    /// Profile-requested posture.
    pub requested_mode: super::types::Mode,
    /// Capability labels requested by the profile.
    pub requested_capabilities: &'a [super::types::Capability],
    /// Capability labels currently allowed by host authority.
    pub host_allowed_capabilities: &'a [super::types::Capability],
    /// Capability labels currently allowed by runtime policy.
    pub policy_allowed_capabilities: &'a [super::types::Capability],
}

/// Result of binding a profile-requested mode to stronger authority checks.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Binding {
    requested_mode: super::types::Mode,
    bindings: super::support::Collection<CapabilityBinding>,
}

impl Binding {
    /// Returns the profile-requested mode metadata.
    #[must_use]
    pub const fn requested_mode(&self) -> super::types::Mode {
        self.requested_mode
    }

    /// Returns per-capability binding decisions.
    #[must_use]
    pub fn bindings(&self) -> &[CapabilityBinding] {
        self.bindings.as_slice()
    }

    /// Returns true only when every requested capability survived stronger checks.
    #[must_use]
    pub fn all_allowed(&self) -> bool {
        self.bindings
            .iter()
            .all(|binding| binding.status() == BindingStatus::AllowedByPolicy)
    }
}

/// Resolves profile-requested mode metadata below host and runtime authority.
#[must_use]
pub fn bind_mode(request: BindingRequest<'_>) -> Binding {
    let mut bindings =
        super::support::Collection::with_capacity(request.requested_capabilities.len());
    for capability in request.requested_capabilities.iter().copied() {
        bindings.push(CapabilityBinding {
            capability,
            status: capability_status(
                capability,
                request.host_allowed_capabilities,
                request.policy_allowed_capabilities,
            ),
        });
    }

    Binding {
        requested_mode: request.requested_mode,
        bindings,
    }
}

fn capability_status(
    capability: super::types::Capability,
    host_allowed: &[super::types::Capability],
    policy_allowed: &[super::types::Capability],
) -> BindingStatus {
    if !host_allowed.contains(&capability) {
        return BindingStatus::DeniedByHostAuthority;
    }

    if !policy_allowed.contains(&capability) {
        return BindingStatus::DeniedByRuntimePolicy;
    }

    BindingStatus::AllowedByPolicy
}

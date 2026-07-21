/// Guarded action class adapted from Clash vocabulary into Animus-owned terms.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GuardedAction {
    /// Model or provider dispatch.
    Model,
    /// Tool invocation.
    Tool,
    /// Filesystem access.
    Filesystem,
    /// Process execution.
    Exec,
    /// Network access.
    Network,
    /// Steel evaluation.
    Steel,
    /// Semantic agent-role assignment, activation, handoff, or output influence.
    AgentRole,
}

impl GuardedAction {
    /// Returns the stable action label used by Nickel fixtures and receipts.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Model => "model",
            Self::Tool => "tool",
            Self::Filesystem => "filesystem",
            Self::Exec => "exec",
            Self::Network => "network",
            Self::Steel => "steel",
            Self::AgentRole => "agent_role",
        }
    }
}

/// Local policy layer that may only narrow, deny, or ask for authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LocalPolicyLayer {
    /// User-supplied local caveat.
    User,
    /// Project-supplied local caveat.
    Project,
    /// Session-supplied local caveat.
    Session,
    /// Harness/default local caveat.
    Harness,
}

impl LocalPolicyLayer {
    /// Returns the stable local-policy layer label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Project => "project",
            Self::Session => "session",
            Self::Harness => "harness",
        }
    }
}

/// Caveat decision supplied by a local policy layer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LocalCaveatDecision {
    /// Caveat does not object to this already-authorized request.
    Allow,
    /// Caveat denies this request locally.
    Deny,
    /// Caveat requires an approval path that produces bounded session authority.
    Ask,
}

impl LocalCaveatDecision {
    /// Returns the stable caveat decision label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Allow => "allow",
            Self::Deny => "deny",
            Self::Ask => "ask",
        }
    }
}

/// Local caveat from a user, project, session, or harness layer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LocalCaveat {
    layer: LocalPolicyLayer,
    decision: LocalCaveatDecision,
}

impl LocalCaveat {
    /// Builds a local caveat value.
    #[must_use]
    pub const fn new(layer: LocalPolicyLayer, decision: LocalCaveatDecision) -> Self {
        Self { layer, decision }
    }

    /// Returns the local-policy layer.
    #[must_use]
    pub const fn layer(self) -> LocalPolicyLayer {
        self.layer
    }

    /// Returns the local caveat decision.
    #[must_use]
    pub const fn decision(self) -> LocalCaveatDecision {
        self.decision
    }
}

/// Bounded caveat counts safe for receipts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CaveatSummary {
    pub(super) allows: u32,
    pub(super) denies: u32,
    pub(super) asks: u32,
}

impl CaveatSummary {
    /// Returns the allow-caveat count.
    #[must_use]
    pub const fn allows(self) -> u32 {
        self.allows
    }

    /// Returns the deny-caveat count.
    #[must_use]
    pub const fn denies(self) -> u32 {
        self.denies
    }

    /// Returns the ask-caveat count.
    #[must_use]
    pub const fn asks(self) -> u32 {
        self.asks
    }
}

/// Host/platform sandbox enforcement state for an authorized action.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SandboxState {
    /// The required descriptor is enforceable by the selected host adapter.
    Enforceable,
    /// The descriptor is malformed.
    Malformed,
    /// The selected host/platform adapter cannot enforce the descriptor.
    Unavailable,
}

/// Optional sandbox constraint attached to an already-authorized action.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SandboxConstraint {
    /// No sandbox was required for this request.
    NotRequired,
    /// A sandbox descriptor was required and is represented only by hash.
    Required {
        /// BLAKE3 hash of the sandbox descriptor.
        descriptor_hash: super::super::SafeHash,
        /// Host/platform enforcement state.
        state: SandboxState,
    },
}

impl SandboxConstraint {
    /// Builds a required sandbox descriptor constraint.
    #[must_use]
    pub fn required(descriptor_hash: super::super::SafeHash, state: SandboxState) -> Self {
        Self::Required {
            descriptor_hash,
            state,
        }
    }
}

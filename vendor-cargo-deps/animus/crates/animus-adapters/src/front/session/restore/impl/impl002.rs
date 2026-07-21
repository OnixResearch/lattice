
impl RestoreDegradedReason {
    /// Returns the stable degraded-reason label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UnsupportedCapability => "unsupported-capability",
            Self::MissingNativeReference => "missing-native-reference",
            Self::MalformedNativeReference => "malformed-native-reference",
            Self::DuplicatedNativeReference => "duplicated-native-reference",
            Self::StaleNativeReference => "stale-native-reference",
            Self::UnsupportedNativeReference => "unsupported-native-reference",
            Self::SensitiveNativeReference => "sensitive-native-reference",
            Self::HistoryReplayDisabled => "history-replay-disabled",
            Self::UnsafeHistoryReplay => "unsafe-history-replay",
            Self::FailedHandoffClaim => "failed-handoff-claim",
            Self::SafeFallback => "safe-fallback",
        }
    }
}

/// Restore capability claims for a planned target.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RestoreCapabilityClaims {
    bits: u8,
}

impl RestoreCapabilityClaims {
    /// Builds an empty safe fallback claim set.
    #[must_use]
    pub const fn fallback() -> Self {
        Self { bits: EMPTY_BITS }
    }

    const fn from_bits(bits: u8) -> Self {
        Self { bits }
    }

    /// Returns true when live process state survived.
    #[must_use]
    pub const fn process_survived(self) -> bool {
        has_bit(self.bits, CLAIM_PROCESS_SURVIVED)
    }

    /// Returns true when layout was reconstructed from a snapshot.
    #[must_use]
    pub const fn layout_reconstructed(self) -> bool {
        has_bit(self.bits, CLAIM_LAYOUT_RECONSTRUCTED)
    }

    /// Returns true when display history was replayed.
    #[must_use]
    pub const fn display_replayed(self) -> bool {
        has_bit(self.bits, CLAIM_DISPLAY_REPLAYED)
    }

    /// Returns true when native resume was requested.
    #[must_use]
    pub const fn native_resume_requested(self) -> bool {
        has_bit(self.bits, CLAIM_NATIVE_RESUME_REQUESTED)
    }

    /// Returns true when handoff transfer was requested.
    #[must_use]
    pub const fn handoff_requested(self) -> bool {
        has_bit(self.bits, CLAIM_HANDOFF_REQUESTED)
    }

    /// Returns true when replayed content was admitted as model-visible context.
    #[must_use]
    pub const fn model_context_admitted(self) -> bool {
        has_bit(self.bits, CLAIM_MODEL_CONTEXT_ADMITTED)
    }
}

/// Host-declared restore capabilities.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RestoreHostCapabilities {
    bits: u8,
}

impl RestoreHostCapabilities {
    /// Builds a host capability set from raw capability bits.
    #[must_use]
    pub const fn from_bits(bits: u8) -> Self {
        Self { bits }
    }

    /// Builds a host capability set supporting every restore mode.
    #[must_use]
    pub const fn all() -> Self {
        Self::from_bits(
            CAPABILITY_LIVE_PROCESS
                | CAPABILITY_SNAPSHOT_LAYOUT
                | CAPABILITY_HISTORY_REPLAY
                | CAPABILITY_NATIVE_RESUME
                | CAPABILITY_HANDOFF_TRANSFER,
        )
    }

    /// Builds a host capability set with handoff transfer disabled.
    #[must_use]
    pub const fn without_handoff() -> Self {
        Self::from_bits(
            CAPABILITY_LIVE_PROCESS
                | CAPABILITY_SNAPSHOT_LAYOUT
                | CAPABILITY_HISTORY_REPLAY
                | CAPABILITY_NATIVE_RESUME,
        )
    }

    /// Returns true when a restore mode is supported.
    #[must_use]
    pub const fn supports(self, mode: RestoreMode) -> bool {
        has_bit(self.bits, capability_bit(mode))
    }
}

/// Restore policy inputs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RestorePolicy {
    /// Whether history replay is enabled.
    pub history_replay_enabled: bool,
    /// Whether replay may become model-visible context.
    pub admit_replay_as_context: bool,
}

/// Native session reference kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeSessionReferenceKind {
    /// Native provider session identifier.
    ProviderSession,
    /// Agent-specific conversation identifier.
    AgentConversation,
    /// Host-owned opaque reference.
    HostOpaque,
}

impl NativeSessionReferenceKind {
    /// Returns the stable kind label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ProviderSession => "provider-session",
            Self::AgentConversation => "agent-conversation",
            Self::HostOpaque => "host-opaque",
        }
    }
}

/// Native session reference freshness.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeSessionFreshness {
    /// Reference is fresh enough to attempt resume.
    Fresh,
    /// Reference is stale and must degrade.
    Stale,
}

/// Native session uniqueness scope.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeSessionUniquenessScope {
    /// Unique within the current host.
    Host,
    /// Unique within the current workspace.
    Workspace,
    /// Unique within the agent/provider.
    Agent,
}

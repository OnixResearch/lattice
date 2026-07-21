/// Stable identity class for a memory descriptor.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum MemoryDescriptorKind {
    /// Reviewed instruction or project guidance.
    Instruction,
    /// Learned fact proposed from prior interaction.
    LearnedFact,
    /// Learned preference proposed from prior interaction.
    LearnedPreference,
}

impl MemoryDescriptorKind {
    /// Returns the stable kind label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Instruction => "instruction",
            Self::LearnedFact => "learned-fact",
            Self::LearnedPreference => "learned-preference",
        }
    }
}

/// Trust scope attached to a memory descriptor.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum MemoryTrustScope {
    /// User/global memory admitted by host policy.
    Global,
    /// Project-local memory requiring project trust admission.
    Project,
    /// Session-local memory for the current run.
    Session,
    /// Host/application-owned memory.
    Host,
}

impl MemoryTrustScope {
    const fn prompt_scope(self) -> PromptResourceScope {
        match self {
            Self::Global => PromptResourceScope::Global,
            Self::Project => PromptResourceScope::Project,
            Self::Session => PromptResourceScope::Session,
            Self::Host => PromptResourceScope::Host,
        }
    }

    /// Returns the stable trust-scope label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Global => "global",
            Self::Project => "project",
            Self::Session => "session",
            Self::Host => "host",
        }
    }
}

/// Freshness state for a memory descriptor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MemoryFreshness {
    /// Fresh enough for prompt influence under normal policy.
    Fresh,
    /// Stale and only includable when the selected policy allows stale memory.
    Stale,
    /// Expired memory that cannot validate for prompt influence.
    Expired,
}

impl MemoryFreshness {
    /// Returns the stable freshness label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Fresh => "fresh",
            Self::Stale => "stale",
            Self::Expired => "expired",
        }
    }
}

/// Prompt-inclusion policy for a descriptor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MemoryInclusionPolicy {
    /// Include when admitted and within prompt budget.
    Always,
    /// Include only after host relevance selection has supplied the descriptor.
    WhenRelevant,
    /// Never expose this descriptor to prompt assembly.
    Never,
}

impl MemoryInclusionPolicy {
    /// Returns the stable inclusion-policy label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Always => "always",
            Self::WhenRelevant => "when-relevant",
            Self::Never => "never",
        }
    }
}

/// Redaction class for a memory descriptor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MemoryRedactionClass {
    /// Body metadata can influence model-visible prompt assembly.
    Public,
    /// Only bounded metadata can be surfaced.
    MetadataOnly,
    /// Secret memory that cannot be exposed to the model.
    Secret,
}

impl MemoryRedactionClass {
    const fn prompt_redaction(self) -> PromptRedactionClass {
        match self {
            Self::Public => PromptRedactionClass::Public,
            Self::MetadataOnly => PromptRedactionClass::MetadataOnly,
            Self::Secret => PromptRedactionClass::Secret,
        }
    }

    /// Returns the stable redaction label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Public => "public",
            Self::MetadataOnly => "metadata-only",
            Self::Secret => "secret",
        }
    }
}

/// Admission status for a descriptor or candidate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MemoryAdmissionStatus {
    /// Host admission allows future prompt influence.
    Admitted,
    /// Host admission denied future prompt influence.
    Denied,
    /// Host admission service was unavailable.
    Unavailable,
}

impl MemoryAdmissionStatus {
    const fn prompt_admission(self) -> PromptSourceAdmissionStatus {
        match self {
            Self::Admitted => PromptSourceAdmissionStatus::Admitted,
            Self::Denied => PromptSourceAdmissionStatus::Denied,
            Self::Unavailable => PromptSourceAdmissionStatus::Unavailable,
        }
    }

    /// Returns the stable admission label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Admitted => "admitted",
            Self::Denied => "denied",
            Self::Unavailable => "unavailable",
        }
    }
}

/// Ordering rank for deterministic layered-memory prompt assembly.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct MemoryPrecedence(u16);

impl MemoryPrecedence {
    /// Builds a memory precedence rank where lower values assemble first.
    #[must_use]
    pub const fn new(rank: u16) -> Self {
        Self(rank)
    }

    /// Returns the stable rank value.
    #[must_use]
    pub const fn rank(self) -> u16 {
        self.0
    }
}

/// Availability status for host memory services.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MemoryAvailabilityStatus {
    /// Memory services are available.
    Available,
    /// Memory services are disabled by policy or product configuration.
    Disabled,
    /// Memory services are unavailable for this host or region.
    Unavailable,
}

/// Opt-in control state for memory use or generation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MemoryOptInControl {
    /// Scope explicitly allows the behavior.
    Enabled,
    /// Scope explicitly disables the behavior.
    Disabled,
    /// Scope has not supplied consent.
    Unset,
}

impl MemoryOptInControl {
    const fn is_enabled(self) -> bool {
        match self {
            Self::Enabled => true,
            Self::Disabled | Self::Unset => false,
        }
    }
}

/// Controls for user, project, and thread memory scopes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MemoryScopeControls {
    /// User-level control.
    pub user: MemoryOptInControl,
    /// Project-level control.
    pub project: MemoryOptInControl,
    /// Thread-level control.
    pub thread: MemoryOptInControl,
}

impl MemoryScopeControls {
    /// Returns true when every scope explicitly enables the behavior.
    #[must_use]
    pub const fn enabled(self) -> bool {
        self.user.is_enabled() && self.project.is_enabled() && self.thread.is_enabled()
    }
}


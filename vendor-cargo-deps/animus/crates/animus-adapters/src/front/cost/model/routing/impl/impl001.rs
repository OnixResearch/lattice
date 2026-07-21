use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::num::NonZeroU16;

const ROLE_LABEL_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("routing_role_label");
const MODEL_LABEL_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("routing_model_label");
const POLICY_TAG_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("routing_policy_tag");
const MAX_ROUTING_ROLES: usize = 8;
const MAX_ROUTING_MODELS: usize = 16;
const MAX_POLICY_TAGS: usize = 8;
const CANONICAL_SEPARATOR: &str = "|";
const CANONICAL_VERSION_PREFIX: &str = "model-routing-receipt-v1";
const UNKNOWN_PRICE_SORT_UNITS: u32 = u32::MAX;
const ZERO_ROUTING_COUNT: usize = 0;

const _: () = assert!(MAX_ROUTING_ROLES > ZERO_ROUTING_COUNT);
const _: () = assert!(MAX_ROUTING_MODELS > ZERO_ROUTING_COUNT);
const _: () = assert!(MAX_POLICY_TAGS > ZERO_ROUTING_COUNT);

/// Missing-price behavior for budgeted routing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RoutingMissingPriceBehavior {
    /// Deny candidates that lack price metadata.
    Deny,
    /// Allow candidates with missing price metadata and record a warning fallback.
    AllowWithWarning,
    /// Defer missing-price handling to a host-owned adapter.
    HostFallback,
}

impl RoutingMissingPriceBehavior {
    /// Returns the stable behavior label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Deny => "deny",
            Self::AllowWithWarning => "allow-with-warning",
            Self::HostFallback => "host-fallback",
        }
    }
}

/// Budget behavior for cost-aware routing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RoutingBudgetBehavior {
    /// Prefer a cheaper downgrade under soft pressure, and deny at hard limit when no downgrade exists.
    PreferCheaperThenDeny,
}

impl RoutingBudgetBehavior {
    /// Returns the stable behavior label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PreferCheaperThenDeny => "prefer-cheaper-then-deny",
        }
    }
}

/// Score bucket used in safe routing decisions and receipts.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum RoutingScoreBucket {
    /// Low complexity score.
    Simple,
    /// Medium complexity score.
    Medium,
    /// High complexity score.
    Complex,
}

impl RoutingScoreBucket {
    /// Returns the stable score bucket label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Simple => "simple",
            Self::Medium => "medium",
            Self::Complex => "complex",
        }
    }
}

/// Budget status for the current routing decision.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RoutingBudgetStatus {
    /// Current usage is below the soft budget.
    WithinBudget,
    /// Current usage is at or above the soft budget and below the hard budget.
    SoftPressure,
    /// Current usage is at or above the hard budget.
    HardLimit,
}

impl RoutingBudgetStatus {
    /// Returns the stable budget status label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::WithinBudget => "within-budget",
            Self::SoftPressure => "soft-pressure",
            Self::HardLimit => "hard-limit",
        }
    }
}

/// Safe fallback or reason class recorded by routing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RoutingFallbackClass {
    /// Decision came from explicit user role hint.
    ExplicitUserHint,
    /// Decision came from complexity scoring.
    ComplexityScore,
    /// Soft budget pressure selected a cheaper role.
    SoftBudgetPressure,
    /// Hard budget selected a cheaper role.
    HardBudgetDowngrade,
    /// Hard budget denied routing.
    HardBudgetDenied,
    /// Candidate price metadata was missing and denied by policy.
    MissingPrice,
    /// Candidate role existed but all models were unavailable.
    UnavailableModel,
    /// No eligible model matched the selected role.
    NoEligibleModel,
}

impl RoutingFallbackClass {
    /// Returns the stable fallback label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ExplicitUserHint => "explicit-user-hint",
            Self::ComplexityScore => "complexity-score",
            Self::SoftBudgetPressure => "soft-budget-pressure",
            Self::HardBudgetDowngrade => "hard-budget-downgrade",
            Self::HardBudgetDenied => "hard-budget-denied",
            Self::MissingPrice => "missing-price",
            Self::UnavailableModel => "unavailable-model",
            Self::NoEligibleModel => "no-eligible-model",
        }
    }
}

/// Routing decision status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RoutingDecisionStatus {
    /// A safe role/model was selected.
    Selected,
    /// Routing was denied before provider dispatch.
    Denied,
}

impl RoutingDecisionStatus {
    /// Returns the stable decision status label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Selected => "selected",
            Self::Denied => "denied",
        }
    }
}

/// Safe model capability class.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum RoutingModelCapability {
    /// Text generation capability.
    Text,
    /// Tool-use capable model.
    ToolUse,
    /// Vision-capable model.
    Vision,
    /// Reasoning-optimized model.
    Reasoning,
}

impl RoutingModelCapability {
    /// Returns the stable capability label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::ToolUse => "tool-use",
            Self::Vision => "vision",
            Self::Reasoning => "reasoning",
        }
    }
}

/// Safe model availability class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RoutingModelAvailability {
    /// Model is available for selection.
    Available,
    /// Model is unavailable for selection.
    Unavailable,
}

impl RoutingModelAvailability {
    /// Returns the stable availability label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Available => "available",
            Self::Unavailable => "unavailable",
        }
    }
}

/// Borrowed routing thresholds before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RoutingThresholdsDraft {
    /// Maximum score considered simple.
    pub simple_max_score: u16,
    /// Maximum score considered medium.
    pub medium_max_score: u16,
}

/// Validated routing thresholds.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RoutingThresholds {
    simple_max_score: u16,
    medium_max_score: u16,
}

impl RoutingThresholds {
    /// Returns the simple bucket maximum score.
    #[must_use]
    pub const fn simple_max_score(self) -> u16 {
        self.simple_max_score
    }

    /// Returns the medium bucket maximum score.
    #[must_use]
    pub const fn medium_max_score(self) -> u16 {
        self.medium_max_score
    }
}

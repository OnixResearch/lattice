const TASK_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("router_task_shape");
const RISK_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("router_risk_band");
const PRESSURE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("router_pressure_band");
const LATENCY_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("router_latency_band");
const COST_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("router_cost_band");
const PRIVACY_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("router_privacy_band");
const FALLBACK_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("router_fallback_eligibility");

const TASK_SIMPLE: &str = "simple";
const TASK_REASONING: &str = "reasoning";
const TASK_CODING: &str = "coding";
const TASK_REVIEW: &str = "review";
const TASK_WORKFLOW: &str = "workflow";
const BAND_LOW: &str = "low";
const BAND_MEDIUM: &str = "medium";
const BAND_HIGH: &str = "high";
const BAND_STRICT: &str = "strict";
const PRIVACY_LOCAL: &str = "local_only";
const PRIVACY_REMOTE: &str = "remote_allowed";
const FALLBACK_ELIGIBLE: &str = "eligible";
const FALLBACK_INELIGIBLE: &str = "ineligible";

/// Number of safe signal buckets exposed by [`Signals::bucket_labels`].
pub const SIGNAL_BUCKET_COUNT: usize = 8;

/// Request task-shape signal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TaskShape {
    /// Simple request shape.
    Simple,
    /// Reasoning-heavy request shape.
    Reasoning,
    /// Coding request shape.
    Coding,
    /// Review request shape.
    Review,
    /// Workflow request shape.
    Workflow,
}

impl TaskShape {
    pub(crate) fn parse(value: &str) -> Result<Self, crate::AdapterContractError> {
        super::parse_value(TASK_FIELD, value, |normalized| match normalized {
            TASK_SIMPLE => Some(Self::Simple),
            TASK_REASONING => Some(Self::Reasoning),
            TASK_CODING => Some(Self::Coding),
            TASK_REVIEW => Some(Self::Review),
            TASK_WORKFLOW => Some(Self::Workflow),
            _ => None,
        })
    }

    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Simple => TASK_SIMPLE,
            Self::Reasoning => TASK_REASONING,
            Self::Coding => TASK_CODING,
            Self::Review => TASK_REVIEW,
            Self::Workflow => TASK_WORKFLOW,
        }
    }
}

/// Low, medium, high, or strict bounded signal band.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum SignalBand {
    /// Low signal band.
    Low,
    /// Medium signal band.
    Medium,
    /// High signal band.
    High,
    /// Strict signal band.
    Strict,
}

impl SignalBand {
    pub(crate) fn parse(
        field: crate::validation::FieldName,
        value: &str,
    ) -> Result<Self, crate::AdapterContractError> {
        super::parse_value(field, value, |normalized| match normalized {
            BAND_LOW => Some(Self::Low),
            BAND_MEDIUM => Some(Self::Medium),
            BAND_HIGH => Some(Self::High),
            BAND_STRICT => Some(Self::Strict),
            _ => None,
        })
    }

    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Low => BAND_LOW,
            Self::Medium => BAND_MEDIUM,
            Self::High => BAND_HIGH,
            Self::Strict => BAND_STRICT,
        }
    }
}

/// Privacy/locality signal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PrivacyBand {
    /// Request must remain local.
    LocalOnly,
    /// Remote backends are eligible.
    RemoteAllowed,
}

impl PrivacyBand {
    pub(crate) fn parse(value: &str) -> Result<Self, crate::AdapterContractError> {
        super::parse_value(PRIVACY_FIELD, value, |normalized| match normalized {
            PRIVACY_LOCAL => Some(Self::LocalOnly),
            PRIVACY_REMOTE => Some(Self::RemoteAllowed),
            _ => None,
        })
    }

    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::LocalOnly => PRIVACY_LOCAL,
            Self::RemoteAllowed => PRIVACY_REMOTE,
        }
    }
}

/// Fallback eligibility signal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FallbackEligibility {
    /// Fallback is eligible.
    Eligible,
    /// Fallback is not eligible.
    Ineligible,
}

impl FallbackEligibility {
    pub(crate) fn parse(value: &str) -> Result<Self, crate::AdapterContractError> {
        super::parse_value(FALLBACK_FIELD, value, |normalized| match normalized {
            FALLBACK_ELIGIBLE => Some(Self::Eligible),
            FALLBACK_INELIGIBLE => Some(Self::Ineligible),
            _ => None,
        })
    }

    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Eligible => FALLBACK_ELIGIBLE,
            Self::Ineligible => FALLBACK_INELIGIBLE,
        }
    }
}

/// Bounded request signals used for deterministic recipe selection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Signals {
    task_shape: TaskShape,
    risk: SignalBand,
    contract_pressure: SignalBand,
    context_pressure: SignalBand,
    latency: SignalBand,
    cost: SignalBand,
    privacy: PrivacyBand,
    fallback: FallbackEligibility,
}

impl Signals {
    /// Projects bounded request metadata into router signals without raw payloads.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] when any signal label is empty,
    /// malformed, or unsupported.
    pub fn project(draft: SignalDraft<'_>) -> Result<Self, crate::AdapterContractError> {
        Ok(Self {
            task_shape: TaskShape::parse(draft.task_shape)?,
            risk: SignalBand::parse(RISK_FIELD, draft.risk_band)?,
            contract_pressure: SignalBand::parse(PRESSURE_FIELD, draft.contract_pressure)?,
            context_pressure: SignalBand::parse(PRESSURE_FIELD, draft.context_pressure)?,
            latency: SignalBand::parse(LATENCY_FIELD, draft.latency_band)?,
            cost: SignalBand::parse(COST_FIELD, draft.cost_band)?,
            privacy: PrivacyBand::parse(draft.privacy_band)?,
            fallback: FallbackEligibility::parse(draft.fallback_eligibility)?,
        })
    }

    /// Returns task shape.
    #[must_use]
    pub const fn task_shape(&self) -> TaskShape {
        self.task_shape
    }

    /// Returns risk band.
    #[must_use]
    pub const fn risk(&self) -> SignalBand {
        self.risk
    }

    /// Returns output-contract pressure.
    #[must_use]
    pub const fn contract_pressure(&self) -> SignalBand {
        self.contract_pressure
    }

    /// Returns privacy/locality band.
    #[must_use]
    pub const fn privacy(&self) -> PrivacyBand {
        self.privacy
    }

    /// Returns safe signal bucket labels for receipts.
    #[must_use]
    pub fn bucket_labels(&self) -> [alloc::string::String; SIGNAL_BUCKET_COUNT] {
        [
            alloc::string::String::from(self.task_shape.as_str()),
            alloc::string::String::from(self.risk.as_str()),
            alloc::string::String::from(self.contract_pressure.as_str()),
            alloc::string::String::from(self.context_pressure.as_str()),
            alloc::string::String::from(self.latency.as_str()),
            alloc::string::String::from(self.cost.as_str()),
            alloc::string::String::from(self.privacy.as_str()),
            alloc::string::String::from(self.fallback.as_str()),
        ]
    }
}

/// Borrowed bounded metadata used by signal projection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SignalDraft<'a> {
    /// Task shape label.
    pub task_shape: &'a str,
    /// Risk band label.
    pub risk_band: &'a str,
    /// Output-contract pressure label.
    pub contract_pressure: &'a str,
    /// Context pressure label.
    pub context_pressure: &'a str,
    /// Latency band label.
    pub latency_band: &'a str,
    /// Cost band label.
    pub cost_band: &'a str,
    /// Privacy/locality label.
    pub privacy_band: &'a str,
    /// Fallback eligibility label.
    pub fallback_eligibility: &'a str,
}

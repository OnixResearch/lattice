const STATUS_OBSERVATION_KIND_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("status_observation_kind");
const STATUS_STATE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("semantic_status_state");

const STATE_IDLE: &str = "idle";
const STATE_WORKING: &str = "working";
const STATE_BLOCKED: &str = "blocked";
const STATE_DONE: &str = "done";
const STATE_UNKNOWN: &str = "unknown";

const KIND_TRUSTED_LIFECYCLE: &str = "trusted_lifecycle";
const KIND_UNTRUSTED_LIFECYCLE: &str = "untrusted_lifecycle";
const KIND_HEURISTIC_FALLBACK: &str = "heuristic_fallback";

/// Semantic lifecycle state shared by runtime hosts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SemanticAgent {
    /// The agent is available for work.
    Idle,
    /// The agent is actively working.
    Working,
    /// The agent needs attention or cannot continue.
    Blocked,
    /// The agent completed its current work.
    Done,
    /// The current semantic state is unknown.
    Unknown,
}

impl SemanticAgent {
    /// Parses a boundary status state string.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] when the value is empty, too long,
    /// contains a NUL character, or is not one of the supported status states.
    pub fn parse(value: &str) -> Result<Self, crate::AdapterContractError> {
        crate::validation::validate_required_adapter_field(STATUS_STATE_FIELD, value)?;
        match value.trim() {
            STATE_IDLE => Ok(Self::Idle),
            STATE_WORKING => Ok(Self::Working),
            STATE_BLOCKED => Ok(Self::Blocked),
            STATE_DONE => Ok(Self::Done),
            STATE_UNKNOWN => Ok(Self::Unknown),
            other => Err(crate::AdapterContractError::UnsupportedStatusState {
                field: STATUS_STATE_FIELD.as_str(),
                value: alloc::string::String::from(other),
            }),
        }
    }

    /// Returns the stable lowercase status spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Idle => STATE_IDLE,
            Self::Working => STATE_WORKING,
            Self::Blocked => STATE_BLOCKED,
            Self::Done => STATE_DONE,
            Self::Unknown => STATE_UNKNOWN,
        }
    }
}

/// Observation authority class supplied by a host integration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObservationKind {
    /// Trusted lifecycle report; current reports own semantic state.
    TrustedLifecycle,
    /// Untrusted lifecycle report; selection records it but must not trust it.
    UntrustedLifecycle,
    /// Host heuristic fallback observation.
    HeuristicFallback,
}

impl ObservationKind {
    /// Parses an observation-kind boundary string.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] when the value is empty, too long,
    /// contains a NUL character, or is not one of the supported observation kinds.
    pub fn parse(value: &str) -> Result<Self, crate::AdapterContractError> {
        crate::validation::validate_required_adapter_field(STATUS_OBSERVATION_KIND_FIELD, value)?;
        match value.trim() {
            KIND_TRUSTED_LIFECYCLE => Ok(Self::TrustedLifecycle),
            KIND_UNTRUSTED_LIFECYCLE => Ok(Self::UntrustedLifecycle),
            KIND_HEURISTIC_FALLBACK => Ok(Self::HeuristicFallback),
            other => Err(
                crate::AdapterContractError::UnsupportedStatusObservationKind {
                    field: STATUS_OBSERVATION_KIND_FIELD.as_str(),
                    value: alloc::string::String::from(other),
                },
            ),
        }
    }

    /// Returns the stable lowercase observation-kind spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::TrustedLifecycle => KIND_TRUSTED_LIFECYCLE,
            Self::UntrustedLifecycle => KIND_UNTRUSTED_LIFECYCLE,
            Self::HeuristicFallback => KIND_HEURISTIC_FALLBACK,
        }
    }
}

/// Logical monotonic sequence used for deterministic freshness checks.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Sequence {
    value: u64,
}

impl Sequence {
    /// Builds a status sequence value.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self { value }
    }

    /// Returns the raw sequence value.
    #[must_use]
    pub const fn value(self) -> u64 {
        self.value
    }
}

/// Logical TTL for an observation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Ttl {
    expires_at: Sequence,
}

impl Ttl {
    /// Builds a TTL whose observation is current through `expires_at`.
    #[must_use]
    pub const fn new(expires_at: Sequence) -> Self {
        Self { expires_at }
    }

    /// Returns the last sequence for which the observation is current.
    #[must_use]
    pub const fn expires_at(self) -> Sequence {
        self.expires_at
    }

    /// Returns whether the TTL is current at `current_sequence`.
    #[must_use]
    pub const fn is_current_at(self, current_sequence: Sequence) -> bool {
        current_sequence.value() <= self.expires_at.value()
    }
}

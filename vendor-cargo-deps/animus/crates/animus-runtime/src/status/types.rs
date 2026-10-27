/// Freshness class assigned to a status decision.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Freshness {
    /// A current lifecycle or heuristic authority was selected.
    Current,
    /// Matching observations existed but all authoritative candidates were stale.
    Stale,
    /// No matching usable observation existed.
    Missing,
    /// Current lifecycle authorities conflicted and the selection failed closed.
    Conflicting,
}

/// Reason a decision selected fallback or unknown state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FallbackReason {
    /// No trusted lifecycle report matched the target.
    NoTrustedLifecycle,
    /// Trusted lifecycle reports matched but were expired by TTL.
    StaleLifecycle,
    /// Only untrusted lifecycle reports matched the target.
    UntrustedLifecycle,
    /// Current trusted lifecycle authorities disagreed at the newest sequence.
    ConflictingLifecycleAuthority,
    /// No observation matched the target.
    NoObservation,
}

/// Reason an observation was not selected.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RejectedReason {
    /// Observation targeted another agent or pane.
    TargetMismatch,
    /// Observation was expired by its TTL.
    Stale,
    /// Observation came from an untrusted lifecycle source.
    UntrustedLifecycle,
    /// Heuristic fallback was suppressed by a trusted lifecycle authority.
    HeuristicSuppressed,
    /// Observation was older than the selected observation.
    OlderSequence,
    /// Observation duplicated the selected lifecycle authority class.
    DuplicateLifecycleAuthority,
    /// Observation conflicted with another current lifecycle authority.
    ConflictingLifecycleAuthority,
}

/// Safe summary of a selected status authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthoritySummary {
    source_id: String,
    kind: animus_adapters::AgentStatusObservationKind,
    state: animus_adapters::SemanticAgentState,
    sequence: animus_adapters::StatusSequence,
}

impl AuthoritySummary {
    pub(super) fn from_observation(observation: &animus_adapters::AgentStatusObservation) -> Self {
        Self {
            source_id: String::from(observation.source_id()),
            kind: observation.kind(),
            state: observation.state(),
            sequence: observation.sequence(),
        }
    }

    /// Returns the selected source identity.
    #[must_use]
    pub fn source_id(&self) -> &str {
        self.source_id.as_str()
    }

    /// Returns the selected observation kind.
    #[must_use]
    pub const fn kind(&self) -> animus_adapters::AgentStatusObservationKind {
        self.kind
    }

    /// Returns the selected semantic state.
    #[must_use]
    pub const fn state(&self) -> animus_adapters::SemanticAgentState {
        self.state
    }

    /// Returns the selected sequence.
    #[must_use]
    pub const fn sequence(&self) -> animus_adapters::StatusSequence {
        self.sequence
    }
}

/// Safe summary of a rejected status candidate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RejectedCandidate {
    authority: AuthoritySummary,
    reason: RejectedReason,
}

impl RejectedCandidate {
    pub(super) fn new(
        observation: &animus_adapters::AgentStatusObservation,
        reason: RejectedReason,
    ) -> Self {
        Self {
            authority: AuthoritySummary::from_observation(observation),
            reason,
        }
    }

    /// Returns the rejected authority summary.
    #[must_use]
    pub const fn authority(&self) -> &AuthoritySummary {
        &self.authority
    }

    /// Returns the rejection reason.
    #[must_use]
    pub const fn reason(&self) -> RejectedReason {
        self.reason
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct RejectionCollector {
    items: Vec<RejectedCandidate>,
}

impl RejectionCollector {
    pub(super) fn with_capacity(capacity: usize) -> Self {
        Self {
            items: Vec::with_capacity(capacity),
        }
    }

    pub(super) fn push(
        &mut self,
        observation: &animus_adapters::AgentStatusObservation,
        reason: RejectedReason,
    ) {
        self.items.push(RejectedCandidate::new(observation, reason));
    }

    pub(super) fn into_vec(self) -> Vec<RejectedCandidate> {
        self.items
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct SelectedObservation<'a> {
    pub(super) observation: &'a animus_adapters::AgentStatusObservation,
    pub(super) index: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ClassifiedObservations<'a> {
    pub(super) trusted: Vec<&'a animus_adapters::AgentStatusObservation>,
    pub(super) heuristics: Vec<&'a animus_adapters::AgentStatusObservation>,
    pub(super) signals: FallbackSignals,
}

/// Predicate signals used to choose a fallback reason.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(super) struct FallbackSignals {
    pub(super) has_target: bool,
    pub(super) has_stale_lifecycle: bool,
    pub(super) has_untrusted_lifecycle: bool,
}

/// Semantic simulation observation event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SimulationProbeEvent {
    /// A normalized message was delivered.
    MessageDelivered,
    /// Pure update produced an accepted, rejected, or ignored outcome.
    TransitionEvaluated,
    /// A deterministic diagnostic was produced.
    DiagnosticProduced,
    /// Logical invalidation was classified.
    InvalidationClassified,
    /// A declarative effect crossed a lifecycle boundary.
    EffectLifecycle,
    /// A finite controlled choice was selected.
    ChoiceSelected,
    /// Canonical accepted state was checkpointed.
    StateCheckpoint,
    /// A pure oracle fact was observed.
    OracleObserved,
}

impl SimulationProbeEvent {
    const fn as_str(self) -> &'static str {
        match self {
            Self::MessageDelivered => "message-delivered",
            Self::TransitionEvaluated => "transition-evaluated",
            Self::DiagnosticProduced => "diagnostic-produced",
            Self::InvalidationClassified => "invalidation-classified",
            Self::EffectLifecycle => "effect-lifecycle",
            Self::ChoiceSelected => "choice-selected",
            Self::StateCheckpoint => "state-checkpoint",
            Self::OracleObserved => "oracle-observed",
        }
    }
}

/// Semantic outcome attached to a simulation probe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SimulationProbeOutcome {
    /// Read-only observation with no transition verdict.
    Observed,
    /// Authoritative state accepted a change.
    Accepted,
    /// Input was rejected with a deterministic diagnostic.
    Rejected,
    /// Input was valid but intentionally had no effect.
    Ignored,
    /// Effect was requested.
    Requested,
    /// Effect execution started.
    Started,
    /// Effect completed successfully.
    Completed,
    /// Effect failed.
    Failed,
    /// Effect was cancelled.
    Cancelled,
    /// Effect was replaced by a newer request.
    Replaced,
    /// Effect timed out.
    TimedOut,
    /// Effect source closed.
    SourceClosed,
    /// Controlled choice was selected.
    Selected,
    /// Oracle condition passed at this observation.
    Passed,
    /// Oracle condition failed at this observation.
    FailedOracle,
    /// Cataloged oracle was not observed.
    Unobserved,
}

impl SimulationProbeOutcome {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Observed => "observed",
            Self::Accepted => "accepted",
            Self::Rejected => "rejected",
            Self::Ignored => "ignored",
            Self::Requested => "requested",
            Self::Started => "started",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
            Self::Replaced => "replaced",
            Self::TimedOut => "timed-out",
            Self::SourceClosed => "source-closed",
            Self::Selected => "selected",
            Self::Passed => "passed",
            Self::FailedOracle => "failed-oracle",
            Self::Unobserved => "unobserved",
        }
    }
}

/// One bounded allowlisted probe metadata pair.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SimulationProbeMetadata {
    /// Allowlisted stable metadata key.
    pub key: String,
    /// Bounded non-sensitive semantic value.
    pub value: String,
}

impl SimulationProbeMetadata {
    /// Creates one metadata pair.
    #[must_use]
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }
}

/// Semantic event, subject, and outcome carried by one probe.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimulationProbeSemantic {
    /// Semantic event class.
    pub event: SimulationProbeEvent,
    /// Stable semantic subject identity.
    pub subject_id: String,
    /// Semantic outcome class.
    pub outcome: SimulationProbeOutcome,
}

impl SimulationProbeSemantic {
    /// Creates one semantic probe fact.
    #[must_use]
    pub fn new(
        event: SimulationProbeEvent,
        subject_id: impl Into<String>,
        outcome: SimulationProbeOutcome,
    ) -> Self {
        Self {
            event,
            subject_id: subject_id.into(),
            outcome,
        }
    }
}

/// Versioned provider-neutral observation record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimulationProbeRecord {
    /// Probe schema version. Version 1 is currently accepted.
    pub schema_version: u16,
    /// Stable checked probe identity.
    pub probe_id: SimulationProbeId,
    /// Stable checked scenario identity.
    pub scenario_id: SimulationScenarioId,
    /// Monotonically increasing scenario-local sequence.
    pub sequence: u64,
    /// Explicit logical tick, when the boundary is time-related.
    pub logical_tick: Option<u64>,
    /// Semantic event class.
    pub event: SimulationProbeEvent,
    /// Stable semantic subject identity.
    pub subject_id: String,
    /// Semantic outcome class.
    pub outcome: SimulationProbeOutcome,
    /// Optional canonical state BLAKE3 identity.
    pub state_identity: Option<String>,
    /// Strictly key-ordered allowlisted metadata.
    pub metadata: Vec<SimulationProbeMetadata>,
}

impl SimulationProbeRecord {
    /// Creates a version-1 probe record with no optional fields.
    #[must_use]
    pub fn new(
        probe_id: impl Into<SimulationProbeId>,
        scenario_id: impl Into<SimulationScenarioId>,
        sequence: u64,
        semantic: SimulationProbeSemantic,
    ) -> Self {
        Self {
            schema_version: PROBE_SCHEMA_VERSION,
            probe_id: probe_id.into(),
            scenario_id: scenario_id.into(),
            sequence,
            logical_tick: None,
            event: semantic.event,
            subject_id: semantic.subject_id,
            outcome: semantic.outcome,
            state_identity: None,
            metadata: Vec::new(),
        }
    }

    /// Attaches an explicit logical tick.
    #[must_use]
    pub const fn with_logical_tick(mut self, logical_tick: u64) -> Self {
        self.logical_tick = Some(logical_tick);
        self
    }

    /// Attaches a canonical state identity.
    #[must_use]
    pub fn with_state_identity(mut self, identity: impl Into<String>) -> Self {
        self.state_identity = Some(identity.into());
        self
    }

    /// Attaches metadata in caller-supplied deterministic key order.
    #[must_use]
    pub fn with_metadata(mut self, metadata: Vec<SimulationProbeMetadata>) -> Self {
        self.metadata = metadata;
        self
    }

    /// Encodes the record as deterministic line-oriented bytes.
    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        let tick = self
            .logical_tick
            .map_or_else(|| "-".to_string(), |value| value.to_string());
        let state = self.state_identity.as_deref().unwrap_or("-");
        let mut output = format!(
            "schema={}\nprobe={}\nscenario={}\nsequence={}\nlogical_tick={}\nevent={}\nsubject={}\noutcome={}\nstate={}\n",
            self.schema_version,
            self.probe_id,
            self.scenario_id,
            self.sequence,
            tick,
            self.event.as_str(),
            self.subject_id,
            self.outcome.as_str(),
            state,
        );
        for pair in &self.metadata {
            output.push_str("metadata.");
            output.push_str(&pair.key);
            output.push('=');
            output.push_str(&pair.value);
            output.push('\n');
        }
        output.into_bytes()
    }
}

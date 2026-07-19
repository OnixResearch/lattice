/// Fixture metadata attached to a replay script.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayMetadata {
    /// Stable fixture ID.
    pub id: String,
    /// Human-readable fixture name.
    pub name: String,
    /// Component or widget family under test.
    pub component: String,
}

impl ReplayMetadata {
    /// Creates replay metadata.
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        component: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            component: component.into(),
        }
    }
}

/// Expected final model state assertion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayAssertion {
    /// State key to compare.
    pub key: String,
    /// Expected state value.
    pub expected: String,
}

impl ReplayAssertion {
    /// Creates an expected state assertion.
    #[must_use]
    pub fn new(key: impl Into<String>, expected: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            expected: expected.into(),
        }
    }
}

/// Optional expected snapshot evidence for replay outputs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplaySnapshotExpectation {
    /// Snapshot label such as `state` or `terminal`.
    pub label: String,
    /// Expected BLAKE3 hash in lowercase hex.
    pub blake3: String,
}

impl ReplaySnapshotExpectation {
    /// Creates a snapshot expectation.
    #[must_use]
    pub fn new(label: impl Into<String>, blake3: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            blake3: blake3.into(),
        }
    }
}

/// Deterministic replay wait condition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayWaitCondition {
    /// Predicate to evaluate against supplied replay target state.
    pub predicate: ReplayWaitPredicate,
    /// Bounded evaluation budget supplied as data, not wall-clock time.
    pub evaluation_budget: usize,
}

impl ReplayWaitCondition {
    /// Creates a deterministic wait condition.
    #[must_use]
    pub const fn new(predicate: ReplayWaitPredicate, evaluation_budget: usize) -> Self {
        Self {
            predicate,
            evaluation_budget,
        }
    }
}

/// Deterministic wait predicate evaluated by the pure replay runner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplayWaitPredicate {
    /// A target state key must equal an expected value.
    StateEquals {
        /// State key to inspect.
        key: String,
        /// Expected value.
        expected: String,
    },
    /// A target state key must contain a substring.
    StateContains {
        /// State key to inspect.
        key: String,
        /// Required substring.
        needle: String,
    },
    /// A snapshot byte stream must contain UTF-8 text.
    SnapshotContains {
        /// Snapshot label to inspect.
        label: String,
        /// Required text.
        needle: String,
    },
    /// The target must expose and focus a specific focus target.
    FocusTarget {
        /// Expected focus target ID.
        target: String,
    },
    /// A target diagnostic event must be present.
    DiagnosticPresent {
        /// Stable diagnostic event code.
        code: String,
    },
    /// Unsupported predicate marker used by negative fixtures.
    Unsupported {
        /// Unsupported predicate label.
        predicate: String,
    },
}

/// Renderer-neutral replay script.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayScript {
    /// Fixture metadata.
    pub metadata: ReplayMetadata,
    /// Logical events applied in order.
    pub events: Vec<super::ReplayEvent>,
    /// Expected final state assertions.
    pub assertions: Vec<ReplayAssertion>,
    /// Optional snapshot expectations.
    pub snapshots: Vec<ReplaySnapshotExpectation>,
}

impl ReplayScript {
    /// Creates a replay script with metadata.
    #[must_use]
    pub fn new(metadata: ReplayMetadata) -> Self {
        Self {
            metadata,
            events: Vec::new(),
            assertions: Vec::new(),
            snapshots: Vec::new(),
        }
    }

    /// Adds a logical event.
    #[must_use]
    pub fn with_event(mut self, event: super::ReplayEvent) -> Self {
        self.events.push(event);
        self
    }

    /// Adds an expected final-state assertion.
    #[must_use]
    pub fn with_assertion(mut self, assertion: ReplayAssertion) -> Self {
        self.assertions.push(assertion);
        self
    }

    /// Adds an expected snapshot hash.
    #[must_use]
    pub fn with_snapshot(mut self, snapshot: ReplaySnapshotExpectation) -> Self {
        self.snapshots.push(snapshot);
        self
    }
}

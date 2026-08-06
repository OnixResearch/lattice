/// Stable probe-boundary validation code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimulationProbeDiagnosticCode {
    /// Schema version is unsupported.
    UnknownSchemaVersion,
    /// Probe or scenario identity is not registered.
    UnknownIdentity,
    /// Sequence duplicates or regresses.
    NonMonotonicSequence,
    /// Event and outcome classes are incompatible.
    InvalidEventOutcome,
    /// Subject or state identity is malformed.
    InvalidSemanticField,
    /// Metadata exceeds the reviewed entry bound.
    MetadataTooLarge,
    /// Metadata keys are not strictly ordered and unique.
    MetadataNotOrdered,
    /// Metadata key is not allowlisted.
    MetadataKeyForbidden,
    /// Metadata value is oversized or malformed.
    MetadataValueInvalid,
    /// Metadata appears to contain sensitive or ambient host data.
    SensitiveMetadata,
    /// Recording history reached its fixed bound.
    HistoryFull,
    /// Observer attempted a reentrant recording call.
    ReentrantObservation,
}

/// Stable probe-boundary diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimulationProbeDiagnostic {
    /// Machine-comparable issue code.
    pub code: SimulationProbeDiagnosticCode,
    /// Safe field name involved in the failure.
    pub field: &'static str,
}

/// Validation result for one probe emission.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimulationProbeDisposition {
    /// Record reached the observer.
    Recorded,
    /// Record was rejected without influencing application behavior.
    Rejected,
    /// Valid record was dropped because the recorder was full.
    Dropped,
}

/// Read-only observation provider.
///
/// The callback has no return value and receives immutable facts, so it cannot
/// supply control decisions to authoritative application logic.
pub trait SimulationProbeObserver {
    /// Observes one already validated immutable record.
    fn observe(&mut self, record: &SimulationProbeRecord);
}

/// Behavior-neutral observer that discards validated records.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoopSimulationProbeObserver;

impl SimulationProbeObserver for NoopSimulationProbeObserver {
    fn observe(&mut self, _record: &SimulationProbeRecord) {}
}

/// Fixed-bound recording observer for tests and local evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordingSimulationProbeObserver {
    records: Vec<SimulationProbeRecord>,
    maximum_records: u32,
}

impl Default for RecordingSimulationProbeObserver {
    fn default() -> Self {
        Self::new(MAX_RECORDS)
    }
}

impl RecordingSimulationProbeObserver {
    /// Creates a recorder capped at the smaller of the requested and reviewed bound.
    #[must_use]
    pub fn new(maximum_records: u32) -> Self {
        Self {
            records: Vec::new(),
            maximum_records: maximum_records.min(MAX_RECORDS),
        }
    }

    /// Returns recorded facts in scenario sequence order.
    #[must_use]
    pub fn records(&self) -> &[SimulationProbeRecord] {
        &self.records
    }

    /// Encodes all records with unambiguous record separators.
    #[must_use]
    pub fn normalized_bytes(&self) -> Vec<u8> {
        let mut output = Vec::new();
        for record in &self.records {
            output.extend_from_slice(b"--- probe ---\n");
            output.extend_from_slice(&record.encode());
        }
        output
    }
}

impl SimulationProbeObserver for RecordingSimulationProbeObserver {
    fn observe(&mut self, record: &SimulationProbeRecord) {
        if u32::try_from(self.records.len()).is_ok_and(|len| len < self.maximum_records) {
            self.records.push(record.clone());
        }
    }
}

/// Validating scenario-local probe session.
#[derive(Debug)]
pub struct SimulationProbeSession<O> {
    observer: O,
    previous_sequence: Option<u64>,
    diagnostics: Vec<SimulationProbeDiagnostic>,
    dropped_records: u32,
    observing: bool,
    observed_records: u32,
}

impl<O: SimulationProbeObserver> SimulationProbeSession<O> {
    /// Creates a session around one behavior-neutral observer.
    #[must_use]
    pub const fn new(observer: O) -> Self {
        Self {
            observer,
            previous_sequence: None,
            diagnostics: Vec::new(),
            dropped_records: 0,
            observing: false,
            observed_records: 0,
        }
    }

    /// Validates and emits one record without returning an application decision.
    pub fn emit(
        &mut self,
        registry: &SimulationIdentityRegistry,
        record: &SimulationProbeRecord,
    ) -> SimulationProbeDisposition {
        if self.observing {
            self.note(
                SimulationProbeDiagnosticCode::ReentrantObservation,
                "observer",
            );
            self.dropped_records = self.dropped_records.saturating_add(1);
            return SimulationProbeDisposition::Dropped;
        }
        let issues = validate_probe_record(registry, self.previous_sequence, record);
        if !issues.is_empty() {
            for issue in issues {
                self.note(issue.code, issue.field);
            }
            return SimulationProbeDisposition::Rejected;
        }
        if self.observed_records >= MAX_RECORDS {
            self.note(SimulationProbeDiagnosticCode::HistoryFull, "history");
            self.dropped_records = self.dropped_records.saturating_add(1);
            return SimulationProbeDisposition::Dropped;
        }

        self.observing = true;
        self.observer.observe(record);
        self.observing = false;
        self.previous_sequence = Some(record.sequence);
        self.observed_records += 1;
        SimulationProbeDisposition::Recorded
    }

    /// Returns bounded probe diagnostics.
    #[must_use]
    pub fn diagnostics(&self) -> &[SimulationProbeDiagnostic] {
        &self.diagnostics
    }

    /// Returns the number of valid records dropped by bounds or reentrancy.
    #[must_use]
    pub const fn dropped_records(&self) -> u32 {
        self.dropped_records
    }

    /// Returns the wrapped observer.
    #[must_use]
    pub const fn observer(&self) -> &O {
        &self.observer
    }

    /// Consumes the session and returns the observer.
    #[must_use]
    pub fn into_observer(self) -> O {
        self.observer
    }

    fn note(&mut self, code: SimulationProbeDiagnosticCode, field: &'static str) {
        if self.diagnostics.len() < MAX_DIAGNOSTICS {
            self.diagnostics
                .push(SimulationProbeDiagnostic { code, field });
        }
    }
}

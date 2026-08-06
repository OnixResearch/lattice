/// Validates one record against schema, identity, ordering, and redaction policy.
#[must_use]
pub fn validate_probe_record(
    registry: &SimulationIdentityRegistry,
    previous_sequence: Option<u64>,
    record: &SimulationProbeRecord,
) -> Vec<SimulationProbeDiagnostic> {
    let mut issues = ProbeIssues::default();
    debug_assert!(issues.0.is_empty());
    debug_assert!(record.metadata.len() <= record.metadata.capacity());
    if record.schema_version != PROBE_SCHEMA_VERSION {
        issues.push(issue(
            SimulationProbeDiagnosticCode::UnknownSchemaVersion,
            "schema_version",
        ));
    }
    if !registry.contains(SimulationIdentityKind::Probe, record.probe_id.as_str()) {
        issues.push(issue(
            SimulationProbeDiagnosticCode::UnknownIdentity,
            "probe_id",
        ));
    }
    if !registry.contains(
        SimulationIdentityKind::Scenario,
        record.scenario_id.as_str(),
    ) {
        issues.push(issue(
            SimulationProbeDiagnosticCode::UnknownIdentity,
            "scenario_id",
        ));
    }
    if previous_sequence.is_some_and(|previous| record.sequence <= previous) {
        issues.push(issue(
            SimulationProbeDiagnosticCode::NonMonotonicSequence,
            "sequence",
        ));
    }
    if !valid_outcome(record.event, record.outcome) {
        issues.push(issue(
            SimulationProbeDiagnosticCode::InvalidEventOutcome,
            "outcome",
        ));
    }
    if !valid_semantic_value(&record.subject_id)
        || record
            .state_identity
            .as_deref()
            .is_some_and(|value| !valid_hash(value))
    {
        issues.push(issue(
            SimulationProbeDiagnosticCode::InvalidSemanticField,
            "subject_id/state_identity",
        ));
    }
    validate_metadata(&record.metadata, &mut issues);
    issues.0
}

#[derive(Default)]
struct ProbeIssues(Vec<SimulationProbeDiagnostic>);

impl ProbeIssues {
    fn push(&mut self, issue: SimulationProbeDiagnostic) {
        if self.0.len() < MAX_DIAGNOSTICS {
            self.0.push(issue);
        }
    }
}

fn validate_metadata(metadata: &[SimulationProbeMetadata], issues: &mut ProbeIssues) {
    debug_assert_eq!(metadata.iter().len(), metadata.len());
    debug_assert!(issues.0.len() <= MAX_DIAGNOSTICS);
    if metadata.len() > MAX_METADATA_ENTRIES {
        issues.push(issue(
            SimulationProbeDiagnosticCode::MetadataTooLarge,
            "metadata",
        ));
    }
    let mut previous: Option<&str> = None;
    let mut seen = std::collections::BTreeSet::new();
    for pair in metadata.iter().take(MAX_METADATA_ENTRIES.saturating_add(1)) {
        if previous.is_some_and(|key| key >= pair.key.as_str()) || !seen.insert(&pair.key) {
            issues.push(issue(
                SimulationProbeDiagnosticCode::MetadataNotOrdered,
                "metadata.key",
            ));
        }
        previous = Some(&pair.key);
        if pair.key.len() > MAX_METADATA_KEY_BYTES || !allowlisted_key(&pair.key) {
            issues.push(issue(
                SimulationProbeDiagnosticCode::MetadataKeyForbidden,
                "metadata.key",
            ));
        }
        if pair.value.len() > MAX_METADATA_VALUE_BYTES || !valid_semantic_value(&pair.value) {
            issues.push(issue(
                SimulationProbeDiagnosticCode::MetadataValueInvalid,
                "metadata.value",
            ));
        }
        if sensitive_value(&pair.value) {
            issues.push(issue(
                SimulationProbeDiagnosticCode::SensitiveMetadata,
                "metadata.value",
            ));
        }
    }
}

fn allowlisted_key(key: &str) -> bool {
    matches!(
        key,
        "adapter.version"
            | "choice.index"
            | "choice.point"
            | "diagnostic.code"
            | "effect.class"
            | "fault.control"
            | "invalidation.class"
            | "message.class"
            | "oracle.category"
            | "oracle.condition"
            | "request.id"
            | "state.projection"
            | "transition.kind"
    )
}

fn valid_semantic_value(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_METADATA_VALUE_BYTES
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_' | b':' | b'+' | b'=')
        })
}

fn sensitive_value(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    value.starts_with('/')
        || value.contains("../")
        || value.contains('\\')
        || lower.contains("password")
        || lower.contains("credential")
        || lower.contains("secret")
        || lower.contains("token=")
        || lower.contains("clipboard")
        || lower.contains("terminal-output")
        || lower.contains("process-output")
        || lower.contains("debug(")
}

fn valid_hash(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

const fn valid_outcome(event: SimulationProbeEvent, outcome: SimulationProbeOutcome) -> bool {
    match event {
        SimulationProbeEvent::MessageDelivered
        | SimulationProbeEvent::DiagnosticProduced
        | SimulationProbeEvent::InvalidationClassified
        | SimulationProbeEvent::StateCheckpoint => {
            matches!(outcome, SimulationProbeOutcome::Observed)
        }
        SimulationProbeEvent::TransitionEvaluated => matches!(
            outcome,
            SimulationProbeOutcome::Accepted
                | SimulationProbeOutcome::Rejected
                | SimulationProbeOutcome::Ignored
        ),
        SimulationProbeEvent::EffectLifecycle => matches!(
            outcome,
            SimulationProbeOutcome::Requested
                | SimulationProbeOutcome::Started
                | SimulationProbeOutcome::Completed
                | SimulationProbeOutcome::Failed
                | SimulationProbeOutcome::Cancelled
                | SimulationProbeOutcome::Replaced
                | SimulationProbeOutcome::TimedOut
                | SimulationProbeOutcome::SourceClosed
        ),
        SimulationProbeEvent::ChoiceSelected => {
            matches!(outcome, SimulationProbeOutcome::Selected)
        }
        SimulationProbeEvent::OracleObserved => matches!(
            outcome,
            SimulationProbeOutcome::Passed
                | SimulationProbeOutcome::FailedOracle
                | SimulationProbeOutcome::Unobserved
        ),
    }
}

const fn issue(
    code: SimulationProbeDiagnosticCode,
    field: &'static str,
) -> SimulationProbeDiagnostic {
    SimulationProbeDiagnostic { code, field }
}

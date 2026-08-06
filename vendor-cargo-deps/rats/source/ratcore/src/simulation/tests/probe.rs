use super::super::*;

fn valid_record(sequence: u64) -> SimulationProbeRecord {
    SimulationProbeRecord::new(
        "simulation.probe.transition-v1",
        "simulation.scenario.workspace-effect-v1",
        sequence,
        SimulationProbeSemantic::new(
            SimulationProbeEvent::TransitionEvaluated,
            "message.logical-tick",
            SimulationProbeOutcome::Accepted,
        ),
    )
    .with_logical_tick(10)
    .with_metadata(vec![SimulationProbeMetadata::new("transition.kind", "accepted")])
}

#[test]
fn deterministic_probe_encoding_and_recording_are_byte_stable() {
    let registry = super::registry();
    let mut first = SimulationProbeSession::new(RecordingSimulationProbeObserver::default());
    let mut second = SimulationProbeSession::new(RecordingSimulationProbeObserver::default());
    for sequence in 1..=3 {
        assert_eq!(first.emit(&registry, &valid_record(sequence)), SimulationProbeDisposition::Recorded);
        assert_eq!(second.emit(&registry, &valid_record(sequence)), SimulationProbeDisposition::Recorded);
    }

    assert_eq!(first.observer().normalized_bytes(), second.observer().normalized_bytes());
    assert!(first.diagnostics().is_empty());
}

#[test]
fn unknown_schema_identity_and_invalid_event_outcome_are_rejected() {
    let registry = super::registry();
    let mut record = valid_record(1);
    record.schema_version = 2;
    record.probe_id = SimulationProbeId::new("simulation.probe.unknown-v1");
    record.outcome = SimulationProbeOutcome::Started;
    let issues = validate_probe_record(&registry, None, &record);
    let codes: Vec<_> = issues.iter().map(|issue| issue.code).collect();

    assert!(codes.contains(&SimulationProbeDiagnosticCode::UnknownSchemaVersion));
    assert!(codes.contains(&SimulationProbeDiagnosticCode::UnknownIdentity));
    assert!(codes.contains(&SimulationProbeDiagnosticCode::InvalidEventOutcome));
}

#[test]
fn duplicate_regressing_and_unordered_metadata_fail_closed() {
    let registry = super::registry();
    let mut session = SimulationProbeSession::new(RecordingSimulationProbeObserver::default());
    assert_eq!(session.emit(&registry, &valid_record(4)), SimulationProbeDisposition::Recorded);
    assert_eq!(session.emit(&registry, &valid_record(4)), SimulationProbeDisposition::Rejected);
    assert_eq!(session.emit(&registry, &valid_record(3)), SimulationProbeDisposition::Rejected);

    let unordered = valid_record(5).with_metadata(vec![
        SimulationProbeMetadata::new("transition.kind", "accepted"),
        SimulationProbeMetadata::new("message.class", "logical-tick"),
    ]);
    let issues = validate_probe_record(&registry, Some(4), &unordered);
    assert!(issues.iter().any(|issue| { issue.code == SimulationProbeDiagnosticCode::MetadataNotOrdered }));
}

#[test]
fn oversized_forbidden_and_sensitive_metadata_never_reaches_observer() {
    let registry = super::registry();
    let mut metadata = vec![SimulationProbeMetadata::new("raw.text", "credential=secret")];
    for index in 0..13 {
        metadata.push(SimulationProbeMetadata::new("message.class", format!("value-{index}")));
    }
    let record = valid_record(1).with_metadata(metadata);
    let mut session = SimulationProbeSession::new(RecordingSimulationProbeObserver::default());

    assert_eq!(session.emit(&registry, &record), SimulationProbeDisposition::Rejected);
    assert!(session.observer().records().is_empty());
    let codes: Vec<_> = session.diagnostics().iter().map(|issue| issue.code).collect();
    assert!(codes.contains(&SimulationProbeDiagnosticCode::MetadataTooLarge));
    assert!(codes.contains(&SimulationProbeDiagnosticCode::MetadataKeyForbidden));
    assert!(codes.contains(&SimulationProbeDiagnosticCode::SensitiveMetadata));
}

#[test]
fn ambient_timestamp_and_path_shaped_values_are_not_allowlisted() {
    let registry = super::registry();
    let timestamp =
        valid_record(1).with_metadata(vec![SimulationProbeMetadata::new("timestamp", "2026-07-16T10:00:00Z")]);
    let path =
        valid_record(1).with_metadata(vec![SimulationProbeMetadata::new("message.class", "/home/operator/private")]);

    assert!(
        validate_probe_record(&registry, None, &timestamp)
            .iter()
            .any(|issue| issue.code == SimulationProbeDiagnosticCode::MetadataKeyForbidden)
    );
    assert!(
        validate_probe_record(&registry, None, &path)
            .iter()
            .any(|issue| issue.code == SimulationProbeDiagnosticCode::SensitiveMetadata)
    );
}

#[test]
fn fixed_history_bound_counts_drops_without_panicking() {
    let registry = super::registry();
    let mut session = SimulationProbeSession::new(NoopSimulationProbeObserver);
    for sequence in 1..=256 {
        assert_eq!(session.emit(&registry, &valid_record(sequence)), SimulationProbeDisposition::Recorded);
    }
    assert_eq!(session.emit(&registry, &valid_record(257)), SimulationProbeDisposition::Dropped);
    assert_eq!(session.dropped_records(), 1);
    assert!(
        session
            .diagnostics()
            .iter()
            .any(|issue| { issue.code == SimulationProbeDiagnosticCode::HistoryFull })
    );
}

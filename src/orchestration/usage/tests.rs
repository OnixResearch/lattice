use super::*;

#[test]
fn owned_outcomes_refine_missing_evidence_but_never_conflict_or_downgrade() {
    for cancelled in [false, true] {
        let partial = RunOutcomeObservation {
            succeeded: false,
            cancelled: None,
        };
        let known = RunOutcomeObservation {
            succeeded: false,
            cancelled: Some(cancelled),
        };
        let mut accounting = RunAccounting {
            schema: RUN_USAGE_ACCOUNTING_SCHEMA.to_string(),
            reserved: super::super::core::BudgetCharge::default(),
            charged: super::super::core::BudgetCharge::default(),
            host_interval: None,
            provider_reports: Vec::new(),
            unknown_tokens: true,
            unknown_cost: true,
            unknown_model_requests: true,
            unknown_subagent_calls: true,
            diagnostic: "outcome refinement".to_string(),
            outcome: Some(partial),
        };
        assert_eq!(accounting.refine_outcome(known), Ok(()));
        assert_eq!(accounting.outcome, Some(known));
        assert_eq!(accounting.refine_outcome(known), Ok(()));
        assert_eq!(accounting.refine_outcome(partial), Ok(()));
        assert_eq!(accounting.outcome, Some(known));
        assert_eq!(
            accounting.refine_outcome(RunOutcomeObservation {
                cancelled: Some(!cancelled),
                ..known
            }),
            Err(ObservationError::ConflictingOutcome)
        );
        assert_eq!(accounting.outcome, Some(known));
        assert_eq!(
            accounting.refine_outcome(RunOutcomeObservation {
                succeeded: true,
                cancelled: Some(false)
            }),
            Err(ObservationError::ConflictingOutcome)
        );
        assert_eq!(accounting.outcome, Some(known));
        assert_eq!(
            accounting.refine_outcome(RunOutcomeObservation {
                succeeded: true,
                cancelled: None
            }),
            Err(ObservationError::ConflictingOutcome)
        );
        assert_eq!(accounting.outcome, Some(known));
        // Historical receipts without an outcome can acquire exact evidence.
        accounting.outcome = None;
        assert_eq!(accounting.refine_outcome(known), Ok(()));
        assert_eq!(accounting.outcome, Some(known));
        accounting.outcome = Some(partial);
        assert_eq!(
            accounting.refine_outcome(RunOutcomeObservation {
                succeeded: true,
                cancelled: Some(false)
            }),
            Err(ObservationError::ConflictingOutcome)
        );
        assert_eq!(accounting.outcome, Some(partial));
    }
}

pub(super) fn observation() -> CommandWorkerObservation {
    CommandWorkerObservation {
        schema: USAGE_OBSERVATION_SCHEMA.to_string(),
        scope: Scope::Run,
        workflow_revision: "revision-1".to_string(),
        instance_id: "instance-1".to_string(),
        dispatch_id: "dispatch-1".to_string(),
        run_id: 7,
        node_id: 3,
        invocation_id: "invocation-1".to_string(),
        worker_profile: "profile-1".to_string(),
        attempt_id: "attempt-1".to_string(),
        agent_invocations: Some(1),
        model_requests: Some(2),
        subagent_calls: Some(1),
        tokens: Some(1_234),
        cost_microunits: Some(567),
        duration_millis: Some(42),
        provenance: UsageProvenance::ProviderReported,
        complete: true,
        accounting: None,
    }
}

#[test]
fn complete_provider_observation_is_not_host_measurement() {
    let observation = observation();
    assert_eq!(observation.validate(), Ok(()));
    assert_eq!(observation.provenance, UsageProvenance::ProviderReported);
    assert_eq!(observation.tokens, Some(1_234));
}

#[test]
fn unknown_and_estimated_observations_do_not_claim_measurements() {
    let mut unknown = observation();
    unknown.provenance = UsageProvenance::Unknown;
    unknown.complete = false;
    unknown.agent_invocations = None;
    unknown.model_requests = None;
    unknown.subagent_calls = None;
    unknown.tokens = None;
    unknown.cost_microunits = None;
    unknown.duration_millis = None;
    assert_eq!(unknown.validate(), Ok(()));
    assert!(unknown.tokens.is_none());

    let mut estimated = observation();
    estimated.provenance = UsageProvenance::Estimated;
    assert!(estimated.is_estimated_or_unknown());
}

#[test]
fn invalid_shapes_are_rejected() {
    let mut unknown_with_values = observation();
    unknown_with_values.provenance = UsageProvenance::Unknown;
    assert_eq!(unknown_with_values.validate(), Err(ObservationError::UnknownCarriesValues));

    let mut empty = observation();
    empty.agent_invocations = None;
    empty.model_requests = None;
    empty.subagent_calls = None;
    empty.tokens = None;
    empty.cost_microunits = None;
    empty.duration_millis = None;
    assert_eq!(empty.validate(), Err(ObservationError::EmptyCompleteObservation));

    let mut identifier = observation();
    identifier.attempt_id.clear();
    assert_eq!(identifier.validate(), Err(ObservationError::InvalidIdentifier));
}

#[test]
fn observation_json_round_trips_after_admission() {
    let observation = observation();
    let json = observation.to_json().expect("observation serializes");
    assert_eq!(CommandWorkerObservation::from_json(&json), Ok(observation));
    assert!(CommandWorkerObservation::from_json("{bad").is_err());
    assert!(CommandWorkerObservation::from_json(&json[..json.len() - 1]).is_err());
    let unknown_field = json.replacen('{', "{\"invented\":true,", 1);
    assert!(CommandWorkerObservation::from_json(&unknown_field).is_err());
    let duplicate = json.replacen('{', "{\"schema\":\"lattice.command-worker-usage.v1\",", 1);
    assert!(CommandWorkerObservation::from_json(&duplicate).is_err());
    let unsupported = json.replace(USAGE_OBSERVATION_SCHEMA, "lattice.command-worker-usage.v999");
    assert!(CommandWorkerObservation::from_json(&unsupported).is_err());
}

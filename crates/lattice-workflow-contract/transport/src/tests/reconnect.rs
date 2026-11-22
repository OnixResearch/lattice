use super::*;

const TEST_VSOCK_PORT: u32 = 10_300;
const TEST_HOST_CID: u32 = 2;
const TEST_SECOND_CID: u32 = 3;

pub(super) fn vsock_profile(allowed_cids: Vec<u32>) -> VsockTransportProfile {
    VsockTransportProfile {
        schema: VSOCK_TRANSPORT_PROFILE_SCHEMA.to_string(),
        port: TEST_VSOCK_PORT,
        limits: transport_profile(TEST_PROFILE_USER_ID).limits,
        peers: VsockPeerPolicy { allowed_cids },
        non_claims: crate::REQUIRED_NON_CLAIMS.iter().map(|value| (*value).to_string()).collect(),
    }
}

// r[verify lattice.workflow_interchange.vsock_profile]
// r[verify lattice.workflow_interchange.vsock_profile.scenario.admitted]
#[test]
fn vsock_profile_admits_sorted_bounded_peer_cids() {
    let profile = vsock_profile(vec![TEST_HOST_CID, TEST_SECOND_CID]);
    assert_eq!(validate_vsock_transport_profile(&profile), Ok(()));
    assert!(vsock_peer_is_admitted(&profile, TEST_HOST_CID));
    assert!(!vsock_peer_is_admitted(&profile, TEST_SECOND_CID.saturating_add(1)));
    assert!(!vsock_peer_is_admitted(&profile, VSOCK_CID_ANY));
}

// r[verify lattice.workflow_interchange.vsock_profile.scenario.refused]
#[test]
fn vsock_profile_refuses_every_invalid_field() {
    let valid = vsock_profile(vec![TEST_HOST_CID]);
    let mut schema = valid.clone();
    schema.schema = TRANSPORT_PROFILE_SCHEMA.to_string();
    assert_eq!(validate_vsock_transport_profile(&schema), Err(AdmissionError::ProfileSchema));
    for port in [0, VSOCK_PORT_ANY] {
        let mut invalid = valid.clone();
        invalid.port = port;
        assert_eq!(validate_vsock_transport_profile(&invalid), Err(AdmissionError::ProfileLimit));
    }
    let mut limits = valid.clone();
    limits.limits.read_deadline_milliseconds = 0;
    assert_eq!(validate_vsock_transport_profile(&limits), Err(AdmissionError::ProfileLimit));
    let too_many = (1..=MAXIMUM_ALLOWED_CIDS.saturating_add(1)).collect::<Vec<_>>();
    for cids in [
        Vec::new(),
        vec![TEST_SECOND_CID, TEST_HOST_CID],
        vec![TEST_HOST_CID, TEST_HOST_CID],
        vec![TEST_HOST_CID, VSOCK_CID_ANY],
        too_many,
    ] {
        let invalid = vsock_profile(cids);
        assert_eq!(validate_vsock_transport_profile(&invalid), Err(AdmissionError::PeerPolicy));
    }
    let mut claims = valid.clone();
    claims.non_claims.pop();
    assert_eq!(validate_vsock_transport_profile(&claims), Err(AdmissionError::ProfileSchema));
    let encoded = serde_json::to_string(&valid).expect("vsock profile encodes");
    let unknown = encoded.replacen("\"port\":", "\"cid\":3,\"port\":", 1);
    assert!(serde_json::from_str::<VsockTransportProfile>(&unknown).is_err());
}

// r[verify lattice.workflow_interchange.reconnect_rules]
#[test]
fn reconnect_rules_recover_instead_of_redispatching() {
    let unknown = ExchangeOutcome::Failure(FailureClass::UnknownAfterWrite);
    let not_accepted = ExchangeOutcome::Failure(FailureClass::NotAccepted);
    let not_found = ExchangeOutcome::Rejected(RejectionCode::NotFound);
    assert_eq!(next_step(CommandKind::Dispatch, not_accepted, false), RecoveryStep::ResendIdentical);
    assert_eq!(next_step(CommandKind::Dispatch, unknown, true), RecoveryStep::RecoverDispatch);
    assert_eq!(next_step(CommandKind::RecoverDispatch, not_found, true), RecoveryStep::ResendDispatch);
    assert_eq!(next_step(CommandKind::RecoverDispatch, not_found, false), RecoveryStep::StopUnknown);
    assert_eq!(next_step(CommandKind::Control, not_accepted, false), RecoveryStep::ResendIdentical);
    assert_eq!(next_step(CommandKind::Control, unknown, true), RecoveryStep::ObserveThenCompare);
    assert_eq!(next_step(CommandKind::Observe, unknown, false), RecoveryStep::RetryBounded);
    assert_eq!(next_step(CommandKind::ReadLog, not_accepted, false), RecoveryStep::RetryBounded);
    assert_eq!(next_step(CommandKind::Reconcile, unknown, false), RecoveryStep::RetryBounded);
    assert_eq!(next_step(CommandKind::Dispatch, ExchangeOutcome::Success, false), RecoveryStep::Complete);
    assert_eq!(next_step(CommandKind::Dispatch, not_found, true), RecoveryStep::StopRejected);
    let invalid = ExchangeOutcome::Failure(FailureClass::InvalidRequest);
    assert_eq!(next_step(CommandKind::Dispatch, invalid, true), RecoveryStep::StopInvalid);
    assert_eq!(control_after_observe(TEST_SEQUENCE, TEST_SEQUENCE), RecoveryStep::ResendIdentical);
    assert_eq!(control_after_observe(TEST_SEQUENCE, TEST_SEQUENCE.saturating_add(1)), RecoveryStep::StopSuperseded);
    assert_eq!(control_after_observe(TEST_SEQUENCE, TEST_SEQUENCE.saturating_sub(1)), RecoveryStep::StopUnknown);
    assert_eq!(
        CommandKind::of(&dispatch_request().command),
        CommandKind::Dispatch,
        "command kind follows the wire operation"
    );
    assert_eq!(ExchangeOutcome::of(&Err(ClientFailure::UnknownAfterWrite)), unknown);
}

// r[verify lattice.workflow_interchange.observation_continuity]
#[test]
fn continuity_rejects_forks_and_names_gaps() {
    let profile = contract_profile();
    let request = workflow_request(&profile);
    let accepted = accepted(&request);
    let previous = observation(&request, &accepted);
    assert_eq!(continuity(None, &previous), Continuity::Continuous);
    assert_eq!(continuity(Some(&previous), &previous), Continuity::Continuous);

    let mut older = previous.clone();
    older.sequence = TEST_SEQUENCE.saturating_sub(1);
    assert_eq!(continuity(Some(&previous), &older), Continuity::Reject {
        reason: ContinuityRejection::SequenceDecreased
    });

    let mut reverted = previous.clone();
    reverted.sequence = TEST_SEQUENCE.saturating_add(1);
    reverted.state = ExecutionState::Unknown;
    reverted.terminal = false;
    assert_eq!(continuity(Some(&previous), &reverted), Continuity::Reject {
        reason: ContinuityRejection::TerminalReverted
    });

    let mut gap = previous.clone();
    for reference in &mut gap.logs {
        reference.gap_before = true;
    }
    let node_id = previous.logs.first().expect("test log exists").node_id.clone();
    assert_eq!(continuity(Some(&previous), &gap), Continuity::Gap {
        nodes: vec![node_id.clone()],
        streams: vec![LogStreamKey {
            node_id,
            stream: LogStream::Stdout,
        }],
    });

    let mut advanced = previous.clone();
    for reference in &mut advanced.logs {
        reference.cursor = TEST_SEQUENCE;
    }
    assert_eq!(continuity(Some(&advanced), &previous), Continuity::Reject {
        reason: ContinuityRejection::CursorRegressed
    });
}

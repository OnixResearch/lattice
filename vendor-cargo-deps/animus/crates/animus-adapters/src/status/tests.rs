const TARGET_ID: &str = "pane:alpha";
const SOURCE_ID: &str = "lifecycle:herdr";
const AGENT_LABEL: &str = "agent-alpha";
const SESSION_VALUE: &str = "session-1";
const VALID_SEQUENCE: crate::StatusSequence = crate::StatusSequence::new(7);
const VALID_TTL: crate::StatusTtl = crate::StatusTtl::new(crate::StatusSequence::new(9));
const VALID_HASH: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const UPPERCASE_HASH: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdeF";
const SHORT_HASH: &str = "0123456789abcdef";
const MALFORMED_STATE: &str = "busy";
const MALFORMED_SOURCE: &str = "bad\0source";
const UNSUPPORTED_SESSION_KIND: &str = "remote";
const NUL_BYTE_INDEX: usize = 3;

struct StaticAdapter {
    batch: crate::HostStatusObservationBatch,
}

impl crate::ComponentAdapter<crate::HostStatusRequest, crate::HostStatusObservationBatch>
    for StaticAdapter
{
    type Error = crate::AdapterContractError;

    fn call(
        &mut self,
        _request: &crate::HostStatusRequest,
    ) -> Result<crate::HostStatusObservationBatch, Self::Error> {
        Ok(self.batch.clone())
    }
}

fn valid_draft() -> crate::AgentStatusObservationDraft<'static> {
    crate::AgentStatusObservationDraft {
        kind: "trusted_lifecycle",
        state: "working",
        source_id: SOURCE_ID,
        target_id: TARGET_ID,
        agent_label: AGENT_LABEL,
        session: Some(crate::AgentSessionReferenceDraft {
            kind: "native",
            value: SESSION_VALUE,
        }),
        message: Some("running tests"),
        presentation: crate::AgentStatusPresentationDraft {
            title: Some("Build pane"),
            display_agent_name: Some("Ralph"),
            activity_text: Some("checking"),
            custom_state_label: Some("needs input"),
        },
        sequence: VALID_SEQUENCE,
        ttl: Some(VALID_TTL),
        evidence_hashes: &[VALID_HASH],
    }
}

#[test]
fn lifecycle_report_validation_accepts_valid_observation() {
    let observation = crate::AgentStatusObservation::new(valid_draft())
        .expect("valid status observation should parse");

    assert_eq!(
        observation.kind(),
        crate::AgentStatusObservationKind::TrustedLifecycle
    );
    assert_eq!(observation.state(), crate::SemanticAgentState::Working);
    assert_eq!(observation.source_id(), SOURCE_ID);
    assert_eq!(observation.target_id(), TARGET_ID);
    assert_eq!(observation.agent_label(), AGENT_LABEL);
    assert_eq!(observation.message(), Some("running tests"));
    assert_eq!(observation.sequence(), VALID_SEQUENCE);
    assert_eq!(observation.ttl(), Some(VALID_TTL));
    assert_eq!(
        observation.evidence_hashes(),
        &[alloc::string::String::from(VALID_HASH)]
    );
    assert_eq!(
        observation.presentation().custom_state_label(),
        Some("needs input")
    );
    let session = observation.session().expect("session should be present");
    assert_eq!(session.kind(), crate::AgentSessionReferenceKind::Native);
    assert_eq!(session.value(), SESSION_VALUE);
}

#[test]
fn host_adapter_uses_typed_observation_contract() {
    let observation = crate::AgentStatusObservation::new(valid_draft())
        .expect("valid status observation should parse");
    let mut adapter = StaticAdapter {
        batch: crate::HostStatusObservationBatch::new(alloc::vec![observation.clone()]),
    };
    let request = crate::HostStatusRequest::new(TARGET_ID).expect("valid target should parse");
    let batch = <StaticAdapter as crate::ComponentAdapter<
        crate::HostStatusRequest,
        crate::HostStatusObservationBatch,
    >>::call(&mut adapter, &request)
    .expect("static adapter should return its typed batch");

    assert_eq!(request.target_id(), TARGET_ID);
    assert_eq!(batch.observations(), &[observation]);
}

#[test]
fn lifecycle_report_rejects_malformed_state() {
    let mut draft = valid_draft();
    draft.state = MALFORMED_STATE;

    let error =
        crate::AgentStatusObservation::new(draft).expect_err("malformed state should be rejected");

    assert_eq!(
        error,
        crate::AdapterContractError::UnsupportedStatusState {
            field: "semantic_status_state",
            value: alloc::string::String::from(MALFORMED_STATE),
        }
    );
}

#[test]
fn lifecycle_report_rejects_malformed_source() {
    let mut draft = valid_draft();
    draft.source_id = MALFORMED_SOURCE;

    let error =
        crate::AgentStatusObservation::new(draft).expect_err("malformed source should be rejected");

    assert_eq!(
        error,
        crate::AdapterContractError::NulCharacter {
            field: "status_source_id",
            byte_index: NUL_BYTE_INDEX,
        }
    );
}

#[test]
fn lifecycle_report_rejects_missing_target() {
    let mut draft = valid_draft();
    draft.target_id = " ";

    let error =
        crate::AgentStatusObservation::new(draft).expect_err("missing target should be rejected");

    assert_eq!(
        error,
        crate::AdapterContractError::EmptyField {
            field: "status_target_id",
        }
    );
}

#[test]
fn lifecycle_report_rejects_unsupported_session_reference() {
    let mut draft = valid_draft();
    draft.session = Some(crate::AgentSessionReferenceDraft {
        kind: UNSUPPORTED_SESSION_KIND,
        value: SESSION_VALUE,
    });

    let error = crate::AgentStatusObservation::new(draft)
        .expect_err("unsupported session reference should be rejected");

    assert_eq!(
        error,
        crate::AdapterContractError::UnsupportedSessionReferenceKind {
            field: "session_reference_kind",
            value: alloc::string::String::from(UNSUPPORTED_SESSION_KIND),
        }
    );
}

#[test]
fn lifecycle_report_rejects_invalid_evidence_hashes() {
    let mut short_draft = valid_draft();
    short_draft.evidence_hashes = &[SHORT_HASH];
    let short_error = crate::AgentStatusObservation::new(short_draft)
        .expect_err("short evidence hash should fail");

    assert_eq!(
        short_error,
        crate::AdapterContractError::InvalidEvidenceHash {
            field: "status_evidence_hash",
            reason: "evidence hash must use the configured BLAKE3 hex length",
        }
    );

    let mut uppercase_draft = valid_draft();
    uppercase_draft.evidence_hashes = &[UPPERCASE_HASH];
    let uppercase_error = crate::AgentStatusObservation::new(uppercase_draft)
        .expect_err("uppercase evidence hash should fail");

    assert_eq!(
        uppercase_error,
        crate::AdapterContractError::InvalidEvidenceHash {
            field: "status_evidence_hash",
            reason: "evidence hash must contain lowercase hex only",
        }
    );
}

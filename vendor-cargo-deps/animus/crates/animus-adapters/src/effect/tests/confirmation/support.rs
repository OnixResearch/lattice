use super::super::super::*;

pub(super) const VALID_POLICY_HASH: &str =
    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
pub(super) const OTHER_POLICY_HASH: &str =
    "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
pub(super) const SUMMARY: &str = "redacted request summary";
pub(super) const RAW_PAYLOAD: &str = "RAW_PROMPT: do not leak this text";
pub(super) const REQUEST_ID: &str = "confirm-request-alpha";
pub(super) const OTHER_REQUEST_ID: &str = "confirm-request-beta";
pub(super) const RESOURCE_LABEL: &str = "workspace-target";
pub(super) const OTHER_RESOURCE_LABEL: &str = "other-workspace-target";
pub(super) const ABILITY_LABEL: &str = "mutate";
pub(super) const SESSION_IDENTITY: &str = "session-alpha";
pub(super) const FRESH_TICK_VALUE: u64 = 7;
pub(super) const EXPIRY_TICK_VALUE: u64 = 10;
pub(super) const STALE_TICK_VALUE: u64 = 11;
pub(super) const EXPECTED_BROKER_CALLS: usize = 1;
const BROKER_CALL_INCREMENT: usize = 1;
pub(super) const NO_PAYLOADS: u32 = 0;
pub(super) const NO_BROKER_CALLS: usize = 0;

pub(super) struct RecordingBroker {
    pub(super) outcome: ServiceOutcome<ConfirmationDecision>,
    pub(super) calls: usize,
}

impl crate::ComponentAdapter<ConfirmationRequest, ServiceOutcome<ConfirmationDecision>>
    for RecordingBroker
{
    type Error = ();

    fn call(
        &mut self,
        _request: &ConfirmationRequest,
    ) -> Result<ServiceOutcome<ConfirmationDecision>, Self::Error> {
        self.calls = self.calls.saturating_add(BROKER_CALL_INCREMENT);
        Ok(self.outcome.clone())
    }
}

pub(super) fn protected_request(summary: &str) -> ConfirmationRequest {
    let policy_hash = SafeHash::new(VALID_POLICY_HASH).expect("policy hash should parse");
    ConfirmationRequest::from_draft(ConfirmationRequestDraft {
        request_id: REQUEST_ID,
        action_kind: ConfirmationActionKind::ToolExecution,
        resource: RESOURCE_LABEL,
        ability: ABILITY_LABEL,
        safe_summary: summary,
        policy_hash: &policy_hash,
        session_identity: SESSION_IDENTITY,
        confirmation_level: ConfirmationLevel::HighRisk,
        expiry: Some(ConfirmationExpiry::new(ConfirmationTick::new(
            EXPIRY_TICK_VALUE,
        ))),
        redaction: RedactionMarker::PayloadHashOnly,
    })
    .expect("typed confirmation request should parse")
}

pub(super) fn service_receipt(outcome: OutcomeKind, reason: Reason) -> Receipt {
    Receipt::new(
        ReceiptDraft::new(
            ServiceKind::Confirmation,
            CapabilityClass::ToolExecution,
            outcome,
            reason,
        )
        .with_payload_count(NO_PAYLOADS)
        .with_redaction(RedactionMarker::PayloadNotCaptured),
    )
}

pub(super) fn evaluated_issue(
    request: &ConfirmationRequest,
    binding: ConfirmationBinding,
    tick: ConfirmationTick,
) -> ConfirmationIssue {
    let decision = ConfirmationDecision::Approved { binding };
    let evaluation =
        evaluate_confirmation_decision(request, &decision, ConfirmationBrokerClass::Injected, tick);

    assert!(!evaluation.dispatch_allowed());
    assert_eq!(evaluation.status(), ConfirmationStatus::Denied);
    evaluation.issue()
}

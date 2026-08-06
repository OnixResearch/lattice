use super::super::super::*;

#[test]
fn absent_broker_denies_by_default_without_raw_payload() {
    let request = ConfirmationRequest::new(CapabilityClass::ToolExecution, super::support::SUMMARY)
        .expect("confirmation request should parse");
    let outcome = default_deny_confirmation(&request);

    assert_eq!(outcome.kind(), OutcomeKind::Denied);
    assert_eq!(outcome.receipt().service_kind(), ServiceKind::Confirmation);
    assert_eq!(outcome.receipt().reason(), Reason::NotInjected);
    assert_eq!(outcome.receipt().payload_count(), super::support::NO_PAYLOADS);
    assert_eq!(outcome.receipt().redaction(), RedactionMarker::PayloadNotCaptured);
    assert!(!std::format!("{outcome:?}").contains(super::support::RAW_PAYLOAD));
}

#[test]
fn blank_summary_is_rejected() {
    let error = ConfirmationRequest::new(CapabilityClass::Confirmation, "   ")
        .expect_err("blank confirmation summary should fail");

    assert!(matches!(error, crate::AdapterContractError::EmptyField { .. }));
}

#[test]
fn typed_envelope_records_action_identity_without_payload_receipt() {
    let request = super::support::protected_request(super::support::RAW_PAYLOAD);
    let receipt = ConfirmationReceipt::new(ConfirmationReceiptDraft {
        request: &request,
        decision_status: ConfirmationStatus::Denied,
        issue: ConfirmationIssue::BrokerDenied,
        broker_class: ConfirmationBrokerClass::Injected,
        timing_class: ConfirmationTimingClass::NotApplicable,
        expiry_class: ConfirmationExpiryClass::Active,
    });

    assert_eq!(request.request_id(), super::support::REQUEST_ID);
    assert_eq!(request.action_kind(), ConfirmationActionKind::ToolExecution);
    assert_eq!(request.resource(), super::support::RESOURCE_LABEL);
    assert_eq!(request.ability(), super::support::ABILITY_LABEL);
    assert_eq!(request.summary(), super::support::RAW_PAYLOAD);
    assert_eq!(request.confirmation_level(), ConfirmationLevel::HighRisk);
    assert_eq!(receipt.action_kind(), ConfirmationActionKind::ToolExecution);
    assert_ne!(receipt.resource_hash().as_str(), super::support::RESOURCE_LABEL);
    assert_ne!(receipt.request_id_hash().as_str(), super::support::REQUEST_ID);
    assert_eq!(receipt.redaction(), RedactionMarker::PayloadHashOnly);
    assert!(!std::format!("{receipt:?}").contains(super::support::RAW_PAYLOAD));
}

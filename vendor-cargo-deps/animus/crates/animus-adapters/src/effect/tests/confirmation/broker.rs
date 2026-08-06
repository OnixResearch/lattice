use super::super::super::*;
use crate::ComponentAdapter;

#[test]
fn host_owned_path_uses_injected_contract() {
    let request = super::support::protected_request(super::support::SUMMARY);
    let outcome = ServiceOutcome::completed(
        ConfirmationDecision::Approved {
            binding: ConfirmationBinding::from_request(&request),
        },
        super::support::service_receipt(OutcomeKind::Completed, Reason::Completed),
    );
    let mut broker = super::support::RecordingBroker {
        outcome,
        calls: super::support::NO_BROKER_CALLS,
    };

    let broker_outcome = broker.call(&request).expect("recording broker should return typed outcome");
    let evaluation = evaluate_confirmation_outcome(
        &request,
        &broker_outcome,
        ConfirmationTick::new(super::support::FRESH_TICK_VALUE),
    );

    assert_eq!(broker.calls, super::support::EXPECTED_BROKER_CALLS);
    assert!(evaluation.dispatch_allowed());
}

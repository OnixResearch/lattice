use super::super::super::*;

#[test]
fn approved_binding_allows_dispatch_when_fresh() {
    let request = super::support::protected_request(super::support::SUMMARY);
    let binding = ConfirmationBinding::from_request(&request);
    let decision = ConfirmationDecision::Approved { binding };
    let outcome = ServiceOutcome::completed(
        decision,
        super::support::service_receipt(OutcomeKind::Completed, Reason::Completed),
    );

    let evaluation = evaluate_confirmation_outcome(
        &request,
        &outcome,
        ConfirmationTick::new(super::support::FRESH_TICK_VALUE),
    );

    assert!(evaluation.dispatch_allowed());
    assert_eq!(evaluation.status(), ConfirmationStatus::Approved);
    assert_eq!(evaluation.issue(), ConfirmationIssue::None);
    assert_eq!(
        evaluation.receipt().expiry_class(),
        ConfirmationExpiryClass::Active
    );
    assert_eq!(
        evaluation.receipt().timing_class(),
        ConfirmationTimingClass::Fresh
    );
}

#[test]
fn non_completed_or_empty_broker_outcomes_fail_closed() {
    let request = super::support::protected_request(super::support::SUMMARY);
    let outcomes: [ServiceOutcome<ConfirmationDecision>; 5] = [
        ServiceOutcome::unavailable(super::support::service_receipt(
            OutcomeKind::Unavailable,
            Reason::HostUnavailable,
        )),
        ServiceOutcome::cancelled(super::support::service_receipt(
            OutcomeKind::Cancelled,
            Reason::Cancelled,
        )),
        ServiceOutcome::timed_out(super::support::service_receipt(
            OutcomeKind::TimedOut,
            Reason::DeadlineExceeded,
        )),
        ServiceOutcome::malformed_output(super::support::service_receipt(
            OutcomeKind::MalformedOutput,
            Reason::InvalidBoundaryPayload,
        )),
        ServiceOutcome::completed(
            ConfirmationDecision::NoDecision,
            super::support::service_receipt(OutcomeKind::Completed, Reason::Completed),
        ),
    ];
    let issues = [
        ConfirmationIssue::BrokerUnavailable,
        ConfirmationIssue::BrokerCancelled,
        ConfirmationIssue::BrokerTimedOut,
        ConfirmationIssue::MalformedDecision,
        ConfirmationIssue::NoDecision,
    ];

    for (outcome, expected_issue) in outcomes.iter().zip(issues) {
        let evaluation = evaluate_confirmation_outcome(
            &request,
            outcome,
            ConfirmationTick::new(super::support::FRESH_TICK_VALUE),
        );
        assert!(!evaluation.dispatch_allowed());
        assert_eq!(evaluation.status(), ConfirmationStatus::Denied);
        assert_eq!(evaluation.issue(), expected_issue);
    }
}

#[test]
fn default_deny_records_missing_without_dispatch() {
    let request = super::support::protected_request(super::support::SUMMARY);
    let outcome = default_deny_confirmation(&request);

    let evaluation = evaluate_confirmation_outcome(
        &request,
        &outcome,
        ConfirmationTick::new(super::support::FRESH_TICK_VALUE),
    );

    assert!(!evaluation.dispatch_allowed());
    assert_eq!(evaluation.issue(), ConfirmationIssue::BrokerAbsent);
    assert_eq!(
        evaluation.receipt().broker_class(),
        ConfirmationBrokerClass::DefaultDeny
    );
}

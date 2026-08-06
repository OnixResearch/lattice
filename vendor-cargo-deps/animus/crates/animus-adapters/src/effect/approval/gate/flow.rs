/// Evaluates a broker outcome and fails closed for every non-approved state.
#[must_use]
pub fn evaluate_confirmation_outcome(
    request: &super::ConfirmationRequest,
    outcome: &super::super::ServiceOutcome<super::ConfirmationDecision>,
    current_tick: super::ConfirmationTick,
) -> super::ConfirmationEvaluation {
    match outcome {
        super::super::ServiceOutcome::Completed { value, .. } => {
            evaluate_confirmation_decision(request, value, super::ConfirmationBrokerClass::Injected, current_tick)
        }
        super::super::ServiceOutcome::Absent { .. } => denied_evaluation(DeniedEvaluationInput {
            request,
            issue: super::ConfirmationIssue::BrokerAbsent,
            broker_class: super::ConfirmationBrokerClass::DefaultDeny,
            timing_class: super::ConfirmationTimingClass::NotApplicable,
            current_tick,
        }),
        super::super::ServiceOutcome::Unavailable { .. } => denied_evaluation(DeniedEvaluationInput {
            request,
            issue: super::ConfirmationIssue::BrokerUnavailable,
            broker_class: super::ConfirmationBrokerClass::Injected,
            timing_class: super::ConfirmationTimingClass::NotApplicable,
            current_tick,
        }),
        super::super::ServiceOutcome::Denied { value, .. } => {
            let issue = denied_issue(value.as_ref());
            denied_evaluation(DeniedEvaluationInput {
                request,
                issue,
                broker_class: broker_class_for_denial(issue),
                timing_class: super::ConfirmationTimingClass::NotApplicable,
                current_tick,
            })
        }
        super::super::ServiceOutcome::Cancelled { .. } => denied_evaluation(DeniedEvaluationInput {
            request,
            issue: super::ConfirmationIssue::BrokerCancelled,
            broker_class: super::ConfirmationBrokerClass::Injected,
            timing_class: super::ConfirmationTimingClass::Cancelled,
            current_tick,
        }),
        super::super::ServiceOutcome::TimedOut { .. } => denied_evaluation(DeniedEvaluationInput {
            request,
            issue: super::ConfirmationIssue::BrokerTimedOut,
            broker_class: super::ConfirmationBrokerClass::Injected,
            timing_class: super::ConfirmationTimingClass::TimedOut,
            current_tick,
        }),
        super::super::ServiceOutcome::MalformedOutput { .. } => denied_evaluation(DeniedEvaluationInput {
            request,
            issue: super::ConfirmationIssue::MalformedDecision,
            broker_class: super::ConfirmationBrokerClass::Injected,
            timing_class: super::ConfirmationTimingClass::NotApplicable,
            current_tick,
        }),
    }
}

/// Evaluates one completed broker decision against a pending request.
#[must_use]
pub fn evaluate_confirmation_decision(
    request: &super::ConfirmationRequest,
    decision: &super::ConfirmationDecision,
    broker_class: super::ConfirmationBrokerClass,
    current_tick: super::ConfirmationTick,
) -> super::ConfirmationEvaluation {
    match decision {
        super::ConfirmationDecision::Approved { binding } => {
            if let Some(issue) = binding_issue(request, binding, current_tick) {
                let timing = timing_for_issue(issue);
                denied_evaluation(DeniedEvaluationInput {
                    request,
                    issue,
                    broker_class,
                    timing_class: timing,
                    current_tick,
                })
            } else {
                approved_evaluation(request, broker_class, current_tick)
            }
        }
        super::ConfirmationDecision::Denied { reason } => denied_evaluation(DeniedEvaluationInput {
            request,
            issue: reason_issue(*reason),
            broker_class,
            timing_class: super::ConfirmationTimingClass::NotApplicable,
            current_tick,
        }),
        super::ConfirmationDecision::NoDecision => denied_evaluation(DeniedEvaluationInput {
            request,
            issue: super::ConfirmationIssue::NoDecision,
            broker_class,
            timing_class: super::ConfirmationTimingClass::NotApplicable,
            current_tick,
        }),
    }
}

fn approved_evaluation(
    request: &super::ConfirmationRequest,
    broker_class: super::ConfirmationBrokerClass,
    current_tick: super::ConfirmationTick,
) -> super::ConfirmationEvaluation {
    let expiry_class = expiry_class(request.expiry(), current_tick);
    super::ConfirmationEvaluation {
        status: super::ConfirmationStatus::Approved,
        issue: super::ConfirmationIssue::None,
        receipt: super::ConfirmationReceipt::new(super::ConfirmationReceiptDraft {
            request,
            decision_status: super::ConfirmationStatus::Approved,
            issue: super::ConfirmationIssue::None,
            broker_class,
            timing_class: super::ConfirmationTimingClass::Fresh,
            expiry_class,
        }),
    }
}

#[derive(Clone, Copy)]
struct DeniedEvaluationInput<'a> {
    request: &'a super::ConfirmationRequest,
    issue: super::ConfirmationIssue,
    broker_class: super::ConfirmationBrokerClass,
    timing_class: super::ConfirmationTimingClass,
    current_tick: super::ConfirmationTick,
}

fn denied_evaluation(input: DeniedEvaluationInput<'_>) -> super::ConfirmationEvaluation {
    let expiry_class = expiry_class(input.request.expiry(), input.current_tick);
    super::ConfirmationEvaluation {
        status: super::ConfirmationStatus::Denied,
        issue: input.issue,
        receipt: super::ConfirmationReceipt::new(super::ConfirmationReceiptDraft {
            request: input.request,
            decision_status: super::ConfirmationStatus::Denied,
            issue: input.issue,
            broker_class: input.broker_class,
            timing_class: input.timing_class,
            expiry_class,
        }),
    }
}

fn binding_issue(
    request: &super::ConfirmationRequest,
    binding: &super::ConfirmationBinding,
    current_tick: super::ConfirmationTick,
) -> Option<super::ConfirmationIssue> {
    debug_assert!(!request.request_id().is_empty());
    debug_assert!(!binding.request_id().is_empty());
    debug_assert!(!request.resource().is_empty());
    debug_assert!(!binding.resource().is_empty());
    debug_assert!(!request.ability().is_empty());
    debug_assert!(!binding.ability().is_empty());

    if binding.request_id() != request.request_id() {
        return Some(super::ConfirmationIssue::MismatchedRequest);
    }
    if binding.action_kind() != request.action_kind() {
        return Some(super::ConfirmationIssue::MismatchedAction);
    }
    if binding.resource() != request.resource() {
        return Some(super::ConfirmationIssue::MismatchedResource);
    }
    if binding.ability() != request.ability() {
        return Some(super::ConfirmationIssue::MismatchedAbility);
    }
    if binding.policy_hash() != request.policy_hash() {
        return Some(super::ConfirmationIssue::MismatchedPolicy);
    }
    if binding.expiry() != request.expiry() {
        return Some(super::ConfirmationIssue::MismatchedExpiry);
    }
    if request.expiry().is_some_and(|expiry| expiry.is_stale(current_tick)) {
        return Some(super::ConfirmationIssue::StaleDecision);
    }
    None
}

const fn timing_for_issue(issue: super::ConfirmationIssue) -> super::ConfirmationTimingClass {
    match issue {
        super::ConfirmationIssue::StaleDecision => super::ConfirmationTimingClass::Expired,
        super::ConfirmationIssue::BrokerCancelled => super::ConfirmationTimingClass::Cancelled,
        super::ConfirmationIssue::BrokerTimedOut => super::ConfirmationTimingClass::TimedOut,
        super::ConfirmationIssue::None
        | super::ConfirmationIssue::BrokerAbsent
        | super::ConfirmationIssue::BrokerUnavailable
        | super::ConfirmationIssue::BrokerDenied
        | super::ConfirmationIssue::MalformedDecision
        | super::ConfirmationIssue::NoDecision
        | super::ConfirmationIssue::MismatchedRequest
        | super::ConfirmationIssue::MismatchedAction
        | super::ConfirmationIssue::MismatchedResource
        | super::ConfirmationIssue::MismatchedAbility
        | super::ConfirmationIssue::MismatchedPolicy
        | super::ConfirmationIssue::MismatchedExpiry => super::ConfirmationTimingClass::NotApplicable,
    }
}

const fn broker_class_for_denial(issue: super::ConfirmationIssue) -> super::ConfirmationBrokerClass {
    match issue {
        super::ConfirmationIssue::BrokerAbsent => super::ConfirmationBrokerClass::DefaultDeny,
        super::ConfirmationIssue::None
        | super::ConfirmationIssue::BrokerUnavailable
        | super::ConfirmationIssue::BrokerDenied
        | super::ConfirmationIssue::BrokerCancelled
        | super::ConfirmationIssue::BrokerTimedOut
        | super::ConfirmationIssue::MalformedDecision
        | super::ConfirmationIssue::NoDecision
        | super::ConfirmationIssue::MismatchedRequest
        | super::ConfirmationIssue::MismatchedAction
        | super::ConfirmationIssue::MismatchedResource
        | super::ConfirmationIssue::MismatchedAbility
        | super::ConfirmationIssue::MismatchedPolicy
        | super::ConfirmationIssue::MismatchedExpiry
        | super::ConfirmationIssue::StaleDecision => super::ConfirmationBrokerClass::Injected,
    }
}

fn denied_issue(decision: Option<&super::ConfirmationDecision>) -> super::ConfirmationIssue {
    match decision {
        Some(super::ConfirmationDecision::Denied { reason }) => reason_issue(*reason),
        Some(super::ConfirmationDecision::NoDecision) => super::ConfirmationIssue::NoDecision,
        Some(super::ConfirmationDecision::Approved { .. }) | None => super::ConfirmationIssue::BrokerDenied,
    }
}

const fn reason_issue(reason: super::super::Reason) -> super::ConfirmationIssue {
    match reason {
        super::super::Reason::NotInjected => super::ConfirmationIssue::BrokerAbsent,
        super::super::Reason::HostUnavailable => super::ConfirmationIssue::BrokerUnavailable,
        super::super::Reason::Cancelled => super::ConfirmationIssue::BrokerCancelled,
        super::super::Reason::DeadlineExceeded => super::ConfirmationIssue::BrokerTimedOut,
        super::super::Reason::InvalidBoundaryPayload => super::ConfirmationIssue::MalformedDecision,
        super::super::Reason::NoDecision => super::ConfirmationIssue::NoDecision,
        super::super::Reason::MismatchedConfirmation => super::ConfirmationIssue::MismatchedRequest,
        super::super::Reason::StaleConfirmation => super::ConfirmationIssue::StaleDecision,
        super::super::Reason::CapabilityDenied
        | super::super::Reason::PolicyDenied
        | super::super::Reason::Completed => super::ConfirmationIssue::BrokerDenied,
    }
}

const fn expiry_class(
    expiry: Option<super::ConfirmationExpiry>,
    current_tick: super::ConfirmationTick,
) -> super::ConfirmationExpiryClass {
    match expiry {
        None => super::ConfirmationExpiryClass::NoExpiry,
        Some(value) if value.is_stale(current_tick) => super::ConfirmationExpiryClass::Expired,
        Some(_value) => super::ConfirmationExpiryClass::Active,
    }
}

//! Attempt settlement and explicit uncertainty recovery.
#![allow(
    unknown_lints,
    non_trait_imports,
    reason = "recovery transitions use the adjacent protocol model vocabulary"
)]

use super::super::identity;
use super::super::model::AttemptState;
use super::super::model::Error;
use super::super::model::Identity;
use super::super::model::Issue;
use super::super::model::SessionEvent;
use super::super::model::SessionPlan;
use super::super::model::SessionRecord;
use super::apply_event;
use super::attempt_record;
use super::create_session;
use super::plan_action;
use super::prepare_attempt;

pub fn settle_attempt(session: &SessionRecord, attempt: &Identity) -> Result<SessionRecord, Error> {
    let current = attempt_record(session, attempt)?;
    if current.observations.len() != current.messages.len() {
        return Err(Issue::ObservationMismatch.into());
    }
    let is_accepted = current.messages.iter().all(|message| {
        current
            .observations
            .iter()
            .any(|observation| observation.message == message.identity && observation.accepted)
    });
    let state_label = if is_accepted { "committed" } else { "failed" };
    let outcome = identity::reference(identity::OUTCOME_DOMAIN, &[attempt.as_str(), state_label])?;
    if is_accepted {
        apply_event(session, &SessionEvent::Committed {
            attempt: attempt.clone(),
            outcome,
        })
    } else {
        apply_event(session, &SessionEvent::Failed {
            attempt: attempt.clone(),
            outcome,
        })
    }
}

pub fn fail_before_dispatch(session: &SessionRecord, attempt: &Identity, issue: Issue) -> Result<SessionRecord, Error> {
    let current = attempt_record(session, attempt)?;
    if !matches!(current.state, AttemptState::Prepared | AttemptState::Authorized | AttemptState::DispatchedUnknown) {
        return Err(Issue::ReorderedMessage.into());
    }
    let outcome =
        identity::reference(identity::OUTCOME_DOMAIN, &[attempt.as_str(), "failed-before-dispatch", issue.code()])?;
    apply_event(session, &SessionEvent::Failed {
        attempt: attempt.clone(),
        outcome,
    })
}

pub fn abandon_unknown(
    session: &SessionRecord,
    attempt: &Identity,
    acknowledge_unknown: bool,
) -> Result<SessionRecord, Error> {
    if !acknowledge_unknown {
        return Err(Issue::AcknowledgementRequired.into());
    }
    if attempt_record(session, attempt)?.state != AttemptState::DispatchedUnknown {
        return Err(Issue::AttemptNotUnknown.into());
    }
    let outcome = identity::reference(identity::OUTCOME_DOMAIN, &[attempt.as_str(), "abandoned-unknown"])?;
    apply_event(session, &SessionEvent::AbandonedUnknown {
        attempt: attempt.clone(),
        outcome,
    })
}

pub fn retry_unknown(
    session: &SessionRecord,
    attempt: &Identity,
    acknowledge_duplicate_risk: bool,
) -> Result<super::super::model::RetryRecovery, Error> {
    if !acknowledge_duplicate_risk {
        return Err(Issue::AcknowledgementRequired.into());
    }
    let current = attempt_record(session, attempt)?;
    if current.state != AttemptState::DispatchedUnknown {
        return Err(Issue::AttemptNotUnknown.into());
    }
    let request = current.request.clone();
    let recovered = abandon_unknown(session, attempt, true)?;
    let action = plan_action(&recovered, request)?;
    let prepared = prepare_attempt(&recovered, action)?;
    Ok(super::super::model::RetryRecovery { recovered, prepared })
}

pub fn replay(plan: SessionPlan, events: &[SessionEvent]) -> Result<SessionRecord, Error> {
    let mut state = create_session(plan)?;
    for event in events {
        state = apply_event(&state, event)?;
    }
    Ok(state)
}

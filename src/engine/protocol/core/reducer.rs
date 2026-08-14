//! Pure protocol event reducer and cursor transitions.
#![allow(
    unknown_lints,
    assertion_density,
    non_trait_imports,
    reason = "fallible reducer branches use the adjacent protocol model vocabulary and direct negative tests"
)]

use super::super::model::AttemptState;
use super::super::model::Error;
use super::super::model::Issue;
use super::super::model::SessionEvent;
use super::super::model::SessionRecord;
use super::apply_committed;
use super::apply_prepared;
use super::apply_terminal;
use super::attempt_record_mut;
use super::session_status;

const VERSION_INCREMENT: u64 = 1;

pub fn apply_event(session: &SessionRecord, event: &SessionEvent) -> Result<SessionRecord, Error> {
    let mut next = session.clone();
    match event {
        SessionEvent::Prepared { attempt } => apply_prepared(&mut next, attempt)?,
        SessionEvent::Authorized { attempt, references } => {
            let record = attempt_record_mut(&mut next, attempt)?;
            if record.state != AttemptState::Prepared
                || references.len() != record.messages.len()
                || references.is_empty()
            {
                return Err(Issue::ReorderedMessage.into());
            }
            record.authorization_references.clone_from(references);
            record.state = AttemptState::Authorized;
        }
        SessionEvent::DispatchedUnknown { attempt } => {
            let record = attempt_record_mut(&mut next, attempt)?;
            if record.state != AttemptState::Authorized {
                return Err(Issue::ReorderedMessage.into());
            }
            record.state = AttemptState::DispatchedUnknown;
        }
        SessionEvent::Observed { attempt, observation } => {
            let record = attempt_record_mut(&mut next, attempt)?;
            if !matches!(record.state, AttemptState::Prepared | AttemptState::DispatchedUnknown)
                || observation.attempt != *attempt
                || record.observations.iter().any(|existing| existing.message == observation.message)
            {
                return Err(Issue::DuplicateMessage.into());
            }
            if !record.messages.iter().any(|message| message.identity == observation.message) {
                return Err(Issue::ObservationMismatch.into());
            }
            record.observations.push(observation.clone());
        }
        SessionEvent::Committed { attempt, outcome } => apply_committed(&mut next, attempt, outcome)?,
        SessionEvent::Failed { attempt, outcome } => apply_terminal(&mut next, attempt, outcome, AttemptState::Failed)?,
        SessionEvent::AbandonedUnknown { attempt, outcome } => {
            apply_terminal(&mut next, attempt, outcome, AttemptState::AbandonedUnknown)?;
        }
    }
    next.version = next.version.checked_add(VERSION_INCREMENT).ok_or(Issue::PersistenceFailed)?;
    next.events.push(event.clone());
    next.status = session_status(&next.cursors);
    Ok(next)
}

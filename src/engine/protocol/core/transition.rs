//! Committed and terminal state transitions.
#![allow(
    unknown_lints,
    non_trait_imports,
    reason = "state transitions use the adjacent protocol model vocabulary"
)]

use super::super::model::Attempt;
use super::super::model::AttemptState;
use super::super::model::Error;
use super::super::model::Identity;
use super::super::model::Issue;
use super::super::model::RoleId;
use super::super::model::RoleProgram;
use super::super::model::SessionRecord;
use super::attempt_record;
use super::attempt_record_mut;
use super::next_attempt_sequence;

pub(super) fn apply_prepared(session: &mut SessionRecord, attempt: &Attempt) -> Result<(), Error> {
    if attempt.sequence != next_attempt_sequence(session)
        || session.attempts.iter().any(|existing| existing.identity == attempt.identity)
        || attempt.state != AttemptState::Prepared
    {
        return Err(Issue::ReorderedMessage.into());
    }
    session.attempts.push(attempt.clone());
    Ok(())
}

pub(super) fn apply_committed(
    session: &mut SessionRecord,
    attempt: &Identity,
    outcome: &Identity,
) -> Result<(), Error> {
    let current = attempt_record(session, attempt)?.clone();
    if current.observations.len() != current.messages.len()
        || current.observations.iter().any(|observation| !observation.accepted)
    {
        return Err(Issue::ObservationMismatch.into());
    }
    advance_cursor(session, current.request.source, current.request.label)?;
    for message in &current.messages {
        advance_cursor(session, message.destination_role, current.request.label)?;
    }
    if let Some(value) = current.value {
        let owner = session.owners.iter_mut().find(|owner| owner.value == value).ok_or(Issue::WrongValue)?;
        owner.owner = current.messages.first().map(|message| message.destination_role).ok_or(Issue::WrongPeer)?;
    }
    apply_terminal(session, attempt, outcome, AttemptState::Committed)
}

pub(super) fn apply_terminal(
    session: &mut SessionRecord,
    attempt: &Identity,
    outcome: &Identity,
    state: AttemptState,
) -> Result<(), Error> {
    let record = attempt_record_mut(session, attempt)?;
    let can_close = match state {
        AttemptState::Committed | AttemptState::Failed => {
            matches!(record.state, AttemptState::Prepared | AttemptState::Authorized | AttemptState::DispatchedUnknown)
        }
        AttemptState::AbandonedUnknown => record.state == AttemptState::DispatchedUnknown,
        AttemptState::Prepared | AttemptState::Authorized | AttemptState::DispatchedUnknown => false,
    };
    if !can_close || record.outcome.is_some() {
        return Err(Issue::ReorderedMessage.into());
    }
    record.state = state;
    record.outcome = Some(outcome.clone());
    Ok(())
}

fn advance_cursor(session: &mut SessionRecord, role: RoleId, label: super::super::model::LabelId) -> Result<(), Error> {
    let cursor = session.cursors.iter_mut().find(|cursor| cursor.role == role).ok_or(Issue::MissingRole)?;
    cursor.protocol = match &cursor.protocol {
        RoleProgram::Send {
            label: expected, next, ..
        }
        | RoleProgram::Receive {
            label: expected, next, ..
        }
        | RoleProgram::SendOwned {
            label: expected, next, ..
        }
        | RoleProgram::ReceiveOwned {
            label: expected, next, ..
        } if *expected == label => (**next).clone(),
        RoleProgram::InternalChoice { branches } | RoleProgram::Offer { branches, .. } => branches
            .iter()
            .find(|branch| branch.label == label)
            .map(|branch| branch.body.clone())
            .ok_or(Issue::WrongLabel)?,
        _ => return Err(Issue::UnexpectedAction.into()),
    };
    Ok(())
}

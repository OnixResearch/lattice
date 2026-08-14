//! Protocol state lookup and identity helpers.
#![allow(
    unknown_lints,
    non_trait_imports,
    reason = "lookup helpers use the adjacent protocol model vocabulary"
)]

use super::super::model::Attempt;
use super::super::model::Error;
use super::super::model::Identity;
use super::super::model::Issue;
use super::super::model::RoleCursor;
use super::super::model::RoleId;
use super::super::model::RoleProgram;
use super::super::model::SessionRecord;
use super::super::model::SessionStatus;

const INITIAL_ATTEMPT_SEQUENCE: u64 = 0;
const ATTEMPT_SEQUENCE_INCREMENT: u64 = 1;

pub(super) fn step_identity(session: &SessionRecord, cursor: &RoleCursor) -> Result<Identity, Error> {
    let role = cursor.role.0.to_string();
    let protocol = serde_json::to_vec(&cursor.protocol).map_err(|_| Error::Serialization)?;
    let protocol_digest = blake3::hash(&protocol).to_hex().to_string();
    super::super::identity::reference(super::super::identity::STEP_DOMAIN, &[
        session.plan.session.as_str(),
        cursor.local_artifact.as_str(),
        role.as_str(),
        protocol_digest.as_str(),
    ])
}

pub(super) fn cursor(session: &SessionRecord, role: RoleId) -> Result<&RoleCursor, Error> {
    session.cursors.iter().find(|cursor| cursor.role == role).ok_or_else(|| Issue::MissingRole.into())
}

pub(super) fn role_instance(
    session: &SessionRecord,
    role: RoleId,
) -> Result<&super::super::model::RoleInstance, Error> {
    session
        .plan
        .roles
        .iter()
        .find(|instance| instance.role == role)
        .ok_or_else(|| Issue::MissingRole.into())
}

pub(super) fn attempt_record<'a>(session: &'a SessionRecord, identity: &Identity) -> Result<&'a Attempt, Error> {
    session
        .attempts
        .iter()
        .find(|attempt| attempt.identity == *identity)
        .ok_or_else(|| Issue::ReorderedMessage.into())
}

pub(super) fn attempt_record_mut<'a>(
    session: &'a mut SessionRecord,
    identity: &Identity,
) -> Result<&'a mut Attempt, Error> {
    session
        .attempts
        .iter_mut()
        .find(|attempt| attempt.identity == *identity)
        .ok_or_else(|| Issue::ReorderedMessage.into())
}

pub(super) fn next_attempt_sequence(session: &SessionRecord) -> u64 {
    session
        .attempts
        .last()
        .map_or(INITIAL_ATTEMPT_SEQUENCE, |attempt| attempt.sequence.saturating_add(ATTEMPT_SEQUENCE_INCREMENT))
}

pub(super) fn exact_label(
    expected: super::super::model::LabelId,
    supplied: super::super::model::LabelId,
) -> Result<(), Error> {
    if expected != supplied {
        return Err(Issue::WrongLabel.into());
    }
    Ok(())
}

pub(super) fn exact_owner(
    session: &SessionRecord,
    value: super::super::model::ValueId,
    source: RoleId,
) -> Result<(), Error> {
    let owner = session.owners.iter().find(|owner| owner.value == value).ok_or(Issue::WrongValue)?;
    if owner.owner != source {
        return Err(Issue::FormerOwner.into());
    }
    Ok(())
}

pub(super) fn session_status(cursors: &[RoleCursor]) -> SessionStatus {
    if cursors.iter().all(|cursor| cursor.protocol == RoleProgram::End) {
        SessionStatus::Complete
    } else {
        SessionStatus::Active
    }
}

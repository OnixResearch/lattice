//! Pure protocol session transitions and replay.
//! r[impl lattice.role_protocol.state]
//! r[impl lattice.role_protocol.envelopes]
//! r[impl lattice.role_protocol.transitions]
//! r[impl lattice.role_protocol.ownership]
//! r[impl lattice.role_protocol.uncertainty]
//! r[impl lattice.role_protocol.replay]
#![allow(
    unknown_lints,
    non_trait_imports,
    reason = "the pure protocol component uses its local model vocabulary to keep ownership clear"
)]
#![allow(
    assertion_density,
    reason = "fallible pure transitions use explicit typed rejection branches; tests cover each invariant"
)]

use super::model::ActionKind;
use super::model::ActionRequest;
use super::model::Attempt;
use super::model::AttemptState;
use super::model::Error;
use super::model::Identity;
use super::model::Issue;
use super::model::MessageEnvelope;
use super::model::Observation;
use super::model::PlannedAction;
use super::model::RoleCursor;
use super::model::RoleId;
use super::model::RoleProgram;
use super::model::SessionEvent;

mod envelope;
mod lookup;
mod planning;
mod recovery;
mod reducer;
mod transition;

use lookup::attempt_record;
use lookup::attempt_record_mut;
use lookup::cursor;
use lookup::exact_label;
use lookup::exact_owner;
use lookup::next_attempt_sequence;
use lookup::role_instance;
use lookup::session_status;
use lookup::step_identity;
use planning::planned_action;
use transition::apply_committed;
use transition::apply_prepared;
use transition::apply_terminal;

use super::model::SessionPlan;
use super::model::SessionRecord;
use super::model::SessionStatus;

const INITIAL_SESSION_VERSION: u64 = 0;

#[derive(serde::Serialize)]
struct MessageMaterial<'a> {
    attempt: &'a Identity,
    draft: &'a Identity,
    source: RoleId,
    destination: RoleId,
    payload: &'a Identity,
}

struct ActionDetails<'a> {
    kind: ActionKind,
    payload_tag: super::model::PayloadTag,
    value: Option<super::model::ValueId>,
    destinations: &'a [RoleId],
}

pub fn create_session(plan: SessionPlan) -> Result<SessionRecord, Error> {
    if plan.roles.len() != plan.cohort.roles.len() {
        return Err(Issue::InvalidPlacement.into());
    }
    let expected = super::intake::plan_session(
        plan.cohort.clone(),
        plan.roles.iter().map(|role| role.placement.clone()).collect(),
        plan.nonce.clone(),
    )?;
    if expected != plan {
        return Err(Issue::IdentityMismatch.into());
    }
    let cursors = plan
        .cohort
        .roles
        .iter()
        .map(|role| RoleCursor {
            role: role.role,
            local_artifact: role.local_artifact.clone(),
            protocol: role.protocol.clone(),
        })
        .collect::<Vec<_>>();
    let status = session_status(&cursors);
    Ok(SessionRecord {
        schema: super::model::SESSION_SCHEMA.to_owned(),
        owners: plan.cohort.initial_owners.clone(),
        plan,
        version: INITIAL_SESSION_VERSION,
        status,
        cursors,
        attempts: Vec::new(),
        events: Vec::new(),
    })
}

pub fn plan_action(session: &SessionRecord, request: ActionRequest) -> Result<PlannedAction, Error> {
    if session.status != SessionStatus::Active {
        return Err(Issue::UnexpectedAction.into());
    }
    if session.attempts.iter().any(|attempt| {
        matches!(attempt.state, AttemptState::Prepared | AttemptState::Authorized | AttemptState::DispatchedUnknown)
    }) {
        return Err(Issue::AttemptInFlight.into());
    }
    let source_cursor = cursor(session, request.source)?;
    match &source_cursor.protocol {
        RoleProgram::Send {
            peer,
            label,
            payload_tag,
            ..
        } => {
            exact_label(*label, request.label)?;
            planned_action(session, request, ActionDetails {
                kind: ActionKind::Transfer,
                payload_tag: *payload_tag,
                value: None,
                destinations: &[*peer],
            })
        }
        RoleProgram::SendOwned {
            peer,
            label,
            payload_tag,
            value,
            ..
        } => {
            exact_label(*label, request.label)?;
            exact_owner(session, *value, request.source)?;
            planned_action(session, request, ActionDetails {
                kind: ActionKind::OwnedTransfer,
                payload_tag: *payload_tag,
                value: Some(*value),
                destinations: &[*peer],
            })
        }
        RoleProgram::InternalChoice { branches } => {
            let branch = branches.iter().find(|branch| branch.label == request.label).ok_or(Issue::WrongLabel)?;
            let mut destinations = Vec::with_capacity(session.plan.roles.len().saturating_sub(1));
            for role in &session.plan.roles {
                if role.role == request.source {
                    continue;
                }
                let candidate = cursor(session, role.role)?;
                if matches!(
                    &candidate.protocol,
                    RoleProgram::Offer { from, branches }
                        if *from == request.source && branches.iter().any(|candidate| candidate.label == branch.label)
                ) {
                    destinations.push(role.role);
                }
            }
            planned_action(session, request, ActionDetails {
                kind: ActionKind::Choice,
                payload_tag: super::model::PayloadTag(0),
                value: None,
                destinations: &destinations,
            })
        }
        _ => Err(Issue::UnexpectedAction.into()),
    }
}

pub fn prepare_attempt(session: &SessionRecord, action: PlannedAction) -> Result<SessionRecord, Error> {
    if action.sequence != next_attempt_sequence(session) {
        return Err(Issue::ReorderedMessage.into());
    }
    let expected = plan_action(session, action.request.clone())?;
    if expected != action {
        return Err(Issue::IdentityMismatch.into());
    }
    let sequence = action.sequence.to_string();
    let attempt_identity = super::identity::reference(super::identity::ATTEMPT_DOMAIN, &[
        session.plan.session.as_str(),
        action.identity.as_str(),
        sequence.as_str(),
    ])?;
    let mut messages = Vec::with_capacity(action.messages.len());
    for message in &action.messages {
        messages.push(envelope::bind(session, &action, message, &attempt_identity)?);
    }
    let attempt = Attempt {
        identity: attempt_identity,
        action: action.identity,
        sequence: action.sequence,
        step: action.step,
        kind: action.kind,
        request: action.request,
        payload_tag: action.payload_tag,
        value: action.value,
        messages,
        authorization_references: Vec::new(),
        state: AttemptState::Prepared,
        observations: Vec::new(),
        outcome: None,
    };
    apply_event(session, &SessionEvent::Prepared { attempt })
}

pub fn record_authorization(
    session: &SessionRecord,
    attempt: &Identity,
    references: Vec<Identity>,
) -> Result<SessionRecord, Error> {
    apply_event(session, &SessionEvent::Authorized {
        attempt: attempt.clone(),
        references,
    })
}

pub fn mark_dispatched_unknown(session: &SessionRecord, attempt: &Identity) -> Result<SessionRecord, Error> {
    apply_event(session, &SessionEvent::DispatchedUnknown {
        attempt: attempt.clone(),
    })
}

pub fn validate_dispatch_envelope(session: &SessionRecord, envelope: &MessageEnvelope) -> Result<(), Error> {
    let attempt = attempt_record(session, &envelope.attempt)?;
    if !matches!(attempt.state, AttemptState::Prepared | AttemptState::Authorized) {
        return Err(Issue::ReorderedMessage.into());
    }
    let stored = attempt
        .messages
        .iter()
        .find(|message| message.identity == envelope.identity)
        .ok_or(Issue::ObservationMismatch)?;
    if stored != envelope {
        return Err(Issue::IdentityMismatch.into());
    }
    Ok(())
}

pub fn admit_observation(
    session: &SessionRecord,
    envelope: &MessageEnvelope,
    accepted: bool,
    evidence_reference: Identity,
    issue: Option<Issue>,
) -> Result<SessionRecord, Error> {
    envelope::admit(session, envelope)?;
    let observation_identity = super::identity::reference(super::identity::OBSERVATION_DOMAIN, &[
        envelope.attempt.as_str(),
        envelope.identity.as_str(),
        evidence_reference.as_str(),
        if accepted { "accepted" } else { "rejected" },
        issue.map_or("none", Issue::code),
    ])?;
    let observation = Observation {
        schema: super::model::OBSERVATION_SCHEMA.to_owned(),
        identity: observation_identity,
        attempt: envelope.attempt.clone(),
        message: envelope.identity.clone(),
        evidence_reference,
        accepted,
        issue,
    };
    apply_event(session, &SessionEvent::Observed {
        attempt: envelope.attempt.clone(),
        observation,
    })
}

pub use recovery::abandon_unknown;
pub use recovery::fail_before_dispatch;
pub use recovery::replay;
pub use recovery::retry_unknown;
pub use recovery::settle_attempt;
pub use reducer::apply_event;

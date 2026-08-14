//! Exact envelope construction and admission.
#![allow(
    unknown_lints,
    assertion_density,
    non_trait_imports,
    reason = "exact envelope checks use the adjacent protocol model vocabulary and adversarial fixtures"
)]

use super::super::model::AttemptState;
use super::super::model::Error;
use super::super::model::Identity;
use super::super::model::Issue;
use super::super::model::MessageEnvelope;
use super::super::model::PlannedAction;
use super::super::model::PlannedMessage;
use super::super::model::RoleProgram;
use super::super::model::SessionRecord;
use super::MessageMaterial;
use super::attempt_record;
use super::cursor;
use super::role_instance;

pub(super) fn bind(
    session: &SessionRecord,
    action: &PlannedAction,
    planned: &PlannedMessage,
    attempt: &Identity,
) -> Result<MessageEnvelope, Error> {
    let source = role_instance(session, action.request.source)?;
    let destination = role_instance(session, planned.destination)?;
    let material = MessageMaterial {
        attempt,
        draft: &planned.draft_identity,
        source: source.role,
        destination: destination.role,
        payload: &action.request.payload_reference,
    };
    let identity = super::super::identity::serialized_reference(super::super::identity::MESSAGE_DOMAIN, &material)?;
    Ok(MessageEnvelope {
        schema: super::super::model::ENVELOPE_SCHEMA.to_owned(),
        identity,
        attempt: attempt.clone(),
        protocol: session.plan.cohort.global_artifact.clone(),
        complete_local_set: session.plan.cohort.complete_local_set.clone(),
        session: session.plan.session.clone(),
        source_role: source.role,
        destination_role: destination.role,
        source_instance: source.identity.clone(),
        destination_instance: destination.identity.clone(),
        source_local_artifact: source.local_artifact.clone(),
        destination_local_artifact: destination.local_artifact.clone(),
        step: action.step.clone(),
        label: action.request.label,
        payload_tag: action.payload_tag,
        value: action.value,
        payload_reference: action.request.payload_reference.clone(),
        schema_set: session.plan.cohort.schema_set.clone(),
        adapter_cohort: destination.placement.adapter_cohort.clone(),
        authority_reference: super::super::identity::reference(super::super::identity::REQUEST_DOMAIN, &[
            attempt.as_str(),
            planned.draft_identity.as_str(),
        ])?,
    })
}

pub(super) fn admit(session: &SessionRecord, envelope: &MessageEnvelope) -> Result<(), Error> {
    let attempt = attempt_record(session, &envelope.attempt)?;
    let has_exact_protocol_context = envelope.schema == super::super::model::ENVELOPE_SCHEMA
        && envelope.protocol == session.plan.cohort.global_artifact
        && envelope.complete_local_set == session.plan.cohort.complete_local_set
        && envelope.session == session.plan.session;
    let has_exact_action_context = envelope.step == attempt.step
        && envelope.label == attempt.request.label
        && envelope.payload_tag == attempt.payload_tag;
    let has_exact_payload_context = envelope.value == attempt.value
        && envelope.payload_reference == attempt.request.payload_reference
        && envelope.schema_set == session.plan.cohort.schema_set;
    let is_dispatched_unknown = attempt.state == AttemptState::DispatchedUnknown;
    if !(has_exact_protocol_context && has_exact_action_context && has_exact_payload_context && is_dispatched_unknown) {
        return Err(Issue::IdentityMismatch.into());
    }
    let source = role_instance(session, envelope.source_role)?;
    let destination = role_instance(session, envelope.destination_role)?;
    let has_exact_instances =
        envelope.source_instance == source.identity && envelope.destination_instance == destination.identity;
    let has_exact_artifacts = envelope.source_local_artifact == source.local_artifact
        && envelope.destination_local_artifact == destination.local_artifact;
    let has_exact_adapter = envelope.adapter_cohort == destination.placement.adapter_cohort;
    if !(has_exact_instances && has_exact_artifacts && has_exact_adapter) {
        return Err(Issue::IdentityMismatch.into());
    }
    let stored = attempt
        .messages
        .iter()
        .find(|message| message.identity == envelope.identity)
        .ok_or(Issue::ObservationMismatch)?;
    if stored != envelope {
        return Err(Issue::IdentityMismatch.into());
    }
    expected_receiver(session, envelope)?;
    if attempt.observations.iter().any(|observation| observation.message == envelope.identity) {
        return Err(Issue::DuplicateMessage.into());
    }
    Ok(())
}

fn expected_receiver(session: &SessionRecord, envelope: &MessageEnvelope) -> Result<(), Error> {
    let destination = cursor(session, envelope.destination_role)?;
    match &destination.protocol {
        RoleProgram::Receive {
            peer,
            label,
            payload_tag,
            ..
        } if *peer == envelope.source_role && *label == envelope.label && *payload_tag == envelope.payload_tag => {
            Ok(())
        }
        RoleProgram::ReceiveOwned {
            peer,
            label,
            payload_tag,
            value,
            ..
        } if *peer == envelope.source_role
            && *label == envelope.label
            && *payload_tag == envelope.payload_tag
            && Some(*value) == envelope.value =>
        {
            Ok(())
        }
        RoleProgram::Offer { from, branches }
            if *from == envelope.source_role && branches.iter().any(|branch| branch.label == envelope.label) =>
        {
            Ok(())
        }
        _ => Err(Issue::UnexpectedAction.into()),
    }
}

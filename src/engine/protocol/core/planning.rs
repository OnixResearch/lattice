//! Exact action and message planning.
#![allow(
    unknown_lints,
    non_trait_imports,
    reason = "deterministic planning uses the adjacent protocol model vocabulary"
)]

use super::super::model::ActionRequest;
use super::super::model::Error;
use super::super::model::PlannedAction;
use super::super::model::PlannedMessage;
use super::super::model::SessionRecord;
use super::ActionDetails;
use super::cursor;
use super::next_attempt_sequence;
use super::role_instance;
use super::step_identity;

pub(super) fn planned_action(
    session: &SessionRecord,
    request: ActionRequest,
    details: ActionDetails<'_>,
) -> Result<PlannedAction, Error> {
    let source = cursor(session, request.source)?;
    let step = step_identity(session, source)?;
    let sequence = next_attempt_sequence(session);
    let sequence_label = sequence.to_string();
    let source_label = request.source.0.to_string();
    let label = request.label.0.to_string();
    let payload = details.payload_tag.0.to_string();
    let value_label = details.value.map_or_else(|| "none".to_owned(), |id| id.0.to_string());
    let identity = super::super::identity::reference(super::super::identity::ACTION_DOMAIN, &[
        session.plan.session.as_str(),
        step.as_str(),
        source_label.as_str(),
        label.as_str(),
        payload.as_str(),
        value_label.as_str(),
        request.payload_reference.as_str(),
        sequence_label.as_str(),
    ])?;
    let messages = details
        .destinations
        .iter()
        .map(|destination| {
            let destination_label = destination.0.to_string();
            let draft_identity = super::super::identity::reference(super::super::identity::MESSAGE_DRAFT_DOMAIN, &[
                identity.as_str(),
                destination_label.as_str(),
            ])?;
            Ok(PlannedMessage {
                destination: *destination,
                draft_identity,
                adapter_cohort: role_instance(session, *destination)?.placement.adapter_cohort.clone(),
            })
        })
        .collect::<Result<Vec<_>, Error>>()?;
    Ok(PlannedAction {
        identity,
        sequence,
        step,
        kind: details.kind,
        request,
        payload_tag: details.payload_tag,
        value: details.value,
        messages,
    })
}

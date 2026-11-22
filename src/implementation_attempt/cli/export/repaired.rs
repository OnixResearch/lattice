//! Export of a repair attempt's passing candidate revision.
//!
//! A repair attempt has no native attempt result. After the same admission as
//! a single-shot export (the request, its intent, the attempt-fence directory
//! and the revision seam), its export reads only the retained disposition of
//! the exact request and the step record of the revision it names. The
//! exported tree is the workspace, which must still capture to that revision's
//! candidate identity; `host-receipt.json` is the revision's host-derived
//! receipt and `native-observation.json` the retained disposition. A missing
//! disposition stays an unknown outcome; a missing, malformed, crossed,
//! contradicted or non-passing disposition is a typed refusal.

// r[impl lattice.implementation_attempt.repair_export]
// r[impl lattice.implementation_attempt.repair_export.scenario.passing]
// r[impl lattice.implementation_attempt.repair_export.scenario.refused]

type Disposition = crate::implementation_attempt::repair::state::Disposition;
type Reason = super::super::refusal::Reason;
type Record = crate::implementation_attempt::repair::revision::Record;
type Unexportable = crate::implementation_attempt::repair::exportable::Unexportable;
type Unretained = super::super::repair::Unretained;
type Verdict = crate::implementation_attempt::repair::checks::Verdict;

pub(super) fn load(inputs: &super::Admitted) -> Result<(super::content::Payload, serde_json::Value), String> {
    let request = &inputs.request;
    let directory = &inputs.directory;
    let admitted = super::super::binding::load(directory, request)?
        .ok_or("retained attempt has no persisted execution binding")?;
    if super::super::binding::refusal(directory)?.is_some()
        || super::super::session::revision::retained_block(directory)?.is_some()
    {
        return Err("repair attempt was not dispatched; it has no candidate".into());
    }
    let refuse = |reason| super::super::refusal::refuse(&request.envelope, reason);
    let observation =
        super::super::repair::retained(request, directory, &inputs.identity).map_err(
            |unretained| match unretained {
                Unretained::Missing => refuse(Reason::RepairOutcomeUnknown),
                Unretained::Malformed | Unretained::Crossed => refuse(Reason::RepairDispositionInvalid),
            },
        )?;
    let record =
        candidate(inputs, &admitted.binding_identity, &observation).map_err(|unexportable| match unexportable {
            Unexportable::NotCandidate => refuse(Reason::RepairNotCandidate),
            Unexportable::Contradicted => refuse(Reason::RepairDispositionInvalid),
        })?;
    debug_assert!(record.receipt.is_usable_candidate(), "only a produced receipt is exported");
    Ok((payload(inputs, &record.receipt)?, observation))
}

/// The passing revision the observation names, bound to this request's
/// envelope and execution binding and to its own passing step record.
fn candidate(
    inputs: &super::Admitted,
    binding_identity: &str,
    observation: &serde_json::Value,
) -> Result<Record, Unexportable> {
    let field = |name: &str| observation.get(name).ok_or(Unexportable::Contradicted);
    let disposition: Disposition =
        serde::Deserialize::deserialize(field("disposition")?).map_err(|_| Unexportable::Contradicted)?;
    let steps = field("steps")?.as_u64().and_then(|steps| u32::try_from(steps).ok());
    let revisions: Vec<Record> =
        serde::Deserialize::deserialize(field("revisions")?).map_err(|_| Unexportable::Contradicted)?;
    debug_assert_eq!(inputs.request.admitted_envelope, inputs.request.envelope.identity(), "admission checked it");
    let binding = crate::implementation_attempt::repair::StepBinding::of(&inputs.request.envelope);
    let retained = crate::implementation_attempt::repair::exportable::Retained {
        disposition: &disposition,
        steps: steps.ok_or(Unexportable::Contradicted)?,
        revisions: &revisions,
    };
    let record = crate::implementation_attempt::repair::exportable::candidate(&binding, retained)?;
    debug_assert_eq!(Some(record), revisions.last(), "the named revision is the last recorded one");
    let envelope = serde_json::to_value(&inputs.request.envelope).map_err(|_| Unexportable::Contradicted)?;
    let is_bound = field("envelope")? == &envelope
        && field("execution_binding")?.as_str() == Some(binding_identity)
        && record.receipt.is_usable_candidate()
        && super::content::receipt_matches(&record.receipt, &inputs.request)
        && is_passed_step(inputs, record);
    if !is_bound {
        return Err(Unexportable::Contradicted);
    }
    Ok(record.clone())
}

/// Whether the step that recorded `record` retained it with a pass.
fn is_passed_step(inputs: &super::Admitted, record: &Record) -> bool {
    let Some(step) = super::super::repair::step_record(&inputs.directory, &inputs.identity, record.iteration) else {
        return false;
    };
    let recorded = step.get("candidate").and_then(|value| <Record as serde::Deserialize>::deserialize(value).ok());
    let verdict = step.get("verdict").and_then(|value| <Verdict as serde::Deserialize>::deserialize(value).ok());
    recorded.as_ref() == Some(record) && verdict == Some(Verdict::Passed)
}

/// The workspace as the exact candidate of `receipt`.
fn payload(
    inputs: &super::Admitted,
    receipt: &crate::candidate::CandidateReceipt,
) -> Result<super::content::Payload, String> {
    let snapshot = crate::candidate_capture::workspace(&inputs.workspace).map_err(|error| error.to_string())?;
    let identity = crate::candidate::snapshot_identity(&snapshot).map_err(|error| error.to_string())?;
    if receipt.candidate_identity.as_deref() != Some(identity.as_str()) {
        return Err("workspace differs from the exact retained candidate revision".into());
    }
    let bytes = serde_json::to_vec_pretty(receipt).map_err(|error| error.to_string())?;
    super::content::payload(&inputs.workspace, snapshot, bytes)
}

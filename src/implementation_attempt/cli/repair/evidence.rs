//! The complete, request-bound retained record set of one repair attempt.
//!
//! Export and inventory act only on this: the typed disposition, refused on
//! any unknown, missing or contradicting field, bound to this request's
//! bytes, envelope, persisted execution binding, policy, lineage and dispatch
//! plan; and the typed intent and record of every step it counts, each this
//! attempt's step at its iteration. Nothing is written.

// r[impl lattice.implementation_attempt.repair_export.scenario.inventory]

type Admitted = lattice_core::execution_binding::admission::bound::Admitted;
type Observation = crate::implementation_attempt::repair::retained::Observation;
type StepRecord = crate::implementation_attempt::repair::retained::StepRecord;
type Unretained = super::records::Unretained;

/// One attempt's retained repair records.
pub(in super::super) struct RecordSet {
    pub(in super::super) observation: Observation,
    /// The disposition exactly as retained.
    pub(in super::super) retained: serde_json::Value,
    /// The record of every step the disposition counts, in order.
    pub(in super::super) steps: Vec<StepRecord>,
    /// A step intent after the counted steps without a record.
    pub(in super::super) has_unrecorded_intent: bool,
}

impl RecordSet {
    /// The records the pure core replays.
    pub(in super::super) fn view(&self) -> crate::implementation_attempt::repair::exportable::Retained<'_> {
        crate::implementation_attempt::repair::exportable::Retained {
            disposition: &self.observation.disposition,
            revisions: &self.observation.revisions,
            steps: &self.steps,
        }
    }
}

/// The record set of this exact request under `admitted`, its persisted
/// execution binding.
pub(super) fn read(
    request: &super::super::Request,
    directory: &std::path::Path,
    identity: &str,
    admitted: &Admitted,
) -> Result<RecordSet, Unretained> {
    debug_assert_eq!(admitted.attempt.attempt_id, request.envelope.attempt_id, "a loaded binding names this attempt");
    let records = super::records::Retained::at(directory, identity);
    let retained: serde_json::Value = typed(&records, super::records::DISPOSITION_FILE)?.ok_or(Unretained::Missing)?;
    let schema = retained.get("schema").and_then(serde_json::Value::as_str);
    if schema == Some(crate::implementation_attempt::repair::retained::LEGACY_OBSERVATION_SCHEMA) {
        return Err(Unretained::Legacy);
    }
    let observation: Observation = serde::Deserialize::deserialize(&retained).map_err(|_| Unretained::Malformed)?;
    if observation.schema != crate::implementation_attempt::repair::retained::OBSERVATION_SCHEMA {
        return Err(Unretained::Malformed);
    }
    let is_same_request =
        records.is_same_request(request, Some(&observation.worker_request), Some(&observation.request_identity));
    if !is_same_request || !is_bound(&observation, request, &admitted.binding_identity) {
        return Err(Unretained::Crossed);
    }
    let policy = request.repair.as_ref().ok_or(Unretained::Crossed)?;
    let max_steps = policy
        .max_iterations
        .min(crate::implementation_attempt::repair::MAX_REPAIR_ITERATIONS)
        .saturating_add(1);
    if observation.steps > max_steps {
        return Err(Unretained::Malformed);
    }
    let binding = crate::implementation_attempt::repair::StepBinding::of(&request.envelope);
    let mut steps = Vec::with_capacity(usize::try_from(observation.steps).map_err(|_| Unretained::Malformed)?);
    for iteration in 1..=observation.steps {
        let intent: crate::implementation_attempt::repair::step::Request =
            typed(&records, &format!("step-{iteration}.intent"))?.ok_or(Unretained::Malformed)?;
        let step: StepRecord = typed(&records, &format!("step-{iteration}.json"))?.ok_or(Unretained::Malformed)?;
        let is_step = step.schema == crate::implementation_attempt::repair::retained::STEP_SCHEMA
            && step.step == intent
            && intent.iteration == iteration
            && intent.binding == binding;
        if !is_step {
            return Err(Unretained::Malformed);
        }
        steps.push(step);
    }
    let candidates = steps.iter().filter_map(|step| step.candidate.as_ref());
    if !candidates.eq(observation.revisions.iter()) {
        return Err(Unretained::Malformed);
    }
    let next = observation.steps.saturating_add(1);
    let has_unrecorded_intent = is_present(&records, &format!("step-{next}.intent"))?;
    let is_beyond = is_present(&records, &format!("step-{next}.json"))?
        || is_present(&records, &format!("step-{}.intent", next.saturating_add(1)))?;
    if is_beyond {
        return Err(Unretained::Malformed);
    }
    debug_assert!(u32::try_from(steps.len()).is_ok_and(|count| count == observation.steps));
    Ok(RecordSet {
        observation,
        retained,
        steps,
        has_unrecorded_intent,
    })
}

/// Whether the disposition is this request's, under its persisted execution
/// binding, and claims neither acceptance nor integration.
fn is_bound(observation: &Observation, request: &super::super::Request, binding_identity: &str) -> bool {
    let lineage = request.lineage.as_ref();
    let withdrawn = match &observation.disposition {
        crate::implementation_attempt::repair::state::Disposition::AuthorityWithdrawn { cause } => Some(*cause),
        _ => None,
    };
    observation.envelope == request.envelope
        && observation.execution_binding == binding_identity
        && request.repair.as_ref().is_some_and(|policy| policy.identity() == observation.repair_policy)
        && observation.budget_lineage == lineage.map(crate::implementation_attempt::lineage::Binding::identity)
        && observation.unaccounted_dimensions
            == lineage.map(crate::implementation_attempt::lineage::Binding::unaccounted)
        && observation.dispatch_plan == request.dispatch_plan
        && observation.authority_withdrawn.as_ref().map(|withdrawal| withdrawal.cause) == withdrawn
        && !observation.accepted
        && !observation.integrated
}

fn typed<T: serde::de::DeserializeOwned>(
    records: &super::records::Retained,
    name: &str,
) -> Result<Option<T>, Unretained> {
    let Some(bytes) = records.read(name).map_err(|_| Unretained::Malformed)? else {
        return Ok(None);
    };
    serde_json::from_slice(&bytes).map(Some).map_err(|_| Unretained::Malformed)
}

fn is_present(records: &super::records::Retained, name: &str) -> Result<bool, Unretained> {
    records.read(name).map(|bytes| bytes.is_some()).map_err(|_| Unretained::Malformed)
}

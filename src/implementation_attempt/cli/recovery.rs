/// Reconcile retained results only. The persisted binding identity is
/// reported unchanged; nothing here observes, rebinds or releases compute.
// r[impl lattice.execution_binding.recovery.scenario.lost_acknowledgement]
pub(super) fn recover(
    store: &crate::store::Store,
    request: &super::Request,
    directory: &std::path::Path,
    binding_identity: &str,
) -> Result<serde_json::Value, String> {
    let result = match crate::candidate_worker::reconcile_attempt(store, &request.envelope.attempt_id)
        .map_err(|error| error.to_string())?
    {
        crate::candidate_worker::Recovery::Published(result) => result,
        crate::candidate_worker::Recovery::Missing | crate::candidate_worker::Recovery::Incomplete { .. } => {
            return Err(format!(
                "possible prior execution has no complete retained result; retain workspace and reconcile, do not rerun; execution binding {binding_identity}"
            ));
        }
    };
    super::artifacts::verify_events(directory, request.output_limit_bytes, &result.stdout_blake3)?;
    validate_receipt(&result, request)?;
    let unchanged = load_unchanged(request, directory)?;
    debug_assert!(result.validate().is_ok(), "reconciled results are validated on load");
    debug_assert_eq!(result.stdout_blake3.len(), 64);
    // Retained request evidence is reported read-only; no serving port is called.
    // r[impl lattice.serving_binding.usage.scenario.dedup]
    let admitted = super::binding::load(directory, request)?.ok_or("retained result has no execution binding")?;
    let mut projected = project(request, directory, &result, unchanged.as_ref());
    if let Some(fields) = projected.as_object_mut() {
        fields.insert("execution_binding".to_owned(), serde_json::Value::from(binding_identity));
        super::binding::usage::attach(fields, request, &admitted, directory)?;
    }
    Ok(projected)
}

/// A pre-spawn refusal with no worker artifacts is a definite non-dispatch.
pub(super) fn not_dispatched(
    request: &super::Request,
    directory: &std::path::Path,
    admitted: &lattice_core::execution_binding::admission::bound::Admitted,
    refusal: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    let is_same_binding =
        refusal.get("binding_identity").and_then(serde_json::Value::as_str) == Some(admitted.binding_identity.as_str());
    let has_worker_artifacts = ["raw-events.jsonl", "omp-observation.json"]
        .iter()
        .any(|name| std::fs::symlink_metadata(directory.join(name)).is_ok());
    if !is_same_binding || has_worker_artifacts {
        return Err("binding refusal contradicts the retained binding or worker artifacts".into());
    }
    debug_assert!(is_same_binding && !has_worker_artifacts);
    debug_assert_eq!(admitted.attempt.attempt_id, request.envelope.attempt_id, "the binding names this attempt");
    let mut observation = serde_json::json!({
        "schema": "lattice.implementation-attempt-observation.v1",
        "envelope": request.envelope,
        "request_identity": request.admitted_envelope,
        "envelope_attempt": admitted.attempt.attempt_id,
        "status": "not_dispatched_binding_rejected",
        "execution_binding": admitted.binding_identity,
        "revalidation": refusal.get("decision"),
        "accepted": false,
        "integrated": false,
    });
    echo_dispatch_plan(request, &mut observation);
    Ok(observation)
}

/// A managed session blocked before spawn, with no worker artifacts, is a
/// definite non-dispatch.
// r[impl lattice.agent_revisions.seam.scenario.attempt]
pub(super) fn revision_blocked(
    request: &super::Request,
    directory: &std::path::Path,
    admitted: &lattice_core::execution_binding::admission::bound::Admitted,
    block: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    let has_worker_artifacts = ["raw-events.jsonl", "omp-observation.json"]
        .iter()
        .any(|name| std::fs::symlink_metadata(directory.join(name)).is_ok());
    let is_same_session = block.get("session").and_then(serde_json::Value::as_str)
        == Some(super::session::revision::identifier(request).as_str());
    if has_worker_artifacts || !is_same_session {
        return Err("revision block contradicts the retained session or worker artifacts".into());
    }
    debug_assert_eq!(admitted.attempt.attempt_id, request.envelope.attempt_id, "the binding names this attempt");
    let mut observation = serde_json::json!({
        "schema": "lattice.implementation-attempt-observation.v1",
        "envelope": request.envelope,
        "request_identity": request.admitted_envelope,
        "envelope_attempt": admitted.attempt.attempt_id,
        "status": "not_dispatched_revision_blocked",
        "execution_binding": admitted.binding_identity,
        "revision_refusal": block.get("refusal"),
        "accepted": false,
        "integrated": false,
    });
    echo_dispatch_plan(request, &mut observation);
    Ok(observation)
}

const CLOSED_PREFIX: &str = "closed-before-execution:";
const DIGEST_HEX: usize = 64;

/// The exclusive intent content of an attempt closed before any execution, by
/// recovery or by a refusal before its intent: the request identity and the
/// BLAKE3 of the envelope's work. It never equals a request identity, so a
/// later run of the same request cannot create its intent.
pub(super) fn closed_marker(
    request_identity: &str,
    envelope: &crate::implementation_attempt::ImplementationAttemptEnvelope,
) -> String {
    format!("{CLOSED_PREFIX}{request_identity}:{}", blake3::hash(envelope.work_id.as_bytes()).to_hex())
}

/// Whether retained intent bytes are a closure marker of any request.
pub(super) fn is_closed_marker(bytes: &[u8]) -> bool {
    bytes.starts_with(CLOSED_PREFIX.as_bytes())
}

/// Whether `bytes` are the closure marker of the request `request_identity`,
/// naming its work or, as written before markers named their work, not.
pub(super) fn is_closed_by(bytes: &[u8], request_identity: &str) -> bool {
    let rest = bytes
        .strip_prefix(CLOSED_PREFIX.as_bytes())
        .and_then(|rest| rest.strip_prefix(request_identity.as_bytes()));
    rest.is_some_and(|rest| rest.is_empty() || closed_work(bytes).is_some())
}

/// The work digest a closure marker names; `None` for a marker written
/// before markers named their work, or for other bytes.
fn closed_work(bytes: &[u8]) -> Option<&[u8]> {
    let rest = bytes.strip_prefix(CLOSED_PREFIX.as_bytes())?;
    let (identity, named) = rest.split_at_checked(DIGEST_HEX)?;
    let work = named.strip_prefix(b":")?;
    (identity.len() == DIGEST_HEX && work.len() == DIGEST_HEX).then_some(work)
}

/// The refusal of a request whose attempt directory holds another request's
/// intent or closure marker, decided under its work's lock. Attempt ids must
/// be unique across the work items of one state root; one that is not finds
/// another work's marker or intent there, which is `crossed_attempt`, with
/// `dispatched:true` exactly for an execution intent (`refuse_crossed`).
/// Another marker of this work, or one that names no work, is
/// `attempt_closed`; another intent of this work is `substituted_request`.
pub(super) fn refuse_retained(
    guard: &super::session::fencing::Guard,
    state: &std::path::Path,
    request: &super::Request,
    directory: &std::path::Path,
) -> Result<String, String> {
    let envelope = &request.envelope;
    let bytes = super::session::intent::retained_bytes(directory)?.ok_or("retained intent disappeared")?;
    let work = blake3::hash(envelope.work_id.as_bytes()).to_hex();
    let reason = if is_closed_marker(&bytes) {
        match closed_work(&bytes) {
            Some(named) if named != work.as_bytes() => super::refusal::Reason::CrossedAttempt,
            Some(_) | None => super::refusal::Reason::AttemptClosed,
        }
    } else if !guard.records_attempt(envelope)? && super::session::fencing::is_recorded_elsewhere(state, envelope)? {
        super::refusal::Reason::CrossedAttempt
    } else {
        super::refusal::Reason::SubstitutedRequest
    };
    Ok(match reason {
        super::refusal::Reason::CrossedAttempt => super::refusal::refuse_crossed(envelope, directory)?,
        _ => super::refusal::refuse(envelope, reason),
    })
}

/// Recovery with no intent closes the attempt by exclusively creating its
/// intent path, so a racing or later run of the same request can never create
/// its own intent. The caller holds the per-work fence lock. Another
/// request's intent or closure marker is refused (`refuse_retained`). `None`
/// means the attempt was closed before execution; otherwise the admission
/// recorded with the intent is returned, never re-derived.
// r[impl lattice.implementation_attempt.consumer.scenario.recovery]
// r[impl lattice.implementation_attempt.consumer.scenario.fence_lock]
pub(super) fn reconcile_intent(
    guard: &super::session::fencing::Guard,
    state: &std::path::Path,
    request: &super::Request,
    directory: &std::path::Path,
    identity: &str,
) -> Result<Option<super::session::intent::Recorded>, String> {
    let session = super::session::revision::identifier(request);
    let subject = super::session::intent::Subject {
        request_identity: identity,
        session: &session,
    };
    let marker = closed_marker(identity, &request.envelope);
    debug_assert_eq!(identity.len(), DIGEST_HEX, "the request identity is one BLAKE3 digest");
    // The marker this recovery may write reads back as a closed attempt.
    debug_assert_eq!(
        super::session::intent::classify(marker.as_bytes(), subject),
        super::session::intent::Record::Closed
    );
    let mut retained = super::session::intent::read(directory, subject)?;
    if retained.is_none() {
        if super::artifacts::create_new(&directory.join(super::session::intent::FILE), marker.as_bytes())? {
            return Ok(None);
        }
        retained = super::session::intent::read(directory, subject)?;
    }
    match retained {
        Some(super::session::intent::Record::Owned(recorded)) => Ok(Some(recorded)),
        Some(super::session::intent::Record::Closed) => Ok(None),
        Some(super::session::intent::Record::Foreign) | None => Err(refuse_retained(guard, state, request, directory)?),
    }
}

/// Recovery found no intent and closed the attempt: the worker never started
/// and never will for this attempt directory.
// r[impl lattice.implementation_attempt.consumer.scenario.recovery]
pub(super) fn not_started(request: &super::Request) -> serde_json::Value {
    let mut observation = serde_json::json!({
        "schema": "lattice.implementation-attempt-observation.v1",
        "envelope": request.envelope,
        "request_identity": request.admitted_envelope,
        "status": "not_started_closed_by_recovery",
        "result": null,
        "accepted": false,
        "integrated": false,
    });
    echo_dispatch_plan(request, &mut observation);
    observation
}

fn echo_dispatch_plan(request: &super::Request, observation: &mut serde_json::Value) {
    if let (Some(plan), Some(fields)) = (&request.dispatch_plan, observation.as_object_mut()) {
        fields.insert("dispatch_plan".to_owned(), serde_json::Value::from(plan.as_str()));
    }
}

fn validate_receipt(result: &crate::candidate_worker::AttemptResult, request: &super::Request) -> Result<(), String> {
    if result.receipt.as_ref().is_some_and(|receipt| !receipt_matches(receipt, request)) {
        return Err("retained candidate does not bind the admitted request".into());
    }
    Ok(())
}

fn receipt_matches(receipt: &crate::candidate::CandidateReceipt, request: &super::Request) -> bool {
    let fence = request.envelope.fence.to_string();
    let actual = (
        &receipt.work_id,
        &receipt.attempt_id,
        receipt.fence.as_str(),
        &receipt.workspace_id,
        &receipt.base_identity,
    );
    let expected = (
        &request.envelope.work_id,
        &request.envelope.attempt_id,
        fence.as_str(),
        &request.envelope.workspace_id,
        &request.omp.base_identity,
    );
    actual == expected
}

struct Unchanged {
    observation: crate::candidate::UnchangedObservation,
    path: std::path::PathBuf,
}

fn load_unchanged(request: &super::Request, directory: &std::path::Path) -> Result<Option<Unchanged>, String> {
    let path = directory.join("unchanged-observation.json");
    if !request.allow_unchanged || !path.is_file() {
        return Ok(None);
    }
    let bytes = super::artifacts::read_bounded(&path, super::MAX_REQUEST_BYTES)?;
    let observation = serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
    Ok(Some(Unchanged { observation, path }))
}

fn project(
    request: &super::Request,
    directory: &std::path::Path,
    result: &crate::candidate_worker::AttemptResult,
    unchanged: Option<&Unchanged>,
) -> serde_json::Value {
    let mut observation = serde_json::json!({"schema":"lattice.implementation-attempt-observation.v1",
        "envelope":request.envelope,"request_identity":request.admitted_envelope,"result":result,
        "artifacts":{"raw_events":directory.join("raw-events.jsonl"),
            "omp_observation":directory.join("omp-observation.json"),
            "unchanged_receipt":unchanged.map(|retained| &retained.path)},
        "unchanged":unchanged.map(|retained| &retained.observation),
        "accepted":false,"integrated":false});
    echo_dispatch_plan(request, &mut observation);
    observation
}

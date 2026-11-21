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
    let model_requests = super::binding::requests::coverage(&admitted, directory)?;
    let mut projected = project(request, directory, &result, unchanged.as_ref());
    if let Some(fields) = projected.as_object_mut() {
        fields.insert("execution_binding".to_owned(), serde_json::Value::from(binding_identity));
        fields.insert(
            "model_requests".to_owned(),
            serde_json::to_value(&model_requests).map_err(|error| error.to_string())?,
        );
        // r[impl lattice.implementation_attempt.lineage.scenario.usage]
        if let Some(draw) = super::repair::single_shot_draw(request) {
            fields.insert("budget_lineage".to_owned(), draw);
        }
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

/// The exclusive intent content of an attempt that recovery closed before any
/// execution. It never equals a request identity, so a later run of the same
/// request cannot create its intent.
pub(super) fn closed_marker(request_identity: &str) -> String {
    format!("closed-before-execution:{request_identity}")
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

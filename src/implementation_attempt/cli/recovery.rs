pub(super) fn recover(
    store: &crate::store::Store,
    request: &super::Request,
    directory: &std::path::Path,
) -> Result<serde_json::Value, String> {
    let result = match crate::candidate_worker::reconcile_attempt(store, &request.envelope.attempt_id)
        .map_err(|error| error.to_string())?
    {
        crate::candidate_worker::Recovery::Published(result) => result,
        crate::candidate_worker::Recovery::Missing | crate::candidate_worker::Recovery::Incomplete { .. } => {
            return Err("possible prior execution has no complete retained result; retain workspace and reconcile, do not rerun".into());
        }
    };
    super::artifacts::verify_events(directory, request.output_limit_bytes, &result.stdout_blake3)?;
    validate_receipt(&result, request)?;
    let unchanged = load_unchanged(request, directory)?;
    Ok(project(request, directory, &result, unchanged.as_ref()))
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
    serde_json::json!({"schema":"lattice.implementation-attempt-observation.v1",
        "envelope":request.envelope,"request_identity":request.admitted_envelope,"result":result,
        "artifacts":{"raw_events":directory.join("raw-events.jsonl"),
            "omp_observation":directory.join("omp-observation.json"),
            "unchanged_receipt":unchanged.map(|retained| &retained.path)},
        "unchanged":unchanged.map(|retained| &retained.observation),
        "accepted":false,"integrated":false})
}

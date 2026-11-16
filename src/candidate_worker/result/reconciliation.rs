pub(in crate::candidate_worker) fn load(
    store: &crate::store::Store,
    attempt_id: &str,
) -> Result<crate::candidate_worker::Recovery, crate::candidate_worker::Error> {
    let result = store.load_candidate_attempt_result(attempt_id)?;
    let receipt = store.load_candidate_receipt(attempt_id)?;
    let usage = store.load_command_worker_usage(attempt_id)?;
    if result
        .as_ref()
        .is_some_and(|result| result.disposition == crate::candidate::CandidateDisposition::Unknown)
    {
        return Ok(incomplete("candidate outcome is unknown; retain workspace and reconcile without redispatch"));
    }
    match result {
        Some(result) => reconcile_receipt(store, attempt_id, result, receipt),
        None => Ok(missing(receipt, usage.is_some())),
    }
}

fn missing(receipt: Option<crate::candidate::CandidateReceipt>, has_usage: bool) -> crate::candidate_worker::Recovery {
    match (receipt, has_usage) {
        (Some(_), _) => incomplete("candidate receipt exists without an attempt result"),
        (None, true) => incomplete("usage observation exists without an attempt result"),
        (None, false) => crate::candidate_worker::Recovery::Missing,
    }
}

fn reconcile_receipt(
    store: &crate::store::Store,
    attempt_id: &str,
    result: crate::candidate_worker::AttemptResult,
    receipt: Option<crate::candidate::CandidateReceipt>,
) -> Result<crate::candidate_worker::Recovery, crate::candidate_worker::Error> {
    if result.disposition != crate::candidate::CandidateDisposition::Produced {
        return Ok(crate::candidate_worker::Recovery::Published(Box::new(result)));
    }
    let Some(receipt) = receipt else {
        return Ok(incomplete("produced attempt result exists without a receipt"));
    };
    if result.receipt.as_ref() != Some(&receipt) {
        return Ok(incomplete("attempt result and stored receipt disagree"));
    }
    reconcile_artifacts(store, attempt_id, result)
}

fn reconcile_artifacts(
    store: &crate::store::Store,
    attempt_id: &str,
    result: crate::candidate_worker::AttemptResult,
) -> Result<crate::candidate_worker::Recovery, crate::candidate_worker::Error> {
    for path in &result.published_artifacts {
        if store
            .load_candidate_artifact(crate::store::ArtifactKey {
                attempt_id,
                relative_path: path,
            })?
            .is_none()
        {
            return Ok(incomplete(&format!("published artifact is missing for {path}")));
        }
    }
    Ok(crate::candidate_worker::Recovery::Published(Box::new(result)))
}

fn incomplete(blocker: &str) -> crate::candidate_worker::Recovery {
    crate::candidate_worker::Recovery::Incomplete {
        blocker: blocker.to_string(),
    }
}

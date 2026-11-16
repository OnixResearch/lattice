pub(in crate::candidate_worker) fn save(
    request: &crate::candidate_worker::Request,
    store: &crate::store::Store,
    outcome: crate::candidate_worker::Outcome,
    compatibility: Option<crate::candidate_worker::Compatibility>,
) -> crate::candidate_worker::Outcome {
    let mut result = crate::candidate_worker::AttemptResult::from_outcome(&outcome);
    result.compatibility = compatibility;
    if store.save_candidate_attempt_result(&request.attempt_id, &result).is_err() {
        return unavailable(outcome);
    }
    if retain_evidence(request, store, &outcome).is_err() {
        tracing::warn!("interrupted candidate artifact publication failed; retain workspace and in-memory evidence");
        return outcome;
    }
    if let Err(blocker) = save_receipt(store, &outcome) {
        return unknown(outcome, blocker);
    }
    outcome
}

fn save_receipt(store: &crate::store::Store, outcome: &crate::candidate_worker::Outcome) -> Result<(), String> {
    match outcome {
        crate::candidate_worker::Outcome::Produced {
            receipt, changed_files, ..
        } => save_produced(store, receipt, changed_files),
        _ => Ok(()),
    }
}

fn unavailable(outcome: crate::candidate_worker::Outcome) -> crate::candidate_worker::Outcome {
    if matches!(&outcome, crate::candidate_worker::Outcome::Interrupted { .. }) {
        return outcome;
    }
    unknown(outcome, "candidate result publication failed; retain workspace and reconcile".to_string())
}

fn unknown(outcome: crate::candidate_worker::Outcome, blocker: String) -> crate::candidate_worker::Outcome {
    let (stdout, stderr) = match outcome {
        crate::candidate_worker::Outcome::Produced { stdout, stderr, .. }
        | crate::candidate_worker::Outcome::Failed { stdout, stderr, .. }
        | crate::candidate_worker::Outcome::Unchanged { stdout, stderr, .. }
        | crate::candidate_worker::Outcome::Unknown { stdout, stderr, .. }
        | crate::candidate_worker::Outcome::Cancelled { stdout, stderr } => (stdout, stderr),
        crate::candidate_worker::Outcome::Blocked { .. } | crate::candidate_worker::Outcome::Interrupted { .. } => {
            (Vec::new(), Vec::new())
        }
    };
    crate::candidate_worker::Outcome::Unknown {
        blocker,
        stdout,
        stderr,
    }
}

fn retain_evidence(
    request: &crate::candidate_worker::Request,
    store: &crate::store::Store,
    outcome: &crate::candidate_worker::Outcome,
) -> Result<(), crate::store::StoreError> {
    if let crate::candidate_worker::Outcome::Interrupted {
        observed_files: Some(files),
        ..
    } = outcome
    {
        // A failed save must not erase the typed interruption or raw evidence.
        save_artifacts(store, &request.attempt_id, files)?;
    }
    Ok(())
}

fn save_produced(
    store: &crate::store::Store,
    receipt: &crate::candidate::CandidateReceipt,
    files: &std::collections::BTreeMap<String, Vec<u8>>,
) -> Result<(), String> {
    store
        .save_candidate_receipt(receipt)
        .map_err(|error| format!("candidate receipt publication failed: {error}"))?;
    save_artifacts(store, &receipt.attempt_id, files)
        .map_err(|error| format!("candidate artifact publication failed: {error}"))
}

fn save_artifacts(
    store: &crate::store::Store,
    attempt_id: &str,
    files: &std::collections::BTreeMap<String, Vec<u8>>,
) -> Result<(), crate::store::StoreError> {
    for (path, bytes) in files {
        store.save_candidate_artifact(
            crate::store::ArtifactKey {
                attempt_id,
                relative_path: path,
            },
            bytes,
        )?;
    }
    Ok(())
}

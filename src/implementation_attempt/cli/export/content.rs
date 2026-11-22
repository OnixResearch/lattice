pub(super) struct Payload {
    pub(super) snapshot: std::collections::BTreeMap<String, Vec<u8>>,
    pub(super) modes: std::collections::BTreeMap<String, u32>,
    pub(super) receipt: Vec<u8>,
}

const UNCHANGED_BLOCKER: &str = "unchanged source is a proposal requiring explicit independent no-change acceptance";

pub(super) fn load(inputs: &super::Admitted, retained: &super::Retained) -> Result<Payload, String> {
    match retained.result.disposition {
        crate::candidate::CandidateDisposition::Produced => produced(inputs, retained),
        crate::candidate::CandidateDisposition::Failed
            if retained.result.blocker.as_deref() == Some(UNCHANGED_BLOCKER) =>
        {
            unchanged(inputs, retained)
        }
        crate::candidate::CandidateDisposition::Blocked
        | crate::candidate::CandidateDisposition::Failed
        | crate::candidate::CandidateDisposition::Cancelled
        | crate::candidate::CandidateDisposition::Unknown => {
            Err("native attempt is not a completed candidate result".into())
        }
    }
}

fn produced(inputs: &super::Admitted, retained: &super::Retained) -> Result<Payload, String> {
    let receipt = bound_receipt(inputs, retained)?;
    let snapshot = crate::candidate_capture::workspace(&inputs.workspace).map_err(|error| error.to_string())?;
    let identity = crate::candidate::snapshot_identity(&snapshot).map_err(|error| error.to_string())?;
    if receipt.candidate_identity.as_deref() != Some(identity.as_str()) {
        return Err("workspace differs from exact retained candidate identity".into());
    }
    check_artifacts(retained, receipt, &snapshot)?;
    let bytes = serde_json::to_vec_pretty(receipt).map_err(|error| error.to_string())?;
    payload(&inputs.workspace, snapshot, bytes)
}

fn bound_receipt<'a>(
    inputs: &super::Admitted,
    retained: &'a super::Retained,
) -> Result<&'a crate::candidate::CandidateReceipt, String> {
    let receipt = retained.result.receipt.as_ref().ok_or("produced result has no retained receipt")?;
    if !receipt.is_usable_candidate() || !receipt_matches(receipt, &inputs.request) {
        return Err("retained produced receipt does not bind the request".into());
    }
    let stored = retained
        .store
        .load_candidate_receipt(&inputs.request.envelope.attempt_id)
        .map_err(|error| error.to_string())?
        .ok_or("native receipt is not retained")?;
    if stored != *receipt {
        return Err("native retained receipt differs from attempt result".into());
    }
    Ok(receipt)
}

pub(super) fn receipt_matches(receipt: &crate::candidate::CandidateReceipt, request: &super::super::Request) -> bool {
    receipt.work_id == request.envelope.work_id
        && receipt.attempt_id == request.envelope.attempt_id
        && receipt.fence == request.envelope.fence.to_string()
        && receipt.workspace_id == request.envelope.workspace_id
        && receipt.base_identity == request.omp.base_identity
}

fn check_artifacts(
    retained: &super::Retained,
    receipt: &crate::candidate::CandidateReceipt,
    snapshot: &std::collections::BTreeMap<String, Vec<u8>>,
) -> Result<(), String> {
    let artifacts = retained.store.list_candidate_artifacts(&receipt.attempt_id).map_err(|error| error.to_string())?;
    let expected_count = receipt.changed_paths.iter().filter(|path| snapshot.contains_key(*path)).count();
    let is_exact = receipt
        .changed_paths
        .iter()
        .all(|path| snapshot.get(path).is_none_or(|bytes| artifacts.get(path) == Some(bytes)));
    if artifacts.len() != expected_count || !is_exact {
        return Err("native retained candidate artifacts differ from exact workspace".into());
    }
    Ok(())
}

fn unchanged(inputs: &super::Admitted, retained: &super::Retained) -> Result<Payload, String> {
    let observed = unchanged_observation(inputs, retained)?;
    let snapshot = crate::candidate_capture::workspace(&inputs.workspace).map_err(|error| error.to_string())?;
    let identity = crate::candidate::snapshot_identity(&snapshot).map_err(|error| error.to_string())?;
    let expected = expected_unchanged(&inputs.request, &snapshot)?;
    if identity != inputs.request.omp.base_identity || observed != expected {
        return Err("native unchanged observation does not bind exact base snapshot".into());
    }
    let bytes = serde_json::to_vec_pretty(&observed).map_err(|error| error.to_string())?;
    payload(&inputs.workspace, snapshot, bytes)
}

fn unchanged_observation(
    inputs: &super::Admitted,
    retained: &super::Retained,
) -> Result<crate::candidate::UnchangedObservation, String> {
    if !inputs.request.allow_unchanged {
        return Err("unchanged result was not explicitly admitted".into());
    }
    if retained.result.receipt.is_some() {
        return Err("unchanged result must not carry a produced receipt".into());
    }
    let value = retained
        .observation
        .get("unchanged")
        .ok_or("native unchanged recovery has no retained unchanged observation")?;
    serde::Deserialize::deserialize(value).map_err(|error| error.to_string())
}

fn expected_unchanged(
    request: &super::super::Request,
    snapshot: &std::collections::BTreeMap<String, Vec<u8>>,
) -> Result<crate::candidate::UnchangedObservation, String> {
    crate::candidate::observe_unchanged_from_snapshots(
        crate::candidate::Bindings {
            work_id: &request.envelope.work_id,
            attempt_id: &request.envelope.attempt_id,
            fence: &request.envelope.fence.to_string(),
            workspace_id: &request.envelope.workspace_id,
        },
        snapshot,
        snapshot,
    )
    .map_err(|error| error.to_string())
}

pub(super) fn payload(
    workspace: &std::path::Path,
    snapshot: std::collections::BTreeMap<String, Vec<u8>>,
    receipt: Vec<u8>,
) -> Result<Payload, String> {
    let modes = crate::candidate_capture::snapshot_modes(workspace, &snapshot).map_err(|error| error.to_string())?;
    Ok(Payload {
        snapshot,
        modes,
        receipt,
    })
}

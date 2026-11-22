/// The blocker of a worker that changed a path outside its write boundary.
pub(super) const WRITE_BOUNDARY_BLOCKER: &str = "worker changed a path outside its admitted write boundary";

type Snapshot = std::collections::BTreeMap<String, Vec<u8>>;

pub(super) fn finish(
    request: &super::Request,
    root: &std::path::Path,
    base: Snapshot,
    base_modes: std::collections::BTreeMap<String, u32>,
    output: crate::agent_node::ProcessOutput,
) -> Result<super::Outcome, super::Error> {
    match capture(root, &base_modes) {
        Ok(candidate) => derive(request, &base, candidate, output),
        Err(blocker) => Ok(unknown(output, blocker)),
    }
}

fn derive(
    request: &super::Request,
    base: &Snapshot,
    candidate: Snapshot,
    output: crate::agent_node::ProcessOutput,
) -> Result<super::Outcome, super::Error> {
    match crate::candidate::produced_receipt_from_snapshots(bindings(request), base, &candidate) {
        Ok(receipt) => Ok(produced(request, receipt, candidate, output)),
        Err(crate::candidate::CandidateReceiptError::NoCandidateChanges) => {
            unchanged(request, base, &candidate, output)
        }
        Err(error) => Ok(unknown(output, format!("candidate receipt failed: {error}"))),
    }
}

fn bindings(request: &super::Request) -> crate::candidate::Bindings<'_> {
    crate::candidate::Bindings {
        work_id: &request.work_id,
        attempt_id: &request.attempt_id,
        fence: &request.fence,
        workspace_id: &request.workspace_id,
    }
}

fn produced(
    request: &super::Request,
    receipt: crate::candidate::CandidateReceipt,
    candidate: Snapshot,
    output: crate::agent_node::ProcessOutput,
) -> super::Outcome {
    if !receipt.changed_paths.iter().all(|path| within_write_paths(path, &request.boundary.write_paths)) {
        return super::Outcome::Failed {
            blocker: WRITE_BOUNDARY_BLOCKER.to_string(),
            stdout: output.stdout,
            stderr: output.stderr,
        };
    }
    materialize(receipt, candidate, output)
}

fn materialize(
    receipt: crate::candidate::CandidateReceipt,
    mut candidate: Snapshot,
    output: crate::agent_node::ProcessOutput,
) -> super::Outcome {
    let changed_files = receipt
        .changed_paths
        .iter()
        .filter_map(|path| candidate.remove(path).map(|bytes| (path.clone(), bytes)))
        .collect();
    super::Outcome::Produced {
        receipt: Box::new(receipt),
        changed_files,
        stdout: output.stdout,
        stderr: output.stderr,
    }
}

fn unchanged(
    request: &super::Request,
    base: &Snapshot,
    candidate: &Snapshot,
    output: crate::agent_node::ProcessOutput,
) -> Result<super::Outcome, super::Error> {
    Ok(super::Outcome::Unchanged {
        observation: Box::new(crate::candidate::observe_unchanged_from_snapshots(bindings(request), base, candidate)?),
        stdout: output.stdout,
        stderr: output.stderr,
    })
}

fn unknown(output: crate::agent_node::ProcessOutput, blocker: String) -> super::Outcome {
    super::Outcome::Unknown {
        blocker,
        stdout: output.stdout,
        stderr: output.stderr,
    }
}

fn within_write_paths(path: &str, write_paths: &[String]) -> bool {
    write_paths
        .iter()
        .any(|write| path == write || path.strip_prefix(write).is_some_and(|suffix| suffix.starts_with('/')))
}

fn capture(root: &std::path::Path, base_modes: &std::collections::BTreeMap<String, u32>) -> Result<Snapshot, String> {
    let candidate =
        crate::candidate_capture::workspace(root).map_err(|error| format!("candidate capture failed: {error}"))?;
    let candidate_modes = crate::candidate_capture::snapshot_modes(root, &candidate)
        .map_err(|_| "candidate file modes could not be observed".to_string())?;
    if candidate_modes.iter().any(|(path, mode)| mode_changed(path, *mode, base_modes)) {
        return Err("byte-only candidate contract rejects file-mode changes".to_string());
    }
    Ok(candidate)
}

fn mode_changed(path: &str, mode: u32, base: &std::collections::BTreeMap<String, u32>) -> bool {
    #[cfg(unix)]
    let has_changed = base.get(path).map_or(mode != 0o644, |base_mode| *base_mode != mode);
    #[cfg(not(unix))]
    let has_changed = base.get(path).is_some_and(|base_mode| *base_mode != mode);
    has_changed
}

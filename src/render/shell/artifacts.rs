use anyhow::Context;

pub(super) fn fetch_objects(
    request: &crate::render::model::Request,
    batch: &mut dyn super::Batch,
    journal: &super::Journal,
    snapshot: &molten_batch_core::ProgressSnapshot,
) -> anyhow::Result<()> {
    anyhow::ensure!(snapshot.tasks.len() <= molten_batch_core::MAX_TASKS, "artifact inventory exceeds producer bound");
    for (task_name, task) in &snapshot.tasks {
        if let molten_batch_core::ProgressTask::Accepted { output, .. } = task {
            fetch_artifact(request, batch, journal, task_name, *output)?;
        }
    }
    Ok(())
}

pub(super) fn retained_objects(
    root: &std::path::Path,
    snapshot: &molten_batch_core::ProgressSnapshot,
) -> anyhow::Result<std::collections::BTreeMap<molten_batch_core::Id, Vec<u8>>> {
    let mut objects = std::collections::BTreeMap::new();
    anyhow::ensure!(snapshot.tasks.len() <= molten_batch_core::MAX_TASKS, "artifact inventory exceeds producer bound");
    for task in snapshot.tasks.values() {
        if let molten_batch_core::ProgressTask::Accepted { output, .. } = task {
            let bytes = super::read_bounded(
                &root.join(format!("{output}.artifact")),
                u64::try_from(molten_batch_core::MAX_CONTENT_BYTES)?,
            )?;
            if objects.len() < molten_batch_core::MAX_TASKS {
                objects.insert(*output, bytes);
            } else {
                anyhow::bail!("accepted inventory exceeds the producer task bound");
            }
        }
    }
    Ok(objects)
}

/// Read-only final acceptance check used by the ordinary operation classifier.
pub(super) fn verify_committed(
    request: &crate::render::model::Request,
    receipt: &super::Receipt,
) -> anyhow::Result<()> {
    super::check_current(request)?;
    anyhow::ensure!(
        receipt.outcome == crate::render::model::Outcome::Completed && receipt.binding == request.id()?,
        "non-completed receipt"
    );
    let root = std::path::PathBuf::from(&request.journal_root).join(request.id()?.to_string());
    let lock = std::fs::File::open(root.join("writer.lock"))?;
    lock.try_lock().context("render writer still active")?;
    let accepted = accepted_record(latest_record(request, &root)?)?;
    verify_receipt(request, &root, receipt, &accepted)?;
    super::check_current(request)?;
    Ok(())
}

fn latest_record(request: &crate::render::model::Request, root: &std::path::Path) -> anyhow::Result<super::Record> {
    let mut latest: Option<super::Record> = None;
    for sequence in 0..super::MAX_RECORDS {
        let file = root.join(format!("{sequence:04}.json"));
        if !file.try_exists()? {
            break;
        }
        let record: super::Record =
            serde_json::from_slice(&super::read_bounded(&file, u64::try_from(super::MAX_RECORD_BYTES)?)?)?;
        anyhow::ensure!(
            record.binding == request.id()? && record.request == *request && record.sequence == sequence,
            "crossed committed record"
        );
        latest = Some(record);
    }
    latest.context("no durable completion record")
}

fn fetch_artifact(
    request: &crate::render::model::Request,
    batch: &mut dyn super::Batch,
    journal: &super::Journal,
    task_name: &str,
    output: molten_batch_core::Id,
) -> anyhow::Result<()> {
    let name = format!("{output}.artifact");
    let path = journal.root.join(&name);
    if !journal.dir.try_exists(&name)? {
        super::check_current(request)?;
        let bytes = download_slot(batch, &journal.root, output)?;
        anyhow::ensure!(
            bytes.len() == artifact_length_bytes(request, task_name)?,
            "accepted artifact has wrong exact length"
        );
        crate::render::model::artifact(output, &bytes, "application/octet-stream")?;
        journal.exact_file(&name, &bytes)?;
    }
    verify_artifact_file(output, &path)?;
    Ok(())
}

fn artifact_length_bytes(request: &crate::render::model::Request, task_name: &str) -> anyhow::Result<usize> {
    if task_name == "assemble" {
        request
            .manifest
            .image_bytes()
            .checked_add(molten_render_pilot::FINAL_HEADER)
            .context("image length overflow")
    } else {
        let index: u32 = task_name.strip_prefix("tile-").context("unexpected tile label")?.parse()?;
        request
            .manifest
            .tile(index)?
            .bytes()
            .checked_add(molten_render_pilot::TILE_HEADER)
            .context("tile length overflow")
    }
}

fn recompute_accepted(
    request: &crate::render::model::Request,
    root: &std::path::Path,
    accepted: &crate::render::model::AcceptedManifest,
) -> anyhow::Result<crate::render::model::AcceptedManifest> {
    let objects = retained_objects(root, &accepted.snapshot)?;
    let mut recomputed = crate::render::model::accept(request, &accepted.snapshot, &objects)?;
    let owner_file = root.join("host-execution.json");
    recomputed.host_execution = if owner_file.try_exists()? {
        Some(serde_json::from_slice(&super::read_bounded(&owner_file, 4096)?)?)
    } else {
        None
    };
    Ok(recomputed)
}

fn download_slot(
    batch: &mut dyn super::Batch,
    root: &std::path::Path,
    output: molten_batch_core::Id,
) -> anyhow::Result<Vec<u8>> {
    // One create-only slot per authoritative ID, including interrupted downloads.
    let temporary = root.join(format!("download-{output}"));
    if !temporary.try_exists()? {
        batch.get(output, &temporary)?;
    }
    super::read_bounded(&temporary, u64::try_from(molten_batch_core::MAX_CONTENT_BYTES)?)
}

fn verify_receipt(
    request: &crate::render::model::Request,
    root: &std::path::Path,
    receipt: &super::Receipt,
    accepted: &crate::render::model::AcceptedManifest,
) -> anyhow::Result<()> {
    anyhow::ensure!(
        receipt.accepted_manifest == Some(molten_orchestrator_codec::identity("lattice.render-accepted.v1", accepted)?),
        "crossed accepted manifest"
    );
    let recomputed = recompute_accepted(request, root, accepted)?;
    anyhow::ensure!(
        recomputed == *accepted && receipt.host_execution == accepted.host_execution,
        "retained accepted artifact mismatch"
    );
    Ok(())
}

fn verify_artifact_file(output: molten_batch_core::Id, path: &std::path::Path) -> anyhow::Result<()> {
    crate::render::model::artifact(
        output,
        &super::read_bounded(path, u64::try_from(molten_batch_core::MAX_CONTENT_BYTES)?)?,
        "application/octet-stream",
    )?;
    Ok(())
}

fn accepted_record(record: super::Record) -> anyhow::Result<crate::render::model::AcceptedManifest> {
    anyhow::ensure!(
        record.outcome == crate::render::model::Outcome::Completed
            && !record.cancellation_requested
            && !record.superseded,
        "completion revoked"
    );
    record.accepted.context("no accepted manifest")
}

use anyhow::Context;

/// Reservation ceilings, not measurements. The governing owner supplies this
/// identity and original absolute deadline; observation/recovery never replaces it.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Budget {
    pub identity: String,
    pub deadline_epoch_millis: u64,
    pub task_attempts: u64,
    pub fuel: u64,
    pub task_wall_millis: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Route {
    pub executable: String,
    pub executable_identity: String,
    pub client_root: String,
    /// Exact authenticated Aspen public key and numeric UDP address.
    pub controller: String,
    pub cluster: molten_batch_core::Id,
    pub generation: u64,
    pub policy_ref: molten_batch_core::Id,
    /// Opt-in conformance interruption, bound by the same immutable request.
    #[serde(default = "no_fixture_fault")]
    pub fixture_fault: Option<super::shell::Fault>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub schema: String,
    pub execution: String,
    pub attempt: String,
    pub fence: String,
    pub budget: Budget,
    pub manifest: molten_render_pilot::Manifest,
    pub route: Route,
    pub component_file: String,
    pub authority_file: String,
    pub journal_root: String,
}

/// This file is supplied by the existing execution/claim owner. Render may read
/// it but must never mint, advance or repair a fence. Revocation fails closed.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Authority {
    pub execution: String,
    pub attempt: String,
    pub fence: String,
    pub budget_identity: String,
    pub current: bool,
}

mod request;
mod validation;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    Pending,
    Unknown,
    Completed,
    Failed,
    CancellationRequested,
    Stopped,
    Superseded,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Artifact {
    pub content: molten_batch_core::Id,
    #[serde(rename = "length")]
    pub length_bytes: u64,
    pub raw_blake3: String,
    pub media_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AcceptedManifest {
    pub schema: String,
    pub binding: molten_batch_core::Id,
    pub execution: String,
    pub attempt: String,
    pub fence: String,
    pub budget: Budget,
    pub host_execution: Option<super::shell::HostExecution>,
    pub producer_revision: String,
    pub render_manifest: molten_batch_core::Id,
    pub run: molten_batch_core::Id,
    pub spec_ref: molten_batch_core::Id,
    pub snapshot: molten_batch_core::ProgressSnapshot,
    pub tiles: std::collections::BTreeMap<String, Artifact>,
    pub image: Artifact,
    pub comparison: String,
    /// Producer's result contract supplies no metered fuel/CPU/memory/billing.
    pub measured_usage: Option<serde_json::Value>,
    pub missing_usage_reason: String,
}

/// Validate producer-owned acceptance facts, including complete fences and
/// canonical request/result identities. Strings/phase alone cannot accept output.
pub fn validate_snapshot(
    request: &Request,
    prepared: &molten_render_pilot::Prepared,
    snapshot: &molten_batch_core::ProgressSnapshot,
) -> anyhow::Result<()> {
    validation::snapshot_binding(request, prepared, snapshot)?;
    for task in &prepared.job.tasks {
        validation::accepted_task(request, snapshot, task)?;
    }
    if snapshot.phase == molten_batch_core::RunPhase::Succeeded {
        anyhow::ensure!(
            !snapshot.cancel_requested
                && snapshot.tasks.values().all(|t| matches!(t, molten_batch_core::ProgressTask::Accepted { .. })),
            "exit-zero partial result"
        );
    }
    Ok(())
}

pub fn artifact(content: molten_batch_core::Id, bytes: &[u8], media_type: &str) -> anyhow::Result<Artifact> {
    anyhow::ensure!(
        molten_render_pilot::content_id(molten_batch_core::ContentKind::Data, bytes)? == content,
        "accepted content identity mismatch"
    );
    Ok(Artifact {
        content,
        length_bytes: u64::try_from(bytes.len())?,
        raw_blake3: blake3::hash(bytes).to_hex().to_string(),
        media_type: media_type.into(),
    })
}

/// Fetch by authoritative accepted IDs first. Producer assemble enforces each
/// tile's digest, assignment, exact coordinates/coverage/encoding and pixel
/// correctness. verify_final is mandatory even for a self-consistent envelope.
pub fn accept(
    request: &Request,
    snapshot: &molten_batch_core::ProgressSnapshot,
    objects: &std::collections::BTreeMap<molten_batch_core::Id, Vec<u8>>,
) -> anyhow::Result<AcceptedManifest> {
    let prepared = request.prepare()?;
    validate_snapshot(request, &prepared, snapshot)?;
    anyhow::ensure!(
        snapshot.phase == molten_batch_core::RunPhase::Succeeded && !snapshot.cancel_requested,
        "render not successfully terminal"
    );
    let mut tiles = std::collections::BTreeMap::new();
    let tile_count = usize::try_from(request.manifest.tile_count())?;
    anyhow::ensure!(tile_count < molten_batch_core::MAX_TASKS, "tile inventory exceeds producer bound");
    let mut ordered = Vec::with_capacity(tile_count);
    for index in 0..request.manifest.tile_count() {
        let name = format!("tile-{index:03}");
        let molten_batch_core::ProgressTask::Accepted { output, .. } = snapshot.tasks[&name] else {
            anyhow::bail!("missing accepted tile")
        };
        let bytes = objects.get(&output).context("accepted tile unavailable")?;
        if tiles.len() < tile_count {
            tiles.insert(name, artifact(output, bytes, "application/vnd.molten.render-tile-v1")?);
            ordered.push(bytes.clone());
        } else {
            anyhow::bail!("tile inventory exceeds the admitted manifest");
        }
    }
    assert_eq!(tiles.len(), tile_count, "one accepted artifact per exact tile label");
    assert_eq!(ordered.len(), tile_count, "assembly inputs preserve every tile in order");
    let molten_batch_core::ProgressTask::Accepted { output, .. } = snapshot.tasks["assemble"] else {
        anyhow::bail!("missing accepted assembly")
    };
    let bytes = objects.get(&output).context("accepted image unavailable")?;
    let image = artifact(output, bytes, "application/vnd.molten.render-rgb-v1")?;
    let assembly = molten_render_pilot::assemble(&request.manifest, &ordered)?;
    anyhow::ensure!(assembly == *bytes, "accepted assembly differs from exact accepted tiles");
    molten_render_pilot::verify_final(&request.manifest, bytes)?;
    Ok(AcceptedManifest {
        schema: "lattice.render-accepted.v1".into(),
        binding: request.id()?,
        execution: request.execution.clone(),
        attempt: request.attempt.clone(),
        fence: request.fence.clone(),
        budget: request.budget.clone(),
        host_execution: None,
        producer_revision: super::PRODUCER_REVISION.into(),
        render_manifest: request.manifest.id()?,
        run: snapshot.run,
        spec_ref: snapshot.spec_ref,
        snapshot: snapshot.clone(),
        tiles,
        image,
        comparison:
            "exact accepted content identity + producer strict same-profile pixels; no cross-backend equality claim"
                .into(),
        measured_usage: None,
        missing_usage_reason: "producer execution-result.v1 has no metered usage fields; reservation is not usage"
            .into(),
    })
}

fn no_fixture_fault() -> Option<super::shell::Fault> {
    None
}

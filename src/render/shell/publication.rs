use anyhow::Context;

pub(super) fn publish(
    request: &crate::render::model::Request,
    journal: &mut super::Journal,
    snapshot: &molten_batch_core::ProgressSnapshot,
    fault: super::Fault,
) -> anyhow::Result<()> {
    assert!(!journal.record.superseded, "a superseded owner cannot publish accepted artifacts");
    assert!(!journal.record.cancellation_requested, "a cancellation intent must block artifact publication");
    let objects = super::artifacts::retained_objects(&journal.root, snapshot)?;
    let mut accepted = crate::render::model::accept(request, snapshot, &objects)?;
    accepted.host_execution = journal.owner()?;
    super::check_current(request)?;
    anyhow::ensure!(
        super::now_millis()? < request.budget.deadline_epoch_millis,
        "original acceptance deadline expired"
    );
    anyhow::ensure!(fault != super::Fault::BeforePublication, "fixture interruption before accepted publication");
    publish_image(request, journal, snapshot, &objects)?;
    journal.exact_file("accepted-manifest.json", &serde_json::to_vec(&accepted)?)?;
    // Final currentness check after file publication: files alone are not an
    // accepted workflow completion. Only this exact durable record is terminal.
    super::check_current(request)?;
    anyhow::ensure!(
        super::now_millis()? < request.budget.deadline_epoch_millis,
        "original acceptance deadline expired"
    );
    journal.record.accepted = Some(accepted);
    journal.record.outcome = crate::render::model::Outcome::Completed;
    Ok(())
}

fn publish_image(
    request: &crate::render::model::Request,
    journal: &super::Journal,
    snapshot: &molten_batch_core::ProgressSnapshot,
    objects: &std::collections::BTreeMap<molten_batch_core::Id, Vec<u8>>,
) -> anyhow::Result<()> {
    let molten_batch_core::ProgressTask::Accepted { output, .. } = snapshot.tasks["assemble"] else {
        anyhow::bail!("missing image")
    };
    let image = objects.get(&output).context("missing accepted image")?;
    let pixels = molten_render_pilot::verify_final(&request.manifest, image)?;
    let mut ppm = format!("P6\n{} {}\n255\n", request.manifest.width, request.manifest.height).into_bytes();
    ppm.extend_from_slice(pixels);
    journal.exact_file("image.ppm", &ppm)?;
    Ok(())
}

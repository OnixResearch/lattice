pub(super) struct PreparedRun<'a> {
    request: &'a crate::render::model::Request,
    pub(super) prepared: molten_render_pilot::Prepared,
    pub(super) run: molten_batch_core::Id,
}

impl<'a> PreparedRun<'a> {
    pub(super) fn new(request: &'a crate::render::model::Request) -> anyhow::Result<Self> {
        let prepared = request.prepare()?;
        let run = request.run_id()?;
        Ok(Self { request, prepared, run })
    }
}

pub(super) fn apply(
    computation: &PreparedRun<'_>,
    batch: &mut dyn super::Batch,
    action: super::Action,
    fault: super::Fault,
    journal: &mut super::Journal,
) -> anyhow::Result<()> {
    let request = computation.request;
    let prepared = &computation.prepared;
    let run = computation.run;
    if action == super::Action::Cancel || super::now_millis()? >= request.budget.deadline_epoch_millis {
        journal.record.cancellation_requested = true;
        journal.record.outcome = crate::render::model::Outcome::CancellationRequested;
        journal.save()?; // cancellation intent before request; not proof of stop
        super::check_current(request)?;
        batch.cancel(run)?;
    } else if action == super::Action::Advance
        && !journal.record.dispatch_possible
        && !journal.record.cancellation_requested
    {
        anyhow::ensure!(super::now_millis()? < request.budget.deadline_epoch_millis, "original deadline expired");
        let component = super::read_bounded(
            std::path::Path::new(&request.component_file),
            u64::try_from(molten_batch_core::MAX_CONTENT_BYTES)?,
        )?;
        anyhow::ensure!(
            molten_render_pilot::content_id(molten_batch_core::ContentKind::Component, &component)?
                == request.manifest.renderer,
            "renderer component changed"
        );
        super::check_current(request)?;
        anyhow::ensure!(
            batch.put(molten_batch_core::ContentKind::Component, std::path::Path::new(&request.component_file))?
                == request.manifest.renderer,
            "crossed component publication"
        );
        for (id, bytes) in &prepared.objects {
            let path = journal.exact_file(&format!("{id}.input"), bytes)?;
            super::check_current(request)?;
            anyhow::ensure!(
                batch.put(molten_batch_core::ContentKind::Data, &path)? == *id,
                "crossed assignment publication"
            );
        }
        let job = journal.exact_file("job.json", &serde_json::to_vec(&prepared.job)?)?;
        journal.record.dispatch_possible = true;
        journal.record.outcome = crate::render::model::Outcome::Unknown;
        journal.save()?; // MUST be durable before possibly dispatching Submit
        assert!(journal.record.dispatch_possible, "durable dispatch intent is required before Submit");
        assert!(!journal.record.cancellation_requested, "cancellation intent forbids a new Submit");
        anyhow::ensure!(fault != super::Fault::AfterIntent, "fixture interruption after durable intent");
        super::check_current(request)?;
        let acknowledged = batch.submit(&request.submission_key()?, &job)?;
        anyhow::ensure!(fault != super::Fault::LostSubmitAck, "fixture discarded submit acknowledgement");
        anyhow::ensure!(acknowledged == run, "crossed submission acknowledgement");
        journal.record.acknowledged = true;
        journal.save()?;
    }
    Ok(())
}

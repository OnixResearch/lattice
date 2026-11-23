//! The bounded worker step of one admitted attempt, after its intent.

fn timeout_secs(request: &super::super::Request, now_ms: u64) -> Result<u64, String> {
    let remaining_ms =
        request.expires_at_ms.checked_sub(now_ms).ok_or("admission expired before a bounded worker step")?;
    let timeout_secs = request.envelope.budget.max_seconds.min(remaining_ms / 1000);
    if timeout_secs == 0 {
        return Err("admission expires before a bounded worker step".into());
    }
    Ok(timeout_secs)
}

/// The bounded worker request of an admitted attempt: its re-admitted
/// workspace boundary, the pinned program and sandbox, and a deadline
/// within the envelope's seconds and the admission's expiry, read on `clock`.
pub(in super::super) fn worker_request(
    request: &super::super::Request,
    clock: &dyn crate::operation::shell::Clock,
) -> Result<crate::candidate_worker::Request, String> {
    request.validate(super::super::epoch_ms(clock)?)?;
    let boundary = crate::candidate_capture::admit_workspace_boundary(
        &request.workspace_root,
        &request.read_paths,
        &request.write_paths,
        false,
        false,
    )
    .map_err(|error| error.to_string())?;
    Ok(build_worker(request, boundary, timeout_secs(request, super::super::epoch_ms(clock)?)?))
}

fn build_worker(
    request: &super::super::Request,
    boundary: crate::candidate_capture::WorkspaceBoundary,
    timeout_secs: u64,
) -> crate::candidate_worker::Request {
    crate::candidate_worker::Request {
        work_id: request.envelope.work_id.clone(),
        attempt_id: request.envelope.attempt_id.clone(),
        fence: request.envelope.fence.to_string(),
        workspace_id: request.envelope.workspace_id.clone(),
        program: request.program.clone(),
        args: Vec::new(),
        boundary,
        require_isolation: true,
        sandbox_program: Some(request.sandbox_program.clone()),
        broker: None,
        timeout_secs,
        output_limit_bytes: request.output_limit_bytes,
    }
}

pub(in super::super) struct WorkerStep {
    pub(in super::super) worker: crate::candidate_worker::Request,
    pub(in super::super) profile: crate::candidate_worker::omp::Profile,
    pub(in super::super) store: crate::store::Store,
    pub(in super::super) output_dir: std::path::PathBuf,
    pub(in super::super) is_unchanged_allowed: bool,
    pub(in super::super) receipt: super::super::binding::usage::WorkerReceipt,
    pub(in super::super) serving: Box<dyn crate::execution_binding::serving::OwnerPort>,
    /// The lineage draw to retain before the result, when the request has a lineage.
    pub(in super::super) draw: Option<super::super::repair::SingleShotDraw>,
}

impl WorkerStep {
    fn retain_events(&self, result: &crate::candidate_worker::omp::RunResult) -> Result<(), String> {
        super::super::artifacts::write_once(&self.output_dir.join("raw-events.jsonl"), result.raw_events())?;
        super::super::artifacts::write_once(
            &self.output_dir.join("omp-observation.json"),
            &serde_json::to_vec(&result.observation).map_err(|error| error.to_string())?,
        )?;
        self.retain_unchanged(result)
    }

    fn retain_unchanged(&self, result: &crate::candidate_worker::omp::RunResult) -> Result<(), String> {
        let has_completed = matches!(&result.observation,
            Ok(observation) if observation.terminal == crate::candidate_worker::omp::Terminal::Completed);
        if self.is_unchanged_allowed
            && has_completed
            && let crate::candidate_worker::Outcome::Unchanged { observation, .. } = &result.outcome
        {
            super::super::artifacts::write_once(
                &self.output_dir.join("unchanged-observation.json"),
                &serde_json::to_vec(observation).map_err(|error| error.to_string())?,
            )?;
        }
        Ok(())
    }

    /// Run the worker behind the broker, which retains request files in the
    /// attempt directory for every request it relays. A served route passes
    /// every request through the serving-owner gate; any other route keeps the
    /// broker's own checks and records each request without attributing it.
    // r[impl lattice.serving_binding.request_admission]
    // r[impl lattice.serving_binding.brokered_evidence]
    fn run_worker(
        &mut self,
        cancellation: &std::sync::atomic::AtomicBool,
        credentials: &dyn crate::credentials::CredentialProvider,
    ) -> Result<crate::candidate_worker::omp::RunResult, String> {
        let admitted = self.receipt.admitted();
        let mut files = super::super::binding::requests::RequestFiles::new(&self.output_dir)?;
        let mut served;
        let mut recorded;
        let gate: &mut dyn crate::candidate_worker::broker::ModelRequestGate = if admitted.serving.is_some() {
            let session = crate::execution_binding::serving::gate::Session::new(
                admitted,
                self.receipt.request_identity(),
                self.serving.as_mut(),
                &mut files,
            )?;
            served = crate::candidate_worker::broker::OwnerGate {
                session,
                route_name: super::super::binding::PRIMARY_ROUTE.to_owned(),
            };
            &mut served
        } else {
            recorded = super::super::binding::requests::recorded(admitted, &mut files);
            &mut recorded
        };
        // A single-shot attempt runs one broker incarnation: indices start at 0.
        let services = crate::candidate_worker::broker::Services {
            credentials,
            gate,
            first_request_index: 0,
        };
        crate::candidate_worker::omp::run(
            &self.worker,
            &self.profile,
            &crate::process::core::RuntimeConfig::default(),
            cancellation,
            services,
        )
        .map_err(|error| error.to_string())
    }

    fn observe(mut self, cancellation: &std::sync::atomic::AtomicBool) -> Result<(), String> {
        let credentials = credentials(&self.profile)?;
        let started_epoch_ms = super::super::now_ms().ok();
        let result = self.run_worker(cancellation, &credentials)?;
        let window = super::super::binding::usage::WorkerWindow {
            started_epoch_ms,
            finished_epoch_ms: super::super::now_ms().ok(),
        };
        self.retain_events(&result)?;
        // The attempt usage observation and the lineage draw precede the
        // published attempt result.
        let coverage = self.receipt.persist(&self.store, window, &self.output_dir)?;
        if let Some(draw) = &self.draw {
            draw.retain(&self.output_dir, &coverage)?;
        }
        let outcome =
            crate::candidate_worker::publish_outcome(&self.worker, &self.store, result.outcome, result.compatibility);
        let retained = crate::candidate_worker::AttemptResult::from_outcome(&outcome);
        retained.validate().map_err(|error| error.to_string())
    }
}

/// The host-only environment credential of the standalone model broker; any
/// other credential reference is refused before launch.
pub(in super::super) fn credentials(
    profile: &crate::candidate_worker::omp::Profile,
) -> Result<crate::credentials::EnvironmentCredentialProvider, String> {
    let reference = profile.model_broker.credential.as_ref();
    if reference.is_some_and(|value| value.provider != "env" || value.field.is_some()) {
        return Err("standalone model broker requires an explicit env credential reference".into());
    }
    let keys: Vec<String> = reference.map(|value| value.key.clone()).into_iter().collect();
    Ok(crate::credentials::EnvironmentCredentialProvider::from_current_env(&keys))
}

pub(in super::super) async fn execute(execution: WorkerStep, directory: &std::path::Path) -> Result<(), String> {
    let cancellation = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let identity = execution.receipt.request_identity().to_owned();
    let signal_task =
        super::super::cancellation::watch(cancellation.clone(), directory.join("cancel.request"), identity);
    let observed = tokio::task::spawn_blocking(move || execution.observe(&cancellation)).await;
    // Always stop and join the watcher, including when the blocking task failed to join.
    let signal_result = super::super::cancellation::finish(signal_task).await;
    observed.map_err(|error| error.to_string())??;
    signal_result
}

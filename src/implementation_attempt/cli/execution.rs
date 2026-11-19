fn timeout_secs(request: &super::Request, now_ms: u64) -> Result<u64, String> {
    let remaining_ms =
        request.expires_at_ms.checked_sub(now_ms).ok_or("admission expired before a bounded worker step")?;
    let timeout_secs = request.envelope.budget.max_seconds.min(remaining_ms / 1000);
    if timeout_secs == 0 {
        return Err("admission expires before a bounded worker step".into());
    }
    Ok(timeout_secs)
}

fn worker_request(request: &super::Request) -> Result<crate::candidate_worker::Request, String> {
    request.validate(super::now_ms()?)?;
    let boundary = crate::candidate_capture::admit_workspace_boundary(
        &request.workspace_root,
        &request.read_paths,
        &request.write_paths,
        false,
        false,
    )
    .map_err(|error| error.to_string())?;
    Ok(build_worker(request, boundary, timeout_secs(request, super::now_ms()?)?))
}

fn build_worker(
    request: &super::Request,
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

// r[impl lattice.execution_binding.recovery]
// r[impl lattice.execution_binding.recovery.scenario.lost_acknowledgement]
pub(super) fn recover_existing(
    request: &super::Request,
    directory: &std::path::Path,
    identity: &str,
) -> Result<(), String> {
    if super::artifacts::read_bounded(&directory.join("intent.json"), identity.len())? != identity.as_bytes() {
        return Err("request differs from persisted execution intent".into());
    }
    // Recovery reads retained evidence only; it never calls the compute port.
    let admitted = super::binding::load(directory, request)?
        .ok_or("execution intent exists without a persisted execution binding; outcome unknown")?;
    if let Some(refusal) = super::binding::refusal(directory)? {
        println!("{}", super::recovery::not_dispatched(directory, &admitted, &refusal)?);
        return Ok(());
    }
    let store_path = directory.join("candidate.redb");
    if !store_path.exists() {
        return Err(format!(
            "execution intent exists without observed result; outcome unknown; execution binding {}",
            admitted.binding_identity
        ));
    }
    debug_assert_eq!(admitted.attempt.attempt_id, request.envelope.attempt_id);
    debug_assert_eq!(admitted.profile_revision, request.envelope.worker_profile);
    let store = crate::store::Store::new(&store_path).map_err(|error| error.to_string())?;
    println!("{}", super::recovery::recover(&store, request, directory, &admitted.binding_identity)?);
    Ok(())
}

struct WorkerStep {
    worker: crate::candidate_worker::Request,
    profile: crate::candidate_worker::omp::Profile,
    store: crate::store::Store,
    output_dir: std::path::PathBuf,
    is_unchanged_allowed: bool,
    receipt: super::binding::usage::WorkerReceipt,
    serving: Box<dyn crate::execution_binding::serving::OwnerPort>,
}

impl WorkerStep {
    fn retain_events(&self, result: &crate::candidate_worker::omp::RunResult) -> Result<(), String> {
        super::artifacts::write_once(&self.output_dir.join("raw-events.jsonl"), result.raw_events())?;
        super::artifacts::write_once(
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
            super::artifacts::write_once(
                &self.output_dir.join("unchanged-observation.json"),
                &serde_json::to_vec(observation).map_err(|error| error.to_string())?,
            )?;
        }
        Ok(())
    }

    /// Run the worker behind the broker. A served route passes every request
    /// through the serving-owner gate with request files in the attempt
    /// directory; any other route keeps the broker's own checks only.
    // r[impl lattice.serving_binding.request_admission]
    fn run_worker(
        &mut self,
        cancellation: &std::sync::atomic::AtomicBool,
        credentials: &dyn crate::credentials::CredentialProvider,
    ) -> Result<crate::candidate_worker::omp::RunResult, String> {
        let admitted = self.receipt.admitted();
        let mut direct = crate::candidate_worker::broker::DirectRoute;
        let mut sink;
        let mut served;
        let gate: &mut dyn crate::candidate_worker::broker::ModelRequestGate = if admitted.serving.is_some() {
            sink = super::binding::requests::RequestFiles::new(&self.output_dir)?;
            let session = crate::execution_binding::serving::gate::Session::new(
                admitted,
                self.receipt.request_identity(),
                self.serving.as_mut(),
                &mut sink,
            )?;
            served = crate::candidate_worker::broker::OwnerGate {
                session,
                route_name: super::binding::PRIMARY_ROUTE.to_owned(),
            };
            &mut served
        } else {
            &mut direct
        };
        let services = crate::candidate_worker::broker::Services { credentials, gate };
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
        let reference = self.profile.model_broker.credential.as_ref();
        if reference.is_some_and(|value| value.provider != "env" || value.field.is_some()) {
            return Err("standalone model broker requires an explicit env credential reference".into());
        }
        let keys: Vec<String> = reference.map(|value| value.key.clone()).into_iter().collect();
        let credentials = crate::credentials::EnvironmentCredentialProvider::from_current_env(&keys);
        let started_epoch_ms = super::now_ms().ok();
        let result = self.run_worker(cancellation, &credentials)?;
        let window = super::binding::usage::WorkerWindow {
            started_epoch_ms,
            finished_epoch_ms: super::now_ms().ok(),
        };
        self.retain_events(&result)?;
        // The attempt usage observation precedes the published attempt result.
        self.receipt.persist(&self.store, window, &self.output_dir)?;
        let outcome =
            crate::candidate_worker::publish_outcome(&self.worker, &self.store, result.outcome, result.compatibility);
        let retained = crate::candidate_worker::AttemptResult::from_outcome(&outcome);
        retained.validate().map_err(|error| error.to_string())
    }
}

async fn execute(execution: WorkerStep, directory: &std::path::Path) -> Result<(), String> {
    let cancellation = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let identity = execution.receipt.request_identity().to_owned();
    let signal_task = super::cancellation::watch(cancellation.clone(), directory.join("cancel.request"), identity);
    let observed = tokio::task::spawn_blocking(move || execution.observe(&cancellation)).await;
    // Always stop and join the watcher, including when the blocking task failed to join.
    let signal_result = super::cancellation::finish(signal_task).await;
    observed.map_err(|error| error.to_string())??;
    signal_result
}

struct Context {
    request: super::Request,
    directory: std::path::PathBuf,
    identity: String,
}

impl Context {
    fn load(request_path: &std::path::Path, state: &std::path::Path) -> Result<Self, String> {
        let (request, bytes) = super::artifacts::load_request(request_path)?;
        // Recovery does not restore expired authority; it only reads retained evidence.
        request.envelope.validate().map_err(|error| error.to_string())?;
        if request.admitted_envelope != request.envelope.identity() {
            return Err("crossed envelope".into());
        }
        super::artifacts::private_directory(state)?;
        let identity = blake3::hash(&bytes).to_hex().to_string();
        let directory = super::artifacts::directory(state, &request);
        super::artifacts::private_directory(&directory)?;
        Ok(Self {
            request,
            directory,
            identity,
        })
    }

    async fn run(self, ports: Option<super::Ports<'_>>) -> Result<(), String> {
        let intent = self.directory.join("intent.json");
        let Some(ports) = ports.filter(|_| !intent.exists()) else {
            return recover_existing(&self.request, &self.directory, &self.identity);
        };
        let worker = worker_request(&self.request)?;
        let admitted = super::binding::admit_and_persist(&self.request, &self.directory, ports.compute)?;
        let store_path = self.directory.join("candidate.redb");
        let store = crate::store::Store::new(&store_path).map_err(|error| error.to_string())?;
        if !super::artifacts::write_once(&intent, self.identity.as_bytes())? {
            return Err("concurrent execution intent requires reconciliation".into());
        }
        super::binding::revalidate(&self.directory, &admitted, ports.compute)?;
        let binding_identity = admitted.binding_identity.clone();
        let execution = WorkerStep {
            worker,
            store,
            profile: self.request.omp.clone(),
            output_dir: self.directory.clone(),
            is_unchanged_allowed: self.request.allow_unchanged,
            receipt: super::binding::usage::WorkerReceipt::new(admitted, self.identity),
            serving: ports.serving,
        };
        execute(execution, &self.directory).await?;
        let store = crate::store::Store::new(&store_path).map_err(|error| error.to_string())?;
        println!("{}", super::recovery::recover(&store, &self.request, &self.directory, &binding_identity)?);
        Ok(())
    }
}

/// `None` reconciles retained evidence only; ports are used only to admit,
/// revalidate and gate a first dispatch.
pub(super) async fn run(
    request_path: &std::path::Path,
    state: &std::path::Path,
    ports: Option<super::Ports<'_>>,
) -> Result<(), String> {
    Context::load(request_path, state)?.run(ports).await
}

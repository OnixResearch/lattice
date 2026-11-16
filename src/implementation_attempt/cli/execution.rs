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

fn recover_existing(request: &super::Request, directory: &std::path::Path, identity: &str) -> Result<(), String> {
    if super::artifacts::read_bounded(&directory.join("intent.json"), identity.len())? != identity.as_bytes() {
        return Err("request differs from persisted execution intent".into());
    }
    let store_path = directory.join("candidate.redb");
    if !store_path.exists() {
        return Err("execution intent exists without observed result; outcome unknown".into());
    }
    let store = crate::store::Store::new(&store_path).map_err(|error| error.to_string())?;
    println!("{}", super::recovery::recover(&store, request, directory)?);
    Ok(())
}

struct WorkerStep {
    worker: crate::candidate_worker::Request,
    profile: crate::candidate_worker::omp::Profile,
    store: crate::store::Store,
    output_dir: std::path::PathBuf,
    is_unchanged_allowed: bool,
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

    fn observe(self, cancellation: &std::sync::atomic::AtomicBool) -> Result<(), String> {
        let reference = self.profile.model_broker.credential.as_ref();
        if reference.is_some_and(|value| value.provider != "env" || value.field.is_some()) {
            return Err("standalone model broker requires an explicit env credential reference".into());
        }
        let keys: Vec<String> = reference.map(|value| value.key.clone()).into_iter().collect();
        let credentials = crate::credentials::EnvironmentCredentialProvider::from_current_env(&keys);
        let result = crate::candidate_worker::omp::run(
            &self.worker,
            &self.profile,
            &crate::process::core::RuntimeConfig::default(),
            cancellation,
            &credentials,
        )
        .map_err(|error| error.to_string())?;
        self.retain_events(&result)?;
        let outcome =
            crate::candidate_worker::publish_outcome(&self.worker, &self.store, result.outcome, result.compatibility);
        let retained = crate::candidate_worker::AttemptResult::from_outcome(&outcome);
        retained.validate().map_err(|error| error.to_string())
    }
}

async fn execute(
    request: &super::Request,
    store: crate::store::Store,
    worker: crate::candidate_worker::Request,
    directory: &std::path::Path,
    identity: String,
) -> Result<(), String> {
    let cancellation = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let signal_task = super::cancellation::watch(cancellation.clone(), directory.join("cancel.request"), identity);
    let execution = WorkerStep {
        worker,
        store,
        profile: request.omp.clone(),
        output_dir: directory.to_path_buf(),
        is_unchanged_allowed: request.allow_unchanged,
    };
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

    async fn run(self, is_run: bool) -> Result<(), String> {
        let intent = self.directory.join("intent.json");
        if !is_run || intent.exists() {
            return recover_existing(&self.request, &self.directory, &self.identity);
        }
        let worker = worker_request(&self.request)?;
        let store_path = self.directory.join("candidate.redb");
        let store = crate::store::Store::new(&store_path).map_err(|error| error.to_string())?;
        if !super::artifacts::write_once(&intent, self.identity.as_bytes())? {
            return Err("concurrent execution intent requires reconciliation".into());
        }
        execute(&self.request, store, worker, &self.directory, self.identity).await?;
        let store = crate::store::Store::new(&store_path).map_err(|error| error.to_string())?;
        println!("{}", super::recovery::recover(&store, &self.request, &self.directory)?);
        Ok(())
    }
}

pub(super) async fn run(request_path: &std::path::Path, state: &std::path::Path, is_run: bool) -> Result<(), String> {
    Context::load(request_path, state)?.run(is_run).await
}

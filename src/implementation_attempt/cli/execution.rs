// r[impl lattice.execution_binding.recovery]
// r[impl lattice.execution_binding.recovery.scenario.lost_acknowledgement]
pub(super) fn recover_existing(
    request: &super::Request,
    state: &std::path::Path,
    directory: &std::path::Path,
    identity: &str,
    revisions: super::session::revision::Store<'_>,
) -> Result<(), String> {
    // The crossed check and the closure marker share the fence lock with
    // every fence record and intent of this work in the state root. A pin an
    // admission of this work left without its intent is closed before
    // anything else; another work's pending admission is never touched.
    let guard = super::session::fencing::Guard::acquire(state, &request.envelope.work_id)?;
    let pending = super::session::revision::reservation::reconcile(revisions, directory, &request.envelope)?;
    if pending == super::session::revision::reservation::Reconciled::OtherWork {
        return Err(super::refusal::refuse_crossed(&request.envelope, directory)?);
    }
    guard.check_crossed(&request.envelope, directory).map_err(|error| error.report(&request.envelope))?;
    let reconciled = super::recovery::reconcile_intent(&guard, state, request, directory, identity);
    drop(guard);
    let Some(recorded) = reconciled? else {
        println!("{}", super::recovery::not_started(request));
        return Ok(());
    };
    // Recovery reads retained evidence only; it never calls the compute port.
    let admitted = super::binding::load(directory, request)?
        .ok_or("execution intent exists without a persisted execution binding; outcome unknown")?;
    if let Some(refusal) = super::binding::refusal(directory)? {
        let observation = super::recovery::not_dispatched(request, directory, &admitted, &refusal)?;
        super::session::revision::close(revisions, request, &recorded)?;
        println!("{observation}");
        return Ok(());
    }
    if let Some(block) = super::session::revision::retained_block(directory)? {
        let observation = super::recovery::revision_blocked(request, directory, &admitted, &block)?;
        super::session::revision::close(revisions, request, &recorded)?;
        println!("{observation}");
        return Ok(());
    }
    // r[impl lattice.implementation_attempt.repair.scenario.dispositions]
    if request.repair.is_some() {
        let observation = super::repair::recover(request, directory, identity, &admitted)?;
        super::session::revision::close(revisions, request, &recorded)?;
        println!("{observation}");
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
    let observation = super::recovery::recover(&store, request, directory, &admitted.binding_identity)?;
    super::session::revision::close(revisions, request, &recorded)?;
    println!("{observation}");
    Ok(())
}

struct Context {
    request: super::Request,
    state: std::path::PathBuf,
    directory: std::path::PathBuf,
    identity: String,
    /// The envelope names a revision of an earlier profile framing.
    is_superseded: bool,
}

impl Context {
    /// Read and validate the request without writing anything.
    fn load(request_path: &std::path::Path, state: &std::path::Path) -> Result<Self, String> {
        let (request, bytes) = super::artifacts::load_request(request_path)?;
        // Recovery does not restore expired authority; it only reads retained evidence.
        request.envelope.validate().map_err(|error| error.to_string())?;
        if request.admitted_envelope != request.envelope.identity() {
            return Err("crossed envelope".into());
        }
        let identity = blake3::hash(&bytes).to_hex().to_string();
        let directory = super::artifacts::directory(state, &request);
        // Attempts retained under an earlier profile-revision framing are never
        // re-executed or recovered by this framing; `run` refuses them.
        let is_superseded = !request.is_current_profile_revision()?;
        Ok(Self {
            request,
            state: state.to_path_buf(),
            directory,
            identity,
            is_superseded,
        })
    }

    /// Create the private state root and attempt directory.
    fn create_directories(&self) -> Result<(), String> {
        super::artifacts::private_directory(&self.state)?;
        super::artifacts::private_directory(&self.directory)
    }

    async fn run<'a>(
        self,
        ports: Option<super::Ports<'a>>,
        operator: Option<super::repair::operator::Standalone<'a>>,
        revisions: super::session::revision::Store<'_>,
    ) -> Result<(), String> {
        if let Some(refused) = self.superseded(revisions) {
            return Err(refused);
        }
        let intent = self.directory.join(super::session::intent::FILE);
        let Some(ports) = ports.filter(|_| !intent.exists()) else {
            self.create_directories()?;
            return recover_existing(&self.request, &self.state, &self.directory, &self.identity, revisions);
        };
        self.first_run(ports, operator, revisions).await
    }

    /// A first run, once no intent was seen: its admission
    /// (`admission::Context::admit_first`), then, still under the per-work
    /// lock, the execution binding, the candidate store and the intent. An
    /// intent or closure marker that appeared meanwhile is reconciled like any
    /// retained one and nothing is admitted; a session this run pinned is
    /// abandoned on every path that ends before its intent.
    // r[impl lattice.implementation_attempt.consumer.scenario.fence_lock]
    async fn first_run<'a>(
        self,
        ports: super::Ports<'a>,
        operator: Option<super::repair::operator::Standalone<'a>>,
        revisions: super::session::revision::Store<'_>,
    ) -> Result<(), String> {
        let super::Ports {
            compute,
            serving,
            repair,
        } = ports;
        let (guard, worker, repair, session) = match self.admit_first(repair, operator, revisions)? {
            admission::First::Retained(guard) => {
                drop(guard);
                return recover_existing(&self.request, &self.state, &self.directory, &self.identity, revisions);
            }
            admission::First::Admitted(ready) => {
                let admission::Ready {
                    guard,
                    worker,
                    repair,
                    session,
                } = *ready;
                (guard, worker, repair, session)
            }
        };
        let (admitted, store) = match self.prepare(&guard, &mut *compute, &session) {
            Ok(prepared) => prepared,
            Err(stop) => {
                // The pin is closed before the refusal closes the attempt.
                let abandoned = super::session::revision::abandon(revisions, &self.request, &session, &self.directory);
                let error = self.stop(&guard, revisions, stop);
                abandoned.map_err(|close| format!("{error}; {close}"))?;
                return Err(error);
            }
        };
        // The lock is released before any worker effect.
        drop(guard);
        let recorded = session.recorded.unwrap_or(super::session::intent::Recorded::UNMANAGED);
        // No worker ever spawns for this attempt once a pre-spawn step fails,
        // so its session is closed.
        let before_spawn = super::binding::revalidate(&self.directory, &admitted, compute)
            .and_then(|()| super::session::revision::permit(revisions, &self.request, &recorded, &self.directory));
        if let Err(error) = before_spawn {
            super::session::revision::close(revisions, &self.request, &recorded)
                .map_err(|close| format!("{error}; {close}"))?;
            return Err(error);
        }
        let receipt = super::binding::usage::WorkerReceipt::new(admitted, self.identity.clone());
        let ports = match repair {
            super::repair::admission::Admitted::SingleShot => {
                let binding_identity = receipt.admitted().binding_identity.clone();
                self.single_shot(worker, store, receipt, serving).await?;
                super::session::revision::close(revisions, &self.request, &recorded)?;
                let store = crate::store::Store::new(self.directory.join("candidate.redb"))
                    .map_err(|error| error.to_string())?;
                println!("{}", super::recovery::recover(&store, &self.request, &self.directory, &binding_identity)?);
                return Ok(());
            }
            super::repair::admission::Admitted::Supplied(ports) => Ok(ports),
            super::repair::admission::Admitted::Operator(granted) => Err(granted),
        };
        let authority = super::repair::Authority {
            compute,
            admitted: receipt.admitted().clone(),
            revisions,
            recorded: recorded.clone(),
            operator: None,
        };
        let run = super::repair::Run {
            request: &self.request,
            state: &self.state,
            directory: &self.directory,
            identity: &self.identity,
            receipt,
            store,
            authority,
        };
        let observation = match ports {
            Ok(ports) => super::repair::execute(run, ports)?,
            Err(granted) => super::repair::operator::execute(run, *granted)?,
        };
        super::session::revision::close(revisions, &self.request, &recorded)?;
        println!("{observation}");
        Ok(())
    }

    /// The one bounded OMP worker step of an attempt without a repair policy.
    async fn single_shot(
        &self,
        worker: crate::candidate_worker::Request,
        store: crate::store::Store,
        receipt: super::binding::usage::WorkerReceipt,
        serving: Box<dyn crate::execution_binding::serving::OwnerPort>,
    ) -> Result<(), String> {
        let execution = super::session::launch::WorkerStep {
            worker,
            store,
            profile: self.request.omp.clone(),
            output_dir: self.directory.clone(),
            is_unchanged_allowed: self.request.allow_unchanged,
            receipt,
            serving,
            draw: super::binding::draw::SingleShot::of(&self.request),
        };
        super::session::launch::execute(execution, &self.directory).await
    }
}

pub(super) mod admission;

/// `None` reconciles retained evidence only; ports are used only to admit,
/// revalidate and gate a first dispatch. `operator` is the standalone CLI's
/// repair ports, used only for a repair policy without embedder ports.
/// `revisions` names the optional agent revision store.
pub(super) async fn run(
    request_path: &std::path::Path,
    state: &std::path::Path,
    ports: Option<super::Ports<'_>>,
    revisions: super::session::revision::Store<'_>,
) -> Result<(), String> {
    run_with(request_path, state, ports, None, revisions).await
}

pub(super) async fn run_with<'a>(
    request_path: &std::path::Path,
    state: &std::path::Path,
    ports: Option<super::Ports<'a>>,
    operator: Option<super::repair::operator::Standalone<'a>>,
    revisions: super::session::revision::Store<'_>,
) -> Result<(), String> {
    Context::load(request_path, state)?.run(ports, operator, revisions).await
}

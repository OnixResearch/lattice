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

/// Recovery with no intent closes the attempt by exclusively creating its
/// intent path, so a racing or later run of the same request can never create
/// its own intent. Different request bytes are a typed substitution refusal.
/// `None` means the attempt was closed before execution; otherwise the
/// admission recorded with the intent is returned, never re-derived.
// r[impl lattice.implementation_attempt.consumer.scenario.recovery]
fn reconcile_intent(
    request: &super::Request,
    directory: &std::path::Path,
    identity: &str,
) -> Result<Option<super::session::intent::Recorded>, String> {
    let session = super::session::revision::identifier(request);
    let subject = super::session::intent::Subject {
        request_identity: identity,
        session: &session,
    };
    let marker = super::recovery::closed_marker(identity);
    debug_assert_eq!(identity.len(), 64, "the request identity is one BLAKE3 digest");
    // The marker this recovery may write reads back as a closed attempt.
    debug_assert_eq!(
        super::session::intent::classify(marker.as_bytes(), subject),
        super::session::intent::Record::Closed
    );
    let mut retained = super::session::intent::read(directory, subject)?;
    if retained.is_none() {
        if super::artifacts::create_new(&directory.join(super::session::intent::FILE), marker.as_bytes())? {
            return Ok(None);
        }
        retained = super::session::intent::read(directory, subject)?;
    }
    match retained {
        Some(super::session::intent::Record::Owned(recorded)) => Ok(Some(recorded)),
        Some(super::session::intent::Record::Closed) => Ok(None),
        Some(super::session::intent::Record::Foreign) | None => {
            Err(super::refusal::refuse(&request.envelope, super::refusal::Reason::SubstitutedRequest))
        }
    }
}

// r[impl lattice.execution_binding.recovery]
// r[impl lattice.execution_binding.recovery.scenario.lost_acknowledgement]
pub(super) fn recover_existing(
    request: &super::Request,
    state: &std::path::Path,
    directory: &std::path::Path,
    identity: &str,
    revisions: super::session::revision::Store<'_>,
) -> Result<(), String> {
    super::session::fencing::check(state, &request.envelope)?;
    let Some(recorded) = reconcile_intent(request, directory, identity)? else {
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
}

impl Context {
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
        // re-executed or recovered by this framing. One whose directory holds an
        // execution intent may have run under that framing.
        if !request.is_current_profile_revision()? {
            let session = super::session::revision::identifier(&request);
            let subject = super::session::intent::Subject {
                request_identity: &identity,
                session: &session,
            };
            let retained = super::session::intent::read(&directory, subject)?;
            let has_retained_intent = !matches!(retained, None | Some(super::session::intent::Record::Closed));
            return Err(super::refusal::refuse_superseded(&request.envelope, has_retained_intent));
        }
        super::artifacts::private_directory(state)?;
        super::artifacts::private_directory(&directory)?;
        Ok(Self {
            request,
            state: state.to_path_buf(),
            directory,
            identity,
        })
    }

    async fn run(
        self,
        ports: Option<super::Ports<'_>>,
        revisions: super::session::revision::Store<'_>,
    ) -> Result<(), String> {
        let intent = self.directory.join(super::session::intent::FILE);
        let Some(ports) = ports.filter(|_| !intent.exists()) else {
            return recover_existing(&self.request, &self.state, &self.directory, &self.identity, revisions);
        };
        let worker = worker_request(&self.request)?;
        // The session is admitted before anything is written; a refusal starts nothing.
        let session = super::session::revision::admit(revisions, &self.request)?;
        let prepared = self.prepare(ports.compute);
        let (admitted, store) = match prepared {
            Ok(prepared) => prepared,
            Err(error) => {
                super::session::revision::abandon(revisions, &self.request, &session)
                    .map_err(|close| format!("{error}; {close}"))?;
                return Err(error);
            }
        };
        let identifier = super::session::revision::identifier(&self.request);
        let subject = super::session::intent::Subject {
            request_identity: &self.identity,
            session: &identifier,
        };
        let bytes = super::session::intent::encode(subject, session.recorded.as_ref())?;
        if !super::artifacts::write_once(&intent, &bytes)? {
            return Err("concurrent execution intent requires reconciliation".into());
        }
        let recorded = session.recorded.unwrap_or(super::session::intent::Recorded::UNMANAGED);
        // No worker ever spawns for this attempt once a pre-spawn step fails,
        // so its session is closed.
        let before_spawn = super::binding::revalidate(&self.directory, &admitted, ports.compute)
            .and_then(|()| super::session::revision::permit(revisions, &self.request, &recorded, &self.directory));
        if let Err(error) = before_spawn {
            super::session::revision::close(revisions, &self.request, &recorded)
                .map_err(|close| format!("{error}; {close}"))?;
            return Err(error);
        }
        let binding_identity = admitted.binding_identity.clone();
        let execution = super::session::launch::WorkerStep {
            worker,
            store,
            profile: self.request.omp.clone(),
            output_dir: self.directory.clone(),
            is_unchanged_allowed: self.request.allow_unchanged,
            receipt: super::binding::usage::WorkerReceipt::new(admitted, self.identity),
            serving: ports.serving,
        };
        super::session::launch::execute(execution, &self.directory).await?;
        super::session::revision::close(revisions, &self.request, &recorded)?;
        let store =
            crate::store::Store::new(self.directory.join("candidate.redb")).map_err(|error| error.to_string())?;
        println!("{}", super::recovery::recover(&store, &self.request, &self.directory, &binding_identity)?);
        Ok(())
    }

    /// The fence record, persisted binding and candidate store, all before
    /// the intent.
    fn prepare(
        &self,
        compute: &mut dyn crate::execution_binding::ComputeAllocationPort,
    ) -> Result<(lattice_core::execution_binding::admission::bound::Admitted, crate::store::Store), String> {
        // r[impl lattice.implementation_attempt.consumer.scenario.refusal]
        super::session::fencing::admit(&self.state, &self.request.envelope)?;
        let admitted = super::binding::admit_and_persist(&self.request, &self.directory, compute)?;
        let store =
            crate::store::Store::new(self.directory.join("candidate.redb")).map_err(|error| error.to_string())?;
        Ok((admitted, store))
    }
}

/// `None` reconciles retained evidence only; ports are used only to admit,
/// revalidate and gate a first dispatch. `revisions` names the optional
/// agent revision store.
pub(super) async fn run(
    request_path: &std::path::Path,
    state: &std::path::Path,
    ports: Option<super::Ports<'_>>,
    revisions: super::session::revision::Store<'_>,
) -> Result<(), String> {
    Context::load(request_path, state)?.run(ports, revisions).await
}

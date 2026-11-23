//! The repair loop's check port over the operator's check catalog.
//!
//! Each declared check runs as one bounded process through the candidate
//! worker's generic bubblewrap route, on the read-only copy of the candidate
//! the loop materialized for it: a cleared environment, a private network,
//! no model broker and no credential, no write path, and bounded output.
//! Before each check the operator's grant and the admission are rechecked
//! and its deadline is capped by the admission's expiry. Only an exit
//! observed on an unchanged copy decides the check.

// r[impl lattice.implementation_attempt.operator_repair.scenario.checks]
// r[impl lattice.implementation_attempt.operator_repair.scenario.authority]

type Pins = crate::implementation_attempt::repair::checks::catalog::Pins;
type CatalogError = crate::implementation_attempt::repair::checks::catalog::Error;
type Observation = crate::implementation_attempt::repair::checks::Observation;
type CheckOutcome = crate::implementation_attempt::repair::checks::Outcome;

/// Runs one check process; the shell owns everything around it.
pub(crate) trait Execute: Send {
    fn execute(
        &mut self,
        check: &crate::candidate_worker::Request,
        cancellation: &std::sync::atomic::AtomicBool,
    ) -> Result<crate::candidate_worker::Outcome, String>;
}

/// The candidate worker's generic bubblewrap route
/// (`candidate_worker::run`), the same workspace provider the OMP worker
/// uses, without a broker.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct Sandboxed;

impl Execute for Sandboxed {
    fn execute(
        &mut self,
        check: &crate::candidate_worker::Request,
        cancellation: &std::sync::atomic::AtomicBool,
    ) -> Result<crate::candidate_worker::Outcome, String> {
        debug_assert!(check.require_isolation && check.broker.is_none() && !check.boundary.network);
        let config = crate::process::core::RuntimeConfig::default();
        crate::candidate_worker::run(check, &config, cancellation).map_err(|error| error.to_string())
    }
}

/// Read the operator's catalog at `path`, outside `workspace`, admit it for
/// `policy`, and verify every pinned executable's bytes. The file is opened
/// without following links and without blocking, and must be a bounded
/// regular file, so a FIFO cannot stall the per-work lock.
pub(in crate::implementation_attempt::cli) fn load(
    path: &std::path::Path,
    workspace: &std::path::Path,
    policy: &crate::implementation_attempt::repair::Policy,
) -> Result<Pins, CatalogError> {
    let canonical = std::fs::canonicalize(path).map_err(|_| CatalogError::Unavailable)?;
    let root = std::fs::canonicalize(workspace).map_err(|_| CatalogError::Unavailable)?;
    if canonical.starts_with(&root) {
        return Err(CatalogError::Workspace);
    }
    debug_assert!(canonical.is_absolute() && root.is_absolute(), "canonical paths are absolute");
    let bytes = read(&canonical)?;
    let catalog = crate::implementation_attempt::repair::checks::catalog::parse(&bytes)?;
    catalog.admit(policy)?;
    verify(&catalog)?;
    debug_assert_eq!(catalog.checks.len(), policy.checks.len(), "an admitted catalog holds the policy's checks");
    Ok(catalog)
}

/// The bytes of the regular file at `path`, at most the catalog bound.
fn read(path: &std::path::Path) -> Result<Vec<u8>, CatalogError> {
    use std::io::Read as _;
    use std::os::unix::fs::OpenOptionsExt as _;
    let limit_bytes = crate::implementation_attempt::repair::checks::catalog::MAX_CATALOG_BYTES;
    let file = std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(path)
        .map_err(|_| CatalogError::Unavailable)?;
    let metadata = file.metadata().map_err(|_| CatalogError::Unavailable)?;
    debug_assert!(limit_bytes > 0, "the catalog bound admits a catalog");
    if !metadata.is_file() {
        return Err(CatalogError::Unavailable);
    }
    let length_bytes = usize::try_from(metadata.len()).map_err(|_| CatalogError::Oversized)?;
    if length_bytes > limit_bytes {
        return Err(CatalogError::Oversized);
    }
    let probe = u64::try_from(limit_bytes).map_err(|_| CatalogError::Oversized)?.saturating_add(1);
    let mut bytes = Vec::with_capacity(length_bytes);
    file.take(probe).read_to_end(&mut bytes).map_err(|_| CatalogError::Unavailable)?;
    debug_assert!(bytes.len() <= limit_bytes.saturating_add(1), "the read is bounded by its probe");
    if bytes.len() > limit_bytes {
        return Err(CatalogError::Oversized);
    }
    Ok(bytes)
}

/// Every pinned executable resolves inside the Nix store to its bytes.
fn verify(catalog: &Pins) -> Result<(), CatalogError> {
    let mut executables = std::iter::once(&catalog.sandbox).chain(catalog.checks.iter().map(|entry| &entry.program));
    if executables.all(|executable| crate::candidate_worker::broker::admission::executable(executable).is_ok()) {
        Ok(())
    } else {
        Err(CatalogError::Mismatched)
    }
}

/// What the catalog check port of one run is bound to.
pub(in crate::implementation_attempt::cli::repair) struct Frame<'a, 'r> {
    pub(in crate::implementation_attempt::cli::repair) request: &'r super::super::super::Request,
    pub(in crate::implementation_attempt::cli::repair) catalog: Pins,
    pub(in crate::implementation_attempt::cli::repair) grant: super::Grant,
    /// The run's records, read for a persisted cancellation request.
    pub(in crate::implementation_attempt::cli::repair) cancel: super::super::records::Retained,
    pub(in crate::implementation_attempt::cli::repair) execute: &'a mut dyn Execute,
    /// The operator's clock, on which the admission's expiry is read.
    pub(in crate::implementation_attempt::cli::repair) clock: &'a dyn crate::operation::shell::Clock,
}

/// The catalog check port of one repair run.
pub(in crate::implementation_attempt::cli::repair) struct CatalogPort<'a, 'r> {
    frame: Frame<'a, 'r>,
}

impl<'a, 'r> CatalogPort<'a, 'r> {
    pub(in crate::implementation_attempt::cli::repair) fn new(frame: Frame<'a, 'r>) -> Self {
        Self { frame }
    }
}

/// A check's timeout: the loop's `deadline`, within the envelope's seconds
/// and the admission's remaining whole seconds on `clock`, and never zero.
fn bounded_timeout_secs(
    request: &super::super::super::Request,
    deadline: std::time::Duration,
    clock: &dyn crate::operation::shell::Clock,
) -> Result<u64, String> {
    let remaining_ms = request
        .expires_at_ms
        .checked_sub(super::super::super::epoch_ms(clock)?)
        .ok_or("admission expired before a check")?;
    let admitted_secs = request.envelope.budget.max_seconds.min(remaining_ms / 1000);
    let timeout_secs = crate::implementation_attempt::repair::step::operator::timeout_secs(deadline, admitted_secs);
    if timeout_secs == 0 {
        return Err("admission expires before a bounded check".into());
    }
    debug_assert!(timeout_secs <= request.envelope.budget.max_seconds, "within the envelope's seconds");
    Ok(timeout_secs)
}

/// The bounded process of `entry` on `candidate`: the request's read paths
/// that exist in the copy, read-only, and no write path, network, broker or
/// credential, stopped after `timeout_secs`.
fn process(
    request: &super::super::super::Request,
    sandbox: &crate::candidate_worker::broker::Executable,
    entry: &crate::implementation_attempt::repair::checks::catalog::Entry,
    candidate: &std::path::Path,
    timeout_secs: u64,
) -> Result<crate::candidate_worker::Request, String> {
    debug_assert!(entry.program.path.is_absolute(), "a pinned program path is absolute");
    debug_assert!(!candidate.as_os_str().is_empty(), "a check runs on its copy");
    debug_assert!(timeout_secs > 0, "a bounded check has time to run");
    let sandbox = crate::candidate_worker::broker::admission::executable(sandbox)?;
    let program = crate::candidate_worker::broker::admission::executable(&entry.program)?;
    let read_paths: Vec<String> =
        request.read_paths.iter().filter(|path| candidate.join(path.as_str()).exists()).cloned().collect();
    let boundary = crate::candidate_capture::admit_workspace_boundary(candidate, &read_paths, &[], false, false)
        .map_err(|error| error.to_string())?;
    let envelope = &request.envelope;
    Ok(crate::candidate_worker::Request {
        work_id: envelope.work_id.clone(),
        attempt_id: envelope.attempt_id.clone(),
        fence: envelope.fence.to_string(),
        workspace_id: envelope.workspace_id.clone(),
        program: path_text(program)?,
        args: entry.args.clone(),
        boundary,
        require_isolation: true,
        sandbox_program: Some(path_text(sandbox)?),
        broker: None,
        timeout_secs,
        output_limit_bytes: crate::implementation_attempt::repair::checks::catalog::OUTPUT_LIMIT_BYTES,
    })
}

fn path_text(path: std::path::PathBuf) -> Result<String, String> {
    path.into_os_string().into_string().map_err(|_| "pinned executable path is not UTF-8".to_owned())
}

impl super::super::CheckPort for CatalogPort<'_, '_> {
    fn run(
        &mut self,
        invocation: &crate::implementation_attempt::repair::checks::Invocation,
        candidate: &std::path::Path,
        deadline: std::time::Duration,
    ) -> Observation {
        let Frame {
            request,
            catalog,
            grant,
            cancel,
            execute,
            clock,
        } = &mut self.frame;
        let executor = catalog.executor();
        let Some(entry) = catalog.entry(&invocation.check) else {
            // The admitted catalog holds every declared check; any other is
            // reported as run by no admitted configuration.
            return Observation {
                check: invocation.check.clone(),
                configuration: String::new(),
                revision: invocation.revision.clone(),
                executor,
                outcome: CheckOutcome::Unknown,
            };
        };
        // A withdrawn grant, an expired admission or a cancellation leaves
        // the check unrun and its outcome unknown.
        let is_current = grant.recheck(request, *clock).is_none() && !cancel.is_cancel_requested();
        let outcome = if is_current {
            bounded_timeout_secs(request, deadline, *clock)
                .and_then(|timeout_secs| process(request, &catalog.sandbox, entry, candidate, timeout_secs))
                .and_then(|process| {
                    super::polled(|cancellation| execute.execute(&process, cancellation), &|| {
                        cancel.is_cancel_requested()
                    })
                })
                .map_or(CheckOutcome::Unknown, |observed| {
                    crate::implementation_attempt::repair::checks::catalog::outcome(&observed, &entry.failure_class)
                })
        } else {
            CheckOutcome::Unknown
        };
        Observation {
            check: invocation.check.clone(),
            configuration: entry.configuration(),
            revision: invocation.revision.clone(),
            executor,
            outcome,
        }
    }
}

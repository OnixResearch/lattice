//! The repair loop's check port over the operator's check catalog.
//!
//! Each declared check runs as one bounded process through the candidate
//! worker's generic bubblewrap route, on the read-only copy of the candidate
//! the loop materialized for it: a cleared environment, a private network,
//! no model broker and no credential, no write path, and bounded output.
//! Only an exit observed on an unchanged copy decides the check.

// r[impl lattice.implementation_attempt.operator_repair.scenario.checks]

type Pins = crate::implementation_attempt::repair::checks::catalog::Pins;
type CatalogError = crate::implementation_attempt::repair::checks::catalog::Error;
type Observation = crate::implementation_attempt::repair::checks::Observation;

/// Runs one check process; the shell owns everything around it.
pub trait Execute: Send {
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
pub struct Sandboxed;

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

/// Read the operator's catalog at `path`, outside `workspace`, verify every
/// pinned executable's bytes, and admit it for `policy`.
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
    let metadata = std::fs::symlink_metadata(&canonical).map_err(|_| CatalogError::Unavailable)?;
    let limit_bytes = u64::try_from(crate::implementation_attempt::repair::checks::catalog::MAX_CATALOG_BYTES)
        .map_err(|_| CatalogError::Oversized)?;
    if metadata.len() > limit_bytes {
        return Err(CatalogError::Oversized);
    }
    let bytes = super::super::super::artifacts::read_bounded(
        &canonical,
        crate::implementation_attempt::repair::checks::catalog::MAX_CATALOG_BYTES,
    )
    .map_err(|_| CatalogError::Unavailable)?;
    let catalog = crate::implementation_attempt::repair::checks::catalog::parse(&bytes)?;
    verify(&catalog)?;
    catalog.admit(policy)?;
    debug_assert_eq!(catalog.checks.len(), policy.checks.len(), "an admitted catalog holds the policy's checks");
    Ok(catalog)
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

/// The catalog check port of one repair run.
pub(in crate::implementation_attempt::cli) struct CatalogPort<'a, 'r> {
    request: &'r super::super::super::Request,
    catalog: Pins,
    execute: &'a mut dyn Execute,
}

impl<'a, 'r> CatalogPort<'a, 'r> {
    pub(in crate::implementation_attempt::cli) fn new(
        request: &'r super::super::super::Request,
        catalog: Pins,
        execute: &'a mut dyn Execute,
    ) -> Self {
        Self {
            request,
            catalog,
            execute,
        }
    }

    /// The bounded process of `entry` on `candidate`: the paths the worker
    /// could see, read-only, no write path, network, broker or credential.
    fn process(
        &self,
        entry: &crate::implementation_attempt::repair::checks::catalog::Entry,
        candidate: &std::path::Path,
        deadline: std::time::Duration,
    ) -> Result<crate::candidate_worker::Request, String> {
        let sandbox = crate::candidate_worker::broker::admission::executable(&self.catalog.sandbox)?;
        let program = crate::candidate_worker::broker::admission::executable(&entry.program)?;
        let mut read_paths = self.request.read_paths.clone();
        read_paths.extend(self.request.write_paths.iter().cloned());
        read_paths.sort();
        read_paths.dedup();
        let boundary = crate::candidate_capture::admit_workspace_boundary(candidate, &read_paths, &[], false, false)
            .map_err(|error| error.to_string())?;
        let envelope = &self.request.envelope;
        let admitted_secs = envelope.budget.max_seconds;
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
            timeout_secs: crate::implementation_attempt::repair::step::operator::timeout_secs(deadline, admitted_secs),
            output_limit_bytes: crate::implementation_attempt::repair::checks::catalog::OUTPUT_LIMIT_BYTES,
        })
    }
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
        let executor = self.catalog.executor();
        let Some(entry) = self.catalog.entry(&invocation.check).cloned() else {
            // The admitted catalog holds every declared check; any other is
            // reported as run by no admitted configuration.
            return Observation {
                check: invocation.check.clone(),
                configuration: String::new(),
                revision: invocation.revision.clone(),
                executor,
                outcome: crate::implementation_attempt::repair::checks::Outcome::Unknown,
            };
        };
        let cancellation = std::sync::atomic::AtomicBool::new(false);
        let outcome = self
            .process(&entry, candidate, deadline)
            .and_then(|process| self.execute.execute(&process, &cancellation))
            .map_or(crate::implementation_attempt::repair::checks::Outcome::Unknown, |observed| {
                crate::implementation_attempt::repair::checks::catalog::outcome(&observed, &entry.failure_class)
            });
        Observation {
            check: invocation.check.clone(),
            configuration: entry.configuration(),
            revision: invocation.revision.clone(),
            executor,
            outcome,
        }
    }
}

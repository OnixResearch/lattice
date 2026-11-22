//! The standalone CLI's operator-authorized repair ports.
//!
//! `lattice implementation-attempt run` supplies [`Standalone`]: the pinned
//! OMP launcher, a check launcher for the operator's check catalog
//! (`run --repair-checks`) and a clock. They are used only for a request
//! with a repair policy, only with a catalog that pins exactly the policy's
//! checks and executor, and only while the operator's `effect_authority`
//! profile grants the exact Agent effect of the OMP worker and the process
//! effect of the checks (`implementation_attempt::repair::operator`).
//! Otherwise the run is refused before the fence record, with no step
//! intent; nothing here grants anything, and an embedder's own ports take
//! precedence.

// r[impl lattice.implementation_attempt.operator_repair]
// r[impl lattice.implementation_attempt.operator_repair.scenario.authority]

pub mod checks;
mod worker;

/// Launches one pinned OMP step; the shell owns everything around it.
pub trait Launch: Send {
    /// Run `worker` with `profile` and stop it when `cancellation` is set.
    fn launch(
        &mut self,
        worker: &crate::candidate_worker::Request,
        profile: &crate::candidate_worker::omp::Profile,
        cancellation: &std::sync::atomic::AtomicBool,
    ) -> Result<crate::candidate_worker::omp::RunResult, String>;
}

/// The pinned OMP route of the single-shot attempt (`candidate_worker::omp`):
/// the same reviewed argv, byte pins, workspace boundary, bubblewrap
/// provider and attempt-local model broker, with the host-only environment
/// credential. A repair policy is never combined with a served route.
#[derive(Debug, Clone, Copy, Default)]
pub struct PinnedOmp;

impl Launch for PinnedOmp {
    fn launch(
        &mut self,
        worker: &crate::candidate_worker::Request,
        profile: &crate::candidate_worker::omp::Profile,
        cancellation: &std::sync::atomic::AtomicBool,
    ) -> Result<crate::candidate_worker::omp::RunResult, String> {
        let credentials = super::super::session::launch::credentials(profile)?;
        let mut gate = crate::candidate_worker::broker::DirectRoute;
        let services = crate::candidate_worker::broker::Services {
            credentials: &credentials,
            gate: &mut gate,
        };
        let config = crate::process::core::RuntimeConfig::default();
        crate::candidate_worker::omp::run(worker, profile, &config, cancellation, services)
            .map_err(|error| error.to_string())
    }
}

/// The standalone CLI's repair ports, before the catalog and the operator's
/// authority are checked.
pub struct Standalone<'a> {
    /// The application configuration whose `effect_authority` names the
    /// operator profile; read only for a request with a repair policy.
    pub config: &'a std::path::Path,
    /// The operator's check catalog; without one a repair request is
    /// refused as `repair_ports_unavailable`.
    pub catalog: Option<&'a std::path::Path>,
    pub launch: Box<dyn Launch>,
    pub execute: Box<dyn checks::Execute>,
    pub clock: &'a dyn crate::operation::shell::Clock,
}

impl<'a> Standalone<'a> {
    /// The pinned OMP route, the sandboxed catalog checks and the system
    /// clock of `lattice implementation-attempt run`.
    #[must_use]
    pub fn pinned(config: &'a std::path::Path, catalog: Option<&'a std::path::Path>) -> Self {
        Self {
            config,
            catalog,
            launch: Box::new(PinnedOmp),
            execute: Box::new(checks::Sandboxed),
            clock: &crate::operation::shell::SystemClock,
        }
    }
}

/// The operator profile that granted the repair effects at admission.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(in super::super) struct Grant {
    profile: std::path::PathBuf,
    identity: String,
}

/// Operator ports with an admitted catalog whose profile granted the
/// repair effects at admission.
pub(in super::super) struct Granted<'a> {
    pub(in super::super) launch: Box<dyn Launch>,
    pub(in super::super) execute: Box<dyn checks::Execute>,
    pub(in super::super) catalog: crate::implementation_attempt::repair::checks::catalog::Pins,
    pub(in super::super) clock: &'a dyn crate::operation::shell::Clock,
    pub(in super::super) grant: Grant,
}

/// Why operator ports were not admitted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Refusal {
    /// No catalog was given (`None`), or it was not admitted.
    Ports(Option<crate::implementation_attempt::repair::checks::catalog::Error>),
    Authority(crate::engine::effect::standalone::SubjectRefusal),
}

/// Before the fence record: the operator's catalog must pin exactly the
/// policy's checks and executor, and the operator's profile must grant
/// the exact repair effects of this request's profile revision now.
pub(super) fn admit<'a>(request: &super::super::Request, operator: Standalone<'a>) -> Result<Granted<'a>, Refusal> {
    let policy = request.repair.as_ref().ok_or(Refusal::Ports(None))?;
    let path = operator.catalog.ok_or(Refusal::Ports(None))?;
    let catalog = checks::load(path, &request.workspace_root, policy).map_err(|error| Refusal::Ports(Some(error)))?;
    let selected = crate::config::load_app_config(operator.config).ok().and_then(|config| config.effect_authority);
    let identity = check(request, selected.as_deref()).map_err(Refusal::Authority)?;
    let profile = selected.ok_or(Refusal::Authority(crate::engine::effect::standalone::SubjectRefusal::Unavailable))?;
    debug_assert!(profile.is_absolute(), "an evaluated profile path is absolute");
    Ok(Granted {
        launch: operator.launch,
        execute: operator.execute,
        catalog,
        clock: operator.clock,
        grant: Grant { profile, identity },
    })
}

fn check(
    request: &super::super::Request,
    profile: Option<&std::path::Path>,
) -> Result<String, crate::engine::effect::standalone::SubjectRefusal> {
    let required =
        crate::implementation_attempt::repair::step::operator::requirements(&request.envelope.worker_profile);
    let mut identity = Err(crate::engine::effect::standalone::SubjectRefusal::NotGranted);
    // Every required tuple is granted by the same evaluated profile.
    for grant in &required.grants {
        let subject = crate::engine::effect::standalone::core::Subject {
            workflow_revision: &required.workflow_revision,
            machine_identity: &required.machine_identity,
            grant,
        };
        let granted = crate::engine::effect::standalone::check_subject(profile, subject)?;
        if identity.as_ref().is_ok_and(|admitted| *admitted != granted) {
            return Err(crate::engine::effect::standalone::SubjectRefusal::NotGranted);
        }
        identity = Ok(granted);
    }
    identity
}

impl Grant {
    /// Before every step's intent: the same profile, evaluated now, must
    /// still grant the worker effect with the identity admitted, and the
    /// admission must not have expired. `Some` names what withdrew it.
    pub(in super::super) fn recheck(&self, request: &super::super::Request) -> Option<serde_json::Value> {
        match check(request, Some(&self.profile)) {
            Ok(identity) if identity == self.identity => {}
            Ok(_) => return Some(serde_json::json!({"authority_refusal": "changed"})),
            Err(refusal) => return Some(serde_json::json!({ "authority_refusal": refusal })),
        }
        match super::super::now_ms() {
            Ok(now_ms) if now_ms < request.expires_at_ms => None,
            Ok(_) => Some(serde_json::json!({"admission_expired": true})),
            Err(error) => Some(serde_json::json!({ "error": error })),
        }
    }
}

/// Drive the loop through the standalone operator's ports: the pinned OMP
/// worker and the catalog checks, with the operator's grant rechecked
/// before every step's intent.
pub(in super::super) fn execute(mut run: super::Run<'_>, granted: Granted<'_>) -> Result<serde_json::Value, String> {
    let Granted {
        mut launch,
        execute: mut process,
        catalog,
        clock,
        grant,
    } = granted;
    run.authority.operator = Some(grant);
    let mut worker = worker::OmpPort::new(run.request, run.directory, &mut *launch);
    let mut checks = checks::CatalogPort::new(run.request, catalog, &mut *process);
    let ports = super::Ports {
        worker: &mut worker,
        checks: &mut checks,
        clock,
    };
    super::execute(run, ports)
}

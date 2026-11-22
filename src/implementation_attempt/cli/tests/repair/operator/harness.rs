//! One operator-repair attempt of a fresh fixture for the operator repair
//! tests.

mod doubles;

use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering;

pub(super) use doubles::Behaviour;
pub(super) use doubles::CHECK;
pub(super) use doubles::Profile;

type Request = crate::implementation_attempt::cli::Request;
type Hook = Box<dyn FnMut() + Send>;

/// How one attempt is admitted.
pub(super) struct Options {
    pub(super) iterations: u32,
    pub(super) elapsed_ms: u64,
    pub(super) profile: Profile,
    /// The admission expires this long after the fixture is built.
    pub(super) expires_in_ms: Option<u64>,
    /// The `unit` check's shell script.
    pub(super) check: &'static str,
    /// A second check, `unit-second`, running this script.
    pub(super) second_check: Option<&'static str>,
    /// Read and write paths other than `src`; their directories are created.
    pub(super) paths: Option<(Vec<&'static str>, Vec<&'static str>)>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            iterations: 2,
            elapsed_ms: 60_000,
            profile: Profile::Granting,
            expires_in_ms: None,
            check: doubles::CHECK,
            second_check: None,
            paths: None,
        }
    }
}

/// One operator-repair attempt of a fresh fixture.
pub(super) struct Operated {
    pub(super) fixture: super::super::Fixture,
    pub(super) request: Request,
    pub(super) path: std::path::PathBuf,
    pub(super) catalog: serde_json::Value,
    script: std::path::PathBuf,
    config: std::path::PathBuf,
    profile: std::path::PathBuf,
    catalog_path: Option<std::path::PathBuf>,
    launches: Arc<AtomicU32>,
    check_runs: Arc<AtomicU32>,
    after: Arc<Mutex<Option<Hook>>>,
    before_repair: Arc<Mutex<Option<Hook>>>,
    after_check: Arc<Mutex<Option<Hook>>>,
    is_lost: Arc<AtomicBool>,
    indices: Arc<Mutex<Vec<u32>>>,
    on_launch: Arc<Mutex<Option<doubles::IndexHook>>>,
}

fn now_ms() -> u64 {
    let observed = crate::operation::shell::Clock::observe_epoch_millis(&crate::operation::shell::SystemClock);
    u64::try_from(observed).expect("clock")
}

impl Operated {
    pub(super) fn new(attempt: &str, iterations: u32, elapsed_ms: u64) -> Self {
        Self::with(attempt, Options {
            iterations,
            elapsed_ms,
            ..Options::default()
        })
    }

    pub(super) fn with_profile(attempt: &str, profile: Profile) -> Self {
        Self::with(attempt, Options {
            profile,
            ..Options::default()
        })
    }

    /// A granting run given `catalog` while its policy names the pinned one.
    pub(super) fn with_catalog(attempt: &str, catalog: serde_json::Value) -> Self {
        let operated = Self::new(attempt, 2, 60_000);
        let path = operated.catalog_path.as_ref().expect("catalog path");
        std::fs::write(path, serde_json::to_vec(&catalog).expect("catalog bytes")).expect("catalog");
        operated
    }

    pub(super) fn with(attempt: &str, options: Options) -> Self {
        let fixture = super::super::Fixture::new();
        let catalog = doubles::catalog(options.check, options.second_check);
        let script = fixture.root.join("fake-omp");
        let mut request = super::super::super::host_local();
        request.envelope.attempt_id = attempt.into();
        request.envelope.work_id = format!("work-{attempt}");
        request.program = script.to_str().expect("script path").to_owned();
        if let Some(expires_in_ms) = options.expires_in_ms {
            request.expires_at_ms = now_ms() + expires_in_ms;
            request.omp.model_broker.expires_at_ms = request.expires_at_ms;
        }
        if let Some((read, write)) = &options.paths {
            for path in write {
                std::fs::create_dir_all(fixture.workspace.join(path)).expect("declared directory");
            }
            request.read_paths = read.iter().map(|path| (*path).to_owned()).collect();
            request.write_paths = write.iter().map(|path| (*path).to_owned()).collect();
        }
        let policy = doubles::policy(&catalog, options.iterations, options.elapsed_ms);
        let request = super::super::readmit(request, &fixture.workspace, Some(policy), None);
        let path = fixture.write(&request);
        let catalog_path = fixture.root.join("catalog.json");
        std::fs::write(&catalog_path, serde_json::to_vec(&catalog).expect("catalog bytes")).expect("catalog");
        let profile_path = fixture.root.join("authority.ncl");
        doubles::write_profile(&profile_path, &request.envelope.worker_profile, options.profile);
        let config_text = format!(
            "{{ data_dir = \"{}\", effect_authority = \"{}\", default_timeout_secs = 60, shell = \"sh\" }}\n",
            fixture.root.join("data").display(),
            profile_path.display(),
        );
        std::fs::write(fixture.root.join("config.ncl"), config_text).expect("config");
        let config = match options.profile {
            Profile::Unconfigured => fixture.root.join("missing-config.ncl"),
            _ => fixture.root.join("config.ncl"),
        };
        Self {
            fixture,
            request,
            path,
            catalog,
            script,
            config,
            profile: profile_path,
            catalog_path: (!matches!(options.profile, Profile::NoCatalog)).then_some(catalog_path),
            launches: Arc::new(AtomicU32::new(0)),
            check_runs: Arc::new(AtomicU32::new(0)),
            after: Arc::new(Mutex::new(None)),
            before_repair: Arc::new(Mutex::new(None)),
            after_check: Arc::new(Mutex::new(None)),
            is_lost: Arc::new(AtomicBool::new(false)),
            indices: Arc::new(Mutex::new(Vec::new())),
            on_launch: Arc::new(Mutex::new(None)),
        }
    }

    /// Write the fake OMP: launch `n` does `behaviours[n - 1]`, then prints
    /// a completed terminal stream; a launch beyond them exits 99. A killed
    /// child is reported as `128 + N`, as bubblewrap reports a signal death
    /// inside the sandbox.
    pub(super) fn script(&self, behaviours: &[Behaviour]) {
        use std::os::unix::fs::PermissionsExt;
        let counter = self.fixture.root.join("omp-count");
        std::fs::write(&counter, b"0\n").expect("counter");
        let counter = counter.display();
        let mut cases = String::new();
        for (index, behaviour) in behaviours.iter().enumerate() {
            let action = match behaviour {
                Behaviour::Write(text) => format!("printf '%s' '{text}' > src/lib.txt"),
                Behaviour::Exit(code) => format!("exit {code}"),
                Behaviour::Killed => "/bin/sh -c 'kill -9 $$'; exit $?".to_owned(),
                Behaviour::Spin => "while :; do :; done".to_owned(),
            };
            cases.push_str(&format!("{}) {action} ;;\n", index + 1));
        }
        let terminal: Vec<String> = doubles::TERMINAL.iter().map(|line| format!("'{line}'")).collect();
        let text = format!(
            "#!/bin/sh\ncount=0\nread -r count < '{counter}'\ncount=$((count + 1))\nprintf '%s\\n' \"$count\" > \
             '{counter}'\ncase \"$count\" in\n{cases}*) exit 99 ;;\nesac\nprintf '%s\\n' {}\n",
            terminal.join(" "),
        );
        std::fs::write(&self.script, text).expect("fake OMP");
        std::fs::set_permissions(&self.script, std::fs::Permissions::from_mode(0o755)).expect("executable");
    }

    pub(super) async fn run(&self) -> Result<(), String> {
        let ports = crate::implementation_attempt::cli::Ports {
            compute: &mut crate::execution_binding::UnavailableComputePort,
            serving: Box::new(crate::execution_binding::serving::UnavailableOwner),
            repair: None,
        };
        let operator = crate::implementation_attempt::cli::repair::operator::Standalone {
            config: &self.config,
            catalog: self.catalog_path.as_deref(),
            launch: Box::new(doubles::Scripted {
                launches: self.launches.clone(),
                after: self.after.clone(),
                before_repair: self.before_repair.clone(),
                is_lost: self.is_lost.clone(),
                indices: self.indices.clone(),
                on_launch: self.on_launch.clone(),
            }),
            execute: Box::new(doubles::Unsandboxed {
                runs: self.check_runs.clone(),
                after: self.after_check.clone(),
            }),
            clock: &crate::operation::shell::SystemClock,
        };
        let revisions = crate::implementation_attempt::cli::session::revision::Store(None);
        let state = &self.fixture.state;
        crate::implementation_attempt::cli::execution::run_with(
            &self.path,
            state,
            Some(ports),
            Some(operator),
            revisions,
        )
        .await
    }

    /// The same run without the operator ports, as an embedder without
    /// repair ports runs it.
    pub(super) async fn run_without_operator(&self) -> Result<(), String> {
        let ports = crate::implementation_attempt::cli::Ports {
            compute: &mut crate::execution_binding::UnavailableComputePort,
            serving: Box::new(crate::execution_binding::serving::UnavailableOwner),
            repair: None,
        };
        let revisions = crate::implementation_attempt::cli::session::revision::Store(None);
        crate::implementation_attempt::cli::execution::run(&self.path, &self.fixture.state, Some(ports), revisions)
            .await
    }

    /// The first forwarded request index of every launch, in order.
    pub(super) fn indices(&self) -> Vec<u32> {
        self.indices.lock().expect("indices").clone()
    }

    /// Run `hook` at every launch with that step's first forwarded index.
    pub(super) fn on_launch(&self, hook: doubles::IndexHook) {
        *self.on_launch.lock().expect("hook") = Some(hook);
    }

    pub(super) fn launches(&self) -> u32 {
        self.launches.load(Ordering::SeqCst)
    }

    pub(super) fn check_runs(&self) -> u32 {
        self.check_runs.load(Ordering::SeqCst)
    }

    pub(super) fn directory(&self) -> std::path::PathBuf {
        self.fixture.directory(&self.request)
    }

    pub(super) fn disposition(&self) -> serde_json::Value {
        self.fixture.disposition(&self.request)
    }

    /// Run `hook` once, after the first launch's OMP step.
    pub(super) fn after_launch(&self, hook: Hook) {
        *self.after.lock().expect("hook") = Some(hook);
    }

    /// Run `hook` once, at the start of the first repair launch, after its
    /// step intent and before the launcher captures the workspace.
    pub(super) fn before_repair(&self, hook: Hook) {
        *self.before_repair.lock().expect("hook") = Some(hook);
    }

    /// Run `hook` once, after the first check.
    pub(super) fn after_check(&self, hook: Hook) {
        *self.after_check.lock().expect("hook") = Some(hook);
    }

    /// Every launch panics after its OMP step, as a lost controller.
    pub(super) fn lose_controller(&self) {
        self.is_lost.store(true, Ordering::SeqCst);
    }

    /// Rewrite the selected operator profile as `profile`.
    pub(super) fn profile_writer(&self, profile: Profile) -> impl FnMut() + Send + 'static {
        let path = self.profile.clone();
        let worker_profile = self.request.envelope.worker_profile.clone();
        move || doubles::write_profile(&path, &worker_profile, profile)
    }

    /// Request cancellation of this attempt once its first step intent exists.
    pub(super) fn cancel_after_intent(&self) -> std::thread::JoinHandle<()> {
        let intent = self.directory().join("repair/step-1.intent");
        let (path, state) = (self.path.clone(), self.fixture.state.clone());
        std::thread::spawn(move || {
            for _ in 0..1_500 {
                if intent.exists() {
                    crate::implementation_attempt::cli::cancellation::request(&path, &state).expect("cancel");
                    return;
                }
                std::thread::sleep(std::time::Duration::from_millis(20));
            }
            panic!("the first step intent never appeared");
        })
    }
}

/// The single-shot request of attempt `attempt` in a fresh fixture, with
/// operator ports that have no configuration or catalog.
pub(super) fn single_shot(attempt: &str) -> Operated {
    let mut operated = Operated::with_profile(attempt, Profile::Unconfigured);
    let request = operated.fixture.request(attempt, None, None);
    operated.path = operated.fixture.write(&request);
    operated.request = request;
    operated.catalog_path = None;
    operated
}

/// The policy that names `catalog`'s executor and checks.
pub(super) fn policy(catalog: &serde_json::Value) -> crate::implementation_attempt::repair::Policy {
    doubles::policy(catalog, 2, 60_000)
}

pub(super) fn json(path: &std::path::Path) -> serde_json::Value {
    serde_json::from_slice(&std::fs::read(path).expect("retained file")).expect("json")
}

/// The sorted paths under `root`, relative to it.
pub(super) fn tree(root: &std::path::Path) -> Vec<std::path::PathBuf> {
    super::super::entries(root)
        .into_iter()
        .map(|path| path.strip_prefix(root).expect("under the root").to_path_buf())
        .collect()
}

pub(super) fn block_on<T>(future: impl Future<Output = T>) -> T {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime")
        .block_on(future)
}

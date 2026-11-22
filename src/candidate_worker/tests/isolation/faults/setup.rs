use super::*;

const SOURCE: &[u8] = b"pub fn answer() -> i32 { 41 }\n";
const RETAINED: &[u8] = b"// independently retained candidate workspace fact\n";
const RUN_TIMEOUT_SECS: u64 = 60;
const PROFILE_LIFETIME_MS: u64 = 90_000;

pub(super) struct Fixture {
    directory: tempfile::TempDir,
    request: Request,
    pub(super) profile: omp::Profile,
}

impl Fixture {
    pub(super) fn new(endpoint: &str) -> Self {
        let directory = tempfile::tempdir().expect("temporary candidate workspace");
        let root = directory.path();
        std::fs::create_dir(root.join("src")).expect("source directory");
        std::fs::write(root.join("src/main.rs"), SOURCE).expect("base source");
        std::fs::write(root.join("src/retained.rs"), RETAINED).expect("retained workspace fact");
        let mut request = request(root, "");
        request.program = std::env::var("LATTICE_OMP").expect("pinned OMP path");
        request.args.clear();
        request.require_isolation = true;
        request.sandbox_program = Some(std::env::var("LATTICE_BWRAP").expect("reviewed bubblewrap path"));
        request.timeout_secs = RUN_TIMEOUT_SECS;
        request.output_limit_bytes = crate::http::MAX_REQUEST_BODY_BYTES;
        let profile = profile(root, endpoint);
        Self {
            directory,
            request,
            profile,
        }
    }

    pub(super) fn run(&self, cancellation: &std::sync::atomic::AtomicBool) -> omp::RunResult {
        let observed = omp::run(
            &self.request,
            &self.profile,
            &crate::process::core::RuntimeConfig::default(),
            cancellation,
            crate::candidate_worker::broker::Services {
                credentials: &crate::credentials::EnvironmentCredentialProvider::from_current_env(&[]),
                gate: &mut crate::candidate_worker::broker::DirectRoute,
                first_request_index: 0,
            },
        )
        .expect("actual selected OMP route");
        assert_eq!(observed.executable_blake3, self.profile.executable_blake3);
        observed
    }

    pub(super) fn assert_workspace_retained(&self) {
        let root = self.directory.path();
        assert_eq!(std::fs::read(root.join("src/main.rs")).expect("retained source"), SOURCE);
        assert_eq!(std::fs::read(root.join("src/retained.rs")).expect("retained independent fact"), RETAINED);
        let files = crate::candidate_capture::workspace(root).expect("independent retained workspace capture");
        assert_eq!(crate::candidate::snapshot_identity(&files).expect("retained identity"), self.profile.base_identity);
    }
}

fn profile(root: &std::path::Path, endpoint: &str) -> omp::Profile {
    let model = std::env::var("LATTICE_OMP_MODEL").expect("exact qualification model");
    omp::Profile {
        revision: std::env::var("LATTICE_OMP_REVISION").expect("reviewed OMP source revision"),
        executable_blake3: std::env::var("LATTICE_OMP_BLAKE3").expect("reviewed OMP bytes"),
        base_identity: crate::candidate::snapshot_identity(
            &crate::candidate_capture::workspace(root).expect("base capture"),
        )
        .expect("base identity"),
        model: format!("lattice/{model}"),
        prompt: "Use the edit tool to replace 41 with 42 in src/main.rs. Do not run commands. Reply done.".into(),
        model_broker: super::super::super::super::broker::Profile {
            schema: super::super::super::super::broker::SCHEMA.into(),
            endpoint: endpoint.into(),
            model,
            credential: None,
            relay: pinned_runtime("LATTICE_SOCAT"),
            shell: pinned_runtime("LATTICE_WORKER_SHELL"),
            sandbox_blake3: std::env::var("LATTICE_BWRAP_BLAKE3").expect("reviewed sandbox bytes"),
            expires_at_ms: super::super::super::super::broker::admission::now_ms().expect("clock")
                + PROFILE_LIFETIME_MS,
            max_requests: 1,
            forwarded: None,
        },
    }
}

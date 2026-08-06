pub(super) const PROFILE_NAME: &str = "quick-check";
pub(super) const OWNER_LABEL: &str = "repo";
pub(super) const WORKSPACE_SOURCE: &str = "workspace";
pub(super) const GLOBAL_SOURCE: &str = "global";
pub(super) const EXPLICIT_SOURCE: &str = "explicit";
pub(super) const APP_MODE_KEY: &str = "APP_MODE";
pub(super) const APP_MODE_VALUE: &str = "ci";
pub(super) const APP_TOKEN_KEY: &str = "APP_TOKEN";
pub(super) const SAFE_CWD: &str = "/repo";
pub(super) const SAFE_TARGET_PATH: &str = "/repo/target";
pub(super) const ESCAPING_PATH: &str = "/tmp/leak";
pub(super) const CARGO_PROGRAM: &str = "cargo";
pub(super) const CHECK_ARG: &str = "check";
pub(super) const TESTS_ARG: &str = "--tests";
pub(super) const CARGO_CHECK_COMMAND: &str = "cargo check --tests";
pub(super) const PURPOSE_KEY: &str = "identity.purpose";
pub(super) const PURPOSE_VALUE: &str = "developer-smoke";
pub(super) const SAFE_POLICY_SOURCE: &str = "test-policy";
pub(super) const TEST_TIMEOUT_SECONDS: u32 = 300;
pub(super) const MAX_TIMEOUT_SECONDS: u32 = 600;
pub(super) const TEST_MEMORY_BYTES: u64 = 134_217_728;
pub(super) const MAX_MEMORY_BYTES: u64 = 268_435_456;
pub(super) const TEST_CPU_MILLIS: u32 = 500;
pub(super) const MAX_CPU_MILLIS: u32 = 1_000;
pub(super) const TEST_LOG_BYTES: u64 = 1_048_576;
pub(super) const MAX_LOG_BYTES: u64 = 2_097_152;
pub(super) const TOO_MANY_LOG_BYTES: u64 = 4_194_304;
pub(super) const FUTURE_SCHEMA_VERSION: u16 = 2;
pub(super) const INVALID_HASH: &str = "raw-policy-source";
pub(super) const FULL_PATH_SOURCE: &str = "/home/example/repo/process-jobs.json";
pub(super) const SECRET_ENV_POLICY_CASES: usize = 3;

pub(super) fn policy() -> super::super::ProcessJobProfilePolicy {
    super::super::ProcessJobProfilePolicy {
        default_backend: super::super::ProcessJobBackend::Native,
        allowed_backends: vec![
            super::super::ProcessJobBackend::Native,
            super::super::ProcessJobBackend::Pueue,
        ],
        allowed_env_prefixes: vec![String::from("APP_")],
        allowed_cwd_prefixes: vec![String::from(SAFE_CWD)],
        allowed_writable_path_prefixes: vec![String::from(SAFE_TARGET_PATH)],
        max_timeout_seconds: Some(MAX_TIMEOUT_SECONDS),
        max_memory_bytes: Some(MAX_MEMORY_BYTES),
        max_cpu_millis: Some(MAX_CPU_MILLIS),
        max_log_bytes: Some(MAX_LOG_BYTES),
        policy_source: String::from(SAFE_POLICY_SOURCE),
        policy_hash: String::new(),
    }
    .with_computed_hash()
}

pub(super) fn valid_profile() -> super::super::ProcessJobProfile {
    super::super::ProcessJobProfile {
        backend: Some(super::super::ProcessJobBackend::Pueue),
        program: Some(String::from(CARGO_PROGRAM)),
        args: vec![String::from(CHECK_ARG), String::from(TESTS_ARG)],
        cwd: Some(String::from(SAFE_CWD)),
        writable_paths: vec![String::from(SAFE_TARGET_PATH)],
        env: vec![super::super::ProcessJobEnvironmentVariable {
            key: String::from(APP_MODE_KEY),
            value: String::from(APP_MODE_VALUE),
        }],
        resources: super::super::ProcessJobResourceLimits {
            timeout_seconds: Some(TEST_TIMEOUT_SECONDS),
            memory_bytes: Some(TEST_MEMORY_BYTES),
            cpu_millis: Some(TEST_CPU_MILLIS),
            log_bytes: Some(TEST_LOG_BYTES),
        },
        notification: super::super::ProcessJobNotificationPolicy {
            notify_on_complete: true,
        },
        metadata: vec![super::super::ProcessJobMetadataField {
            key: String::from(PURPOSE_KEY),
            value: String::from(PURPOSE_VALUE),
        }],
        ..super::super::ProcessJobProfile::default()
    }
}

pub(super) fn command_profile() -> super::super::ProcessJobProfile {
    super::super::ProcessJobProfile {
        command: Some(String::from(CARGO_CHECK_COMMAND)),
        cwd: Some(String::from(SAFE_CWD)),
        writable_paths: vec![String::from(SAFE_TARGET_PATH)],
        notification: super::super::ProcessJobNotificationPolicy {
            notify_on_complete: true,
        },
        ..super::super::ProcessJobProfile::default()
    }
}

pub(super) fn manifest(profile: super::super::ProcessJobProfile) -> super::super::ProcessJobProfileManifest {
    super::super::ProcessJobProfileManifest::current(vec![super::super::NamedProcessJobProfile {
        name: String::from(PROFILE_NAME),
        profile,
    }])
}

pub(super) fn source(
    precedence: super::super::ProcessJobProfileSourcePrecedence,
    safe_label: &str,
    profile: super::super::ProcessJobProfile,
) -> super::super::ProcessJobProfileSource {
    super::super::ProcessJobProfileSource::new(precedence, safe_label, manifest(profile)).expect("safe source")
}

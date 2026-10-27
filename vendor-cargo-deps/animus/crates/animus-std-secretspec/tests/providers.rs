//! Positive and negative provider conformance through real bounded workers.

use animus_adapters::secret_resolution::{
    SecretResolutionOperation, SecretResolutionRequest, SecretResolutionService,
};
use animus_std_secretspec::{
    BoundedSecretSpecResolver, PolicyError, ProviderCredentialSink, ProviderInputs,
    RESOLVER_IDENTITY, ResolutionOutcome, ResolverDeployment, SecretSpecPolicy, SopsCredentialName,
    parse_policy_json,
};
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use tempfile::TempDir;

const ENV_VALUE: &str = "environment-fixture-secret";
const SYSTEMD_VALUE: &str = "systemd-fixture-secret";
const SOPS_VALUE: &str = "fixture-sops-secret";
const SOPS_PROVIDER_CREDENTIAL: &str = "fixture-age-provider-credential";
const REDACTION_SENTINEL: &str = "must-not-appear-in-receipts";
const SHORT_TIMEOUT_MS: &str = "100";
const CONFIGURED_TIMEOUT_MS: &str = "10000";

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root")
}

fn manifest_path() -> PathBuf {
    repo_root().join("fixtures/secret-resolution/secretspec.toml")
}

fn policy_bytes() -> Vec<u8> {
    std::fs::read(repo_root().join("config/generated/secret-resolution-policy.json"))
        .expect("generated policy")
}

fn policy_with_timeout(timeout: &str) -> SecretSpecPolicy {
    let source = String::from_utf8(policy_bytes()).expect("policy UTF-8");
    let changed = source.replace(CONFIGURED_TIMEOUT_MS, timeout);
    parse_policy_json(changed.as_bytes()).expect("checked timeout policy")
}

fn request(policy: &SecretSpecPolicy, profile_name: &str) -> SecretResolutionRequest {
    let profile = policy.profile(profile_name).expect("profile exists");
    SecretResolutionRequest {
        resolver_identity: RESOLVER_IDENTITY.to_string(),
        manifest_identity: profile.manifest_identity().to_string(),
        profile: profile.profile().to_string(),
        scope: profile.scope().to_string(),
        declaration: profile.declaration().to_string(),
        provider: profile.provider(),
        purpose: profile.purpose().to_string(),
        destination: profile.destination(),
        operation: SecretResolutionOperation::Read,
        provider_fallback: false,
        policy_identity: policy.identity().to_string(),
        bounds: profile.bounds(),
    }
}

fn deployment() -> ResolverDeployment {
    ResolverDeployment {
        worker_executable: PathBuf::from(env!("CARGO_BIN_EXE_animus-secretspec-worker")),
        working_directory: repo_root(),
        manifest_path: manifest_path(),
    }
}

fn resolve_detailed(
    policy: SecretSpecPolicy,
    profile: &str,
    inputs: ProviderInputs,
    cancellation: &AtomicBool,
) -> Result<animus_std_secretspec::ResolutionSuccess, Box<animus_std_secretspec::ResolutionFailure>>
{
    let plan = policy
        .admit(request(&policy, profile))
        .expect("explicit request admitted");
    let mut resolver = BoundedSecretSpecResolver::new(policy, deployment(), inputs);
    resolver.resolve_with_cancellation(&plan, cancellation)
}

#[test]
fn environment_provider_delivers_ephemeral_value_and_safe_receipt() {
    let policy = parse_policy_json(&policy_bytes()).expect("checked policy");
    let plan = policy
        .admit(request(&policy, "environment-host"))
        .expect("explicit request admitted");
    let mut resolver = BoundedSecretSpecResolver::new(
        policy,
        deployment(),
        ProviderInputs::environment(ENV_VALUE.to_string()),
    );
    let success = SecretResolutionService::resolve(&mut resolver, &plan).expect("environment read");
    success
        .credential
        .expose_to(|value| assert_eq!(value, ENV_VALUE.as_bytes()));
    assert!(success.receipt.is_safe());
    assert!(!format!("{:?}", success.receipt).contains(ENV_VALUE));
}

#[test]
fn systemd_credential_provider_reads_one_explicit_file() {
    let credentials = TempDir::new().expect("credential directory");
    std::fs::write(credentials.path().join("SYSTEMD_API_KEY"), SYSTEMD_VALUE)
        .expect("credential fixture");
    let success = resolve_detailed(
        parse_policy_json(&policy_bytes()).expect("checked policy"),
        "systemd-host",
        ProviderInputs::systemd_credential(credentials.path().to_path_buf()),
        &AtomicBool::new(false),
    )
    .expect("systemd read");
    success
        .credential
        .expose_to(|value| assert_eq!(value, SYSTEMD_VALUE.as_bytes()));
    assert!(success.receipt.is_safe());
}

#[cfg(unix)]
#[test]
fn sops_provider_runs_through_both_bounded_workers() {
    let fixture = sops_fixture("success");
    let success = resolve_detailed(
        parse_policy_json(&policy_bytes()).expect("checked policy"),
        "sops-host",
        fixture.inputs(),
        &AtomicBool::new(false),
    )
    .expect("SOPS read");
    success
        .credential
        .expose_to(|value| assert_eq!(value, SOPS_VALUE.as_bytes()));
    assert!(success.receipt.is_safe());
}

#[test]
fn policy_rejects_missing_profile_scope_declaration_fallback_write_and_provider() {
    let policy = parse_policy_json(&policy_bytes()).expect("checked policy");
    let base = request(&policy, "environment-host");
    let mut cases = Vec::new();
    let mut missing_profile = base.clone();
    missing_profile.profile.clear();
    cases.push(missing_profile);
    let mut bad_scope = base.clone();
    bad_scope.scope = "other-scope".to_string();
    cases.push(bad_scope);
    let mut bad_declaration = base.clone();
    bad_declaration.declaration = "OTHER_KEY".to_string();
    cases.push(bad_declaration);
    let mut fallback = base.clone();
    fallback.provider_fallback = true;
    cases.push(fallback);
    let mut write = base.clone();
    write.operation = SecretResolutionOperation::Update;
    cases.push(write);
    let mut provider = base;
    provider.provider = animus_adapters::secret_resolution::SecretProviderScheme::Sops;
    cases.push(provider);
    for candidate in cases {
        assert!(policy.admit(candidate).is_err());
    }
}

#[test]
fn unreviewed_ambient_audited_plaintext_and_excessive_policies_fail_closed() {
    let source = String::from_utf8(policy_bytes()).expect("policy UTF-8");
    let cases = [
        (
            source.replace(
                "a8794e46ec9664a0e1a3869cc3105d0853937e48",
                "b8794e46ec9664a0e1a3869cc3105d0853937e48",
            ),
            PolicyError::CohortMismatch,
        ),
        (
            source.replace(
                "\"inherit_environment\": false",
                "\"inherit_environment\": true",
            ),
            PolicyError::InvalidProfile,
        ),
        (
            source.replace("\"enabled\": false", "\"enabled\": true"),
            PolicyError::InvalidProfile,
        ),
        (
            source.replace("\"value_count_max\": 1", "\"value_count_max\": 2"),
            PolicyError::InvalidBounds,
        ),
        (
            source.replace(
                "\"schema_version\": 1,",
                "\"schema_version\": 1, \"plaintext\": \"forbidden\",",
            ),
            PolicyError::InvalidJson,
        ),
    ];
    for (candidate, expected) in cases {
        assert_eq!(
            parse_policy_json(candidate.as_bytes()).err(),
            Some(expected)
        );
    }
}

#[test]
fn missing_manifest_and_provider_input_fail_before_resolution() {
    let policy = parse_policy_json(&policy_bytes()).expect("checked policy");
    let plan = policy
        .admit(request(&policy, "environment-host"))
        .expect("explicit request admitted");
    let mut missing_manifest = deployment();
    missing_manifest.manifest_path = missing_manifest
        .manifest_path
        .with_file_name("absent-secretspec.toml");
    let mut resolver = BoundedSecretSpecResolver::new(
        policy,
        missing_manifest,
        ProviderInputs::environment(ENV_VALUE.to_string()),
    );
    let error = resolver
        .resolve_with_cancellation(&plan, &AtomicBool::new(false))
        .err()
        .expect("missing manifest");
    assert_eq!(error.outcome, ResolutionOutcome::ManifestRejected);

    let policy = parse_policy_json(&policy_bytes()).expect("checked policy");
    let plan = policy
        .admit(request(&policy, "environment-host"))
        .expect("explicit request admitted");
    let mut resolver = BoundedSecretSpecResolver::new(
        policy,
        deployment(),
        ProviderInputs::environment(String::new()),
    );
    let error = resolver
        .resolve_with_cancellation(&plan, &AtomicBool::new(false))
        .err()
        .expect("missing provider credential");
    assert_eq!(error.outcome, ResolutionOutcome::PolicyRejected);
}

#[cfg(unix)]
#[test]
fn sops_missing_nonzero_malformed_timeout_cancel_and_overflow_are_distinct() {
    let cases = [
        ("missing", ResolutionOutcome::ProviderExecutableMissing),
        ("nonzero", ResolutionOutcome::ProviderProcessFailed),
        ("malformed", ResolutionOutcome::ProviderOutputMalformed),
        ("timeout", ResolutionOutcome::TimedOut),
        ("overflow", ResolutionOutcome::OutputLimitExceeded),
    ];
    for (mode, expected) in cases {
        let fixture = sops_fixture(mode);
        let policy = if mode == "timeout" {
            policy_with_timeout(SHORT_TIMEOUT_MS)
        } else {
            parse_policy_json(&policy_bytes()).expect("checked policy")
        };
        let error = resolve_detailed(
            policy,
            "sops-host",
            fixture.inputs(),
            &AtomicBool::new(false),
        )
        .err()
        .expect("negative SOPS fixture");
        assert_eq!(error.outcome, expected, "mode {mode}");
        assert!(error.receipt.is_safe());
    }

    let fixture = sops_fixture("success");
    let error = resolve_detailed(
        parse_policy_json(&policy_bytes()).expect("checked policy"),
        "sops-host",
        fixture.inputs_without_provider_credential(),
        &AtomicBool::new(false),
    )
    .err()
    .expect("missing SOPS provider credential");
    assert_eq!(error.outcome, ResolutionOutcome::ProviderProcessFailed);

    let fixture = sops_fixture("timeout");
    let cancellation = AtomicBool::new(true);
    let error = resolve_detailed(
        parse_policy_json(&policy_bytes()).expect("checked policy"),
        "sops-host",
        fixture.inputs(),
        &cancellation,
    )
    .err()
    .expect("cancelled SOPS fixture");
    assert_eq!(error.outcome, ResolutionOutcome::Cancelled);
}

#[test]
fn value_does_not_enter_receipt_transcript_snapshot_or_derived_hash() {
    let success = resolve_detailed(
        parse_policy_json(&policy_bytes()).expect("checked policy"),
        "environment-host",
        ProviderInputs::environment(REDACTION_SENTINEL.to_string()),
        &AtomicBool::new(false),
    )
    .expect("environment read");
    let receipt = format!("{:?}", success.receipt);
    let value_hash = blake3::hash(REDACTION_SENTINEL.as_bytes())
        .to_hex()
        .to_string();
    assert!(!receipt.contains(REDACTION_SENTINEL));
    assert!(!receipt.contains(&value_hash));
    assert!(!receipt.contains("transcript"));
    assert!(!receipt.contains("snapshot"));
}

#[test]
fn representative_provider_sink_receives_only_a_borrowed_value() {
    struct Sink {
        delivered: bool,
    }
    impl ProviderCredentialSink for Sink {
        type Error = ();

        fn deliver(&mut self, declaration: &str, value: &[u8]) -> Result<(), Self::Error> {
            assert_eq!(declaration, "ENV_API_KEY");
            assert_eq!(value, ENV_VALUE.as_bytes());
            self.delivered = true;
            Ok(())
        }
    }

    let success = resolve_detailed(
        parse_policy_json(&policy_bytes()).expect("checked policy"),
        "environment-host",
        ProviderInputs::environment(ENV_VALUE.to_string()),
        &AtomicBool::new(false),
    )
    .expect("environment read");
    let mut sink = Sink { delivered: false };
    success
        .credential
        .deliver_to_provider(&mut sink)
        .expect("borrowed delivery");
    assert!(sink.delivered);
}

#[cfg(unix)]
struct SopsFixture {
    _directory: TempDir,
    guard_directory: PathBuf,
    executable: PathBuf,
    mode: String,
}

#[cfg(unix)]
impl SopsFixture {
    fn inputs(&self) -> ProviderInputs {
        ProviderInputs::sops(
            self.guard_directory.clone(),
            self.executable.clone(),
            vec![
                (SopsCredentialName::FixtureMode, self.mode.clone()),
                (
                    SopsCredentialName::AgeKey,
                    SOPS_PROVIDER_CREDENTIAL.to_string(),
                ),
            ],
        )
    }

    fn inputs_without_provider_credential(&self) -> ProviderInputs {
        ProviderInputs::sops(
            self.guard_directory.clone(),
            self.executable.clone(),
            vec![(SopsCredentialName::FixtureMode, self.mode.clone())],
        )
    }
}

#[cfg(unix)]
fn sops_fixture(mode: &str) -> SopsFixture {
    use std::os::unix::fs::symlink;

    let directory = TempDir::new().expect("SOPS fixture directory");
    let guard_directory = directory.path().join("guard");
    std::fs::create_dir(&guard_directory).expect("guard directory");
    symlink(
        env!("CARGO_BIN_EXE_animus-secretspec-sops-guard"),
        guard_directory.join("sops"),
    )
    .expect("guard symlink");
    let executable = if mode == "missing" {
        directory.path().join("missing-sops")
    } else {
        PathBuf::from(env!("CARGO_BIN_EXE_animus-secretspec-sops-fixture"))
    };
    SopsFixture {
        _directory: directory,
        guard_directory,
        executable,
        mode: mode.to_string(),
    }
}

//! Paired consumer-visible admission, delivery, and redaction boundaries.
#![cfg(feature = "animus-secretspec")]

use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use animus_std_secretspec::ResolutionOutcome;
use animus_std_secretspec::secret_resolution::SecretResolutionPlan;
use animus_std_secretspec::secret_resolution::SecretResolverAvailability;
use animus_std_secretspec::secret_resolution::SecretResolverCapabilities;
use lattice::credentials::CredentialDestination;
use lattice::credentials::CredentialProvider;
use lattice::credentials::CredentialResolutionContext;
use lattice::credentials::CredentialSubject;
use lattice::credentials::SecretRef;
use lattice::credentials::SecretValue;
use lattice::credentials::config::DeclarationMapping;
use lattice::credentials::config::ProviderScheme;
use lattice::credentials::config::RequestBounds;
use lattice::credentials::config::SecretSpecDiagnostic;
use lattice::credentials::config::SecretSpecProviderConfig;
use lattice::credentials::secretspec::AnimusSecretSpecProvider;
use lattice::credentials::secretspec::SecretSpecHost;
use lattice::credentials::secretspec::adapt_request;
use lattice::credentials::secretspec::map_outcome;

const SENTINEL: &str = "hostile-secret-provider-value-not-public";
const IDENTITY: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

fn config(scheme: ProviderScheme) -> SecretSpecProviderConfig {
    SecretSpecProviderConfig {
        id: "reviewed-secret".into(),
        cohort: "0.17.0".into(),
        manifest_identity: IDENTITY.into(),
        policy_identity: IDENTITY.into(),
        profile: "service".into(),
        scope: "workflow".into(),
        declarations: vec![DeclarationMapping {
            key: "token".into(),
            field: None,
            declaration: "API_TOKEN".into(),
            scheme,
        }],
        allowed_schemes: vec![scheme],
        reason: "workflow-authentication".into(),
        bounds: RequestBounds {
            input_max_bytes: 4096,
            value_max_bytes: 1024,
            stdout_max_bytes: 1056,
            stderr_max_bytes: 1024,
            timeout_ms: 5000,
            poll_interval_ms: 10,
            teardown_timeout_ms: 1000,
        },
    }
}

fn reference() -> SecretRef {
    SecretRef::new("reviewed-secret", "token", None)
}
fn context(destination: CredentialDestination) -> CredentialResolutionContext {
    let subject = if destination == CredentialDestination::WebhookAuthentication {
        CredentialSubject::TriggerAuthentication {
            workflow_id: "workflow-1".into(),
            trigger_id: "trigger-1".into(),
        }
    } else if destination == CredentialDestination::TriggerInvocation {
        CredentialSubject::TriggerInvocation {
            workflow_id: "workflow-1".into(),
            trigger_id: "trigger-1".into(),
        }
    } else {
        CredentialSubject::Node {
            workflow_id: "workflow-1".into(),
            node_id: "node-1".into(),
        }
    };
    CredentialResolutionContext {
        subject,
        binding: "API_TOKEN".into(),
        reference: reference(),
        destination,
        reason: "workflow-authentication".into(),
    }
}

struct FixtureHost {
    calls: AtomicUsize,
    value: String,
    outcome: Option<SecretSpecDiagnostic>,
}
impl FixtureHost {
    fn success(value: &str) -> Self {
        Self {
            calls: AtomicUsize::new(0),
            value: value.into(),
            outcome: None,
        }
    }
}
impl SecretSpecHost for FixtureHost {
    fn capabilities(&self) -> SecretResolverCapabilities {
        SecretResolverCapabilities {
            read: true,
            bounded_worker: true,
            providers: vec![
                animus_std_secretspec::secret_resolution::SecretProviderScheme::Environment,
                animus_std_secretspec::secret_resolution::SecretProviderScheme::SystemdCredential,
                animus_std_secretspec::secret_resolution::SecretProviderScheme::Sops,
            ],
        }
    }
    fn resolve(
        &self,
        plan: &SecretResolutionPlan,
        cancellation: &AtomicBool,
    ) -> Result<SecretValue, SecretSpecDiagnostic> {
        assert_eq!(plan.request().declaration, "API_TOKEN");
        assert!(!plan.request().provider_fallback);
        assert!(!cancellation.load(Ordering::Acquire));
        self.calls.fetch_add(1, Ordering::AcqRel);
        match self.outcome {
            Some(outcome) => Err(outcome),
            None => Ok(SecretValue::new(&self.value)),
        }
    }
}

// r[verify lattice.credentials.animus_secret_providers.resolve.scenario.node]
// r[verify lattice.credentials.animus_secret_providers.resolve.scenario.trigger]
#[test]
fn each_scheme_delivers_only_at_the_current_admitted_node_or_trigger_boundary() {
    for scheme in [
        ProviderScheme::Environment,
        ProviderScheme::SystemdCredential,
        ProviderScheme::Sops,
    ] {
        for destination in [
            CredentialDestination::ShellEnvironment,
            CredentialDestination::HttpHeader,
            CredentialDestination::AgentEnvironment,
            CredentialDestination::WebhookAuthentication,
            CredentialDestination::TriggerInvocation,
        ] {
            let host = Arc::new(FixtureHost::success(SENTINEL));
            let provider =
                AnimusSecretSpecProvider::new(config(scheme), host.clone(), Arc::new(AtomicBool::new(false))).unwrap();
            let value = provider.resolve_in_context(&reference(), &context(destination), None).unwrap();
            assert_eq!(value.expose_for_execution(), SENTINEL);
            assert!(!format!("{value:?}").contains(SENTINEL));
            assert_eq!(host.calls.load(Ordering::Acquire), 1);
            let mut wrong_binding = context(destination);
            wrong_binding.reference.key = "another-declaration".into();
            assert!(provider.resolve_in_context(&reference(), &wrong_binding, None).is_err());
            assert!(provider.resolve(&reference()).is_err());
            assert_eq!(host.calls.load(Ordering::Acquire), 1, "denied bindings must not contact providers");
        }
    }
}

// r[verify lattice.credentials.animus_secret_providers.request.scenario.admitted]
#[test]
fn optional_field_selects_an_explicit_declaration_without_extracting_plaintext() {
    let mut cfg = config(ProviderScheme::Environment);
    cfg.declarations[0].field = Some("password".into());
    let field_ref = SecretRef::new("reviewed-secret", "token", Some("password".into()));
    let mut ctx = context(CredentialDestination::AgentEnvironment);
    ctx.reference = field_ref.clone();
    let host = FixtureHost::success(SENTINEL);
    let plan =
        adapt_request(&field_ref, &cfg, &ctx, &host.capabilities(), SecretResolverAvailability::Available).unwrap();
    assert_eq!(plan.request().declaration, "API_TOKEN");
    assert_eq!(plan.request().manifest_identity, IDENTITY);
    assert_eq!(plan.request().scope, "workflow");
    assert_eq!(plan.request().purpose, "workflow-authentication");
    assert!(
        adapt_request(
            &reference(),
            &cfg,
            &context(CredentialDestination::AgentEnvironment),
            &host.capabilities(),
            SecretResolverAvailability::Available
        )
        .is_err()
    );
}

// r[verify lattice.credentials.animus_secret_providers.request.scenario.rejected]
#[test]
fn missing_capability_context_reason_destination_or_declaration_blocks_before_host_dispatch() {
    let cfg = config(ProviderScheme::Environment);
    let host = Arc::new(FixtureHost::success(SENTINEL));
    let provider = AnimusSecretSpecProvider::new(cfg.clone(), host.clone(), Arc::new(AtomicBool::new(false))).unwrap();
    let base = context(CredentialDestination::ShellEnvironment);
    let mut denied = Vec::new();
    let mut candidate = base.clone();
    candidate.reference.key = "other-declaration".into();
    denied.push(candidate);
    let mut candidate = base.clone();
    candidate.reason.clear();
    denied.push(candidate);
    let mut candidate = base.clone();
    candidate.reason = "other-purpose".into();
    denied.push(candidate);
    let mut candidate = base.clone();
    candidate.binding = "../../unsafe".into();
    denied.push(candidate);
    let mut candidate = base.clone();
    candidate.destination = CredentialDestination::WebhookAuthentication;
    denied.push(candidate);
    let mut candidate = base;
    candidate.subject = CredentialSubject::Node {
        workflow_id: "".into(),
        node_id: "node-1".into(),
    };
    denied.push(candidate);
    for candidate in denied {
        assert!(provider.resolve_in_context(&reference(), &candidate, None).is_err());
    }
    let missing = SecretRef::new("reviewed-secret", "undeclared", None);
    let mut ctx = context(CredentialDestination::HttpHeader);
    ctx.reference = missing.clone();
    assert!(provider.resolve_in_context(&missing, &ctx, None).is_err());
    let mut capabilities = host.capabilities();
    capabilities.bounded_worker = false;
    assert_eq!(
        adapt_request(
            &reference(),
            &cfg,
            &context(CredentialDestination::ShellEnvironment),
            &capabilities,
            SecretResolverAvailability::Available
        ),
        Err(SecretSpecDiagnostic::CapabilityMissing)
    );
    assert!(
        adapt_request(
            &reference(),
            &cfg,
            &context(CredentialDestination::ShellEnvironment),
            &host.capabilities(),
            SecretResolverAvailability::Missing
        )
        .is_err()
    );
    assert_eq!(host.calls.load(Ordering::Acquire), 0);
}

// r[verify lattice.credentials.animus_secret_providers.resolve.scenario.failure]
// r[verify lattice.credentials.animus_secret_providers.redaction.scenario.failure]
#[test]
fn typed_failures_and_rejected_values_never_enter_diagnostics_or_probe_material() {
    let cfg = config(ProviderScheme::Environment);
    for outcome in [
        SecretSpecDiagnostic::Unavailable,
        SecretSpecDiagnostic::ManifestRejected,
        SecretSpecDiagnostic::ProviderDenied,
        SecretSpecDiagnostic::ProviderFailed,
        SecretSpecDiagnostic::TimedOut,
        SecretSpecDiagnostic::Cancelled,
        SecretSpecDiagnostic::CleanupUncertain,
    ] {
        let host = Arc::new(FixtureHost {
            calls: AtomicUsize::new(0),
            value: SENTINEL.into(),
            outcome: Some(outcome),
        });
        let provider =
            AnimusSecretSpecProvider::new(cfg.clone(), host.clone(), Arc::new(AtomicBool::new(false))).unwrap();
        let error = provider
            .resolve_in_context(&reference(), &context(CredentialDestination::ShellEnvironment), None)
            .unwrap_err();
        assert!(error.to_string().contains(outcome.code()));
        assert!(!error.to_string().contains(SENTINEL));
        let calls = host.calls.load(Ordering::Acquire);
        let probe = provider.probe(&reference());
        assert!(!probe.reachable);
        assert!(!serde_json::to_string(&probe).unwrap().contains(SENTINEL));
        assert_eq!(host.calls.load(Ordering::Acquire), calls, "reachability cannot load secret material");
    }
    for rejected in [String::new(), "x".repeat(1025), "unsafe\0value".into()] {
        let provider = AnimusSecretSpecProvider::new(
            cfg.clone(),
            Arc::new(FixtureHost::success(&rejected)),
            Arc::new(AtomicBool::new(false)),
        )
        .unwrap();
        let error = provider
            .resolve_in_context(&reference(), &context(CredentialDestination::ShellEnvironment), None)
            .unwrap_err();
        assert!(error.to_string().contains(SecretSpecDiagnostic::ValueRejected.code()));
    }
    let host = Arc::new(FixtureHost::success(SENTINEL));
    let provider = AnimusSecretSpecProvider::new(cfg, host.clone(), Arc::new(AtomicBool::new(true))).unwrap();
    assert!(
        provider
            .resolve_in_context(&reference(), &context(CredentialDestination::ShellEnvironment), None)
            .unwrap_err()
            .to_string()
            .contains(SecretSpecDiagnostic::Cancelled.code())
    );
    assert_eq!(host.calls.load(Ordering::Acquire), 0);
}

#[test]
fn every_producer_outcome_has_a_fixed_non_success_diagnostic() {
    let pairs = [
        (ResolutionOutcome::PolicyRejected, SecretSpecDiagnostic::ProviderDenied),
        (ResolutionOutcome::ManifestRejected, SecretSpecDiagnostic::ManifestRejected),
        (ResolutionOutcome::WorkerUnavailable, SecretSpecDiagnostic::Unavailable),
        (ResolutionOutcome::ProviderExecutableMissing, SecretSpecDiagnostic::Unavailable),
        (ResolutionOutcome::ProviderProcessFailed, SecretSpecDiagnostic::ProviderFailed),
        (ResolutionOutcome::ProviderOutputMalformed, SecretSpecDiagnostic::ValueRejected),
        (ResolutionOutcome::TimedOut, SecretSpecDiagnostic::TimedOut),
        (ResolutionOutcome::Cancelled, SecretSpecDiagnostic::Cancelled),
        (ResolutionOutcome::OutputLimitExceeded, SecretSpecDiagnostic::ValueRejected),
        (ResolutionOutcome::WorkerProtocolRejected, SecretSpecDiagnostic::ValueRejected),
        (ResolutionOutcome::WorkerInfrastructureFailed, SecretSpecDiagnostic::CleanupUncertain),
    ];
    for (outcome, expected) in pairs {
        assert_eq!(map_outcome(outcome), expected);
    }
}

#[test]
fn registry_validates_declared_fields_without_loading_values_and_rejects_duplicate_ids() {
    use lattice::credentials::CredentialProviderCapabilities;
    use lattice::credentials::CredentialProviderMetadata;
    use lattice::credentials::CredentialProviderRegistry;
    let host = Arc::new(FixtureHost::success(SENTINEL));
    let provider = AnimusSecretSpecProvider::new(
        config(ProviderScheme::Environment),
        host.clone(),
        Arc::new(AtomicBool::new(false)),
    )
    .unwrap();
    let metadata = CredentialProviderMetadata {
        id: "reviewed-secret".into(),
        display_name: "Reviewed SecretSpec".into(),
        capabilities: CredentialProviderCapabilities::read_only_without_fields(),
    };
    let mut registry = CredentialProviderRegistry::new();
    registry.insert_provider(metadata.clone(), Box::new(provider)).unwrap();
    assert!(registry.validate_secret_ref(&reference()).is_ok());
    assert!(registry.validate_secret_ref(&SecretRef::new("reviewed-secret", "undeclared", None)).is_err());
    assert!(registry.validate_secret_ref(&SecretRef::new("missing-provider", "token", None)).is_err());
    assert!(
        registry
            .validate_secret_ref(&SecretRef::new("reviewed-secret", "token", Some("password".into())))
            .is_err()
    );
    let duplicate = AnimusSecretSpecProvider::new(
        config(ProviderScheme::Environment),
        host.clone(),
        Arc::new(AtomicBool::new(false)),
    )
    .unwrap();
    assert!(registry.insert_provider(metadata, Box::new(duplicate)).is_err());
    assert_eq!(host.calls.load(Ordering::Acquire), 0);
    assert!(!registry.probe(&reference()).reachable);
    assert_eq!(host.calls.load(Ordering::Acquire), 0);
}

fn checked_policy(cfg: &SecretSpecProviderConfig) -> Vec<u8> {
    let mapping = &cfg.declarations[0];
    let (provider, selector) = match mapping.scheme {
        ProviderScheme::Environment => ("environment", "env://"),
        ProviderScheme::SystemdCredential => ("systemd_credential", "systemd-credential://"),
        ProviderScheme::Sops => ("sops", "sops://encrypted.json?format=json"),
    };
    serde_json::to_vec(&serde_json::json!({
        "schema_version": 1,
        "resolver_kind": animus_std_secretspec::RESOLVER_IDENTITY,
        "ambient": { "inherit_environment": false, "manifest_discovery": false },
        "audit": { "enabled": false, "authoritative": false },
        "write_operations_allowed": false,
        "non_claims": ["caller-authority", "credential-correctness", "credential-freshness", "provider-integrity", "audit-completeness", "production-readiness", "release-eligibility"],
        "secretspec": { "version": animus_std_secretspec::SECRETSPEC_VERSION,
            "commit": animus_std_secretspec::SECRETSPEC_COMMIT,
            "crate_checksum": animus_std_secretspec::SECRETSPEC_CRATE_CHECKSUM,
            "license": "Apache-2.0", "features": ["sops"] },
        "bounded_exec": { "commit": animus_std_secretspec::BOUNDED_EXEC_COMMIT, "license": "AGPL-3.0-or-later" },
        "profiles": [{ "profile": cfg.profile, "scope": cfg.scope, "declaration": mapping.declaration,
            "manifest_identity": cfg.manifest_identity, "provider": provider, "provider_selector": selector,
            "purpose": cfg.reason, "destination": "provider_dispatch",
            "bounds": { "value_count_max": 1, "input_max_bytes": cfg.bounds.input_max_bytes,
                "value_max_bytes": cfg.bounds.value_max_bytes, "stdout_max_bytes": cfg.bounds.stdout_max_bytes,
                "stderr_max_bytes": cfg.bounds.stderr_max_bytes, "timeout_ms": cfg.bounds.timeout_ms,
                "poll_interval_ms": cfg.bounds.poll_interval_ms, "teardown_timeout_ms": cfg.bounds.teardown_timeout_ms }
        }]
    })).unwrap()
}

// r[verify lattice.credentials.animus_secret_providers.config.scenario.valid]
// r[verify lattice.credentials.animus_secret_providers.config.scenario.invalid]
#[test]
fn trusted_host_composition_is_explicit_identity_bound_and_does_not_probe_inputs() {
    use lattice::credentials::config::HostInputConfig;
    use lattice::credentials::config::SecretSpecHostConfig;
    use lattice::credentials::secretspec::build_provider;
    let root = tempfile::tempdir().unwrap();
    let manifest_path = root.path().join("secretspec.toml");
    let manifest = b"[project]\nname = 'lattice-consumer-fixture'\nrevision = '1.0'\n";
    std::fs::write(&manifest_path, manifest).unwrap();
    let policy_path = root.path().join("policy.json");
    for scheme in [
        ProviderScheme::Environment,
        ProviderScheme::SystemdCredential,
        ProviderScheme::Sops,
    ] {
        let mut cfg = config(scheme);
        cfg.manifest_identity = blake3::hash(manifest).to_hex().to_string();
        let policy = checked_policy(&cfg);
        cfg.policy_identity = blake3::hash(&policy).to_hex().to_string();
        std::fs::write(&policy_path, &policy).unwrap();
        let input = match scheme {
            ProviderScheme::Environment => HostInputConfig::Environment {
                variable: "LATTICE_ABSENT_INPUT_NEVER_PROBED".into(),
            },
            ProviderScheme::SystemdCredential => HostInputConfig::SystemdCredential {
                directory: root.path().join("credentials"),
            },
            ProviderScheme::Sops => HostInputConfig::Sops {
                guard_directory: root.path().join("guard"),
                executable: root.path().join("sops"),
                credential_variables: Vec::new(),
            },
        };
        let host = SecretSpecHostConfig {
            provider_id: cfg.id.clone(),
            policy_path: policy_path.clone(),
            manifest_path: manifest_path.clone(),
            worker_executable: root.path().join("absent-worker"),
            working_directory: root.path().to_path_buf(),
            input,
        };
        assert!(
            build_provider(cfg.clone(), &host, Arc::new(AtomicBool::new(false))).is_ok(),
            "composition must not probe env, service files or executables"
        );
        assert!(!format!("{host:?}").contains(&root.path().display().to_string()));
        assert!(!format!("{:?}", host.input).contains(&root.path().display().to_string()));
        let mut wrong_id = host.clone();
        wrong_id.provider_id = "other-provider".into();
        assert!(build_provider(cfg.clone(), &wrong_id, Arc::new(AtomicBool::new(false))).is_err());
        let mut relative = host.clone();
        relative.manifest_path = "secretspec.toml".into();
        assert!(relative.validate().is_err());
        let mut wrong_policy = cfg.clone();
        wrong_policy.policy_identity = IDENTITY.into();
        assert!(build_provider(wrong_policy, &host, Arc::new(AtomicBool::new(false))).is_err());
        let mut wrong_manifest = cfg.clone();
        wrong_manifest.manifest_identity = IDENTITY.into();
        assert!(build_provider(wrong_manifest, &host, Arc::new(AtomicBool::new(false))).is_err());
        let mut undeclared = cfg;
        undeclared.declarations[0].declaration = "OTHER_TOKEN".into();
        assert!(build_provider(undeclared, &host, Arc::new(AtomicBool::new(false))).is_err());
    }
}

// r[verify lattice.credentials.animus_secret_providers.resolve.scenario.failure]
#[test]
fn actual_host_missing_worker_and_oversized_policy_fail_without_private_paths_or_material() {
    use lattice::credentials::config::HostInputConfig;
    use lattice::credentials::config::SecretSpecHostConfig;
    use lattice::credentials::secretspec::build_provider;
    let root = tempfile::tempdir().unwrap();
    let manifest_path = root.path().join("secretspec.toml");
    std::fs::write(&manifest_path, "[project]\nname='fixture'\n").unwrap();
    let mut cfg = config(ProviderScheme::SystemdCredential);
    cfg.manifest_identity = blake3::hash(&std::fs::read(&manifest_path).unwrap()).to_hex().to_string();
    let policy = checked_policy(&cfg);
    cfg.policy_identity = blake3::hash(&policy).to_hex().to_string();
    let policy_path = root.path().join("policy.json");
    std::fs::write(&policy_path, &policy).unwrap();
    let host = SecretSpecHostConfig {
        provider_id: cfg.id.clone(),
        policy_path: policy_path.clone(),
        manifest_path,
        worker_executable: root.path().join("missing-worker-private-coordinate"),
        working_directory: root.path().to_path_buf(),
        input: HostInputConfig::SystemdCredential {
            directory: root.path().to_path_buf(),
        },
    };
    let provider = build_provider(cfg.clone(), &host, Arc::new(AtomicBool::new(false))).unwrap();
    let failure = provider
        .resolve_in_context(&reference(), &context(CredentialDestination::ShellEnvironment), None)
        .unwrap_err();
    assert!(failure.to_string().contains(SecretSpecDiagnostic::Unavailable.code()));
    assert!(!failure.to_string().contains("missing-worker-private-coordinate"));
    assert!(!failure.to_string().contains(&root.path().display().to_string()));
    std::fs::write(&policy_path, vec![b' '; 1_048_577]).unwrap();
    assert!(build_provider(cfg, &host, Arc::new(AtomicBool::new(false))).is_err());
}

// r[verify lattice.credentials.animus_secret_providers.redaction.scenario.success]
// r[verify lattice.credentials.animus_secret_providers.resolve.scenario.failure]
#[tokio::test]
async fn real_shell_delivery_is_redacted_before_history_storage_and_failures_block_the_effect() {
    use std::collections::HashMap;

    use lattice::credentials::NodeCredentialRefs;
    use lattice::engine::executor::Executor;
    use lattice::engine::executor::NodeStatus;
    use lattice::engine::executor::RunId;
    use lattice::engine::executor::node_execution_cache_key_with_credentials;
    use lattice::engine::workflow::Workflow;
    use lattice::engine::workflow::WorkflowTimestamp;
    use lattice::run::history::detail_record_from_results;
    use lattice::store::Store;
    use lattice::workflow_format::workflow_to_nickel;
    let directory = tempfile::tempdir().unwrap();
    let effect = directory.path().join("effect-observed");
    let mut workflow = Workflow::new("secretspec-redaction", WorkflowTimestamp::epoch());
    let command =
        format!("printf '%s' \"$API_TOKEN\"; printf '%s' \"$API_TOKEN\" >&2; printf 'observed' > {:?}", effect);
    let node_id = workflow.add_shell_node("credential-consumer", &command);
    workflow.set_node_credentials(node_id, NodeCredentialRefs {
        env: [("API_TOKEN".into(), reference())].into(),
    });
    let host = Arc::new(FixtureHost::success(SENTINEL));
    let provider =
        AnimusSecretSpecProvider::new(config(ProviderScheme::Sops), host.clone(), Arc::new(AtomicBool::new(false)))
            .unwrap();
    let executor = Executor::new_with_credential_provider(Box::new(provider));
    let cache_before = node_execution_cache_key_with_credentials(
        node_id,
        workflow.node_config(node_id).unwrap(),
        workflow.node_credentials(node_id),
        &HashMap::new(),
    );
    let result = executor.execute_node(&workflow, node_id, &HashMap::new()).await;
    let NodeStatus::Completed { outputs, .. } = &result.status else {
        panic!("expected credential delivery, got {:?}", result.status);
    };
    assert_eq!(std::fs::read_to_string(&effect).unwrap(), "observed");
    assert_eq!(host.calls.load(Ordering::Acquire), 1);
    for port in &workflow.graph.node(node_id).unwrap().output_ports {
        if matches!(port.label.as_str(), "stdout" | "stderr") {
            assert_eq!(outputs.get(&port.id).unwrap().as_display_string(), "<redacted>");
        }
    }
    let cache_after = node_execution_cache_key_with_credentials(
        node_id,
        workflow.node_config(node_id).unwrap(),
        workflow.node_credentials(node_id),
        &HashMap::new(),
    );
    assert_eq!(cache_before, cache_after);
    let detail = detail_record_from_results(RunId(984), &workflow, std::slice::from_ref(&result));
    let database_path = directory.path().join("redacted.redb");
    {
        let store = Store::new(&database_path).unwrap();
        store.save_workflow(&workflow).unwrap();
        store.save_run_details(&detail).unwrap();
    }
    let reopened = Store::new(&database_path).unwrap();
    let loaded_workflow = reopened.load_workflow("secretspec-redaction").unwrap().unwrap();
    let loaded_detail = reopened.load_run_details(RunId(984)).unwrap().unwrap();
    for surface in [
        serde_json::to_string(&loaded_workflow).unwrap(),
        workflow_to_nickel(&loaded_workflow),
        serde_json::to_string(&loaded_detail).unwrap(),
        format!("{result:?}"),
        format!("{cache_after:?}"),
    ] {
        assert!(!surface.contains(SENTINEL));
        assert!(!surface.contains(&blake3::hash(SENTINEL.as_bytes()).to_hex().to_string()));
    }
    let bytes = std::fs::read(&database_path).unwrap();
    assert!(!bytes.windows(SENTINEL.len()).any(|window| window == SENTINEL.as_bytes()));
    std::fs::remove_file(&effect).unwrap();
    let denied_host = Arc::new(FixtureHost {
        calls: AtomicUsize::new(0),
        value: SENTINEL.into(),
        outcome: Some(SecretSpecDiagnostic::TimedOut),
    });
    let denied_provider =
        AnimusSecretSpecProvider::new(config(ProviderScheme::Sops), denied_host, Arc::new(AtomicBool::new(false)))
            .unwrap();
    let denied = Executor::new_with_credential_provider(Box::new(denied_provider))
        .execute_node(&workflow, node_id, &HashMap::new())
        .await;
    assert!(matches!(denied.status, NodeStatus::Failed { .. }));
    assert!(!effect.exists(), "credential failure must block the dependent shell");
    assert!(!format!("{denied:?}").contains(SENTINEL));
}

#[test]
fn cancellation_observed_after_host_return_discards_material_before_delivery() {
    struct CancelOnReturnHost(FixtureHost);
    impl SecretSpecHost for CancelOnReturnHost {
        fn capabilities(&self) -> SecretResolverCapabilities {
            self.0.capabilities()
        }
        fn resolve(
            &self,
            plan: &SecretResolutionPlan,
            cancellation: &AtomicBool,
        ) -> Result<SecretValue, SecretSpecDiagnostic> {
            let result = self.0.resolve(plan, cancellation);
            cancellation.store(true, Ordering::Release);
            result
        }
    }
    let host = Arc::new(CancelOnReturnHost(FixtureHost::success(SENTINEL)));
    let provider = AnimusSecretSpecProvider::new(
        config(ProviderScheme::Environment),
        host.clone(),
        Arc::new(AtomicBool::new(false)),
    )
    .unwrap();
    let failure = provider
        .resolve_in_context(&reference(), &context(CredentialDestination::AgentEnvironment), None)
        .unwrap_err();
    assert_eq!(host.0.calls.load(Ordering::Acquire), 1);
    assert!(failure.to_string().contains(SecretSpecDiagnostic::Cancelled.code()));
    assert!(!format!("{failure:?}").contains(SENTINEL));
}

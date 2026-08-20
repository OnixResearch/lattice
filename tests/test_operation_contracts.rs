// r[verify lattice.operations.catalog.valid]
// r[verify lattice.operations.catalog.invalid]
// r[verify lattice.operations.instances.valid]
// r[verify lattice.operations.instances.override_denied]
// r[verify lattice.operations.preflight.admitted]
// r[verify lattice.operations.preflight.denied]
// r[verify lattice.operations.direct_run.success]
// r[verify lattice.operations.direct_run.output_rejected]
// r[verify lattice.operations.promotion.preview]
// r[verify lattice.operations.promotion.denied]
// r[verify lattice.operations.pilot.valid]
// r[verify lattice.operations.pilot.denied]

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::sync::atomic::AtomicI64;
use std::sync::atomic::Ordering;

use lattice::engine::effect::execution::RunContextProvider as _;
use lattice::operation;
use lattice::operation::shell::Clock as _;

const TEST_TIMEOUT_SECONDS: u64 = 1;
const TEST_INPUT_BYTES: usize = 4_096;
const TEST_OUTPUT_BYTES: usize = 16_384;
const TEST_ARTIFACT_BYTES: usize = 1_024;
const TEST_FIELD_BYTES: usize = 128;
const TEST_START_EPOCH_MILLIS: i64 = 10_000;
const TEST_CLOCK_INCREMENT_MILLIS: i64 = 10;
const TEST_WORKFLOW_RUN_ID: u64 = 1;
const TEST_NONZERO_EXIT_CODE: u8 = 7;
const TEST_EXECUTABLE_MODE: u32 = 0o700;
const TEST_NON_EXECUTABLE_MODE: u32 = 0o600;
const EXPECTED_HOST_TOOL_COUNT: usize = 2;
const TEST_OVERSIZED_OUTPUT_BYTES: usize = TEST_OUTPUT_BYTES + 1;

struct Fixture {
    _root: tempfile::TempDir,
    repository_root: std::path::PathBuf,
    executable: std::path::PathBuf,
    catalog: operation::ValidatedCatalog,
    instance: operation::ValidatedInstance,
    plan: operation::InvocationPlan,
}

fn bounds() -> operation::Bounds {
    operation::Bounds {
        timeout_seconds: TEST_TIMEOUT_SECONDS,
        input_bytes: TEST_INPUT_BYTES,
        output_bytes: TEST_OUTPUT_BYTES,
        artifact_bytes: TEST_ARTIFACT_BYTES,
    }
}

fn empty_contract() -> operation::model::RecordContract {
    operation::model::RecordContract {
        fields: Vec::new(),
        deny_unknown_fields: true,
    }
}

fn output_contract() -> operation::model::RecordContract {
    operation::model::RecordContract {
        fields: vec![operation::model::FieldContract {
            name: "valid".to_string(),
            value_type: operation::model::ValueType::Boolean,
            required: true,
            max_bytes: TEST_FIELD_BYTES,
        }],
        deny_unknown_fields: false,
    }
}

fn method(name: &str) -> operation::model::Method {
    operation::model::Method {
        name: name.to_string(),
        description: format!("Read-only Cairn {name} operation"),
        arguments: empty_contract(),
        output: output_contract(),
        preflight: vec![
            operation::model::PreflightCheck::RuntimeAvailable,
            operation::model::PreflightCheck::ExecutableAvailable,
            operation::model::PreflightCheck::RepositoryRootAdmitted,
            operation::model::PreflightCheck::CredentialReferencesValid,
        ],
        capabilities: vec![operation::CapabilityClass::ProcessRead],
        side_effect: operation::model::SideEffectClass::ReadOnly,
    }
}

fn catalog(executor_identity: &str) -> operation::model::Catalog {
    let mut operation_type = operation::OperationType {
        id: "cairn.repository".to_string(),
        schema_version: operation::model::TYPE_SCHEMA_VERSION,
        revision: operation::model::BLAKE3_PREFIX.to_string() + &"0".repeat(operation::model::BLAKE3_HEX_LENGTH),
        description: "Read-only Cairn repository inspection".to_string(),
        tags: vec!["cairn".to_string(), "read-only".to_string()],
        methods: vec![
            method(operation::runtime::CAIRN_CHANGE_LIST_METHOD),
            method(operation::runtime::CAIRN_VALIDATE_METHOD),
        ],
        runtime: operation::model::RuntimeRequirement {
            kind: operation::runtime::CAIRN_RUNTIME_KIND.to_string(),
            key: operation::runtime::CAIRN_RUNTIME_KEY.to_string(),
            executor_identity: executor_identity.to_string(),
        },
        execution_requirements: operation::ExecutionRequirements::cairn_read_only(),
        capabilities: vec![operation::CapabilityClass::ProcessRead],
        credential_bindings: Vec::new(),
        bounds: bounds(),
        required_non_claims: operation::model::REQUIRED_NON_CLAIMS.iter().map(|claim| (*claim).to_string()).collect(),
    };
    operation_type.revision = operation::identity::type_revision(&operation_type).expect("type revision");
    operation::Catalog {
        schema: operation::model::CATALOG_SCHEMA.to_string(),
        types: vec![operation_type],
    }
}

fn make_script_executable(root: &std::path::Path, script: &str) -> std::path::PathBuf {
    use std::os::unix::fs::PermissionsExt as _;

    let path = root.join("cairn-read-only-test-driver");
    std::fs::write(&path, script).expect("write test driver");
    let mut permissions = std::fs::metadata(&path).expect("driver metadata").permissions();
    permissions.set_mode(TEST_EXECUTABLE_MODE);
    std::fs::set_permissions(&path, permissions).expect("make test driver executable");
    path
}

fn fixture(output: &str) -> Fixture {
    assert!(!output.contains('\''), "test driver output must not contain a single quote");
    fixture_with_script(&format!("#!/bin/sh\nprintf '%s\\n' '{}'\n", output))
}

fn fixture_with_script(script: &str) -> Fixture {
    let root = tempfile::tempdir().expect("temporary fixture root");
    let executable = make_script_executable(root.path(), script);
    let repository_root = root.path().join("repository");
    std::fs::create_dir(&repository_root).expect("repository root");
    let executor_identity = operation::identity::executor_file(&executable).expect("executor identity");
    let catalog = operation::catalog::validate(catalog(&executor_identity)).expect("valid catalog");
    let definition = operation::OperationInstance {
        id: "local-repository".to_string(),
        display_alias: "Local repository".to_string(),
        type_id: "cairn.repository".to_string(),
        type_revision: catalog.types["cairn.repository"].definition.revision.clone(),
        target: BTreeMap::from([(
            operation::runtime::REPOSITORY_ROOT_TARGET.to_string(),
            serde_json::Value::String(repository_root.to_string_lossy().into_owned()),
        )]),
        tags: BTreeMap::from([("environment".to_string(), "test".to_string())]),
        method_defaults: BTreeMap::new(),
        credentials: BTreeMap::new(),
    };
    let instance = operation::instance::validate(
        operation::InstanceFile {
            schema: operation::model::INSTANCE_SCHEMA.to_string(),
            instances: vec![definition],
        },
        &catalog,
    )
    .expect("valid instance")
    .remove("local-repository")
    .expect("fixture instance");
    let plan = plan(&catalog, &instance, &executable);
    Fixture {
        _root: root,
        repository_root,
        executable,
        catalog,
        instance,
        plan,
    }
}

fn plan(
    catalog: &operation::ValidatedCatalog,
    instance: &operation::ValidatedInstance,
    executable: &std::path::Path,
) -> operation::InvocationPlan {
    let operation_type = &catalog.types["cairn.repository"].definition;
    let expected_catalog_identity = catalog.identity.clone();
    let expected_instance_identity = instance.identity.clone();
    operation::planning::plan(operation::PlanRequest {
        catalog,
        instance,
        method: operation::runtime::CAIRN_CHANGE_LIST_METHOD,
        arguments: BTreeMap::new(),
        expected_catalog_identity: &expected_catalog_identity,
        expected_instance_identity: &expected_instance_identity,
        runtime: operation::RuntimeFact {
            kind: operation_type.runtime.kind.clone(),
            key: operation_type.runtime.key.clone(),
            executor_identity: operation_type.runtime.executor_identity.clone(),
            executable_path: executable.to_string_lossy().into_owned(),
            available: true,
            maximum_bounds: operation_type.bounds,
        },
        policy: operation::CapabilityPolicy {
            allowed: vec![operation::CapabilityClass::ProcessRead],
            allow_mutation: false,
        },
        preflight: operation::PreflightFacts {
            executable_available: true,
            repository_root_admitted: true,
            credential_references_valid: true,
        },
    })
    .expect("valid plan")
}

#[derive(Debug)]
struct TestClock {
    next: AtomicI64,
}

impl TestClock {
    fn new() -> Self {
        Self {
            next: AtomicI64::new(TEST_START_EPOCH_MILLIS),
        }
    }
}

impl operation::shell::Clock for TestClock {
    fn observe_epoch_millis(&self) -> i64 {
        self.next.fetch_add(TEST_CLOCK_INCREMENT_MILLIS, Ordering::SeqCst)
    }
}

fn execute_once(fixture: &Fixture, cancellation: Option<&std::sync::atomic::AtomicBool>) -> operation::DirectRunResult {
    let plan = &fixture.plan;
    let store_root = tempfile::tempdir().expect("operation store root");
    let store = lattice::store::Store::new(store_root.path().join("lattice.redb")).expect("operation store");
    let authority = operation::shell::LocalReadOnlyAuthorityProvider::new(&fixture.catalog, &fixture.instance, plan)
        .expect("local operation authority");
    operation::shell::run_direct(operation::shell::DirectRunRequest {
        store: &store,
        plan,
        authority_provider: &authority,
        process: &lattice::process::core::RuntimeConfig::default(),
        credentials: &lattice::credentials::EmptyCredentialProvider,
        clock: &TestClock::new(),
        cancellation,
    })
    .expect("durable operation result")
}

#[test]
fn catalog_instance_and_plan_are_identity_bound_and_deterministic() {
    let fixture = fixture(r#"{"valid":true,"changes":[]}"#);
    let repeated = plan(&fixture.catalog, &fixture.instance, &fixture.executable);

    assert_eq!(fixture.plan, repeated);
    assert!(operation::identity::validate(&fixture.plan.identity));
    assert_eq!(fixture.plan.type_revision, fixture.instance.definition.type_revision);
    assert_eq!(
        fixture.plan.required_non_claims,
        operation::model::REQUIRED_NON_CLAIMS.iter().map(|claim| (*claim).to_string()).collect::<Vec<_>>()
    );

    let mut display_only = fixture.instance.definition.clone();
    display_only.display_alias = "Changed display alias".to_string();
    display_only.tags.insert("display".to_string(), "changed".to_string());
    assert_eq!(
        operation::identity::instance(&display_only).expect("display-only instance identity"),
        fixture.instance.identity
    );

    let original_type = &fixture.catalog.types["cairn.repository"].definition;
    let original_method_identity =
        operation::identity::method(&original_type.revision, &original_type.methods[0]).expect("method identity");
    let mut description_only = original_type.clone();
    description_only.description = "Changed type description".to_string();
    description_only.tags = vec!["display-only".to_string()];
    description_only.methods[0].description = "Changed method description".to_string();
    assert_eq!(
        operation::identity::type_revision(&description_only).expect("description-only type revision"),
        original_type.revision
    );
    assert_eq!(
        operation::identity::method(&original_type.revision, &description_only.methods[0])
            .expect("description-only method identity"),
        original_method_identity
    );
}

#[test]
fn catalog_rejects_stale_revision_and_instance_rejects_behavior_or_secret_targets() {
    let fixture = fixture(r#"{"valid":true}"#);
    let mut stale = catalog(&fixture.catalog.types["cairn.repository"].definition.runtime.executor_identity);
    stale.types[0].bounds.output_bytes = stale.types[0].bounds.output_bytes.saturating_sub(1);
    assert!(operation::catalog::validate(stale).is_err());

    let mut duplicate = catalog(&fixture.catalog.types["cairn.repository"].definition.runtime.executor_identity);
    duplicate.types.push(duplicate.types[0].clone());
    assert!(operation::catalog::validate(duplicate).is_err());

    let mut unsupported_schema =
        catalog(&fixture.catalog.types["cairn.repository"].definition.runtime.executor_identity);
    unsupported_schema.types[0].schema_version = operation::model::TYPE_SCHEMA_VERSION.saturating_add(1);
    unsupported_schema.types[0].revision =
        operation::identity::type_revision(&unsupported_schema.types[0]).expect("unsupported schema revision");
    assert!(operation::catalog::validate(unsupported_schema).is_err());

    let mut unsupported_runtime =
        catalog(&fixture.catalog.types["cairn.repository"].definition.runtime.executor_identity);
    unsupported_runtime.types[0].runtime.key = "unknown-runtime".to_string();
    unsupported_runtime.types[0].revision =
        operation::identity::type_revision(&unsupported_runtime.types[0]).expect("unsupported runtime revision");
    assert!(operation::catalog::validate(unsupported_runtime).is_err());

    let mut secret_schema = catalog(&fixture.catalog.types["cairn.repository"].definition.runtime.executor_identity);
    secret_schema.types[0].methods[0].arguments.fields.push(operation::model::FieldContract {
        name: "api_key".to_string(),
        value_type: operation::model::ValueType::String,
        required: false,
        max_bytes: TEST_FIELD_BYTES,
    });
    secret_schema.types[0].revision =
        operation::identity::type_revision(&secret_schema.types[0]).expect("secret schema revision");
    assert!(operation::catalog::validate(secret_schema).is_err());

    let mut unsafe_instance = fixture.instance.definition.clone();
    unsafe_instance.target.insert("command".to_string(), serde_json::json!("rm -rf"));
    unsafe_instance.target.insert("api_token".to_string(), serde_json::json!("secret"));
    unsafe_instance.tags.insert("access_token".to_string(), "secret".to_string());
    unsafe_instance
        .credentials
        .insert("undeclared".to_string(), lattice::credentials::SecretRef::new("", "", None));
    assert!(
        operation::instance::validate(
            operation::InstanceFile {
                schema: operation::model::INSTANCE_SCHEMA.to_string(),
                instances: vec![unsafe_instance],
            },
            &fixture.catalog,
        )
        .is_err()
    );

    let oversized_value = serde_json::Value::Object(
        (0..=operation::model::MAXIMUM_SCHEMA_FIELDS)
            .map(|index| (format!("field_{index}"), serde_json::Value::Bool(true)))
            .collect(),
    );
    assert!(
        !operation::schema::validate_value(
            &operation::model::RecordContract {
                fields: Vec::new(),
                deny_unknown_fields: false,
            },
            &oversized_value,
            "oversized",
            operation::model::DiagnosticCode::InvalidValue,
        )
        .is_empty()
    );

    let mut oversized_instance = fixture.instance.definition.clone();
    for index in 0..operation::model::MAXIMUM_TARGET_FACTS {
        oversized_instance.target.insert(format!("fact_{index}"), serde_json::Value::Bool(true));
    }
    assert!(
        operation::instance::validate(
            operation::InstanceFile {
                schema: operation::model::INSTANCE_SCHEMA.to_string(),
                instances: vec![oversized_instance],
            },
            &fixture.catalog,
        )
        .is_err()
    );
}

#[test]
fn planning_rejects_runtime_capability_and_preflight_drift() {
    let fixture = fixture(r#"{"valid":true}"#);
    let expected_catalog_identity = fixture.catalog.identity.clone();
    let expected_instance_identity = fixture.instance.identity.clone();
    let result = operation::planning::plan(operation::PlanRequest {
        catalog: &fixture.catalog,
        instance: &fixture.instance,
        method: operation::runtime::CAIRN_CHANGE_LIST_METHOD,
        arguments: BTreeMap::new(),
        expected_catalog_identity: &expected_catalog_identity,
        expected_instance_identity: &expected_instance_identity,
        runtime: operation::RuntimeFact {
            kind: operation::runtime::CAIRN_RUNTIME_KIND.to_string(),
            key: "wrong-runtime".to_string(),
            executor_identity: fixture.catalog.types["cairn.repository"].definition.runtime.executor_identity.clone(),
            executable_path: fixture.executable.to_string_lossy().into_owned(),
            available: true,
            maximum_bounds: bounds(),
        },
        policy: operation::CapabilityPolicy {
            allowed: Vec::new(),
            allow_mutation: false,
        },
        preflight: operation::PreflightFacts {
            executable_available: true,
            repository_root_admitted: false,
            credential_references_valid: true,
        },
    });

    assert!(result.is_err());
}

#[test]
fn plan_identity_rejects_field_tampering() {
    let fixture = fixture(r#"{"valid":true}"#);
    let mut tampered = fixture.plan.clone();
    tampered.method.push_str("-tampered");

    assert!(operation::planning::validate_plan_identity(&tampered).is_err());

    let mut backend_drift = fixture.plan.clone();
    backend_drift.backend_route.backend_identity =
        operation::identity::hash_bytes("test.backend-drift", b"another-backend");
    backend_drift.identity = operation::identity::invocation(&backend_drift).expect("backend-drift identity");
    assert!(operation::planning::validate_plan_identity(&backend_drift).is_err());

    let mut self_consistent_drift = fixture.plan.clone();
    self_consistent_drift.output_contract.deny_unknown_fields =
        !self_consistent_drift.output_contract.deny_unknown_fields;
    self_consistent_drift.output_contract_identity =
        operation::identity::schema(&self_consistent_drift.output_contract).expect("drifted output contract identity");
    self_consistent_drift.identity =
        operation::identity::invocation(&self_consistent_drift).expect("self-consistent invocation identity");
    assert!(operation::planning::validate_plan_identity(&self_consistent_drift).is_ok());
    assert!(
        operation::planning::validate_plan_admission(&self_consistent_drift, &fixture.catalog, &fixture.instance,)
            .is_err()
    );
}

// r[verify lattice.execution_backend_orchestration.planner_mapping]
// r[verify lattice.execution_backend_orchestration.verification]
#[test]
fn animus_backend_mapping_is_exact_for_every_reviewed_class() {
    let requirements = operation::ExecutionRequirements::cairn_read_only();
    let classes = [
        operation::BackendClass::EmbeddedComponent,
        operation::BackendClass::GuardedLocalProcess,
        operation::BackendClass::IsolatedHost,
        operation::BackendClass::ExternalMachine,
    ];

    for class in classes {
        assert_eq!(
            operation::backend::compatibility_projection(class, &requirements).expect("reviewed planner class maps"),
            class
        );
    }
}

// r[verify lattice.execution_backend_orchestration.dispatch]
// r[verify lattice.execution_backend_orchestration.verification]
#[test]
fn backend_retry_denies_substitution_and_unknown_outcomes() {
    let fixture = fixture(r#"{"valid":true}"#);
    let attempt = operation::BackendAttemptRecord::admitted(
        &fixture.plan,
        lattice::engine::executor::RunId(TEST_WORKFLOW_RUN_ID),
    );
    let mut changed = fixture.plan.backend_route.clone();
    changed.backend_identity = operation::identity::hash_bytes("test.backend-substitution", b"different");

    assert!(operation::backend::admit_retry_route(&attempt, &changed, false).is_err());
    assert!(operation::backend::admit_retry_route(&attempt, &fixture.plan.backend_route, true).is_err());
    assert!(operation::backend::admit_retry_route(&attempt, &fixture.plan.backend_route, false).is_ok());
}

#[test]
fn direct_run_persists_started_then_terminal_metadata_and_rejects_duplicate_dispatch() {
    let fixture = fixture(r#"{"valid":true,"changes":[]}"#);
    let store_root = tempfile::tempdir().expect("store root");
    let store = lattice::store::Store::new(store_root.path().join("lattice.redb")).expect("store");
    let authority =
        operation::shell::LocalReadOnlyAuthorityProvider::new(&fixture.catalog, &fixture.instance, &fixture.plan)
            .expect("direct operation authority");
    let credentials = lattice::credentials::EmptyCredentialProvider;
    let process = lattice::process::core::RuntimeConfig::default();
    let result = operation::shell::run_direct(operation::shell::DirectRunRequest {
        store: &store,
        plan: &fixture.plan,
        authority_provider: &authority,
        process: &process,
        credentials: &credentials,
        clock: &TestClock::new(),
        cancellation: None,
    })
    .expect("direct operation run");

    assert_eq!(result.record.state, operation::RunState::Completed, "{}", result.record.diagnostic);
    assert_eq!(result.output, Some(serde_json::json!({"valid": true, "changes": []})));
    let durable = store
        .load_operation_run(&fixture.plan.identity)
        .expect("load operation record")
        .expect("durable operation record");
    assert_eq!(durable, result.record);
    assert_eq!(durable.backend_attempt.state, operation::BackendAttemptState::Observed);
    assert_eq!(durable.backend_attempt.route.planner_cohort, operation::backend::PLANNER_COHORT);
    let safe_backend_json = serde_json::to_string(&durable.backend_attempt.safe_view()).expect("safe backend view");
    assert!(!safe_backend_json.contains(fixture.executable.to_string_lossy().as_ref()));
    assert!(!safe_backend_json.contains("command"));
    assert!(!safe_backend_json.contains("credential"));
    assert!(store.load_run(result.record.run_id).expect("standard run").is_some());
    assert!(
        store
            .load_run_details(result.record.run_id)
            .expect("run details")
            .is_some_and(|details| details.nodes[0].outputs.is_empty())
    );
    assert!(
        operation::shell::run_direct(operation::shell::DirectRunRequest {
            store: &store,
            plan: &fixture.plan,
            authority_provider: &authority,
            process: &process,
            credentials: &credentials,
            clock: &TestClock::new(),
            cancellation: None,
        })
        .is_err()
    );
}

#[test]
fn missing_executor_after_planning_persists_unknown_without_retry() {
    let fixture = fixture(r#"{"valid":true}"#);
    let authority =
        operation::shell::LocalReadOnlyAuthorityProvider::new(&fixture.catalog, &fixture.instance, &fixture.plan)
            .expect("missing executor authority");
    std::fs::remove_file(&fixture.executable).expect("remove executor after planning");
    let store_root = tempfile::tempdir().expect("store root");
    let store = lattice::store::Store::new(store_root.path().join("lattice.redb")).expect("store");
    let result = operation::shell::run_direct(operation::shell::DirectRunRequest {
        store: &store,
        plan: &fixture.plan,
        authority_provider: &authority,
        process: &lattice::process::core::RuntimeConfig::default(),
        credentials: &lattice::credentials::EmptyCredentialProvider,
        clock: &TestClock::new(),
        cancellation: None,
    })
    .expect("unknown result remains durable");

    assert_eq!(result.record.state, operation::RunState::Unknown);
    assert_eq!(result.record.backend_attempt.state, operation::BackendAttemptState::Unknown);
    assert!(result.output.is_none());
    assert!(
        operation::shell::run_direct(operation::shell::DirectRunRequest {
            store: &store,
            plan: &fixture.plan,
            authority_provider: &authority,
            process: &lattice::process::core::RuntimeConfig::default(),
            credentials: &lattice::credentials::EmptyCredentialProvider,
            clock: &TestClock::new(),
            cancellation: None,
        })
        .is_err()
    );
}

#[test]
fn malformed_typed_output_fails_closed_without_persisting_content() {
    let fixture = fixture(r#"{"changes":[]}"#);
    let store_root = tempfile::tempdir().expect("store root");
    let store = lattice::store::Store::new(store_root.path().join("lattice.redb")).expect("store");
    let authority =
        operation::shell::LocalReadOnlyAuthorityProvider::new(&fixture.catalog, &fixture.instance, &fixture.plan)
            .expect("malformed output authority");
    let result = operation::shell::run_direct(operation::shell::DirectRunRequest {
        store: &store,
        plan: &fixture.plan,
        authority_provider: &authority,
        process: &lattice::process::core::RuntimeConfig::default(),
        credentials: &lattice::credentials::EmptyCredentialProvider,
        clock: &TestClock::new(),
        cancellation: None,
    })
    .expect("deterministic output rejection");

    assert_eq!(result.record.state, operation::RunState::Failed);
    assert!(result.record.output_identity.is_none());
    assert!(result.output.is_none());
    assert!(!serde_json::to_string(&result.record).expect("record JSON").contains("changes"));
}

#[test]
fn cairn_pilot_denies_unsafe_admission_and_bounded_process_failures() {
    use std::os::unix::fs::PermissionsExt as _;

    let admission_fixture = fixture(r#"{"valid":true}"#);
    let operation_type = &admission_fixture.catalog.types["cairn.repository"].definition;
    let expected_catalog_identity = admission_fixture.catalog.identity.clone();
    let expected_instance_identity = admission_fixture.instance.identity.clone();
    let denied_plan = |method: &str, arguments: BTreeMap<String, serde_json::Value>| {
        operation::planning::plan(operation::PlanRequest {
            catalog: &admission_fixture.catalog,
            instance: &admission_fixture.instance,
            method,
            arguments,
            expected_catalog_identity: &expected_catalog_identity,
            expected_instance_identity: &expected_instance_identity,
            runtime: operation::RuntimeFact {
                kind: operation_type.runtime.kind.clone(),
                key: operation_type.runtime.key.clone(),
                executor_identity: operation_type.runtime.executor_identity.clone(),
                executable_path: admission_fixture.executable.to_string_lossy().into_owned(),
                available: true,
                maximum_bounds: operation_type.bounds,
            },
            policy: operation::CapabilityPolicy {
                allowed: vec![operation::CapabilityClass::ProcessRead],
                allow_mutation: false,
            },
            preflight: operation::PreflightFacts {
                executable_available: true,
                repository_root_admitted: true,
                credential_references_valid: true,
            },
        })
    };
    assert!(denied_plan("delete", BTreeMap::new()).is_err());
    assert!(
        denied_plan(
            operation::runtime::CAIRN_CHANGE_LIST_METHOD,
            BTreeMap::from([("extra_argument".to_string(), serde_json::Value::String("unsafe".to_string()))]),
        )
        .is_err()
    );

    let unsafe_root = fixture(r#"{"valid":true}"#);
    std::fs::remove_dir(&unsafe_root.repository_root).expect("remove repository root");
    std::os::unix::fs::symlink(unsafe_root._root.path(), &unsafe_root.repository_root).expect("unsafe root symlink");
    let unsafe_result = execute_once(&unsafe_root, None);
    assert_eq!(unsafe_result.record.state, operation::RunState::Failed);
    assert!(unsafe_result.record.diagnostic.contains("repository root is unsafe"));

    let unsafe_executor = fixture(r#"{"valid":true}"#);
    let mut permissions =
        std::fs::metadata(&unsafe_executor.executable).expect("unsafe executor metadata").permissions();
    permissions.set_mode(TEST_NON_EXECUTABLE_MODE);
    std::fs::set_permissions(&unsafe_executor.executable, permissions).expect("remove executor permission");
    assert!(operation::identity::executor_file(&unsafe_executor.executable).is_err());
    let executor_symlink = unsafe_executor._root.path().join("executor-symlink");
    std::os::unix::fs::symlink(&unsafe_executor.executable, &executor_symlink).expect("executor symlink");
    assert!(operation::identity::executor_file(&executor_symlink).is_err());

    let changed_executor = fixture(r#"{"valid":true}"#);
    let marker = changed_executor._root.path().join("unexpected-execution");
    assert!(!marker.to_string_lossy().contains('\''));
    std::fs::write(
        &changed_executor.executable,
        format!("#!/bin/sh\nprintf '%s' 'executed' > '{}'\nprintf '%s\\n' '{{\"valid\":true}}'\n", marker.display()),
    )
    .expect("replace executor after planning");
    let changed_result = execute_once(&changed_executor, None);
    assert_eq!(changed_result.record.state, operation::RunState::Failed);
    assert!(changed_result.record.diagnostic.contains("identity changed"));
    assert!(!marker.exists());

    let runtime_limited = fixture_with_script(
        "#!/bin/sh\nprintf '%s' 'executed' > unexpected-runtime-execution\nprintf '%s\\n' '{\"valid\":true}'\n",
    );
    let runtime_marker = runtime_limited.repository_root.join("unexpected-runtime-execution");
    let mut limited_process = lattice::process::core::RuntimeConfig::default();
    limited_process.limits.stdout_max_bytes = runtime_limited.plan.bounds.output_bytes.saturating_sub(1);
    let store_root = tempfile::tempdir().expect("runtime limit store root");
    let store = lattice::store::Store::new(store_root.path().join("lattice.redb")).expect("runtime limit store");
    let authority = operation::shell::LocalReadOnlyAuthorityProvider::new(
        &runtime_limited.catalog,
        &runtime_limited.instance,
        &runtime_limited.plan,
    )
    .expect("runtime limit authority");
    let limited_result = operation::shell::run_direct(operation::shell::DirectRunRequest {
        store: &store,
        plan: &runtime_limited.plan,
        authority_provider: &authority,
        process: &limited_process,
        credentials: &lattice::credentials::EmptyCredentialProvider,
        clock: &TestClock::new(),
        cancellation: None,
    })
    .expect("runtime limit rejection");
    assert_eq!(limited_result.record.state, operation::RunState::Failed);
    assert!(limited_result.record.diagnostic.contains("request was rejected"));
    assert!(!runtime_marker.exists());

    let timeout = fixture_with_script("#!/bin/sh\nwhile :; do :; done\n");
    let timeout_result = execute_once(&timeout, None);
    assert_eq!(timeout_result.record.state, operation::RunState::Unknown);
    assert!(timeout_result.record.diagnostic.contains("timed out"));

    let cancelled = fixture(r#"{"valid":true}"#);
    let cancellation = std::sync::atomic::AtomicBool::new(true);
    let cancelled_result = execute_once(&cancelled, Some(&cancellation));
    assert_eq!(cancelled_result.record.state, operation::RunState::Unknown);
    assert!(cancelled_result.record.diagnostic.contains("cancelled"));

    let oversized_output = "x".repeat(TEST_OVERSIZED_OUTPUT_BYTES);
    let truncated = fixture(&oversized_output);
    let truncated_result = execute_once(&truncated, None);
    assert_eq!(truncated_result.record.state, operation::RunState::Unknown);
    assert!(truncated_result.record.diagnostic.contains("truncated"));

    let nonzero = fixture_with_script(&format!("#!/bin/sh\nexit {TEST_NONZERO_EXIT_CODE}\n"));
    let nonzero_result = execute_once(&nonzero, None);
    assert_eq!(nonzero_result.record.state, operation::RunState::Failed);
    assert!(nonzero_result.record.diagnostic.contains("nonzero"));

    let overclaiming = fixture(r#"{"valid":true,"release_ready":true}"#);
    let overclaiming_result = execute_once(&overclaiming, None);
    assert_eq!(overclaiming_result.record.state, operation::RunState::Failed);
    assert!(overclaiming_result.record.diagnostic.contains("prohibited"));
}

#[test]
fn promotion_is_preview_first_create_only_and_round_trips_the_same_plan() {
    let fixture = fixture(r#"{"valid":true}"#);
    let output_root = tempfile::tempdir().expect("promotion root");
    let destination = output_root.path().join("promoted-operation.ncl");
    let preview = operation::promotion::create(&fixture.plan, "promoted-operation", &destination, false)
        .expect("promotion preview");
    assert!(!preview.created);
    assert!(!destination.exists());

    let created = operation::promotion::create(&fixture.plan, "promoted-operation", &destination, true)
        .expect("promotion create");
    assert!(created.created);
    assert!(operation::promotion::create(&fixture.plan, "promoted-operation", &destination, false).is_err());
    assert!(operation::promotion::create(&fixture.plan, "promoted-operation", &destination, true,).is_err());
    assert_eq!(std::fs::read_dir(output_root.path()).expect("promotion directory").count(), 1);

    let imported = lattice::workflow_format::load_workflow_nickel_file(&destination).expect("promoted workflow import");
    let operation_plan = imported
        .configs
        .values()
        .find_map(|config| match config {
            lattice::engine::workflow::NodeConfig::Operation(plan) => Some(plan.as_ref()),
            _ => None,
        })
        .expect("promoted operation node");
    assert_eq!(operation_plan, &fixture.plan);
    assert_eq!(created.plan.invocation_identity, fixture.plan.identity);
    assert!(
        !created
            .plan
            .safe_summary
            .contains(&fixture.plan.target[operation::runtime::REPOSITORY_ROOT_TARGET].to_string())
    );
}

#[test]
fn promotion_denies_stale_unsafe_unrepresentable_and_secret_bearing_input() {
    use std::os::unix::ffi::OsStringExt as _;

    let fixture = fixture(r#"{"valid":true}"#);
    let output_root = tempfile::tempdir().expect("promotion denial root");
    let destination = output_root.path().join("denied.ncl");

    let mut stale = fixture.plan.clone();
    stale.type_revision.push_str("-stale");
    assert!(operation::promotion::preview(&stale, "stale", &destination).is_err());

    let mut contract_drift = fixture.plan.clone();
    contract_drift.output_contract.deny_unknown_fields = !contract_drift.output_contract.deny_unknown_fields;
    assert!(operation::promotion::preview(&contract_drift, "contract-drift", &destination).is_err());

    let relative = std::path::Path::new("../unsafe.ncl");
    assert!(operation::promotion::preview(&fixture.plan, "relative", relative).is_err());
    let absolute_traversal = output_root.path().join("nested/../unsafe.ncl");
    assert!(operation::promotion::preview(&fixture.plan, "traversal", &absolute_traversal).is_err());
    assert!(
        operation::promotion::preview(&fixture.plan, "wrong-extension", &output_root.path().join("unsafe.json"))
            .is_err()
    );

    let invalid_utf8_name = std::ffi::OsString::from_vec(vec![u8::MAX, b'.', b'n', b'c', b'l']);
    assert!(
        operation::promotion::preview(&fixture.plan, "invalid-utf8", &output_root.path().join(invalid_utf8_name))
            .is_err()
    );

    let collision_destination = output_root.path().join("collision.ncl");
    let collision_preview =
        operation::promotion::preview(&fixture.plan, "collision", &collision_destination).expect("collision preview");
    let collision_suffix = collision_preview
        .content_identity
        .strip_prefix(operation::model::BLAKE3_PREFIX)
        .expect("content identity prefix");
    let collision_staging = output_root.path().join(format!(".collision.ncl.{collision_suffix}.tmp"));
    std::fs::write(&collision_staging, "preserve staging collision").expect("staging collision fixture");
    assert!(operation::promotion::create(&fixture.plan, "collision", &collision_destination, true).is_err());
    assert!(!collision_destination.exists());
    assert_eq!(
        std::fs::read_to_string(&collision_staging).expect("preserved staging collision"),
        "preserve staging collision"
    );

    let mut secret_bearing = fixture.plan.clone();
    secret_bearing
        .arguments
        .insert("api_token".to_string(), serde_json::Value::String("plaintext".to_string()));
    secret_bearing.arguments_identity =
        operation::identity::arguments(&secret_bearing.arguments).expect("arguments identity");
    secret_bearing.identity = operation::identity::invocation(&secret_bearing).expect("secret-bearing identity");
    assert!(operation::promotion::preview(&secret_bearing, "secret-bearing", &destination).is_err());
    assert!(!destination.exists());
}

#[tokio::test]
async fn promoted_workflow_executes_the_same_plan_through_product_authority() {
    let fixture = fixture(r#"{"valid":true,"changes":[]}"#);
    let (workflow, _) =
        operation::promotion::workflow_for_plan(&fixture.plan, "promoted-operation").expect("promoted workflow");
    let store_root = tempfile::tempdir().expect("workflow store root");
    let store = lattice::store::Store::new(store_root.path().join("lattice.redb")).expect("store");
    let authority =
        operation::shell::LocalReadOnlyAuthorityProvider::new(&fixture.catalog, &fixture.instance, &fixture.plan)
            .expect("workflow operation authority");
    let context = authority
        .load(lattice::engine::executor::RunId(TEST_WORKFLOW_RUN_ID), Some(store))
        .expect("workflow authority context");
    let mut executor = lattice::engine::executor::Executor::new();
    let (_, results) = executor.run_authorized(&workflow, &HashMap::new(), &context).await;

    let [result] = results.as_slice() else {
        panic!("expected one operation result");
    };
    let lattice::engine::executor::NodeStatus::Completed { outputs, cached } = &result.status else {
        panic!("promoted operation did not complete: {:?}", result.status);
    };
    assert!(!cached);
    assert_eq!(outputs.len(), 1);
    assert!(outputs.values().any(|value| matches!(
        &value.data,
        lattice::engine::value::ValueData::Json(json)
            if json == &serde_json::json!({"valid": true, "changes": []})
    )));
}

#[tokio::test]
async fn local_operation_authority_denies_a_different_process_payload() {
    let authorized = fixture(r#"{"valid":true}"#);
    let denied = fixture_with_script(
        "#!/bin/sh\nprintf '%s' 'executed' > unexpected-authority-execution\nprintf '%s\\n' '{\"valid\":true}'\n",
    );
    let marker = denied.repository_root.join("unexpected-authority-execution");
    let (workflow, _) =
        operation::promotion::workflow_for_plan(&denied.plan, "denied-operation").expect("denied workflow");
    let store_root = tempfile::tempdir().expect("authority store root");
    let store = lattice::store::Store::new(store_root.path().join("lattice.redb")).expect("authority store");
    let authority = operation::shell::LocalReadOnlyAuthorityProvider::new(
        &authorized.catalog,
        &authorized.instance,
        &authorized.plan,
    )
    .expect("exact local authority");
    let context = authority
        .load(lattice::engine::executor::RunId(TEST_WORKFLOW_RUN_ID), Some(store))
        .expect("authority context");
    let mut executor = lattice::engine::executor::Executor::new();
    let (_, results) = executor.run_authorized(&workflow, &HashMap::new(), &context).await;

    let [result] = results.as_slice() else {
        panic!("expected one denied operation result");
    };
    assert!(matches!(result.status, lattice::engine::executor::NodeStatus::Failed { .. }));
    assert!(!marker.exists());
}

#[test]
fn authored_nickel_catalog_and_instances_load_and_negative_fixtures_fail_closed() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let catalog_path = root.join("builtins/operations/catalog.ncl");
    let instance_path = root.join("builtins/operations/instances.example.ncl");
    let catalog = operation::format::load_catalog(&catalog_path).expect("authored catalog");
    let instances = operation::format::load_instances(&instance_path, &catalog).expect("authored instances");
    let executor_path =
        std::fs::canonicalize(root.join("builtins/operations/cairn-read-only-fixture")).expect("fixture executor path");

    assert_eq!(catalog.types.len(), 1);
    assert_eq!(instances.len(), 1);
    assert_eq!(
        operation::identity::executor_file(&executor_path).expect("fixture executor identity"),
        catalog.types["cairn.repository"].definition.runtime.executor_identity
    );
    assert!(
        operation::format::load_catalog(&root.join("tests/fixtures/operations/catalog-unknown-field.ncl")).is_err()
    );
    assert!(
        operation::format::load_catalog(
            &root.join("tests/fixtures/operations/catalog-unknown-execution-requirement.ncl")
        )
        .is_err()
    );
    assert!(
        operation::format::load_instances(
            &root.join("tests/fixtures/operations/instance-behavior-override.ncl"),
            &catalog,
        )
        .is_err()
    );
    let oversized_arguments = " ".repeat(operation::model::MAXIMUM_OPERATION_BYTES + 1);
    assert!(operation::format::parse_arguments(&oversized_arguments).is_err());
}

#[test]
fn operation_metadata_discovery_does_not_require_an_executor_or_credential_provider() {
    let fixture = fixture(r#"{"valid":true}"#);
    let types = operation::format::type_metadata(&fixture.catalog);
    let instances = operation::format::instance_metadata(&BTreeMap::from([(
        fixture.instance.definition.id.clone(),
        fixture.instance.clone(),
    )]));
    let host_definitions = operation::format::host_tool_definitions(&fixture.catalog).expect("safe host metadata");

    assert_eq!(types.len(), 1);
    assert_eq!(instances.len(), 1);
    assert_eq!(instances[0].credential_reference_count, 0);
    assert_eq!(instances[0].target_fact_count, 1);
    assert_eq!(host_definitions.len(), EXPECTED_HOST_TOOL_COUNT);
    assert!(
        host_definitions
            .iter()
            .all(|definition| definition.status() == animus_adapters::tool::DefinitionStatus::Inactive)
    );
    assert_eq!(host_definitions[0].source_identity(), fixture.catalog.types["cairn.repository"].definition.revision);
    assert!(!format!("{host_definitions:?}").contains(fixture.executable.to_string_lossy().as_ref()));
}

#[test]
fn operation_data_inputs_bind_exact_versions_into_the_invocation_identity() {
    let fixture = fixture(r#"{"valid":true}"#);
    let input = lattice::operation_data::ResolvedInput {
        version_identity: operation::identity::hash_bytes("test.operation-data-version.v1", b"version"),
        content_identity: operation::identity::hash_bytes("test.operation-data-content.v1", b"content"),
    };
    let bound =
        lattice::operation_data::bind_resolved_inputs(&fixture.plan, vec![input.clone()]).expect("bound invocation");
    lattice::operation_data::validate_bound_invocation_plan(&bound).expect("valid bound invocation");
    assert_eq!(bound.base_invocation_identity, fixture.plan.identity);
    assert_eq!(bound.resolved_data_inputs, vec![input.clone()]);

    let mut tampered = bound;
    tampered.resolved_data_inputs[0].content_identity =
        operation::identity::hash_bytes("test.operation-data-content.v1", b"changed");
    assert!(lattice::operation_data::validate_bound_invocation_plan(&tampered).is_err());
    assert!(lattice::operation_data::bind_resolved_inputs(&fixture.plan, vec![input.clone(), input]).is_err());
}

#[test]
fn test_clock_is_monotonic() {
    let clock = TestClock::new();
    assert!(clock.observe_epoch_millis() < clock.observe_epoch_millis());
}

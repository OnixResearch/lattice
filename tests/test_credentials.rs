//! Tests for credential references, providers, redaction, and cache identity.

use std::collections::HashMap;
use std::io::Write;
use std::sync::Arc;
use std::sync::Mutex;

use lattice::agent_node::FakeProcessRunner;
use lattice::agent_node::ProcessError;
use lattice::agent_node::ProcessOutput;
use lattice::config::store_path;
use lattice::credentials::CommandCredentialProvider;
use lattice::credentials::CommandCredentialProviderConfig;
use lattice::credentials::CredentialError;
use lattice::credentials::CredentialProvider;
use lattice::credentials::CredentialProviderCapabilities;
use lattice::credentials::CredentialProviderMetadata;
use lattice::credentials::CredentialProviderRegistry;
use lattice::credentials::EnvironmentCredentialProvider;
use lattice::credentials::LocalCredentialVault;
use lattice::credentials::LocalVaultCredentialProvider;
use lattice::credentials::MapCredentialProvider;
use lattice::credentials::NodeCredentialRefs;
use lattice::credentials::SecretRef;
use lattice::credentials::redact_text;
use lattice::credentials::seal_local_vault_entry;
use lattice::credentials::test_provider_reference;
use lattice::engine::executor::Executor;
use lattice::engine::executor::NodeStatus;
use lattice::engine::executor::RunId;
use lattice::engine::executor::node_execution_cache_key_with_credentials;
use lattice::engine::workflow::Workflow;
use lattice::run::history::detail_record_from_results;
use lattice::store::Store;
use lattice::workflow_format::WorkflowFormatError;
use lattice::workflow_format::load_workflow_nickel_file;
use lattice::workflow_format::workflow_to_nickel;
use lattice::workflow_validation::DiagnosticSeverity;
use lattice::workflow_validation::WorkflowDiagnosticCode;
use lattice::workflow_validation::format_diagnostic;
use lattice::workflow_validation::validate_workflow;
use lattice::workflow_validation::validate_workflow_with_credential_registry;
use pretty_assertions::assert_eq;
use tracing::instrument::WithSubscriber;

const PROVIDER_NAME: &str = "env";
const SECRET_KEY: &str = "api_token";
const OTHER_SECRET_KEY: &str = "other_token";
const SECRET_ENV_NAME: &str = "TOKEN";
const SECRET_VALUE: &str = "super-secret-token";
const OTHER_SECRET_VALUE: &str = "other-secret-token";
const REDACTED_VALUE: &str = "<redacted>";
const WORKFLOW_NAME: &str = "credential-workflow";
const NODE_LABEL: &str = "credential-node";
const PRINT_TOKEN_COMMAND: &str = "printf \"$TOKEN\"; printf \"$TOKEN\" >&2";
const STDOUT_LABEL: &str = "stdout";
const STDERR_LABEL: &str = "stderr";
const VAULT_KEY_MATERIAL: &str = "test-vault-key";
const VAULT_NONCE: &[u8] = b"credential-nonce";
const COMMAND_PROVIDER_PROGRAM: &str = "credential-command";
const COMMAND_OUTPUT_LIMIT_BYTES: usize = 4;
const OVERLAPPING_SECRET_VALUE: &str = "secret-token";
const PLAINTEXT_IMPORT_VALUE: &str = "inline-plaintext-secret";
const CONFIG_FILE_NAME: &str = "config.ncl";
const WORKFLOW_FILE_NAME: &str = "credential.ncl";
const CREDENTIAL_STORE_FILE_NAME: &str = "credential-evidence.redb";
const RUN_ID_VALUE: u64 = 73;
const DEFAULT_TIMEOUT_SECS: u64 = 60;
const MAX_PARALLELISM: usize = 2;
const PLAINTEXT_DIAGNOSTIC: &str = "contains forbidden plaintext secret material";

#[derive(Clone)]
struct CapturedLogWriter {
    bytes: Arc<Mutex<Vec<u8>>>,
}

impl Write for CapturedLogWriter {
    fn write(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
        self.bytes.lock().expect("captured log lock").extend_from_slice(buffer);
        Ok(buffer.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn secret_ref() -> SecretRef {
    SecretRef::new(PROVIDER_NAME, SECRET_KEY, None)
}

fn credential_refs(secret_ref: SecretRef) -> NodeCredentialRefs {
    NodeCredentialRefs {
        env: [(SECRET_ENV_NAME.to_string(), secret_ref)].into(),
    }
}

fn workflow_with_secret_ref() -> (Workflow, rat_nodegraph::model::NodeId) {
    let mut workflow = Workflow::new(WORKFLOW_NAME);
    let node_id = workflow.add_shell_node(NODE_LABEL, PRINT_TOKEN_COMMAND);
    workflow.set_node_credentials(node_id, credential_refs(secret_ref()));
    (workflow, node_id)
}

fn output_text(
    workflow: &Workflow,
    node_id: rat_nodegraph::model::NodeId,
    outputs: &HashMap<rat_nodegraph::model::PortId, lattice::engine::Value>,
    label: &str,
) -> String {
    let node = workflow.graph.node(node_id).expect("node exists");
    let port = node.output_ports.iter().find(|port| port.label == label).expect("output port exists");
    outputs.get(&port.id).expect("output value exists").as_display_string()
}

fn assert_text_omits_secret(surface: &str, value: &str) {
    assert!(!surface.contains(value), "surface leaked sensitive value: {surface}");
}

fn assert_bytes_omit_secret(surface: &[u8], value: &str) {
    assert!(!surface.windows(value.len()).any(|window| window == value.as_bytes()));
}

fn write_import_config(temp_dir: &tempfile::TempDir) -> (std::path::PathBuf, std::path::PathBuf) {
    let data_dir = temp_dir.path().join("data");
    let config_path = temp_dir.path().join(CONFIG_FILE_NAME);
    std::fs::write(
        &config_path,
        format!(
            "{{ data_dir = {:?}, default_timeout_secs = {DEFAULT_TIMEOUT_SECS}, shell = \"sh\", max_parallelism = {MAX_PARALLELISM} }}\n",
            data_dir.display().to_string(),
        ),
    )
    .expect("import config written");
    (config_path, data_dir)
}

fn nickel_with_plaintext_value(workflow: &Workflow, value: &str) -> String {
    workflow_to_nickel(workflow).replace("field = [] }", &format!("field = [], value = {value} }}"))
}

// r[verify n8nclone.credentials.secret_refs.model]
// r[verify n8nclone.credentials.secret_refs.model.store_ref]
// r[verify n8nclone.workflow_format.credentials.providers.export]
#[tokio::test]
async fn resolved_secret_is_not_saved_or_exported_with_workflow() {
    let (workflow, node_id) = workflow_with_secret_ref();
    let provider = MapCredentialProvider::new([(secret_ref().identity(), SECRET_VALUE.to_string())].into());
    let executor = Executor::new_with_credential_provider(Box::new(provider));
    let result = executor.execute_node(&workflow, node_id, &HashMap::new()).await;
    assert!(matches!(result.status, NodeStatus::Completed { .. }));

    let temp_dir = tempfile::tempdir().expect("temporary workflow store directory");
    let database_path = temp_dir.path().join(CREDENTIAL_STORE_FILE_NAME);
    let store = Store::new(&database_path).expect("workflow store opens");
    store.save_workflow(&workflow).expect("workflow saves after secret resolution");
    let loaded = store.load_workflow(WORKFLOW_NAME).expect("workflow loads").expect("workflow exists");
    let serialized = serde_json::to_string(&loaded).expect("loaded workflow serializes");
    let exported = workflow_to_nickel(&loaded);
    let persisted = std::fs::read(&database_path).expect("workflow database reads");

    assert!(serialized.contains(PROVIDER_NAME));
    assert!(serialized.contains(SECRET_KEY));
    assert!(exported.contains(PROVIDER_NAME));
    assert!(exported.contains(SECRET_KEY));
    assert_text_omits_secret(&serialized, SECRET_VALUE);
    assert_text_omits_secret(&exported, SECRET_VALUE);
    assert_bytes_omit_secret(&persisted, SECRET_VALUE);
}

// r[verify n8nclone.workflow_format.credentials.providers.import]
#[test]
fn nickel_import_round_trip_preserves_opaque_provider_reference_without_resolution() {
    let (workflow, node_id) = workflow_with_secret_ref();
    let temp_dir = tempfile::tempdir().expect("temporary credential import directory");
    let workflow_path = temp_dir.path().join(WORKFLOW_FILE_NAME);
    std::fs::write(&workflow_path, workflow_to_nickel(&workflow)).expect("credential workflow written");

    let imported =
        load_workflow_nickel_file(&workflow_path).expect("credential workflow imports without provider access");
    let imported_ref = imported
        .node_credentials(node_id)
        .and_then(|refs| refs.env.get(SECRET_ENV_NAME))
        .expect("opaque credential reference imports");

    assert_eq!(imported_ref.provider, PROVIDER_NAME);
    assert_eq!(imported_ref.key, SECRET_KEY);
    assert_eq!(imported_ref.field, None);
}

// r[verify n8nclone.credentials.provider_interface]
// r[verify n8nclone.credentials.provider_interface.resolve]
// r[verify n8nclone.credentials.redaction]
// r[verify n8nclone.credentials.redaction.output]
#[tokio::test]
async fn credential_provider_redacts_output_logs_history_json_and_persistence() {
    let (workflow, node_id) = workflow_with_secret_ref();
    let provider = MapCredentialProvider::new([(secret_ref().identity(), SECRET_VALUE.to_string())].into());
    let executor = Executor::new_with_credential_provider(Box::new(provider));
    let captured_log_bytes = Arc::new(Mutex::new(Vec::new()));
    let writer_bytes = Arc::clone(&captured_log_bytes);
    let subscriber = tracing_subscriber::fmt()
        .without_time()
        .with_ansi(false)
        .with_target(false)
        .with_writer(move || CapturedLogWriter {
            bytes: Arc::clone(&writer_bytes),
        })
        .finish();
    let result = executor.execute_node(&workflow, node_id, &HashMap::new()).with_subscriber(subscriber).await;

    let NodeStatus::Completed { outputs, .. } = &result.status else {
        panic!("expected completed node, got {:?}", result.status);
    };
    let stdout = output_text(&workflow, node_id, outputs, STDOUT_LABEL);
    let stderr = output_text(&workflow, node_id, outputs, STDERR_LABEL);
    let result_debug = format!("{result:?}");
    let detail = detail_record_from_results(RunId(RUN_ID_VALUE), &workflow, std::slice::from_ref(&result));
    let detail_json = serde_json::to_string(&detail).expect("run detail serializes");
    let temp_dir = tempfile::tempdir().expect("temporary run detail directory");
    let database_path = temp_dir.path().join(CREDENTIAL_STORE_FILE_NAME);
    let store = Store::new(&database_path).expect("run detail store opens");
    store.save_run_details(&detail).expect("run details save");
    let loaded = store.load_run_details(RunId(RUN_ID_VALUE)).expect("run details load").expect("run details exist");
    let loaded_json = serde_json::to_string(&loaded).expect("loaded details serialize");
    let persisted = std::fs::read(&database_path).expect("run detail database reads");
    let captured_logs = String::from_utf8(captured_log_bytes.lock().expect("captured log lock").clone())
        .expect("captured logs are UTF-8");

    assert_eq!(stdout, REDACTED_VALUE);
    assert_eq!(stderr, REDACTED_VALUE);
    assert!(detail_json.contains(REDACTED_VALUE));
    assert!(loaded_json.contains(REDACTED_VALUE));
    assert!(captured_logs.contains(NODE_LABEL));
    for surface in [
        &stdout,
        &stderr,
        &result_debug,
        &detail_json,
        &loaded_json,
        &captured_logs,
    ] {
        assert_text_omits_secret(surface, SECRET_VALUE);
    }
    assert_bytes_omit_secret(&persisted, SECRET_VALUE);
}

// r[verify n8nclone.credentials.provider_interface]
// r[verify n8nclone.credentials.provider_interface.missing]
#[tokio::test]
async fn missing_secret_fails_before_shell_execution() {
    let (workflow, node_id) = workflow_with_secret_ref();
    let executor = Executor::new();

    let result = executor.execute_node(&workflow, node_id, &HashMap::new()).await;

    match result.status {
        NodeStatus::Failed { error } => assert!(error.contains("missing secret"), "error: {error}"),
        other => panic!("expected failed node, got {other:?}"),
    }
}

// r[verify n8nclone.credentials.redaction]
#[test]
fn redaction_masks_overlapping_duplicate_and_empty_secret_values() {
    let redacted = redact_text("prefix super-secret-token suffix secret-token other-secret-token", &[
        OVERLAPPING_SECRET_VALUE.to_string(),
        SECRET_VALUE.to_string(),
        String::new(),
        SECRET_VALUE.to_string(),
        OTHER_SECRET_VALUE.to_string(),
    ]);

    assert_eq!(redacted, "prefix <redacted> suffix <redacted> <redacted>");
    assert_text_omits_secret(&redacted, SECRET_VALUE);
    assert_text_omits_secret(&redacted, OVERLAPPING_SECRET_VALUE);
}

// r[verify n8nclone.credentials.validation]
// r[verify n8nclone.credentials.validation.malformed]
#[test]
fn malformed_secret_reference_is_a_fatal_validation_diagnostic() {
    let mut workflow = Workflow::new(WORKFLOW_NAME);
    let node_id = workflow.add_shell_node(NODE_LABEL, PRINT_TOKEN_COMMAND);
    workflow.set_node_credentials(node_id, credential_refs(SecretRef::new("", SECRET_KEY, None)));

    let diagnostics = validate_workflow(&workflow);

    assert!(diagnostics.iter().any(|diagnostic| diagnostic.code == WorkflowDiagnosticCode::MalformedSecretRef));
}

// r[verify n8nclone.credentials.redaction]
// r[verify n8nclone.credentials.redaction.validation]
#[test]
fn validation_text_and_json_preserve_reference_identity_without_plaintext_value() {
    let field_ref = SecretRef::new(PROVIDER_NAME, SECRET_KEY, Some("field".to_string()));
    let mut workflow = Workflow::new(WORKFLOW_NAME);
    let node_id = workflow.add_shell_node(NODE_LABEL, PRINT_TOKEN_COMMAND);
    workflow.set_node_credentials(node_id, credential_refs(field_ref.clone()));
    let mut registry = CredentialProviderRegistry::new();
    registry
        .insert_provider(
            provider_metadata(PROVIDER_NAME),
            Box::new(EnvironmentCredentialProvider::from_map(
                [(SECRET_KEY.to_string(), SECRET_VALUE.to_string())].into(),
            )),
        )
        .expect("provider registered");

    let diagnostics = validate_workflow_with_credential_registry(&workflow, &registry);
    let diagnostic = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == WorkflowDiagnosticCode::UnsupportedCredentialField)
        .expect("unsupported field diagnostic");
    let text = format_diagnostic(diagnostic);
    let json = serde_json::to_string(diagnostic).expect("credential diagnostic serializes");
    let identity = field_ref.identity();

    assert_eq!(diagnostic.severity, DiagnosticSeverity::Fatal);
    assert!(text.contains(&identity));
    assert!(json.contains(&identity));
    assert_text_omits_secret(&text, SECRET_VALUE);
    assert_text_omits_secret(&json, SECRET_VALUE);
}

// r[verify n8nclone.credentials.cache_identity]
// r[verify n8nclone.credentials.cache_identity.reference]
#[test]
fn cache_identity_uses_secret_reference_not_secret_value() {
    let (workflow, node_id) = workflow_with_secret_ref();
    let config = workflow.node_config(node_id).expect("node config exists");
    let key_from_ref =
        node_execution_cache_key_with_credentials(node_id, config, workflow.node_credentials(node_id), &HashMap::new());

    let mut changed_workflow = Workflow::new(WORKFLOW_NAME);
    let changed_node_id = changed_workflow.add_shell_node(NODE_LABEL, PRINT_TOKEN_COMMAND);
    changed_workflow
        .set_node_credentials(changed_node_id, credential_refs(SecretRef::new(PROVIDER_NAME, OTHER_SECRET_KEY, None)));
    let changed_config = changed_workflow.node_config(changed_node_id).expect("node config exists");
    let key_from_other_ref = node_execution_cache_key_with_credentials(
        node_id,
        changed_config,
        changed_workflow.node_credentials(changed_node_id),
        &HashMap::new(),
    );
    let repeated_key =
        node_execution_cache_key_with_credentials(node_id, config, workflow.node_credentials(node_id), &HashMap::new());

    assert_eq!(key_from_ref, repeated_key);
    assert_ne!(key_from_ref, key_from_other_ref);
}

fn provider_metadata(provider_id: &str) -> CredentialProviderMetadata {
    CredentialProviderMetadata {
        id: provider_id.to_string(),
        display_name: provider_id.to_string(),
        capabilities: CredentialProviderCapabilities::read_only_without_fields(),
    }
}

fn command_output(status_code: Option<i32>, stdout: &str) -> ProcessOutput {
    ProcessOutput {
        status_code,
        stdout: stdout.as_bytes().to_vec(),
        stderr: Vec::new(),
    }
}

// r[verify n8nclone.credentials.provider_registry]
// r[verify n8nclone.credentials.provider_registry.resolve]
// r[verify n8nclone.credentials.provider_registry.unknown]
// r[verify n8nclone.workflow_validation.credentials.providers]
#[test]
fn registry_resolves_registered_provider_and_validation_rejects_unknown_provider() {
    let mut registry = CredentialProviderRegistry::new();
    registry
        .insert_provider(
            provider_metadata(PROVIDER_NAME),
            Box::new(EnvironmentCredentialProvider::from_map(
                [(SECRET_KEY.to_string(), SECRET_VALUE.to_string())].into(),
            )),
        )
        .expect("provider registered");
    let (workflow, _node_id) = workflow_with_secret_ref();
    let value = registry.resolve(&secret_ref()).expect("registered provider resolves");
    let unknown_ref = SecretRef::new("missing", SECRET_KEY, None);
    let mut unknown_workflow = Workflow::new(WORKFLOW_NAME);
    let node_id = unknown_workflow.add_shell_node(NODE_LABEL, PRINT_TOKEN_COMMAND);
    unknown_workflow.set_node_credentials(node_id, credential_refs(unknown_ref));

    let diagnostics = validate_workflow_with_credential_registry(&unknown_workflow, &registry);

    assert_eq!(value.expose_for_execution(), SECRET_VALUE);
    assert!(validate_workflow_with_credential_registry(&workflow, &registry).is_empty());
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == WorkflowDiagnosticCode::UnknownCredentialProvider)
    );
}

// r[verify n8nclone.workflow_validation.credentials.providers]
#[test]
fn registry_validation_rejects_unsupported_field_access() {
    let mut registry = CredentialProviderRegistry::new();
    registry
        .insert_provider(provider_metadata(PROVIDER_NAME), Box::new(MapCredentialProvider::default()))
        .expect("provider registered");
    let mut workflow = Workflow::new(WORKFLOW_NAME);
    let node_id = workflow.add_shell_node(NODE_LABEL, PRINT_TOKEN_COMMAND);
    workflow.set_node_credentials(
        node_id,
        credential_refs(SecretRef::new(PROVIDER_NAME, SECRET_KEY, Some("field".into()))),
    );

    let diagnostics = validate_workflow_with_credential_registry(&workflow, &registry);

    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == WorkflowDiagnosticCode::UnsupportedCredentialField)
    );
}

// r[verify n8nclone.credentials.providers.env]
// r[verify n8nclone.credentials.providers.env.resolve]
// r[verify n8nclone.credentials.providers.env.missing]
#[test]
fn environment_provider_resolves_present_secret_and_rejects_missing_or_empty_secret() {
    let provider = EnvironmentCredentialProvider::from_map(
        [
            (SECRET_KEY.to_string(), SECRET_VALUE.to_string()),
            (OTHER_SECRET_KEY.to_string(), String::new()),
        ]
        .into(),
    );

    let value = provider.resolve(&secret_ref()).expect("env secret resolves");
    let missing = provider.resolve(&SecretRef::new(PROVIDER_NAME, "missing", None));
    let empty = provider.resolve(&SecretRef::new(PROVIDER_NAME, OTHER_SECRET_KEY, None));

    assert_eq!(value.expose_for_execution(), SECRET_VALUE);
    assert!(matches!(missing, Err(CredentialError::MissingSecret { .. })));
    assert!(matches!(empty, Err(CredentialError::MissingSecret { .. })));
}

// r[verify n8nclone.credentials.providers.local_vault]
#[test]
fn local_vault_provider_resolves_and_rejects_tampered_or_wrong_key_entries() {
    let entry = seal_local_vault_entry(SECRET_KEY, SECRET_VALUE, VAULT_KEY_MATERIAL, VAULT_NONCE);
    let provider = LocalVaultCredentialProvider::new(
        LocalCredentialVault::new([(SECRET_KEY.to_string(), entry.clone())].into()),
        VAULT_KEY_MATERIAL,
    );
    let wrong_key_provider = LocalVaultCredentialProvider::new(
        LocalCredentialVault::new([(SECRET_KEY.to_string(), entry.clone())].into()),
        "wrong-key",
    );
    let mut tampered_entry = entry;
    tampered_entry.ciphertext_hex.push_str("00");
    let tampered_provider = LocalVaultCredentialProvider::new(
        LocalCredentialVault::new([(SECRET_KEY.to_string(), tampered_entry)].into()),
        VAULT_KEY_MATERIAL,
    );

    let value = provider.resolve(&secret_ref()).expect("vault resolves");
    let wrong_key = wrong_key_provider.resolve(&secret_ref());
    let tampered = tampered_provider.resolve(&secret_ref());

    assert_eq!(value.expose_for_execution(), SECRET_VALUE);
    assert!(matches!(wrong_key, Err(CredentialError::TamperedSecret { .. })));
    assert!(matches!(tampered, Err(CredentialError::TamperedSecret { .. })));
}

// r[verify n8nclone.credentials.providers.command]
#[test]
fn command_provider_resolves_json_and_rejects_failures() {
    let config = CommandCredentialProviderConfig::new(COMMAND_PROVIDER_PROGRAM);
    let provider = CommandCredentialProvider::new(
        config.clone(),
        FakeProcessRunner::new(Ok(command_output(Some(0), "{\"value\":\"super-secret-token\"}"))),
    );
    let nonzero = CommandCredentialProvider::new(
        config.clone(),
        FakeProcessRunner::new(Ok(command_output(Some(1), "super-secret-token"))),
    );
    let malformed =
        CommandCredentialProvider::new(config.clone(), FakeProcessRunner::new(Ok(command_output(Some(0), "{"))));
    let timeout = CommandCredentialProvider::new(config, FakeProcessRunner::new(Err(ProcessError::Timeout)));

    let value = provider.resolve(&secret_ref()).expect("command resolves");
    let failed = nonzero.resolve(&secret_ref());
    let malformed_json = malformed.resolve(&secret_ref());
    let timed_out = timeout.resolve(&secret_ref());

    assert_eq!(value.expose_for_execution(), SECRET_VALUE);
    assert!(matches!(failed, Err(CredentialError::ProviderFailure { .. })));
    assert!(matches!(malformed_json, Err(CredentialError::ProviderFailure { .. })));
    assert!(matches!(timed_out, Err(CredentialError::ProviderFailure { .. })));
}

// r[verify n8nclone.credentials.providers.command]
#[test]
fn command_provider_enforces_output_limit() {
    let mut config = CommandCredentialProviderConfig::new(COMMAND_PROVIDER_PROGRAM);
    config.output_limit_bytes = COMMAND_OUTPUT_LIMIT_BYTES;
    let provider = CommandCredentialProvider::new(
        config,
        FakeProcessRunner::new(Ok(command_output(Some(0), "super-secret-token"))),
    );

    let result = provider.resolve(&secret_ref());

    assert!(matches!(result, Err(CredentialError::ProviderFailure { .. })));
}

// r[verify n8nclone.credentials.provider_diagnostics.redaction]
// r[verify n8nclone.tui_ux.credentials.providers.test]
#[test]
fn provider_probe_hides_resolved_secret_and_redacts_error_key() {
    let provider = EnvironmentCredentialProvider::from_map([(SECRET_KEY.to_string(), SECRET_VALUE.to_string())].into());
    let success = test_provider_reference(&provider, &secret_ref());
    let failure = test_provider_reference(&provider, &SecretRef::new(PROVIDER_NAME, OTHER_SECRET_KEY, None));

    assert!(success.reachable);
    assert!(!success.message.contains(SECRET_VALUE));
    assert!(!failure.reachable);
    assert!(!failure.message.contains(OTHER_SECRET_KEY));
}

// r[verify n8nclone.credentials.provider_cache_identity]
#[test]
fn provider_backed_cache_identity_excludes_resolved_secret_values() {
    let (workflow, node_id) = workflow_with_secret_ref();
    let config = workflow.node_config(node_id).expect("node config exists");
    let key_from_ref =
        node_execution_cache_key_with_credentials(node_id, config, workflow.node_credentials(node_id), &HashMap::new());
    let provider_a =
        EnvironmentCredentialProvider::from_map([(SECRET_KEY.to_string(), SECRET_VALUE.to_string())].into());
    let provider_b =
        EnvironmentCredentialProvider::from_map([(SECRET_KEY.to_string(), OTHER_SECRET_VALUE.to_string())].into());

    let value_a = provider_a.resolve(&secret_ref()).expect("secret a resolves");
    let value_b = provider_b.resolve(&secret_ref()).expect("secret b resolves");
    let repeated_key =
        node_execution_cache_key_with_credentials(node_id, config, workflow.node_credentials(node_id), &HashMap::new());

    assert_ne!(value_a.expose_for_execution(), value_b.expose_for_execution());
    assert_eq!(key_from_ref, repeated_key);
}

// r[verify n8nclone.credentials.secret_refs.model]
// r[verify n8nclone.credentials.secret_refs.model.reject_plaintext]
// r[verify n8nclone.workflow_format.credentials.providers.reject_plaintext]
#[test]
fn nickel_import_rejects_all_non_null_plaintext_credential_shapes_with_safe_error() {
    let (workflow, _node_id) = workflow_with_secret_ref();
    let temp_dir = tempfile::tempdir().expect("temporary credential directory");
    let workflow_path = temp_dir.path().join(WORKFLOW_FILE_NAME);
    let plaintext_cases = [
        format!("{PLAINTEXT_IMPORT_VALUE:?}"),
        format!("{{ nested = {PLAINTEXT_IMPORT_VALUE:?} }}"),
        format!("[{PLAINTEXT_IMPORT_VALUE:?}]"),
        RUN_ID_VALUE.to_string(),
    ];

    for plaintext in plaintext_cases {
        std::fs::write(&workflow_path, nickel_with_plaintext_value(&workflow, &plaintext))
            .expect("credential workflow written");
        let error = load_workflow_nickel_file(&workflow_path).expect_err("inline secret rejected");
        let rendered = error.to_string();

        assert!(
            matches!(&error, WorkflowFormatError::PlaintextCredentialMaterial { .. }),
            "unexpected import error: {error:?}"
        );
        assert!(rendered.contains(PLAINTEXT_DIAGNOSTIC));
        assert_text_omits_secret(&rendered, PLAINTEXT_IMPORT_VALUE);
    }

    std::fs::write(&workflow_path, nickel_with_plaintext_value(&workflow, "null"))
        .expect("null credential compatibility workflow written");
    let loaded = load_workflow_nickel_file(&workflow_path).expect("null compatibility value imports");
    assert_eq!(
        loaded.node_credentials(loaded.graph.nodes().next().expect("loaded node").id),
        workflow.credentials.values().next()
    );
}

// r[verify n8nclone.credentials.secret_refs.model.reject_plaintext]
#[test]
fn cli_plaintext_credential_rejection_is_safe_and_precedes_store_creation() {
    let (workflow, _node_id) = workflow_with_secret_ref();
    let temp_dir = tempfile::tempdir().expect("temporary credential CLI directory");
    let (config_path, data_dir) = write_import_config(&temp_dir);
    let workflow_path = temp_dir.path().join(WORKFLOW_FILE_NAME);
    std::fs::write(&workflow_path, nickel_with_plaintext_value(&workflow, &format!("{PLAINTEXT_IMPORT_VALUE:?}")))
        .expect("plaintext workflow written");

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_lattice"))
        .arg("--config")
        .arg(&config_path)
        .arg("import")
        .arg(&workflow_path)
        .output()
        .expect("credential import command runs");
    let stdout = String::from_utf8(output.stdout).expect("import stdout is UTF-8");
    let stderr = String::from_utf8(output.stderr).expect("import stderr is UTF-8");

    assert!(!output.status.success());
    assert!(stderr.contains(PLAINTEXT_DIAGNOSTIC), "stderr: {stderr}");
    assert_text_omits_secret(&stdout, PLAINTEXT_IMPORT_VALUE);
    assert_text_omits_secret(&stderr, PLAINTEXT_IMPORT_VALUE);
    assert!(!store_path(&data_dir).exists());
}

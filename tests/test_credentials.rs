//! Tests for credential references, providers, redaction, and cache identity.

use std::collections::HashMap;

use lattice::agent_node::FakeProcessRunner;
use lattice::agent_node::ProcessError;
use lattice::agent_node::ProcessOutput;
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
use lattice::engine::executor::node_execution_cache_key_with_credentials;
use lattice::engine::workflow::Workflow;
use lattice::workflow_format::WorkflowFormatError;
use lattice::workflow_format::load_workflow_nickel_file;
use lattice::workflow_format::workflow_to_nickel;
use lattice::workflow_validation::WorkflowDiagnosticCode;
use lattice::workflow_validation::validate_workflow;
use lattice::workflow_validation::validate_workflow_with_credential_registry;
use pretty_assertions::assert_eq;

const PROVIDER_NAME: &str = "env";
const SECRET_KEY: &str = "api_token";
const OTHER_SECRET_KEY: &str = "other_token";
const SECRET_ENV_NAME: &str = "TOKEN";
const SECRET_VALUE: &str = "super-secret-token";
const OTHER_SECRET_VALUE: &str = "other-secret-token";
const REDACTED_VALUE: &str = "<redacted>";
const WORKFLOW_NAME: &str = "credential-workflow";
const NODE_LABEL: &str = "credential-node";
const PRINT_TOKEN_COMMAND: &str = "printf \"$TOKEN\"";
const SINGLE_RESULT_INDEX: usize = 0;
const VAULT_KEY_MATERIAL: &str = "test-vault-key";
const VAULT_NONCE: &[u8] = b"credential-nonce";
const COMMAND_PROVIDER_PROGRAM: &str = "credential-command";
const COMMAND_OUTPUT_LIMIT_BYTES: usize = 4;

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

// r[verify n8nclone.credentials.secret_refs.model]
#[test]
fn workflow_serializes_secret_reference_without_plaintext_secret() {
    let (workflow, _node_id) = workflow_with_secret_ref();

    let serialized = serde_json::to_string(&workflow).expect("workflow serializes");

    assert!(serialized.contains(PROVIDER_NAME));
    assert!(serialized.contains(SECRET_KEY));
    assert!(!serialized.contains(SECRET_VALUE));
}

// r[verify n8nclone.credentials.provider_interface]
// r[verify n8nclone.credentials.redaction]
#[tokio::test]
async fn credential_provider_resolves_secret_env_and_redacts_output() {
    let (workflow, node_id) = workflow_with_secret_ref();
    let provider = MapCredentialProvider::new([(secret_ref().identity(), SECRET_VALUE.to_string())].into());
    let executor = Executor::new_with_credential_provider(Box::new(provider));

    let result = executor.execute_node(&workflow, node_id, &HashMap::new()).await;

    match result.status {
        NodeStatus::Completed { outputs, .. } => {
            let node = workflow.graph.node(node_id).expect("node exists");
            let stdout_port = node.output_ports[SINGLE_RESULT_INDEX].id;
            let stdout = outputs.get(&stdout_port).expect("stdout exists");
            assert_eq!(stdout.as_display_string(), REDACTED_VALUE);
        }
        other => panic!("expected completed node, got {other:?}"),
    }
}

// r[verify n8nclone.credentials.provider_interface]
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
fn redaction_masks_secret_values() {
    let redacted = redact_text("prefix super-secret-token suffix other-secret-token", &[
        SECRET_VALUE.to_string(),
        OTHER_SECRET_VALUE.to_string(),
    ]);

    assert_eq!(redacted, "prefix <redacted> suffix <redacted>");
}

// r[verify n8nclone.credentials.validation]
#[test]
fn malformed_secret_reference_is_a_fatal_validation_diagnostic() {
    let mut workflow = Workflow::new(WORKFLOW_NAME);
    let node_id = workflow.add_shell_node(NODE_LABEL, PRINT_TOKEN_COMMAND);
    workflow.set_node_credentials(node_id, credential_refs(SecretRef::new("", SECRET_KEY, None)));

    let diagnostics = validate_workflow(&workflow);

    assert!(diagnostics.iter().any(|diagnostic| diagnostic.code == WorkflowDiagnosticCode::MalformedSecretRef));
}

// r[verify n8nclone.credentials.cache_identity]
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
#[test]
fn nickel_import_rejects_inline_plaintext_secret_field() {
    let (workflow, _node_id) = workflow_with_secret_ref();
    let temp_dir = tempfile::tempdir().expect("temporary credential directory");
    let workflow_path = temp_dir.path().join("credential.ncl");
    let nickel = workflow_to_nickel(&workflow).replace("field = [] }", "field = [], value = \"plaintext\" }");
    std::fs::write(&workflow_path, nickel).expect("credential workflow written");

    let error = load_workflow_nickel_file(&workflow_path).expect_err("inline secret rejected");

    assert!(matches!(error, WorkflowFormatError::Json(_) | WorkflowFormatError::Nickel(_)));
}

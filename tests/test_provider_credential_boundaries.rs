use std::collections::HashMap;

use lattice::agent_node::FakeProcessRunner;
use lattice::agent_node::ProcessError;
use lattice::agent_node::ProcessOutput;
use lattice::credentials::CommandCredentialProvider;
use lattice::credentials::CommandCredentialProviderConfig;
use lattice::credentials::CredentialError;
use lattice::credentials::CredentialProvider;
use lattice::credentials::EnvironmentCredentialProvider;
use lattice::credentials::LocalCredentialVault;
use lattice::credentials::LocalVaultCredentialProvider;
use lattice::credentials::MapCredentialProvider;
use lattice::credentials::NodeCredentialRefs;
use lattice::credentials::SecretRef;
use lattice::credentials::seal_local_vault_entry;
use lattice::credentials::test_provider_reference;
use lattice::engine::executor::Executor;
use lattice::engine::executor::NodeStatus;
use lattice::engine::executor::node_execution_cache_key_with_credentials;
use lattice::engine::workflow::Workflow;
use lattice::workflow_format::workflow_to_nickel;

const CACHE_PROVIDER: &str = "env";
const COMMAND_PROVIDER: &str = "command";
const VAULT_PROVIDER: &str = "local-vault";
const SECRET_KEY: &str = "api_token";
const OTHER_SECRET_KEY: &str = "secondary_token";
const SECRET_FIELD: &str = "token";
const SECRET_ENV_NAME: &str = "TOKEN";
const SECRET_VALUE: &str = "provider-boundary-secret";
const OTHER_SECRET_VALUE: &str = "different-provider-secret";
const VAULT_KEY_MATERIAL: &str = "provider-vault-key";
const VAULT_NONCE: &[u8] = b"provider-vault-nonce";
const COMMAND_PROGRAM: &str = "credential-command";
const COMMAND_TIMEOUT_SECS: u64 = 1;
const COMMAND_OUTPUT_LIMIT_BYTES: usize = 1024;
const SUCCESS_STATUS_CODE: i32 = 0;
const SIDE_EFFECT_CONTENT: &str = "ran";
const SIDE_EFFECT_FILE_NAME: &str = "provider-side-effect";
const WORKFLOW_NAME: &str = "provider-boundary-workflow";
const NODE_NAME: &str = "provider-boundary-node";
const SAFE_SUCCESS_MESSAGE: &str = "secret resolved";
const SAFE_FAILURE_MESSAGE: &str = "provider resolution failed";
const LEAKY_BACKEND_TEXT: &str = "backend included plaintext";

#[derive(Debug)]
struct LeakyFailureProvider;

impl CredentialProvider for LeakyFailureProvider {
    fn resolve(&self, secret_ref: &SecretRef) -> Result<lattice::credentials::SecretValue, CredentialError> {
        Err(CredentialError::ProviderFailure {
            identity: secret_ref.identity(),
            message: format!("{LEAKY_BACKEND_TEXT}: {SECRET_VALUE}"),
        })
    }
}

fn secret_ref(provider: &str, key: &str, field: Option<&str>) -> SecretRef {
    SecretRef::new(provider, key, field.map(str::to_string))
}

fn credential_refs(secret_ref: SecretRef) -> NodeCredentialRefs {
    NodeCredentialRefs {
        env: HashMap::from([(SECRET_ENV_NAME.to_string(), secret_ref)]),
    }
}

fn workflow_for_side_effect(
    secret_ref: SecretRef,
    side_effect_path: &std::path::Path,
) -> (Workflow, lattice::engine::workflow::NodeId) {
    let mut workflow = Workflow::new(WORKFLOW_NAME, lattice::engine::workflow::WorkflowTimestamp::epoch());
    let command = format!(
        "test -n \"${SECRET_ENV_NAME}\" && printf {SIDE_EFFECT_CONTENT} > {:?}",
        side_effect_path.display().to_string(),
    );
    let node_id = workflow.add_shell_node(NODE_NAME, command);
    workflow.set_node_credentials(node_id, credential_refs(secret_ref));
    (workflow, node_id)
}

fn command_output(secret: &str) -> ProcessOutput {
    ProcessOutput {
        status_code: Some(SUCCESS_STATUS_CODE),
        stdout: serde_json::json!({ "value": secret }).to_string().into_bytes(),
        stderr: Vec::new(),
    }
}

fn command_config() -> CommandCredentialProviderConfig {
    let mut config = CommandCredentialProviderConfig::new(COMMAND_PROGRAM);
    config.timeout_secs = COMMAND_TIMEOUT_SECS;
    config.output_limit_bytes = COMMAND_OUTPUT_LIMIT_BYTES;
    config
}

// r[verify n8nclone.credentials.provider_cache_identity.no_plaintext]
#[test]
fn cache_identity_includes_provider_key_and_field_but_not_resolved_values() {
    let temp_dir = tempfile::tempdir().expect("temporary cache directory");
    let side_effect_path = temp_dir.path().join(SIDE_EFFECT_FILE_NAME);
    let base_ref = secret_ref(CACHE_PROVIDER, SECRET_KEY, None);
    let (workflow, node_id) = workflow_for_side_effect(base_ref.clone(), &side_effect_path);
    let config = workflow.node_config(node_id).expect("node config exists");
    let inputs = HashMap::new();
    let base_key =
        node_execution_cache_key_with_credentials(node_id, config, Some(&credential_refs(base_ref.clone())), &inputs);
    let repeated_key =
        node_execution_cache_key_with_credentials(node_id, config, Some(&credential_refs(base_ref.clone())), &inputs);
    let provider_key = node_execution_cache_key_with_credentials(
        node_id,
        config,
        Some(&credential_refs(secret_ref(COMMAND_PROVIDER, SECRET_KEY, None))),
        &inputs,
    );
    let secret_key = node_execution_cache_key_with_credentials(
        node_id,
        config,
        Some(&credential_refs(secret_ref(CACHE_PROVIDER, OTHER_SECRET_KEY, None))),
        &inputs,
    );
    let field_key = node_execution_cache_key_with_credentials(
        node_id,
        config,
        Some(&credential_refs(secret_ref(CACHE_PROVIDER, SECRET_KEY, Some(SECRET_FIELD)))),
        &inputs,
    );
    let provider_a =
        EnvironmentCredentialProvider::from_map([(SECRET_KEY.to_string(), SECRET_VALUE.to_string())].into());
    let provider_b =
        EnvironmentCredentialProvider::from_map([(SECRET_KEY.to_string(), OTHER_SECRET_VALUE.to_string())].into());

    assert_ne!(
        provider_a.resolve(&base_ref).expect("first secret resolves").expose_for_execution(),
        provider_b.resolve(&base_ref).expect("second secret resolves").expose_for_execution(),
    );
    assert_eq!(base_key, repeated_key);
    assert_ne!(base_key, provider_key);
    assert_ne!(base_key, secret_key);
    assert_ne!(base_key, field_key);
    assert!(!format!("{base_key:?}").contains(SECRET_VALUE));
    assert!(!format!("{base_key:?}").contains(OTHER_SECRET_VALUE));
}

// r[verify n8nclone.credentials.providers.command.resolve]
// r[verify n8nclone.credentials.providers.command.timeout]
#[tokio::test]
async fn command_provider_reaches_execution_and_timeout_blocks_side_effects() {
    let success_dir = tempfile::tempdir().expect("temporary command success directory");
    let success_path = success_dir.path().join(SIDE_EFFECT_FILE_NAME);
    let command_ref = secret_ref(COMMAND_PROVIDER, SECRET_KEY, None);
    let (success_workflow, success_node_id) = workflow_for_side_effect(command_ref.clone(), &success_path);
    let success_provider =
        CommandCredentialProvider::new(command_config(), FakeProcessRunner::new(Ok(command_output(SECRET_VALUE))));
    let success = Executor::new_with_credential_provider(Box::new(success_provider))
        .execute_node(&success_workflow, success_node_id, &HashMap::new())
        .await;

    assert!(matches!(success.status, NodeStatus::Completed { .. }));
    assert_eq!(std::fs::read_to_string(&success_path).expect("side effect reads"), SIDE_EFFECT_CONTENT);
    assert!(!format!("{success:?}").contains(SECRET_VALUE));

    let timeout_dir = tempfile::tempdir().expect("temporary command timeout directory");
    let timeout_path = timeout_dir.path().join(SIDE_EFFECT_FILE_NAME);
    let (timeout_workflow, timeout_node_id) = workflow_for_side_effect(command_ref, &timeout_path);
    let timeout_provider =
        CommandCredentialProvider::new(command_config(), FakeProcessRunner::new(Err(ProcessError::Timeout)));
    let timeout = Executor::new_with_credential_provider(Box::new(timeout_provider))
        .execute_node(&timeout_workflow, timeout_node_id, &HashMap::new())
        .await;

    assert!(matches!(timeout.status, NodeStatus::Failed { ref error } if error.contains("process timed out")));
    assert!(!timeout_path.exists());
    assert!(!format!("{timeout:?}").contains(SECRET_VALUE));
}

// r[verify n8nclone.credentials.providers.local_vault.resolve]
// r[verify n8nclone.credentials.providers.local_vault.tamper]
#[tokio::test]
async fn local_vault_keeps_workflow_opaque_and_tamper_diagnostics_safe() {
    let entry = seal_local_vault_entry(SECRET_KEY, SECRET_VALUE, VAULT_KEY_MATERIAL, VAULT_NONCE);
    let vault_ref = secret_ref(VAULT_PROVIDER, SECRET_KEY, None);
    let success_dir = tempfile::tempdir().expect("temporary vault success directory");
    let success_path = success_dir.path().join(SIDE_EFFECT_FILE_NAME);
    let (workflow, node_id) = workflow_for_side_effect(vault_ref.clone(), &success_path);
    let workflow_text = workflow_to_nickel(&workflow);
    let provider = LocalVaultCredentialProvider::new(
        LocalCredentialVault::new([(SECRET_KEY.to_string(), entry.clone())].into()),
        VAULT_KEY_MATERIAL,
    );
    let success = Executor::new_with_credential_provider(Box::new(provider))
        .execute_node(&workflow, node_id, &HashMap::new())
        .await;

    assert!(workflow_text.contains(VAULT_PROVIDER));
    assert!(workflow_text.contains(SECRET_KEY));
    assert!(!workflow_text.contains(SECRET_VALUE));
    assert!(!workflow_text.contains(VAULT_KEY_MATERIAL));
    assert!(matches!(success.status, NodeStatus::Completed { .. }));
    assert_eq!(std::fs::read_to_string(&success_path).expect("vault side effect reads"), SIDE_EFFECT_CONTENT);

    let mut tampered_entry = entry;
    tampered_entry.ciphertext_hex.push_str("00");
    let tamper_dir = tempfile::tempdir().expect("temporary vault tamper directory");
    let tamper_path = tamper_dir.path().join(SIDE_EFFECT_FILE_NAME);
    let (tamper_workflow, tamper_node_id) = workflow_for_side_effect(vault_ref.clone(), &tamper_path);
    let tampered_provider = LocalVaultCredentialProvider::new(
        LocalCredentialVault::new([(SECRET_KEY.to_string(), tampered_entry)].into()),
        VAULT_KEY_MATERIAL,
    );
    let tampered = Executor::new_with_credential_provider(Box::new(tampered_provider))
        .execute_node(&tamper_workflow, tamper_node_id, &HashMap::new())
        .await;
    let rendered = format!("{tampered:?}");

    assert!(matches!(tampered.status, NodeStatus::Failed { ref error } if error.contains(&vault_ref.identity())));
    assert!(!tamper_path.exists());
    assert!(!rendered.contains(SECRET_VALUE));
    assert!(!rendered.contains(VAULT_KEY_MATERIAL));
}

// r[verify n8nclone.credentials.provider_diagnostics.redaction.error]
// r[verify n8nclone.credentials.provider_diagnostics.redaction.test]
#[test]
fn provider_probe_rendering_uses_fixed_safe_messages_and_opaque_identity() {
    let reference = secret_ref(CACHE_PROVIDER, SECRET_KEY, Some(SECRET_FIELD));
    let success_provider = MapCredentialProvider::new([(reference.identity(), SECRET_VALUE.to_string())].into());
    let success = test_provider_reference(&success_provider, &reference);
    let failure = test_provider_reference(&LeakyFailureProvider, &reference);
    let success_json = serde_json::to_string(&success).expect("success probe serializes");
    let failure_json = serde_json::to_string(&failure).expect("failure probe serializes");

    assert!(success.reachable);
    assert_eq!(success.identity, reference.identity());
    assert_eq!(success.message, SAFE_SUCCESS_MESSAGE);
    assert!(!failure.reachable);
    assert_eq!(failure.identity, reference.identity());
    assert_eq!(failure.message, SAFE_FAILURE_MESSAGE);
    for rendered in [&success_json, &failure_json] {
        assert!(rendered.contains(&reference.identity()));
        assert!(!rendered.contains(SECRET_VALUE));
        assert!(!rendered.contains(LEAKY_BACKEND_TEXT));
    }
}

//! Tests for credential references, providers, redaction, and cache identity.

use std::collections::HashMap;

use n8nclone::credentials::MapCredentialProvider;
use n8nclone::credentials::NodeCredentialRefs;
use n8nclone::credentials::SecretRef;
use n8nclone::credentials::redact_text;
use n8nclone::engine::executor::Executor;
use n8nclone::engine::executor::NodeStatus;
use n8nclone::engine::executor::node_execution_cache_key_with_credentials;
use n8nclone::engine::workflow::Workflow;
use n8nclone::workflow_format::WorkflowFormatError;
use n8nclone::workflow_format::load_workflow_nickel_file;
use n8nclone::workflow_format::workflow_to_nickel;
use n8nclone::workflow_validation::WorkflowDiagnosticCode;
use n8nclone::workflow_validation::validate_workflow;
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

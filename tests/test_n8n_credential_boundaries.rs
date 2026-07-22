//! Exact boundary observations for n8n credential references and parity safety.

use std::collections::HashMap;
use std::process::Command;

use lattice::credentials::MapCredentialProvider;
use lattice::credentials::NodeCredentialRefs;
use lattice::credentials::SecretRef;
use lattice::engine::executor::Executor;
use lattice::engine::executor::NodeStatus;
use lattice::engine::workflow::Workflow;
use lattice::n8n_json::N8nDiagnosticCode;
use lattice::n8n_json::import_n8n_json_str;
use lattice::n8n_json::report_to_pretty_json;
use lattice::store::Store;

const PARITY_COMMAND_FIXTURE: &str = include_str!("fixtures/n8n/parity/supported/command-pipeline.json");
const PARITY_COMMAND_REPORT: &str = include_str!("fixtures/n8n/parity/supported/command-pipeline.report.json");
const PARITY_HTTP_FIXTURE: &str = include_str!("fixtures/n8n/parity/supported/http-credential-chain.json");
const PARITY_HTTP_REPORT: &str = include_str!("fixtures/n8n/parity/supported/http-credential-chain.report.json");
const PARITY_DEFAULT_HTTP_FIXTURE: &str = include_str!("fixtures/n8n/parity/supported/default-http-method.json");
const PARITY_WEBHOOK_FIXTURE: &str = include_str!("fixtures/n8n/parity/unsupported/webhook-trigger.json");
const PARITY_WEBHOOK_REPORT: &str = include_str!("fixtures/n8n/parity/unsupported/webhook-trigger.report.json");
const PARITY_SCHEDULE_FIXTURE: &str = include_str!("fixtures/n8n/parity/unsupported/schedule-trigger.json");
const PARITY_EXPRESSION_FIXTURE: &str = include_str!("fixtures/n8n/parity/unsupported/expression-url.json");
const PARITY_PLAINTEXT_FIXTURE: &str = include_str!("fixtures/n8n/parity/unsupported/plaintext-node-credential.json");
const PARITY_MALFORMED_FIXTURE: &str = include_str!("fixtures/n8n/parity/unsupported/malformed-connection.json");

const CONFIG_FILE_NAME: &str = "config.ncl";
const DATA_DIR_NAME: &str = "data";
const STORE_FILE_NAME: &str = "lattice.redb";
const IMPORT_FILE_NAME: &str = "plaintext-import.json";
const REPORT_FILE_NAME: &str = "mapping-report.json";
const EXPORT_FILE_NAME: &str = "credential-export.json";
const EXPORT_WORKFLOW_NAME: &str = "n8n-credential-export";
const IMPORT_WORKFLOW_NAME: &str = "n8n-imported-credential-execution";
const IMPORT_NODE_NAME: &str = "Execute with credential";
const EXPORTED_NODE_NAME: &str = "Credential shell";
const CREDENTIAL_TYPE: &str = "httpHeaderAuth";
const CREDENTIAL_BINDING: &str = "N8N_CREDENTIAL_HTTPHEADERAUTH";
const IMPORT_CREDENTIAL_KEY: &str = "cred-execution";
const EXPORT_CREDENTIAL_KEY: &str = "cred-export";
const N8N_PROVIDER: &str = "n8n";
const PLAINTEXT_SECRET_SENTINEL: &str = "plaintext-secret";
const REFERENCE_GUIDANCE: &str = "opaque id/name reference";
const SIDE_EFFECT_FILE_NAME: &str = "side-effect.txt";
const SIDE_EFFECT_CONTENT: &str = "ran";
const DEFAULT_TIMEOUT_SECS: u64 = 60;
const MAX_PARALLELISM: usize = 4;
const DESIGNATED_SENTINEL_COUNT: usize = 1;

fn secret_ref(key: &str) -> SecretRef {
    SecretRef::new(N8N_PROVIDER, key, None)
}

fn credential_refs(key: &str) -> NodeCredentialRefs {
    NodeCredentialRefs {
        env: HashMap::from([(CREDENTIAL_BINDING.to_string(), secret_ref(key))]),
    }
}

fn provider_for(key: &str) -> MapCredentialProvider {
    MapCredentialProvider::new(HashMap::from([(secret_ref(key).identity(), PLAINTEXT_SECRET_SENTINEL.to_string())]))
}

fn write_test_config(temp_dir: &tempfile::TempDir) -> (std::path::PathBuf, std::path::PathBuf) {
    let data_dir = temp_dir.path().join(DATA_DIR_NAME);
    std::fs::create_dir_all(&data_dir).expect("data directory created");
    let config_path = temp_dir.path().join(CONFIG_FILE_NAME);
    std::fs::write(
        &config_path,
        format!(
            "{{\n  data_dir = {:?},\n  default_timeout_secs = {DEFAULT_TIMEOUT_SECS},\n  shell = \"sh\",\n  max_parallelism = {MAX_PARALLELISM},\n}}\n",
            data_dir.display().to_string(),
        ),
    )
    .expect("config written");
    (config_path, data_dir)
}

fn imported_shell_fixture(command: &str) -> String {
    serde_json::json!({
        "name": IMPORT_WORKFLOW_NAME,
        "nodes": [{
            "id": "credential-shell",
            "name": IMPORT_NODE_NAME,
            "type": "n8n-nodes-base.executeCommand",
            "position": [0, 0],
            "parameters": { "command": command },
            "credentials": {
                CREDENTIAL_TYPE: {
                    "id": IMPORT_CREDENTIAL_KEY,
                    "name": "Execution credential"
                }
            }
        }],
        "connections": {}
    })
    .to_string()
}

fn output_text(output: &std::process::Output) -> String {
    format!("{}{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr),)
}

fn designated_plaintext_diagnostic_is_safe() {
    let result = import_n8n_json_str(PARITY_PLAINTEXT_FIXTURE);
    let report = report_to_pretty_json(&result.report);

    assert!(result.workflow.is_none());
    assert!(
        result
            .report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == N8nDiagnosticCode::PlaintextCredentialMaterial)
    );
    assert!(result.report.diagnostics.iter().any(|diagnostic| diagnostic.message.contains(REFERENCE_GUIDANCE)));
    assert!(!report.contains(PLAINTEXT_SECRET_SENTINEL));
}

// r[verify n8nclone.credentials.n8n_json.placeholders.reference]
// r[verify lattice.repository_validation.n8n_credential_completion]
// r[verify lattice.repository_validation.n8n_credential_completion.scenario.boundaries]
#[tokio::test]
async fn imported_placeholder_resolves_before_execution_and_missing_value_blocks_side_effect() {
    let temp_dir = tempfile::tempdir().expect("temporary execution directory");
    let side_effect_path = temp_dir.path().join(SIDE_EFFECT_FILE_NAME);
    let command = format!(
        "test -n \"${CREDENTIAL_BINDING}\" && printf {SIDE_EFFECT_CONTENT} > {:?}",
        side_effect_path.display().to_string(),
    );
    let result = import_n8n_json_str(&imported_shell_fixture(&command));
    let workflow = result.workflow.expect("opaque credential fixture imports");
    let node_id = workflow.graph.nodes().next().expect("imported node exists").id;
    let imported_ref = workflow
        .node_credentials(node_id)
        .and_then(|references| references.env.get(CREDENTIAL_BINDING))
        .expect("opaque reference is stored");

    assert_eq!(imported_ref, &secret_ref(IMPORT_CREDENTIAL_KEY));
    assert!(result.report.diagnostics.is_empty());

    let executor = Executor::new_with_credential_provider(Box::new(provider_for(IMPORT_CREDENTIAL_KEY)));
    let completed = executor.execute_node(&workflow, node_id, &HashMap::new()).await;
    assert!(matches!(completed.status, NodeStatus::Completed { .. }));
    assert_eq!(std::fs::read_to_string(&side_effect_path).expect("side effect recorded"), SIDE_EFFECT_CONTENT);

    std::fs::remove_file(&side_effect_path).expect("side effect reset");
    let missing = Executor::new().execute_node(&workflow, node_id, &HashMap::new()).await;
    assert!(matches!(missing.status, NodeStatus::Failed { ref error } if error.contains("missing secret")));
    assert!(!side_effect_path.exists());
}

// r[verify n8nclone.credentials.n8n_json.placeholders.export]
// r[verify lattice.repository_validation.n8n_credential_completion.scenario.boundaries]
#[tokio::test]
async fn resolved_reference_stays_opaque_across_save_raw_store_and_cli_export() {
    let temp_dir = tempfile::tempdir().expect("temporary export directory");
    let (config_path, data_dir) = write_test_config(&temp_dir);
    let database_path = data_dir.join(STORE_FILE_NAME);
    let mut workflow = Workflow::new(EXPORT_WORKFLOW_NAME);
    let node_id = workflow.add_shell_node(EXPORTED_NODE_NAME, format!("test -n \"${CREDENTIAL_BINDING}\""));
    workflow.set_node_credentials(node_id, credential_refs(EXPORT_CREDENTIAL_KEY));

    let executor = Executor::new_with_credential_provider(Box::new(provider_for(EXPORT_CREDENTIAL_KEY)));
    let completed = executor.execute_node(&workflow, node_id, &HashMap::new()).await;
    assert!(matches!(completed.status, NodeStatus::Completed { .. }));

    let store = Store::new(&database_path).expect("store opens");
    store.save_workflow(&workflow).expect("reference-bearing workflow saves");
    drop(store);

    let output_path = temp_dir.path().join(EXPORT_FILE_NAME);
    let report_path = temp_dir.path().join(REPORT_FILE_NAME);
    let output = Command::new(env!("CARGO_BIN_EXE_lattice"))
        .arg("--config")
        .arg(&config_path)
        .arg("export-n8n")
        .arg(EXPORT_WORKFLOW_NAME)
        .arg(&output_path)
        .arg("--report")
        .arg(&report_path)
        .output()
        .expect("n8n export command runs");
    let exported_text = std::fs::read_to_string(&output_path).expect("n8n export reads");
    let report_text = std::fs::read_to_string(&report_path).expect("n8n export report reads");
    let database_text = String::from_utf8_lossy(&std::fs::read(&database_path).expect("raw store reads")).into_owned();
    let exported: serde_json::Value = serde_json::from_str(&exported_text).expect("n8n export decodes");
    let expected_identity = secret_ref(EXPORT_CREDENTIAL_KEY).identity();

    assert!(output.status.success(), "output: {}", output_text(&output));
    assert_eq!(exported["nodes"][0]["credentials"][CREDENTIAL_BINDING]["id"], expected_identity);
    assert_eq!(exported["nodes"][0]["credentials"][CREDENTIAL_BINDING]["name"], expected_identity);
    for observed in [&exported_text, &report_text, &database_text, &output_text(&output)] {
        assert!(!observed.contains(PLAINTEXT_SECRET_SENTINEL));
    }
}

// r[verify n8nclone.credentials.n8n_json.placeholders.plaintext]
// r[verify n8nclone.credentials.n8n_parity_secret_safety]
// r[verify lattice.repository_validation.n8n_credential_completion.scenario.boundaries]
// r[verify lattice.repository_validation.n8n_credential_completion.scenario.parity]
#[test]
fn plaintext_cli_import_reports_safe_guidance_without_creating_store() {
    let temp_dir = tempfile::tempdir().expect("temporary import directory");
    let (config_path, data_dir) = write_test_config(&temp_dir);
    let import_path = temp_dir.path().join(IMPORT_FILE_NAME);
    let report_path = temp_dir.path().join(REPORT_FILE_NAME);
    let database_path = data_dir.join(STORE_FILE_NAME);
    std::fs::write(&import_path, PARITY_PLAINTEXT_FIXTURE).expect("plaintext fixture written");

    let output = Command::new(env!("CARGO_BIN_EXE_lattice"))
        .arg("--config")
        .arg(&config_path)
        .arg("import-n8n")
        .arg(&import_path)
        .arg("--report")
        .arg(&report_path)
        .output()
        .expect("n8n import command runs");
    let report = std::fs::read_to_string(&report_path).expect("safe rejection report reads");
    let command_output = output_text(&output);

    assert!(!output.status.success());
    assert!(!database_path.exists());
    assert!(report.contains(REFERENCE_GUIDANCE));
    assert!(!report.contains(PLAINTEXT_SECRET_SENTINEL));
    assert!(!command_output.contains(PLAINTEXT_SECRET_SENTINEL));
}

// r[verify n8nclone.credentials.n8n_parity_secret_safety]
// r[verify lattice.repository_validation.n8n_credential_completion.scenario.parity]
#[test]
fn parity_sentinel_is_bounded_to_one_negative_input_and_all_outputs_are_safe() {
    let plaintext_free_inputs = [
        PARITY_COMMAND_FIXTURE,
        PARITY_HTTP_FIXTURE,
        PARITY_DEFAULT_HTTP_FIXTURE,
        PARITY_WEBHOOK_FIXTURE,
        PARITY_SCHEDULE_FIXTURE,
        PARITY_EXPRESSION_FIXTURE,
        PARITY_MALFORMED_FIXTURE,
    ];
    let checked_in_reports = [PARITY_COMMAND_REPORT, PARITY_HTTP_REPORT, PARITY_WEBHOOK_REPORT];

    assert!(plaintext_free_inputs.iter().all(|fixture| !fixture.contains(PLAINTEXT_SECRET_SENTINEL)));
    assert!(checked_in_reports.iter().all(|report| !report.contains(PLAINTEXT_SECRET_SENTINEL)));
    assert_eq!(PARITY_PLAINTEXT_FIXTURE.matches(PLAINTEXT_SECRET_SENTINEL).count(), DESIGNATED_SENTINEL_COUNT);
    designated_plaintext_diagnostic_is_safe();
}

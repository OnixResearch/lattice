//! Tests for operator doctor diagnostics.

use std::path::Path;
use std::process::Command;

use lattice::credentials::SecretRef;
use lattice::engine::workflow::Workflow;
use lattice::scheduler::OverlapPolicy;
use lattice::store::Store;
use lattice::triggers::IdempotencyPolicy;
use lattice::triggers::TriggerDefinition;
use lattice::triggers::TriggerKind;
use lattice::triggers::WebhookAuthPolicy;
use lattice::triggers::WebhookTrigger;

const CONFIG_FILE_NAME: &str = "config.ncl";
const STORE_FILE_NAME: &str = "lattice.redb";
const DATA_DIR_NAME: &str = "data";
const DEFAULT_TIMEOUT_SECS: u64 = 60;
const MAX_PARALLELISM: usize = 4;
const DOCTOR_WORKFLOW_NAME: &str = "doctor-workflow";
const DOCTOR_TRIGGER_ID: &str = "doctor-trigger";
const DOCTOR_WEBHOOK_PATH: &str = "/hooks/doctor";
const DOCTOR_WEBHOOK_METHOD: &str = "POST";
const DOCTOR_CONTENT_TYPE: &str = "application/json";
const DOCTOR_SECRET_PROVIDER: &str = "env";
const MISSING_DOCTOR_SECRET_KEY: &str = "LATTICE_DOCTOR_TEST_TOKEN_SHOULD_NOT_EXIST";
const SECRET_SENTINEL: &str = "super-secret-doctor-token";

fn write_config(temp_dir: &tempfile::TempDir) -> (std::path::PathBuf, std::path::PathBuf) {
    let data_dir = temp_dir.path().join(DATA_DIR_NAME);
    std::fs::create_dir_all(&data_dir).expect("data directory created");
    let config_path = temp_dir.path().join(CONFIG_FILE_NAME);
    let data_dir_string = data_dir.display().to_string();
    std::fs::write(
        &config_path,
        format!(
            "{{\n  data_dir = {:?},\n  default_timeout_secs = {DEFAULT_TIMEOUT_SECS},\n  shell = \"sh\",\n  max_parallelism = {MAX_PARALLELISM},\n}}\n",
            data_dir_string,
        ),
    )
    .expect("config written");
    (config_path, data_dir)
}

fn doctor_command(config_path: &Path, output: &str) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_lattice"));
    command.arg("--config").arg(config_path).arg("doctor").arg("--output").arg(output);
    command
}

fn run_doctor(config_path: &Path, output: &str) -> std::process::Output {
    doctor_command(config_path, output).output().expect("doctor command runs")
}

fn run_doctor_with_environment_secret(config_path: &Path, output: &str) -> std::process::Output {
    doctor_command(config_path, output)
        .env(MISSING_DOCTOR_SECRET_KEY, SECRET_SENTINEL)
        .output()
        .expect("doctor command with environment secret runs")
}

fn doctor_trigger() -> TriggerDefinition {
    TriggerDefinition {
        id: DOCTOR_TRIGGER_ID.to_string(),
        workflow_name: DOCTOR_WORKFLOW_NAME.to_string(),
        enabled: true,
        kind: TriggerKind::Webhook(WebhookTrigger {
            method: DOCTOR_WEBHOOK_METHOD.to_string(),
            path: DOCTOR_WEBHOOK_PATH.to_string(),
            accepted_content_types: vec![DOCTOR_CONTENT_TYPE.to_string()],
            auth_policy: WebhookAuthPolicy::Bearer {
                secret_ref: SecretRef::new(DOCTOR_SECRET_PROVIDER, MISSING_DOCTOR_SECRET_KEY, None),
            },
        }),
        input_bindings: Vec::new(),
        overlap_policy: OverlapPolicy::Forbid,
        idempotency_policy: IdempotencyPolicy::None,
    }
}

// r[verify n8nclone.operator_diagnostics.doctor_command.healthy]
#[test]
fn doctor_cli_json_succeeds_for_healthy_config_and_store() {
    let temp_dir = tempfile::tempdir().expect("temp dir created");
    let (config_path, data_dir) = write_config(&temp_dir);
    Store::new(data_dir.join(STORE_FILE_NAME)).expect("store created");

    let output = run_doctor(&config_path, "json");
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let report: serde_json::Value = serde_json::from_str(&stdout).expect("doctor json parses");

    assert!(output.status.success());
    assert_eq!(report["status"].as_str(), Some("healthy"));
    assert!(stdout.contains("workflows"));
}

// r[verify n8nclone.operator_diagnostics.doctor_command.fatal]
#[test]
fn doctor_cli_missing_config_fails_safely() {
    let temp_dir = tempfile::tempdir().expect("temp dir created");
    let missing_config = temp_dir.path().join(CONFIG_FILE_NAME);

    let output = run_doctor(&missing_config, "text");
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");

    assert!(!output.status.success());
    assert!(stdout.contains("doctor status=fatal"));
    assert!(stdout.contains("config file not found"));
}

// r[verify n8nclone.operator_diagnostics.doctor_checks.config_store]
#[test]
fn doctor_cli_bad_nickel_and_missing_store_fail_without_creating_store() {
    let temp_dir = tempfile::tempdir().expect("temp dir created");
    let bad_config_path = temp_dir.path().join("bad.ncl");
    std::fs::write(&bad_config_path, format!("{{ data_dir = \"{SECRET_SENTINEL}\", broken = "))
        .expect("bad config written");
    let (config_path, data_dir) = write_config(&temp_dir);
    let store_path = data_dir.join(STORE_FILE_NAME);

    let bad_config_text_output = run_doctor(&bad_config_path, "text");
    let bad_config_json_output = run_doctor(&bad_config_path, "json");
    let missing_store_output = run_doctor(&config_path, "text");
    let bad_text_stdout = String::from_utf8(bad_config_text_output.stdout).expect("bad text stdout utf8");
    let bad_text_stderr = String::from_utf8(bad_config_text_output.stderr).expect("bad text stderr utf8");
    let bad_json_stdout = String::from_utf8(bad_config_json_output.stdout).expect("bad JSON stdout utf8");
    let bad_json_stderr = String::from_utf8(bad_config_json_output.stderr).expect("bad JSON stderr utf8");

    assert!(!bad_config_text_output.status.success());
    assert!(!bad_config_json_output.status.success());
    assert!(!missing_store_output.status.success());
    assert!(!bad_text_stdout.contains(SECRET_SENTINEL));
    assert!(!bad_text_stderr.contains(SECRET_SENTINEL));
    assert!(!bad_json_stdout.contains(SECRET_SENTINEL));
    assert!(!bad_json_stderr.contains(SECRET_SENTINEL));
    assert!(!store_path.exists());
}

// r[verify n8nclone.operator_diagnostics.doctor_checks.credentials]
// r[verify n8nclone.operator_diagnostics.doctor_checks.webhook_listener]
#[test]
fn doctor_cli_reports_missing_credential_provider_without_secret_values() {
    let temp_dir = tempfile::tempdir().expect("temp dir created");
    let (config_path, data_dir) = write_config(&temp_dir);
    let store = Store::new(data_dir.join(STORE_FILE_NAME)).expect("store created");
    let mut workflow = Workflow::new(DOCTOR_WORKFLOW_NAME, lattice::engine::workflow::WorkflowTimestamp::epoch());
    workflow.triggers.push(doctor_trigger());
    store.save_workflow(&workflow).expect("workflow with trigger saved");
    store.save_trigger(&workflow.triggers[0]).expect("trigger saved");
    drop(store);

    let output = run_doctor(&config_path, "text");
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");

    assert!(!output.status.success());
    assert!(stdout.contains("credentials"));
    assert!(stdout.contains("webhook_listener"));
}

#[test]
fn doctor_cli_never_discloses_a_present_environment_secret() {
    let temp_dir = tempfile::tempdir().expect("temp dir created");
    let (config_path, data_dir) = write_config(&temp_dir);
    let store = Store::new(data_dir.join(STORE_FILE_NAME)).expect("store created");
    let mut workflow = Workflow::new(DOCTOR_WORKFLOW_NAME, lattice::engine::workflow::WorkflowTimestamp::epoch());
    workflow.triggers.push(doctor_trigger());
    store.save_workflow(&workflow).expect("workflow with trigger saved");
    store.save_trigger(&workflow.triggers[0]).expect("trigger saved");
    drop(store);

    let text_output = run_doctor_with_environment_secret(&config_path, "text");
    let json_output = run_doctor_with_environment_secret(&config_path, "json");
    let text_stdout = String::from_utf8(text_output.stdout).expect("text stdout utf8");
    let text_stderr = String::from_utf8(text_output.stderr).expect("text stderr utf8");
    let json_stdout = String::from_utf8(json_output.stdout).expect("JSON stdout utf8");
    let json_stderr = String::from_utf8(json_output.stderr).expect("JSON stderr utf8");
    let report: serde_json::Value = serde_json::from_str(&json_stdout).expect("doctor JSON parses");

    assert!(text_output.status.success());
    assert!(json_output.status.success());
    assert_eq!(report["status"].as_str(), Some("healthy"));
    assert!(text_stdout.contains("credentials"));
    assert!(json_stdout.contains("credentials"));
    assert!(!text_stdout.contains(SECRET_SENTINEL));
    assert!(!text_stderr.contains(SECRET_SENTINEL));
    assert!(!json_stdout.contains(SECRET_SENTINEL));
    assert!(!json_stderr.contains(SECRET_SENTINEL));
}

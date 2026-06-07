//! Tests for operator doctor diagnostics.

use std::path::Path;
use std::process::Command;

use n8nclone::credentials::SecretRef;
use n8nclone::doctor;
use n8nclone::engine::workflow::Workflow;
use n8nclone::scheduler::OverlapPolicy;
use n8nclone::store::Store;
use n8nclone::triggers::IdempotencyPolicy;
use n8nclone::triggers::TriggerDefinition;
use n8nclone::triggers::TriggerKind;
use n8nclone::triggers::WebhookAuthPolicy;
use n8nclone::triggers::WebhookTrigger;
use pretty_assertions::assert_eq;

const CONFIG_FILE_NAME: &str = "config.ncl";
const STORE_FILE_NAME: &str = "n8nclone.redb";
const DATA_DIR_NAME: &str = "data";
const DEFAULT_TIMEOUT_SECS: u64 = 60;
const MAX_PARALLELISM: usize = 4;
const DOCTOR_WORKFLOW_NAME: &str = "doctor-workflow";
const DOCTOR_TRIGGER_ID: &str = "doctor-trigger";
const DOCTOR_WEBHOOK_PATH: &str = "/hooks/doctor";
const DOCTOR_WEBHOOK_METHOD: &str = "POST";
const DOCTOR_CONTENT_TYPE: &str = "application/json";
const DOCTOR_SECRET_PROVIDER: &str = "env";
const MISSING_DOCTOR_SECRET_KEY: &str = "N8NCLONE_DOCTOR_TEST_TOKEN_SHOULD_NOT_EXIST";
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

fn run_doctor(config_path: &Path, output: &str) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_n8nclone"))
        .arg("--config")
        .arg(config_path)
        .arg("doctor")
        .arg("--output")
        .arg(output)
        .output()
        .expect("doctor command runs")
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

// r[verify n8nclone.operator_diagnostics.doctor_core.healthy]
#[test]
fn doctor_core_reports_healthy_and_warning_summaries() {
    let healthy = doctor::evaluate_doctor_facts(&doctor::DoctorFacts::new(vec![
        doctor::DoctorFact::healthy(doctor::CHECK_CONFIG, "config ok"),
        doctor::DoctorFact::healthy(doctor::CHECK_STORE, "store ok"),
    ]));
    let warning = doctor::evaluate_doctor_facts(&doctor::DoctorFacts::new(vec![doctor::DoctorFact::warning(
        doctor::CHECK_WEBHOOK_LISTENER,
        "listener not configured",
    )]));

    assert_eq!(healthy.status, doctor::DoctorStatus::Healthy);
    assert_eq!(warning.status, doctor::DoctorStatus::Warning);
}

// r[verify n8nclone.operator_diagnostics.doctor_core.fatal]
// r[verify n8nclone.operator_diagnostics.doctor_redaction.text]
// r[verify n8nclone.operator_diagnostics.doctor_redaction.json]
#[test]
fn doctor_core_reports_fatal_findings_and_redacts_output() {
    let facts = doctor::DoctorFacts::new(vec![
        doctor::DoctorFact::fatal(doctor::CHECK_CREDENTIALS, format!("credential provider returned {SECRET_SENTINEL}"))
            .with_remediation(format!("remove Authorization: Bearer {SECRET_SENTINEL}"))
            .with_redaction(SECRET_SENTINEL),
    ]);

    let report = doctor::evaluate_doctor_facts(&facts);
    let text = doctor::render_doctor_text(&report);
    let json = doctor::render_doctor_json(&report);

    assert_eq!(report.status, doctor::DoctorStatus::Fatal);
    assert!(!text.contains(SECRET_SENTINEL));
    assert!(!json.contains(SECRET_SENTINEL));
    assert!(text.contains("[redacted]"));
    assert!(json.contains("[redacted]"));
}

// r[verify n8nclone.operator_diagnostics.doctor_command.healthy]
#[test]
fn doctor_cli_json_succeeds_for_healthy_config_and_store() {
    let temp_dir = tempfile::tempdir().expect("temp dir created");
    let (config_path, data_dir) = write_config(&temp_dir);
    Store::new(data_dir.join(STORE_FILE_NAME)).expect("store created");

    let output = run_doctor(&config_path, "json");
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let report: doctor::DoctorReport = serde_json::from_str(&stdout).expect("doctor json parses");

    assert!(output.status.success());
    assert_eq!(report.status, doctor::DoctorStatus::Healthy);
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
    std::fs::write(&bad_config_path, "{ data_dir = ").expect("bad config written");
    let (config_path, data_dir) = write_config(&temp_dir);
    let store_path = data_dir.join(STORE_FILE_NAME);

    let bad_config_output = run_doctor(&bad_config_path, "text");
    let missing_store_output = run_doctor(&config_path, "text");

    assert!(!bad_config_output.status.success());
    assert!(!missing_store_output.status.success());
    assert!(!store_path.exists());
}

// r[verify n8nclone.operator_diagnostics.doctor_checks.credentials]
// r[verify n8nclone.operator_diagnostics.doctor_checks.webhook_listener]
#[test]
fn doctor_cli_reports_missing_credential_provider_without_secret_values() {
    let temp_dir = tempfile::tempdir().expect("temp dir created");
    let (config_path, data_dir) = write_config(&temp_dir);
    let store = Store::new(data_dir.join(STORE_FILE_NAME)).expect("store created");
    let mut workflow = Workflow::new(DOCTOR_WORKFLOW_NAME);
    workflow.triggers.push(doctor_trigger());
    store.save_workflow(&workflow).expect("workflow with trigger saved");
    drop(store);

    let output = run_doctor(&config_path, "text");
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");

    assert!(!output.status.success());
    assert!(stdout.contains("credentials"));
    assert!(stdout.contains("webhook_listener"));
    assert!(!stdout.contains(SECRET_SENTINEL));
}

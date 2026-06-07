use std::collections::HashMap;
use std::collections::HashSet;
use std::path::Path;
use std::process::Command;

use n8nclone::credentials::MapCredentialProvider;
use n8nclone::credentials::SecretRef;
use n8nclone::engine::RunId;
use n8nclone::headless::HeadlessRunStatus;
use n8nclone::headless::run_admitted_trigger_plan;
use n8nclone::scheduler::OverlapPolicy;
use n8nclone::store::Store;
use n8nclone::triggers::IdempotencyPolicy;
use n8nclone::triggers::TriggerDefinition;
use n8nclone::triggers::TriggerInputBinding;
use n8nclone::triggers::TriggerKind;
use n8nclone::triggers::TriggerRunRecord;
use n8nclone::triggers::TriggerRunStatus;
use n8nclone::triggers::WebhookAuthPolicy;
use n8nclone::triggers::WebhookTrigger;
use n8nclone::webhooks;
use n8nclone::workflow_format::load_workflow_nickel_file;
use n8nclone::workflow_validation;
use pretty_assertions::assert_eq;

const TRIGGER_ID: &str = "new-lead";
const WORKFLOW_NAME: &str = "lead-workflow";
const WEBHOOK_METHOD: &str = "POST";
const WEBHOOK_PATH: &str = "/hooks/new-lead";
const MISSING_PATH: &str = "/hooks/missing";
const CONTENT_TYPE_JSON: &str = "application/json";
const CONTENT_TYPE_TEXT: &str = "text/plain";
const HEADER_CONTENT_TYPE: &str = "Content-Type";
const HEADER_AUTHORIZATION: &str = "Authorization";
const HEADER_IDEMPOTENCY_KEY: &str = "Idempotency-Key";
const HEADER_SOURCE: &str = "X-Source";
const QUERY_SOURCE: &str = "website";
const SECRET_PROVIDER: &str = "env";
const SECRET_KEY: &str = "WEBHOOK_SECRET";
const SECRET_VALUE: &str = "super-secret";
const BAD_TOKEN: &str = "bad-token";
const EVENT_KEY: &str = "event-1";
const PAYLOAD_SECRET: &str = "payload-secret";
const RAW_JSON_BODY: &[u8] = br#"{"name":"Ada","secret":"payload-secret"}"#;
const RAW_TEXT_BODY: &[u8] = b"plain text body";
const MALFORMED_JSON_BODY: &[u8] = br#"{"name":"Ada""#;
const LIMIT_BYTES: usize = 1024;
const RUN_ID_VALUE: u64 = 42;
const CURRENT_EPOCH_MS: i64 = 1_000;
const HISTORY_LIMIT: usize = 1;
const EXPECTED_TRIGGER_COUNT: u64 = 1;
const DEFAULT_TIMEOUT_SECS: u64 = 60;
const MAX_PARALLELISM: usize = 1;
const CLI_CONFIG_FILE_NAME: &str = "config.ncl";
const CLI_DATA_DIR_NAME: &str = "data";
const STORE_FILE_NAME: &str = "n8nclone.redb";
const COOKBOOK_WORKFLOW_PATH: &str = "examples/workflows/webhook-cookbook.ncl";
const COOKBOOK_WORKFLOW_NAME: &str = "webhook-cookbook";
const COOKBOOK_TRIGGER_ID: &str = "cookbook-lead";
const COOKBOOK_WEBHOOK_PATH: &str = "/hooks/cookbook-lead";
const COOKBOOK_SECRET_KEY: &str = "N8NCLONE_WEBHOOK_TOKEN";

fn limits() -> webhooks::RequestLimits {
    webhooks::RequestLimits {
        max_body_bytes: LIMIT_BYTES,
    }
}

fn secret_ref() -> SecretRef {
    SecretRef::new(SECRET_PROVIDER, SECRET_KEY, None)
}

fn provider_with_secret() -> MapCredentialProvider {
    let mut secrets = HashMap::new();
    secrets.insert(secret_ref().identity(), SECRET_VALUE.to_string());
    MapCredentialProvider::new(secrets)
}

fn cookbook_secret_ref() -> SecretRef {
    SecretRef::new(SECRET_PROVIDER, COOKBOOK_SECRET_KEY, None)
}

fn cookbook_provider_with_secret() -> MapCredentialProvider {
    let mut secrets = HashMap::new();
    secrets.insert(cookbook_secret_ref().identity(), SECRET_VALUE.to_string());
    MapCredentialProvider::new(secrets)
}

fn write_cli_config(temp_dir: &tempfile::TempDir) -> (std::path::PathBuf, std::path::PathBuf) {
    let data_dir = temp_dir.path().join(CLI_DATA_DIR_NAME);
    std::fs::create_dir_all(&data_dir).expect("data directory created");
    let config_path = temp_dir.path().join(CLI_CONFIG_FILE_NAME);
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

fn save_workflow_with_triggers(store: &Store, workflow: &n8nclone::engine::Workflow) {
    store.save_workflow(workflow).expect("workflow saves");
    for trigger in &workflow.triggers {
        store.save_trigger(trigger).expect("trigger saves");
    }
}

fn save_loaded_cookbook(store: &Store) -> n8nclone::engine::Workflow {
    let workflow = load_workflow_nickel_file(Path::new(COOKBOOK_WORKFLOW_PATH)).expect("cookbook workflow imports");
    save_workflow_with_triggers(store, &workflow);
    workflow
}

fn webhook_trigger(enabled: bool, auth_policy: WebhookAuthPolicy) -> TriggerDefinition {
    TriggerDefinition {
        id: TRIGGER_ID.to_string(),
        workflow_name: WORKFLOW_NAME.to_string(),
        enabled,
        kind: TriggerKind::Webhook(WebhookTrigger {
            method: WEBHOOK_METHOD.to_string(),
            path: WEBHOOK_PATH.to_string(),
            accepted_content_types: vec![CONTENT_TYPE_JSON.to_string()],
            auth_policy,
        }),
        input_bindings: Vec::<TriggerInputBinding>::new(),
        overlap_policy: OverlapPolicy::Forbid,
        idempotency_policy: IdempotencyPolicy::EventKey,
    }
}

fn valid_request() -> webhooks::RawRequest {
    request_with_parts(WEBHOOK_METHOD, WEBHOOK_PATH, CONTENT_TYPE_JSON, bearer(SECRET_VALUE), RAW_JSON_BODY)
}

fn valid_cookbook_request() -> webhooks::RawRequest {
    request_with_parts(WEBHOOK_METHOD, COOKBOOK_WEBHOOK_PATH, CONTENT_TYPE_JSON, bearer(SECRET_VALUE), RAW_JSON_BODY)
}

fn rejected_response_body(decision: &webhooks::Decision) -> String {
    let webhooks::Decision::Rejected(rejection) = decision else {
        panic!("decision must be rejected");
    };
    String::from_utf8(webhooks::rejected(rejection).body).expect("rejection response body is utf8")
}

fn request_with_parts(
    method: &str,
    path: &str,
    content_type: &str,
    authorization: String,
    body: &[u8],
) -> webhooks::RawRequest {
    webhooks::RawRequest {
        method: method.to_string(),
        target: format!("{path}?source={QUERY_SOURCE}"),
        headers: vec![
            (HEADER_CONTENT_TYPE.to_string(), content_type.to_string()),
            (HEADER_AUTHORIZATION.to_string(), authorization),
            (HEADER_IDEMPOTENCY_KEY.to_string(), EVENT_KEY.to_string()),
            (HEADER_SOURCE.to_string(), "header".to_string()),
        ],
        body: body.to_vec(),
    }
}

fn bearer(token: &str) -> String {
    format!("Bearer {token}")
}

fn empty_context() -> n8nclone::triggers::TriggerAdmissionContext {
    webhooks::admission_context(&[], CURRENT_EPOCH_MS)
}

fn started_record() -> TriggerRunRecord {
    TriggerRunRecord {
        trigger_id: TRIGGER_ID.to_string(),
        workflow_name: WORKFLOW_NAME.to_string(),
        run_id: Some(RunId(RUN_ID_VALUE)),
        idempotency_key: Some(EVENT_KEY.to_string()),
        status: TriggerRunStatus::Started,
        payload_summary: "[payload redacted]".to_string(),
        message: "trigger accepted".to_string(),
    }
}

// r[verify n8nclone.scheduling.webhook_listener.planning]
// r[verify n8nclone.scheduling.webhook_listener.planning.normalize]
#[test]
fn planner_normalizes_json_headers_query_and_matching_candidates() {
    let trigger = webhook_trigger(true, WebhookAuthPolicy::NoAuthLocalOnly);
    let request = valid_request();

    let plan = webhooks::plan_request(&request, &[trigger], limits()).expect("request plans");

    assert_eq!(plan.candidate_trigger_ids, vec![TRIGGER_ID.to_string()]);
    assert_eq!(plan.event.method.as_deref(), Some(WEBHOOK_METHOD));
    assert_eq!(plan.event.path.as_deref(), Some(WEBHOOK_PATH));
    assert_eq!(plan.event.content_type.as_deref(), Some(CONTENT_TYPE_JSON));
    assert_eq!(plan.event.query.get("source").map(String::as_str), Some(QUERY_SOURCE));
    assert_eq!(plan.event.idempotency_key.as_deref(), Some(EVENT_KEY));
    assert_eq!(
        plan.event.json_body.as_ref().and_then(|body| body.pointer("/name")).and_then(|name| name.as_str()),
        Some("Ada")
    );
}

// r[verify n8nclone.scheduling.webhook_listener.planning.bad_body]
#[test]
fn planner_rejects_malformed_json_but_keeps_non_json_raw_body_unparsed() {
    let trigger = webhook_trigger(true, WebhookAuthPolicy::NoAuthLocalOnly);
    let malformed =
        request_with_parts(WEBHOOK_METHOD, WEBHOOK_PATH, CONTENT_TYPE_JSON, bearer(SECRET_VALUE), MALFORMED_JSON_BODY);
    let raw_text =
        request_with_parts(WEBHOOK_METHOD, WEBHOOK_PATH, CONTENT_TYPE_TEXT, bearer(SECRET_VALUE), RAW_TEXT_BODY);

    let diagnostic = webhooks::plan_request(&malformed, std::slice::from_ref(&trigger), limits())
        .expect_err("malformed JSON rejected");
    let plan = webhooks::plan_request(&raw_text, &[trigger], limits()).expect("raw body plans");

    assert_eq!(diagnostic.code, webhooks::PlanningCode::MalformedJson);
    assert_eq!(plan.event.json_body, None);
    assert_eq!(plan.event.raw_body, RAW_TEXT_BODY);
}

// r[verify n8nclone.scheduling.webhook_listener.responses.accepted]
// r[verify n8nclone.scheduling.webhook_listener.responses.rejected]
#[test]
fn response_mapping_returns_stable_json_without_payload_or_secret_material() {
    let accepted = webhooks::accepted(TRIGGER_ID, RunId(RUN_ID_VALUE));
    let rejection = webhooks::rejected_for_code(webhooks::RejectionCode::InvalidAuth);
    let accepted_body = String::from_utf8(accepted.body).expect("accepted response utf8");
    let rejected_body = String::from_utf8(rejection.body).expect("rejected response utf8");

    assert!(accepted_body.contains(TRIGGER_ID));
    assert!(accepted_body.contains(&RUN_ID_VALUE.to_string()));
    assert!(rejected_body.contains("invalid_auth"));
    assert!(!accepted_body.contains(SECRET_VALUE));
    assert!(!accepted_body.contains(PAYLOAD_SECRET));
    assert!(!rejected_body.contains(SECRET_VALUE));
    assert!(!rejected_body.contains(PAYLOAD_SECRET));
}

// r[verify n8nclone.scheduling.webhook_listener.admission.valid]
// r[verify n8nclone.scheduling.webhook_listener.admission.duplicate]
#[test]
fn admission_accepts_valid_request_and_rejects_duplicate_idempotency() {
    let trigger = webhook_trigger(true, WebhookAuthPolicy::Bearer {
        secret_ref: secret_ref(),
    });
    let provider = provider_with_secret();
    let duplicate_context = webhooks::admission_context(&[started_record()], CURRENT_EPOCH_MS);

    let accepted = webhooks::decide_request(
        &valid_request(),
        std::slice::from_ref(&trigger),
        &provider,
        &empty_context(),
        limits(),
    );
    let duplicate = webhooks::decide_request(&valid_request(), &[trigger], &provider, &duplicate_context, limits());

    assert!(matches!(accepted, webhooks::Decision::Admitted(_)));
    assert!(
        matches!(duplicate, webhooks::Decision::Rejected(rejection) if rejection.code == webhooks::RejectionCode::DuplicateIdempotencyKey)
    );
}

// r[verify n8nclone.scheduling.webhook_listener.admission.redaction]
#[test]
fn auth_failure_response_and_history_omit_secrets_headers_and_raw_payloads() {
    let trigger = webhook_trigger(true, WebhookAuthPolicy::Bearer {
        secret_ref: secret_ref(),
    });
    let provider = provider_with_secret();
    let request = request_with_parts(WEBHOOK_METHOD, WEBHOOK_PATH, CONTENT_TYPE_JSON, bearer(BAD_TOKEN), RAW_JSON_BODY);

    let decision = webhooks::decide_request(&request, &[trigger], &provider, &empty_context(), limits());
    let webhooks::Decision::Rejected(rejection) = decision else {
        panic!("auth failure must reject");
    };
    let response_body = String::from_utf8(webhooks::rejected(&rejection).body).expect("response utf8");
    let record = webhooks::run_record_for_rejection(&rejection).expect("auth rejection records history");
    let record_debug = format!("{record:?}");

    assert_eq!(rejection.code, webhooks::RejectionCode::InvalidAuth);
    assert!(!response_body.contains(SECRET_VALUE));
    assert!(!response_body.contains(BAD_TOKEN));
    assert!(!response_body.contains(PAYLOAD_SECRET));
    assert!(!record_debug.contains(SECRET_VALUE));
    assert!(!record_debug.contains(HEADER_AUTHORIZATION));
    assert!(!record_debug.contains(PAYLOAD_SECRET));
}

// r[verify n8nclone.scheduling.webhook_listener.process.missing]
#[test]
fn missing_path_disabled_bad_method_bad_content_and_malformed_body_reject_without_runs() {
    let store = Store::new(tempfile::tempdir().expect("tempdir").path().join("webhooks.redb")).expect("store opens");
    let provider = provider_with_secret();
    let trigger = webhook_trigger(true, WebhookAuthPolicy::Bearer {
        secret_ref: secret_ref(),
    });
    let disabled = webhook_trigger(false, WebhookAuthPolicy::Bearer {
        secret_ref: secret_ref(),
    });
    let missing =
        request_with_parts(WEBHOOK_METHOD, MISSING_PATH, CONTENT_TYPE_JSON, bearer(SECRET_VALUE), RAW_JSON_BODY);
    let bad_method = request_with_parts("GET", WEBHOOK_PATH, CONTENT_TYPE_JSON, bearer(SECRET_VALUE), RAW_JSON_BODY);
    let bad_content =
        request_with_parts(WEBHOOK_METHOD, WEBHOOK_PATH, CONTENT_TYPE_TEXT, bearer(SECRET_VALUE), RAW_TEXT_BODY);
    let malformed =
        request_with_parts(WEBHOOK_METHOD, WEBHOOK_PATH, CONTENT_TYPE_JSON, bearer(SECRET_VALUE), MALFORMED_JSON_BODY);

    let missing_decision =
        webhooks::decide_request(&missing, std::slice::from_ref(&trigger), &provider, &empty_context(), limits());
    let disabled_decision = webhooks::decide_request(
        &valid_request(),
        std::slice::from_ref(&disabled),
        &provider,
        &empty_context(),
        limits(),
    );
    let method_decision =
        webhooks::decide_request(&bad_method, std::slice::from_ref(&trigger), &provider, &empty_context(), limits());
    let content_decision =
        webhooks::decide_request(&bad_content, std::slice::from_ref(&trigger), &provider, &empty_context(), limits());
    let body_decision = webhooks::decide_request(&malformed, &[trigger], &provider, &empty_context(), limits());
    let history = store.list_runs(HISTORY_LIMIT).expect("run history lists");

    assert!(
        matches!(missing_decision, webhooks::Decision::Rejected(rejection) if rejection.code == webhooks::RejectionCode::NotFound)
    );
    assert!(
        matches!(disabled_decision, webhooks::Decision::Rejected(rejection) if rejection.code == webhooks::RejectionCode::NotFound)
    );
    assert!(
        matches!(method_decision, webhooks::Decision::Rejected(rejection) if rejection.code == webhooks::RejectionCode::MethodMismatch)
    );
    assert!(
        matches!(content_decision, webhooks::Decision::Rejected(rejection) if rejection.code == webhooks::RejectionCode::UnsupportedContentType)
    );
    assert!(
        matches!(body_decision, webhooks::Decision::Rejected(rejection) if rejection.code == webhooks::RejectionCode::MalformedJson)
    );
    assert!(history.is_empty());
}

// r[verify n8nclone.tui_ux.webhook_listener.cli.dry_run]
#[test]
fn dry_run_reports_safe_status_and_redacted_credential_diagnostics() {
    let trigger = webhook_trigger(true, WebhookAuthPolicy::Bearer {
        secret_ref: secret_ref(),
    });
    let provider = MapCredentialProvider::new(HashMap::new());
    let report = webhooks::dry_run_report(
        webhooks::DEFAULT_BIND_ADDR,
        &[trigger],
        &HashSet::from([WORKFLOW_NAME.to_string()]),
        &provider,
    );
    let output = webhooks::render_dry_run_report(&report);

    assert_eq!(report.loaded_trigger_count, EXPECTED_TRIGGER_COUNT);
    assert_eq!(report.enabled_trigger_count, EXPECTED_TRIGGER_COUNT);
    assert!(!report.diagnostics.is_empty());
    assert!(output.contains("status=dry-run-blocked"));
    assert!(!output.contains(SECRET_KEY));
    assert!(!output.contains(SECRET_VALUE));
}

// r[verify n8nclone.workflow_format.webhook_cookbook_example.import]
// r[verify n8nclone.scheduling.webhook_cookbook_operator_flow.dry_run]
#[test]
fn webhook_cookbook_import_persists_trigger_and_dry_runs() {
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let (config_path, data_dir) = write_cli_config(&temp_dir);
    let status = Command::new(env!("CARGO_BIN_EXE_n8nclone"))
        .arg("--config")
        .arg(&config_path)
        .arg("import")
        .arg(COOKBOOK_WORKFLOW_PATH)
        .status()
        .expect("cookbook import command runs");
    let store = Store::new(data_dir.join(STORE_FILE_NAME)).expect("store opens");
    let workflow = load_workflow_nickel_file(Path::new(COOKBOOK_WORKFLOW_PATH)).expect("cookbook workflow imports");
    let diagnostics = workflow_validation::validate_workflow(&workflow);
    let triggers = store.list_triggers().expect("cookbook triggers list");

    let provider = cookbook_provider_with_secret();
    let report = webhooks::dry_run_report(
        webhooks::DEFAULT_BIND_ADDR,
        &triggers,
        &HashSet::from([COOKBOOK_WORKFLOW_NAME.to_string()]),
        &provider,
    );

    assert!(status.success());
    assert_eq!(workflow.meta.name, COOKBOOK_WORKFLOW_NAME);
    assert!(diagnostics.is_empty());
    assert_eq!(triggers.len(), 1);
    assert_eq!(triggers[0].id, COOKBOOK_TRIGGER_ID);
    assert!(report.diagnostics.is_empty());
    assert_eq!(report.enabled_trigger_count, EXPECTED_TRIGGER_COUNT);
}

// r[verify n8nclone.scheduling.webhook_cookbook_operator_flow.accepted]
#[tokio::test]
async fn webhook_cookbook_valid_request_starts_one_run_and_records_safe_history() {
    let store = Store::new(tempfile::tempdir().expect("tempdir").path().join("cookbook.redb")).expect("store opens");
    save_loaded_cookbook(&store);
    let triggers = store.list_triggers().expect("cookbook triggers list");
    let provider = cookbook_provider_with_secret();
    let decision =
        webhooks::decide_request(&valid_cookbook_request(), &triggers, &provider, &empty_context(), limits());
    let webhooks::Decision::Admitted(plan) = decision else {
        panic!("valid cookbook request must admit");
    };

    let summary = run_admitted_trigger_plan(&store, &plan).await.expect("cookbook trigger starts workflow");
    let history = store.list_trigger_runs(HISTORY_LIMIT).expect("trigger history lists");

    assert_eq!(summary.status, HeadlessRunStatus::Succeeded);
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].trigger_id, COOKBOOK_TRIGGER_ID);
    assert_eq!(history[0].status, TriggerRunStatus::Started);
    assert!(!format!("{history:?}").contains(SECRET_VALUE));
    assert!(!format!("{history:?}").contains(PAYLOAD_SECRET));
}

// r[verify n8nclone.scheduling.webhook_cookbook_operator_flow.rejected]
// r[verify n8nclone.credentials.webhook_cookbook_secret_safety.references]
// r[verify n8nclone.credentials.webhook_cookbook_secret_safety.redaction]
#[test]
fn webhook_cookbook_rejected_requests_do_not_start_runs_or_expose_secrets() {
    let store = Store::new(tempfile::tempdir().expect("tempdir").path().join("cookbook.redb")).expect("store opens");
    let workflow_text = std::fs::read_to_string(COOKBOOK_WORKFLOW_PATH).expect("cookbook workflow text reads");
    let docs = std::fs::read_to_string("examples/README.md").expect("examples docs read");
    save_loaded_cookbook(&store);
    let triggers = store.list_triggers().expect("cookbook triggers list");
    let provider = cookbook_provider_with_secret();
    let missing =
        request_with_parts(WEBHOOK_METHOD, MISSING_PATH, CONTENT_TYPE_JSON, bearer(SECRET_VALUE), RAW_JSON_BODY);
    let invalid_auth =
        request_with_parts(WEBHOOK_METHOD, COOKBOOK_WEBHOOK_PATH, CONTENT_TYPE_JSON, bearer(BAD_TOKEN), RAW_JSON_BODY);
    let malformed = request_with_parts(
        WEBHOOK_METHOD,
        COOKBOOK_WEBHOOK_PATH,
        CONTENT_TYPE_JSON,
        bearer(SECRET_VALUE),
        MALFORMED_JSON_BODY,
    );

    let missing_decision = webhooks::decide_request(&missing, &triggers, &provider, &empty_context(), limits());
    let auth_decision = webhooks::decide_request(&invalid_auth, &triggers, &provider, &empty_context(), limits());
    let malformed_decision = webhooks::decide_request(&malformed, &triggers, &provider, &empty_context(), limits());
    let runs = store.list_runs(HISTORY_LIMIT).expect("runs list");
    let public_rejections = format!(
        "{}\n{}\n{}",
        rejected_response_body(&missing_decision),
        rejected_response_body(&auth_decision),
        rejected_response_body(&malformed_decision),
    );
    let auth_record_debug = match &auth_decision {
        webhooks::Decision::Rejected(rejection) => webhooks::run_record_for_rejection(rejection)
            .map(|record| format!("{record:?}"))
            .unwrap_or_default(),
        webhooks::Decision::Admitted(_) => String::new(),
    };

    assert!(
        matches!(missing_decision, webhooks::Decision::Rejected(rejection) if rejection.code == webhooks::RejectionCode::NotFound)
    );
    assert!(
        matches!(auth_decision, webhooks::Decision::Rejected(rejection) if rejection.code == webhooks::RejectionCode::InvalidAuth)
    );
    assert!(
        matches!(malformed_decision, webhooks::Decision::Rejected(rejection) if rejection.code == webhooks::RejectionCode::MalformedJson)
    );
    assert!(runs.is_empty());
    assert!(workflow_text.contains(COOKBOOK_SECRET_KEY));
    assert!(docs.contains(COOKBOOK_SECRET_KEY));
    assert!(!workflow_text.contains(SECRET_VALUE));
    assert!(!docs.contains(SECRET_VALUE));
    assert!(!public_rejections.contains(SECRET_VALUE));
    assert!(!public_rejections.contains(BAD_TOKEN));
    assert!(!public_rejections.contains(PAYLOAD_SECRET));
    assert!(!auth_record_debug.contains(SECRET_VALUE));
    assert!(!auth_record_debug.contains(BAD_TOKEN));
    assert!(!auth_record_debug.contains(PAYLOAD_SECRET));
}

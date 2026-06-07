//! Tests for external trigger and webhook admission.

use std::collections::HashMap;
use std::collections::HashSet;

use n8nclone::config::AppConfig;
use n8nclone::credentials::MapCredentialProvider;
use n8nclone::credentials::SecretRef;
use n8nclone::engine::RunId;
use n8nclone::engine::ValueData;
use n8nclone::engine::Workflow;
use n8nclone::headless::HeadlessRunStatus;
use n8nclone::headless::run_admitted_trigger_plan;
use n8nclone::scheduler::MissedRunPolicy;
use n8nclone::scheduler::OverlapPolicy;
use n8nclone::scheduler::RecurrenceRule;
use n8nclone::scheduler::ScheduleDefinition;
use n8nclone::store::Store;
use n8nclone::triggers::IdempotencyPolicy;
use n8nclone::triggers::TriggerAdmissionContext;
use n8nclone::triggers::TriggerDefinition;
use n8nclone::triggers::TriggerDiagnosticCode;
use n8nclone::triggers::TriggerEvent;
use n8nclone::triggers::TriggerInputBinding;
use n8nclone::triggers::TriggerKind;
use n8nclone::triggers::TriggerPayloadSource;
use n8nclone::triggers::TriggerRunStatus;
use n8nclone::triggers::WebhookAuthPolicy;
use n8nclone::triggers::WebhookTrigger;
use n8nclone::triggers::admit_trigger_event;
use n8nclone::triggers::bind_trigger_payload;
use n8nclone::triggers::resolve_webhook_auth_secret;
use n8nclone::triggers::schedule_due_plan_from_trigger;
use n8nclone::triggers::schedule_trigger_from_schedule;
use n8nclone::triggers::scoped_idempotency_key;
use n8nclone::triggers::trigger_run_record_started;
use n8nclone::triggers::validate_trigger_definition;
use n8nclone::triggers::webhook_signature;
use n8nclone::tui::App;
use n8nclone::workflow_format::load_workflow_nickel_file;
use n8nclone::workflow_format::workflow_to_nickel;
use n8nclone::workflow_format::workflow_to_portable;
use n8nclone::workflow_validation;
use n8nclone::workflow_validation::WorkflowDiagnosticCode;
use pretty_assertions::assert_eq;

const TRIGGER_ID: &str = "new-lead";
const WORKFLOW_NAME: &str = "lead-workflow";
const MISSING_WORKFLOW: &str = "missing-workflow";
const WEBHOOK_METHOD: &str = "POST";
const WEBHOOK_PATH: &str = "/hooks/new-lead";
const CONTENT_TYPE_JSON: &str = "application/json";
const INPUT_LABEL_NAME: &str = "lead_name";
const INPUT_LABEL_SOURCE: &str = "source";
const INPUT_LABEL_HEADER: &str = "header_source";
const INPUT_LABEL_RAW: &str = "raw_body";
const JSON_POINTER_NAME: &str = "/name";
const QUERY_SOURCE: &str = "source";
const HEADER_SOURCE: &str = "x-source";
const SECRET_PROVIDER: &str = "env";
const SECRET_KEY: &str = "WEBHOOK_SECRET";
const SECRET_VALUE: &str = "super-secret";
const SIGNATURE_HEADER: &str = "x-signature";
const EVENT_KEY: &str = "event-1";
const CURRENT_EPOCH_MS: i64 = 1_000;
const NEXT_RUN_EPOCH_MS: i64 = 1_000;
const INTERVAL_SECS: u64 = 60;
const EXPECTED_BINDING_COUNT: usize = 4;
const STORE_FILE: &str = "triggers.redb";
const WORKFLOW_FILE: &str = "workflow.ncl";
const STARTED_RUN_ID: u64 = 7;

fn workflow_names() -> HashSet<String> {
    HashSet::from([WORKFLOW_NAME.to_string()])
}

fn secret_ref() -> SecretRef {
    SecretRef::new(SECRET_PROVIDER, SECRET_KEY, None)
}

fn webhook_trigger(auth_policy: WebhookAuthPolicy) -> TriggerDefinition {
    TriggerDefinition {
        id: TRIGGER_ID.to_string(),
        workflow_name: WORKFLOW_NAME.to_string(),
        enabled: true,
        kind: TriggerKind::Webhook(WebhookTrigger {
            method: WEBHOOK_METHOD.to_string(),
            path: WEBHOOK_PATH.to_string(),
            accepted_content_types: vec![CONTENT_TYPE_JSON.to_string()],
            auth_policy,
        }),
        input_bindings: vec![
            TriggerInputBinding {
                input_label: INPUT_LABEL_NAME.to_string(),
                source: TriggerPayloadSource::JsonPointer(JSON_POINTER_NAME.to_string()),
                type_tag: "json".to_string(),
                required: true,
            },
            TriggerInputBinding {
                input_label: INPUT_LABEL_SOURCE.to_string(),
                source: TriggerPayloadSource::QueryParam(QUERY_SOURCE.to_string()),
                type_tag: "string".to_string(),
                required: true,
            },
            TriggerInputBinding {
                input_label: INPUT_LABEL_HEADER.to_string(),
                source: TriggerPayloadSource::Header(HEADER_SOURCE.to_string()),
                type_tag: "string".to_string(),
                required: true,
            },
            TriggerInputBinding {
                input_label: INPUT_LABEL_RAW.to_string(),
                source: TriggerPayloadSource::RawBody,
                type_tag: "bytes".to_string(),
                required: false,
            },
        ],
        overlap_policy: OverlapPolicy::Forbid,
        idempotency_policy: IdempotencyPolicy::EventKey,
    }
}

fn webhook_event(secret: Option<&str>) -> TriggerEvent {
    let raw_body = br#"{"name":"Ada"}"#.to_vec();
    let mut headers = HashMap::new();
    headers.insert("content-type".to_string(), CONTENT_TYPE_JSON.to_string());
    headers.insert(SIGNATURE_HEADER.to_string(), webhook_signature(SECRET_VALUE, &raw_body));
    headers.insert(HEADER_SOURCE.to_string(), "header".to_string());
    let mut query = HashMap::new();
    query.insert(QUERY_SOURCE.to_string(), "website".to_string());
    TriggerEvent {
        method: Some(WEBHOOK_METHOD.to_string()),
        path: Some(WEBHOOK_PATH.to_string()),
        content_type: Some(CONTENT_TYPE_JSON.to_string()),
        headers,
        query,
        json_body: Some(serde_json::json!({ "name": "Ada" })),
        raw_body,
        idempotency_key: Some(EVENT_KEY.to_string()),
        resolved_auth_secret: secret.map(str::to_string),
    }
}

fn empty_context() -> TriggerAdmissionContext {
    TriggerAdmissionContext {
        current_epoch_ms: CURRENT_EPOCH_MS,
        running_trigger_ids: HashSet::new(),
        accepted_idempotency_keys: HashSet::new(),
    }
}

// r[verify n8nclone.scheduling.triggers.model.save]
// r[verify n8nclone.scheduling.triggers.model.missing_workflow]
#[test]
fn trigger_validation_accepts_saved_workflow_and_rejects_missing_workflow() {
    let valid = webhook_trigger(WebhookAuthPolicy::NoAuthLocalOnly);
    let mut missing = valid.clone();
    missing.workflow_name = MISSING_WORKFLOW.to_string();
    let mut workflow = Workflow::new(WORKFLOW_NAME.to_string());
    workflow.triggers.push(missing.clone());

    assert!(validate_trigger_definition(&valid, &workflow_names()).is_empty());
    let diagnostics = validate_trigger_definition(&missing, &workflow_names());
    let workflow_diagnostics = workflow_validation::validate_workflow(&workflow);
    assert!(diagnostics.iter().any(|diagnostic| diagnostic.code == TriggerDiagnosticCode::MissingWorkflow));
    assert!(
        workflow_diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == WorkflowDiagnosticCode::InvalidTriggerConfig)
    );
}

// r[verify n8nclone.workflow_execution.triggers.payload_binding.json]
// r[verify n8nclone.workflow_execution.triggers.payload_binding.missing]
// r[verify n8nclone.workflow_execution.triggers.payload_binding.type_mismatch]
#[test]
fn payload_binding_supports_json_query_header_and_raw_body_failures() {
    let trigger = webhook_trigger(WebhookAuthPolicy::NoAuthLocalOnly);
    let event = webhook_event(None);
    let values = bind_trigger_payload(&trigger, &event).expect("payload binds");
    let mut missing_event = event.clone();
    missing_event.json_body = Some(serde_json::json!({}));
    let mut type_mismatch = trigger.clone();
    type_mismatch.input_bindings[0].type_tag = "string".to_string();

    assert_eq!(values.len(), EXPECTED_BINDING_COUNT);
    assert!(matches!(values[INPUT_LABEL_NAME].data, ValueData::Json(_)));
    assert!(matches!(values[INPUT_LABEL_SOURCE].data, ValueData::String(_)));
    assert!(matches!(values[INPUT_LABEL_HEADER].data, ValueData::String(_)));
    assert!(matches!(values[INPUT_LABEL_RAW].data, ValueData::Bytes(_)));
    let missing_diagnostics = bind_trigger_payload(&trigger, &missing_event).expect_err("missing required field");
    let mismatch_diagnostics = bind_trigger_payload(&type_mismatch, &event).expect_err("type mismatch");

    assert!(
        missing_diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == TriggerDiagnosticCode::MissingPayloadField)
    );
    assert!(
        mismatch_diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == TriggerDiagnosticCode::PayloadTypeMismatch)
    );
}

// r[verify n8nclone.scheduling.triggers.admission.enabled]
// r[verify n8nclone.scheduling.triggers.admission.idempotency]
// r[verify n8nclone.scheduling.triggers.admission.overlap]
#[test]
fn admission_accepts_valid_event_and_rejects_idempotent_or_overlapping_events() {
    let trigger = webhook_trigger(WebhookAuthPolicy::NoAuthLocalOnly);
    let event = webhook_event(None);
    let accepted = admit_trigger_event(&trigger, &event, &empty_context()).expect("valid event admitted");
    let mut duplicate_context = empty_context();
    duplicate_context.accepted_idempotency_keys.insert(scoped_idempotency_key(TRIGGER_ID, EVENT_KEY));
    let mut running_context = empty_context();
    running_context.running_trigger_ids.insert(TRIGGER_ID.to_string());

    assert_eq!(accepted.trigger_id, TRIGGER_ID);
    assert_eq!(accepted.idempotency_key.as_deref(), Some(EVENT_KEY));
    assert!(accepted.payload_summary.contains("redacted"));
    let duplicate = admit_trigger_event(&trigger, &event, &duplicate_context).expect_err("duplicate rejected");
    let overlapping = admit_trigger_event(&trigger, &event, &running_context).expect_err("overlap rejected");
    assert!(duplicate.iter().any(|diagnostic| diagnostic.code == TriggerDiagnosticCode::DuplicateIdempotencyKey));
    assert!(overlapping.iter().any(|diagnostic| diagnostic.code == TriggerDiagnosticCode::OverlapPrevented));
}

// r[verify n8nclone.scheduling.triggers.webhook.method]
// r[verify n8nclone.scheduling.triggers.webhook.content_type]
// r[verify n8nclone.credentials.webhook_auth.redacted]
#[test]
fn webhook_admission_checks_method_content_type_and_redacted_auth() {
    let trigger = webhook_trigger(WebhookAuthPolicy::Blake3Signature {
        secret_ref: secret_ref(),
        header: SIGNATURE_HEADER.to_string(),
    });
    let mut event = webhook_event(Some(SECRET_VALUE));
    assert!(admit_trigger_event(&trigger, &event, &empty_context()).is_ok());

    event.method = Some("GET".to_string());
    event.content_type = Some("text/plain".to_string());
    event.resolved_auth_secret = Some("wrong-secret".to_string());
    let diagnostics = admit_trigger_event(&trigger, &event, &empty_context()).expect_err("webhook rejected");
    let debug = format!("{diagnostics:?}");

    assert!(diagnostics.iter().any(|diagnostic| diagnostic.code == TriggerDiagnosticCode::MethodMismatch));
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == TriggerDiagnosticCode::UnsupportedContentType)
    );
    assert!(diagnostics.iter().any(|diagnostic| diagnostic.code == TriggerDiagnosticCode::InvalidAuth));
    assert!(!debug.contains(SECRET_VALUE));
}

// r[verify n8nclone.credentials.webhook_auth.provider]
// r[verify n8nclone.credentials.webhook_auth.missing]
#[test]
fn webhook_auth_resolves_secrets_through_credential_provider() {
    let mut secrets = HashMap::new();
    secrets.insert(secret_ref().identity(), SECRET_VALUE.to_string());
    let provider = MapCredentialProvider::new(secrets);
    let policy = WebhookAuthPolicy::Bearer {
        secret_ref: secret_ref(),
    };
    let missing_policy = WebhookAuthPolicy::Bearer {
        secret_ref: SecretRef::new(SECRET_PROVIDER, "MISSING", None),
    };

    assert_eq!(
        resolve_webhook_auth_secret(&provider, &policy).expect("secret resolves").as_deref(),
        Some(SECRET_VALUE),
    );
    assert_eq!(
        resolve_webhook_auth_secret(&provider, &missing_policy).expect_err("missing secret"),
        TriggerDiagnosticCode::MissingAuthSecret,
    );
}

// r[verify n8nclone.scheduling.triggers.scheduler_compat]
#[test]
fn schedule_trigger_preserves_schedule_semantics() {
    let schedule = ScheduleDefinition {
        id: TRIGGER_ID.to_string(),
        workflow_name: WORKFLOW_NAME.to_string(),
        recurrence: RecurrenceRule::EverySeconds {
            interval_secs: INTERVAL_SECS,
        },
        enabled: true,
        overlap_policy: OverlapPolicy::Forbid,
        missed_run_policy: MissedRunPolicy::RunOnce,
        next_run_epoch_ms: NEXT_RUN_EPOCH_MS,
    };
    let trigger = schedule_trigger_from_schedule(&schedule);
    let due = schedule_due_plan_from_trigger(&trigger, NEXT_RUN_EPOCH_MS, &HashSet::new()).expect("schedule due");

    assert_eq!(trigger.workflow_name, WORKFLOW_NAME);
    assert_eq!(due.schedule_id, TRIGGER_ID);
}

// r[verify n8nclone.workflow_format.triggers.nickel]
#[test]
fn workflow_format_round_trips_triggers_through_portable_and_nickel() {
    let mut workflow = Workflow::new(WORKFLOW_NAME.to_string());
    workflow.triggers.push(webhook_trigger(WebhookAuthPolicy::NoAuthLocalOnly));
    let portable = workflow_to_portable(&workflow);
    let nickel = workflow_to_nickel(&workflow);
    let tempdir = tempfile::tempdir().expect("tempdir");
    let path = tempdir.path().join(WORKFLOW_FILE);
    std::fs::write(&path, nickel).expect("write nickel workflow");
    let loaded = load_workflow_nickel_file(&path).expect("load workflow");

    assert_eq!(portable.triggers.len(), 1);
    assert_eq!(loaded.triggers.len(), 1);
    assert_eq!(loaded.triggers[0].id, TRIGGER_ID);
}

// r[verify n8nclone.scheduling.triggers.model.persist]
// r[verify n8nclone.scheduling.triggers.history]
#[test]
fn store_persists_triggers_and_trigger_history() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let store = Store::new(tempdir.path().join(STORE_FILE)).expect("open store");
    let trigger = webhook_trigger(WebhookAuthPolicy::NoAuthLocalOnly);
    let plan = admit_trigger_event(&trigger, &webhook_event(None), &empty_context()).expect("event admitted");
    let record = trigger_run_record_started(&plan, RunId(STARTED_RUN_ID));

    store.save_trigger(&trigger).expect("save trigger");
    store.save_trigger_run(&record).expect("save trigger run");
    let loaded = store.load_trigger(TRIGGER_ID).expect("load trigger").expect("trigger exists");
    let history = store.list_trigger_runs(1).expect("list trigger history");

    assert_eq!(loaded.id, TRIGGER_ID);
    assert_eq!(history[0].status, TriggerRunStatus::Started);
    assert_eq!(history[0].run_id, Some(RunId(STARTED_RUN_ID)));
}

// r[verify n8nclone.scheduling.triggers.webhook.headless_run]
// r[verify n8nclone.scheduling.triggers.history]
#[tokio::test]
async fn admitted_trigger_plan_starts_headless_run_and_records_history() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let store = Store::new(tempdir.path().join(STORE_FILE)).expect("open store");
    let workflow = Workflow::new(WORKFLOW_NAME.to_string());
    let trigger = webhook_trigger(WebhookAuthPolicy::NoAuthLocalOnly);
    let plan = admit_trigger_event(&trigger, &webhook_event(None), &empty_context()).expect("event admitted");
    store.save_workflow(&workflow).expect("save workflow");

    let summary = run_admitted_trigger_plan(&store, &plan).await.expect("headless run starts");
    let history = store.list_trigger_runs(1).expect("list trigger history");

    assert_eq!(summary.workflow_name, WORKFLOW_NAME);
    assert_eq!(summary.status, HeadlessRunStatus::Succeeded);
    assert_eq!(history[0].status, TriggerRunStatus::Started);
    assert_eq!(history[0].run_id, Some(summary.run_id));
}

// r[verify n8nclone.tui_ux.triggers.manage]
#[test]
fn tui_exposes_minimal_trigger_status_lines() {
    let mut workflow = Workflow::new(WORKFLOW_NAME.to_string());
    workflow.triggers.push(webhook_trigger(WebhookAuthPolicy::NoAuthLocalOnly));
    let mut app = App::new(AppConfig::default());
    app.workflow = workflow;
    let lines = app.trigger_status_lines();

    assert_eq!(lines.len(), 1);
    assert!(lines[0].contains(TRIGGER_ID));
    assert!(lines[0].contains("enabled"));
}

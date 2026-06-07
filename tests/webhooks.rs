use std::collections::HashMap;
use std::collections::HashSet;

use n8nclone::credentials::MapCredentialProvider;
use n8nclone::credentials::SecretRef;
use n8nclone::engine::RunId;
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

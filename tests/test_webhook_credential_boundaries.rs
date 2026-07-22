use lattice::credentials::MapCredentialProvider;
use lattice::credentials::SecretRef;
use lattice::headless::HeadlessRunStatus;
use lattice::headless::run_admitted_trigger_plan;
use lattice::scheduler::OverlapPolicy;
use lattice::store::Store;
use lattice::triggers::IdempotencyPolicy;
use lattice::triggers::TriggerDefinition;
use lattice::triggers::TriggerInputBinding;
use lattice::triggers::TriggerKind;
use lattice::triggers::TriggerRunStatus;
use lattice::triggers::WebhookAuthPolicy;
use lattice::triggers::WebhookTrigger;
use lattice::triggers::credential_ref_fingerprint;
use lattice::webhooks;

const TRIGGER_ID: &str = "credential-webhook";
const WORKFLOW_NAME: &str = "credential-webhook-workflow";
const NODE_NAME: &str = "credential-webhook-node";
const WEBHOOK_METHOD: &str = "POST";
const WEBHOOK_PATH: &str = "/hooks/credential-webhook";
const CONTENT_TYPE_JSON: &str = "application/json";
const SECRET_PROVIDER: &str = "env";
const SECRET_KEY: &str = "WEBHOOK_CREDENTIAL_TOKEN";
const SECRET_VALUE: &str = "webhook-persistence-secret";
const BAD_TOKEN: &str = "invalid-webhook-token";
const PAYLOAD_SECRET: &str = "private-payload-value";
const RAW_BODY: &[u8] = br#"{"name":"Ada","secret":"private-payload-value"}"#;
const EVENT_KEY: &str = "credential-event";
const STORE_FILE_NAME: &str = "webhook-credential.redb";
const HISTORY_LIMIT: usize = 16;
const CURRENT_EPOCH_MS: i64 = 1_000;
const HEADER_CONTENT_TYPE: &str = "Content-Type";
const HEADER_AUTHORIZATION: &str = "Authorization";
const HEADER_IDEMPOTENCY_KEY: &str = "Idempotency-Key";

fn secret_ref() -> SecretRef {
    SecretRef::new(SECRET_PROVIDER, SECRET_KEY, None)
}

fn provider() -> MapCredentialProvider {
    MapCredentialProvider::new([(secret_ref().identity(), SECRET_VALUE.to_string())].into())
}

fn trigger() -> TriggerDefinition {
    TriggerDefinition {
        id: TRIGGER_ID.to_string(),
        workflow_name: WORKFLOW_NAME.to_string(),
        enabled: true,
        kind: TriggerKind::Webhook(WebhookTrigger {
            method: WEBHOOK_METHOD.to_string(),
            path: WEBHOOK_PATH.to_string(),
            accepted_content_types: vec![CONTENT_TYPE_JSON.to_string()],
            auth_policy: WebhookAuthPolicy::Bearer {
                secret_ref: secret_ref(),
            },
        }),
        input_bindings: Vec::<TriggerInputBinding>::new(),
        overlap_policy: OverlapPolicy::Forbid,
        idempotency_policy: IdempotencyPolicy::EventKey,
    }
}

fn request(token: &str) -> webhooks::RawRequest {
    webhooks::RawRequest {
        method: WEBHOOK_METHOD.to_string(),
        target: WEBHOOK_PATH.to_string(),
        headers: vec![
            (HEADER_CONTENT_TYPE.to_string(), CONTENT_TYPE_JSON.to_string()),
            (HEADER_AUTHORIZATION.to_string(), format!("Bearer {token}")),
            (HEADER_IDEMPOTENCY_KEY.to_string(), EVENT_KEY.to_string()),
        ],
        body: RAW_BODY.to_vec(),
    }
}

fn context() -> lattice::triggers::TriggerAdmissionContext {
    webhooks::admission_context(&[], CURRENT_EPOCH_MS)
}

fn assert_bytes_omit(bytes: &[u8], value: &str) {
    assert!(!String::from_utf8_lossy(bytes).contains(value));
}

// r[verify n8nclone.credentials.webhook_auth.valid]
#[tokio::test]
async fn valid_webhook_auth_admits_and_persists_no_plaintext_credentials() {
    let temp_dir = tempfile::tempdir().expect("temporary valid webhook directory");
    let database_path = temp_dir.path().join(STORE_FILE_NAME);
    let store = Store::new(&database_path).expect("valid webhook store opens");
    let mut workflow = lattice::engine::workflow::Workflow::new(WORKFLOW_NAME);
    workflow.add_shell_node(NODE_NAME, "printf ok".to_string());
    store.save_workflow(&workflow).expect("webhook workflow saves");
    let decision = webhooks::decide_request(&request(SECRET_VALUE), &[trigger()], &provider(), &context(), limits());
    let webhooks::Decision::Admitted(plan) = decision else {
        panic!("valid authenticated request must be admitted");
    };

    let summary = run_admitted_trigger_plan(&store, &plan).await.expect("admitted webhook runs");
    let records = store.list_trigger_runs(HISTORY_LIMIT).expect("valid webhook history lists");
    let record_json = serde_json::to_string(&records).expect("valid webhook history serializes");
    let persisted = std::fs::read(&database_path).expect("valid webhook store reads");

    assert_eq!(summary.status, HeadlessRunStatus::Succeeded);
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].trigger_id, TRIGGER_ID);
    assert_eq!(records[0].status, TriggerRunStatus::Started);
    for surface in [record_json.as_bytes(), persisted.as_slice()] {
        assert_bytes_omit(surface, SECRET_VALUE);
        assert_bytes_omit(surface, HEADER_AUTHORIZATION);
        assert_bytes_omit(surface, PAYLOAD_SECRET);
    }
}

// r[verify n8nclone.credentials.webhook_auth.redaction]
#[test]
fn rejected_webhook_auth_persists_only_safe_trigger_and_reference_context() {
    let temp_dir = tempfile::tempdir().expect("temporary rejected webhook directory");
    let database_path = temp_dir.path().join(STORE_FILE_NAME);
    let store = Store::new(&database_path).expect("rejected webhook store opens");
    let reference = secret_ref();
    let fingerprint = credential_ref_fingerprint(&reference);
    let decision = webhooks::decide_request(&request(BAD_TOKEN), &[trigger()], &provider(), &context(), limits());
    let webhooks::Decision::Rejected(rejection) = decision else {
        panic!("invalid authenticated request must be rejected");
    };
    let reply = webhooks::render(&webhooks::rejected(&rejection));
    let record = webhooks::run_record_for_rejection(&rejection).expect("auth rejection creates safe history");
    store.save_trigger_run(&record).expect("auth rejection history saves");
    let records = store.list_trigger_runs(HISTORY_LIMIT).expect("rejected webhook history lists");
    let record_json = serde_json::to_string(&records).expect("rejected webhook history serializes");
    let persisted = std::fs::read(&database_path).expect("rejected webhook store reads");

    assert_eq!(records.len(), 1);
    assert_eq!(records[0].trigger_id, TRIGGER_ID);
    assert_eq!(records[0].status, TriggerRunStatus::Rejected);
    assert!(records[0].message.contains(&fingerprint));
    assert!(record_json.contains(TRIGGER_ID));
    assert!(record_json.contains(&fingerprint));
    for surface in [reply.as_slice(), record_json.as_bytes(), persisted.as_slice()] {
        assert_bytes_omit(surface, SECRET_VALUE);
        assert_bytes_omit(surface, BAD_TOKEN);
        assert_bytes_omit(surface, HEADER_AUTHORIZATION);
        assert_bytes_omit(surface, PAYLOAD_SECRET);
        assert_bytes_omit(surface, SECRET_PROVIDER);
        assert_bytes_omit(surface, SECRET_KEY);
    }
}

fn limits() -> webhooks::RequestLimits {
    webhooks::RequestLimits {
        max_body_bytes: RAW_BODY.len(),
    }
}

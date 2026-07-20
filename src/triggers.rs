//! Pure trigger admission, webhook auth facts, and payload binding.

use std::collections::HashMap;
use std::collections::HashSet;

use serde::Deserialize;
use serde::Serialize;

use crate::credentials;
use crate::credentials::CredentialProvider;
use crate::credentials::SecretRef;
use crate::engine::RunId;
use crate::engine::Value;
use crate::scheduler;
use crate::scheduler::MissedRunPolicy;
use crate::scheduler::OverlapPolicy;
use crate::scheduler::RecurrenceRule;
use crate::scheduler::ScheduleDefinition;

const TYPE_TAG_STRING: &str = "string";
const TYPE_TAG_BYTES: &str = "bytes";
const TYPE_TAG_JSON: &str = "json";
const TYPE_TAG_ANY: &str = "any";
const AUTH_HEADER_BEARER_PREFIX: &str = "Bearer ";
const REDACTED_PAYLOAD_SUMMARY: &str = "[payload redacted]";
// r[impl lattice.project_identity.stable-identities.scenario.webhook-domain]
const TRIGGER_SIGNATURE_DOMAIN: &[u8] = b"n8nclone-webhook-signature:v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TriggerDefinition {
    pub id: String,
    pub workflow_name: String,
    pub enabled: bool,
    pub kind: TriggerKind,
    pub input_bindings: Vec<TriggerInputBinding>,
    pub overlap_policy: OverlapPolicy,
    pub idempotency_policy: IdempotencyPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TriggerKind {
    Manual,
    Schedule {
        recurrence: RecurrenceRule,
        missed_run_policy: MissedRunPolicy,
        next_run_epoch_ms: i64,
    },
    Webhook(WebhookTrigger),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WebhookTrigger {
    pub method: String,
    pub path: String,
    pub accepted_content_types: Vec<String>,
    pub auth_policy: WebhookAuthPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WebhookAuthPolicy {
    NoAuthLocalOnly,
    Bearer { secret_ref: SecretRef },
    Blake3Signature { secret_ref: SecretRef, header: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TriggerInputBinding {
    pub input_label: String,
    pub source: TriggerPayloadSource,
    pub type_tag: String,
    pub required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TriggerPayloadSource {
    JsonPointer(String),
    QueryParam(String),
    Header(String),
    RawBody,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IdempotencyPolicy {
    None,
    EventKey,
    PayloadJsonPointer(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TriggerEvent {
    pub method: Option<String>,
    pub path: Option<String>,
    pub content_type: Option<String>,
    pub headers: HashMap<String, String>,
    pub query: HashMap<String, String>,
    pub json_body: Option<serde_json::Value>,
    pub raw_body: Vec<u8>,
    pub idempotency_key: Option<String>,
    pub resolved_auth_secret: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TriggerAdmissionContext {
    pub current_epoch_ms: i64,
    pub running_trigger_ids: HashSet<String>,
    pub accepted_idempotency_keys: HashSet<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerRunPlan {
    pub trigger_id: String,
    pub workflow_name: String,
    pub bound_inputs: HashMap<String, Value>,
    pub idempotency_key: Option<String>,
    pub payload_summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TriggerDiagnostic {
    pub trigger_id: String,
    pub code: TriggerDiagnosticCode,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TriggerDiagnosticCode {
    EmptyTriggerId,
    EmptyWorkflowName,
    MissingWorkflow,
    Disabled,
    MethodMismatch,
    PathMismatch,
    UnsupportedContentType,
    MissingAuthSecret,
    InvalidAuth,
    MissingIdempotencyKey,
    DuplicateIdempotencyKey,
    OverlapPrevented,
    MissingPayloadField,
    PayloadTypeMismatch,
    MalformedSecretRef,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TriggerRunRecord {
    pub trigger_id: String,
    pub workflow_name: String,
    pub run_id: Option<RunId>,
    pub idempotency_key: Option<String>,
    pub status: TriggerRunStatus,
    pub payload_summary: String,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TriggerRunStatus {
    Started,
    Rejected,
    Failed,
}

// r[impl n8nclone.scheduling.triggers.model]
pub fn validate_trigger_definition(
    trigger: &TriggerDefinition,
    workflow_names: &HashSet<String>,
) -> Vec<TriggerDiagnostic> {
    let mut diagnostics = Vec::new();
    if trigger.id.trim().is_empty() {
        diagnostics.push(diagnostic(trigger, TriggerDiagnosticCode::EmptyTriggerId, "trigger id cannot be empty"));
    }
    if trigger.workflow_name.trim().is_empty() {
        diagnostics.push(diagnostic(
            trigger,
            TriggerDiagnosticCode::EmptyWorkflowName,
            "workflow name cannot be empty",
        ));
    } else if !workflow_names.contains(&trigger.workflow_name) {
        diagnostics.push(diagnostic(
            trigger,
            TriggerDiagnosticCode::MissingWorkflow,
            "trigger workflow does not exist",
        ));
    }
    diagnostics.extend(validate_auth_policy(trigger));
    diagnostics
}

fn validate_auth_policy(trigger: &TriggerDefinition) -> Vec<TriggerDiagnostic> {
    let TriggerKind::Webhook(webhook) = &trigger.kind else {
        return Vec::new();
    };
    let secret_ref = match &webhook.auth_policy {
        WebhookAuthPolicy::NoAuthLocalOnly => return Vec::new(),
        WebhookAuthPolicy::Bearer { secret_ref } | WebhookAuthPolicy::Blake3Signature { secret_ref, .. } => secret_ref,
    };
    credentials::validate_secret_ref(secret_ref)
        .err()
        .map(|error| diagnostic(trigger, TriggerDiagnosticCode::MalformedSecretRef, &error.to_string()))
        .into_iter()
        .collect()
}

// r[impl n8nclone.workflow_execution.triggers.payload_binding]
pub fn bind_trigger_payload(
    trigger: &TriggerDefinition,
    event: &TriggerEvent,
) -> Result<HashMap<String, Value>, Vec<TriggerDiagnostic>> {
    let mut diagnostics = Vec::new();
    let mut values = HashMap::new();
    for binding in &trigger.input_bindings {
        match value_for_binding(binding, event) {
            Some(value) if payload_value_compatible(&value, &binding.type_tag) => {
                values.insert(binding.input_label.clone(), value);
            }
            Some(_) => diagnostics.push(diagnostic(
                trigger,
                TriggerDiagnosticCode::PayloadTypeMismatch,
                &format!("payload value for '{}' has incompatible type", binding.input_label),
            )),
            None if binding.required => diagnostics.push(diagnostic(
                trigger,
                TriggerDiagnosticCode::MissingPayloadField,
                &format!("missing required payload field for '{}'", binding.input_label),
            )),
            None => {}
        }
    }
    if diagnostics.is_empty() {
        Ok(values)
    } else {
        Err(diagnostics)
    }
}

fn value_for_binding(binding: &TriggerInputBinding, event: &TriggerEvent) -> Option<Value> {
    match &binding.source {
        TriggerPayloadSource::JsonPointer(pointer) => {
            event.json_body.as_ref()?.pointer(pointer).cloned().map(Value::from_json)
        }
        TriggerPayloadSource::QueryParam(name) => event.query.get(name).cloned().map(Value::from_string),
        TriggerPayloadSource::Header(name) => header_value(&event.headers, name).map(Value::from_string),
        TriggerPayloadSource::RawBody => Some(Value::from_bytes(event.raw_body.clone())),
    }
}

fn payload_value_compatible(value: &Value, type_tag: &str) -> bool {
    match type_tag {
        TYPE_TAG_ANY => true,
        TYPE_TAG_STRING => matches!(value.data, crate::engine::ValueData::String(_)),
        TYPE_TAG_BYTES => matches!(value.data, crate::engine::ValueData::Bytes(_)),
        TYPE_TAG_JSON => matches!(value.data, crate::engine::ValueData::Json(_)),
        _ => false,
    }
}

// r[impl n8nclone.scheduling.triggers.admission]
// r[impl n8nclone.scheduling.triggers.webhook]
pub fn admit_trigger_event(
    trigger: &TriggerDefinition,
    event: &TriggerEvent,
    context: &TriggerAdmissionContext,
) -> Result<TriggerRunPlan, Vec<TriggerDiagnostic>> {
    let mut diagnostics = Vec::new();
    if !trigger.enabled {
        diagnostics.push(diagnostic(trigger, TriggerDiagnosticCode::Disabled, "trigger is disabled"));
    }
    diagnostics.extend(validate_kind_admission(trigger, event));
    let idempotency_key = idempotency_key(trigger, event);
    diagnostics.extend(validate_idempotency(trigger, idempotency_key.as_deref(), context));
    if trigger.overlap_policy == OverlapPolicy::Forbid && context.running_trigger_ids.contains(&trigger.id) {
        diagnostics.push(diagnostic(
            trigger,
            TriggerDiagnosticCode::OverlapPrevented,
            "trigger already has a running workflow",
        ));
    }
    if diagnostics.is_empty() {
        match bind_trigger_payload(trigger, event) {
            Ok(bound_inputs) => Ok(TriggerRunPlan {
                trigger_id: trigger.id.clone(),
                workflow_name: trigger.workflow_name.clone(),
                bound_inputs,
                idempotency_key,
                payload_summary: safe_payload_summary(event),
            }),
            Err(binding_diagnostics) => Err(binding_diagnostics),
        }
    } else {
        Err(diagnostics)
    }
}

fn validate_kind_admission(trigger: &TriggerDefinition, event: &TriggerEvent) -> Vec<TriggerDiagnostic> {
    match &trigger.kind {
        TriggerKind::Manual | TriggerKind::Schedule { .. } => Vec::new(),
        TriggerKind::Webhook(webhook) => validate_webhook_admission(trigger, webhook, event),
    }
}

fn validate_webhook_admission(
    trigger: &TriggerDefinition,
    webhook: &WebhookTrigger,
    event: &TriggerEvent,
) -> Vec<TriggerDiagnostic> {
    let mut diagnostics = Vec::new();
    if event.method.as_deref().map(normalize_method) != Some(normalize_method(&webhook.method)) {
        diagnostics.push(diagnostic(trigger, TriggerDiagnosticCode::MethodMismatch, "webhook method does not match"));
    }
    if event.path.as_deref() != Some(webhook.path.as_str()) {
        diagnostics.push(diagnostic(trigger, TriggerDiagnosticCode::PathMismatch, "webhook path does not match"));
    }
    if !content_type_allowed(webhook, event.content_type.as_deref()) {
        diagnostics.push(diagnostic(
            trigger,
            TriggerDiagnosticCode::UnsupportedContentType,
            "webhook content type is not accepted",
        ));
    }
    if let Err(code) = validate_webhook_auth(&webhook.auth_policy, event) {
        diagnostics.push(diagnostic(trigger, code, "webhook authentication failed"));
    }
    diagnostics
}

fn content_type_allowed(webhook: &WebhookTrigger, content_type: Option<&str>) -> bool {
    if webhook.accepted_content_types.is_empty() {
        return true;
    }
    let Some(content_type) = content_type else {
        return false;
    };
    webhook.accepted_content_types.iter().any(|accepted| content_type.starts_with(accepted))
}

// r[impl n8nclone.credentials.webhook_auth]
pub fn validate_webhook_auth(policy: &WebhookAuthPolicy, event: &TriggerEvent) -> Result<(), TriggerDiagnosticCode> {
    match policy {
        WebhookAuthPolicy::NoAuthLocalOnly => Ok(()),
        WebhookAuthPolicy::Bearer { .. } => {
            let expected = event.resolved_auth_secret.as_ref().ok_or(TriggerDiagnosticCode::MissingAuthSecret)?;
            let authorization =
                header_value(&event.headers, "authorization").ok_or(TriggerDiagnosticCode::InvalidAuth)?;
            let token =
                authorization.strip_prefix(AUTH_HEADER_BEARER_PREFIX).ok_or(TriggerDiagnosticCode::InvalidAuth)?;
            if token == expected {
                Ok(())
            } else {
                Err(TriggerDiagnosticCode::InvalidAuth)
            }
        }
        WebhookAuthPolicy::Blake3Signature { header, .. } => {
            let secret = event.resolved_auth_secret.as_ref().ok_or(TriggerDiagnosticCode::MissingAuthSecret)?;
            let provided = header_value(&event.headers, header).ok_or(TriggerDiagnosticCode::InvalidAuth)?;
            let expected = webhook_signature(secret, &event.raw_body);
            if provided == expected {
                Ok(())
            } else {
                Err(TriggerDiagnosticCode::InvalidAuth)
            }
        }
    }
}

// r[impl n8nclone.credentials.webhook_auth]
pub fn resolve_webhook_auth_secret(
    provider: &dyn CredentialProvider,
    policy: &WebhookAuthPolicy,
) -> Result<Option<String>, TriggerDiagnosticCode> {
    let secret_ref = match policy {
        WebhookAuthPolicy::NoAuthLocalOnly => return Ok(None),
        WebhookAuthPolicy::Bearer { secret_ref } | WebhookAuthPolicy::Blake3Signature { secret_ref, .. } => secret_ref,
    };
    provider
        .resolve(secret_ref)
        .map(|secret| Some(secret.expose_for_execution().to_string()))
        .map_err(|_| TriggerDiagnosticCode::MissingAuthSecret)
}

pub fn webhook_signature(secret: &str, raw_body: &[u8]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(TRIGGER_SIGNATURE_DOMAIN);
    hasher.update(secret.as_bytes());
    hasher.update(raw_body);
    hasher.finalize().to_hex().to_string()
}

fn idempotency_key(trigger: &TriggerDefinition, event: &TriggerEvent) -> Option<String> {
    match &trigger.idempotency_policy {
        IdempotencyPolicy::None => None,
        IdempotencyPolicy::EventKey => event.idempotency_key.clone(),
        IdempotencyPolicy::PayloadJsonPointer(pointer) => {
            event.json_body.as_ref().and_then(|value| value.pointer(pointer)).map(json_value_to_key)
        }
    }
}

fn validate_idempotency(
    trigger: &TriggerDefinition,
    key: Option<&str>,
    context: &TriggerAdmissionContext,
) -> Vec<TriggerDiagnostic> {
    if matches!(trigger.idempotency_policy, IdempotencyPolicy::None) {
        return Vec::new();
    }
    let Some(key) = key else {
        return vec![diagnostic(
            trigger,
            TriggerDiagnosticCode::MissingIdempotencyKey,
            "trigger requires an idempotency key",
        )];
    };
    let scoped_key = scoped_idempotency_key(&trigger.id, key);
    if context.accepted_idempotency_keys.contains(&scoped_key) {
        return vec![diagnostic(
            trigger,
            TriggerDiagnosticCode::DuplicateIdempotencyKey,
            "trigger idempotency key was already accepted",
        )];
    }
    Vec::new()
}

pub fn scoped_idempotency_key(trigger_id: &str, idempotency_key: &str) -> String {
    format!("{trigger_id}:{idempotency_key}")
}

fn json_value_to_key(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(value) => value.clone(),
        _ => value.to_string(),
    }
}

fn header_value(headers: &HashMap<String, String>, name: &str) -> Option<String> {
    headers.iter().find(|(key, _)| key.eq_ignore_ascii_case(name)).map(|(_, value)| value.clone())
}

fn normalize_method(method: &str) -> String {
    method.trim().to_ascii_uppercase()
}

fn safe_payload_summary(event: &TriggerEvent) -> String {
    if event.raw_body.is_empty() && event.json_body.is_none() {
        return "empty payload".to_string();
    }
    REDACTED_PAYLOAD_SUMMARY.to_string()
}

fn diagnostic(trigger: &TriggerDefinition, code: TriggerDiagnosticCode, message: &str) -> TriggerDiagnostic {
    TriggerDiagnostic {
        trigger_id: trigger.id.clone(),
        code,
        message: message.to_string(),
    }
}

// r[impl n8nclone.scheduling.triggers.scheduler_compat]
pub fn schedule_trigger_from_schedule(schedule: &ScheduleDefinition) -> TriggerDefinition {
    TriggerDefinition {
        id: schedule.id.clone(),
        workflow_name: schedule.workflow_name.clone(),
        enabled: schedule.enabled,
        kind: TriggerKind::Schedule {
            recurrence: schedule.recurrence,
            missed_run_policy: schedule.missed_run_policy,
            next_run_epoch_ms: schedule.next_run_epoch_ms,
        },
        input_bindings: Vec::new(),
        overlap_policy: schedule.overlap_policy,
        idempotency_policy: IdempotencyPolicy::None,
    }
}

pub fn schedule_from_schedule_trigger(trigger: &TriggerDefinition) -> Option<ScheduleDefinition> {
    let TriggerKind::Schedule {
        recurrence,
        missed_run_policy,
        next_run_epoch_ms,
    } = trigger.kind
    else {
        return None;
    };
    Some(ScheduleDefinition {
        id: trigger.id.clone(),
        workflow_name: trigger.workflow_name.clone(),
        recurrence,
        enabled: trigger.enabled,
        overlap_policy: trigger.overlap_policy,
        missed_run_policy,
        next_run_epoch_ms,
    })
}

pub fn schedule_due_plan_from_trigger(
    trigger: &TriggerDefinition,
    current_epoch_ms: i64,
    running_trigger_ids: &HashSet<String>,
) -> Option<scheduler::DueRunPlan> {
    let schedule = schedule_from_schedule_trigger(trigger)?;
    scheduler::evaluate_due_runs(&[schedule], current_epoch_ms, running_trigger_ids)
        .due_runs
        .into_iter()
        .next()
}

// r[impl n8nclone.scheduling.triggers.history]
pub fn trigger_run_record_started(plan: &TriggerRunPlan, run_id: RunId) -> TriggerRunRecord {
    TriggerRunRecord {
        trigger_id: plan.trigger_id.clone(),
        workflow_name: plan.workflow_name.clone(),
        run_id: Some(run_id),
        idempotency_key: plan.idempotency_key.clone(),
        status: TriggerRunStatus::Started,
        payload_summary: plan.payload_summary.clone(),
        message: "trigger accepted".to_string(),
    }
}

pub fn trigger_run_record_rejected(
    trigger: &TriggerDefinition,
    diagnostics: &[TriggerDiagnostic],
    event: &TriggerEvent,
) -> TriggerRunRecord {
    let message = diagnostics
        .first()
        .map(|diagnostic| diagnostic.message.clone())
        .unwrap_or_else(|| "trigger rejected".to_string());
    TriggerRunRecord {
        trigger_id: trigger.id.clone(),
        workflow_name: trigger.workflow_name.clone(),
        run_id: None,
        idempotency_key: idempotency_key(trigger, event),
        status: TriggerRunStatus::Rejected,
        payload_summary: safe_payload_summary(event),
        message,
    }
}

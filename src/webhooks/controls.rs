#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DryRunDiagnostic {
    pub trigger_id: String,
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DryRunReport {
    pub bind_addr: String,
    pub loaded_trigger_count: u64,
    pub enabled_trigger_count: u64,
    pub diagnostics: Vec<DryRunDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Status {
    pub bind_addr: String,
    pub loaded_trigger_count: u64,
    pub enabled_trigger_count: u64,
    pub state: &'static str,
}

impl Status {
    pub fn listening(bind_addr: &str, loaded_trigger_count: u64, enabled_trigger_count: u64) -> Self {
        Self {
            bind_addr: bind_addr.to_string(),
            loaded_trigger_count,
            enabled_trigger_count,
            state: super::shared::LISTENER_STATUS_LISTENING,
        }
    }
}

pub fn dry_run_report(
    bind_addr: &str,
    triggers: &[crate::triggers::TriggerDefinition],
    workflow_names: &std::collections::HashSet<String>,
    provider: &dyn crate::credentials::CredentialProvider,
) -> DryRunReport {
    let trigger_count = triggers.len();
    let diagnostic_count = trigger_count.saturating_mul(super::shared::DRY_RUN_DIAGNOSTICS_PER_TRIGGER);
    let mut diagnostics = Vec::with_capacity(diagnostic_count);
    for trigger in triggers {
        diagnostics.extend(definition_diagnostics(trigger, workflow_names));
        diagnostics.extend(credential_diagnostics(trigger, provider));
    }
    assert!(diagnostics.len() <= diagnostic_count);

    DryRunReport {
        bind_addr: bind_addr.to_string(),
        loaded_trigger_count: super::shared::count_as_u64(triggers.len()),
        enabled_trigger_count: enabled_trigger_count(triggers),
        diagnostics,
    }
}

// r[impl n8nclone.tui_ux.webhook_listener.cli]
pub fn render_status(status: &Status) -> String {
    status_line(status)
}

// r[impl n8nclone.tui_ux.webhook_listener.cli]
pub fn render_dry_run_report(report: &DryRunReport) -> String {
    assert!(report.loaded_trigger_count >= report.enabled_trigger_count);
    let state = if report.diagnostics.is_empty() {
        super::shared::LISTENER_STATUS_DRY_RUN_READY
    } else {
        super::shared::LISTENER_STATUS_DRY_RUN_BLOCKED
    };
    assert_eq!(report.diagnostics.is_empty(), state == super::shared::LISTENER_STATUS_DRY_RUN_READY);
    let status = Status {
        bind_addr: report.bind_addr.clone(),
        loaded_trigger_count: report.loaded_trigger_count,
        enabled_trigger_count: report.enabled_trigger_count,
        state,
    };
    let mut output = status_line(&status);
    output.push_str(if report.diagnostics.is_empty() {
        super::shared::DRY_RUN_READY_MESSAGE
    } else {
        super::shared::DRY_RUN_BLOCKED_MESSAGE
    });
    output.push('\n');
    for diagnostic in &report.diagnostics {
        output.push_str(&format!(
            "diagnostic trigger={} code={} message={}\n",
            diagnostic.trigger_id, diagnostic.code, diagnostic.message
        ));
    }
    output
}

pub fn secret_refs(triggers: &[crate::triggers::TriggerDefinition]) -> Vec<crate::credentials::SecretRef> {
    let mut refs = Vec::with_capacity(triggers.len());
    for trigger in triggers {
        if let Some(secret_ref) = secret_ref(trigger) {
            refs.push(secret_ref.clone());
        }
    }
    refs.sort_by_key(|secret_ref| secret_ref.identity());
    refs.dedup_by_key(|secret_ref| secret_ref.identity());
    assert!(refs.len() <= triggers.len());
    refs
}

pub fn run_record_for_rejection(rejection: &super::flow::Rejection) -> Option<crate::triggers::TriggerRunRecord> {
    let trigger = rejection.trigger.as_ref()?;
    let event = rejection.event.as_ref()?;
    if rejection.diagnostics.is_empty() {
        return None;
    }
    Some(crate::triggers::trigger_run_record_rejected(trigger, &rejection.diagnostics, event))
}

pub fn admission_context(
    records: &[crate::triggers::TriggerRunRecord],
    current_epoch_ms: i64,
) -> crate::triggers::TriggerAdmissionContext {
    crate::triggers::TriggerAdmissionContext {
        current_epoch_ms,
        running_trigger_ids: std::collections::HashSet::new(),
        accepted_idempotency_keys: accepted_idempotency_keys(records),
    }
}

pub fn accepted_idempotency_keys(records: &[crate::triggers::TriggerRunRecord]) -> std::collections::HashSet<String> {
    let mut keys = std::collections::HashSet::with_capacity(records.len());
    for record in records.iter().filter(|record| record.status == crate::triggers::TriggerRunStatus::Started) {
        if let Some(key) = &record.idempotency_key {
            keys.insert(crate::triggers::scoped_idempotency_key(&record.trigger_id, key));
        }
    }
    assert!(keys.len() <= records.len());
    keys
}

pub fn enabled_trigger_count(triggers: &[crate::triggers::TriggerDefinition]) -> u64 {
    let count = triggers
        .iter()
        .filter(|trigger| trigger.enabled && matches!(trigger.kind, crate::triggers::TriggerKind::Webhook(_)))
        .count();
    super::shared::count_as_u64(count)
}

fn definition_diagnostics(
    trigger: &crate::triggers::TriggerDefinition,
    workflow_names: &std::collections::HashSet<String>,
) -> Vec<DryRunDiagnostic> {
    let source = crate::triggers::validate_trigger_definition(trigger, workflow_names);
    let mut diagnostics = Vec::with_capacity(source.len());
    for diagnostic in source {
        diagnostics.push(DryRunDiagnostic {
            trigger_id: diagnostic.trigger_id,
            code: super::shared::DRY_RUN_DEFINITION_CODE.to_string(),
            message: diagnostic.message,
        });
    }
    diagnostics
}

fn credential_diagnostics(
    trigger: &crate::triggers::TriggerDefinition,
    provider: &dyn crate::credentials::CredentialProvider,
) -> Vec<DryRunDiagnostic> {
    let Some(secret_ref) = secret_ref(trigger) else {
        return Vec::new();
    };
    match provider.resolve(secret_ref) {
        Ok(_) => Vec::new(),
        Err(error) => vec![DryRunDiagnostic {
            trigger_id: non_empty_trigger_id(trigger),
            code: super::shared::DRY_RUN_CREDENTIAL_CODE.to_string(),
            message: crate::credentials::redact_text(&error.to_string(), std::slice::from_ref(&secret_ref.key)),
        }],
    }
}

fn secret_ref(trigger: &crate::triggers::TriggerDefinition) -> Option<&crate::credentials::SecretRef> {
    let crate::triggers::TriggerKind::Webhook(webhook) = &trigger.kind else {
        return None;
    };
    match &webhook.auth_policy {
        crate::triggers::WebhookAuthPolicy::NoAuthLocalOnly => None,
        crate::triggers::WebhookAuthPolicy::Bearer { secret_ref }
        | crate::triggers::WebhookAuthPolicy::Blake3Signature { secret_ref, .. } => Some(secret_ref),
    }
}

fn non_empty_trigger_id(trigger: &crate::triggers::TriggerDefinition) -> String {
    if trigger.id.trim().is_empty() {
        super::shared::EMPTY_TRIGGER_ID.to_string()
    } else {
        trigger.id.clone()
    }
}

fn status_line(status: &Status) -> String {
    format!(
        "{} bind={} loaded_triggers={} enabled_webhook_triggers={} status={}\n",
        super::shared::STATUS_LINE_PREFIX,
        status.bind_addr,
        status.loaded_trigger_count,
        status.enabled_trigger_count,
        status.state,
    )
}

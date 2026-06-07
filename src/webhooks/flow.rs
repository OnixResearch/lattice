#[derive(Debug, Clone)]
pub struct Rejection {
    pub code: super::shared::RejectionCode,
    pub message: String,
    pub trigger: Option<crate::triggers::TriggerDefinition>,
    pub event: Option<crate::triggers::TriggerEvent>,
    pub diagnostics: Vec<crate::triggers::TriggerDiagnostic>,
}

#[derive(Debug, Clone)]
pub enum Decision {
    Admitted(crate::triggers::TriggerRunPlan),
    Rejected(Box<Rejection>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reply {
    pub status_code: u16,
    pub content_type: &'static str,
    pub body: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
struct AcceptedBody<'a> {
    status: &'static str,
    trigger_id: &'a str,
    run_id: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
struct RejectedBody<'a> {
    status: &'static str,
    code: super::shared::RejectionCode,
    message: &'a str,
}

// r[impl n8nclone.scheduling.webhook_listener.admission]
pub fn decide_request(
    request: &super::planning::RawRequest,
    triggers: &[crate::triggers::TriggerDefinition],
    provider: &dyn crate::credentials::CredentialProvider,
    context: &crate::triggers::TriggerAdmissionContext,
    limits: super::planning::RequestLimits,
) -> Decision {
    let plan = match super::planning::plan_request(request, triggers, limits) {
        Ok(plan) => plan,
        Err(diagnostic) => return Decision::Rejected(Box::new(rejection_from_planning(diagnostic))),
    };
    assert!(plan.event.path.is_some());
    assert!(plan.candidate_trigger_ids.iter().all(|candidate_id| !candidate_id.trim().is_empty()));

    if plan.candidate_trigger_ids.is_empty() {
        return Decision::Rejected(Box::new(not_found_rejection(Some(plan.event))));
    }

    match first_candidate_outcome(&plan, triggers, provider, context) {
        Some(Ok(run_plan)) => Decision::Admitted(run_plan),
        Some(Err(rejection)) => Decision::Rejected(rejection),
        None => Decision::Rejected(Box::new(not_found_rejection(Some(plan.event)))),
    }
}

pub(crate) fn rejection_from_planning(diagnostic: super::planning::RequestDiagnostic) -> Rejection {
    let code = super::shared::rejection_code_for_planning(diagnostic.code);
    Rejection {
        code,
        message: diagnostic.message,
        trigger: None,
        event: None,
        diagnostics: Vec::new(),
    }
}

pub fn execution_failed_rejection(trigger_id: &str) -> Rejection {
    Rejection {
        code: super::shared::RejectionCode::ExecutionFailed,
        message: super::shared::SAFE_EXECUTION_FAILED_MESSAGE.to_string(),
        trigger: Some(crate::triggers::TriggerDefinition {
            id: trigger_id.to_string(),
            workflow_name: String::new(),
            enabled: false,
            kind: crate::triggers::TriggerKind::Manual,
            input_bindings: Vec::new(),
            overlap_policy: crate::scheduler::OverlapPolicy::Forbid,
            idempotency_policy: crate::triggers::IdempotencyPolicy::None,
        }),
        event: None,
        diagnostics: Vec::new(),
    }
}

fn first_candidate_outcome(
    plan: &super::planning::RequestPlan,
    triggers: &[crate::triggers::TriggerDefinition],
    provider: &dyn crate::credentials::CredentialProvider,
    context: &crate::triggers::TriggerAdmissionContext,
) -> Option<Result<crate::triggers::TriggerRunPlan, Box<Rejection>>> {
    let mut first_rejection = None;
    for candidate_id in &plan.candidate_trigger_ids {
        let Some(trigger) = triggers.iter().find(|trigger| &trigger.id == candidate_id) else {
            continue;
        };
        match try_admit_candidate(trigger, &plan.event, provider, context) {
            Ok(run_plan) => return Some(Ok(run_plan)),
            Err(rejection) if first_rejection.is_none() => first_rejection = Some(rejection),
            Err(_) => {}
        }
    }
    first_rejection.map(Err)
}

fn try_admit_candidate(
    trigger: &crate::triggers::TriggerDefinition,
    plan_event: &crate::triggers::TriggerEvent,
    provider: &dyn crate::credentials::CredentialProvider,
    context: &crate::triggers::TriggerAdmissionContext,
) -> Result<crate::triggers::TriggerRunPlan, Box<Rejection>> {
    let mut event = plan_event.clone();
    event.resolved_auth_secret = resolved_secret(provider, trigger);
    crate::triggers::admit_trigger_event(trigger, &event, context)
        .map_err(|diagnostics| Box::new(rejection_from_trigger_diagnostics(trigger.clone(), event, diagnostics)))
}

fn resolved_secret(
    provider: &dyn crate::credentials::CredentialProvider,
    trigger: &crate::triggers::TriggerDefinition,
) -> Option<String> {
    let crate::triggers::TriggerKind::Webhook(webhook) = &trigger.kind else {
        return None;
    };
    crate::triggers::resolve_webhook_auth_secret(provider, &webhook.auth_policy).ok().flatten()
}

fn rejection_from_trigger_diagnostics(
    trigger: crate::triggers::TriggerDefinition,
    event: crate::triggers::TriggerEvent,
    diagnostics: Vec<crate::triggers::TriggerDiagnostic>,
) -> Rejection {
    let code = diagnostics
        .first()
        .map(|diagnostic| super::shared::rejection_code_for_trigger(diagnostic.code))
        .unwrap_or(super::shared::RejectionCode::InvalidRequest);
    Rejection {
        code,
        message: super::shared::safe_message_for_rejection(code).to_string(),
        trigger: Some(trigger),
        event: Some(event),
        diagnostics,
    }
}

fn not_found_rejection(event: Option<crate::triggers::TriggerEvent>) -> Rejection {
    Rejection {
        code: super::shared::RejectionCode::NotFound,
        message: super::shared::SAFE_NOT_FOUND_MESSAGE.to_string(),
        trigger: None,
        event,
        diagnostics: Vec::new(),
    }
}

// r[impl n8nclone.scheduling.webhook_listener.responses]
pub fn accepted(trigger_id: &str, run_id: crate::engine::RunId) -> Reply {
    let body = AcceptedBody {
        status: super::shared::STATUS_ACCEPTED,
        trigger_id,
        run_id: run_id.0,
    };
    Reply {
        status_code: super::shared::HTTP_STATUS_ACCEPTED,
        content_type: super::shared::HTTP_CONTENT_TYPE_JSON,
        body: json_body(&body),
    }
}

// r[impl n8nclone.scheduling.webhook_listener.responses]
pub fn rejected(rejection: &Rejection) -> Reply {
    rejected_for_code(rejection.code)
}

pub fn rejected_for_code(code: super::shared::RejectionCode) -> Reply {
    let body = RejectedBody {
        status: super::shared::STATUS_REJECTED,
        code,
        message: super::shared::safe_message_for_rejection(code),
    };
    Reply {
        status_code: status_for_code(code),
        content_type: super::shared::HTTP_CONTENT_TYPE_JSON,
        body: json_body(&body),
    }
}

pub fn for_planning(diagnostic: &super::planning::RequestDiagnostic) -> Reply {
    rejected(&rejection_from_planning(diagnostic.clone()))
}

pub fn execution_failed(trigger_id: &str) -> Reply {
    rejected(&execution_failed_rejection(trigger_id))
}

pub fn render(reply: &Reply) -> Vec<u8> {
    assert_eq!(reply.content_type, super::shared::HTTP_CONTENT_TYPE_JSON);
    let header = format!(
        "HTTP/1.1 {} {}\r\ncontent-type: {}\r\ncontent-length: {}\r\nconnection: {}\r\n\r\n",
        reply.status_code,
        reason_phrase(reply.status_code),
        reply.content_type,
        reply.body.len(),
        super::shared::HTTP_CONNECTION_CLOSE,
    );
    let mut bytes = Vec::with_capacity(header.len().saturating_add(reply.body.len()));
    bytes.extend_from_slice(header.as_bytes());
    bytes.extend_from_slice(&reply.body);
    bytes
}

fn json_body<T: serde::Serialize>(body: &T) -> Vec<u8> {
    match serde_json::to_vec(body) {
        Ok(bytes) => bytes,
        Err(_) => b"{}".to_vec(),
    }
}

fn status_for_code(code: super::shared::RejectionCode) -> u16 {
    match code {
        super::shared::RejectionCode::NotFound => super::shared::HTTP_STATUS_NOT_FOUND,
        super::shared::RejectionCode::InvalidRequest | super::shared::RejectionCode::MalformedJson => {
            super::shared::HTTP_STATUS_BAD_REQUEST
        }
        super::shared::RejectionCode::BodyTooLarge => super::shared::HTTP_STATUS_PAYLOAD_TOO_LARGE,
        super::shared::RejectionCode::MethodMismatch => super::shared::HTTP_STATUS_METHOD_NOT_ALLOWED,
        super::shared::RejectionCode::UnsupportedContentType => super::shared::HTTP_STATUS_UNSUPPORTED_MEDIA_TYPE,
        super::shared::RejectionCode::MissingAuthSecret | super::shared::RejectionCode::InvalidAuth => {
            super::shared::HTTP_STATUS_UNAUTHORIZED
        }
        super::shared::RejectionCode::MissingIdempotencyKey
        | super::shared::RejectionCode::DuplicateIdempotencyKey
        | super::shared::RejectionCode::OverlapPrevented
        | super::shared::RejectionCode::Disabled => super::shared::HTTP_STATUS_CONFLICT,
        super::shared::RejectionCode::MissingPayloadField | super::shared::RejectionCode::PayloadTypeMismatch => {
            super::shared::HTTP_STATUS_BAD_REQUEST
        }
        super::shared::RejectionCode::ExecutionFailed => super::shared::HTTP_STATUS_INTERNAL_SERVER_ERROR,
    }
}

fn reason_phrase(status_code: u16) -> &'static str {
    match status_code {
        super::shared::HTTP_STATUS_ACCEPTED => super::shared::HTTP_REASON_ACCEPTED,
        super::shared::HTTP_STATUS_BAD_REQUEST => super::shared::HTTP_REASON_BAD_REQUEST,
        super::shared::HTTP_STATUS_UNAUTHORIZED => super::shared::HTTP_REASON_UNAUTHORIZED,
        super::shared::HTTP_STATUS_NOT_FOUND => super::shared::HTTP_REASON_NOT_FOUND,
        super::shared::HTTP_STATUS_METHOD_NOT_ALLOWED => super::shared::HTTP_REASON_METHOD_NOT_ALLOWED,
        super::shared::HTTP_STATUS_CONFLICT => super::shared::HTTP_REASON_CONFLICT,
        super::shared::HTTP_STATUS_PAYLOAD_TOO_LARGE => super::shared::HTTP_REASON_PAYLOAD_TOO_LARGE,
        super::shared::HTTP_STATUS_UNSUPPORTED_MEDIA_TYPE => super::shared::HTTP_REASON_UNSUPPORTED_MEDIA_TYPE,
        super::shared::HTTP_STATUS_INTERNAL_SERVER_ERROR => super::shared::HTTP_REASON_INTERNAL_SERVER_ERROR,
        _ => super::shared::DEFAULT_SAFE_REJECTION_MESSAGE,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RequestLimits {
    pub max_body_bytes: usize,
}

impl Default for RequestLimits {
    fn default() -> Self {
        Self {
            max_body_bytes: super::shared::DEFAULT_MAX_BODY_BYTES,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawRequest {
    pub method: String,
    pub target: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestPlan {
    pub event: crate::triggers::TriggerEvent,
    pub candidate_trigger_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestDiagnostic {
    pub code: super::shared::PlanningCode,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TargetParts {
    path: String,
    query: std::collections::HashMap<String, String>,
}

// r[impl n8nclone.scheduling.webhook_listener.planning]
pub fn plan_request(
    request: &RawRequest,
    triggers: &[crate::triggers::TriggerDefinition],
    limits: RequestLimits,
) -> Result<RequestPlan, RequestDiagnostic> {
    if request.body.len() > limits.max_body_bytes {
        return Err(diagnostic(super::shared::PlanningCode::BodyTooLarge, super::shared::SAFE_BODY_TOO_LARGE_MESSAGE));
    }

    let target = parse_target(&request.target)?;
    assert!(target.path.starts_with('/'));
    let event = event_from_request(request, &target)?;
    let candidate_trigger_ids = candidate_ids(triggers, &target.path);
    assert_eq!(event.path.as_deref(), Some(target.path.as_str()));

    Ok(RequestPlan {
        event,
        candidate_trigger_ids,
    })
}

pub(crate) fn diagnostic(code: super::shared::PlanningCode, message: &str) -> RequestDiagnostic {
    RequestDiagnostic {
        code,
        message: message.to_string(),
    }
}

fn event_from_request(
    request: &RawRequest,
    target: &TargetParts,
) -> Result<crate::triggers::TriggerEvent, RequestDiagnostic> {
    let headers = normalized_headers(&request.headers);
    let content_type = headers.get(super::shared::CONTENT_TYPE_HEADER).cloned();
    let json_body = optional_json_body(content_type.as_deref(), &request.body)?;
    Ok(crate::triggers::TriggerEvent {
        method: Some(normalize_method(&request.method)),
        path: Some(target.path.clone()),
        content_type,
        headers,
        query: target.query.clone(),
        json_body,
        raw_body: request.body.clone(),
        idempotency_key: header_value_from_pairs(&request.headers, super::shared::IDEMPOTENCY_KEY_HEADER),
        resolved_auth_secret: None,
    })
}

fn parse_target(target: &str) -> Result<TargetParts, RequestDiagnostic> {
    let (path, raw_query) = split_target(target);
    if !path.starts_with('/') {
        return Err(diagnostic(super::shared::PlanningCode::InvalidRequest, super::shared::SAFE_BAD_REQUEST_MESSAGE));
    }
    Ok(TargetParts {
        path: path.to_string(),
        query: parse_query(raw_query),
    })
}

fn split_target(target: &str) -> (&str, &str) {
    match target.split_once('?') {
        Some(parts) => parts,
        None => (target, ""),
    }
}

fn parse_query(raw_query: &str) -> std::collections::HashMap<String, String> {
    let query_pair_count = raw_query.split('&').take(super::shared::MAX_QUERY_PAIR_COUNT).count();
    let mut query = std::collections::HashMap::with_capacity(query_pair_count);
    for (index, (name, value)) in url::form_urlencoded::parse(raw_query.as_bytes()).enumerate() {
        if index >= super::shared::MAX_QUERY_PAIR_COUNT {
            break;
        }
        query.insert(name.into_owned(), value.into_owned());
    }
    assert!(query.len() <= super::shared::MAX_QUERY_PAIR_COUNT);
    query
}

fn normalized_headers(header_pairs: &[(String, String)]) -> std::collections::HashMap<String, String> {
    let mut headers = std::collections::HashMap::with_capacity(header_pairs.len());
    for (name, value) in header_pairs {
        let normalized_name = name.trim().to_ascii_lowercase();
        if !normalized_name.is_empty() {
            headers.insert(normalized_name, value.trim().to_string());
        }
    }
    assert!(headers.len() <= header_pairs.len());
    headers
}

fn header_value_from_pairs(header_pairs: &[(String, String)], name: &str) -> Option<String> {
    header_pairs
        .iter()
        .rev()
        .find(|(header_name, _)| header_name.eq_ignore_ascii_case(name))
        .map(|(_, value)| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn optional_json_body(content_type: Option<&str>, body: &[u8]) -> Result<Option<serde_json::Value>, RequestDiagnostic> {
    if !content_type.is_some_and(content_type_declares_json) {
        return Ok(None);
    }
    serde_json::from_slice(body)
        .map(Some)
        .map_err(|_| diagnostic(super::shared::PlanningCode::MalformedJson, super::shared::SAFE_MALFORMED_JSON_MESSAGE))
}

fn content_type_declares_json(content_type: &str) -> bool {
    let base = match content_type.split(';').next() {
        Some(value) => value.trim().to_ascii_lowercase(),
        None => String::new(),
    };
    base == super::shared::JSON_CONTENT_TYPE || base.ends_with(super::shared::JSON_CONTENT_TYPE_SUFFIX)
}

fn candidate_ids(triggers: &[crate::triggers::TriggerDefinition], path: &str) -> Vec<String> {
    let mut ids = Vec::with_capacity(triggers.len());
    for trigger in triggers.iter().filter(|trigger| trigger.enabled) {
        if let crate::triggers::TriggerKind::Webhook(webhook) = &trigger.kind
            && webhook.path == path
        {
            ids.push(trigger.id.clone());
        }
    }
    ids.sort();
    assert!(ids.len() <= triggers.len());
    ids
}

fn normalize_method(method: &str) -> String {
    method.trim().to_ascii_uppercase()
}

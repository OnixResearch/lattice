fn append_event(
    request: &mut crate::workflow::gate::model::ApprovalRequest,
    kind: crate::workflow::gate::model::ApprovalEventKind,
) -> Result<(), AdmissionError> {
    if request.events.len() >= MAX_REQUEST_EVENTS {
        return Err(error(AdmissionCode::IdentityFailure, "approval request event count exceeds its bound"));
    }
    let initial_event_count = request.events.len();
    request.events.reserve(EVENT_COUNT_INCREMENT);
    let previous = request.events.last().map(|event| event.id.clone());
    let sequence = previous.as_ref().map(|_| request.events.len() as u64).unwrap_or(INITIAL_EVENT_SEQUENCE);
    if let Some(last) = request.events.last() {
        let expected = last
            .sequence
            .checked_add(EVENT_SEQUENCE_INCREMENT)
            .ok_or_else(|| error(AdmissionCode::IdentityFailure, "approval event sequence is exhausted"))?;
        if sequence != expected {
            return Err(error(AdmissionCode::IdentityFailure, "approval event sequence is not contiguous"));
        }
    }
    let state_digest = state_digest(request)?;
    let material = EventIdentityMaterial {
        request_id: &request.id,
        previous: previous.as_ref(),
        sequence,
        kind,
        state_digest: &state_digest,
    };
    let id = crate::workflow::gate::identity::ApprovalEventId::from_generated(hash(EVENT_HASH_DOMAIN, &material)?);
    request.events.push(crate::workflow::gate::model::ApprovalEvent {
        id,
        previous,
        sequence,
        kind,
        state_digest,
    });
    debug_assert_eq!(request.events.len(), initial_event_count.saturating_add(EVENT_COUNT_INCREMENT));
    debug_assert!(request.events.len() <= MAX_REQUEST_EVENTS);
    Ok(())
}

fn state_digest(
    request: &crate::workflow::gate::model::ApprovalRequest,
) -> Result<crate::workflow::gate::identity::ApprovalStateDigest, AdmissionError> {
    let material = StateIdentityMaterial {
        request_id: &request.id,
        status: request.status,
        decision_id: request.decision.as_ref().map(|decision| &decision.id),
        active_resume_owner: request.active_resume_owner.as_ref(),
        completed_resume_id: request.completed_resume_id.as_ref(),
        completed_nodes: &request.completed_nodes,
        graph_events: &request.graph_events,
        version: request.version,
    };
    Ok(crate::workflow::gate::identity::ApprovalStateDigest::from_generated(hash(
        STATE_HASH_DOMAIN,
        &material,
    )?))
}

fn increment_version(request: &mut crate::workflow::gate::model::ApprovalRequest) -> Result<(), AdmissionError> {
    request.version = request
        .version
        .checked_add(REQUEST_VERSION_INCREMENT)
        .ok_or_else(|| error(AdmissionCode::IdentityFailure, "approval request version is exhausted"))?;
    Ok(())
}

fn deadline(start: i64, ttl_millis: u64) -> Result<i64, AdmissionError> {
    let ttl = i64::try_from(ttl_millis)
        .map_err(|_| error(AdmissionCode::TimeOverflow, "approval expiry interval cannot be represented"))?;
    start
        .checked_add(ttl)
        .ok_or_else(|| error(AdmissionCode::TimeOverflow, "approval expiry deadline overflowed"))
}

fn hash<T: serde::Serialize + ?Sized>(domain: &[u8], value: &T) -> Result<String, AdmissionError> {
    let bytes = serde_json::to_vec(value)
        .map_err(|_| error(AdmissionCode::IdentityFailure, "approval identity encoding failed"))?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain);
    hasher.update(HASH_FRAME_SEPARATOR);
    hasher.update(&bytes);
    Ok(hasher.finalize().to_hex().to_string())
}

struct CollectionValidation<'a, T> {
    values: &'a [T],
    count: usize,
    label: &'a str,
}

fn validate_count_and_unique<T: Ord>(input: CollectionValidation<'_, T>, issues: &mut IssueSink<'_>) {
    if input.values.len() > input.count {
        issues
            .push(issue(ValidationCode::TooManyValues, format!("approval {} exceed the reviewed bound", input.label)));
    }
    if has_duplicates(input.values) {
        issues.push(issue(ValidationCode::DuplicateValue, format!("approval {} must be unique", input.label)));
    }
}

fn has_duplicates<T: Ord>(values: &[T]) -> bool {
    let mut seen = std::collections::BTreeSet::new();
    values.iter().any(|value| !seen.insert(value))
}

fn has_duplicates_by<'a>(values: impl Iterator<Item = &'a str>) -> bool {
    let mut seen = std::collections::BTreeSet::new();
    values.into_iter().any(|value| !seen.insert(value))
}

fn is_valid_type_tag(value: &str) -> bool {
    matches!(value, "string" | "bytes" | "json" | "image" | "exitcode" | "any")
}

fn is_valid_reference(value: &str) -> bool {
    [
        !value.trim().is_empty(),
        value.len() <= MAX_REFERENCE_BYTES,
        value.trim() == value,
        !value.chars().any(char::is_control),
    ]
    .into_iter()
    .all(std::convert::identity)
}

fn issue(code: ValidationCode, message: impl Into<String>) -> ValidationIssue {
    ValidationIssue {
        code,
        message: message.into(),
    }
}

fn error(code: AdmissionCode, message: impl Into<String>) -> AdmissionError {
    AdmissionError {
        code,
        message: message.into(),
    }
}

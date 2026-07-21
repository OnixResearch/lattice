fn fact_matches_query(fact: &ProvenanceFact, query: &ProvenanceFactQuery) -> bool {
    matches_filter(query.artifact_hash.as_deref(), fact_artifact_hash(fact))
        && matches_filter(query.tool_kind.as_deref(), fact_tool_kind(fact))
        && matches_filter(query.error_class.as_deref(), fact_error_class(fact))
        && matches_filter(query.requirement_id.as_deref(), fact_requirement_id(fact))
        && matches_filter(query.request_shape.as_deref(), fact_request_shape(fact))
        && matches_authorization(query.authorization_status, fact_authorization_status(fact))
        && matches_filter(query.effect_ability.as_deref(), fact_effect_ability(fact))
        && matches_filter(query.effect_resource.as_deref(), fact_effect_resource(fact))
}

fn matches_filter(expected: Option<&str>, actual: Option<&str>) -> bool {
    expected.is_none_or(|expected_value| actual == Some(expected_value))
}

fn matches_authorization(
    expected: Option<ProvenanceAuthorizationStatus>,
    actual: Option<ProvenanceAuthorizationStatus>,
) -> bool {
    expected.is_none_or(|expected_value| actual == Some(expected_value))
}

fn fact_artifact_hash(fact: &ProvenanceFact) -> Option<&str> {
    match fact {
        ProvenanceFact::ModelActivity(value) => Some(value.artifact_hash.as_str()),
        ProvenanceFact::ToolActivity(value) => Some(value.artifact_hash.as_str()),
        ProvenanceFact::Artifact(value) => Some(value.artifact_hash.as_str()),
        ProvenanceFact::RuntimeBlock(_)
        | ProvenanceFact::Error(_)
        | ProvenanceFact::Authorization(_)
        | ProvenanceFact::Observation(_)
        | ProvenanceFact::Opaque(_) => None,
    }
}

fn fact_tool_kind(fact: &ProvenanceFact) -> Option<&str> {
    match fact {
        ProvenanceFact::ToolActivity(value) => Some(value.tool_kind.as_str()),
        ProvenanceFact::ModelActivity(_)
        | ProvenanceFact::RuntimeBlock(_)
        | ProvenanceFact::Error(_)
        | ProvenanceFact::Artifact(_)
        | ProvenanceFact::Authorization(_)
        | ProvenanceFact::Observation(_)
        | ProvenanceFact::Opaque(_) => None,
    }
}

fn fact_error_class(fact: &ProvenanceFact) -> Option<&str> {
    match fact {
        ProvenanceFact::Error(value) => Some(value.error_class.as_str()),
        ProvenanceFact::ModelActivity(_)
        | ProvenanceFact::ToolActivity(_)
        | ProvenanceFact::RuntimeBlock(_)
        | ProvenanceFact::Artifact(_)
        | ProvenanceFact::Authorization(_)
        | ProvenanceFact::Observation(_)
        | ProvenanceFact::Opaque(_) => None,
    }
}

fn fact_requirement_id(fact: &ProvenanceFact) -> Option<&str> {
    match fact {
        ProvenanceFact::ModelActivity(value) => value.requirement_id.as_deref(),
        ProvenanceFact::RuntimeBlock(value) => value.requirement_id.as_deref(),
        ProvenanceFact::Error(value) => value.requirement_id.as_deref(),
        ProvenanceFact::Observation(value) => value.requirement_id.as_deref(),
        ProvenanceFact::ToolActivity(_)
        | ProvenanceFact::Artifact(_)
        | ProvenanceFact::Authorization(_)
        | ProvenanceFact::Opaque(_) => None,
    }
}

fn fact_request_shape(fact: &ProvenanceFact) -> Option<&str> {
    match fact {
        ProvenanceFact::ModelActivity(value) => Some(value.request_shape.as_str()),
        ProvenanceFact::ToolActivity(_)
        | ProvenanceFact::RuntimeBlock(_)
        | ProvenanceFact::Error(_)
        | ProvenanceFact::Artifact(_)
        | ProvenanceFact::Authorization(_)
        | ProvenanceFact::Observation(_)
        | ProvenanceFact::Opaque(_) => None,
    }
}

fn fact_authorization_status(fact: &ProvenanceFact) -> Option<ProvenanceAuthorizationStatus> {
    match fact {
        ProvenanceFact::Authorization(value) => Some(value.status),
        ProvenanceFact::ModelActivity(_)
        | ProvenanceFact::ToolActivity(_)
        | ProvenanceFact::RuntimeBlock(_)
        | ProvenanceFact::Error(_)
        | ProvenanceFact::Artifact(_)
        | ProvenanceFact::Observation(_)
        | ProvenanceFact::Opaque(_) => None,
    }
}

fn fact_effect_ability(fact: &ProvenanceFact) -> Option<&str> {
    match fact {
        ProvenanceFact::ToolActivity(value) => Some(value.effect_ability.as_str()),
        ProvenanceFact::Authorization(value) => Some(value.effect_ability.as_str()),
        ProvenanceFact::ModelActivity(_)
        | ProvenanceFact::RuntimeBlock(_)
        | ProvenanceFact::Error(_)
        | ProvenanceFact::Artifact(_)
        | ProvenanceFact::Observation(_)
        | ProvenanceFact::Opaque(_) => None,
    }
}

fn fact_effect_resource(fact: &ProvenanceFact) -> Option<&str> {
    match fact {
        ProvenanceFact::ToolActivity(value) => Some(value.effect_resource.as_str()),
        ProvenanceFact::Authorization(value) => Some(value.effect_resource.as_str()),
        ProvenanceFact::ModelActivity(_)
        | ProvenanceFact::RuntimeBlock(_)
        | ProvenanceFact::Error(_)
        | ProvenanceFact::Artifact(_)
        | ProvenanceFact::Observation(_)
        | ProvenanceFact::Opaque(_) => None,
    }
}

fn artifact_hash(
    kind: ProvenanceArtifactKind,
    redaction: ProvenanceRedactionClass,
    payload: &str,
    dependencies: &[Text],
) -> Text {
    let mut normalized = Text::from(PROVENANCE_ARTIFACT_SCHEMA);
    push_hash_part(&mut normalized, kind.as_str());
    push_hash_part(&mut normalized, redaction.as_str());
    push_hash_part(&mut normalized, payload);
    for dependency in dependencies {
        push_hash_part(&mut normalized, dependency.as_str());
    }
    blake3_hex(normalized.as_bytes())
}

fn receipt_hash(input: ReceiptHashInput<'_>) -> Text {
    let mut normalized = Text::from(PROVENANCE_RECEIPT_SCHEMA);
    push_hash_part(&mut normalized, input.id);
    push_hash_part(&mut normalized, input.status);
    push_hash_part(
        &mut normalized,
        input.issue_code.unwrap_or(EMPTY_OPTION_MARKER),
    );
    push_hash_part(&mut normalized, input.actor_label);
    push_hash_part(&mut normalized, input.session_label);
    push_hash_part(&mut normalized, input.trace_label);
    push_hash_part(&mut normalized, input.event_count.to_string().as_str());
    for artifact_hash in input.artifact_hashes {
        push_hash_part(&mut normalized, artifact_hash.as_str());
    }
    push_hash_part(
        &mut normalized,
        input.mutation_kind.unwrap_or(EMPTY_OPTION_MARKER),
    );
    push_hash_part(
        &mut normalized,
        input.effect_kind.unwrap_or(EMPTY_OPTION_MARKER),
    );
    push_hash_part(
        &mut normalized,
        input.backend_label.unwrap_or(EMPTY_OPTION_MARKER),
    );
    push_hash_part(
        &mut normalized,
        input.interchange_label.unwrap_or(EMPTY_OPTION_MARKER),
    );
    for marker in input.redaction_markers {
        push_hash_part(&mut normalized, marker.as_str());
    }
    blake3_hex(normalized.as_bytes())
}

fn push_hash_part(normalized: &mut Text, part: &str) {
    normalized.push(HASH_PART_SEPARATOR);
    normalized.push_str(part.len().to_string().as_str());
    normalized.push(HASH_LENGTH_SEPARATOR);
    normalized.push_str(part);
}

fn blake3_hex(bytes: &[u8]) -> Text {
    blake3::hash(bytes).to_hex().to_string()
}

fn validate_safe_text(field: &'static str, value: Text) -> Result<Text, ProvenanceError> {
    if value.is_empty() {
        return Err(ProvenanceError::new(
            ProvenanceErrorKind::EmptyField,
            field,
            UNKNOWN_SAFE_FIELD,
            "value must not be empty",
        ));
    }
    if value.len() > MAX_SAFE_FIELD_CHARS {
        return Err(ProvenanceError::new(
            ProvenanceErrorKind::FieldTooLong,
            field,
            UNKNOWN_SAFE_FIELD,
            "value exceeds safe metadata limit",
        ));
    }
    if let Some(marker) = contains_unsafe_material(value.as_str()) {
        return Err(ProvenanceError::new(
            ProvenanceErrorKind::UnsafeMetadata,
            field,
            marker.class,
            "value contains unsafe material marker",
        ));
    }
    Ok(value)
}

fn validate_optional_safe(
    field: &'static str,
    value: Option<&str>,
) -> Result<Option<Text>, ProvenanceError> {
    value
        .map(|value| validate_safe_text(field, value.to_owned()))
        .transpose()
}

fn validate_safe_list(
    field: &'static str,
    values: &[&str],
) -> Result<Collection<Text>, ProvenanceError> {
    values
        .iter()
        .map(|value| validate_safe_text(field, (*value).to_owned()))
        .collect()
}

fn validate_parent_ids(values: &[&str]) -> Result<Collection<Text>, ProvenanceError> {
    validate_safe_list("parent_id", values)
}

fn validate_hashes(
    field: &'static str,
    values: &[&str],
) -> Result<Collection<Text>, ProvenanceError> {
    values
        .iter()
        .map(|value| Ok(validate_hash(field, value)?.to_owned()))
        .collect()
}

fn validate_count(field: &'static str, value: u64) -> Result<(), ProvenanceError> {
    if value > MAX_RECEIPT_COUNT {
        return Err(ProvenanceError::new(
            ProvenanceErrorKind::FieldTooLong,
            field,
            value.to_string(),
            "count exceeds receipt bound",
        ));
    }
    Ok(())
}

fn validate_hash<'a>(field: &'static str, value: &'a str) -> Result<&'a str, ProvenanceError> {
    if value.len() != HASH_HEX_LENGTH || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(ProvenanceError::new(
            ProvenanceErrorKind::InvalidHash,
            field,
            UNKNOWN_SAFE_FIELD,
            "expected BLAKE3 hex digest",
        ));
    }
    Ok(value)
}

fn contains_unsafe_material(value: &str) -> Option<UnsafeMarker> {
    let lowered = value.to_ascii_lowercase();
    UNSAFE_MARKERS
        .iter()
        .copied()
        .find(|marker| lowered.contains(marker.needle))
}

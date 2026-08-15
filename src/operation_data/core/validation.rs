fn validate_draft(draft: &RecordDraft, policy: &AdmissionPolicy<'_>) -> Result<(), Error> {
    validate_text(&draft.logical_key)?;
    validate_text(&draft.output_name)?;
    validate_text(&draft.outcome)?;
    validate_producer(&draft.producer)?;
    validate_identity(&draft.output_contract_identity)?;
    validate_identity(&draft.retention.identity)?;
    if draft.sequence > MAXIMUM_VERSION_SEQUENCE {
        return Err(Error::Limit("version sequence"));
    }
    validate_tags(&draft.tags)?;
    for value in draft.tags.values() {
        if prohibited_fragment(value.as_bytes(), policy.prohibited_fragments) {
            return Err(Error::Secret);
        }
    }
    for value in [
        draft.logical_key.as_str(),
        draft.output_name.as_str(),
        draft.outcome.as_str(),
        draft.producer.operation_type.as_str(),
        draft.producer.instance.as_str(),
        draft.producer.method.as_str(),
        draft.producer.node.as_str(),
    ] {
        if private_path(value) || prohibited_fragment(value.as_bytes(), policy.prohibited_fragments) {
            return Err(Error::Secret);
        }
    }
    if draft.sensitivity == SensitivityClass::Sensitive && !policy.allow_sensitive {
        return Err(Error::Secret);
    }
    let mut normalized_non_claims = draft.required_non_claims.clone();
    normalized_non_claims.sort();
    normalized_non_claims.dedup();
    validate_non_claims(&normalized_non_claims)?;
    for value in &normalized_non_claims {
        if private_path(value) || prohibited_fragment(value.as_bytes(), policy.prohibited_fragments) {
            return Err(Error::Secret);
        }
    }
    match &draft.payload {
        PayloadDraft::Resource {
            schema_identity,
            selectable_paths,
            value,
        } => {
            validate_identity(schema_identity)?;
            let mut normalized_paths = selectable_paths.clone();
            normalized_paths.sort();
            normalized_paths.dedup();
            validate_selectable_paths(&normalized_paths)?;
            if value_contains_secret(value, policy.prohibited_fragments) {
                return Err(Error::Secret);
            }
        }
        PayloadDraft::File { media_class, bytes } => {
            validate_text(media_class)?;
            if private_path(media_class) || prohibited_fragment(media_class.as_bytes(), policy.prohibited_fragments) {
                return Err(Error::Secret);
            }
            if bytes.len() > MAX_RECORD_BYTES {
                return Err(Error::Limit("file bytes"));
            }
        }
    }
    Ok(())
}

pub fn validate_metadata(metadata: &RecordMetadata) -> Result<(), Error> {
    if metadata.schema != RECORD_SCHEMA {
        return Err(Error::Invalid("record schema"));
    }
    validate_identity(&metadata.record_identity)?;
    validate_identity(&metadata.version_identity)?;
    validate_identity(&metadata.output_contract_identity)?;
    validate_identity(&metadata.retention.identity)?;
    validate_identity(&metadata.payload.content_identity)?;
    validate_identity(&metadata.payload.safe_locator_identity)?;
    validate_text(&metadata.logical_key)?;
    validate_text(&metadata.output_name)?;
    validate_text(&metadata.outcome)?;
    validate_producer(&metadata.producer)?;
    validate_tags(&metadata.tags)?;
    validate_non_claims(&metadata.required_non_claims)?;
    validate_payload_shape(metadata)?;
    if metadata.sequence > MAXIMUM_VERSION_SEQUENCE {
        return Err(Error::Limit("version sequence"));
    }
    if metadata.payload.storage_backend != STORAGE_BACKEND || !safe_locator(&metadata.payload.safe_locator) {
        return Err(Error::Invalid("payload storage"));
    }
    if metadata.payload.byte_length_bytes > u64::try_from(MAX_RECORD_BYTES).map_err(|_| Error::Encoding)? {
        return Err(Error::Limit("payload bytes"));
    }
    let content_suffix = metadata
        .payload
        .content_identity
        .strip_prefix(crate::operation::model::BLAKE3_PREFIX)
        .ok_or(Error::Invalid("content identity"))?;
    let expected_locator = format!("{LOCATOR_PREFIX}{content_suffix}{LOCATOR_SUFFIX}");
    let expected_locator_identity = crate::operation::identity::hash_bytes(LOCATOR_DOMAIN, expected_locator.as_bytes());
    if metadata.payload.safe_locator != expected_locator
        || metadata.payload.safe_locator_identity != expected_locator_identity
    {
        return Err(Error::Invalid("payload locator identity"));
    }
    let expected_record_identity = record_identity(metadata)?;
    if metadata.record_identity != expected_record_identity {
        return Err(Error::Invalid("record identity"));
    }
    let expected_version_identity = version_identity(metadata)?;
    if metadata.version_identity != expected_version_identity {
        return Err(Error::Invalid("version identity"));
    }
    let encoded = serde_json::to_vec(metadata).map_err(|_| Error::Encoding)?;
    if encoded.len() > MAX_METADATA_BYTES {
        return Err(Error::Limit("metadata bytes"));
    }
    Ok(())
}

fn validate_producer(producer: &Producer) -> Result<(), Error> {
    validate_text(&producer.operation_type)?;
    validate_identity(&producer.type_revision)?;
    validate_text(&producer.instance)?;
    validate_text(&producer.method)?;
    validate_identity(&producer.invocation)?;
    validate_text(&producer.node)?;
    validate_identity(&producer.workflow_revision)
}

fn validate_tags(tags: &std::collections::BTreeMap<String, String>) -> Result<(), Error> {
    if tags.len() > MAX_TAGS {
        return Err(Error::Limit("tag count"));
    }
    for (key, value) in tags {
        validate_text(key)?;
        validate_text(value)?;
        if secret_key(key) || private_path(value) {
            return Err(Error::Secret);
        }
    }
    Ok(())
}

fn validate_non_claims(non_claims: &[String]) -> Result<(), Error> {
    if non_claims.len() > MAX_NON_CLAIMS {
        return Err(Error::Limit("non-claim count"));
    }
    if non_claims.windows(2).any(|window| window[0] >= window[1]) {
        return Err(Error::Invalid("non-claim order"));
    }
    for value in non_claims {
        validate_text(value)?;
    }
    for required in REQUIRED_NON_CLAIMS {
        if !non_claims.iter().any(|value| value == required) {
            return Err(Error::Invalid("required non-claim"));
        }
    }
    Ok(())
}

fn validate_payload_shape(metadata: &RecordMetadata) -> Result<(), Error> {
    match metadata.record_class {
        RecordClass::Resource => {
            let schema = metadata.payload.schema_identity.as_ref().ok_or(Error::Invalid("resource schema"))?;
            validate_identity(schema)?;
            if metadata.payload.media_class.is_some() {
                return Err(Error::Invalid("resource media class"));
            }
            validate_selectable_paths(&metadata.payload.selectable_paths)
        }
        RecordClass::File => {
            if metadata.payload.schema_identity.is_some() || !metadata.payload.selectable_paths.is_empty() {
                return Err(Error::Invalid("file structured fields"));
            }
            let media = metadata.payload.media_class.as_ref().ok_or(Error::Invalid("file media class"))?;
            validate_text(media)
        }
    }
}

pub(super) fn validate_selectable_paths(paths: &[Vec<String>]) -> Result<(), Error> {
    if paths.len() > MAX_SELECTABLE_PATHS {
        return Err(Error::Limit("selectable path count"));
    }
    if paths.windows(2).any(|window| window[0] >= window[1]) {
        return Err(Error::Invalid("selectable path order"));
    }
    for path in paths {
        if path.is_empty() || path.len() > MAX_FIELD_PATH_PARTS {
            return Err(Error::Limit("selectable field path"));
        }
        for part in path {
            validate_text(part)?;
        }
    }
    Ok(())
}

pub(super) fn validate_identity(value: &str) -> Result<(), Error> {
    if crate::operation::identity::validate(value) {
        Ok(())
    } else {
        Err(Error::Invalid("BLAKE3 identity"))
    }
}

pub(super) fn validate_text(value: &str) -> Result<(), Error> {
    if value.is_empty() || value.len() > MAX_TEXT_BYTES || value.chars().any(char::is_control) {
        Err(Error::Invalid("bounded text"))
    } else {
        Ok(())
    }
}

fn safe_locator(value: &str) -> bool {
    value.starts_with(LOCATOR_PREFIX)
        && value.ends_with(LOCATOR_SUFFIX)
        && !value.contains("..")
        && !value.contains('\\')
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'/' | b'.' | b'-'))
}

fn secret_key(key: &str) -> bool {
    let normalized = key.to_ascii_lowercase();
    SECRET_KEY_FRAGMENTS.iter().any(|fragment| normalized.contains(fragment))
}

fn private_path(value: &str) -> bool {
    PRIVATE_PATH_PREFIXES.iter().any(|prefix| value.starts_with(prefix))
}

fn value_contains_secret(value: &serde_json::Value, fragments: &[String]) -> bool {
    match value {
        serde_json::Value::Array(values) => values.iter().any(|value| value_contains_secret(value, fragments)),
        serde_json::Value::Object(values) => {
            values.iter().any(|(key, value)| secret_key(key) || value_contains_secret(value, fragments))
        }
        serde_json::Value::String(value) => {
            private_path(value) || prohibited_fragment(value.as_bytes(), fragments) || value.starts_with("Bearer ")
        }
        serde_json::Value::Null | serde_json::Value::Bool(_) | serde_json::Value::Number(_) => false,
    }
}

fn contains_prohibited_bytes(bytes: &[u8], fragments: &[String]) -> bool {
    prohibited_fragment(bytes, fragments)
        || std::str::from_utf8(bytes).is_ok_and(|value| value.contains("Authorization: Bearer "))
}

fn prohibited_fragment(bytes: &[u8], fragments: &[String]) -> bool {
    fragments
        .iter()
        .filter(|fragment| !fragment.is_empty())
        .any(|fragment| bytes.windows(fragment.len()).any(|window| window == fragment.as_bytes()))
}

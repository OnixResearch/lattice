fn validate_local_location(field: &'static str, value: &str) -> Result<Text, BatchError> {
    let text = required_text(field, value)?;
    if is_remote_location(text.as_str()) {
        return Err(BatchError::UnsupportedLocation { field, value: text });
    }
    Ok(text)
}

fn required_text(field: &'static str, value: &str) -> Result<Text, BatchError> {
    let normalized = value.trim();
    if normalized.is_empty() {
        return Err(BatchError::EmptyField { field });
    }
    Ok(Text::from(normalized))
}

fn validate_optional_safe_text(
    field: &'static str,
    value: Option<&str>,
) -> Result<Option<Text>, BatchError> {
    value
        .map(|present| validate_safe_metadata(field, present))
        .transpose()
}

fn validate_safe_label(field: &'static str, value: &str) -> Result<Text, BatchError> {
    let text = required_text(field, value)?;
    if is_remote_location(text.as_str()) || !is_safe_label(text.as_str()) {
        return Err(BatchError::InvalidLabel { field, value: text });
    }
    Ok(text)
}

fn validate_safe_metadata(field: &'static str, value: &str) -> Result<Text, BatchError> {
    let text = required_text(field, value)?;
    if let Some(marker) = leak_marker(text.as_str()) {
        return Err(BatchError::UnsafeMetadata { field, marker });
    }
    Ok(text)
}

fn is_safe_label(value: &str) -> bool {
    value
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.'))
}

fn is_remote_location(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.starts_with(HTTP_SCHEME)
        || lower.starts_with(HTTPS_SCHEME)
        || lower.starts_with(S3_SCHEME)
        || lower.contains(SCHEME_SEPARATOR)
        || lower.starts_with(UNC_PREFIX)
}

fn leak_marker(value: &str) -> Option<&'static str> {
    let lower = value.to_ascii_lowercase();
    LEAK_MARKERS
        .iter()
        .find(|marker| lower.contains(marker.needle))
        .map(|marker| marker.marker)
}

fn unsupported_value(field: &'static str, value: &str) -> BatchError {
    BatchError::UnsupportedValue {
        field,
        value: Text::from(value),
    }
}

fn plan_hash(parts: PlanHashParts<'_>) -> Text {
    let mut normalized = Text::from(BATCH_PLAN_SCHEMA);
    push_part(&mut normalized, parts.source_location);
    push_part(&mut normalized, parts.output_location);
    push_part(&mut normalized, parts.output_format.as_str());
    push_part(
        &mut normalized,
        parts.concurrency.limit().to_string().as_str(),
    );
    push_part(&mut normalized, parts.execution_mode.as_str());
    push_part(&mut normalized, parts.redaction_policy.as_str());
    for job in parts.jobs {
        push_part(&mut normalized, job.job_hash());
    }
    blake3_hex(normalized.as_bytes())
}

fn receipt_hash(parts: ReceiptHashParts<'_>) -> Text {
    let mut normalized = Text::from(BATCH_RECEIPT_SCHEMA);
    push_part(&mut normalized, parts.run_id);
    push_part(&mut normalized, parts.plan_hash);
    push_part(&mut normalized, parts.output_format.as_str());
    push_part(&mut normalized, parts.redaction_policy.as_str());
    for job in parts.jobs {
        push_part(&mut normalized, job.job_id());
        push_part(&mut normalized, job.status().as_str());
        push_part(&mut normalized, job.prompt_bytes().to_string().as_str());
        push_part(&mut normalized, job.model_handle().unwrap_or_default());
        push_part(&mut normalized, job.session_handle().unwrap_or_default());
        push_part(&mut normalized, job.objective_summary());
        push_part(&mut normalized, job.output_label().as_str());
        push_part(&mut normalized, job.output_evidence_hash());
        push_part(&mut normalized, job.redaction_marker());
    }
    blake3_hex(normalized.as_bytes())
}

fn hash_parts(parts: &[&str]) -> Text {
    let mut normalized = Text::new();
    for part in parts {
        push_part(&mut normalized, part);
    }
    blake3_hex(normalized.as_bytes())
}

fn push_part(output: &mut Text, part: &str) {
    output.push('|');
    output.push_str(part.len().to_string().as_str());
    output.push(':');
    output.push_str(part);
}

fn blake3_hex(bytes: &[u8]) -> Text {
    blake3::hash(bytes).to_hex().to_string()
}

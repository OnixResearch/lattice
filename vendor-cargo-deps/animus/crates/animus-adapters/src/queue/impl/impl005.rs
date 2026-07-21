fn normalize_required(
    field: crate::validation::FieldName,
    value: Option<&str>,
) -> Result<String, QueueContractError> {
    let raw = value.ok_or(crate::AdapterContractError::EmptyField {
        field: field.as_str(),
    })?;
    Ok(crate::validation::normalized_adapter_field(field, raw)?)
}

fn normalize_label_slice(
    field: crate::validation::FieldName,
    labels: &[&str],
) -> Result<Vec<String>, QueueContractError> {
    labels
        .iter()
        .map(|label| Ok(crate::validation::normalized_adapter_field(field, label)?))
        .collect()
}

fn validate_log_counts(
    draft: &QueueLogViewportDraft<'_>,
    bounds: QueueLogBounds,
) -> Result<(), QueueContractError> {
    if draft.visible_line_count > draft.total_line_count {
        return Err(QueueContractError::InconsistentLogCounts {
            visible_field: VISIBLE_LINES_FIELD,
            total_field: TOTAL_LINES_FIELD,
        });
    }
    if draft.visible_byte_count > draft.total_byte_count {
        return Err(QueueContractError::InconsistentLogCounts {
            visible_field: VISIBLE_BYTES_FIELD,
            total_field: TOTAL_BYTES_FIELD,
        });
    }
    if draft.visible_line_count > bounds.max_visible_lines() {
        return Err(QueueContractError::LogWindowTooLarge {
            field: VISIBLE_LINES_FIELD,
            max: bounds.max_visible_lines(),
            actual: draft.visible_line_count,
        });
    }
    if draft.visible_byte_count > bounds.max_visible_bytes() {
        return Err(QueueContractError::LogWindowTooLarge {
            field: VISIBLE_BYTES_FIELD,
            max: bounds.max_visible_bytes(),
            actual: draft.visible_byte_count,
        });
    }
    if output_is_partial(draft) && !draft.truncated {
        return Err(QueueContractError::MissingTruncationMarker);
    }
    Ok(())
}

const fn output_is_partial(draft: &QueueLogViewportDraft<'_>) -> bool {
    draft.visible_line_count < draft.total_line_count
        || draft.visible_byte_count < draft.total_byte_count
}

fn validate_capability(
    kind: QueueInteractionKind,
    capabilities: QueueCapabilities,
) -> Result<(), QueueContractError> {
    let supported = match kind {
        QueueInteractionKind::Wait => capabilities.wait(),
        QueueInteractionKind::Stream => capabilities.stream(),
        QueueInteractionKind::Poll => capabilities.poll(),
        QueueInteractionKind::SendInput => capabilities.send_input(),
    };
    if supported {
        Ok(())
    } else {
        Err(QueueContractError::UnsupportedCapability {
            capability: kind.capability(),
        })
    }
}

#[cfg(test)]
mod tests {
    include!("../tests/tests001.rs");
    include!("../tests/tests002.rs");
}

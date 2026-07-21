pub(super) fn required_text(
    field_name: crate::validation::FieldName,
    value: Option<&str>,
) -> Result<super::Text, super::Error> {
    match value {
        Some(text) => Ok(crate::validation::normalized_adapter_field(
            field_name, text,
        )?),
        None => Err(crate::AdapterContractError::EmptyField {
            field: field_name.as_str(),
        }
        .into()),
    }
}

pub(super) fn required_collection(
    field_name: crate::validation::FieldName,
    values: &[&str],
) -> Result<super::Collection<super::Text>, super::Error> {
    if values.is_empty() {
        let issue = if field_name.as_str() == super::SUCCESS_FIELD.as_str() {
            super::PromptHygieneIssue::MissingSuccessCriteria
        } else if field_name.as_str() == super::VALIDATION_FIELD.as_str() {
            super::PromptHygieneIssue::MissingValidationRule
        } else {
            super::PromptHygieneIssue::MissingStopRule
        };
        return Err(super::Error::PromptHygiene { issue });
    }
    optional_collection(field_name, values)
}

pub(super) fn optional_collection(
    field_name: crate::validation::FieldName,
    values: &[&str],
) -> Result<super::Collection<super::Text>, super::Error> {
    let actual_items = values.len();
    if actual_items > super::MAX_POLICY_ITEMS {
        return Err(super::Error::TooManyItems {
            field: field_name.as_str(),
            max_items: super::MAX_POLICY_ITEMS,
            actual_items,
        });
    }
    let mut normalized = super::Collection::with_capacity(actual_items);
    for value in values {
        normalized.push(crate::validation::normalized_adapter_field(
            field_name, value,
        )?);
    }
    Ok(normalized)
}

pub(super) fn priority(priority: u16) -> Result<(), super::Error> {
    if !(super::MIN_POLICY_PRIORITY..=super::MAX_POLICY_PRIORITY).contains(&priority) {
        return Err(super::Error::PriorityOutOfRange {
            min: super::MIN_POLICY_PRIORITY,
            max: super::MAX_POLICY_PRIORITY,
            actual: priority,
        });
    }
    Ok(())
}

pub(super) fn reject_hygiene_markers(value: &str) -> Result<(), super::Error> {
    let lowered = value.to_ascii_lowercase();
    if lowered.contains(super::PROCESS_HEAVY_MARKER) {
        return Err(super::Error::PromptHygiene {
            issue: super::PromptHygieneIssue::ProcessHeavy,
        });
    }
    if lowered.contains(super::UNBOUNDED_MARKER) {
        return Err(super::Error::PromptHygiene {
            issue: super::PromptHygieneIssue::UnboundedInstruction,
        });
    }
    if lowered.contains(super::STALE_MODEL_MARKER) {
        return Err(super::Error::PromptHygiene {
            issue: super::PromptHygieneIssue::StaleModelGuidance,
        });
    }
    Ok(())
}

pub(super) fn parse_kind(value: &str) -> Option<super::Kind> {
    match value {
        "global-posture" => Some(super::Kind::GlobalPosture),
        "project-overlay" => Some(super::Kind::ProjectOverlay),
        "tool-guidance" => Some(super::Kind::ToolGuidance),
        _unsupported => None,
    }
}

pub(super) fn parse_scope(value: &str) -> Option<super::SourceScope> {
    match value {
        "global" => Some(super::SourceScope::Global),
        "session" => Some(super::SourceScope::Session),
        "project" => Some(super::SourceScope::Project),
        "tool" => Some(super::SourceScope::Tool),
        _unsupported => None,
    }
}

pub(super) fn parse_trust(value: &str) -> Option<super::TrustStatus> {
    match value {
        "trusted" => Some(super::TrustStatus::Trusted),
        "untrusted" => Some(super::TrustStatus::Untrusted),
        _unsupported => None,
    }
}

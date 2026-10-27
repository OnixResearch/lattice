const FAILURE_MESSAGE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("adapter_failure_message");
const CORRELATION_ID_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("adapter_correlation_id");
const RECEIPT_REFERENCE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("adapter_receipt_reference");
const UNSAFE_METADATA_MARKERS: &[&str] = &[
    "/home/",
    "raw_prompt:",
    "provider_payload:",
    "model_output:",
    "tool_output:",
    "begin private key",
    "api_key=",
    "aws_secret_access_key",
    "sk-",
];

pub(super) fn required_message(
    value: &str,
) -> Result<alloc::string::String, crate::AdapterContractError> {
    safe_required_text(FAILURE_MESSAGE_FIELD, value)
}

pub(super) fn optional_correlation_id(
    value: Option<&str>,
) -> Result<Option<alloc::string::String>, crate::AdapterContractError> {
    safe_optional_text(CORRELATION_ID_FIELD, value)
}

pub(super) fn optional_receipt_reference(
    value: Option<&str>,
) -> Result<Option<alloc::string::String>, crate::AdapterContractError> {
    safe_optional_text(RECEIPT_REFERENCE_FIELD, value)
}

fn safe_required_text(
    field: crate::validation::FieldName,
    value: &str,
) -> Result<alloc::string::String, crate::AdapterContractError> {
    let normalized = crate::validation::normalized_adapter_field(field, value)?;
    reject_unsafe(field, normalized.as_str())?;
    Ok(normalized)
}

fn safe_optional_text(
    field: crate::validation::FieldName,
    value: Option<&str>,
) -> Result<Option<alloc::string::String>, crate::AdapterContractError> {
    value
        .map(|text| safe_required_text(field, text))
        .transpose()
}

fn reject_unsafe(
    field: crate::validation::FieldName,
    value: &str,
) -> Result<(), crate::AdapterContractError> {
    let lower = value.to_ascii_lowercase();
    for marker in UNSAFE_METADATA_MARKERS {
        if lower.contains(marker) {
            return Err(crate::AdapterContractError::UnsupportedStatusState {
                field: field.as_str(),
                value: alloc::string::String::from(*marker),
            });
        }
    }
    Ok(())
}

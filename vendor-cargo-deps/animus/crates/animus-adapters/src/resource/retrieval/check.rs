pub(super) fn required_text(
    field: crate::validation::FieldName,
    value: Option<&str>,
) -> Result<alloc::string::String, super::failure::Error> {
    value
        .map(|raw| crate::validation::normalized_adapter_field(field, raw).map_err(super::failure::Error::from))
        .transpose()?
        .ok_or(super::failure::Error::MissingField { field: field.as_str() })
}

pub(super) fn conditions(
    field: crate::validation::FieldName,
    values: &[&str],
) -> Result<alloc::vec::Vec<alloc::string::String>, super::failure::Error> {
    assert!(
        values.len() <= crate::validation::MAX_ADAPTER_FIELD_CHARS,
        "condition list count fits adapter diagnostics"
    );
    condition_count(field, values)?;
    values
        .iter()
        .map(|value| crate::validation::normalized_adapter_field(field, value).map_err(super::failure::Error::from))
        .collect()
}

fn condition_count(field: crate::validation::FieldName, values: &[&str]) -> Result<(), super::failure::Error> {
    if values.is_empty() {
        return Err(super::failure::Error::EmptyConditionList { field: field.as_str() });
    }
    if values.len() > super::MAX_CONDITIONS {
        return Err(super::failure::Error::ConditionLimit {
            field: field.as_str(),
            max: super::MAX_CONDITIONS,
            actual: values.len(),
        });
    }
    Ok(())
}

pub(super) fn limit(value: Option<u32>) -> Result<Option<core::num::NonZeroU32>, super::failure::Error> {
    value
        .map(|raw| core::num::NonZeroU32::new(raw).ok_or(super::failure::Error::InvalidLimit))
        .transpose()
}

pub(super) fn missing_evidence(
    status: super::facts::EvidenceStatus,
    values: &[&str],
) -> Result<alloc::vec::Vec<alloc::string::String>, super::failure::Error> {
    if status == super::facts::EvidenceStatus::Sufficient {
        return Ok(alloc::vec::Vec::new());
    }
    if values.is_empty() {
        return Err(super::failure::Error::MissingEvidenceNames);
    }
    conditions(super::EVIDENCE_FIELD, values)
}

pub(super) const fn raw_body(value: Option<&str>) -> Result<(), super::failure::Error> {
    if value.is_some() {
        return Err(super::failure::Error::RawBodyEmbedded);
    }
    Ok(())
}

pub(super) fn safe_reference(
    field: crate::validation::FieldName,
    value: Option<&str>,
) -> Result<alloc::string::String, super::failure::Error> {
    let reference = required_text(field, value)?;
    if unsafe_reference(reference.as_str()) {
        return Err(super::failure::Error::UnsafeReference { reference });
    }
    Ok(reference)
}

pub(super) fn content_hash(value: Option<&str>) -> Result<Option<crate::artifact::Hash>, super::failure::Error> {
    value
        .map(crate::artifact::Hash::new)
        .transpose()
        .map_err(|_error| super::failure::Error::InvalidHash)
}

fn unsafe_reference(value: &str) -> bool {
    super::UNSAFE_REFERENCE_MARKERS.iter().any(|marker| value.contains(marker))
}

pub(super) fn truncate_chars(mut value: alloc::string::String, max_chars: usize) -> alloc::string::String {
    let char_count = value.chars().count();
    if char_count <= max_chars {
        return value;
    }
    let byte_index = value.char_indices().map(|(index, _character)| index).nth(max_chars).unwrap_or(value.len());
    value.truncate(byte_index);
    value
}

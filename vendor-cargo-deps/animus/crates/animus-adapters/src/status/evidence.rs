const EVIDENCE_HASH_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("status_evidence_hash");
const EVIDENCE_HASH_NOT_HEX_REASON: &str = "evidence hash must contain lowercase hex only";
const EVIDENCE_HASH_LENGTH_REASON: &str = "evidence hash must use the configured BLAKE3 hex length";

/// Character length of a lowercase BLAKE3 evidence hash in hex form.
pub const SAFE_EVIDENCE_HASH_HEX_CHARS: usize = 64;

const _: () = assert!(SAFE_EVIDENCE_HASH_HEX_CHARS > 0);

pub(super) fn collect_hashes(
    values: &[&str],
) -> Result<alloc::vec::Vec<alloc::string::String>, crate::AdapterContractError> {
    values
        .iter()
        .map(|value| {
            validate_hash(value)?;
            Ok(alloc::string::String::from(*value))
        })
        .collect()
}

fn validate_hash(value: &str) -> Result<(), crate::AdapterContractError> {
    crate::validation::validate_required_adapter_field(EVIDENCE_HASH_FIELD, value)?;
    let actual_chars = value.chars().count();
    assert!(actual_chars > 0, "validated hash cannot be empty");
    assert!(
        actual_chars <= value.len(),
        "char count must fit in byte length"
    );
    if actual_chars != SAFE_EVIDENCE_HASH_HEX_CHARS {
        return Err(crate::AdapterContractError::InvalidEvidenceHash {
            field: EVIDENCE_HASH_FIELD.as_str(),
            reason: EVIDENCE_HASH_LENGTH_REASON,
        });
    }
    if !value
        .chars()
        .all(|character| character.is_ascii_hexdigit() && !character.is_ascii_uppercase())
    {
        return Err(crate::AdapterContractError::InvalidEvidenceHash {
            field: EVIDENCE_HASH_FIELD.as_str(),
            reason: EVIDENCE_HASH_NOT_HEX_REASON,
        });
    }
    Ok(())
}

use crate::types::{
    ToonError,
    ToonResult,
};

/// Validate that array length matches expected value.
pub fn validate_array_length(expected: usize, actual: usize) -> ToonResult<()> {
    // Array length mismatches should always error, regardless of strict mode
    if expected != actual {
        return Err(ToonError::length_mismatch(expected, actual));
    }
    Ok(())
}

/// Validate field list for tabular arrays (no duplicates, non-empty names).
pub fn validate_field_list(fields: &[String]) -> ToonResult<()> {
    if fields.is_empty() {
        return Err(ToonError::InvalidInput(
            "Field list cannot be empty for tabular arrays".to_string(),
        ));
    }

    // Check for duplicate field names
    for i in 0..fields.len() {
        for j in (i + 1)..fields.len() {
            if fields[i] == fields[j] {
                return Err(ToonError::InvalidInput(format!(
                    "Duplicate field name: '{}'",
                    fields[i]
                )));
            }
        }
    }

    for field in fields {
        if field.is_empty() {
            return Err(ToonError::InvalidInput(
                "Field name cannot be empty".to_string(),
            ));
        }
    }

    Ok(())
}

/// Validate that a tabular row has the expected number of values.
pub fn validate_row_length(
    row_index: usize,
    expected_fields: usize,
    actual_values: usize,
) -> ToonResult<()> {
    if expected_fields != actual_values {
        return Err(ToonError::InvalidStructure(format!(
            "Row {row_index} has {actual_values} values but expected {expected_fields} fields"
        )));
    }
    Ok(())
}

/// Validate that detected and expected delimiters match.
pub fn validate_delimiter_consistency(
    detected: Option<crate::types::Delimiter>,
    expected: Option<crate::types::Delimiter>,
) -> ToonResult<()> {
    if let (Some(detected), Some(expected)) = (detected, expected) {
        if detected != expected {
            return Err(ToonError::InvalidDelimiter(format!(
                "Detected delimiter {detected} but expected {expected}"
            )));
        }
    }
    Ok(())
}

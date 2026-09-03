use serde_json::Value;

use crate::types::{
    ToonError,
    ToonResult,
};

/// Validate that nesting depth doesn't exceed the maximum.
pub fn validate_depth(depth: usize, max_depth: usize) -> ToonResult<()> {
    if depth > max_depth {
        return Err(ToonError::InvalidStructure(
            "Maximum nesting depth of {max_depth} exceeded".to_string(),
        ));
    }
    Ok(())
}

/// Validate that a field name is not empty.
pub fn validate_field_name(name: &str) -> ToonResult<()> {
    if name.is_empty() {
        return Err(ToonError::InvalidInput(
            "Field name cannot be empty".to_string(),
        ));
    }
    Ok(())
}

/// Recursively validate a JSON value and all nested fields.
pub fn validate_value(value: &Value) -> ToonResult<()> {
    match value {
        Value::Object(obj) => {
            for (key, val) in obj {
                validate_field_name(key)?;
                validate_value(val)?;
            }
        }
        Value::Array(arr) => {
            for val in arr {
                validate_value(val)?;
            }
        }
        _ => {}
    }
    Ok(())
}

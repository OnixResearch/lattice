pub fn is_primitive(value: &serde_json::Value) -> bool {
    matches!(
        value,
        serde_json::Value::Null
            | serde_json::Value::Bool(_)
            | serde_json::Value::Number(_)
            | serde_json::Value::String(_)
    )
}

pub fn all_primitives(values: &[serde_json::Value]) -> bool {
    values.iter().all(is_primitive)
}

/// Recursively normalize JSON values.
pub fn normalize_value(value: serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Null => serde_json::Value::Null,
        serde_json::Value::Bool(b) => serde_json::Value::Bool(b),
        serde_json::Value::Number(n) => serde_json::Value::Number(n),
        serde_json::Value::String(s) => serde_json::Value::String(s),
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(arr.into_iter().map(normalize_value).collect())
        }
        serde_json::Value::Object(obj) => serde_json::Value::Object(
            obj.into_iter()
                .map(|(k, v)| (k, normalize_value(v)))
                .collect(),
        ),
    }
}

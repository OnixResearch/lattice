use crate::types::{
    is_identifier_segment,
    JsonValue as Value,
    KeyFoldingMode,
};

/// Result of chain analysis for folding.
pub struct FoldableChain {
    /// The folded key path (e.g., "a.b.c")
    pub folded_key: String,
    /// The leaf value at the end of the chain
    pub leaf_value: Value,
    /// Number of segments that were folded
    pub depth_folded: usize,
}

/// Check if a value is a single-key object suitable for folding.
fn is_single_key_object(value: &Value) -> Option<(&String, &Value)> {
    if let Value::Object(obj) = value {
        if obj.len() == 1 {
            return obj.iter().next();
        }
    }
    None
}

/// Analyze if a key-value pair can be folded into dotted notation.
pub fn analyze_foldable_chain(
    key: &str,
    value: &Value,
    flatten_depth: usize,
    existing_keys: &[&String],
) -> Option<FoldableChain> {
    if !is_identifier_segment(key) {
        return None;
    }

    let mut segments = vec![key.to_string()];
    let mut current_value = value;

    // Follow single-key object chain until we hit a multi-key object or leaf
    while let Some((next_key, next_value)) = is_single_key_object(current_value) {
        if segments.len() >= flatten_depth {
            break;
        }

        if !is_identifier_segment(next_key) {
            break;
        }

        segments.push(next_key.clone());
        current_value = next_value;
    }

    // Must fold at least 2 segments to be worthwhile
    if segments.len() < 2 {
        return None;
    }

    let folded_key = segments.join(".");

    // Don't fold if it would collide with an existing key
    if existing_keys.contains(&&folded_key) {
        return None;
    }

    Some(FoldableChain {
        folded_key,
        leaf_value: current_value.clone(),
        depth_folded: segments.len(),
    })
}

pub fn should_fold(mode: KeyFoldingMode, chain: &Option<FoldableChain>) -> bool {
    match mode {
        KeyFoldingMode::Off => false,
        KeyFoldingMode::Safe => chain.is_some(),
    }
}

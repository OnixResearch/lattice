#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum KeyFoldingMode {
    /// No folding performed. All objects use standard nesting.
    #[default]
    Off,
    /// Fold eligible chains according to safety rules.
    Safe,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PathExpansionMode {
    /// Dotted keys are treated as literal keys. No expansion.
    #[default]
    Off,
    /// Expand eligible dotted keys according to safety rules.
    Safe,
}

/// Check if a key segment is a valid IdentifierSegment (stricter than unquoted
/// keys).
pub fn is_identifier_segment(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    let mut chars = s.chars();

    // First character must be letter or underscore
    let first = match chars.next() {
        Some(c) => c,
        None => return false,
    };

    if !first.is_alphabetic() && first != '_' {
        return false;
    }

    // Remaining characters: letters, digits, or underscore (NO dots)
    chars.all(|c| c.is_alphanumeric() || c == '_')
}

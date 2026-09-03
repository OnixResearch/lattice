use crate::constants;

/// Check if a string looks like a keyword or number (needs quoting).
pub fn is_literal_like(s: &str) -> bool {
    is_keyword(s) || is_numeric_like(s)
}

#[inline]
pub fn is_keyword(s: &str) -> bool {
    constants::is_keyword(s)
}

#[inline]
pub fn is_structural_char(ch: char) -> bool {
    constants::is_structural_char(ch)
}

/// Check if a string looks like a number (starts with digit, no leading zeros).
pub fn is_numeric_like(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;

    if chars[i] == '-' {
        i += 1;
    }

    if i >= chars.len() {
        return false;
    }

    if !chars[i].is_ascii_digit() {
        return false;
    }

    if chars[i] == '0' && i + 1 < chars.len() && chars[i + 1].is_ascii_digit() {
        return false;
    }

    let has_valid_chars = chars[i..].iter().all(|c| {
        c.is_ascii_digit() || *c == '.' || *c == 'e' || *c == 'E' || *c == '+' || *c == '-'
    });

    has_valid_chars
}

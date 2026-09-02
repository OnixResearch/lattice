use crate::types::Number;

/// Format a number in TOON canonical form (no exponents, no trailing zeros).
pub fn format_canonical_number(n: &Number) -> String {
    if let Some(i) = n.as_i64() {
        return i.to_string();
    }

    if let Some(u) = n.as_u64() {
        return u.to_string();
    }

    if let Some(f) = n.as_f64() {
        return format_f64_canonical(f);
    }

    n.to_string()
}

fn format_f64_canonical(f: f64) -> String {
    // Normalize integer-valued floats to integers
    if f.is_finite() && f.fract() == 0.0 && f.abs() <= i64::MAX as f64 {
        return format!("{}", f as i64);
    }

    let default_format = format!("{f}");

    // Handle cases where Rust would use exponential notation
    if default_format.contains('e') || default_format.contains('E') {
        format_without_exponent(f)
    } else {
        remove_trailing_zeros(&default_format)
    }
}

fn format_without_exponent(f: f64) -> String {
    if !f.is_finite() {
        return "0".to_string();
    }

    if f.abs() >= 1.0 {
        let abs_f = f.abs();
        let int_part = abs_f.trunc();
        let frac_part = abs_f.fract();

        if frac_part == 0.0 {
            format!("{}{}", if f < 0.0 { "-" } else { "" }, int_part as i64)
        } else {
            // High precision to avoid exponent, then trim trailing zeros
            let result = format!("{f:.17}");
            remove_trailing_zeros(&result)
        }
    } else if f == 0.0 {
        "0".to_string()
    } else {
        // Small numbers: use high precision to avoid exponent
        let result = format!("{f:.17}",);
        remove_trailing_zeros(&result)
    }
}

fn remove_trailing_zeros(s: &str) -> String {
    if !s.contains('.') {
        // No decimal point, return as-is
        return s.to_string();
    }

    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() != 2 {
        return s.to_string();
    }

    let int_part = parts[0];
    let mut frac_part = parts[1].to_string();

    frac_part = frac_part.trim_end_matches('0').to_string();

    if frac_part.is_empty() {
        // All zeros removed, return as integer
        int_part.to_string()
    } else {
        format!("{int_part}.{frac_part}")
    }
}

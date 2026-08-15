fn validate_ordered_identifiers(
    values: &[String],
    maximum: usize,
    path: &str,
    diagnostics: &mut Vec<super::model::Diagnostic>,
) {
    if values.len() > maximum {
        diagnostics.push(super::diagnostic(
            super::model::DiagnosticCode::Limit,
            path,
            "identifier list exceeds the admitted limit",
        ));
    }
    for value in values.iter().take(maximum) {
        if !super::identity::valid_identifier(value) {
            diagnostics.push(super::diagnostic(
                super::model::DiagnosticCode::InvalidValue,
                path,
                "identifier list contains a non-canonical value",
            ));
        }
    }
    let bounded = &values[..values.len().min(maximum)];
    if bounded.windows(ORDER_COMPARISON_WINDOW).any(|pair| pair[0] >= pair[1]) {
        diagnostics.push(super::diagnostic(
            super::model::DiagnosticCode::Duplicate,
            path,
            "identifier list must be sorted and unique",
        ));
    }
}

fn validate_capabilities(
    values: &[super::model::CapabilityClass],
    path: &str,
    diagnostics: &mut Vec<super::model::Diagnostic>,
) {
    validate_sorted_unique(values, super::model::MAXIMUM_CAPABILITY_CLASSES, path, diagnostics);
}

fn validate_sorted_unique<T: Ord>(
    values: &[T],
    maximum: usize,
    path: &str,
    diagnostics: &mut Vec<super::model::Diagnostic>,
) {
    if values.len() > maximum {
        diagnostics.push(super::diagnostic(
            super::model::DiagnosticCode::Limit,
            path,
            "list exceeds the admitted limit",
        ));
    }
    let bounded = &values[..values.len().min(maximum)];
    if bounded.windows(ORDER_COMPARISON_WINDOW).any(|pair| pair[0] >= pair[1]) {
        diagnostics.push(super::diagnostic(
            super::model::DiagnosticCode::Duplicate,
            path,
            "list must be sorted and unique",
        ));
    }
}

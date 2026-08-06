//! Renderer mapping coverage and fallback validation.

pub(super) fn validate_mappings(
    theme: &super::contract::AdaptiveThemeDefinition<'_>,
    diagnostics: &mut Vec<super::contract::AdaptiveThemeDiagnostic>,
) {
    let mut seen = std::collections::BTreeSet::new();
    for mapping in theme.renderer_mappings {
        if !seen.insert((mapping.renderer, mapping.family)) {
            diagnostics.push(diagnostic(
                super::contract::AdaptiveThemeDiagnosticCode::IncompleteRendererMapping,
                "adaptive renderer family mapping is duplicated",
            ));
        }
        validate_mapping_metadata(mapping, diagnostics);
    }
    for renderer in required_renderers() {
        for family in required_families() {
            if !seen.contains(&(renderer, family)) {
                diagnostics.push(diagnostic(
                    super::contract::AdaptiveThemeDiagnosticCode::IncompleteRendererMapping,
                    "adaptive renderer family mapping is missing",
                ));
            }
        }
    }
}

fn validate_mapping_metadata(
    mapping: &super::contract::AdaptiveRendererMapping<'_>,
    diagnostics: &mut Vec<super::contract::AdaptiveThemeDiagnostic>,
) {
    let is_valid = match mapping.status {
        super::model::AdaptiveMappingStatus::Supported => mapping.fallback.is_none(),
        super::model::AdaptiveMappingStatus::Approximated => mapping.fallback.is_some() && mapping.reason.is_some(),
        super::model::AdaptiveMappingStatus::DeferredToPlatform | super::model::AdaptiveMappingStatus::Unsupported => {
            mapping.reason.is_some()
        }
    };
    let is_unit_valid = mapping.family != super::model::AdaptiveTokenFamily::ControlMetrics || mapping.unit.is_some();
    if !is_valid || !is_unit_valid {
        diagnostics.push(diagnostic(
            super::contract::AdaptiveThemeDiagnosticCode::MissingMappingReason,
            "mapping status requires a reason, named fallback, or metric unit",
        ));
    }
}

fn required_renderers() -> [super::model::AdaptiveRendererTarget; 3] {
    [
        super::model::AdaptiveRendererTarget::Ratatui,
        super::model::AdaptiveRendererTarget::MeteoriteCss,
        super::model::AdaptiveRendererTarget::IcedPilot,
    ]
}

fn required_families() -> [super::model::AdaptiveTokenFamily; 7] {
    [
        super::model::AdaptiveTokenFamily::SurfaceElevation,
        super::model::AdaptiveTokenFamily::Typography,
        super::model::AdaptiveTokenFamily::ControlMetrics,
        super::model::AdaptiveTokenFamily::FocusIndicator,
        super::model::AdaptiveTokenFamily::InteractionState,
        super::model::AdaptiveTokenFamily::Contrast,
        super::model::AdaptiveTokenFamily::Motion,
    ]
}

fn diagnostic(
    code: super::contract::AdaptiveThemeDiagnosticCode,
    message: &str,
) -> super::contract::AdaptiveThemeDiagnostic {
    super::contract::AdaptiveThemeDiagnostic::new(code, "renderer-mapping", message)
}

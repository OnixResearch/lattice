use super::super::*;

fn generated() -> AdaptiveThemeDefinition<'static> {
    crate::generated_adaptive_theme_tokens::GENERATED_ADAPTIVE_THEME
}

#[test]
fn generated_adaptive_schema_has_complete_bounded_coverage() {
    let validation = validate_adaptive_theme(&generated());

    assert!(validation.is_valid());
}

#[test]
fn missing_duplicate_and_invalid_profiles_fail_before_export() {
    let base = generated();
    let missing_density = AdaptiveThemeDefinition {
        density_profiles: &base.density_profiles[..2],
        ..base
    };
    assert_code(&validate_adaptive_theme(&missing_density), AdaptiveThemeDiagnosticCode::MissingProfile);
    assert!(export_adaptive_ratatui(&missing_density).is_err());

    let mut surfaces = base.surfaces.to_vec();
    surfaces.push(base.surfaces[0]);
    let duplicate_surface = AdaptiveThemeDefinition {
        surfaces: &surfaces,
        ..base
    };
    assert_code(&validate_adaptive_theme(&duplicate_surface), AdaptiveThemeDiagnosticCode::DuplicateToken);

    let mut density_profiles = base.density_profiles.to_vec();
    density_profiles[0].minimum_target.amount = 0;
    let invalid_metric = AdaptiveThemeDefinition {
        density_profiles: &density_profiles,
        ..base
    };
    assert_code(&validate_adaptive_theme(&invalid_metric), AdaptiveThemeDiagnosticCode::InvalidMetric);

    density_profiles[0].minimum_target = ThemeMetric {
        amount: 2,
        unit: ThemeMetricUnit::CssPixel,
    };
    let invalid_unit = AdaptiveThemeDefinition {
        density_profiles: &density_profiles,
        ..base
    };
    assert_code(&validate_adaptive_theme(&invalid_unit), AdaptiveThemeDiagnosticCode::InvalidMetric);
}

#[test]
fn contradictory_state_policy_and_mapping_metadata_fail_closed() {
    let base = generated();
    let mut precedence = base.state_precedence.to_vec();
    precedence.swap(0, 4);
    let unsafe_precedence = AdaptiveThemeDefinition {
        state_precedence: &precedence,
        ..base
    };
    assert_code(&validate_adaptive_theme(&unsafe_precedence), AdaptiveThemeDiagnosticCode::InvalidStatePrecedence);

    precedence.pop();
    let incomplete_precedence = AdaptiveThemeDefinition {
        state_precedence: &precedence,
        ..base
    };
    assert_code(
        &validate_adaptive_theme(&incomplete_precedence),
        AdaptiveThemeDiagnosticCode::InvalidStatePrecedence,
    );

    let retained_mapping_count = base.renderer_mappings.len().saturating_sub(1);
    let missing_mapping = AdaptiveThemeDefinition {
        renderer_mappings: &base.renderer_mappings[..retained_mapping_count],
        ..base
    };
    assert_code(&validate_adaptive_theme(&missing_mapping), AdaptiveThemeDiagnosticCode::IncompleteRendererMapping);

    let mut mappings = base.renderer_mappings.to_vec();
    mappings[0].fallback = None;
    let unexplained_mapping = AdaptiveThemeDefinition {
        renderer_mappings: &mappings,
        ..base
    };
    assert_code(&validate_adaptive_theme(&unexplained_mapping), AdaptiveThemeDiagnosticCode::MissingMappingReason);
}

#[test]
fn incomplete_contrast_motion_typography_and_state_tokens_are_rejected() {
    let base = generated();
    let cases = [
        AdaptiveThemeDefinition {
            contrast_profiles: &base.contrast_profiles[..2],
            ..base
        },
        AdaptiveThemeDefinition {
            motion_profiles: &base.motion_profiles[..1],
            ..base
        },
        AdaptiveThemeDefinition {
            typography: &base.typography[..3],
            ..base
        },
        AdaptiveThemeDefinition {
            state_tokens: &base.state_tokens[..6],
            ..base
        },
    ];
    for candidate in cases {
        assert!(!validate_adaptive_theme(&candidate).is_valid());
    }
}

fn assert_code(validation: &AdaptiveThemeValidationReport, code: AdaptiveThemeDiagnosticCode) {
    assert!(validation.diagnostics.iter().any(|diagnostic| diagnostic.code == code));
}

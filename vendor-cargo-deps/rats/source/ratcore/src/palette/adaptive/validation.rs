//! Adaptive theme schema validation.

const METRIC_MAX: u16 = 64;
const DIAGNOSTIC_BOUND: usize = 32;

/// Validates complete adaptive token/profile and renderer mapping coverage.
#[must_use]
pub fn validate_adaptive_theme(
    theme: &super::contract::AdaptiveThemeDefinition<'_>,
) -> super::contract::AdaptiveThemeValidationReport {
    let mut diagnostics = Vec::with_capacity(DIAGNOSTIC_BOUND);
    if theme.generation == 0 {
        diagnostics.push(diagnostic(
            super::contract::AdaptiveThemeDiagnosticCode::MissingProfile,
            "generation",
            "adaptive theme generation must be nonzero",
        ));
    }
    validate_surfaces(theme, &mut diagnostics);
    validate_typography(theme, &mut diagnostics);
    validate_density(theme, &mut diagnostics);
    validate_contrast(theme, &mut diagnostics);
    validate_motion(theme, &mut diagnostics);
    validate_focus(theme, &mut diagnostics);
    validate_states(theme, &mut diagnostics);
    super::mapping_validation::validate_mappings(theme, &mut diagnostics);
    super::contract::AdaptiveThemeValidationReport { diagnostics }
}

fn validate_surfaces(
    theme: &super::contract::AdaptiveThemeDefinition<'_>,
    diagnostics: &mut Vec<super::contract::AdaptiveThemeDiagnostic>,
) {
    let mut seen = std::collections::BTreeSet::new();
    for token in theme.surfaces {
        if !seen.insert(token.level) {
            diagnostics.push(duplicate("surface"));
        }
    }
    for required in [
        super::model::SurfaceLevel::Base,
        super::model::SurfaceLevel::Raised,
        super::model::SurfaceLevel::Overlay,
        super::model::SurfaceLevel::Modal,
    ] {
        if !seen.contains(&required) {
            diagnostics.push(missing("surface"));
        }
    }
}

fn validate_typography(
    theme: &super::contract::AdaptiveThemeDefinition<'_>,
    diagnostics: &mut Vec<super::contract::AdaptiveThemeDiagnostic>,
) {
    let mut seen = std::collections::BTreeSet::new();
    for token in theme.typography {
        if !seen.insert(token.role) {
            diagnostics.push(duplicate("typography"));
        }
    }
    for required in [
        super::model::TypographyRole::Body,
        super::model::TypographyRole::Label,
        super::model::TypographyRole::Heading,
        super::model::TypographyRole::Code,
    ] {
        if !seen.contains(&required) {
            diagnostics.push(missing("typography"));
        }
    }
}

fn validate_density(
    theme: &super::contract::AdaptiveThemeDefinition<'_>,
    diagnostics: &mut Vec<super::contract::AdaptiveThemeDiagnostic>,
) {
    let mut seen = std::collections::BTreeSet::new();
    for profile in theme.density_profiles {
        if !seen.insert(profile.density) {
            diagnostics.push(duplicate("density"));
        }
        for metric in [profile.minimum_target, profile.gap, profile.padding] {
            if !is_logical_metric(metric) {
                diagnostics.push(diagnostic(
                    super::contract::AdaptiveThemeDiagnosticCode::InvalidMetric,
                    "density",
                    "density metrics must be bounded positive logical steps",
                ));
            }
        }
    }
    for required in [
        super::model::DensityPreference::Compact,
        super::model::DensityPreference::Comfortable,
        super::model::DensityPreference::Touch,
    ] {
        if !seen.contains(&required) {
            diagnostics.push(missing("density"));
        }
    }
}

fn validate_contrast(
    theme: &super::contract::AdaptiveThemeDefinition<'_>,
    diagnostics: &mut Vec<super::contract::AdaptiveThemeDiagnostic>,
) {
    let mut seen = std::collections::BTreeSet::new();
    for profile in theme.contrast_profiles {
        if !seen.insert(profile.contrast) {
            diagnostics.push(duplicate("contrast"));
        }
        let is_emphasis_valid =
            (1..=3).contains(&profile.text_emphasis) && (1..=3).contains(&profile.boundary_emphasis);
        let is_deferral_valid = profile.is_platform_color_deferred
            == (profile.contrast == super::model::ContrastPreference::ForcedPlatform);
        if !is_emphasis_valid || !is_deferral_valid {
            diagnostics.push(diagnostic(
                super::contract::AdaptiveThemeDiagnosticCode::InvalidMetric,
                "contrast",
                "contrast emphasis or forced-platform deferral is invalid",
            ));
        }
    }
    for required in [
        super::model::ContrastPreference::Standard,
        super::model::ContrastPreference::Increased,
        super::model::ContrastPreference::ForcedPlatform,
    ] {
        if !seen.contains(&required) {
            diagnostics.push(missing("contrast"));
        }
    }
}

fn validate_motion(
    theme: &super::contract::AdaptiveThemeDefinition<'_>,
    diagnostics: &mut Vec<super::contract::AdaptiveThemeDiagnostic>,
) {
    let seen = theme.motion_profiles.iter().copied().collect::<std::collections::BTreeSet<_>>();
    if seen.len() != theme.motion_profiles.len() {
        diagnostics.push(duplicate("motion"));
    }
    for required in [
        super::model::MotionPreference::Full,
        super::model::MotionPreference::Reduced,
    ] {
        if !seen.contains(&required) {
            diagnostics.push(missing("motion"));
        }
    }
}

fn validate_focus(
    theme: &super::contract::AdaptiveThemeDefinition<'_>,
    diagnostics: &mut Vec<super::contract::AdaptiveThemeDiagnostic>,
) {
    if !is_logical_metric(theme.focus.thickness) {
        diagnostics.push(diagnostic(
            super::contract::AdaptiveThemeDiagnosticCode::InvalidMetric,
            "focus",
            "focus thickness must be a bounded positive logical step",
        ));
    }
}

fn validate_states(
    theme: &super::contract::AdaptiveThemeDefinition<'_>,
    diagnostics: &mut Vec<super::contract::AdaptiveThemeDiagnostic>,
) {
    let token_states = theme.state_tokens.iter().map(|token| token.state).collect::<std::collections::BTreeSet<_>>();
    let precedence = theme.state_precedence.iter().copied().collect::<std::collections::BTreeSet<_>>();
    let required = required_states();
    if token_states.len() != theme.state_tokens.len() || token_states != required {
        diagnostics.push(duplicate_or_missing("state-token"));
    }
    if precedence.len() != theme.state_precedence.len() || precedence != required {
        diagnostics.push(invalid_precedence());
        return;
    }
    if !safety_precedes_interaction(theme.state_precedence) {
        diagnostics.push(invalid_precedence());
    }
}

fn is_logical_metric(metric: super::model::ThemeMetric) -> bool {
    (1..=METRIC_MAX).contains(&metric.amount) && metric.unit == super::model::ThemeMetricUnit::LogicalStep
}

fn safety_precedes_interaction(states: &[super::model::InteractionState]) -> bool {
    let disabled = states.iter().position(|state| *state == super::model::InteractionState::Disabled);
    let invalid = states.iter().position(|state| *state == super::model::InteractionState::Invalid);
    let pressed = states.iter().position(|state| *state == super::model::InteractionState::Pressed);
    let hovered = states.iter().position(|state| *state == super::model::InteractionState::Hovered);
    matches!((disabled, invalid, pressed, hovered), (Some(d), Some(i), Some(p), Some(h)) if d < p && d < h && i < p && i < h)
}

fn required_states() -> std::collections::BTreeSet<super::model::InteractionState> {
    [
        super::model::InteractionState::Base,
        super::model::InteractionState::Hovered,
        super::model::InteractionState::Pressed,
        super::model::InteractionState::Selected,
        super::model::InteractionState::Focused,
        super::model::InteractionState::Invalid,
        super::model::InteractionState::Disabled,
    ]
    .into_iter()
    .collect()
}

fn diagnostic(
    code: super::contract::AdaptiveThemeDiagnosticCode,
    subject: &str,
    message: &str,
) -> super::contract::AdaptiveThemeDiagnostic {
    super::contract::AdaptiveThemeDiagnostic::new(code, subject, message)
}

fn duplicate(subject: &str) -> super::contract::AdaptiveThemeDiagnostic {
    diagnostic(
        super::contract::AdaptiveThemeDiagnosticCode::DuplicateToken,
        subject,
        "adaptive token/profile identity is duplicated",
    )
}

fn missing(subject: &str) -> super::contract::AdaptiveThemeDiagnostic {
    diagnostic(
        super::contract::AdaptiveThemeDiagnosticCode::MissingProfile,
        subject,
        "required adaptive token/profile is missing",
    )
}

fn duplicate_or_missing(subject: &str) -> super::contract::AdaptiveThemeDiagnostic {
    diagnostic(
        super::contract::AdaptiveThemeDiagnosticCode::DuplicateToken,
        subject,
        "adaptive state token coverage is duplicated or incomplete",
    )
}

fn invalid_precedence() -> super::contract::AdaptiveThemeDiagnostic {
    diagnostic(
        super::contract::AdaptiveThemeDiagnosticCode::InvalidStatePrecedence,
        "state-precedence",
        "state precedence is incomplete, duplicated, or unsafe",
    )
}

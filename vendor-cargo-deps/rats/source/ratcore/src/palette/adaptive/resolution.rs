//! Adaptive profile, metric, state, and motion resolution.

const CSS_PIXEL_SCALE: u16 = 8;
const NATIVE_PIXEL_SCALE: u16 = 8;
const FAMILY_COUNT: usize = 7;

/// Converts theme motion preference into the existing animation policy input.
#[must_use]
pub const fn animation_reduced_motion_input(preference: super::contract::ThemePreference) -> bool {
    preference.is_reduced_motion_enabled()
}

/// Resolves one validated preference for a renderer target.
pub fn resolve_adaptive_theme<'a>(
    theme: &'a super::contract::AdaptiveThemeDefinition<'a>,
    current_preset: &'a str,
    preference: super::contract::ThemePreference,
    renderer: super::model::AdaptiveRendererTarget,
) -> Result<super::contract::AdaptiveThemeProjection<'a>, super::contract::AdaptiveThemeDiagnostic> {
    ensure_theme_and_preference(theme, preference)?;
    let contrast = theme
        .contrast_profiles
        .iter()
        .find(|profile| profile.contrast == preference.contrast)
        .copied()
        .ok_or_else(|| missing("contrast"))?;
    let mappings = mappings_for_renderer(theme, renderer);
    if mappings.len() != FAMILY_COUNT {
        return Err(missing("renderer-mapping"));
    }
    let control_metrics = project_control_metrics(theme, preference.density, renderer)?;
    Ok(super::contract::AdaptiveThemeProjection {
        preset_name: selected_preset(current_preset, preference.color_scheme),
        preference,
        control_metrics,
        contrast,
        mappings,
        is_reduced_motion_enabled: preference.is_reduced_motion_enabled(),
    })
}

/// Converts logical density metrics into explicit renderer units.
pub fn project_control_metrics(
    theme: &super::contract::AdaptiveThemeDefinition<'_>,
    density: super::model::DensityPreference,
    renderer: super::model::AdaptiveRendererTarget,
) -> Result<super::contract::RendererControlMetrics, super::contract::AdaptiveThemeDiagnostic> {
    let profile = theme
        .density_profiles
        .iter()
        .find(|profile| profile.density == density)
        .ok_or_else(|| missing("density"))?;
    let mapping = theme
        .renderer_mappings
        .iter()
        .find(|mapping| {
            mapping.renderer == renderer && mapping.family == super::model::AdaptiveTokenFamily::ControlMetrics
        })
        .ok_or_else(|| missing("control-mapping"))?;
    if mapping.status == super::model::AdaptiveMappingStatus::Unsupported {
        return Err(super::contract::AdaptiveThemeDiagnostic::new(
            super::contract::AdaptiveThemeDiagnosticCode::UnsupportedMapping,
            "control-mapping",
            mapping.reason.unwrap_or("renderer mapping is unsupported"),
        ));
    }
    let unit = mapping.unit.ok_or_else(|| missing("control-unit"))?;
    let scale = renderer_scale(renderer);
    Ok(super::contract::RendererControlMetrics {
        renderer,
        density,
        minimum_target: convert_metric(profile.minimum_target, unit, scale)?,
        gap: convert_metric(profile.gap, unit, scale)?,
        padding: convert_metric(profile.padding, unit, scale)?,
    })
}

/// Resolves shared state precedence while preserving safety/focus flags.
pub fn resolve_interaction_state(
    theme: &super::contract::AdaptiveThemeDefinition<'_>,
    input: super::contract::InteractionStateInput,
) -> Result<super::contract::ResolvedInteractionState, super::contract::AdaptiveThemeDiagnostic> {
    let validation = super::validation::validate_adaptive_theme(theme);
    if let Some(first) = validation.diagnostics.into_iter().next() {
        return Err(first);
    }
    let dominant = theme
        .state_precedence
        .iter()
        .copied()
        .find(|state| state_is_active(*state, input))
        .ok_or_else(|| missing("state-precedence"))?;
    let dominant_role = state_role(theme, dominant)?;
    Ok(super::contract::ResolvedInteractionState {
        dominant,
        dominant_role,
        is_disabled_visible: input.is_disabled,
        is_invalid_visible: input.is_invalid,
        is_selected_visible: input.is_selected,
        is_focus_visible: input.is_focused,
        focus_role: theme.focus.role,
    })
}

fn ensure_theme_and_preference(
    theme: &super::contract::AdaptiveThemeDefinition<'_>,
    preference: super::contract::ThemePreference,
) -> Result<(), super::contract::AdaptiveThemeDiagnostic> {
    let validation = super::validation::validate_adaptive_theme(theme);
    if let Some(first) = validation.diagnostics.into_iter().next() {
        return Err(first);
    }
    if preference.generation != theme.generation {
        return Err(super::contract::AdaptiveThemeDiagnostic::new(
            super::contract::AdaptiveThemeDiagnosticCode::StalePreference,
            "generation",
            "theme preference generation is stale",
        ));
    }
    if !theme.motion_profiles.contains(&preference.motion) {
        return Err(missing("motion"));
    }
    Ok(())
}

fn mappings_for_renderer<'a>(
    theme: &'a super::contract::AdaptiveThemeDefinition<'a>,
    renderer: super::model::AdaptiveRendererTarget,
) -> Vec<super::contract::AdaptiveRendererMapping<'a>> {
    let mut mappings = Vec::with_capacity(FAMILY_COUNT);
    mappings.extend(theme.renderer_mappings.iter().filter(|mapping| mapping.renderer == renderer).copied());
    mappings.sort_by_key(|mapping| mapping.family);
    mappings
}

fn state_is_active(state: super::model::InteractionState, input: super::contract::InteractionStateInput) -> bool {
    match state {
        super::model::InteractionState::Base => true,
        super::model::InteractionState::Hovered => input.is_hovered,
        super::model::InteractionState::Pressed => input.is_pressed,
        super::model::InteractionState::Selected => input.is_selected,
        super::model::InteractionState::Focused => input.is_focused,
        super::model::InteractionState::Invalid => input.is_invalid,
        super::model::InteractionState::Disabled => input.is_disabled,
    }
}

fn state_role(
    theme: &super::contract::AdaptiveThemeDefinition<'_>,
    state: super::model::InteractionState,
) -> Result<super::model::AdaptiveColorRole, super::contract::AdaptiveThemeDiagnostic> {
    theme
        .state_tokens
        .iter()
        .find(|token| token.state == state)
        .map(|token| token.role)
        .ok_or_else(|| missing("state-token"))
}

fn selected_preset(current: &str, preference: super::model::ColorSchemePreference) -> &str {
    match preference {
        super::model::ColorSchemePreference::Preset => current,
        super::model::ColorSchemePreference::Light => "rats-light",
        super::model::ColorSchemePreference::Dark => "rats-terminal-dark",
    }
}

fn renderer_scale(renderer: super::model::AdaptiveRendererTarget) -> u16 {
    match renderer {
        super::model::AdaptiveRendererTarget::Ratatui => 1,
        super::model::AdaptiveRendererTarget::MeteoriteCss => CSS_PIXEL_SCALE,
        super::model::AdaptiveRendererTarget::IcedPilot => NATIVE_PIXEL_SCALE,
    }
}

fn convert_metric(
    source: super::model::ThemeMetric,
    unit: super::model::ThemeMetricUnit,
    scale: u16,
) -> Result<super::model::ThemeMetric, super::contract::AdaptiveThemeDiagnostic> {
    let amount = source.amount.checked_mul(scale).ok_or_else(|| {
        super::contract::AdaptiveThemeDiagnostic::new(
            super::contract::AdaptiveThemeDiagnosticCode::InvalidMetric,
            "metric",
            "renderer metric conversion overflowed",
        )
    })?;
    Ok(super::model::ThemeMetric { amount, unit })
}

fn missing(subject: &str) -> super::contract::AdaptiveThemeDiagnostic {
    super::contract::AdaptiveThemeDiagnostic::new(
        super::contract::AdaptiveThemeDiagnosticCode::MissingProfile,
        subject,
        "required adaptive profile or mapping is missing",
    )
}

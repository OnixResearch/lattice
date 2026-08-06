//! Pure preference parsing and renderer-observation normalization.

/// Validates a raw explicit preference DTO.
pub fn accept_raw_theme_preference(
    raw: super::contract::RawThemePreference<'_>,
    expected_generation: u64,
) -> Result<super::contract::ThemePreference, super::contract::AdaptiveThemeDiagnostic> {
    validate_generation(raw.generation, expected_generation)?;
    Ok(super::contract::ThemePreference {
        color_scheme: parse_color_scheme(raw.color_scheme)?,
        density: parse_density(raw.density)?,
        contrast: parse_contrast(raw.contrast)?,
        motion: parse_motion(raw.motion)?,
        generation: raw.generation,
    })
}

/// Normalizes explicit renderer observations without consulting ambient state.
pub fn normalize_theme_observation(
    observation: super::contract::ThemePreferenceObservation,
    expected_generation: u64,
) -> Result<super::contract::ThemePreference, super::contract::AdaptiveThemeDiagnostic> {
    validate_generation(observation.generation, expected_generation)?;
    if observation.is_compact_requested && observation.is_touch_requested {
        return Err(contradictory("density"));
    }
    if observation.is_increased_contrast_requested && observation.is_forced_colors_observed {
        return Err(contradictory("contrast"));
    }
    let density = if observation.is_touch_requested {
        super::model::DensityPreference::Touch
    } else if observation.is_compact_requested {
        super::model::DensityPreference::Compact
    } else {
        super::model::DensityPreference::Comfortable
    };
    let contrast = if observation.is_forced_colors_observed {
        super::model::ContrastPreference::ForcedPlatform
    } else if observation.is_increased_contrast_requested {
        super::model::ContrastPreference::Increased
    } else {
        super::model::ContrastPreference::Standard
    };
    let motion = if observation.is_reduced_motion_observed {
        super::model::MotionPreference::Reduced
    } else {
        super::model::MotionPreference::Full
    };
    Ok(super::contract::ThemePreference {
        color_scheme: observation.color_scheme.unwrap_or(super::model::ColorSchemePreference::Preset),
        density,
        contrast,
        motion,
        generation: observation.generation,
    })
}

fn validate_generation(actual: u64, expected: u64) -> Result<(), super::contract::AdaptiveThemeDiagnostic> {
    if actual == expected && actual > 0 {
        return Ok(());
    }
    Err(super::contract::AdaptiveThemeDiagnostic::new(
        super::contract::AdaptiveThemeDiagnosticCode::StalePreference,
        "generation",
        format!("theme preference generation {actual} does not match {expected}"),
    ))
}

fn parse_color_scheme(
    value: &str,
) -> Result<super::model::ColorSchemePreference, super::contract::AdaptiveThemeDiagnostic> {
    match value {
        "preset" => Ok(super::model::ColorSchemePreference::Preset),
        "light" => Ok(super::model::ColorSchemePreference::Light),
        "dark" => Ok(super::model::ColorSchemePreference::Dark),
        _ => Err(unknown("color_scheme", value)),
    }
}

fn parse_density(value: &str) -> Result<super::model::DensityPreference, super::contract::AdaptiveThemeDiagnostic> {
    match value {
        "compact" => Ok(super::model::DensityPreference::Compact),
        "comfortable" => Ok(super::model::DensityPreference::Comfortable),
        "touch" => Ok(super::model::DensityPreference::Touch),
        _ => Err(unknown("density", value)),
    }
}

fn parse_contrast(value: &str) -> Result<super::model::ContrastPreference, super::contract::AdaptiveThemeDiagnostic> {
    match value {
        "standard" => Ok(super::model::ContrastPreference::Standard),
        "increased" => Ok(super::model::ContrastPreference::Increased),
        "forced-platform" => Ok(super::model::ContrastPreference::ForcedPlatform),
        _ => Err(unknown("contrast", value)),
    }
}

fn parse_motion(value: &str) -> Result<super::model::MotionPreference, super::contract::AdaptiveThemeDiagnostic> {
    match value {
        "full" => Ok(super::model::MotionPreference::Full),
        "reduced" => Ok(super::model::MotionPreference::Reduced),
        _ => Err(unknown("motion", value)),
    }
}

fn unknown(field: &str, value: &str) -> super::contract::AdaptiveThemeDiagnostic {
    super::contract::AdaptiveThemeDiagnostic::new(
        super::contract::AdaptiveThemeDiagnosticCode::UnknownPreference,
        field,
        format!("unknown {field} preference `{value}`"),
    )
}

fn contradictory(field: &str) -> super::contract::AdaptiveThemeDiagnostic {
    super::contract::AdaptiveThemeDiagnostic::new(
        super::contract::AdaptiveThemeDiagnosticCode::ContradictoryPreference,
        field,
        format!("contradictory {field} observations"),
    )
}

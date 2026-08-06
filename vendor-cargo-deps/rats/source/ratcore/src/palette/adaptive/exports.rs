//! Deterministic adaptive renderer exports and freshness checks.

const CSS_SCALE: u16 = 8;
const EXPORT_CAPACITY: usize = 2_048;

/// Exports generated Ratatui adaptive mapping metadata.
pub fn export_adaptive_ratatui(
    theme: &super::contract::AdaptiveThemeDefinition<'_>,
) -> Result<String, super::contract::AdaptiveThemeDiagnostic> {
    ensure_valid(theme)?;
    let mut output = String::with_capacity(EXPORT_CAPACITY);
    output.push_str("generator=rats-adaptive-theme-v1\nsource-blake3=");
    output.push_str(crate::generated_adaptive_theme_tokens::ADAPTIVE_THEME_SOURCE_BLAKE3);
    output.push_str("\nrenderer=ratatui\n");
    append_density(theme, super::model::AdaptiveRendererTarget::Ratatui, &mut output)?;
    append_mappings(theme, super::model::AdaptiveRendererTarget::Ratatui, &mut output);
    Ok(output)
}

/// Exports generated Meteorite adaptive CSS without changing default color CSS.
pub fn export_adaptive_meteorite_css(
    theme: &super::contract::AdaptiveThemeDefinition<'_>,
) -> Result<String, super::contract::AdaptiveThemeDiagnostic> {
    ensure_valid(theme)?;
    let mut output = String::with_capacity(EXPORT_CAPACITY);
    output.push_str("/* Generated from themes/rats-theme-tokens.ncl; source-blake3=");
    output.push_str(crate::generated_adaptive_theme_tokens::ADAPTIVE_THEME_SOURCE_BLAKE3);
    output.push_str("; adaptive-only additive tokens. */\n:root {\n");
    append_css_density(theme, &mut output)?;
    append_css_semantics(theme, &mut output);
    output.push_str("}\n\n@media (prefers-contrast: more) {\n  :root {\n    --rats-contrast-text-emphasis: 2;\n    --rats-contrast-boundary-emphasis: 3;\n  }\n}\n\n@media (forced-colors: active) {\n  :root {\n    --rats-focus-indicator-color: Highlight;\n    --rats-adaptive-text-color: CanvasText;\n    --rats-adaptive-surface-color: Canvas;\n  }\n}\n");
    Ok(output)
}

/// Exports bounded Iced-pilot capability metadata without Iced values.
pub fn export_adaptive_iced_pilot(
    theme: &super::contract::AdaptiveThemeDefinition<'_>,
) -> Result<String, super::contract::AdaptiveThemeDiagnostic> {
    ensure_valid(theme)?;
    let mut output = String::with_capacity(EXPORT_CAPACITY);
    output.push_str("generator=rats-adaptive-theme-v1\nsource-blake3=");
    output.push_str(crate::generated_adaptive_theme_tokens::ADAPTIVE_THEME_SOURCE_BLAKE3);
    output.push_str("\nrenderer=iced-pilot\nscope=experimental-mounted\n");
    append_mappings(theme, super::model::AdaptiveRendererTarget::IcedPilot, &mut output);
    Ok(output)
}

fn append_density(
    theme: &super::contract::AdaptiveThemeDefinition<'_>,
    renderer: super::model::AdaptiveRendererTarget,
    output: &mut String,
) -> Result<(), super::contract::AdaptiveThemeDiagnostic> {
    use std::fmt::Write as _;
    for density in required_densities() {
        let metrics = super::resolution::project_control_metrics(theme, density, renderer)?;
        let _ = writeln!(
            output,
            "density.{}.target={}cell gap={}cell padding={}cell",
            density_label(density),
            metrics.minimum_target.amount,
            metrics.gap.amount,
            metrics.padding.amount
        );
    }
    Ok(())
}

fn append_css_density(
    theme: &super::contract::AdaptiveThemeDefinition<'_>,
    output: &mut String,
) -> Result<(), super::contract::AdaptiveThemeDiagnostic> {
    use std::fmt::Write as _;
    for density in required_densities() {
        let metrics = super::resolution::project_control_metrics(
            theme,
            density,
            super::model::AdaptiveRendererTarget::MeteoriteCss,
        )?;
        let label = density_label(density);
        let _ = writeln!(output, "  --rats-density-{label}-target: {}px;", metrics.minimum_target.amount);
        let _ = writeln!(output, "  --rats-density-{label}-gap: {}px;", metrics.gap.amount);
        let _ = writeln!(output, "  --rats-density-{label}-padding: {}px;", metrics.padding.amount);
    }
    Ok(())
}

fn append_css_semantics(theme: &super::contract::AdaptiveThemeDefinition<'_>, output: &mut String) {
    use std::fmt::Write as _;
    let focus_pixels = theme.focus.thickness.amount.saturating_mul(CSS_SCALE);
    let _ = writeln!(output, "  --rats-focus-indicator-width: {focus_pixels}px;");
    for surface in theme.surfaces {
        let _ = writeln!(
            output,
            "  --rats-surface-{}-elevation: {};",
            surface_label(surface.level),
            elevation_value(surface.elevation)
        );
    }
    for token in theme.typography {
        let _ = writeln!(
            output,
            "  --rats-type-{}-emphasis: {};",
            typography_label(token.role),
            typography_value(token.emphasis)
        );
    }
}

fn append_mappings(
    theme: &super::contract::AdaptiveThemeDefinition<'_>,
    renderer: super::model::AdaptiveRendererTarget,
    output: &mut String,
) {
    use std::fmt::Write as _;
    let mut mappings =
        theme.renderer_mappings.iter().filter(|mapping| mapping.renderer == renderer).collect::<Vec<_>>();
    mappings.sort_by_key(|mapping| mapping.family);
    for mapping in mappings {
        let _ = writeln!(
            output,
            "mapping.{}={} fallback={} reason={}",
            family_label(mapping.family),
            status_label(mapping.status),
            mapping.fallback.unwrap_or("-"),
            mapping.reason.unwrap_or("-")
        );
    }
}

fn ensure_valid(
    theme: &super::contract::AdaptiveThemeDefinition<'_>,
) -> Result<(), super::contract::AdaptiveThemeDiagnostic> {
    let report = super::validation::validate_adaptive_theme(theme);
    match report.diagnostics.into_iter().next() {
        Some(first) => Err(first),
        None => Ok(()),
    }
}

pub(super) const fn density_label(value: super::model::DensityPreference) -> &'static str {
    match value {
        super::model::DensityPreference::Compact => "compact",
        super::model::DensityPreference::Comfortable => "comfortable",
        super::model::DensityPreference::Touch => "touch",
    }
}

pub(super) const fn status_label(value: super::model::AdaptiveMappingStatus) -> &'static str {
    match value {
        super::model::AdaptiveMappingStatus::Supported => "supported",
        super::model::AdaptiveMappingStatus::Approximated => "approximated",
        super::model::AdaptiveMappingStatus::DeferredToPlatform => "deferred-to-platform",
        super::model::AdaptiveMappingStatus::Unsupported => "unsupported",
    }
}

fn required_densities() -> [super::model::DensityPreference; 3] {
    [
        super::model::DensityPreference::Compact,
        super::model::DensityPreference::Comfortable,
        super::model::DensityPreference::Touch,
    ]
}

fn surface_label(value: super::model::SurfaceLevel) -> &'static str {
    match value {
        super::model::SurfaceLevel::Base => "base",
        super::model::SurfaceLevel::Raised => "raised",
        super::model::SurfaceLevel::Overlay => "overlay",
        super::model::SurfaceLevel::Modal => "modal",
    }
}

fn elevation_value(value: super::model::ElevationIntent) -> u8 {
    match value {
        super::model::ElevationIntent::Flat => 0,
        super::model::ElevationIntent::Low => 1,
        super::model::ElevationIntent::Medium => 2,
        super::model::ElevationIntent::High => 3,
    }
}

fn typography_label(value: super::model::TypographyRole) -> &'static str {
    match value {
        super::model::TypographyRole::Body => "body",
        super::model::TypographyRole::Label => "label",
        super::model::TypographyRole::Heading => "heading",
        super::model::TypographyRole::Code => "code",
    }
}

fn typography_value(value: super::model::TypographyEmphasis) -> &'static str {
    match value {
        super::model::TypographyEmphasis::Regular => "regular",
        super::model::TypographyEmphasis::Medium => "medium",
        super::model::TypographyEmphasis::Strong => "strong",
        super::model::TypographyEmphasis::Monospace => "monospace",
    }
}

fn family_label(value: super::model::AdaptiveTokenFamily) -> &'static str {
    match value {
        super::model::AdaptiveTokenFamily::SurfaceElevation => "surface-elevation",
        super::model::AdaptiveTokenFamily::Typography => "typography",
        super::model::AdaptiveTokenFamily::ControlMetrics => "control-metrics",
        super::model::AdaptiveTokenFamily::FocusIndicator => "focus-indicator",
        super::model::AdaptiveTokenFamily::InteractionState => "interaction-state",
        super::model::AdaptiveTokenFamily::Contrast => "contrast",
        super::model::AdaptiveTokenFamily::Motion => "motion",
    }
}

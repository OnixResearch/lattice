use super::super::*;

fn generated() -> AdaptiveThemeDefinition<'static> {
    crate::generated_adaptive_theme_tokens::GENERATED_ADAPTIVE_THEME
}

fn preference(motion: MotionPreference) -> ThemePreference {
    ThemePreference {
        color_scheme: ColorSchemePreference::Preset,
        density: DensityPreference::Comfortable,
        contrast: ContrastPreference::Standard,
        motion,
        generation: generated().generation,
    }
}

#[test]
fn disabled_invalid_and_focus_semantics_survive_hover_and_press() {
    let resolved = resolve_interaction_state(&generated(), InteractionStateInput {
        is_hovered: true,
        is_pressed: true,
        is_selected: true,
        is_focused: true,
        is_invalid: true,
        is_disabled: true,
    })
    .expect("generated state policy should resolve");

    assert_eq!(resolved.dominant, InteractionState::Disabled);
    assert_eq!(resolved.dominant_role, AdaptiveColorRole::TextMuted);
    assert!(resolved.is_disabled_visible);
    assert!(resolved.is_invalid_visible);
    assert!(resolved.is_selected_visible);
    assert!(resolved.is_focus_visible);
    assert_eq!(resolved.focus_role, AdaptiveColorRole::Primary);

    let invalid = resolve_interaction_state(&generated(), InteractionStateInput {
        is_hovered: true,
        is_pressed: true,
        is_focused: true,
        is_invalid: true,
        ..InteractionStateInput::default()
    })
    .expect("invalid state should resolve");
    assert_eq!(invalid.dominant, InteractionState::Invalid);
    assert!(invalid.is_focus_visible);
}

#[test]
fn density_metrics_use_explicit_renderer_units_for_all_mounted_profiles() {
    let compact = project_control_metrics(&generated(), DensityPreference::Compact, AdaptiveRendererTarget::Ratatui)
        .expect("terminal compact mapping should resolve");
    assert_eq!(compact.minimum_target.amount, 2);
    assert_eq!(compact.minimum_target.unit, ThemeMetricUnit::TerminalCell);

    let touch = project_control_metrics(&generated(), DensityPreference::Touch, AdaptiveRendererTarget::MeteoriteCss)
        .expect("CSS touch mapping should resolve");
    assert_eq!(touch.minimum_target.amount, 32);
    assert_eq!(touch.minimum_target.unit, ThemeMetricUnit::CssPixel);

    let iced = project_control_metrics(&generated(), DensityPreference::Comfortable, AdaptiveRendererTarget::IcedPilot)
        .expect("mounted Iced pilot metrics should resolve");
    assert_eq!(iced.minimum_target.amount, 24);
    assert_eq!(iced.minimum_target.unit, ThemeMetricUnit::NativeLogicalPixel);
}

#[test]
fn renderer_projection_keeps_approximation_deferral_and_experimental_scope_visible() {
    let theme = generated();
    let ratatui = resolve_adaptive_theme(
        &theme,
        "custom-preset",
        preference(MotionPreference::Full),
        AdaptiveRendererTarget::Ratatui,
    )
    .expect("Ratatui profile should resolve");
    assert_eq!(ratatui.preset_name, "custom-preset");
    assert!(ratatui.mappings.iter().any(|mapping| mapping.status == AdaptiveMappingStatus::Approximated));

    let mut forced = preference(MotionPreference::Full);
    forced.color_scheme = ColorSchemePreference::Light;
    forced.contrast = ContrastPreference::ForcedPlatform;
    let meteorite = resolve_adaptive_theme(&theme, "custom-preset", forced, AdaptiveRendererTarget::MeteoriteCss)
        .expect("Meteorite forced-color profile should resolve");
    assert_eq!(meteorite.preset_name, "rats-light");
    assert!(meteorite.contrast.is_platform_color_deferred);
    assert!(meteorite.mappings.iter().any(|mapping| {
        mapping.family == AdaptiveTokenFamily::Contrast && mapping.status == AdaptiveMappingStatus::DeferredToPlatform
    }));

    let iced = resolve_adaptive_theme(
        &theme,
        "custom-preset",
        preference(MotionPreference::Full),
        AdaptiveRendererTarget::IcedPilot,
    )
    .expect("mounted Iced pilot profile should resolve experimentally");
    assert!(iced.mappings.iter().any(|mapping| {
        mapping.family == AdaptiveTokenFamily::Typography && mapping.status == AdaptiveMappingStatus::Approximated
    }));
    assert!(!iced.mappings.iter().any(|mapping| mapping.status == AdaptiveMappingStatus::Unsupported));
}

#[test]
fn motion_preference_delegates_to_existing_animation_completion_policy() {
    let intent = crate::animation_parity::AnimationIntent::new(
        "adaptive.motion",
        "fade",
        "theme.preview",
        std::time::Duration::from_millis(100),
    )
    .expect("positive duration should validate")
    .with_reduced_motion("snap");

    let full = crate::animation_parity::advance_animation(
        &intent,
        std::time::Duration::ZERO,
        animation_reduced_motion_input(preference(MotionPreference::Full)),
    );
    let reduced = crate::animation_parity::advance_animation(
        &intent,
        std::time::Duration::ZERO,
        animation_reduced_motion_input(preference(MotionPreference::Reduced)),
    );
    assert!(!full.completed);
    assert!(reduced.completed);
    assert_eq!(reduced.progress.get(), 10_000);

    let unsupported = crate::animation_parity::project_animation(
        &reduced,
        crate::animation_parity::AnimationRendererAdapter::Missing,
    );
    assert!(!unsupported.diagnostics.is_empty());
}

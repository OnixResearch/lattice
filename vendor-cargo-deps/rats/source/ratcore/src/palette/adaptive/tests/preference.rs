use super::super::*;

const GENERATION: u64 = 1;

#[test]
fn every_bounded_raw_preference_combination_is_explicit() {
    for color_scheme in ["preset", "light", "dark"] {
        for density in ["compact", "comfortable", "touch"] {
            for contrast in ["standard", "increased", "forced-platform"] {
                for motion in ["full", "reduced"] {
                    let accepted = accept_raw_theme_preference(
                        RawThemePreference {
                            color_scheme,
                            density,
                            contrast,
                            motion,
                            generation: GENERATION,
                        },
                        GENERATION,
                    );
                    assert!(accepted.is_ok());
                }
            }
        }
    }
}

#[test]
fn unknown_missing_and_stale_raw_preferences_are_diagnostic() {
    for raw in [
        RawThemePreference {
            color_scheme: "auto",
            density: "comfortable",
            contrast: "standard",
            motion: "full",
            generation: GENERATION,
        },
        RawThemePreference {
            color_scheme: "dark",
            density: "",
            contrast: "standard",
            motion: "full",
            generation: GENERATION,
        },
        RawThemePreference {
            color_scheme: "dark",
            density: "comfortable",
            contrast: "extreme",
            motion: "full",
            generation: GENERATION,
        },
        RawThemePreference {
            color_scheme: "dark",
            density: "comfortable",
            contrast: "standard",
            motion: "automatic",
            generation: GENERATION,
        },
    ] {
        assert_eq!(
            accept_raw_theme_preference(raw, GENERATION).expect_err("unknown preference must fail").code,
            AdaptiveThemeDiagnosticCode::UnknownPreference
        );
    }

    let stale = RawThemePreference {
        color_scheme: "dark",
        density: "comfortable",
        contrast: "standard",
        motion: "full",
        generation: 2,
    };
    assert_eq!(
        accept_raw_theme_preference(stale, GENERATION).expect_err("stale preference must fail").code,
        AdaptiveThemeDiagnosticCode::StalePreference
    );
}

#[test]
fn renderer_observations_normalize_without_ambient_reads() {
    let observation = ThemePreferenceObservation {
        renderer: AdaptiveRendererTarget::MeteoriteCss,
        color_scheme: Some(ColorSchemePreference::Dark),
        is_compact_requested: false,
        is_touch_requested: true,
        is_increased_contrast_requested: false,
        is_forced_colors_observed: true,
        is_reduced_motion_observed: true,
        generation: GENERATION,
    };
    let accepted = normalize_theme_observation(observation, GENERATION).expect("explicit observation should normalize");
    assert_eq!(accepted.color_scheme, ColorSchemePreference::Dark);
    assert_eq!(accepted.density, DensityPreference::Touch);
    assert_eq!(accepted.contrast, ContrastPreference::ForcedPlatform);
    assert_eq!(accepted.motion, MotionPreference::Reduced);

    for renderer in [
        AdaptiveRendererTarget::Ratatui,
        AdaptiveRendererTarget::MeteoriteCss,
        AdaptiveRendererTarget::IcedPilot,
    ] {
        let accepted = normalize_theme_observation(
            ThemePreferenceObservation {
                renderer,
                color_scheme: None,
                is_compact_requested: false,
                is_touch_requested: false,
                is_increased_contrast_requested: false,
                is_forced_colors_observed: false,
                is_reduced_motion_observed: false,
                generation: GENERATION,
            },
            GENERATION,
        )
        .expect("explicit defaults should normalize");
        assert_eq!(accepted.color_scheme, ColorSchemePreference::Preset);
        assert_eq!(accepted.density, DensityPreference::Comfortable);
        assert_eq!(accepted.contrast, ContrastPreference::Standard);
        assert_eq!(accepted.motion, MotionPreference::Full);
    }
}

#[test]
fn contradictory_observations_fail_without_fallback() {
    let mut observation = ThemePreferenceObservation {
        renderer: AdaptiveRendererTarget::Ratatui,
        color_scheme: None,
        is_compact_requested: true,
        is_touch_requested: true,
        is_increased_contrast_requested: false,
        is_forced_colors_observed: false,
        is_reduced_motion_observed: false,
        generation: GENERATION,
    };
    assert_eq!(
        normalize_theme_observation(observation, GENERATION)
            .expect_err("contradictory density must fail")
            .code,
        AdaptiveThemeDiagnosticCode::ContradictoryPreference
    );

    observation.is_compact_requested = false;
    observation.is_touch_requested = false;
    observation.is_increased_contrast_requested = true;
    observation.is_forced_colors_observed = true;
    assert_eq!(
        normalize_theme_observation(observation, GENERATION)
            .expect_err("contradictory contrast must fail")
            .code,
        AdaptiveThemeDiagnosticCode::ContradictoryPreference
    );
}

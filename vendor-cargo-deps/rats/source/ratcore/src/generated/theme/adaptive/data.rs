//! Generated adaptive theme schema values.

const LOGICAL_ONE: crate::theme_tokens::ThemeMetric = crate::theme_tokens::ThemeMetric {
    amount: 1,
    unit: crate::theme_tokens::ThemeMetricUnit::LogicalStep,
};
const LOGICAL_TWO: crate::theme_tokens::ThemeMetric = crate::theme_tokens::ThemeMetric {
    amount: 2,
    unit: crate::theme_tokens::ThemeMetricUnit::LogicalStep,
};
const LOGICAL_THREE: crate::theme_tokens::ThemeMetric = crate::theme_tokens::ThemeMetric {
    amount: 3,
    unit: crate::theme_tokens::ThemeMetricUnit::LogicalStep,
};
const LOGICAL_FOUR: crate::theme_tokens::ThemeMetric = crate::theme_tokens::ThemeMetric {
    amount: 4,
    unit: crate::theme_tokens::ThemeMetricUnit::LogicalStep,
};

const SURFACES: &[crate::theme_tokens::SurfaceToken] = &[
    crate::theme_tokens::SurfaceToken {
        level: crate::theme_tokens::SurfaceLevel::Base,
        elevation: crate::theme_tokens::ElevationIntent::Flat,
    },
    crate::theme_tokens::SurfaceToken {
        level: crate::theme_tokens::SurfaceLevel::Raised,
        elevation: crate::theme_tokens::ElevationIntent::Low,
    },
    crate::theme_tokens::SurfaceToken {
        level: crate::theme_tokens::SurfaceLevel::Overlay,
        elevation: crate::theme_tokens::ElevationIntent::Medium,
    },
    crate::theme_tokens::SurfaceToken {
        level: crate::theme_tokens::SurfaceLevel::Modal,
        elevation: crate::theme_tokens::ElevationIntent::High,
    },
];

const TYPOGRAPHY: &[crate::theme_tokens::TypographyToken] = &[
    crate::theme_tokens::TypographyToken {
        role: crate::theme_tokens::TypographyRole::Body,
        emphasis: crate::theme_tokens::TypographyEmphasis::Regular,
    },
    crate::theme_tokens::TypographyToken {
        role: crate::theme_tokens::TypographyRole::Label,
        emphasis: crate::theme_tokens::TypographyEmphasis::Medium,
    },
    crate::theme_tokens::TypographyToken {
        role: crate::theme_tokens::TypographyRole::Heading,
        emphasis: crate::theme_tokens::TypographyEmphasis::Strong,
    },
    crate::theme_tokens::TypographyToken {
        role: crate::theme_tokens::TypographyRole::Code,
        emphasis: crate::theme_tokens::TypographyEmphasis::Monospace,
    },
];

const DENSITY_PROFILES: &[crate::theme_tokens::DensityProfile] = &[
    crate::theme_tokens::DensityProfile {
        density: crate::theme_tokens::DensityPreference::Compact,
        minimum_target: LOGICAL_TWO,
        gap: LOGICAL_ONE,
        padding: LOGICAL_ONE,
    },
    crate::theme_tokens::DensityProfile {
        density: crate::theme_tokens::DensityPreference::Comfortable,
        minimum_target: LOGICAL_THREE,
        gap: LOGICAL_TWO,
        padding: LOGICAL_TWO,
    },
    crate::theme_tokens::DensityProfile {
        density: crate::theme_tokens::DensityPreference::Touch,
        minimum_target: LOGICAL_FOUR,
        gap: LOGICAL_TWO,
        padding: LOGICAL_THREE,
    },
];

const CONTRAST_PROFILES: &[crate::theme_tokens::ContrastProfile] = &[
    crate::theme_tokens::ContrastProfile {
        contrast: crate::theme_tokens::ContrastPreference::Standard,
        text_emphasis: 1,
        boundary_emphasis: 1,
        is_platform_color_deferred: false,
    },
    crate::theme_tokens::ContrastProfile {
        contrast: crate::theme_tokens::ContrastPreference::Increased,
        text_emphasis: 2,
        boundary_emphasis: 3,
        is_platform_color_deferred: false,
    },
    crate::theme_tokens::ContrastProfile {
        contrast: crate::theme_tokens::ContrastPreference::ForcedPlatform,
        text_emphasis: 3,
        boundary_emphasis: 3,
        is_platform_color_deferred: true,
    },
];

const MOTION_PROFILES: &[crate::theme_tokens::MotionPreference] = &[
    crate::theme_tokens::MotionPreference::Full,
    crate::theme_tokens::MotionPreference::Reduced,
];

const STATE_TOKENS: &[crate::theme_tokens::InteractionStateToken] = &[
    state_token(crate::theme_tokens::InteractionState::Base, crate::theme_tokens::AdaptiveColorRole::Text),
    state_token(crate::theme_tokens::InteractionState::Hovered, crate::theme_tokens::AdaptiveColorRole::Primary),
    state_token(crate::theme_tokens::InteractionState::Pressed, crate::theme_tokens::AdaptiveColorRole::Primary),
    state_token(crate::theme_tokens::InteractionState::Selected, crate::theme_tokens::AdaptiveColorRole::Primary),
    state_token(crate::theme_tokens::InteractionState::Focused, crate::theme_tokens::AdaptiveColorRole::Primary),
    state_token(crate::theme_tokens::InteractionState::Invalid, crate::theme_tokens::AdaptiveColorRole::Error),
    state_token(crate::theme_tokens::InteractionState::Disabled, crate::theme_tokens::AdaptiveColorRole::TextMuted),
];

const STATE_PRECEDENCE: &[crate::theme_tokens::InteractionState] = &[
    crate::theme_tokens::InteractionState::Disabled,
    crate::theme_tokens::InteractionState::Invalid,
    crate::theme_tokens::InteractionState::Pressed,
    crate::theme_tokens::InteractionState::Selected,
    crate::theme_tokens::InteractionState::Hovered,
    crate::theme_tokens::InteractionState::Focused,
    crate::theme_tokens::InteractionState::Base,
];

const fn state_token(
    state: crate::theme_tokens::InteractionState,
    role: crate::theme_tokens::AdaptiveColorRole,
) -> crate::theme_tokens::InteractionStateToken {
    crate::theme_tokens::InteractionStateToken { state, role }
}

/// Generated adaptive theme definition.
pub const GENERATED_ADAPTIVE_THEME: crate::theme_tokens::AdaptiveThemeDefinition<'static> =
    crate::theme_tokens::AdaptiveThemeDefinition {
        generation: 1,
        surfaces: SURFACES,
        typography: TYPOGRAPHY,
        density_profiles: DENSITY_PROFILES,
        contrast_profiles: CONTRAST_PROFILES,
        motion_profiles: MOTION_PROFILES,
        focus: crate::theme_tokens::FocusIndicatorToken {
            role: crate::theme_tokens::AdaptiveColorRole::Primary,
            thickness: LOGICAL_TWO,
            visibility: crate::theme_tokens::FocusVisibility::KeyboardOrProgrammatic,
        },
        state_tokens: STATE_TOKENS,
        state_precedence: STATE_PRECEDENCE,
        renderer_mappings: super::mappings::ADAPTIVE_RENDERER_MAPPINGS,
    };

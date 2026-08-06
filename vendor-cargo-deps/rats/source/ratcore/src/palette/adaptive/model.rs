//! Plain adaptive theme definitions and resolved values.

/// Existing preset color-scheme selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ColorSchemePreference {
    /// Keep the caller-selected preset.
    Preset,
    /// Select the built-in light preset.
    Light,
    /// Select the built-in dark preset.
    Dark,
}

/// Bounded control-density axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DensityPreference {
    /// Dense keyboard-oriented controls.
    Compact,
    /// Default desktop controls.
    Comfortable,
    /// Larger pointer/touch targets.
    Touch,
}

/// Bounded contrast axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ContrastPreference {
    /// Preset contrast.
    Standard,
    /// Stronger text and boundaries.
    Increased,
    /// Let platform forced-color values own authored color selection.
    ForcedPlatform,
}

/// Explicit user motion preference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MotionPreference {
    /// Normal animation policy.
    Full,
    /// Existing reduced-motion policy is enabled.
    Reduced,
}

/// Unit carried by an adaptive metric.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMetricUnit {
    /// Renderer-neutral bounded intent step.
    LogicalStep,
    /// Terminal cell.
    TerminalCell,
    /// CSS logical pixel.
    CssPixel,
    /// Native renderer logical pixel.
    NativeLogicalPixel,
}

/// Positive bounded adaptive metric candidate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThemeMetric {
    /// Numeric amount, validated as `1..=64`.
    pub amount: u16,
    /// Explicit unit.
    pub unit: ThemeMetricUnit,
}

/// Semantic surface hierarchy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SurfaceLevel {
    /// Application background.
    Base,
    /// Raised panel or control.
    Raised,
    /// Temporary overlay.
    Overlay,
    /// Modal surface.
    Modal,
}

/// Renderer-neutral elevation emphasis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElevationIntent {
    /// No elevation emphasis.
    Flat,
    /// Low separation.
    Low,
    /// Medium overlay separation.
    Medium,
    /// Highest modal separation.
    High,
}

/// Semantic typography role.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TypographyRole {
    /// Primary body text.
    Body,
    /// Compact labels and metadata.
    Label,
    /// Section or surface headings.
    Heading,
    /// Code and fixed-width data.
    Code,
}

/// Typography emphasis independent of a concrete font.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypographyEmphasis {
    /// Normal emphasis.
    Regular,
    /// Medium label emphasis.
    Medium,
    /// Strong heading emphasis.
    Strong,
    /// Monospace intent without a font handle.
    Monospace,
}

/// Semantic color role reused from generated palette meaning.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdaptiveColorRole {
    /// Primary accent.
    Primary,
    /// Base surface.
    Surface,
    /// Primary text.
    Text,
    /// Muted text.
    TextMuted,
    /// Invalid/error state.
    Error,
}

/// State vocabulary used by deterministic precedence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum InteractionState {
    /// Base state.
    Base,
    /// Pointer hover.
    Hovered,
    /// Active press.
    Pressed,
    /// Selected value.
    Selected,
    /// Independently visible focus.
    Focused,
    /// Invalid safety state.
    Invalid,
    /// Disabled safety state.
    Disabled,
}

/// Focus indicator visibility policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusVisibility {
    /// Show for keyboard/programmatic focus rather than pointer hover.
    KeyboardOrProgrammatic,
    /// Always show accepted focus.
    Always,
}

/// Semantic family covered by each renderer mapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AdaptiveTokenFamily {
    /// Surface hierarchy and elevation.
    SurfaceElevation,
    /// Typography roles and emphasis.
    Typography,
    /// Density-aware control metrics.
    ControlMetrics,
    /// Focus indicator.
    FocusIndicator,
    /// Interaction state roles.
    InteractionState,
    /// Contrast behavior.
    Contrast,
    /// Motion preference bridge.
    Motion,
}

/// Renderer scope for adaptive mapping metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AdaptiveRendererTarget {
    /// Ratatui terminal projection.
    Ratatui,
    /// Meteorite CSS projection.
    MeteoriteCss,
    /// Experimental mounted Iced native pilot.
    IcedPilot,
}

/// Truthful renderer capability status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdaptiveMappingStatus {
    /// Directly representable at the semantic level.
    Supported,
    /// Represented by a named renderer-specific fallback.
    Approximated,
    /// Platform values intentionally supersede authored values.
    DeferredToPlatform,
    /// Not represented by this renderer scope.
    Unsupported,
}

/// Surface token record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SurfaceToken {
    /// Surface level.
    pub level: SurfaceLevel,
    /// Elevation intent.
    pub elevation: ElevationIntent,
}

/// Typography token record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TypographyToken {
    /// Typography role.
    pub role: TypographyRole,
    /// Semantic emphasis.
    pub emphasis: TypographyEmphasis,
}

/// Density-aware logical metrics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DensityProfile {
    /// Density identity.
    pub density: DensityPreference,
    /// Minimum control target intent.
    pub minimum_target: ThemeMetric,
    /// Inter-control gap intent.
    pub gap: ThemeMetric,
    /// Internal padding intent.
    pub padding: ThemeMetric,
}

/// Contrast profile semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContrastProfile {
    /// Contrast identity.
    pub contrast: ContrastPreference,
    /// Relative text emphasis in `1..=3`.
    pub text_emphasis: u8,
    /// Relative boundary emphasis in `1..=3`.
    pub boundary_emphasis: u8,
    /// Whether platform colors intentionally own color values.
    pub is_platform_color_deferred: bool,
}

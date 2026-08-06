//! Adaptive theme contract records and diagnostics.

/// Focus indicator semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FocusIndicatorToken {
    /// Semantic color role.
    pub role: super::model::AdaptiveColorRole,
    /// Logical thickness intent.
    pub thickness: super::model::ThemeMetric,
    /// Visibility policy.
    pub visibility: super::model::FocusVisibility,
}

/// Interaction-state semantic role.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InteractionStateToken {
    /// State identity.
    pub state: super::model::InteractionState,
    /// Semantic color role.
    pub role: super::model::AdaptiveColorRole,
}

/// Renderer capability mapping for one adaptive family.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdaptiveRendererMapping<'a> {
    /// Renderer target.
    pub renderer: super::model::AdaptiveRendererTarget,
    /// Adaptive token family.
    pub family: super::model::AdaptiveTokenFamily,
    /// Capability status.
    pub status: super::model::AdaptiveMappingStatus,
    /// Named approximation fallback, when required.
    pub fallback: Option<&'a str>,
    /// Limitation or deferral reason, when required.
    pub reason: Option<&'a str>,
    /// Renderer unit for metric mappings.
    pub unit: Option<super::model::ThemeMetricUnit>,
}

/// Complete typed adaptive theme definition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdaptiveThemeDefinition<'a> {
    /// Monotonic source generation.
    pub generation: u64,
    /// Required surface hierarchy.
    pub surfaces: &'a [super::model::SurfaceToken],
    /// Required typography roles.
    pub typography: &'a [super::model::TypographyToken],
    /// Compact, comfortable, and touch profiles.
    pub density_profiles: &'a [super::model::DensityProfile],
    /// Standard, increased, and forced-platform contrast profiles.
    pub contrast_profiles: &'a [super::model::ContrastProfile],
    /// Full and reduced motion profile coverage.
    pub motion_profiles: &'a [super::model::MotionPreference],
    /// Focus indicator semantics.
    pub focus: FocusIndicatorToken,
    /// State-to-role records.
    pub state_tokens: &'a [InteractionStateToken],
    /// Dominant-state precedence; focus is resolved independently.
    pub state_precedence: &'a [super::model::InteractionState],
    /// Per-renderer adaptive family mappings.
    pub renderer_mappings: &'a [AdaptiveRendererMapping<'a>],
}

/// Raw renderer/OS/browser facts supplied by an adapter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThemePreferenceObservation {
    /// Renderer that observed the facts.
    pub renderer: super::model::AdaptiveRendererTarget,
    /// Optional color-scheme observation; absent keeps the preset.
    pub color_scheme: Option<super::model::ColorSchemePreference>,
    /// Compact density was explicitly requested.
    pub is_compact_requested: bool,
    /// Touch density was explicitly requested.
    pub is_touch_requested: bool,
    /// Increased contrast was explicitly requested.
    pub is_increased_contrast_requested: bool,
    /// Platform forced-colors mode was observed.
    pub is_forced_colors_observed: bool,
    /// Reduced motion was observed.
    pub is_reduced_motion_observed: bool,
    /// Theme generation observed by the adapter.
    pub generation: u64,
}

/// Validated explicit theme preference.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThemePreference {
    /// Existing color-scheme/preset selection.
    pub color_scheme: super::model::ColorSchemePreference,
    /// Density profile.
    pub density: super::model::DensityPreference,
    /// Contrast profile.
    pub contrast: super::model::ContrastPreference,
    /// Motion profile.
    pub motion: super::model::MotionPreference,
    /// Theme generation observed at normalization.
    pub generation: u64,
}

impl ThemePreference {
    /// Returns the existing animation reduced-motion input.
    #[must_use]
    pub const fn is_reduced_motion_enabled(self) -> bool {
        matches!(self.motion, super::model::MotionPreference::Reduced)
    }
}

/// Untrusted compatibility DTO for explicit preference strings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RawThemePreference<'a> {
    /// `preset`, `light`, or `dark`.
    pub color_scheme: &'a str,
    /// `compact`, `comfortable`, or `touch`.
    pub density: &'a str,
    /// `standard`, `increased`, or `forced-platform`.
    pub contrast: &'a str,
    /// `full` or `reduced`.
    pub motion: &'a str,
    /// Theme generation observed by the caller.
    pub generation: u64,
}

/// Component state facts resolved by shared precedence.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct InteractionStateInput {
    /// Component is hovered.
    pub is_hovered: bool,
    /// Component is pressed.
    pub is_pressed: bool,
    /// Component is selected.
    pub is_selected: bool,
    /// Component has accepted focus.
    pub is_focused: bool,
    /// Component is invalid.
    pub is_invalid: bool,
    /// Component is disabled.
    pub is_disabled: bool,
}

/// Resolved state semantics; safety and focus remain independently visible.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolvedInteractionState {
    /// Dominant style state under shared precedence.
    pub dominant: super::model::InteractionState,
    /// Semantic role for the dominant state.
    pub dominant_role: super::model::AdaptiveColorRole,
    /// Disabled safety semantics remain active.
    pub is_disabled_visible: bool,
    /// Invalid safety semantics remain active.
    pub is_invalid_visible: bool,
    /// Selected semantics remain available to projections.
    pub is_selected_visible: bool,
    /// Focus is independently projectable.
    pub is_focus_visible: bool,
    /// Focus indicator role.
    pub focus_role: super::model::AdaptiveColorRole,
}

/// Renderer-unit density projection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RendererControlMetrics {
    /// Renderer target.
    pub renderer: super::model::AdaptiveRendererTarget,
    /// Density profile.
    pub density: super::model::DensityPreference,
    /// Minimum target in renderer units.
    pub minimum_target: super::model::ThemeMetric,
    /// Gap in renderer units.
    pub gap: super::model::ThemeMetric,
    /// Padding in renderer units.
    pub padding: super::model::ThemeMetric,
}

/// Supplied checked-in adaptive renderer artifacts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdaptiveGeneratedArtifacts<'a> {
    /// Ratatui mapping bytes.
    pub ratatui: &'a [u8],
    /// Meteorite CSS bytes.
    pub meteorite_css: &'a [u8],
    /// Iced-pilot capability mapping bytes.
    pub iced_pilot: &'a [u8],
}

/// Deterministic adaptive artifact identities.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdaptiveArtifactHashes {
    /// Ratatui mapping BLAKE3.
    pub ratatui: String,
    /// Meteorite CSS BLAKE3.
    pub meteorite_css: String,
    /// Iced-pilot mapping BLAKE3.
    pub iced_pilot: String,
}

/// Resolved adaptive profile for one renderer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdaptiveThemeProjection<'a> {
    /// Existing generated preset selected for colors.
    pub preset_name: &'a str,
    /// Validated preference.
    pub preference: ThemePreference,
    /// Renderer-unit control metrics.
    pub control_metrics: RendererControlMetrics,
    /// Selected contrast semantics.
    pub contrast: super::model::ContrastProfile,
    /// Mapping metadata for every adaptive family.
    pub mappings: Vec<AdaptiveRendererMapping<'a>>,
    /// Existing animation reduced-motion input.
    pub is_reduced_motion_enabled: bool,
}

/// Stable adaptive theme diagnostic code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AdaptiveThemeDiagnosticCode {
    /// Required profile or token is missing.
    MissingProfile,
    /// Token/profile identity appears more than once.
    DuplicateToken,
    /// Metric amount or unit is invalid.
    InvalidMetric,
    /// State precedence is incomplete or unsafe.
    InvalidStatePrecedence,
    /// Renderer family mapping is missing or duplicated.
    IncompleteRendererMapping,
    /// Mapping status omits a required fallback or reason.
    MissingMappingReason,
    /// Explicit observed preferences contradict each other.
    ContradictoryPreference,
    /// Raw preference vocabulary is unknown.
    UnknownPreference,
    /// Preference generation does not match the theme.
    StalePreference,
    /// Renderer mapping cannot project requested metrics.
    UnsupportedMapping,
    /// Generated adaptive artifact content is stale.
    StaleAdaptiveArtifact,
}

/// Deterministic adaptive theme diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdaptiveThemeDiagnostic {
    /// Stable code.
    pub code: AdaptiveThemeDiagnosticCode,
    /// Failing token/profile/artifact identity.
    pub subject: String,
    /// Human-readable detail.
    pub message: String,
}

impl AdaptiveThemeDiagnostic {
    pub(super) fn new(
        code: AdaptiveThemeDiagnosticCode,
        subject: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            subject: subject.into(),
            message: message.into(),
        }
    }
}

impl std::fmt::Display for AdaptiveThemeDiagnostic {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}: {}", self.subject, self.message)
    }
}

impl std::error::Error for AdaptiveThemeDiagnostic {}

/// Adaptive schema validation report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdaptiveThemeValidationReport {
    /// Deterministically ordered diagnostics.
    pub diagnostics: Vec<AdaptiveThemeDiagnostic>,
}

impl AdaptiveThemeValidationReport {
    /// Whether the schema is accepted.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.diagnostics.is_empty()
    }
}

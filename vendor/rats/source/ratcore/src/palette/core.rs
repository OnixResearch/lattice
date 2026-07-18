//! Shared Rats theme token validation and renderer projections.
//!
//! Theme source is authored in Nickel and checked in under
//! `themes/rats-theme-tokens.ncl`. Runtime code consumes checked-in generated
//! Rust/CSS projections and can use this module to validate semantic token
//! completeness, color syntax, variant uniqueness, renderer mappings, and
//! generated artifact freshness.

const REQUIRED_ROLES: &[&str] = &[
    "primary",
    "secondary",
    "background",
    "surface",
    "text",
    "text-muted",
    "success",
    "warning",
    "error",
];
const HEX_COLOR_LENGTH: usize = 7;
const HEX_PREFIX: char = '#';
const CSS_VAR_PREFIX: &str = "--rats-";
const SUPPORTED_THEME_GENERATOR: &str = "rats-theme-token-generator-v1";
const RENDERER_METEORITE_CSS: &str = "meteorite-css";
const RENDERER_RATATUI_RUST: &str = "ratatui-rust";
const THEME_ARTIFACT_ROOT: &str = "theme-artifacts";
const RENDERER_ARTIFACTS_PER_PRESET: usize = 2;
const PREVIEW_STATES: &[&str] = &[
    "semantic-colors",
    "emphasis",
    "variants",
    "disabled",
    "focused",
    "danger-warning-success",
];

mod generated;
mod validation;

pub use generated::{
    expected_theme_artifacts, theme_preview_matrix, validate_theme_artifact_freshness,
    validate_theme_preview_matrix,
};
/// Stable theme validation issue code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ThemeIssueCode {
    /// A required semantic token is missing.
    MissingToken,
    /// A color is not valid `#rrggbb` syntax.
    InvalidColor,
    /// A variant alias is duplicated.
    DuplicateVariant,
    /// Renderer mapping metadata is unsupported.
    UnsupportedRendererMapping,
    /// Generated artifacts do not match projected output.
    StaleGeneratedOutput,
    /// Generated artifact is missing.
    MissingGeneratedArtifact,
    /// Generator metadata is unsupported.
    UnsupportedGenerator,
}

/// Deterministic theme validation issue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThemeIssue {
    /// Stable issue code.
    pub code: ThemeIssueCode,
    /// Theme name or field associated with the issue.
    pub subject: String,
    /// Human-readable diagnostic.
    pub message: String,
}

impl ThemeIssue {
    fn new(code: ThemeIssueCode, subject: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code,
            subject: subject.into(),
            message: message.into(),
        }
    }
}

/// Theme validation report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThemeValidationReport {
    /// Deterministic validation issues.
    pub issues: Vec<ThemeIssue>,
}

impl ThemeValidationReport {
    /// Returns true when the theme set validates.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.issues.is_empty()
    }
}

/// Typed theme token source snapshot supplied by the imperative shell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThemeTokenSource {
    /// Source path or stable identity.
    pub path: String,
    /// Canonical source bytes.
    pub bytes: Vec<u8>,
    /// Generator identity expected to produce artifacts.
    pub generator: String,
}

impl ThemeTokenSource {
    /// Creates a theme token source snapshot.
    #[must_use]
    pub fn new(
        path: impl Into<String>,
        bytes: impl Into<Vec<u8>>,
        generator: impl Into<String>,
    ) -> Self {
        Self {
            path: path.into(),
            bytes: bytes.into(),
            generator: generator.into(),
        }
    }
}

/// Checked-in generated theme artifact supplied for freshness validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThemeGeneratedArtifact {
    /// Artifact path or stable identity.
    pub path: String,
    /// Renderer artifact label.
    pub renderer: String,
    /// Generator identity recorded in the artifact metadata.
    pub generator: String,
    /// BLAKE3 identity of the source bytes used to generate the artifact.
    pub source_blake3: String,
    /// Artifact content bytes.
    pub content: Vec<u8>,
}

impl ThemeGeneratedArtifact {
    /// Creates a generated theme artifact.
    #[must_use]
    pub fn new(
        path: impl Into<String>,
        renderer: impl Into<String>,
        generator: impl Into<String>,
        source_blake3: impl Into<String>,
        content: impl Into<Vec<u8>>,
    ) -> Self {
        Self {
            path: path.into(),
            renderer: renderer.into(),
            generator: generator.into(),
            source_blake3: source_blake3.into(),
            content: content.into(),
        }
    }
}

/// Theme preview matrix row for one renderer and preset.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThemePreviewRow {
    /// Theme preset name.
    pub theme_name: String,
    /// Renderer label.
    pub renderer: String,
    /// Preview state groups rendered by the showcase.
    pub states: Vec<String>,
    /// Semantic roles required by the preview.
    pub roles: Vec<String>,
}

impl ThemePreviewRow {
    /// Creates a theme preview row.
    #[must_use]
    pub fn new(
        theme_name: impl Into<String>,
        renderer: impl Into<String>,
        states: Vec<String>,
        roles: Vec<String>,
    ) -> Self {
        Self {
            theme_name: theme_name.into(),
            renderer: renderer.into(),
            states,
            roles,
        }
    }
}

/// Validates the checked-in generated theme presets.
#[must_use]
pub fn validate_generated_themes() -> ThemeValidationReport {
    validate_theme_presets(crate::generated_theme_tokens::GENERATED_THEME_PRESETS)
}

/// Validates generated theme presets.
#[must_use]
pub fn validate_theme_presets(
    presets: &[crate::generated_theme_tokens::GeneratedThemePreset],
) -> ThemeValidationReport {
    let mut issues = Vec::with_capacity(presets.len());
    let mut variants = std::collections::BTreeSet::new();
    for preset in presets {
        if !variants.insert(preset.variant) {
            issues.push(ThemeIssue::new(
                ThemeIssueCode::DuplicateVariant,
                preset.variant,
                "theme variant aliases must be unique",
            ));
        }
        issues.extend(validation::token_issues(preset));
        issues.extend(validation::css_freshness_issues(preset));
    }
    ThemeValidationReport { issues }
}

/// Exports a CSS custom property block for a generated theme preset.
#[must_use]
pub fn export_css(preset: &crate::generated_theme_tokens::GeneratedThemePreset) -> String {
    let mut css = format!(":root[data-rats-theme=\"{}\"] {{\n", preset.name);
    for color in preset.colors {
        css.push_str("  ");
        css.push_str(CSS_VAR_PREFIX);
        css.push_str(color.role);
        css.push_str(": ");
        css.push_str(color.hex);
        css.push_str(";\n");
    }
    css.push_str("}\n");
    css
}

/// Exports renderer-neutral role names preserved across ratatui and Meteorite.
#[must_use]
pub fn semantic_role_names(
    preset: &crate::generated_theme_tokens::GeneratedThemePreset,
) -> Vec<&'static str> {
    preset.colors.iter().map(|color| color.role).collect()
}

/// Computes the BLAKE3 identity for canonical theme source bytes.
#[must_use]
pub fn theme_source_blake3(source_bytes: &[u8]) -> String {
    crate::interaction_replay::blake3_hex(source_bytes)
}

/// Exports a deterministic ratatui-oriented token listing.
#[must_use]
pub fn export_ratatui_tokens(
    preset: &crate::generated_theme_tokens::GeneratedThemePreset,
) -> String {
    let mut output = format!("theme={}\n", preset.name);
    for color in preset.colors {
        output.push_str(color.role);
        output.push('=');
        output.push_str(color.hex);
        output.push('\n');
    }
    output
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

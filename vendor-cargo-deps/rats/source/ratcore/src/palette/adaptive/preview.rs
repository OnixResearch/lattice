//! Fixture-backed semantic adaptive theme preview evidence.

const VALID_FIXTURE_COUNT: usize = 7;
const TOTAL_FIXTURE_COUNT: usize = 8;
const RENDERER_COUNT: usize = 3;
const FAMILY_COUNT: usize = 7;

/// Evidence classification for adaptive previews.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdaptivePreviewEvidenceClass {
    /// Semantic profile/mapping evidence, never pixel equality.
    SemanticOnly,
}

/// One adaptive theme preview row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdaptiveThemePreviewRow {
    /// Stable fixture identity.
    pub fixture_id: String,
    /// Renderer capability scope.
    pub renderer: super::model::AdaptiveRendererTarget,
    /// Accepted preference; malformed fixtures have none.
    pub preference: Option<super::contract::ThemePreference>,
    /// Mapping status for every adaptive token family.
    pub mapping_statuses: Vec<super::model::AdaptiveMappingStatus>,
    /// Explicit malformed/unsupported diagnostics.
    pub diagnostics: Vec<super::contract::AdaptiveThemeDiagnostic>,
    /// Fresh generated artifact identity.
    pub artifact_blake3: String,
    /// Semantic-only evidence classification.
    pub evidence_class: AdaptivePreviewEvidenceClass,
}

/// Builds compact/comfortable/touch/contrast/motion/state and malformed rows.
#[must_use]
pub fn adaptive_theme_preview_matrix(
    theme: &super::contract::AdaptiveThemeDefinition<'_>,
    artifacts: super::contract::AdaptiveGeneratedArtifacts<'_>,
) -> Vec<AdaptiveThemePreviewRow> {
    let valid = preview_preferences(theme.generation);
    let hashes = super::artifacts::adaptive_artifact_hashes(artifacts);
    let row_bound = VALID_FIXTURE_COUNT.saturating_mul(RENDERER_COUNT).saturating_add(1);
    let mut rows = Vec::with_capacity(row_bound);
    for (fixture_id, preference) in valid {
        for renderer in required_renderers() {
            rows.push(valid_preview_row(fixture_id, preference, renderer, theme, &hashes));
        }
    }
    rows.push(malformed_preview_row(theme.generation, &hashes));
    rows
}

/// Validates profile coverage, mapping reasons, diagnostics, and fresh hashes.
#[must_use]
pub fn validate_adaptive_theme_previews(
    theme: &super::contract::AdaptiveThemeDefinition<'_>,
    artifacts: super::contract::AdaptiveGeneratedArtifacts<'_>,
    rows: &[AdaptiveThemePreviewRow],
) -> super::contract::AdaptiveThemeValidationReport {
    let mut diagnostics = super::validation::validate_adaptive_theme(theme).diagnostics;
    diagnostics.extend(super::artifacts::validate_adaptive_artifact_freshness(theme, artifacts).diagnostics);
    let expected_hashes = super::artifacts::adaptive_artifact_hashes(artifacts);
    let mut identities = std::collections::BTreeSet::new();
    for row in rows {
        let identity = (row.fixture_id.as_str(), row.renderer);
        if !identities.insert(identity) {
            diagnostics.push(evidence_diagnostic("duplicate preview row"));
        }
        if row.fixture_id != "adaptive.malformed" && row.mapping_statuses.len() != FAMILY_COUNT {
            diagnostics.push(evidence_diagnostic("preview mapping coverage is incomplete"));
        }
        let expected_hash = hash_for_renderer(&expected_hashes, row.renderer);
        if row.artifact_blake3 != expected_hash {
            diagnostics.push(super::contract::AdaptiveThemeDiagnostic::new(
                super::contract::AdaptiveThemeDiagnosticCode::StaleAdaptiveArtifact,
                row.fixture_id.clone(),
                "adaptive preview artifact hash is stale",
            ));
        }
        if row.fixture_id == "adaptive.malformed" {
            if row.preference.is_some() || row.diagnostics.is_empty() {
                diagnostics.push(evidence_diagnostic("malformed preview must retain its diagnostic"));
            }
        } else if row.preference.is_none() {
            diagnostics.push(evidence_diagnostic("valid adaptive preview is missing its preference"));
        }
    }
    for required in required_fixture_ids() {
        if !rows.iter().any(|row| row.fixture_id == required) {
            diagnostics.push(evidence_diagnostic("required adaptive preview is missing"));
        }
    }
    super::contract::AdaptiveThemeValidationReport { diagnostics }
}

fn valid_preview_row(
    fixture_id: &str,
    preference: super::contract::ThemePreference,
    renderer: super::model::AdaptiveRendererTarget,
    theme: &super::contract::AdaptiveThemeDefinition<'_>,
    hashes: &super::contract::AdaptiveArtifactHashes,
) -> AdaptiveThemePreviewRow {
    let diagnostics = super::resolution::resolve_adaptive_theme(theme, "rats-terminal-dark", preference, renderer)
        .err()
        .into_iter()
        .collect();
    AdaptiveThemePreviewRow {
        fixture_id: fixture_id.to_owned(),
        renderer,
        preference: Some(preference),
        mapping_statuses: mapping_statuses(theme, renderer),
        diagnostics,
        artifact_blake3: hash_for_renderer(hashes, renderer).to_owned(),
        evidence_class: AdaptivePreviewEvidenceClass::SemanticOnly,
    }
}

fn malformed_preview_row(generation: u64, hashes: &super::contract::AdaptiveArtifactHashes) -> AdaptiveThemePreviewRow {
    let rejected = super::preference::accept_raw_theme_preference(
        super::contract::RawThemePreference {
            color_scheme: "dark",
            density: "tiny",
            contrast: "standard",
            motion: "full",
            generation,
        },
        generation,
    );
    AdaptiveThemePreviewRow {
        fixture_id: "adaptive.malformed".to_owned(),
        renderer: super::model::AdaptiveRendererTarget::MeteoriteCss,
        preference: None,
        mapping_statuses: Vec::with_capacity(0),
        diagnostics: rejected.err().into_iter().collect(),
        artifact_blake3: hashes.meteorite_css.clone(),
        evidence_class: AdaptivePreviewEvidenceClass::SemanticOnly,
    }
}

fn mapping_statuses(
    theme: &super::contract::AdaptiveThemeDefinition<'_>,
    renderer: super::model::AdaptiveRendererTarget,
) -> Vec<super::model::AdaptiveMappingStatus> {
    let mut mappings =
        theme.renderer_mappings.iter().filter(|mapping| mapping.renderer == renderer).collect::<Vec<_>>();
    mappings.sort_by_key(|mapping| mapping.family);
    mappings.iter().map(|mapping| mapping.status).collect()
}

fn preview_preferences(generation: u64) -> [(&'static str, super::contract::ThemePreference); VALID_FIXTURE_COUNT] {
    let base = preference(generation);
    [
        ("adaptive.compact", super::contract::ThemePreference {
            density: super::model::DensityPreference::Compact,
            ..base
        }),
        ("adaptive.comfortable", base),
        ("adaptive.touch", super::contract::ThemePreference {
            density: super::model::DensityPreference::Touch,
            ..base
        }),
        ("adaptive.increased-contrast", super::contract::ThemePreference {
            contrast: super::model::ContrastPreference::Increased,
            ..base
        }),
        ("adaptive.forced-platform", super::contract::ThemePreference {
            contrast: super::model::ContrastPreference::ForcedPlatform,
            ..base
        }),
        ("adaptive.reduced-motion", super::contract::ThemePreference {
            motion: super::model::MotionPreference::Reduced,
            ..base
        }),
        ("adaptive.interaction-state", base),
    ]
}

fn preference(generation: u64) -> super::contract::ThemePreference {
    super::contract::ThemePreference {
        color_scheme: super::model::ColorSchemePreference::Preset,
        density: super::model::DensityPreference::Comfortable,
        contrast: super::model::ContrastPreference::Standard,
        motion: super::model::MotionPreference::Full,
        generation,
    }
}

fn required_renderers() -> [super::model::AdaptiveRendererTarget; RENDERER_COUNT] {
    [
        super::model::AdaptiveRendererTarget::Ratatui,
        super::model::AdaptiveRendererTarget::MeteoriteCss,
        super::model::AdaptiveRendererTarget::IcedPilot,
    ]
}

fn required_fixture_ids() -> [&'static str; TOTAL_FIXTURE_COUNT] {
    [
        "adaptive.compact",
        "adaptive.comfortable",
        "adaptive.touch",
        "adaptive.increased-contrast",
        "adaptive.forced-platform",
        "adaptive.reduced-motion",
        "adaptive.interaction-state",
        "adaptive.malformed",
    ]
}

fn hash_for_renderer<'a>(
    hashes: &'a super::contract::AdaptiveArtifactHashes,
    renderer: super::model::AdaptiveRendererTarget,
) -> &'a str {
    match renderer {
        super::model::AdaptiveRendererTarget::Ratatui => &hashes.ratatui,
        super::model::AdaptiveRendererTarget::MeteoriteCss => &hashes.meteorite_css,
        super::model::AdaptiveRendererTarget::IcedPilot => &hashes.iced_pilot,
    }
}

fn evidence_diagnostic(message: &str) -> super::contract::AdaptiveThemeDiagnostic {
    super::contract::AdaptiveThemeDiagnostic::new(
        super::contract::AdaptiveThemeDiagnosticCode::MissingProfile,
        "adaptive-preview",
        message,
    )
}

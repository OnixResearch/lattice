use super::*;

/// Returns expected generated artifacts for a theme source and preset set.
#[must_use]
pub fn expected_theme_artifacts(
    source: &ThemeTokenSource,
    presets: &[crate::generated_theme_tokens::GeneratedThemePreset],
) -> Vec<ThemeGeneratedArtifact> {
    let source_hash = theme_source_blake3(&source.bytes);
    let artifact_count = presets.len().saturating_mul(RENDERER_ARTIFACTS_PER_PRESET);
    let mut artifacts = Vec::with_capacity(artifact_count);
    debug_assert!(!source.generator.is_empty());
    debug_assert!(artifact_count >= presets.len());
    for preset in presets {
        artifacts.push(ThemeGeneratedArtifact::new(
            theme_artifact_path(ThemeArtifactPathInput {
                theme_name: preset.name,
                renderer: RENDERER_METEORITE_CSS,
            }),
            RENDERER_METEORITE_CSS,
            SUPPORTED_THEME_GENERATOR,
            source_hash.clone(),
            export_css(preset).into_bytes(),
        ));
        artifacts.push(ThemeGeneratedArtifact::new(
            theme_artifact_path(ThemeArtifactPathInput {
                theme_name: preset.name,
                renderer: RENDERER_RATATUI_RUST,
            }),
            RENDERER_RATATUI_RUST,
            SUPPORTED_THEME_GENERATOR,
            source_hash.clone(),
            export_ratatui_tokens(preset).into_bytes(),
        ));
    }
    artifacts
}

/// Validates generated artifact freshness against typed source and presets.
#[must_use]
pub fn validate_theme_artifact_freshness(
    source: &ThemeTokenSource,
    artifacts: &[ThemeGeneratedArtifact],
    presets: &[crate::generated_theme_tokens::GeneratedThemePreset],
) -> ThemeValidationReport {
    let mut issues = validate_theme_presets(presets).issues;
    if source.generator != SUPPORTED_THEME_GENERATOR {
        issues.push(ThemeIssue::new(
            ThemeIssueCode::UnsupportedGenerator,
            source.path.clone(),
            format!("unsupported theme generator `{}`", source.generator),
        ));
    }
    let expected = expected_theme_artifacts(source, presets);
    let artifact_map = artifacts
        .iter()
        .map(|artifact| {
            (
                (artifact.path.as_str(), artifact.renderer.as_str()),
                artifact,
            )
        })
        .collect::<std::collections::BTreeMap<_, _>>();
    debug_assert!(artifact_map.len() <= artifacts.len());
    debug_assert!(expected.len() <= presets.len().saturating_mul(RENDERER_ARTIFACTS_PER_PRESET));
    for artifact in artifacts {
        issues.extend(single_artifact_metadata_issues(source, artifact));
    }
    for expected_artifact in &expected {
        match artifact_map.get(&(
            expected_artifact.path.as_str(),
            expected_artifact.renderer.as_str(),
        )) {
            Some(actual) => {
                if actual.content != expected_artifact.content {
                    issues.push(ThemeIssue::new(
                        ThemeIssueCode::StaleGeneratedOutput,
                        actual.path.clone(),
                        "generated theme artifact content is stale",
                    ));
                }
            }
            None => issues.push(ThemeIssue::new(
                ThemeIssueCode::MissingGeneratedArtifact,
                expected_artifact.path.clone(),
                "expected generated theme artifact is missing",
            )),
        }
    }
    ThemeValidationReport { issues }
}

/// Builds fixture-backed theme preview rows for each built-in renderer.
#[must_use]
pub fn theme_preview_matrix(
    presets: &[crate::generated_theme_tokens::GeneratedThemePreset],
) -> Vec<ThemePreviewRow> {
    let renderers = [RENDERER_RATATUI_RUST, RENDERER_METEORITE_CSS];
    let rows = presets
        .iter()
        .flat_map(|preset| {
            renderers.into_iter().map(move |renderer| {
                ThemePreviewRow::new(
                    preset.name,
                    renderer,
                    PREVIEW_STATES
                        .iter()
                        .map(|state| (*state).to_string())
                        .collect(),
                    REQUIRED_ROLES
                        .iter()
                        .map(|role| (*role).to_string())
                        .collect(),
                )
            })
        })
        .collect::<Vec<_>>();
    debug_assert_eq!(rows.len(), presets.len().saturating_mul(renderers.len()));
    debug_assert!(rows.iter().all(|row| !row.states.is_empty()));
    rows
}

/// Validates theme preview rows against presets and renderer mappings.
#[must_use]
pub fn validate_theme_preview_matrix(
    rows: &[ThemePreviewRow],
    presets: &[crate::generated_theme_tokens::GeneratedThemePreset],
) -> ThemeValidationReport {
    let preset_map = presets
        .iter()
        .map(|preset| (preset.name, preset))
        .collect::<std::collections::BTreeMap<_, _>>();
    let mut issues = Vec::with_capacity(rows.len());
    debug_assert!(preset_map.len() <= presets.len());
    debug_assert!(issues.capacity() >= rows.len());
    for row in rows {
        if !supported_artifact_renderer(&row.renderer) {
            issues.push(ThemeIssue::new(
                ThemeIssueCode::UnsupportedRendererMapping,
                row.renderer.clone(),
                "theme preview renderer mapping is unsupported",
            ));
        }
        match preset_map.get(row.theme_name.as_str()) {
            Some(preset) => issues.extend(preview_role_issues(row, preset)),
            None => issues.push(ThemeIssue::new(
                ThemeIssueCode::MissingToken,
                row.theme_name.clone(),
                "theme preview references an unknown preset",
            )),
        }
    }
    ThemeValidationReport { issues }
}

fn single_artifact_metadata_issues(
    source: &ThemeTokenSource,
    artifact: &ThemeGeneratedArtifact,
) -> Vec<ThemeIssue> {
    let mut issues = Vec::new();
    debug_assert!(!artifact.path.is_empty());
    debug_assert!(!artifact.renderer.is_empty());
    if !supported_artifact_renderer(&artifact.renderer) {
        issues.push(ThemeIssue::new(
            ThemeIssueCode::UnsupportedRendererMapping,
            artifact.renderer.clone(),
            "generated theme artifact renderer is unsupported",
        ));
    }
    if artifact.generator != SUPPORTED_THEME_GENERATOR {
        issues.push(ThemeIssue::new(
            ThemeIssueCode::UnsupportedGenerator,
            artifact.path.clone(),
            format!("unsupported artifact generator `{}`", artifact.generator),
        ));
    }
    let source_hash = theme_source_blake3(&source.bytes);
    if artifact.source_blake3 != source_hash {
        issues.push(ThemeIssue::new(
            ThemeIssueCode::StaleGeneratedOutput,
            artifact.path.clone(),
            "generated theme artifact source identity is stale",
        ));
    }
    issues
}

fn preview_role_issues(
    row: &ThemePreviewRow,
    preset: &crate::generated_theme_tokens::GeneratedThemePreset,
) -> Vec<ThemeIssue> {
    let roles = preset
        .colors
        .iter()
        .map(|color| color.role)
        .collect::<std::collections::BTreeSet<_>>();
    row.roles
        .iter()
        .filter(|role| !roles.contains(role.as_str()))
        .map(|role| {
            ThemeIssue::new(
                ThemeIssueCode::MissingToken,
                role.clone(),
                format!("theme preview role `{role}` is missing from preset"),
            )
        })
        .collect()
}

struct ThemeArtifactPathInput<'a> {
    theme_name: &'a str,
    renderer: &'a str,
}

fn theme_artifact_path(input: ThemeArtifactPathInput<'_>) -> String {
    let theme_name = input.theme_name;
    let renderer = input.renderer;
    format!("{THEME_ARTIFACT_ROOT}/{theme_name}/{renderer}.generated")
}

fn supported_artifact_renderer(renderer: &str) -> bool {
    matches!(renderer, RENDERER_METEORITE_CSS | RENDERER_RATATUI_RUST)
}

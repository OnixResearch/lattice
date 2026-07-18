use super::*;

#[test]
fn generated_presets_validate() {
    let outcome = validate_generated_themes();

    assert!(outcome.is_valid());
    assert!(
        semantic_role_names(&crate::generated_theme_tokens::GENERATED_THEME_PRESETS[0])
            .contains(&"primary")
    );
}

#[test]
fn css_export_preserves_semantic_roles() {
    let css = export_css(&crate::generated_theme_tokens::GENERATED_THEME_PRESETS[0]);

    assert!(css.contains("--rats-primary"));
    assert!(css.contains("--rats-warning"));
}

#[test]
fn invalid_theme_reports_missing_bad_color_duplicate_and_stale() {
    let bad_colors = &[crate::generated_theme_tokens::GeneratedColorToken {
        role: "primary",
        hex: "blue",
    }];
    let presets = &[
        crate::generated_theme_tokens::GeneratedThemePreset {
            name: "bad",
            variant: "dark",
            colors: bad_colors,
            css: "stale",
        },
        crate::generated_theme_tokens::GeneratedThemePreset {
            name: "duplicate",
            variant: "dark",
            colors: crate::generated_theme_tokens::RATS_LIGHT_COLORS,
            css: crate::generated_theme_tokens::RATS_LIGHT_CSS,
        },
    ];

    let outcome = validate_theme_presets(presets);

    assert!(!outcome.is_valid());
    assert!(!outcome.issues.is_empty());
    assert_issue(&outcome, ThemeIssueCode::MissingToken);
    assert_issue(&outcome, ThemeIssueCode::InvalidColor);
    assert_issue(&outcome, ThemeIssueCode::DuplicateVariant);
    assert_issue(&outcome, ThemeIssueCode::StaleGeneratedOutput);
}

#[test]
fn empty_renderer_projection_is_rejected() {
    let presets = &[crate::generated_theme_tokens::GeneratedThemePreset {
        name: "empty",
        variant: "empty",
        colors: &[],
        css: ":root[data-rats-theme=\"empty\"] {\n}\n",
    }];

    let outcome = validate_theme_presets(presets);

    assert_issue(&outcome, ThemeIssueCode::UnsupportedRendererMapping);
}

#[test]
fn fresh_theme_artifacts_validate_against_source_identity() {
    let source = theme_source();
    let artifacts = expected_theme_artifacts(
        &source,
        crate::generated_theme_tokens::GENERATED_THEME_PRESETS,
    );

    let outcome = validate_theme_artifact_freshness(
        &source,
        &artifacts,
        crate::generated_theme_tokens::GENERATED_THEME_PRESETS,
    );

    assert!(outcome.is_valid());
    assert_eq!(
        artifacts.len(),
        crate::generated_theme_tokens::GENERATED_THEME_PRESETS
            .len()
            .saturating_mul(RENDERER_ARTIFACTS_PER_PRESET)
    );
}

#[test]
fn theme_artifact_guard_rejects_missing_stale_and_unsupported_metadata() {
    let bad_source = ThemeTokenSource::new(
        "themes/rats-theme-tokens.ncl",
        theme_source_bytes(),
        "old-generator",
    );
    let mut artifacts = expected_theme_artifacts(
        &bad_source,
        crate::generated_theme_tokens::GENERATED_THEME_PRESETS,
    );
    artifacts.pop();
    if let Some(first) = artifacts.first_mut() {
        first.content = b"stale".to_vec();
        first.source_blake3 = "stale-source".to_string();
    }
    artifacts.push(ThemeGeneratedArtifact::new(
        "theme-artifacts/bad/browser.generated",
        "browser",
        "old-generator",
        "stale-source",
        b"bad".to_vec(),
    ));

    let outcome = validate_theme_artifact_freshness(
        &bad_source,
        &artifacts,
        crate::generated_theme_tokens::GENERATED_THEME_PRESETS,
    );

    assert!(!outcome.is_valid());
    assert!(outcome.issues.len() >= EXPECTED_ARTIFACT_GUARD_ISSUES);
    assert_issue(&outcome, ThemeIssueCode::MissingGeneratedArtifact);
    assert_issue(&outcome, ThemeIssueCode::StaleGeneratedOutput);
    assert_issue(&outcome, ThemeIssueCode::UnsupportedGenerator);
    assert_issue(&outcome, ThemeIssueCode::UnsupportedRendererMapping);
}

#[test]
fn theme_preview_matrix_covers_built_in_renderers_and_states() {
    let rows = theme_preview_matrix(crate::generated_theme_tokens::GENERATED_THEME_PRESETS);

    let outcome = validate_theme_preview_matrix(
        &rows,
        crate::generated_theme_tokens::GENERATED_THEME_PRESETS,
    );

    assert!(outcome.is_valid());
    assert!(rows.iter().any(|row| row.renderer == RENDERER_RATATUI_RUST));
    assert!(
        rows.iter()
            .any(|row| row.renderer == RENDERER_METEORITE_CSS)
    );
    assert!(
        rows.iter()
            .all(|row| row.states.len() == PREVIEW_STATES.len())
    );
}

#[test]
fn theme_preview_matrix_rejects_missing_roles_and_renderer_mapping() {
    let rows = vec![ThemePreviewRow::new(
        "rats-light",
        "browser",
        vec!["focused".to_string()],
        vec!["missing-role".to_string()],
    )];

    let outcome = validate_theme_preview_matrix(
        &rows,
        crate::generated_theme_tokens::GENERATED_THEME_PRESETS,
    );

    assert!(!outcome.is_valid());
    assert_issue(&outcome, ThemeIssueCode::MissingToken);
    assert_issue(&outcome, ThemeIssueCode::UnsupportedRendererMapping);
}

const EXPECTED_ARTIFACT_GUARD_ISSUES: usize = 4;

fn theme_source() -> ThemeTokenSource {
    ThemeTokenSource::new(
        "themes/rats-theme-tokens.ncl",
        theme_source_bytes(),
        SUPPORTED_THEME_GENERATOR,
    )
}

fn theme_source_bytes() -> Vec<u8> {
    include_bytes!("../../../themes/rats-theme-tokens.ncl").to_vec()
}

fn assert_issue(report: &ThemeValidationReport, code: ThemeIssueCode) {
    assert!(report.issues.iter().any(|issue| issue.code == code));
}

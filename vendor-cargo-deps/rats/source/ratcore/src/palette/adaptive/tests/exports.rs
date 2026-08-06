use super::super::*;

const BASELINE_CSS_BLAKE3: &str = "f09964062c16eda2587f9e3b147a4988822511f7897d6de622fc09ff302e4c92";
const BASELINE_RUST_BLAKE3: &str = "4807ff2a516e637750d2476480cbbfec739b76726066c0d63e8b8d0d23de8a83";
const ADAPTIVE_RATATUI_BLAKE3: &str = "580e59cc61aa762f4d951c611d1494a3305d0efb948e52545a7da4c1247bca3d";
const ADAPTIVE_CSS_BLAKE3: &str = "a82e901927d5c7c45abe167c5c1bbd490e85b475d03cd6b82de6403e282e9e6f";
const ADAPTIVE_ICED_BLAKE3: &str = "33ae467fec163e124fd4b3732cffecb1cedcb14cefed27d7479d9e359bf7c3b1";

fn generated() -> AdaptiveThemeDefinition<'static> {
    crate::generated_adaptive_theme_tokens::GENERATED_ADAPTIVE_THEME
}

fn artifacts() -> AdaptiveGeneratedArtifacts<'static> {
    AdaptiveGeneratedArtifacts {
        ratatui: crate::generated_adaptive_theme_tokens::ADAPTIVE_RATATUI_MAPPING.as_bytes(),
        meteorite_css: crate::generated_adaptive_theme_tokens::ADAPTIVE_METEORITE_CSS.as_bytes(),
        iced_pilot: crate::generated_adaptive_theme_tokens::ADAPTIVE_ICED_PILOT_MAPPING.as_bytes(),
    }
}

#[test]
fn adaptive_renderer_artifacts_are_fresh_and_truthfully_scoped() {
    let validation = validate_adaptive_artifact_freshness(&generated(), artifacts());
    assert!(validation.is_valid());

    assert_eq!(
        export_adaptive_ratatui(&generated()).expect("generated schema should export"),
        crate::generated_adaptive_theme_tokens::ADAPTIVE_RATATUI_MAPPING
    );
    assert_eq!(
        export_adaptive_meteorite_css(&generated()).expect("generated schema should export"),
        crate::generated_adaptive_theme_tokens::ADAPTIVE_METEORITE_CSS
    );
    assert_eq!(
        export_adaptive_iced_pilot(&generated()).expect("generated schema should export"),
        crate::generated_adaptive_theme_tokens::ADAPTIVE_ICED_PILOT_MAPPING
    );
    assert!(crate::generated_adaptive_theme_tokens::ADAPTIVE_METEORITE_CSS.contains("@media (forced-colors: active)"));
    assert!(crate::generated_adaptive_theme_tokens::ADAPTIVE_ICED_PILOT_MAPPING.contains("scope=experimental-mounted"));
}

#[test]
fn adaptive_artifact_hashes_are_deterministic_semantic_evidence() {
    let hashes = adaptive_artifact_hashes(artifacts());
    assert_eq!(hashes.ratatui, ADAPTIVE_RATATUI_BLAKE3);
    assert_eq!(hashes.meteorite_css, ADAPTIVE_CSS_BLAKE3);
    assert_eq!(hashes.iced_pilot, ADAPTIVE_ICED_BLAKE3);
}

#[test]
fn stale_adaptive_artifact_fails_without_mutating_checked_in_bytes() {
    let stale = AdaptiveGeneratedArtifacts {
        ratatui: b"stale",
        ..artifacts()
    };
    let validation = validate_adaptive_artifact_freshness(&generated(), stale);
    assert!(validation.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == AdaptiveThemeDiagnosticCode::StaleAdaptiveArtifact && diagnostic.subject == "ratatui"
    }));
}

#[test]
fn existing_default_color_artifacts_remain_byte_identical() {
    let existing_css = include_bytes!("../../../../../meteorite/crates/met-core/generated/theme-tokens.css");
    let existing_rust = include_bytes!("../../../generated/theme/tokens.rs");

    assert_eq!(crate::interaction_replay::blake3_hex(existing_css), BASELINE_CSS_BLAKE3);
    assert_eq!(crate::interaction_replay::blake3_hex(existing_rust), BASELINE_RUST_BLAKE3);
}

#[test]
fn adaptive_preview_matrix_covers_profiles_malformed_input_and_mapping_status() {
    let rows = adaptive_theme_preview_matrix(&generated(), artifacts());
    let validation = validate_adaptive_theme_previews(&generated(), artifacts(), &rows);
    assert!(validation.is_valid());
    assert!(rows.iter().any(|row| row.fixture_id == "adaptive.compact"));
    assert!(rows.iter().any(|row| row.fixture_id == "adaptive.touch"));
    assert!(rows.iter().any(|row| row.fixture_id == "adaptive.forced-platform"));
    assert!(rows.iter().any(|row| row.fixture_id == "adaptive.reduced-motion"));
    assert!(rows.iter().any(|row| {
        row.fixture_id == "adaptive.malformed" && row.preference.is_none() && !row.diagnostics.is_empty()
    }));
    assert!(rows.iter().any(|row| {
        row.renderer == AdaptiveRendererTarget::MeteoriteCss
            && row.mapping_statuses.contains(&AdaptiveMappingStatus::DeferredToPlatform)
    }));
    assert!(rows.iter().any(|row| {
        row.renderer == AdaptiveRendererTarget::IcedPilot
            && row.mapping_statuses.contains(&AdaptiveMappingStatus::Approximated)
            && !row.mapping_statuses.contains(&AdaptiveMappingStatus::Unsupported)
    }));
}

#[test]
fn preview_rejects_stale_hash_and_missing_mapping_reason() {
    let mut rows = adaptive_theme_preview_matrix(&generated(), artifacts());
    if let Some(first) = rows.first_mut() {
        first.artifact_blake3 = "stale".to_owned();
    }
    let stale = validate_adaptive_theme_previews(&generated(), artifacts(), &rows);
    assert!(
        stale
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.code == AdaptiveThemeDiagnosticCode::StaleAdaptiveArtifact })
    );

    let base = generated();
    let mut mappings = base.renderer_mappings.to_vec();
    mappings[0].reason = None;
    let malformed = AdaptiveThemeDefinition {
        renderer_mappings: &mappings,
        ..base
    };
    let validation = validate_adaptive_theme_previews(&malformed, artifacts(), &rows);
    assert!(
        validation
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.code == AdaptiveThemeDiagnosticCode::MissingMappingReason })
    );
}

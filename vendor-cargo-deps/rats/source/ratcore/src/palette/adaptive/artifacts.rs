//! Adaptive generated artifact freshness and semantic hashes.

/// Validates supplied renderer artifacts against source identity and exports.
#[must_use]
pub fn validate_adaptive_artifact_freshness(
    theme: &super::contract::AdaptiveThemeDefinition<'_>,
    actual: super::contract::AdaptiveGeneratedArtifacts<'_>,
) -> super::contract::AdaptiveThemeValidationReport {
    let mut diagnostics = super::validation::validate_adaptive_theme(theme).diagnostics;
    let actual_source_hash =
        crate::interaction_replay::blake3_hex(crate::generated_adaptive_theme_tokens::ADAPTIVE_THEME_SOURCE_BYTES);
    if actual_source_hash != crate::generated_adaptive_theme_tokens::ADAPTIVE_THEME_SOURCE_BLAKE3 {
        diagnostics.push(super::contract::AdaptiveThemeDiagnostic::new(
            super::contract::AdaptiveThemeDiagnosticCode::StaleAdaptiveArtifact,
            "theme-source",
            "adaptive generated source identity is stale",
        ));
    }
    compare_export(super::exports::export_adaptive_ratatui(theme), actual.ratatui, "ratatui", &mut diagnostics);
    compare_export(
        super::exports::export_adaptive_meteorite_css(theme),
        actual.meteorite_css,
        "meteorite-css",
        &mut diagnostics,
    );
    compare_export(
        super::exports::export_adaptive_iced_pilot(theme),
        actual.iced_pilot,
        "iced-pilot",
        &mut diagnostics,
    );
    super::contract::AdaptiveThemeValidationReport { diagnostics }
}

/// Returns BLAKE3 identities for supplied adaptive artifacts.
#[must_use]
pub fn adaptive_artifact_hashes(
    artifacts: super::contract::AdaptiveGeneratedArtifacts<'_>,
) -> super::contract::AdaptiveArtifactHashes {
    super::contract::AdaptiveArtifactHashes {
        ratatui: crate::interaction_replay::blake3_hex(artifacts.ratatui),
        meteorite_css: crate::interaction_replay::blake3_hex(artifacts.meteorite_css),
        iced_pilot: crate::interaction_replay::blake3_hex(artifacts.iced_pilot),
    }
}

fn compare_export(
    expected: Result<String, super::contract::AdaptiveThemeDiagnostic>,
    actual: &[u8],
    subject: &str,
    diagnostics: &mut Vec<super::contract::AdaptiveThemeDiagnostic>,
) {
    match expected {
        Ok(expected) if expected.as_bytes() == actual => {}
        Ok(_) => diagnostics.push(super::contract::AdaptiveThemeDiagnostic::new(
            super::contract::AdaptiveThemeDiagnosticCode::StaleAdaptiveArtifact,
            subject,
            "generated adaptive theme artifact is stale",
        )),
        Err(diagnostic) => diagnostics.push(diagnostic),
    }
}

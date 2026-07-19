use super::*;

pub(super) fn token_issues(
    preset: &crate::generated_theme_tokens::GeneratedThemePreset,
) -> Vec<ThemeIssue> {
    let roles = preset
        .colors
        .iter()
        .map(|color| color.role)
        .collect::<std::collections::BTreeSet<_>>();
    let issue_bound = REQUIRED_ROLES
        .len()
        .saturating_add(preset.colors.len())
        .saturating_add(1);
    let mut issues = Vec::with_capacity(issue_bound);
    debug_assert!(roles.len() <= preset.colors.len());
    debug_assert!(issues.capacity() >= issue_bound);
    for required in REQUIRED_ROLES {
        if !roles.contains(required) {
            issues.push(ThemeIssue::new(
                ThemeIssueCode::MissingToken,
                preset.name,
                format!("missing semantic role `{required}`"),
            ));
        }
    }
    for color in preset.colors {
        if !is_hex_color(color.hex) {
            issues.push(ThemeIssue::new(
                ThemeIssueCode::InvalidColor,
                color.role,
                format!("invalid color `{}`", color.hex),
            ));
        }
    }
    if preset.colors.is_empty() {
        issues.push(ThemeIssue::new(
            ThemeIssueCode::UnsupportedRendererMapping,
            preset.name,
            "theme must project at least one renderer color token",
        ));
    }
    issues
}

pub(super) fn css_freshness_issues(
    preset: &crate::generated_theme_tokens::GeneratedThemePreset,
) -> Vec<ThemeIssue> {
    let expected = export_css(preset);
    if expected == preset.css {
        Vec::new()
    } else {
        vec![ThemeIssue::new(
            ThemeIssueCode::StaleGeneratedOutput,
            preset.name,
            "generated CSS projection is stale",
        )]
    }
}

fn is_hex_color(value: &str) -> bool {
    value.len() == HEX_COLOR_LENGTH
        && value.starts_with(HEX_PREFIX)
        && value[1..].bytes().all(|byte| byte.is_ascii_hexdigit())
}

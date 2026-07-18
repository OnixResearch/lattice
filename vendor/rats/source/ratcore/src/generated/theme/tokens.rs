//! Checked-in generated Rats theme token projections.
//!
//! Source: `themes/rats-theme-tokens.ncl`. Runtime crates consume these stable
//! constants instead of evaluating Nickel.

/// Generated semantic color token.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GeneratedColorToken {
    /// Semantic role name.
    pub role: &'static str,
    /// Hex color value.
    pub hex: &'static str,
}

/// Generated theme preset projection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GeneratedThemePreset {
    /// Preset name.
    pub name: &'static str,
    /// Variant alias.
    pub variant: &'static str,
    /// Semantic colors.
    pub colors: &'static [GeneratedColorToken],
    /// CSS custom property block.
    pub css: &'static str,
}

/// Terminal dark colors.
pub const RATS_TERMINAL_DARK_COLORS: &[GeneratedColorToken] = &[
    GeneratedColorToken {
        role: "primary",
        hex: "#5ea1ff",
    },
    GeneratedColorToken {
        role: "secondary",
        hex: "#66d9ef",
    },
    GeneratedColorToken {
        role: "background",
        hex: "#101216",
    },
    GeneratedColorToken {
        role: "surface",
        hex: "#1b1f27",
    },
    GeneratedColorToken {
        role: "text",
        hex: "#f4f4f5",
    },
    GeneratedColorToken {
        role: "text-muted",
        hex: "#9ca3af",
    },
    GeneratedColorToken {
        role: "success",
        hex: "#22c55e",
    },
    GeneratedColorToken {
        role: "warning",
        hex: "#facc15",
    },
    GeneratedColorToken {
        role: "error",
        hex: "#ef4444",
    },
];

/// Light colors.
pub const RATS_LIGHT_COLORS: &[GeneratedColorToken] = &[
    GeneratedColorToken {
        role: "primary",
        hex: "#2563eb",
    },
    GeneratedColorToken {
        role: "secondary",
        hex: "#0891b2",
    },
    GeneratedColorToken {
        role: "background",
        hex: "#ffffff",
    },
    GeneratedColorToken {
        role: "surface",
        hex: "#f8fafc",
    },
    GeneratedColorToken {
        role: "text",
        hex: "#111827",
    },
    GeneratedColorToken {
        role: "text-muted",
        hex: "#6b7280",
    },
    GeneratedColorToken {
        role: "success",
        hex: "#16a34a",
    },
    GeneratedColorToken {
        role: "warning",
        hex: "#d97706",
    },
    GeneratedColorToken {
        role: "error",
        hex: "#dc2626",
    },
];

/// Terminal dark CSS projection.
pub const RATS_TERMINAL_DARK_CSS: &str = ":root[data-rats-theme=\"rats-terminal-dark\"] {\n  --rats-primary: #5ea1ff;\n  --rats-secondary: #66d9ef;\n  --rats-background: #101216;\n  --rats-surface: #1b1f27;\n  --rats-text: #f4f4f5;\n  --rats-text-muted: #9ca3af;\n  --rats-success: #22c55e;\n  --rats-warning: #facc15;\n  --rats-error: #ef4444;\n}\n";

/// Light CSS projection.
pub const RATS_LIGHT_CSS: &str = ":root[data-rats-theme=\"rats-light\"] {\n  --rats-primary: #2563eb;\n  --rats-secondary: #0891b2;\n  --rats-background: #ffffff;\n  --rats-surface: #f8fafc;\n  --rats-text: #111827;\n  --rats-text-muted: #6b7280;\n  --rats-success: #16a34a;\n  --rats-warning: #d97706;\n  --rats-error: #dc2626;\n}\n";

/// Generated theme presets.
pub const GENERATED_THEME_PRESETS: &[GeneratedThemePreset] = &[
    GeneratedThemePreset {
        name: "rats-terminal-dark",
        variant: "dark",
        colors: RATS_TERMINAL_DARK_COLORS,
        css: RATS_TERMINAL_DARK_CSS,
    },
    GeneratedThemePreset {
        name: "rats-light",
        variant: "light",
        colors: RATS_LIGHT_COLORS,
        css: RATS_LIGHT_CSS,
    },
];

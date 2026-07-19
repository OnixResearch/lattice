//! Shared theme for rat-widgets styling.

use ratatui::style::{Color, Modifier, Style};

/// Shared style configuration for rat-widgets.
///
/// Pass to any widget's `render_themed()` method to override the default
/// hardcoded colors.  All fields have sensible defaults that match the
/// original widget colors.
pub struct WidgetTheme {
    /// Primary accent (borders, highlights)
    pub primary: Color,
    /// Secondary accent
    pub secondary: Color,
    /// Success state
    pub success: Color,
    /// Warning state / selection highlight
    pub warning: Color,
    /// Error state
    pub error: Color,
    /// Primary text
    pub text: Color,
    /// Muted/secondary text
    pub text_muted: Color,
    /// Disabled/placeholder text
    pub text_disabled: Color,
    /// Focused border
    pub border_focused: Color,
    /// Normal border
    pub border_normal: Color,
    /// Background for overlays
    pub background: Color,
}

impl Default for WidgetTheme {
    fn default() -> Self {
        Self {
            primary: Color::Blue,
            secondary: Color::Cyan,
            success: Color::Green,
            warning: Color::Yellow,
            error: Color::Red,
            text: Color::White,
            text_muted: Color::DarkGray,
            text_disabled: Color::DarkGray,
            border_focused: Color::Blue,
            border_normal: Color::DarkGray,
            background: Color::Reset,
        }
    }
}

impl WidgetTheme {
    /// Style for highlighted/selected items: warning color + bold.
    pub fn highlight_style(&self) -> Style {
        Style::default()
            .fg(self.warning)
            .add_modifier(Modifier::BOLD)
    }

    /// Style for selected rows: text_disabled background, text foreground.
    pub fn selected_style(&self) -> Style {
        Style::default().bg(self.text_disabled).fg(self.text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_preserves_original_colors() {
        let t = WidgetTheme::default();
        assert_eq!(t.primary, Color::Blue);
        assert_eq!(t.secondary, Color::Cyan);
        assert_eq!(t.warning, Color::Yellow);
        assert_eq!(t.error, Color::Red);
        assert_eq!(t.success, Color::Green);
        assert_eq!(t.text, Color::White);
        assert_eq!(t.text_muted, Color::DarkGray);
        assert_eq!(t.text_disabled, Color::DarkGray);
    }

    #[test]
    fn highlight_style_uses_warning() {
        let t = WidgetTheme::default();
        let s = t.highlight_style();
        assert_eq!(s.fg, Some(Color::Yellow));
    }

    #[test]
    fn selected_style_uses_disabled_bg() {
        let t = WidgetTheme::default();
        let s = t.selected_style();
        assert_eq!(s.bg, Some(Color::DarkGray));
        assert_eq!(s.fg, Some(Color::White));
    }

    #[test]
    fn custom_theme_propagates() {
        let t = WidgetTheme {
            primary: Color::Magenta,
            warning: Color::Rgb(255, 165, 0),
            text: Color::Gray,
            text_disabled: Color::Black,
            ..Default::default()
        };
        assert_eq!(t.primary, Color::Magenta);
        let hl = t.highlight_style();
        assert_eq!(hl.fg, Some(Color::Rgb(255, 165, 0)));
        let sel = t.selected_style();
        assert_eq!(sel.bg, Some(Color::Black));
        assert_eq!(sel.fg, Some(Color::Gray));
    }
}

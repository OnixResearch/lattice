//! Tree visual styling configuration.

use ratatui::style::{Color, Style};

/// Controls the visual appearance of a tree widget.
#[derive(Debug, Clone)]
pub struct TreeStyle {
    /// Characters per indent level.
    pub indent_width: u16,
    /// Tee connector for non-last siblings (e.g., "├─").
    pub guide_tee: String,
    /// Corner connector for last sibling (e.g., "└─").
    pub guide_corner: String,
    /// Vertical pipe for continued parent lines (e.g., "│ ").
    pub guide_pipe: String,
    /// Blank space where no parent line continues (e.g., "  ").
    pub guide_space: String,
    /// Indicator for collapsed nodes with children (e.g., "▸ ").
    pub expand_indicator: String,
    /// Indicator for expanded nodes (e.g., "▾ ").
    pub collapse_indicator: String,
    /// Placeholder matching indicator width for leaf nodes (e.g., "  ").
    pub leaf_indicator: String,
    /// Style applied to node icons.
    pub icon_style: Style,
    /// Style applied to the row at the cursor position.
    pub selected_style: Style,
    /// Style applied to all other rows.
    pub normal_style: Style,
}

impl Default for TreeStyle {
    fn default() -> Self {
        Self {
            indent_width: 2,
            guide_tee: "├─".into(),
            guide_corner: "└─".into(),
            guide_pipe: "│ ".into(),
            guide_space: "  ".into(),
            expand_indicator: "▸ ".into(),
            collapse_indicator: "▾ ".into(),
            leaf_indicator: "  ".into(),
            icon_style: Style::default(),
            selected_style: Style::default().bg(Color::DarkGray).bold(),
            normal_style: Style::default(),
        }
    }
}

impl TreeStyle {
    pub fn with_indent_width(mut self, width: u16) -> Self {
        self.indent_width = width;
        self
    }

    pub fn with_guide_chars(
        mut self,
        tee: impl Into<String>,
        corner: impl Into<String>,
        pipe: impl Into<String>,
        space: impl Into<String>,
    ) -> Self {
        self.guide_tee = tee.into();
        self.guide_corner = corner.into();
        self.guide_pipe = pipe.into();
        self.guide_space = space.into();
        self
    }

    pub fn with_expand_indicator(mut self, s: impl Into<String>) -> Self {
        self.expand_indicator = s.into();
        self
    }

    pub fn with_collapse_indicator(mut self, s: impl Into<String>) -> Self {
        self.collapse_indicator = s.into();
        self
    }

    pub fn with_leaf_indicator(mut self, s: impl Into<String>) -> Self {
        self.leaf_indicator = s.into();
        self
    }

    pub fn with_icon_style(mut self, style: Style) -> Self {
        self.icon_style = style;
        self
    }

    pub fn with_selected_style(mut self, style: Style) -> Self {
        self.selected_style = style;
        self
    }

    pub fn with_normal_style(mut self, style: Style) -> Self {
        self.normal_style = style;
        self
    }
}

impl Default for super::OverlayStyle {
    fn default() -> Self {
        Self {
            border: ratatui::style::Style::new().fg(ratatui::style::Color::Gray),
            title: ratatui::style::Style::new()
                .fg(ratatui::style::Color::White)
                .add_modifier(ratatui::style::Modifier::BOLD),
            backdrop: ratatui::style::Style::new().bg(ratatui::style::Color::Black),
            fill: ratatui::style::Style::new().bg(ratatui::style::Color::Black),
        }
    }
}

impl super::OverlayStyle {
    /// Set the border style.
    pub fn with_border(mut self, border: ratatui::style::Style) -> Self {
        self.border = border;
        self
    }

    /// Set the title style.
    pub fn with_title(mut self, title: ratatui::style::Style) -> Self {
        self.title = title;
        self
    }

    /// Set the backdrop style.
    pub fn with_backdrop(mut self, backdrop: ratatui::style::Style) -> Self {
        self.backdrop = backdrop;
        self
    }

    /// Set the fill style.
    pub fn with_fill(mut self, fill: ratatui::style::Style) -> Self {
        self.fill = fill;
        self
    }
}

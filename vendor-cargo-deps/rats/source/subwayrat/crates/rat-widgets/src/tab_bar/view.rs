use ratatui::Frame;
use ratatui::layout::Alignment;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::widgets::Block;
use ratatui::widgets::Paragraph;

/// Ratatui-owned trigger rendering for one immutable shared tabs projection.
pub struct ProjectedTabBar<'a> {
    projection: &'a ratcore::headless_components::TabsProjection,
    selected_style: Style,
    focused_style: Style,
    inactive_style: Style,
    disabled_style: Style,
    separator: &'a str,
}

impl<'a> ProjectedTabBar<'a> {
    /// Creates a local visual wrapper without taking ownership of panel content.
    #[must_use]
    pub fn new(projection: &'a ratcore::headless_components::TabsProjection) -> Self {
        Self {
            projection,
            selected_style: Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            focused_style: Style::default().add_modifier(Modifier::UNDERLINED),
            inactive_style: Style::default(),
            disabled_style: Style::default().fg(Color::DarkGray),
            separator: " | ",
        }
    }

    /// Overrides the renderer-local selected style.
    #[must_use]
    pub const fn selected_style(mut self, style: Style) -> Self {
        self.selected_style = style;
        self
    }

    /// Overrides the renderer-local focused style.
    #[must_use]
    pub const fn focused_style(mut self, style: Style) -> Self {
        self.focused_style = style;
        self
    }

    /// Overrides the renderer-local inactive style.
    #[must_use]
    pub const fn inactive_style(mut self, style: Style) -> Self {
        self.inactive_style = style;
        self
    }

    /// Overrides the renderer-local disabled style.
    #[must_use]
    pub const fn disabled_style(mut self, style: Style) -> Self {
        self.disabled_style = style;
        self
    }

    /// Overrides the renderer-local separator text.
    #[must_use]
    pub const fn separator(mut self, separator: &'a str) -> Self {
        self.separator = separator;
        self
    }

    /// Renders projected trigger labels while the caller renders panel slots.
    pub fn render(&self, frame: &mut Frame<'_>, area: Rect, block: Option<Block<'_>>) {
        let mut paragraph = Paragraph::new(self.line()).alignment(Alignment::Center);
        if let Some(wrapper) = block {
            paragraph = paragraph.block(wrapper);
        }
        frame.render_widget(paragraph, area);
    }

    fn line(&self) -> Line<'a> {
        let mut spans = Vec::new();
        for tab in self.projection.tabs.iter().filter(|tab| !tab.is_hidden()) {
            if !spans.is_empty() {
                spans.push(Span::raw(self.separator));
            }
            let style = if tab.is_disabled() {
                self.disabled_style
            } else if tab.is_selected() {
                self.selected_style
            } else if tab.is_focused() {
                self.focused_style
            } else {
                self.inactive_style
            };
            spans.push(Span::styled(tab.label.clone(), style));
        }
        Line::from(spans)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn projected_view_uses_shared_visibility_selection_and_labels() {
        let mut state = ratcore::headless_components::canonical_tabs_state(
            ratcore::headless_components::TabsActivationPolicy::Manual,
        )
        .expect("fixture state");
        let _ =
            ratcore::headless_components::reduce_tabs(&mut state, ratcore::headless_components::TabsMessage::MoveNext);
        let projection = ratcore::headless_components::project_tabs(&state);
        let line = ProjectedTabBar::new(&projection).line();

        assert_eq!(line.spans.len(), 5);
        assert_eq!(line.spans[0].content.as_ref(), "Overview");
        assert_eq!(line.spans[2].content.as_ref(), "Details");
        assert_eq!(line.spans[4].content.as_ref(), "History");
        assert!(line.spans[0].style.add_modifier.contains(Modifier::BOLD));
        assert!(line.spans[2].style.add_modifier.contains(Modifier::UNDERLINED));
    }
}

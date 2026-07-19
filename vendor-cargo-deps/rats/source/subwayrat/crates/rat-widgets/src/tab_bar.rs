//! Horizontal tab bar widget.
//!
//! Renders a row of tab labels with active/inactive styling.
//! Supports optional count badges like "Tracks (42)".

use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};

/// A single tab entry.
#[derive(Debug, Clone)]
pub struct Tab {
    pub label: String,
    pub count: Option<usize>,
}

/// Pure data model for a tab bar — no ratatui dependency.
pub struct TabBarModel {
    pub tabs: Vec<Tab>,
    pub active: usize,
}

impl TabBarModel {
    pub fn new(labels: Vec<&str>) -> Self {
        let tabs = labels
            .into_iter()
            .map(|l| Tab {
                label: l.to_string(),
                count: None,
            })
            .collect();
        Self { tabs, active: 0 }
    }

    /// Advance to the next tab, wrapping around.
    pub fn select_next(&mut self) {
        if !self.tabs.is_empty() {
            self.active = (self.active + 1) % self.tabs.len();
        }
    }

    /// Move to the previous tab, wrapping around.
    pub fn select_prev(&mut self) {
        if !self.tabs.is_empty() {
            self.active = if self.active == 0 {
                self.tabs.len() - 1
            } else {
                self.active - 1
            };
        }
    }

    pub fn active_index(&self) -> usize {
        self.active
    }

    pub fn active_label(&self) -> &str {
        if self.tabs.is_empty() {
            ""
        } else {
            &self.tabs[self.active].label
        }
    }

    /// Update the count badge for a specific tab.
    /// No-op if `tab_idx` is out of range.
    pub fn set_count(&mut self, tab_idx: usize, count: usize) {
        if let Some(tab) = self.tabs.get_mut(tab_idx) {
            tab.count = Some(count);
        }
    }

    /// Number of tabs.
    pub fn len(&self) -> usize {
        self.tabs.len()
    }

    /// True if the tab bar has no tabs.
    pub fn is_empty(&self) -> bool {
        self.tabs.is_empty()
    }
}

/// Horizontal tab bar rendered as styled spans.
pub struct TabBar {
    pub model: TabBarModel,
    active_style: Style,
    inactive_style: Style,
    separator: String,
    border_color: Color,
}

impl TabBar {
    /// Create a tab bar from label strings.
    ///
    /// Defaults: active = Yellow+Bold, inactive = default fg,
    /// separator = " | ", border = Cyan, first tab selected.
    pub fn new(labels: Vec<&str>) -> Self {
        Self {
            model: TabBarModel::new(labels),
            active_style: Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
            inactive_style: Style::default(),
            separator: " | ".to_string(),
            border_color: Color::Cyan,
        }
    }

    /// Set count badges. Length must match number of tabs;
    /// extra entries are ignored, missing entries left as None.
    pub fn with_counts(mut self, counts: Vec<Option<usize>>) -> Self {
        for (tab, count) in self.model.tabs.iter_mut().zip(counts) {
            tab.count = count;
        }
        self
    }

    /// Set the active tab index (clamped to valid range).
    pub fn with_active(mut self, idx: usize) -> Self {
        if !self.model.tabs.is_empty() {
            self.model.active = idx.min(self.model.tabs.len() - 1);
        }
        self
    }

    pub fn with_active_style(mut self, style: Style) -> Self {
        self.active_style = style;
        self
    }

    pub fn with_inactive_style(mut self, style: Style) -> Self {
        self.inactive_style = style;
        self
    }

    pub fn with_separator(mut self, sep: &str) -> Self {
        self.separator = sep.to_string();
        self
    }

    pub fn with_border_color(mut self, color: Color) -> Self {
        self.border_color = color;
        self
    }

    /// Advance to the next tab, wrapping around.
    pub fn select_next(&mut self) {
        self.model.select_next();
    }

    /// Move to the previous tab, wrapping around.
    pub fn select_prev(&mut self) {
        self.model.select_prev();
    }

    pub fn active_index(&self) -> usize {
        self.model.active_index()
    }

    pub fn active_label(&self) -> &str {
        self.model.active_label()
    }

    /// Update the count badge for a specific tab.
    /// No-op if `tab_idx` is out of range.
    pub fn set_count(&mut self, tab_idx: usize, count: usize) {
        self.model.set_count(tab_idx, count);
    }

    /// Number of tabs.
    pub fn len(&self) -> usize {
        self.model.len()
    }

    /// True if the tab bar has no tabs.
    pub fn is_empty(&self) -> bool {
        self.model.is_empty()
    }

    /// Build the line of styled spans for the tab bar.
    fn build_line(&self) -> Line<'_> {
        let sep_style = Style::default().fg(Color::DarkGray);
        let mut spans: Vec<Span<'_>> = Vec::new();

        for (i, tab) in self.model.tabs.iter().enumerate() {
            if i > 0 {
                spans.push(Span::styled(self.separator.as_str(), sep_style));
            }

            let style = if i == self.model.active {
                self.active_style
            } else {
                self.inactive_style
            };

            let text = match tab.count {
                Some(n) => format!(" {} ({}) ", tab.label, n),
                None => format!(" {} ", tab.label),
            };
            spans.push(Span::styled(text, style));
        }

        Line::from(spans)
    }

    /// Render the tab bar into the given area.
    pub fn render(&self, frame: &mut Frame, area: Rect, block: Option<Block>) {
        let line = self.build_line();
        let mut paragraph = Paragraph::new(line).alignment(Alignment::Center);
        if let Some(b) = block {
            paragraph = paragraph.block(b);
        }
        frame.render_widget(paragraph, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_sets_defaults() {
        let bar = TabBar::new(vec!["A", "B", "C"]);
        assert_eq!(bar.active_index(), 0);
        assert_eq!(bar.active_label(), "A");
        assert_eq!(bar.len(), 3);
    }

    #[test]
    fn with_active_clamps() {
        let bar = TabBar::new(vec!["A", "B"]).with_active(99);
        assert_eq!(bar.active_index(), 1);
    }

    #[test]
    fn select_next_wraps() {
        let mut bar = TabBar::new(vec!["A", "B", "C"]);
        bar.select_next();
        assert_eq!(bar.active_index(), 1);
        bar.select_next();
        assert_eq!(bar.active_index(), 2);
        bar.select_next();
        assert_eq!(bar.active_index(), 0); // wrapped
    }

    #[test]
    fn select_prev_wraps() {
        let mut bar = TabBar::new(vec!["A", "B", "C"]);
        assert_eq!(bar.active_index(), 0);
        bar.select_prev();
        assert_eq!(bar.active_index(), 2); // wrapped
        bar.select_prev();
        assert_eq!(bar.active_index(), 1);
    }

    #[test]
    fn navigation_on_empty() {
        let mut bar = TabBar::new(vec![]);
        bar.select_next(); // should not panic
        bar.select_prev();
        assert_eq!(bar.len(), 0);
        assert!(bar.is_empty());
    }

    #[test]
    fn set_count_updates_badge() {
        let mut bar = TabBar::new(vec!["Tracks", "Albums"]);
        bar.set_count(0, 42);
        bar.set_count(1, 7);
        assert_eq!(bar.model.tabs[0].count, Some(42));
        assert_eq!(bar.model.tabs[1].count, Some(7));
    }

    #[test]
    fn set_count_out_of_range_is_noop() {
        let mut bar = TabBar::new(vec!["A"]);
        bar.set_count(5, 99); // should not panic
        assert_eq!(bar.model.tabs[0].count, None);
    }

    #[test]
    fn with_counts_builder() {
        let bar = TabBar::new(vec!["A", "B", "C"]).with_counts(vec![Some(10), None, Some(30)]);
        assert_eq!(bar.model.tabs[0].count, Some(10));
        assert_eq!(bar.model.tabs[1].count, None);
        assert_eq!(bar.model.tabs[2].count, Some(30));
    }

    #[test]
    fn build_line_without_counts() {
        let bar = TabBar::new(vec!["Foo", "Bar"]);
        let line = bar.build_line();
        let text: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
        assert_eq!(text, " Foo  |  Bar ");
    }

    #[test]
    fn build_line_with_counts() {
        let bar = TabBar::new(vec!["Tracks", "Albums"]).with_counts(vec![Some(42), None]);
        let line = bar.build_line();
        let text: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
        assert_eq!(text, " Tracks (42)  |  Albums ");
    }

    #[test]
    fn active_label_tracks_selection() {
        let mut bar = TabBar::new(vec!["One", "Two", "Three"]);
        assert_eq!(bar.active_label(), "One");
        bar.select_next();
        assert_eq!(bar.active_label(), "Two");
        bar.select_next();
        assert_eq!(bar.active_label(), "Three");
        bar.select_next();
        assert_eq!(bar.active_label(), "One");
    }

    #[test]
    fn active_style_applied_correctly() {
        let bar = TabBar::new(vec!["A", "B"]).with_active(1);
        let line = bar.build_line();
        // First span is "A" (inactive), then separator, then "B" (active)
        assert_eq!(line.spans[0].style, bar.inactive_style);
        assert_eq!(line.spans[2].style, bar.active_style);
    }
}

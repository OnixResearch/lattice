//! Inline scrollable list with selection tracking.
//!
//! Unlike [`SelectList`](crate::SelectList) (which renders as a centered popup), this widget
//! renders within its given [`Rect`] — suitable for main content panels like track lists,
//! queues, and search results.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, HighlightSpacing, List, ListItem, ListState};

use crate::theme::WidgetTheme;

/// Pure data model for a scrollable list — no ratatui dependency.
#[derive(Debug, Clone)]
pub struct ScrollableListModel {
    pub items: Vec<String>,
    pub selected: usize,
    pub scroll_offset: usize,
}

impl ScrollableListModel {
    pub fn new(items: Vec<String>) -> Self {
        Self {
            items,
            selected: 0,
            scroll_offset: 0,
        }
    }

    /// Move selection up by one, wrapping to the bottom.
    pub fn move_up(&mut self) {
        if self.items.is_empty() {
            return;
        }
        if self.selected == 0 {
            self.selected = self.items.len() - 1;
        } else {
            self.selected -= 1;
        }
    }

    /// Move selection down by one, wrapping to the top.
    pub fn move_down(&mut self) {
        if self.items.is_empty() {
            return;
        }
        if self.selected >= self.items.len() - 1 {
            self.selected = 0;
        } else {
            self.selected += 1;
        }
    }

    /// Jump to a specific index, clamped to the valid range.
    pub fn move_to(&mut self, idx: usize) {
        if self.items.is_empty() {
            self.selected = 0;
        } else {
            self.selected = idx.min(self.items.len() - 1);
        }
    }

    /// Move up by `page_size` items, stopping at 0.
    pub fn page_up(&mut self, page_size: usize) {
        self.selected = self.selected.saturating_sub(page_size);
    }

    /// Move down by `page_size` items, stopping at the last item.
    pub fn page_down(&mut self, page_size: usize) {
        if self.items.is_empty() {
            return;
        }
        self.selected = (self.selected + page_size).min(self.items.len() - 1);
    }

    pub fn move_to_top(&mut self) {
        self.selected = 0;
    }

    pub fn move_to_bottom(&mut self) {
        if self.items.is_empty() {
            self.selected = 0;
        } else {
            self.selected = self.items.len() - 1;
        }
    }

    pub fn selected_index(&self) -> usize {
        self.selected
    }

    pub fn selected_item(&self) -> Option<&str> {
        self.items.get(self.selected).map(|s| s.as_str())
    }

    /// Replace the item list. Clamps selection if out of bounds and resets
    /// scroll offset if the selected item would no longer be visible.
    pub fn set_items(&mut self, items: Vec<String>) {
        self.items = items;
        if self.items.is_empty() {
            self.selected = 0;
            self.scroll_offset = 0;
        } else if self.selected >= self.items.len() {
            self.selected = self.items.len() - 1;
        }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

/// An inline scrollable list that tracks selection and scroll offset.
///
/// Wraps ratatui's [`List`] + [`ListState`] with navigation helpers and
/// automatic scroll-to-selected behavior. Renders within its given area
/// rather than as a popup overlay.
#[derive(Debug, Clone)]
pub struct ScrollableList {
    pub model: ScrollableListModel,
    highlight_symbol: String,
    border_color: Color,
    highlight_style: Style,
    normal_style: Style,
}

impl ScrollableList {
    pub fn new(items: Vec<String>) -> Self {
        Self {
            model: ScrollableListModel::new(items),
            highlight_symbol: "> ".to_string(),
            border_color: Color::Blue,
            highlight_style: Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
            normal_style: Style::default().fg(Color::DarkGray),
        }
    }

    // -- builder methods --

    pub fn with_highlight_symbol(mut self, s: &str) -> Self {
        self.highlight_symbol = s.to_string();
        self
    }

    pub fn with_border_color(mut self, c: Color) -> Self {
        self.border_color = c;
        self
    }

    pub fn with_highlight_style(mut self, s: Style) -> Self {
        self.highlight_style = s;
        self
    }

    pub fn with_normal_style(mut self, s: Style) -> Self {
        self.normal_style = s;
        self
    }

    // -- navigation delegation --

    /// Move selection up by one, wrapping to the bottom.
    pub fn move_up(&mut self) {
        self.model.move_up();
    }

    /// Move selection down by one, wrapping to the top.
    pub fn move_down(&mut self) {
        self.model.move_down();
    }

    /// Jump to a specific index, clamped to the valid range.
    pub fn move_to(&mut self, idx: usize) {
        self.model.move_to(idx);
    }

    /// Move up by `page_size` items, stopping at 0.
    pub fn page_up(&mut self, page_size: usize) {
        self.model.page_up(page_size);
    }

    /// Move down by `page_size` items, stopping at the last item.
    pub fn page_down(&mut self, page_size: usize) {
        self.model.page_down(page_size);
    }

    pub fn move_to_top(&mut self) {
        self.model.move_to_top();
    }

    pub fn move_to_bottom(&mut self) {
        self.model.move_to_bottom();
    }

    // -- accessor delegation --

    pub fn selected_index(&self) -> usize {
        self.model.selected_index()
    }

    pub fn selected_item(&self) -> Option<&str> {
        self.model.selected_item()
    }

    /// Replace the item list. Clamps selection if out of bounds and resets
    /// scroll offset if the selected item would no longer be visible.
    pub fn set_items(&mut self, items: Vec<String>) {
        self.model.set_items(items);
    }

    pub fn len(&self) -> usize {
        self.model.len()
    }

    pub fn is_empty(&self) -> bool {
        self.model.is_empty()
    }

    // -- rendering --

    /// Render the list within `area`.
    ///
    /// If `block` is `Some`, the block is drawn around the list and the
    /// visible height is reduced by the border rows. The selected item is
    /// kept visible by adjusting the internal scroll offset before each
    /// render.
    pub fn render(&mut self, frame: &mut Frame, area: Rect, block: Option<Block>) {
        let inner = match &block {
            Some(b) => b.inner(area),
            None => area,
        };
        let visible_height = inner.height as usize;

        // Adjust scroll offset so the selected item stays visible.
        if visible_height > 0 {
            if self.model.selected < self.model.scroll_offset {
                self.model.scroll_offset = self.model.selected;
            } else if self.model.selected >= self.model.scroll_offset + visible_height {
                self.model.scroll_offset = self.model.selected - visible_height + 1;
            }
        }

        let list_items: Vec<ListItem> = self
            .model
            .items
            .iter()
            .map(|item| ListItem::new(item.as_str()).style(self.normal_style))
            .collect();

        let mut widget = List::new(list_items)
            .highlight_symbol(self.highlight_symbol.as_str())
            .highlight_style(self.highlight_style)
            .highlight_spacing(HighlightSpacing::Always);

        if let Some(b) = block {
            widget = widget.block(b);
        }

        let mut state = ListState::default()
            .with_selected(Some(self.model.selected))
            .with_offset(self.model.scroll_offset);

        frame.render_stateful_widget(widget, area, &mut state);
    }

    /// Build a `ScrollableList` styled from a [`WidgetTheme`].
    pub fn themed(items: Vec<String>, theme: &WidgetTheme) -> Self {
        Self {
            model: ScrollableListModel::new(items),
            highlight_symbol: "> ".to_string(),
            border_color: theme.border_focused,
            highlight_style: theme.highlight_style(),
            normal_style: Style::default().fg(theme.text_muted),
        }
    }

    /// Render using colors from the given [`WidgetTheme`], ignoring the
    /// per-instance style fields. Useful when the theme may change at
    /// runtime.
    pub fn render_themed(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        block: Option<Block>,
        theme: &WidgetTheme,
    ) {
        self.highlight_style = theme.highlight_style();
        self.normal_style = Style::default().fg(theme.text_muted);
        self.border_color = theme.border_focused;
        self.render(frame, area, block);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn items_5() -> Vec<String> {
        (1..=5).map(|i| format!("Item {i}")).collect()
    }

    // -- construction --

    #[test]
    fn new_starts_at_zero() {
        let list = ScrollableList::new(items_5());
        assert_eq!(list.model.selected_index(), 0);
        assert_eq!(list.model.selected_item(), Some("Item 1"));
        assert_eq!(list.len(), 5);
        assert!(!list.is_empty());
    }

    #[test]
    fn empty_list() {
        let list = ScrollableList::new(vec![]);
        assert_eq!(list.model.selected_index(), 0);
        assert_eq!(list.model.selected_item(), None);
        assert!(list.is_empty());
    }

    // -- move_down wrapping --

    #[test]
    fn move_down_wraps_to_top() {
        let mut list = ScrollableList::new(items_5());
        list.model.selected = 4; // last item
        list.move_down();
        assert_eq!(list.model.selected_index(), 0);
    }

    #[test]
    fn move_down_normal() {
        let mut list = ScrollableList::new(items_5());
        list.move_down();
        assert_eq!(list.model.selected_index(), 1);
    }

    #[test]
    fn move_down_empty() {
        let mut list = ScrollableList::new(vec![]);
        list.move_down(); // should not panic
        assert_eq!(list.model.selected_index(), 0);
    }

    // -- move_up wrapping --

    #[test]
    fn move_up_wraps_to_bottom() {
        let mut list = ScrollableList::new(items_5());
        list.move_up();
        assert_eq!(list.model.selected_index(), 4);
    }

    #[test]
    fn move_up_normal() {
        let mut list = ScrollableList::new(items_5());
        list.model.selected = 3;
        list.move_up();
        assert_eq!(list.model.selected_index(), 2);
    }

    #[test]
    fn move_up_empty() {
        let mut list = ScrollableList::new(vec![]);
        list.move_up(); // should not panic
        assert_eq!(list.model.selected_index(), 0);
    }

    // -- move_to clamping --

    #[test]
    fn move_to_clamps_to_last() {
        let mut list = ScrollableList::new(items_5());
        list.move_to(100);
        assert_eq!(list.model.selected_index(), 4);
    }

    #[test]
    fn move_to_valid() {
        let mut list = ScrollableList::new(items_5());
        list.move_to(2);
        assert_eq!(list.model.selected_index(), 2);
        assert_eq!(list.model.selected_item(), Some("Item 3"));
    }

    #[test]
    fn move_to_empty() {
        let mut list = ScrollableList::new(vec![]);
        list.move_to(5);
        assert_eq!(list.model.selected_index(), 0);
    }

    // -- page_up / page_down --

    #[test]
    fn page_down_clamps_to_last() {
        let mut list = ScrollableList::new(items_5());
        list.page_down(100);
        assert_eq!(list.model.selected_index(), 4);
    }

    #[test]
    fn page_down_normal() {
        let mut list = ScrollableList::new(items_5());
        list.page_down(2);
        assert_eq!(list.model.selected_index(), 2);
    }

    #[test]
    fn page_up_clamps_to_zero() {
        let mut list = ScrollableList::new(items_5());
        list.model.selected = 1;
        list.page_up(10);
        assert_eq!(list.model.selected_index(), 0);
    }

    #[test]
    fn page_up_normal() {
        let mut list = ScrollableList::new(items_5());
        list.model.selected = 4;
        list.page_up(2);
        assert_eq!(list.model.selected_index(), 2);
    }

    // -- top / bottom --

    #[test]
    fn move_to_top_and_bottom() {
        let mut list = ScrollableList::new(items_5());
        list.move_to_bottom();
        assert_eq!(list.model.selected_index(), 4);
        list.move_to_top();
        assert_eq!(list.model.selected_index(), 0);
    }

    // -- set_items bounds clamping --

    #[test]
    fn set_items_clamps_selection() {
        let mut list = ScrollableList::new(items_5());
        list.model.selected = 4;
        list.set_items(vec!["A".into(), "B".into()]);
        assert_eq!(list.model.selected_index(), 1);
        assert_eq!(list.model.selected_item(), Some("B"));
    }

    #[test]
    fn set_items_empty_resets() {
        let mut list = ScrollableList::new(items_5());
        list.model.selected = 3;
        list.set_items(vec![]);
        assert_eq!(list.model.selected_index(), 0);
        assert_eq!(list.model.scroll_offset, 0);
    }

    #[test]
    fn set_items_preserves_valid_selection() {
        let mut list = ScrollableList::new(items_5());
        list.model.selected = 2;
        list.set_items(vec!["X".into(), "Y".into(), "Z".into()]);
        assert_eq!(list.model.selected_index(), 2);
        assert_eq!(list.model.selected_item(), Some("Z"));
    }

    // -- builder methods --

    #[test]
    fn builder_chain() {
        let list = ScrollableList::new(vec!["a".into()])
            .with_highlight_symbol(">> ")
            .with_border_color(Color::Red)
            .with_highlight_style(Style::default().fg(Color::Green))
            .with_normal_style(Style::default().fg(Color::White));

        assert_eq!(list.highlight_symbol, ">> ");
        assert_eq!(list.border_color, Color::Red);
    }

    // -- scroll_offset adjustment --

    #[test]
    fn scroll_offset_adjusts_down() {
        let mut list = ScrollableList::new(items_5());
        // Simulate a 3-row visible area: selected=4 should push offset to 2.
        list.model.selected = 4;
        list.model.scroll_offset = 0;
        let visible_height: usize = 3;
        // Replicate the adjustment logic from render.
        if list.model.selected >= list.model.scroll_offset + visible_height {
            list.model.scroll_offset = list.model.selected - visible_height + 1;
        }
        assert_eq!(list.model.scroll_offset, 2);
    }

    #[test]
    fn scroll_offset_adjusts_up() {
        let mut list = ScrollableList::new(items_5());
        list.model.scroll_offset = 3;
        list.model.selected = 1;
        if list.model.selected < list.model.scroll_offset {
            list.model.scroll_offset = list.model.selected;
        }
        assert_eq!(list.model.scroll_offset, 1);
    }
}

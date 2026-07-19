//! Toolbar model, overflow layout, and renderer.
//!
//! The toolbar keeps item focus, toggle state, orientation, and first visible
//! item in a pure model. Rendering updates the first visible item just enough to
//! keep the focused control visible in the supplied area.

use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

const TOOLBAR_ITEM_PADDING: u16 = 1;
const TOOLBAR_ITEM_PADDING_TOTAL: u16 = TOOLBAR_ITEM_PADDING * 2;
const TOOLBAR_ITEM_GAP: u16 = 1;
const TOOLBAR_SEPARATOR_LEN: u16 = 1;
const TOOLBAR_ITEM_CROSS_LEN: u16 = 1;
const HORIZONTAL_SEPARATOR: &str = "│";
const VERTICAL_SEPARATOR: &str = "─";
const HORIZONTAL_LEADING_OVERFLOW: &str = "‹";
const HORIZONTAL_TRAILING_OVERFLOW: &str = "›";
const VERTICAL_LEADING_OVERFLOW: &str = "▲";
const VERTICAL_TRAILING_OVERFLOW: &str = "▼";
const ICON_LABEL_GAP: &str = " ";
const ELLIPSIS: &str = "…";
const ELLIPSIS_WIDTH: usize = 1;

/// Toolbar primary-axis orientation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ToolbarOrientation {
    /// Render items from left to right.
    #[default]
    Horizontal,
    /// Render items from top to bottom.
    Vertical,
}

/// A control or visual separator in a toolbar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolbarItem {
    /// Stateless button item.
    Button {
        /// Optional icon text.
        icon: Option<String>,
        /// Optional label text.
        label: Option<String>,
        /// Disabled buttons are rendered but skipped by focus navigation.
        enabled: bool,
    },
    /// Toggle button item backed by `ToolbarModel::toggle_states`.
    Toggle {
        /// Optional icon text.
        icon: Option<String>,
        /// Optional label text.
        label: Option<String>,
        /// Disabled toggles are rendered but skipped by focus navigation.
        enabled: bool,
    },
    /// Non-focusable visual separator.
    Separator,
}

impl ToolbarItem {
    /// Create an enabled text button.
    pub fn button(label: impl Into<String>) -> Self {
        Self::Button {
            icon: None,
            label: Some(label.into()),
            enabled: true,
        }
    }

    /// Create an enabled icon button.
    pub fn icon_button(icon: impl Into<String>, label: impl Into<String>) -> Self {
        Self::Button {
            icon: Some(icon.into()),
            label: Some(label.into()),
            enabled: true,
        }
    }

    /// Create a disabled text button.
    pub fn disabled_button(label: impl Into<String>) -> Self {
        Self::Button {
            icon: None,
            label: Some(label.into()),
            enabled: false,
        }
    }

    /// Create an enabled text toggle.
    pub fn toggle(label: impl Into<String>) -> Self {
        Self::Toggle {
            icon: None,
            label: Some(label.into()),
            enabled: true,
        }
    }

    /// Create an enabled icon toggle.
    pub fn icon_toggle(icon: impl Into<String>, label: impl Into<String>) -> Self {
        Self::Toggle {
            icon: Some(icon.into()),
            label: Some(label.into()),
            enabled: true,
        }
    }

    /// Create a disabled text toggle.
    pub fn disabled_toggle(label: impl Into<String>) -> Self {
        Self::Toggle {
            icon: None,
            label: Some(label.into()),
            enabled: false,
        }
    }

    /// Create a visual separator.
    pub fn separator() -> Self {
        Self::Separator
    }

    /// True when this item can receive focus.
    pub fn is_selectable(&self) -> bool {
        match self {
            Self::Button { enabled, .. } | Self::Toggle { enabled, .. } => *enabled,
            Self::Separator => false,
        }
    }

    /// True when this item is enabled.
    pub fn is_enabled(&self) -> bool {
        match self {
            Self::Button { enabled, .. } | Self::Toggle { enabled, .. } => *enabled,
            Self::Separator => false,
        }
    }

    /// True when this item is a toggle control.
    pub fn is_toggle(&self) -> bool {
        matches!(self, Self::Toggle { .. })
    }

    /// Return the rendered content without toolbar padding.
    pub fn content_text(&self) -> String {
        match self {
            Self::Button { icon, label, .. } | Self::Toggle { icon, label, .. } => {
                combine_icon_label(icon.as_deref(), label.as_deref())
            }
            Self::Separator => String::new(),
        }
    }
}

/// Activation result for a toolbar item.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ToolbarActivation {
    /// Activated item index.
    pub index: usize,
    /// New toggle state when the item was a toggle.
    pub toggled: Option<bool>,
}

/// Pure toolbar state.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ToolbarModel {
    /// Toolbar items.
    pub items: Vec<ToolbarItem>,
    /// Focused item index.
    pub focused: Option<usize>,
    /// Toggle state parallel to `items`; non-toggle entries remain false.
    pub toggle_states: Vec<bool>,
    /// Primary-axis orientation.
    pub orientation: ToolbarOrientation,
    /// First visible item index used for overflow scrolling.
    pub scroll_offset: usize,
}

impl ToolbarModel {
    /// Create a horizontal toolbar and focus the first selectable item.
    pub fn new(items: Vec<ToolbarItem>) -> Self {
        let toggle_states = vec![false; items.len()];
        let focused = first_selectable_index(&items);
        Self {
            items,
            focused,
            toggle_states,
            orientation: ToolbarOrientation::Horizontal,
            scroll_offset: 0,
        }
    }

    /// Replace toolbar items and reset focus/toggle/scroll state.
    pub fn with_items(mut self, items: Vec<ToolbarItem>) -> Self {
        self.items = items;
        self.toggle_states = vec![false; self.items.len()];
        self.focused = first_selectable_index(&self.items);
        self.scroll_offset = 0;
        self
    }

    /// Set toolbar orientation.
    pub fn with_orientation(mut self, orientation: ToolbarOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Focus an item if it is selectable.
    pub fn set_focus(&mut self, index: usize) -> bool {
        if self
            .items
            .get(index)
            .is_some_and(ToolbarItem::is_selectable)
        {
            self.focused = Some(index);
            true
        } else {
            false
        }
    }

    /// Clear toolbar focus.
    pub fn clear_focus(&mut self) {
        self.focused = None;
    }

    /// Move focus to the next selectable item, clamping at the end.
    pub fn move_next(&mut self) -> Option<usize> {
        self.focused = next_selectable_index(&self.items, self.focused);
        self.focused
    }

    /// Move focus to the previous selectable item, clamping at the start.
    pub fn move_prev(&mut self) -> Option<usize> {
        self.focused = previous_selectable_index(&self.items, self.focused);
        self.focused
    }

    /// Activate the focused item.
    pub fn activate(&mut self) -> Option<ToolbarActivation> {
        let index = self.focused?;
        let item = self.items.get(index)?;
        if !item.is_selectable() {
            return None;
        }

        if item.is_toggle() {
            let state = self.toggle_states.get_mut(index)?;
            *state = !*state;
            Some(ToolbarActivation {
                index,
                toggled: Some(*state),
            })
        } else {
            Some(ToolbarActivation {
                index,
                toggled: None,
            })
        }
    }

    /// Return a toggle state for a toggle item.
    pub fn is_toggled(&self, index: usize) -> Option<bool> {
        if self.items.get(index).is_some_and(ToolbarItem::is_toggle) {
            self.toggle_states.get(index).copied()
        } else {
            None
        }
    }

    /// Set a toggle item state. Returns false for non-toggle or missing items.
    pub fn set_toggled(&mut self, index: usize, toggled: bool) -> bool {
        if !self.items.get(index).is_some_and(ToolbarItem::is_toggle) {
            return false;
        }
        let Some(state) = self.toggle_states.get_mut(index) else {
            return false;
        };
        *state = toggled;
        true
    }
}

/// Visual style for a toolbar.
#[derive(Debug, Clone)]
pub struct ToolbarStyle {
    /// Style used to fill the full toolbar area.
    pub background: Style,
    /// Style for enabled buttons.
    pub item: Style,
    /// Style for the focused item.
    pub focused_item: Style,
    /// Style for disabled items.
    pub disabled_item: Style,
    /// Style for active toggles.
    pub toggled_item: Style,
    /// Style for separators.
    pub separator: Style,
    /// Style for overflow indicators.
    pub overflow: Style,
}

impl Default for ToolbarStyle {
    fn default() -> Self {
        Self {
            background: Style::default().bg(Color::Black),
            item: Style::default().fg(Color::White),
            focused_item: Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
            disabled_item: Style::default().fg(Color::DarkGray),
            toggled_item: Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            separator: Style::default().fg(Color::DarkGray),
            overflow: Style::default().fg(Color::Gray),
        }
    }
}

impl ToolbarStyle {
    /// Set the toolbar background style.
    pub fn with_background(mut self, background: Style) -> Self {
        self.background = background;
        self
    }

    /// Set the enabled item style.
    pub fn with_item(mut self, item: Style) -> Self {
        self.item = item;
        self
    }

    /// Set the focused item style.
    pub fn with_focused_item(mut self, focused_item: Style) -> Self {
        self.focused_item = focused_item;
        self
    }

    /// Set the disabled item style.
    pub fn with_disabled_item(mut self, disabled_item: Style) -> Self {
        self.disabled_item = disabled_item;
        self
    }

    /// Set the toggled item style.
    pub fn with_toggled_item(mut self, toggled_item: Style) -> Self {
        self.toggled_item = toggled_item;
        self
    }

    /// Set the separator style.
    pub fn with_separator(mut self, separator: Style) -> Self {
        self.separator = separator;
        self
    }

    /// Set the overflow indicator style.
    pub fn with_overflow(mut self, overflow: Style) -> Self {
        self.overflow = overflow;
        self
    }
}

/// Layout for one rendered toolbar item.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ToolbarItemLayout {
    /// Index into `ToolbarModel::items`.
    pub index: usize,
    /// Rendered item rectangle.
    pub rect: Rect,
}

/// Computed toolbar layout.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ToolbarLayout {
    /// Rendered item rectangles.
    pub items: Vec<ToolbarItemLayout>,
    /// True when items exist before `scroll_offset`.
    pub leading_overflow: bool,
    /// True when items remain after the rendered item window.
    pub trailing_overflow: bool,
}

/// Render a toolbar and return its visible item layout.
pub fn render_toolbar(
    frame: &mut Frame,
    area: Rect,
    model: &mut ToolbarModel,
    style: &ToolbarStyle,
) -> ToolbarLayout {
    let primary_len = primary_len(area, model.orientation);
    ensure_focus_visible(model, primary_len);
    let layout = compute_toolbar_layout(area, model);

    if area.width == 0 || area.height == 0 {
        return layout;
    }

    let buffer = frame.buffer_mut();
    fill_area(buffer, area, style.background);

    for item_layout in &layout.items {
        let item = &model.items[item_layout.index];
        render_toolbar_item(
            buffer,
            item_layout.rect,
            model,
            item_layout.index,
            item,
            style,
        );
    }

    render_overflow_indicators(buffer, area, model.orientation, &layout, style);

    layout
}

fn compute_toolbar_layout(area: Rect, model: &ToolbarModel) -> ToolbarLayout {
    if area.width == 0 || area.height == 0 || model.items.is_empty() {
        return ToolbarLayout::default();
    }

    let mut layouts = Vec::new();
    let mut cursor = primary_start(area, model.orientation);
    let primary_end =
        primary_start(area, model.orientation).saturating_add(primary_len(area, model.orientation));
    let mut next_index = model.scroll_offset.min(model.items.len());

    while next_index < model.items.len() {
        let item = &model.items[next_index];
        let item_len = item_primary_len(item, model.orientation);
        let with_gap = if next_index == model.scroll_offset {
            item_len
        } else {
            item_len.saturating_add(TOOLBAR_ITEM_GAP)
        };

        if cursor.saturating_add(with_gap) > primary_end && !layouts.is_empty() {
            break;
        }

        if next_index != model.scroll_offset {
            cursor = cursor.saturating_add(TOOLBAR_ITEM_GAP);
        }

        let visible_len = item_len.min(primary_end.saturating_sub(cursor));
        if visible_len == 0 {
            break;
        }

        layouts.push(ToolbarItemLayout {
            index: next_index,
            rect: item_rect(area, model.orientation, cursor, visible_len),
        });
        cursor = cursor.saturating_add(item_len);
        next_index += 1;

        if cursor >= primary_end {
            break;
        }
    }

    ToolbarLayout {
        items: layouts,
        leading_overflow: model.scroll_offset > 0,
        trailing_overflow: next_index < model.items.len(),
    }
}

fn ensure_focus_visible(model: &mut ToolbarModel, available_primary_len: u16) {
    if model.items.is_empty() {
        model.scroll_offset = 0;
        model.focused = None;
        return;
    }

    model.scroll_offset = model.scroll_offset.min(model.items.len().saturating_sub(1));
    let Some(focused) = model.focused else {
        return;
    };

    if focused < model.scroll_offset {
        model.scroll_offset = focused;
        return;
    }

    while !focused_fits_from_offset(model, focused, available_primary_len)
        && model.scroll_offset < focused
    {
        model.scroll_offset += 1;
    }
}

fn focused_fits_from_offset(
    model: &ToolbarModel,
    focused: usize,
    available_primary_len: u16,
) -> bool {
    if available_primary_len == 0 {
        return false;
    }

    let mut used = 0u16;
    for index in model.scroll_offset..=focused {
        if index > model.scroll_offset {
            used = used.saturating_add(TOOLBAR_ITEM_GAP);
        }
        used = used.saturating_add(item_primary_len(&model.items[index], model.orientation));
        if used > available_primary_len {
            return index == model.scroll_offset;
        }
    }
    true
}

fn render_toolbar_item(
    buffer: &mut Buffer,
    rect: Rect,
    model: &ToolbarModel,
    index: usize,
    item: &ToolbarItem,
    style: &ToolbarStyle,
) {
    match item {
        ToolbarItem::Separator => render_separator(buffer, rect, model.orientation, style),
        ToolbarItem::Button { .. } | ToolbarItem::Toggle { .. } => {
            let item_style = toolbar_item_style(model, index, item, style).patch(style.background);
            fill_area(buffer, rect, item_style);
            let text = padded_toolbar_text(item.content_text());
            render_text(buffer, rect, model.orientation, &text, item_style);
        }
    }
}

fn render_separator(
    buffer: &mut Buffer,
    rect: Rect,
    orientation: ToolbarOrientation,
    style: &ToolbarStyle,
) {
    let separator_style = style.separator.patch(style.background);
    match orientation {
        ToolbarOrientation::Horizontal => {
            if rect.width > 0 && rect.height > 0 {
                buffer.set_string(rect.x, rect.y, HORIZONTAL_SEPARATOR, separator_style);
            }
        }
        ToolbarOrientation::Vertical => {
            if rect.width > 0 && rect.height > 0 {
                let line = VERTICAL_SEPARATOR.repeat(usize::from(rect.width));
                buffer.set_line(
                    rect.x,
                    rect.y,
                    &Line::from(Span::styled(line, separator_style)),
                    rect.width,
                );
            }
        }
    }
}

fn render_overflow_indicators(
    buffer: &mut Buffer,
    area: Rect,
    orientation: ToolbarOrientation,
    layout: &ToolbarLayout,
    style: &ToolbarStyle,
) {
    let overflow_style = style.overflow.patch(style.background);
    match orientation {
        ToolbarOrientation::Horizontal => {
            if layout.leading_overflow && area.width > 0 && area.height > 0 {
                buffer.set_string(area.x, area.y, HORIZONTAL_LEADING_OVERFLOW, overflow_style);
            }
            if layout.trailing_overflow && area.width > 0 && area.height > 0 {
                buffer.set_string(
                    rect_right(area).saturating_sub(1),
                    area.y,
                    HORIZONTAL_TRAILING_OVERFLOW,
                    overflow_style,
                );
            }
        }
        ToolbarOrientation::Vertical => {
            if layout.leading_overflow && area.width > 0 && area.height > 0 {
                buffer.set_string(area.x, area.y, VERTICAL_LEADING_OVERFLOW, overflow_style);
            }
            if layout.trailing_overflow && area.width > 0 && area.height > 0 {
                buffer.set_string(
                    area.x,
                    rect_bottom(area).saturating_sub(1),
                    VERTICAL_TRAILING_OVERFLOW,
                    overflow_style,
                );
            }
        }
    }
}

fn toolbar_item_style(
    model: &ToolbarModel,
    index: usize,
    item: &ToolbarItem,
    style: &ToolbarStyle,
) -> Style {
    if !item.is_enabled() {
        style.disabled_item
    } else if model.focused == Some(index) {
        style.focused_item
    } else if item.is_toggle() && model.toggle_states.get(index).copied().unwrap_or_default() {
        style.toggled_item
    } else {
        style.item
    }
}

fn render_text(
    buffer: &mut Buffer,
    rect: Rect,
    orientation: ToolbarOrientation,
    text: &str,
    style: Style,
) {
    match orientation {
        ToolbarOrientation::Horizontal => {
            if rect.width == 0 || rect.height == 0 {
                return;
            }
            let clipped = truncate_to_width(text, rect.width);
            buffer.set_string(rect.x, rect.y, clipped, style);
        }
        ToolbarOrientation::Vertical => {
            if rect.width == 0 || rect.height == 0 {
                return;
            }
            let clipped = truncate_to_width(text, rect.width);
            buffer.set_string(rect.x, rect.y, clipped, style);
        }
    }
}

fn item_rect(area: Rect, orientation: ToolbarOrientation, cursor: u16, primary_len: u16) -> Rect {
    match orientation {
        ToolbarOrientation::Horizontal => Rect::new(
            cursor,
            area.y,
            primary_len,
            TOOLBAR_ITEM_CROSS_LEN.min(area.height),
        ),
        ToolbarOrientation::Vertical => Rect::new(area.x, cursor, area.width, primary_len),
    }
}

fn primary_start(area: Rect, orientation: ToolbarOrientation) -> u16 {
    match orientation {
        ToolbarOrientation::Horizontal => area.x,
        ToolbarOrientation::Vertical => area.y,
    }
}

fn primary_len(area: Rect, orientation: ToolbarOrientation) -> u16 {
    match orientation {
        ToolbarOrientation::Horizontal => area.width,
        ToolbarOrientation::Vertical => area.height,
    }
}

fn item_primary_len(item: &ToolbarItem, orientation: ToolbarOrientation) -> u16 {
    match orientation {
        ToolbarOrientation::Horizontal => match item {
            ToolbarItem::Separator => TOOLBAR_SEPARATOR_LEN,
            ToolbarItem::Button { .. } | ToolbarItem::Toggle { .. } => {
                text_width_u16(&item.content_text()).saturating_add(TOOLBAR_ITEM_PADDING_TOTAL)
            }
        },
        ToolbarOrientation::Vertical => TOOLBAR_ITEM_CROSS_LEN,
    }
}

fn first_selectable_index(items: &[ToolbarItem]) -> Option<usize> {
    items.iter().position(ToolbarItem::is_selectable)
}

fn next_selectable_index(items: &[ToolbarItem], current: Option<usize>) -> Option<usize> {
    if items.is_empty() {
        return None;
    }

    let start = current
        .unwrap_or_default()
        .min(items.len().saturating_sub(1));
    for index in start.saturating_add(1)..items.len() {
        if items[index].is_selectable() {
            return Some(index);
        }
    }

    if current.is_none() {
        first_selectable_index(items)
    } else if items.get(start).is_some_and(ToolbarItem::is_selectable) {
        Some(start)
    } else {
        first_selectable_index(items)
    }
}

fn previous_selectable_index(items: &[ToolbarItem], current: Option<usize>) -> Option<usize> {
    if items.is_empty() {
        return None;
    }

    let start = current
        .unwrap_or_else(|| items.len().saturating_sub(1))
        .min(items.len().saturating_sub(1));
    for index in (0..start).rev() {
        if items[index].is_selectable() {
            return Some(index);
        }
    }

    if current.is_none() {
        items.iter().rposition(ToolbarItem::is_selectable)
    } else if items.get(start).is_some_and(ToolbarItem::is_selectable) {
        Some(start)
    } else {
        items.iter().rposition(ToolbarItem::is_selectable)
    }
}

fn combine_icon_label(icon: Option<&str>, label: Option<&str>) -> String {
    match (icon, label) {
        (Some(icon), Some(label)) if !icon.is_empty() && !label.is_empty() => {
            format!("{icon}{ICON_LABEL_GAP}{label}")
        }
        (Some(icon), _) => icon.to_string(),
        (_, Some(label)) => label.to_string(),
        (None, None) => String::new(),
    }
}

fn padded_toolbar_text(text: String) -> String {
    format!(" {text} ")
}

fn truncate_to_width(text: &str, max_width: u16) -> String {
    let max_width = usize::from(max_width);
    if UnicodeWidthStr::width(text) <= max_width {
        return text.to_string();
    }

    if max_width == 0 {
        return String::new();
    }

    if max_width <= ELLIPSIS_WIDTH {
        return ELLIPSIS.to_string();
    }

    let content_width = max_width - ELLIPSIS_WIDTH;
    let mut used_width = 0usize;
    let mut truncated = String::new();

    for character in text.chars() {
        let character_width = UnicodeWidthChar::width(character).unwrap_or_default();
        if used_width + character_width > content_width {
            break;
        }
        used_width += character_width;
        truncated.push(character);
    }

    truncated.push_str(ELLIPSIS);
    truncated
}

fn text_width_u16(text: &str) -> u16 {
    let width = UnicodeWidthStr::width(text);
    width.min(usize::from(u16::MAX)) as u16
}

fn fill_area(buffer: &mut Buffer, area: Rect, style: Style) {
    for y in area.y..rect_bottom(area) {
        for x in area.x..rect_right(area) {
            buffer[(x, y)].set_char(' ').set_style(style);
        }
    }
}

fn rect_right(rect: Rect) -> u16 {
    rect.x.saturating_add(rect.width)
}

fn rect_bottom(rect: Rect) -> u16 {
    rect.y.saturating_add(rect.height)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    const SAVE_INDEX: usize = 2;
    const BOLD_INDEX: usize = 4;
    const LAST_SELECTABLE_INDEX: usize = 4;
    const FIRST_SELECTABLE_INDEX: usize = 2;
    const OUT_OF_RANGE_INDEX: usize = 99;
    const HORIZONTAL_WIDTH: u16 = 10;
    const HORIZONTAL_HEIGHT: u16 = 1;
    const VERTICAL_WIDTH: u16 = 12;
    const VERTICAL_HEIGHT: u16 = 4;
    const OVERFLOW_INDICATOR_X: u16 = HORIZONTAL_WIDTH - 1;
    const FIRST_ROW: u16 = 0;
    const SECOND_ROW: u16 = 1;
    const THIRD_ROW: u16 = 2;
    const FOURTH_ROW: u16 = 3;
    const BUTTON_TEXT_X: u16 = 1;
    const RENDER_WIDTH: u16 = 20;
    const RENDER_HEIGHT: u16 = 1;

    #[test]
    fn focus_navigation_skips_separators_and_disabled_items() {
        let mut model = ToolbarModel::new(sample_items());

        assert_eq!(model.focused, Some(SAVE_INDEX));
        assert_eq!(model.move_next(), Some(BOLD_INDEX));
        assert_eq!(model.move_next(), Some(BOLD_INDEX));
        assert_eq!(model.move_prev(), Some(SAVE_INDEX));
        assert!(!model.set_focus(OUT_OF_RANGE_INDEX));
    }

    #[test]
    fn activate_button_returns_index_without_toggle_change() {
        let mut model = ToolbarModel::new(sample_items());

        let activation = model.activate();

        assert_eq!(
            activation,
            Some(ToolbarActivation {
                index: SAVE_INDEX,
                toggled: None,
            })
        );
    }

    #[test]
    fn activate_toggle_flips_state() {
        let mut model = ToolbarModel::new(sample_items());
        assert!(model.set_focus(BOLD_INDEX));

        let first = model.activate();
        let second = model.activate();

        assert_eq!(
            first,
            Some(ToolbarActivation {
                index: BOLD_INDEX,
                toggled: Some(true),
            })
        );
        assert_eq!(
            second,
            Some(ToolbarActivation {
                index: BOLD_INDEX,
                toggled: Some(false),
            })
        );
        assert_eq!(model.is_toggled(BOLD_INDEX), Some(false));
    }

    #[test]
    fn empty_or_all_disabled_toolbar_has_no_focus_or_activation() {
        let mut empty = ToolbarModel::new(Vec::new());
        let mut disabled = ToolbarModel::new(vec![
            ToolbarItem::separator(),
            ToolbarItem::disabled_button("Disabled"),
        ]);

        assert_eq!(empty.focused, None);
        assert_eq!(empty.move_next(), None);
        assert_eq!(empty.activate(), None);
        assert_eq!(disabled.focused, None);
        assert_eq!(disabled.move_next(), None);
        assert_eq!(disabled.activate(), None);
    }

    #[test]
    fn horizontal_render_shows_trailing_overflow_indicator() {
        let mut model = ToolbarModel::new(vec![
            ToolbarItem::button("One"),
            ToolbarItem::button("Two"),
            ToolbarItem::button("Three"),
        ]);

        let (buffer, layout) = render_horizontal(&mut model, HORIZONTAL_WIDTH);

        assert!(layout.trailing_overflow);
        assert_eq!(
            buffer[(OVERFLOW_INDICATOR_X, FIRST_ROW)].symbol(),
            HORIZONTAL_TRAILING_OVERFLOW
        );
    }

    #[test]
    fn render_scrolls_window_to_keep_focus_visible() {
        let mut model = ToolbarModel::new(vec![
            ToolbarItem::button("One"),
            ToolbarItem::button("Two"),
            ToolbarItem::button("Three"),
            ToolbarItem::button("Four"),
        ]);
        assert!(model.set_focus(LAST_SELECTABLE_INDEX - 1));

        let (_buffer, layout) = render_horizontal(&mut model, HORIZONTAL_WIDTH);

        assert!(model.scroll_offset > 0);
        assert!(
            layout
                .items
                .iter()
                .any(|item| item.index == LAST_SELECTABLE_INDEX - 1)
        );
    }

    #[test]
    fn vertical_toolbar_renders_separator_and_overflow() {
        let mut model = ToolbarModel::new(vec![
            ToolbarItem::button("One"),
            ToolbarItem::separator(),
            ToolbarItem::button("Two"),
            ToolbarItem::button("Three"),
        ])
        .with_orientation(ToolbarOrientation::Vertical);

        let (buffer, layout) = render_vertical(&mut model);

        assert!(layout.trailing_overflow);
        assert_eq!(buffer[(BUTTON_TEXT_X, FIRST_ROW)].symbol(), "O");
        assert_eq!(buffer[(0, THIRD_ROW)].symbol(), VERTICAL_SEPARATOR);
        assert_eq!(buffer[(0, FOURTH_ROW)].symbol(), VERTICAL_TRAILING_OVERFLOW);
    }

    #[test]
    fn set_toggled_rejects_non_toggle_and_out_of_range_items() {
        let mut model = ToolbarModel::new(sample_items());

        assert!(!model.set_toggled(SAVE_INDEX, true));
        assert!(!model.set_toggled(OUT_OF_RANGE_INDEX, true));
        assert!(model.set_toggled(BOLD_INDEX, true));
        assert_eq!(model.is_toggled(BOLD_INDEX), Some(true));
        assert_eq!(model.is_toggled(SAVE_INDEX), None);
    }

    fn sample_items() -> Vec<ToolbarItem> {
        vec![
            ToolbarItem::separator(),
            ToolbarItem::disabled_button("Disabled"),
            ToolbarItem::button("Save"),
            ToolbarItem::separator(),
            ToolbarItem::toggle("Bold"),
        ]
    }

    fn render_horizontal(
        model: &mut ToolbarModel,
        width: u16,
    ) -> (ratatui::buffer::Buffer, ToolbarLayout) {
        let backend = TestBackend::new(width, HORIZONTAL_HEIGHT);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut layout = ToolbarLayout::default();
        terminal
            .draw(|frame| {
                let area = frame.area();
                layout = render_toolbar(frame, area, model, &ToolbarStyle::default());
            })
            .unwrap();
        (terminal.backend().buffer().clone(), layout)
    }

    fn render_vertical(model: &mut ToolbarModel) -> (ratatui::buffer::Buffer, ToolbarLayout) {
        let backend = TestBackend::new(VERTICAL_WIDTH, VERTICAL_HEIGHT);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut layout = ToolbarLayout::default();
        terminal
            .draw(|frame| {
                let area = frame.area();
                layout = render_toolbar(frame, area, model, &ToolbarStyle::default());
            })
            .unwrap();
        (terminal.backend().buffer().clone(), layout)
    }

    fn render_toolbar_to_buffer(model: &mut ToolbarModel) -> ratatui::buffer::Buffer {
        let backend = TestBackend::new(RENDER_WIDTH, RENDER_HEIGHT);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                let area = frame.area();
                render_toolbar(frame, area, model, &ToolbarStyle::default());
            })
            .unwrap();
        terminal.backend().buffer().clone()
    }

    #[test]
    fn focused_item_uses_focused_style_when_rendered() {
        let mut model = ToolbarModel::new(vec![ToolbarItem::button("Save")]);
        let buffer = render_toolbar_to_buffer(&mut model);

        assert_eq!(buffer[(BUTTON_TEXT_X, FIRST_ROW)].symbol(), "S");
        assert_eq!(buffer[(BUTTON_TEXT_X, FIRST_ROW)].fg, Color::Yellow);
    }
}

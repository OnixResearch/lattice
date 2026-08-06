//! Menu bar model, dropdown layout, and render helpers.
//!
//! The model owns pure navigation state while render functions only translate
//! computed layouts into ratatui buffers. Consumers remain responsible for
//! mapping keys or mouse events to these model methods.

use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Clear;
use unicode_width::UnicodeWidthChar;
use unicode_width::UnicodeWidthStr;

#[path = "menu_bar/layout.rs"]
mod layout;
#[path = "menu_bar/reducer.rs"]
mod reducer;

use crate::MenuItem;
use crate::MenuNav;

const MENU_LABEL_HORIZONTAL_PADDING: u16 = 1;
const MENU_BAR_HEIGHT: u16 = 1;
const DROPDOWN_BORDER_WIDTH: u16 = 2;
const DROPDOWN_ITEM_HORIZONTAL_PADDING: u16 = 1;
const DROPDOWN_ITEM_VERTICAL_PADDING: u16 = 2;
const DROPDOWN_ACCEL_GAP: usize = 2;
const DROPDOWN_SUBMENU_MARKER_GAP: usize = 1;
const DROPDOWN_SUBMENU_MARKER: &str = "›";
const DROPDOWN_SEPARATOR: &str = "─";
const ELLIPSIS: &str = "…";
const ELLIPSIS_WIDTH: usize = 1;

/// Pure data model for a top menu bar and its active dropdown.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MenuBarModel {
    /// Top-level menu labels.
    pub labels: Vec<String>,
    /// Active top-level menu index.
    pub active: usize,
    /// Whether the active menu's dropdown is open.
    pub open: bool,
    /// Dropdown item lists parallel to `labels`.
    pub dropdowns: Vec<Vec<MenuItem>>,
    /// Navigation state for the active dropdown.
    pub nav: MenuNav,
}

impl MenuBarModel {
    /// Create a closed menu bar with empty dropdowns for each label.
    pub fn new<I, S>(labels: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let labels: Vec<String> = labels.into_iter().map(Into::into).collect();
        let dropdowns = vec![Vec::new(); labels.len()];
        Self {
            labels,
            active: 0,
            open: false,
            dropdowns,
            nav: MenuNav::new(),
        }
    }

    /// Create a menu bar from `(label, dropdown_items)` pairs.
    pub fn from_menus<I, S>(menus: I) -> Self
    where
        I: IntoIterator<Item = (S, Vec<MenuItem>)>,
        S: Into<String>,
    {
        let mut labels = Vec::new();
        let mut dropdowns = Vec::new();
        for (label, items) in menus {
            labels.push(label.into());
            dropdowns.push(items);
        }

        Self {
            labels,
            active: 0,
            open: false,
            dropdowns,
            nav: MenuNav::new(),
        }
    }

    /// Replace one dropdown item list. Out-of-range indices are ignored.
    pub fn with_dropdown(mut self, menu_index: usize, items: Vec<MenuItem>) -> Self {
        if let Some(dropdown) = self.dropdowns.get_mut(menu_index) {
            *dropdown = items;
        }
        self
    }

    /// Set the active menu, clamping to the last label.
    pub fn with_active(mut self, active: usize) -> Self {
        self.set_active(active);
        self
    }

    /// Return the number of top-level menus.
    pub fn len(&self) -> usize {
        self.labels.len()
    }

    /// True when there are no top-level menus.
    pub fn is_empty(&self) -> bool {
        self.labels.is_empty()
    }

    /// Return the active top-level label.
    pub fn active_label(&self) -> Option<&str> {
        self.labels.get(self.active).map(String::as_str)
    }

    /// Return the active dropdown item list.
    pub fn active_items(&self) -> &[MenuItem] {
        self.dropdowns.get(self.active).map(Vec::as_slice).unwrap_or_default()
    }

    /// Return the current submenu level for the active dropdown.
    pub fn current_items(&self) -> Option<&[MenuItem]> {
        self.nav.current_items(self.active_items())
    }

    /// Set the active menu, clamping to the last label. Open menus reset focus.
    pub fn set_active(&mut self, active: usize) -> Option<usize> {
        self.active = reducer::clamp_active_index(reducer::ActiveIndexRequest {
            active,
            menu_count: self.labels.len(),
        })?;
        if self.open {
            self.focus_active_dropdown();
        }
        Some(self.active)
    }

    /// Open the active dropdown and focus its first selectable item.
    pub fn open(&mut self) -> bool {
        if self.is_empty() {
            return false;
        }
        self.open = true;
        self.focus_active_dropdown();
        true
    }

    /// Close the active dropdown and clear dropdown focus/submenus.
    pub fn close(&mut self) {
        self.open = false;
        self.nav.clear();
    }

    /// Move to the previous top-level menu, wrapping at the start.
    pub fn move_left(&mut self) -> Option<usize> {
        self.active = reducer::previous_active_index(reducer::ActiveIndexRequest {
            active: self.active,
            menu_count: self.labels.len(),
        })?;
        if self.open {
            self.focus_active_dropdown();
        }
        Some(self.active)
    }

    /// Move to the next top-level menu, wrapping at the end.
    pub fn move_right(&mut self) -> Option<usize> {
        self.active = reducer::next_active_index(reducer::ActiveIndexRequest {
            active: self.active,
            menu_count: self.labels.len(),
        })?;
        if self.open {
            self.focus_active_dropdown();
        }
        Some(self.active)
    }

    /// Move focus down in the current dropdown level, opening if needed.
    pub fn move_down(&mut self) -> Option<usize> {
        if !self.open {
            self.open();
            return self.nav.focused();
        }

        let active = self.active;
        let items = self.dropdowns.get(active).map(Vec::as_slice).unwrap_or_default();
        let current_items = self.nav.current_items(items).unwrap_or_default();
        self.nav.move_down(current_items)
    }

    /// Move focus up in the current dropdown level, opening if needed.
    pub fn move_up(&mut self) -> Option<usize> {
        if !self.open {
            self.open();
            return self.nav.focused();
        }

        let active = self.active;
        let items = self.dropdowns.get(active).map(Vec::as_slice).unwrap_or_default();
        let current_items = self.nav.current_items(items).unwrap_or_default();
        self.nav.move_up(current_items)
    }

    /// Enter the focused submenu in the active dropdown.
    pub fn enter_submenu(&mut self) -> bool {
        if !self.open {
            return false;
        }
        let active = self.active;
        let items = self.dropdowns.get(active).map(Vec::as_slice).unwrap_or_default();
        self.nav.enter_submenu(items)
    }

    /// Leave the current submenu in the active dropdown.
    pub fn leave_submenu(&mut self) -> bool {
        self.nav.leave_submenu()
    }

    /// Activate the focused dropdown item, returning its label and closing.
    pub fn activate(&mut self) -> Option<String> {
        if !self.open {
            return None;
        }

        let label = self.nav.current_item(self.active_items()).and_then(MenuItem::label).map(str::to_string);

        if label.is_some() {
            self.close();
        }

        label
    }

    fn focus_active_dropdown(&mut self) {
        self.nav.clear();
        let active = self.active;
        let items = self.dropdowns.get(active).map(Vec::as_slice).unwrap_or_default();
        self.nav.focus_first(items);
    }
}

/// Visual style for the top menu bar.
#[derive(Debug, Clone)]
pub struct MenuBarStyle {
    /// Style used to fill the full menu bar area.
    pub background: Style,
    /// Style for inactive top-level labels.
    pub label: Style,
    /// Style for the active top-level label while closed.
    pub active_label: Style,
    /// Style for the active top-level label while its dropdown is open.
    pub open_label: Style,
}

impl Default for MenuBarStyle {
    fn default() -> Self {
        Self {
            background: Style::default().bg(Color::Black),
            label: Style::default().fg(Color::Gray),
            active_label: Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            open_label: Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        }
    }
}

impl MenuBarStyle {
    /// Set the menu bar background style.
    pub fn with_background(mut self, background: Style) -> Self {
        self.background = background;
        self
    }

    /// Set the inactive label style.
    pub fn with_label(mut self, label: Style) -> Self {
        self.label = label;
        self
    }

    /// Set the active label style used while the dropdown is closed.
    pub fn with_active_label(mut self, active_label: Style) -> Self {
        self.active_label = active_label;
        self
    }

    /// Set the active label style used while the dropdown is open.
    pub fn with_open_label(mut self, open_label: Style) -> Self {
        self.open_label = open_label;
        self
    }
}

/// Visual style for a dropdown menu.
#[derive(Debug, Clone)]
pub struct DropdownStyle {
    /// Style used for dropdown background fill.
    pub background: Style,
    /// Style used for the dropdown border.
    pub border: Style,
    /// Style for enabled menu item labels.
    pub item: Style,
    /// Style for disabled menu item labels.
    pub disabled_item: Style,
    /// Style for the focused menu item row.
    pub selected_item: Style,
    /// Style for accelerator hints.
    pub accel: Style,
    /// Style for submenu markers.
    pub submenu_marker: Style,
    /// Style for separator rows.
    pub separator: Style,
}

impl Default for DropdownStyle {
    fn default() -> Self {
        Self {
            background: Style::default().bg(Color::Black),
            border: Style::default().fg(Color::Gray),
            item: Style::default().fg(Color::White),
            disabled_item: Style::default().fg(Color::DarkGray),
            selected_item: Style::default().fg(Color::Black).bg(Color::Yellow).add_modifier(Modifier::BOLD),
            accel: Style::default().fg(Color::Gray),
            submenu_marker: Style::default().fg(Color::White),
            separator: Style::default().fg(Color::DarkGray),
        }
    }
}

impl DropdownStyle {
    /// Set the dropdown background style.
    pub fn with_background(mut self, background: Style) -> Self {
        self.background = background;
        self
    }

    /// Set the dropdown border style.
    pub fn with_border(mut self, border: Style) -> Self {
        self.border = border;
        self
    }

    /// Set the enabled item style.
    pub fn with_item(mut self, item: Style) -> Self {
        self.item = item;
        self
    }

    /// Set the disabled item style.
    pub fn with_disabled_item(mut self, disabled_item: Style) -> Self {
        self.disabled_item = disabled_item;
        self
    }

    /// Set the selected item style.
    pub fn with_selected_item(mut self, selected_item: Style) -> Self {
        self.selected_item = selected_item;
        self
    }

    /// Set the accelerator hint style.
    pub fn with_accel(mut self, accel: Style) -> Self {
        self.accel = accel;
        self
    }

    /// Set the submenu marker style.
    pub fn with_submenu_marker(mut self, submenu_marker: Style) -> Self {
        self.submenu_marker = submenu_marker;
        self
    }

    /// Set the separator style.
    pub fn with_separator(mut self, separator: Style) -> Self {
        self.separator = separator;
        self
    }
}

/// Layout information for one top-level menu label.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MenuLabelLayout {
    /// Index into `MenuBarModel::labels`.
    pub index: usize,
    /// Label rectangle.
    pub rect: Rect,
}

/// Computed menu bar layout.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MenuBarLayout {
    /// Rendered label rectangles.
    pub labels: Vec<MenuLabelLayout>,
}

/// Layout information for one dropdown row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DropdownRowLayout {
    /// Index into the rendered item slice.
    pub index: usize,
    /// Row rectangle inside the dropdown border.
    pub rect: Rect,
}

/// Computed dropdown layout.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DropdownLayout {
    /// Full dropdown rect including border.
    pub outer: Rect,
    /// Inner item-list rect.
    pub inner: Rect,
    /// Rendered row rectangles.
    pub rows: Vec<DropdownRowLayout>,
}

/// Compute label rectangles for a menu bar without rendering.
pub fn compute_menu_bar_layout(area: Rect, model: &MenuBarModel) -> MenuBarLayout {
    if area.width == 0 || area.height == 0 {
        return MenuBarLayout::default();
    }

    let mut x = area.x;
    let right = rect_right(area);
    let mut labels = Vec::new();

    for (index, label) in model.labels.iter().enumerate() {
        let width = menu_label_width(label);
        if width == 0 || x.saturating_add(width) > right {
            break;
        }
        labels.push(MenuLabelLayout {
            index,
            rect: Rect::new(x, area.y, width, MENU_BAR_HEIGHT),
        });
        x = x.saturating_add(width);
    }

    MenuBarLayout { labels }
}

/// Render the top menu bar and return its computed label layout.
pub fn render_menu_bar(frame: &mut Frame, area: Rect, model: &MenuBarModel, style: &MenuBarStyle) -> MenuBarLayout {
    let layout = compute_menu_bar_layout(area, model);

    if area.width == 0 || area.height == 0 {
        return layout;
    }

    let buffer = frame.buffer_mut();
    fill_area(buffer, area, style.background);

    for label in &layout.labels {
        let item_style = menu_label_style(model, label.index, style).patch(style.background);
        fill_area(buffer, label.rect, item_style);
        let text = padded_menu_label(&model.labels[label.index]);
        buffer.set_string(label.rect.x, label.rect.y, text, item_style);
    }

    layout
}

/// Return the dropdown width including border, clamped to `minimum_width`.
pub fn compute_dropdown_width(items: &[MenuItem], minimum_width: u16) -> u16 {
    let item_width = items.iter().map(dropdown_item_content_width).max().unwrap_or_default();
    let padded_width = item_width.saturating_add(usize::from(DROPDOWN_ITEM_HORIZONTAL_PADDING.saturating_mul(2)));
    let bordered_width = padded_width.saturating_add(usize::from(DROPDOWN_BORDER_WIDTH));
    usize_to_u16(bordered_width).max(minimum_width)
}

/// Compute a dropdown area below a top-level menu label.
pub fn dropdown_area_below(viewport: Rect, anchor: Rect, items: &[MenuItem], minimum_width: u16) -> Rect {
    if viewport.width == 0 || viewport.height == 0 {
        return Rect::new(viewport.x, viewport.y, 0, 0);
    }

    let requested_width = compute_dropdown_width(items, minimum_width).min(viewport.width);
    let requested_height = dropdown_outer_height(items).min(viewport.height);
    layout::dropdown_rect(layout::DropdownRectRequest {
        viewport,
        anchor,
        requested_width,
        requested_height,
    })
}

/// Compute dropdown row layout for an exact outer area.
pub fn compute_dropdown_layout(area: Rect, items: &[MenuItem]) -> DropdownLayout {
    if area.width == 0 || area.height == 0 {
        return DropdownLayout::default();
    }

    let inner = Block::default().borders(Borders::ALL).inner(area);
    let row_count = usize::from(inner.height).min(items.len());
    let rows = (0..row_count)
        .map(|index| DropdownRowLayout {
            index,
            rect: Rect::new(inner.x, inner.y + usize_to_u16(index), inner.width, MENU_BAR_HEIGHT),
        })
        .collect();

    DropdownLayout {
        outer: area,
        inner,
        rows,
    }
}

/// Render a bordered dropdown item list into an exact outer area.
pub fn render_dropdown(
    frame: &mut Frame,
    area: Rect,
    items: &[MenuItem],
    nav: &MenuNav,
    style: &DropdownStyle,
) -> DropdownLayout {
    let layout = compute_dropdown_layout(area, items);

    if area.width == 0 || area.height == 0 {
        return layout;
    }

    frame.render_widget(Clear, area);
    fill_area(frame.buffer_mut(), area, style.background);

    let block = Block::default().borders(Borders::ALL).border_style(style.border);
    frame.render_widget(block, area);

    let selected = nav.focused();
    for row in &layout.rows {
        let item = &items[row.index];
        render_dropdown_row(frame.buffer_mut(), row.rect, item, selected == Some(row.index), style);
    }

    layout
}

fn menu_label_width(label: &str) -> u16 {
    text_width_u16(label).saturating_add(MENU_LABEL_HORIZONTAL_PADDING.saturating_mul(2))
}

fn padded_menu_label(label: &str) -> String {
    format!(" {label} ")
}

fn menu_label_style(model: &MenuBarModel, index: usize, style: &MenuBarStyle) -> Style {
    if model.active == index && model.open {
        style.open_label
    } else if model.active == index {
        style.active_label
    } else {
        style.label
    }
}

fn dropdown_outer_height(items: &[MenuItem]) -> u16 {
    usize_to_u16(items.len()).saturating_add(DROPDOWN_ITEM_VERTICAL_PADDING)
}

fn dropdown_item_content_width(item: &MenuItem) -> usize {
    match item {
        MenuItem::Action { label, accel, .. } => item_text_width(label, accel.as_deref(), false),
        MenuItem::Submenu { label, accel, .. } => item_text_width(label, accel.as_deref(), true),
        MenuItem::Separator => UnicodeWidthStr::width(DROPDOWN_SEPARATOR),
    }
}

fn item_text_width(label: &str, accel: Option<&str>, submenu: bool) -> usize {
    let label_width = UnicodeWidthStr::width(label);
    let accel_width = accel.map(|value| DROPDOWN_ACCEL_GAP + UnicodeWidthStr::width(value)).unwrap_or_default();
    let submenu_width = if submenu {
        DROPDOWN_SUBMENU_MARKER_GAP + UnicodeWidthStr::width(DROPDOWN_SUBMENU_MARKER)
    } else {
        0
    };
    label_width.saturating_add(accel_width).saturating_add(submenu_width)
}

fn render_dropdown_row(buffer: &mut Buffer, rect: Rect, item: &MenuItem, selected: bool, style: &DropdownStyle) {
    if rect.width == 0 || rect.height == 0 {
        return;
    }

    let row_style = dropdown_row_style(item, selected, style).patch(style.background);
    fill_area(buffer, rect, row_style);

    match item {
        MenuItem::Action { label, accel, .. } => {
            render_dropdown_item_text(buffer, rect, label, accel.as_deref(), false, row_style, style);
        }
        MenuItem::Submenu { label, accel, .. } => {
            render_dropdown_item_text(buffer, rect, label, accel.as_deref(), true, row_style, style);
        }
        MenuItem::Separator => render_separator(buffer, rect, style.separator.patch(style.background)),
    }
}

fn dropdown_row_style(item: &MenuItem, selected: bool, style: &DropdownStyle) -> Style {
    if selected && item.is_selectable() {
        style.selected_item
    } else if item.is_enabled() {
        style.item
    } else {
        style.disabled_item
    }
}

fn render_dropdown_item_text(
    buffer: &mut Buffer,
    rect: Rect,
    label: &str,
    accel: Option<&str>,
    submenu: bool,
    row_style: Style,
    style: &DropdownStyle,
) {
    let available_width = rect.width.saturating_sub(DROPDOWN_ITEM_HORIZONTAL_PADDING.saturating_mul(2));
    if available_width == 0 {
        return;
    }

    let content_x = rect.x.saturating_add(DROPDOWN_ITEM_HORIZONTAL_PADDING);
    let content_right = content_x.saturating_add(available_width);
    let marker_width = if submenu {
        text_width_u16(DROPDOWN_SUBMENU_MARKER)
    } else {
        0
    };
    let marker_gap = if submenu {
        usize_to_u16(DROPDOWN_SUBMENU_MARKER_GAP)
    } else {
        0
    };

    if submenu && marker_width <= available_width {
        let marker_x = content_right.saturating_sub(marker_width);
        buffer.set_string(marker_x, rect.y, DROPDOWN_SUBMENU_MARKER, style.submenu_marker.patch(row_style));
    }

    let accel_width = accel.map(text_width_u16).unwrap_or_default();
    let reserved_right = marker_width.saturating_add(marker_gap);
    let accel_x = if accel_width > 0 {
        content_right.saturating_sub(reserved_right).saturating_sub(accel_width)
    } else {
        content_right.saturating_sub(reserved_right)
    };

    if let Some(accel) = accel
        && accel_width > 0
        && accel_x >= content_x
    {
        let available = content_right.saturating_sub(reserved_right).saturating_sub(accel_x);
        let text = truncate_to_width(accel, available);
        buffer.set_string(accel_x, rect.y, text, style.accel.patch(row_style));
    }

    let label_right = if accel_width > 0 && accel_x > content_x {
        accel_x.saturating_sub(usize_to_u16(DROPDOWN_ACCEL_GAP))
    } else {
        content_right.saturating_sub(reserved_right)
    };
    let label_width = label_right.saturating_sub(content_x);
    let label_text = truncate_to_width(label, label_width);
    if !label_text.is_empty() {
        buffer.set_string(content_x, rect.y, label_text, row_style);
    }
}

fn render_separator(buffer: &mut Buffer, rect: Rect, style: Style) {
    if rect.width == 0 {
        return;
    }

    let line = DROPDOWN_SEPARATOR.repeat(usize::from(rect.width));
    buffer.set_line(rect.x, rect.y, &Line::from(Span::styled(line, style)), rect.width);
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

fn usize_to_u16(value: usize) -> u16 {
    value.min(usize::from(u16::MAX)) as u16
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
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    use super::*;

    const DEFAULT_DROPDOWN_MIN_WIDTH: u16 = 8;
    const MENU_BAR_WIDTH: u16 = 32;
    const MENU_BAR_RENDER_HEIGHT: u16 = 1;
    const MENU_FILE_INDEX: usize = 0;
    const MENU_EDIT_INDEX: usize = 1;
    const MENU_VIEW_INDEX: usize = 2;
    const OPEN_ITEM_INDEX: usize = 0;
    const SAVE_ITEM_INDEX: usize = 3;
    const RECENT_ITEM_INDEX: usize = 4;
    const RENDER_SUBMENU_INDEX: usize = 3;
    const SUBMENU_CHILD_INDEX: usize = 0;
    const DROPDOWN_WIDTH: u16 = 20;
    const DROPDOWN_HEIGHT: u16 = 6;
    const DROPDOWN_ROW_X: u16 = 1;
    const DROPDOWN_CONTENT_X: u16 = 2;
    const DROPDOWN_FIRST_ROW_Y: u16 = 1;
    const DROPDOWN_SECOND_ROW_Y: u16 = 2;
    const DROPDOWN_THIRD_ROW_Y: u16 = 3;
    const EXPECTED_FILE_LABEL_WIDTH: u16 = 6;
    const EXPECTED_WIDE_DROPDOWN_WIDTH: u16 = 16;
    const TOO_SMALL_WIDTH: u16 = 4;
    const ANCHOR_X: u16 = 28;
    const ANCHOR_Y: u16 = 0;
    const ANCHOR_WIDTH: u16 = 4;
    const ANCHOR_HEIGHT: u16 = 1;
    const VIEWPORT_WIDTH: u16 = 30;
    const VIEWPORT_HEIGHT: u16 = 8;
    const DROPDOWN_FOURTH_ROW_Y: u16 = 4;
    const DROPDOWN_MARKER_X: u16 = 17;

    #[test]
    fn menu_bar_switches_active_menu_while_open() {
        let mut model = sample_model();

        assert!(model.open());
        assert_eq!(model.active, MENU_FILE_INDEX);
        assert_eq!(model.nav.focused(), Some(OPEN_ITEM_INDEX));

        assert_eq!(model.move_right(), Some(MENU_EDIT_INDEX));
        assert!(model.open);
        assert_eq!(model.active_label(), Some("Edit"));
        assert_eq!(model.nav.focused(), Some(OPEN_ITEM_INDEX));

        assert_eq!(model.move_left(), Some(MENU_FILE_INDEX));
        assert_eq!(model.move_left(), Some(MENU_VIEW_INDEX));
        assert_eq!(model.active_label(), Some("View"));
    }

    #[test]
    fn dropdown_navigation_skips_disabled_and_separators() {
        let mut model = sample_model();

        assert!(model.open());
        assert_eq!(model.nav.focused(), Some(OPEN_ITEM_INDEX));
        assert_eq!(model.move_down(), Some(SAVE_ITEM_INDEX));
        assert_eq!(model.move_down(), Some(RECENT_ITEM_INDEX));
        assert_eq!(model.move_down(), Some(OPEN_ITEM_INDEX));
        assert_eq!(model.move_up(), Some(RECENT_ITEM_INDEX));
    }

    #[test]
    fn submenu_enter_and_leave_tracks_open_path() {
        let mut model = sample_model();

        assert!(model.open());
        assert_eq!(model.move_down(), Some(SAVE_ITEM_INDEX));
        assert_eq!(model.move_down(), Some(RECENT_ITEM_INDEX));
        assert!(model.enter_submenu());
        assert_eq!(model.nav.submenu_path(), &[RECENT_ITEM_INDEX]);
        assert_eq!(model.nav.focused(), Some(SUBMENU_CHILD_INDEX));

        assert!(model.leave_submenu());
        assert_eq!(model.nav.submenu_path(), &[]);
        assert_eq!(model.nav.focused(), Some(RECENT_ITEM_INDEX));
    }

    #[test]
    fn activate_returns_selected_label_and_closes_menu() {
        let mut model = sample_model();

        assert!(model.open());
        assert_eq!(model.move_down(), Some(SAVE_ITEM_INDEX));

        let activated = model.activate();

        assert_eq!(activated.as_deref(), Some("Save"));
        assert!(!model.open);
        assert_eq!(model.nav.focused(), None);
        assert_eq!(model.nav.submenu_path(), &[]);
    }

    #[test]
    fn empty_menu_model_rejects_navigation_without_panicking() {
        let mut model = MenuBarModel::default();

        assert!(!model.open());
        assert_eq!(model.move_left(), None);
        assert_eq!(model.move_right(), None);
        assert_eq!(model.move_down(), None);
        assert_eq!(model.move_up(), None);
        assert_eq!(model.activate(), None);
        assert!(!model.enter_submenu());
        assert!(!model.leave_submenu());
    }

    #[test]
    fn dropdown_width_uses_content_width_and_minimum() {
        let items = vec![
            MenuItem::action_with_accel("Open", "Ctrl+O"),
            MenuItem::submenu("Recent", vec![MenuItem::action("one")]),
        ];

        assert_eq!(compute_dropdown_width(&items, DEFAULT_DROPDOWN_MIN_WIDTH), EXPECTED_WIDE_DROPDOWN_WIDTH);
        assert_eq!(compute_dropdown_width(&[], DEFAULT_DROPDOWN_MIN_WIDTH), DEFAULT_DROPDOWN_MIN_WIDTH);
    }

    #[test]
    fn menu_bar_layout_stops_at_available_width() {
        let model = sample_model();
        let area = Rect::new(0, 0, TOO_SMALL_WIDTH, MENU_BAR_RENDER_HEIGHT);

        let layout = compute_menu_bar_layout(area, &model);

        assert!(layout.labels.is_empty());
    }

    #[test]
    fn menu_bar_render_highlights_open_active_label() {
        let mut model = sample_model();
        model.open();
        let style = MenuBarStyle::default().with_open_label(Style::default().fg(Color::Red));

        let buffer = render_menu_bar_to_buffer(&model, &style);

        assert_eq!(buffer[(0, 0)].symbol(), " ");
        assert_eq!(buffer[(1, 0)].symbol(), "F");
        assert_eq!(buffer[(1, 0)].fg, Color::Red);

        let layout = compute_menu_bar_layout(Rect::new(0, 0, MENU_BAR_WIDTH, MENU_BAR_HEIGHT), &model);
        assert_eq!(layout.labels[MENU_FILE_INDEX].rect.width, EXPECTED_FILE_LABEL_WIDTH);
    }

    #[test]
    fn dropdown_render_shows_disabled_separator_selected_and_submenu_marker() {
        let items = vec![
            MenuItem::action("Open"),
            MenuItem::separator(),
            MenuItem::disabled_action("Disabled"),
            MenuItem::submenu("Recent", vec![MenuItem::action("one")]),
        ];
        let mut nav = MenuNav::new();
        assert!(nav.set_focus(&items, RENDER_SUBMENU_INDEX));
        let style = DropdownStyle::default()
            .with_disabled_item(Style::default().fg(Color::DarkGray))
            .with_selected_item(Style::default().fg(Color::Red))
            .with_separator(Style::default().fg(Color::Blue));

        let buffer = render_dropdown_to_buffer(&items, &nav, &style);

        assert_eq!(buffer[(DROPDOWN_ROW_X, DROPDOWN_SECOND_ROW_Y)].symbol(), DROPDOWN_SEPARATOR);
        assert_eq!(buffer[(DROPDOWN_ROW_X, DROPDOWN_SECOND_ROW_Y)].fg, Color::Blue);
        assert_eq!(buffer[(DROPDOWN_CONTENT_X, DROPDOWN_THIRD_ROW_Y)].symbol(), "D");
        assert_eq!(buffer[(DROPDOWN_CONTENT_X, DROPDOWN_THIRD_ROW_Y)].fg, Color::DarkGray);
        assert_eq!(buffer[(DROPDOWN_MARKER_X, DROPDOWN_FOURTH_ROW_Y)].symbol(), DROPDOWN_SUBMENU_MARKER);
        assert_eq!(buffer[(DROPDOWN_ROW_X, DROPDOWN_FOURTH_ROW_Y)].fg, Color::Red);
    }

    #[test]
    fn dropdown_area_clamps_to_viewport_right_edge() {
        let viewport = Rect::new(0, 0, VIEWPORT_WIDTH, VIEWPORT_HEIGHT);
        let anchor = Rect::new(ANCHOR_X, ANCHOR_Y, ANCHOR_WIDTH, ANCHOR_HEIGHT);
        let items = vec![MenuItem::action_with_accel("Open", "Ctrl+O")];

        let area = dropdown_area_below(viewport, anchor, &items, DEFAULT_DROPDOWN_MIN_WIDTH);

        assert_eq!(rect_right(area), VIEWPORT_WIDTH);
        assert_eq!(area.y, ANCHOR_HEIGHT);
    }

    fn sample_model() -> MenuBarModel {
        MenuBarModel::from_menus([
            ("File", vec![
                MenuItem::action("Open"),
                MenuItem::separator(),
                MenuItem::disabled_action("Disabled"),
                MenuItem::action("Save"),
                MenuItem::submenu("Recent", vec![MenuItem::action("one")]),
            ]),
            ("Edit", vec![MenuItem::action("Undo")]),
            ("View", vec![MenuItem::action("Zoom")]),
        ])
    }

    fn render_menu_bar_to_buffer(model: &MenuBarModel, style: &MenuBarStyle) -> Buffer {
        let backend = TestBackend::new(MENU_BAR_WIDTH, MENU_BAR_RENDER_HEIGHT);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                let area = frame.area();
                render_menu_bar(frame, area, model, style);
            })
            .unwrap();
        terminal.backend().buffer().clone()
    }

    fn render_dropdown_to_buffer(items: &[MenuItem], nav: &MenuNav, style: &DropdownStyle) -> Buffer {
        let backend = TestBackend::new(DROPDOWN_WIDTH, DROPDOWN_HEIGHT);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                let area = frame.area();
                render_dropdown(frame, area, items, nav, style);
            })
            .unwrap();
        terminal.backend().buffer().clone()
    }
}

//! Context menu model and floating popup renderer.
//!
//! Context menus share `MenuItem`, `MenuNav`, and dropdown rendering with the
//! menu bar while adding visibility state and viewport-aware anchor placement.

use ratatui::Frame;
use ratatui::layout::Rect;

use crate::DropdownLayout;
use crate::DropdownStyle;
use crate::MenuItem;
use crate::MenuNav;
use crate::compute_dropdown_width;
use crate::render_dropdown;

const DEFAULT_CONTEXT_MENU_MIN_WIDTH: u16 = 8;
const CONTEXT_MENU_BORDER_HEIGHT: u16 = 2;

/// Anchor position for a floating context menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ContextMenuAnchor {
    /// Requested x coordinate in terminal cells.
    pub x: u16,
    /// Requested y coordinate in terminal cells.
    pub y: u16,
}

impl ContextMenuAnchor {
    /// Create a context menu anchor.
    pub fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }
}

/// Pure data model for a context menu.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ContextMenuModel {
    /// Menu entries displayed in the root context menu.
    pub items: Vec<MenuItem>,
    /// Requested popup anchor position.
    pub anchor: ContextMenuAnchor,
    /// Whether the context menu should be rendered.
    pub visible: bool,
    /// Navigation state for the current menu/submenu level.
    pub nav: MenuNav,
}

impl ContextMenuModel {
    /// Create a hidden context menu with the supplied items.
    pub fn new(items: Vec<MenuItem>) -> Self {
        Self {
            items,
            anchor: ContextMenuAnchor::default(),
            visible: false,
            nav: MenuNav::new(),
        }
    }

    /// Set root menu items.
    pub fn with_items(mut self, items: Vec<MenuItem>) -> Self {
        self.items = items;
        self.nav.clear();
        self
    }

    /// Set the context menu anchor.
    pub fn with_anchor(mut self, anchor: ContextMenuAnchor) -> Self {
        self.anchor = anchor;
        self
    }

    /// Set the context menu visibility.
    pub fn with_visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        if visible {
            self.focus_root();
        } else {
            self.nav.clear();
        }
        self
    }

    /// Show the context menu at an anchor position and focus the first item.
    pub fn show(&mut self, anchor: ContextMenuAnchor) {
        self.anchor = anchor;
        self.visible = true;
        self.focus_root();
    }

    /// Hide the context menu and clear all navigation/submenu state.
    pub fn dismiss(&mut self) {
        self.visible = false;
        self.nav.clear();
    }

    /// Return the current item list after following submenu state.
    pub fn current_items(&self) -> Option<&[MenuItem]> {
        self.nav.current_items(self.items.as_slice())
    }

    /// Move focus down in the current menu level.
    pub fn move_down(&mut self) -> Option<usize> {
        if !self.visible {
            return None;
        }

        let items = self.items.as_slice();
        let current_items = self.nav.current_items(items).unwrap_or_default();
        self.nav.move_down(current_items)
    }

    /// Move focus up in the current menu level.
    pub fn move_up(&mut self) -> Option<usize> {
        if !self.visible {
            return None;
        }

        let items = self.items.as_slice();
        let current_items = self.nav.current_items(items).unwrap_or_default();
        self.nav.move_up(current_items)
    }

    /// Enter the focused submenu.
    pub fn enter_submenu(&mut self) -> bool {
        self.visible && self.nav.enter_submenu(self.items.as_slice())
    }

    /// Leave the current submenu.
    pub fn leave_submenu(&mut self) -> bool {
        self.visible && self.nav.leave_submenu()
    }

    /// Activate the focused item, returning its label and dismissing the menu.
    pub fn activate(&mut self) -> Option<String> {
        if !self.visible {
            return None;
        }

        let label = self.nav.current_item(self.items.as_slice()).and_then(MenuItem::label).map(str::to_string);

        if label.is_some() {
            self.dismiss();
        }

        label
    }

    fn focus_root(&mut self) {
        self.nav.clear();
        self.nav.focus_first(self.items.as_slice());
    }
}

/// Compute a viewport-clamped context menu area for the current menu level.
pub fn compute_context_menu_area(viewport: Rect, model: &ContextMenuModel, minimum_width: u16) -> Option<Rect> {
    if !model.visible || viewport.width == 0 || viewport.height == 0 {
        return None;
    }

    let items = model.current_items().unwrap_or_default();
    let width = compute_dropdown_width(items, minimum_width).min(viewport.width);
    let height = context_menu_height(items).min(viewport.height);
    let max_x = rect_right(viewport).saturating_sub(width).max(viewport.x);
    let max_y = rect_bottom(viewport).saturating_sub(height).max(viewport.y);
    let x = model.anchor.x.clamp(viewport.x, max_x);
    let y = model.anchor.y.clamp(viewport.y, max_y);

    Some(Rect::new(x, y, width, height))
}

/// Render a context menu when visible, returning its dropdown layout.
pub fn render_context_menu(
    frame: &mut Frame,
    viewport: Rect,
    model: &ContextMenuModel,
    style: &DropdownStyle,
) -> Option<DropdownLayout> {
    render_context_menu_with_min_width(frame, viewport, model, style, DEFAULT_CONTEXT_MENU_MIN_WIDTH)
}

/// Render a context menu using a caller-supplied minimum width.
pub fn render_context_menu_with_min_width(
    frame: &mut Frame,
    viewport: Rect,
    model: &ContextMenuModel,
    style: &DropdownStyle,
    minimum_width: u16,
) -> Option<DropdownLayout> {
    let area = compute_context_menu_area(viewport, model, minimum_width)?;
    let items = model.current_items().unwrap_or_default();
    Some(render_dropdown(frame, area, items, &model.nav, style))
}

fn context_menu_height(items: &[MenuItem]) -> u16 {
    usize_to_u16(items.len()).saturating_add(CONTEXT_MENU_BORDER_HEIGHT)
}

fn usize_to_u16(value: usize) -> u16 {
    value.min(usize::from(u16::MAX)) as u16
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

    const VIEWPORT_X: u16 = 0;
    const VIEWPORT_Y: u16 = 0;
    const VIEWPORT_WIDTH: u16 = 20;
    const VIEWPORT_HEIGHT: u16 = 8;
    const RIGHT_EDGE_ANCHOR_X: u16 = 18;
    const BOTTOM_EDGE_ANCHOR_Y: u16 = 7;
    const TOP_EDGE_ANCHOR_Y: u16 = 0;
    const DEFAULT_ANCHOR_X: u16 = 2;
    const DEFAULT_ANCHOR_Y: u16 = 2;
    const EXPECTED_OPEN_INDEX: usize = 0;
    const EXPECTED_SAVE_INDEX: usize = 2;
    const SUBMENU_INDEX: usize = 3;
    const SUBMENU_CHILD_INDEX: usize = 0;
    const TEST_MINIMUM_WIDTH: u16 = 12;
    const EXPECTED_RIGHT_CLAMP_X: u16 = VIEWPORT_WIDTH - TEST_MINIMUM_WIDTH;
    const EXPECTED_TWO_ITEM_HEIGHT: u16 = 4;
    const EXPECTED_BOTTOM_CLAMP_Y: u16 = VIEWPORT_HEIGHT - EXPECTED_TWO_ITEM_HEIGHT;
    const RENDER_WIDTH: u16 = 20;
    const RENDER_HEIGHT: u16 = 8;
    const RENDER_LABEL_X: u16 = 4;
    const RENDER_LABEL_Y: u16 = 3;

    #[test]
    fn show_focuses_first_selectable_item() {
        let mut model = ContextMenuModel::new(sample_items());

        model.show(ContextMenuAnchor::new(DEFAULT_ANCHOR_X, DEFAULT_ANCHOR_Y));

        assert!(model.visible);
        assert_eq!(model.anchor, ContextMenuAnchor::new(DEFAULT_ANCHOR_X, DEFAULT_ANCHOR_Y));
        assert_eq!(model.nav.focused(), Some(EXPECTED_OPEN_INDEX));
    }

    #[test]
    fn navigation_delegates_to_menu_nav_and_skips_unselectable_items() {
        let mut model = ContextMenuModel::new(sample_items()).with_visible(true);

        assert_eq!(model.move_down(), Some(EXPECTED_SAVE_INDEX));
        assert_eq!(model.move_down(), Some(SUBMENU_INDEX));
        assert_eq!(model.move_up(), Some(EXPECTED_SAVE_INDEX));
    }

    #[test]
    fn activate_returns_label_and_dismisses_menu() {
        let mut model = ContextMenuModel::new(sample_items()).with_visible(true);

        assert_eq!(model.move_down(), Some(EXPECTED_SAVE_INDEX));
        let activated = model.activate();

        assert_eq!(activated.as_deref(), Some("Save"));
        assert!(!model.visible);
        assert_eq!(model.nav.focused(), None);
    }

    #[test]
    fn invisible_context_menu_rejects_navigation_and_rendering() {
        let mut model = ContextMenuModel::new(sample_items());
        let viewport = test_viewport();

        assert_eq!(model.move_down(), None);
        assert_eq!(model.move_up(), None);
        assert_eq!(model.activate(), None);
        assert!(!model.enter_submenu());
        assert_eq!(compute_context_menu_area(viewport, &model, TEST_MINIMUM_WIDTH), None);
    }

    #[test]
    fn area_repositions_when_overflowing_right_edge() {
        let mut model = ContextMenuModel::new(short_items());
        model.show(ContextMenuAnchor::new(RIGHT_EDGE_ANCHOR_X, TOP_EDGE_ANCHOR_Y));

        let area = compute_context_menu_area(test_viewport(), &model, TEST_MINIMUM_WIDTH)
            .expect("visible context menu should compute an area");

        assert_eq!(area.x, EXPECTED_RIGHT_CLAMP_X);
        assert_eq!(rect_right(area), VIEWPORT_WIDTH);
    }

    #[test]
    fn area_repositions_when_overflowing_bottom_edge() {
        let mut model = ContextMenuModel::new(short_items());
        model.show(ContextMenuAnchor::new(DEFAULT_ANCHOR_X, BOTTOM_EDGE_ANCHOR_Y));

        let area = compute_context_menu_area(test_viewport(), &model, TEST_MINIMUM_WIDTH)
            .expect("visible context menu should compute an area");

        assert_eq!(area.y, EXPECTED_BOTTOM_CLAMP_Y);
        assert_eq!(rect_bottom(area), VIEWPORT_HEIGHT);
    }

    #[test]
    fn dismiss_clears_submenu_stack() {
        let mut model = ContextMenuModel::new(sample_items()).with_visible(true);

        assert_eq!(model.move_down(), Some(EXPECTED_SAVE_INDEX));
        assert_eq!(model.move_down(), Some(SUBMENU_INDEX));
        assert!(model.enter_submenu());
        assert_eq!(model.nav.submenu_path(), &[SUBMENU_INDEX]);
        assert_eq!(model.nav.focused(), Some(SUBMENU_CHILD_INDEX));

        model.dismiss();

        assert!(!model.visible);
        assert_eq!(model.nav.submenu_path(), &[]);
        assert_eq!(model.nav.focused(), None);
    }

    #[test]
    fn render_context_menu_uses_positioned_dropdown() {
        let mut model = ContextMenuModel::new(short_items());
        model.show(ContextMenuAnchor::new(DEFAULT_ANCHOR_X, DEFAULT_ANCHOR_Y));

        let buffer = render_to_buffer(&model);

        assert_eq!(buffer[(RENDER_LABEL_X, RENDER_LABEL_Y)].symbol(), "O");
    }

    fn sample_items() -> Vec<MenuItem> {
        vec![
            MenuItem::action("Open"),
            MenuItem::separator(),
            MenuItem::action("Save"),
            MenuItem::submenu("More", vec![MenuItem::action("Details")]),
        ]
    }

    fn short_items() -> Vec<MenuItem> {
        vec![MenuItem::action("Open"), MenuItem::action("Save")]
    }

    fn test_viewport() -> Rect {
        Rect::new(VIEWPORT_X, VIEWPORT_Y, VIEWPORT_WIDTH, VIEWPORT_HEIGHT)
    }

    fn render_to_buffer(model: &ContextMenuModel) -> ratatui::buffer::Buffer {
        let backend = TestBackend::new(RENDER_WIDTH, RENDER_HEIGHT);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                let viewport = frame.area();
                render_context_menu(frame, viewport, model, &DropdownStyle::default());
            })
            .unwrap();
        terminal.backend().buffer().clone()
    }
}

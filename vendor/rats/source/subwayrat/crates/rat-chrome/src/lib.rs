//! Structural chrome primitives for ratatui.
//!
//! `rat-chrome` provides pure state models plus thin rendering shells for
//! OS-style and IDE-style application chrome. Components compute layout data
//! first, render second, and return child/content `Rect`s when callers need to
//! draw their own body widgets.
//!
//! # Components
//!
//! - [`overlay`]: centered/anchored popup frames with optional backdrop dimming.
//! - [`title_bar`]: label alignment plus right-aligned indicator focus.
//! - [`menu_item`], [`menu_bar`], [`context_menu`]: shared menu items,
//!   keyboard navigation, top menu bars, dropdowns, and context popups.
//! - [`toolbar`]: button/toggle toolbars with overflow scrolling.
//! - [`status_bar`]: left/center/right non-interactive status segments.
//! - [`breadcrumb`]: path segments with active selection and left truncation.
//! - [`tooltip`]: wrapped floating help text with vertical flipping.
//! - [`split_pane`]: left/right or top/bottom child-region computation.
//! - [`dialog`]: modal frame composition returning a content rect.
//!
//! # Overlay frame
//!
//! ```rust,ignore
//! use rat_chrome::{OverlayAnchor, OverlayModel, OverlaySize, OverlayStyle, overlay_frame};
//! # use ratatui::{Frame, layout::Rect, widgets::Paragraph};
//! # fn draw(frame: &mut Frame, area: Rect) {
//! let model = OverlayModel::default()
//!     .with_anchor(OverlayAnchor::Center)
//!     .with_width(OverlaySize::Fixed(40))
//!     .with_height(OverlaySize::Fixed(8))
//!     .with_title(" Overlay ")
//!     .with_backdrop(true);
//! let layout = overlay_frame(frame, area, &model, &OverlayStyle::default());
//! frame.render_widget(Paragraph::new("Body content"), layout.inner);
//! # }
//! ```
//!
//! # Menu bar and context menu
//!
//! ```rust,ignore
//! use rat_chrome::{MenuBarModel, MenuItem, MenuBarStyle, render_menu_bar};
//! # use ratatui::{Frame, layout::Rect};
//! # fn draw(frame: &mut Frame, area: Rect) {
//! let mut menu = MenuBarModel::from_menus([("File", vec![MenuItem::action("Open")])]);
//! menu.open();
//! render_menu_bar(frame, area, &menu, &MenuBarStyle::default());
//! # }
//! ```
//!
//! # Split pane and dialog content
//!
//! ```rust,ignore
//! use rat_chrome::{DialogModel, DialogSize, DialogStyle, render_dialog_frame};
//! # use ratatui::{Frame, layout::Rect, widgets::Paragraph};
//! # fn draw(frame: &mut Frame, area: Rect) {
//! let dialog = DialogModel::new("Confirm", ["Ok", "Cancel"])
//!     .with_size(DialogSize::Fixed { width: 40, height: 10 })
//!     .with_dim(true);
//! let layout = render_dialog_frame(frame, area, &dialog, &DialogStyle::default());
//! frame.render_widget(Paragraph::new("Dialog body"), layout.content);
//! # }
//! ```
//!
//! Intended migration targets in this workspace include duplicated popup code
//! in `rat-widgets`, `rat-branches`, `rat-leaderkey`, `rat-streaming`, and
//! `rat-capture`. Deferred follow-ups: animation, hit testing, mouse routing,
//! and broader popup refactors.

pub mod breadcrumb;
pub mod context_menu;
pub mod dialog;
mod heading;
mod layer;
pub mod menu_bar;
pub mod menu_item;
pub mod overlay {
    pub use crate::layer::{
        OverlayAnchor, OverlayLayout, OverlayModel, OverlaySize, OverlayStyle, frame, layout,
        render_frame,
    };
}
mod partition;
mod segments;
pub mod split_pane {
    pub use crate::partition::{
        DividerPos, SplitOrientation, SplitPaneModel, SplitPaneStyle, SplitRegions,
        compute_split_regions, render_split_pane,
    };
}
pub mod status_bar {
    pub use crate::segments::{
        StatusBar, StatusBarLayout, compute_status_bar_layout, render_status_bar,
    };
}
pub mod title_bar {
    pub use crate::heading::{
        TitleAlignment, TitleBarIndicatorLayout, TitleBarLayout, TitleBarModel, TitleBarStyle,
        compute_title_bar_layout,
    };
    pub use crate::render_title_bar;
}
pub mod toolbar;
pub mod tooltip;

pub use breadcrumb::{
    BreadcrumbLayout, BreadcrumbModel, BreadcrumbSegmentLayout, BreadcrumbStyle,
    compute_breadcrumb_layout, render_breadcrumb,
};
pub use context_menu::{
    ContextMenuAnchor, ContextMenuModel, compute_context_menu_area, render_context_menu,
    render_context_menu_with_min_width,
};
pub use dialog::{
    DialogLayout, DialogModel, DialogSize, DialogStyle, compute_dialog_layout, render_dialog_frame,
};
pub use heading::{
    TitleAlignment, TitleBarIndicatorLayout, TitleBarLayout, TitleBarModel, TitleBarStyle,
    compute_title_bar_layout,
};
pub use layer::{OverlayAnchor, OverlayLayout, OverlayModel, OverlaySize, OverlayStyle};
pub use menu_bar::{
    DropdownLayout, DropdownRowLayout, DropdownStyle, MenuBarLayout, MenuBarModel, MenuBarStyle,
    MenuLabelLayout, compute_dropdown_layout, compute_dropdown_width, compute_menu_bar_layout,
    dropdown_area_below, render_dropdown, render_menu_bar,
};
pub use menu_item::{MenuItem, MenuNav};
pub use partition::{
    DividerPos, SplitOrientation, SplitPaneModel, SplitPaneStyle, SplitRegions,
    compute_split_regions, render_split_pane,
};
pub use segments::{StatusBar, StatusBarLayout, compute_status_bar_layout, render_status_bar};
pub use toolbar::{
    ToolbarActivation, ToolbarItem, ToolbarItemLayout, ToolbarLayout, ToolbarModel,
    ToolbarOrientation, ToolbarStyle, render_toolbar,
};
pub use tooltip::{
    Tooltip, TooltipAnchor, TooltipBorder, TooltipLayout, TooltipStyle, compute_tooltip_layout,
    render_tooltip,
};

impl Default for TitleBarStyle {
    fn default() -> Self {
        Self {
            background: ratatui::style::Style::new().bg(ratatui::style::Color::Blue),
            label: ratatui::style::Style::new()
                .fg(ratatui::style::Color::White)
                .add_modifier(ratatui::style::Modifier::BOLD),
            indicator: ratatui::style::Style::new().fg(ratatui::style::Color::Gray),
            focused_indicator: ratatui::style::Style::new()
                .fg(ratatui::style::Color::Yellow)
                .add_modifier(ratatui::style::Modifier::BOLD),
        }
    }
}

impl TitleBarStyle {
    /// Set the title bar background style.
    pub fn with_background(mut self, background: ratatui::style::Style) -> Self {
        self.background = background;
        self
    }

    /// Set the title label style.
    pub fn with_label(mut self, label: ratatui::style::Style) -> Self {
        self.label = label;
        self
    }

    /// Set the non-focused indicator style.
    pub fn with_indicator(mut self, indicator: ratatui::style::Style) -> Self {
        self.indicator = indicator;
        self
    }

    /// Set the focused indicator style.
    pub fn with_focused_indicator(mut self, focused_indicator: ratatui::style::Style) -> Self {
        self.focused_indicator = focused_indicator;
        self
    }
}

/// Render a title bar and return the computed layout.
pub fn render_title_bar(
    frame: &mut ratatui::Frame,
    area: ratatui::layout::Rect,
    model: &TitleBarModel,
    style: &TitleBarStyle,
) -> TitleBarLayout {
    let layout = compute_title_bar_layout(area, model);
    if area.width == 0 || area.height == 0 {
        return layout;
    }

    let buffer = frame.buffer_mut();
    fill_title_area(buffer, area, style.background);
    paint_title_label(buffer, &layout, style);
    paint_title_indicators(buffer, model, &layout, style);
    layout
}

fn fill_title_area(
    buffer: &mut ratatui::buffer::Buffer,
    area: ratatui::layout::Rect,
    style: ratatui::style::Style,
) {
    for y in area.y..title_bar_bottom(area) {
        for x in area.x..title_bar_right(area) {
            buffer[(x, y)].set_char(' ').set_style(style);
        }
    }
}

fn paint_title_label(
    buffer: &mut ratatui::buffer::Buffer,
    layout: &TitleBarLayout,
    style: &TitleBarStyle,
) {
    if layout.rendered_label.is_empty() {
        return;
    }
    buffer.set_string(
        layout.label.x,
        layout.label.y,
        layout.rendered_label.as_str(),
        style.label.patch(style.background),
    );
}

fn paint_title_indicators(
    buffer: &mut ratatui::buffer::Buffer,
    model: &TitleBarModel,
    layout: &TitleBarLayout,
    style: &TitleBarStyle,
) {
    for indicator in &layout.indicators {
        buffer.set_string(
            indicator.rect.x,
            indicator.rect.y,
            model.indicators[indicator.index].as_str(),
            title_indicator_style(model, indicator.index, style).patch(style.background),
        );
    }
}

fn title_indicator_style(
    model: &TitleBarModel,
    index: usize,
    style: &TitleBarStyle,
) -> ratatui::style::Style {
    if model.focused_indicator == Some(index) {
        return style.focused_indicator;
    }
    style.indicator
}

fn title_bar_right(rect: ratatui::layout::Rect) -> u16 {
    rect.x.saturating_add(rect.width)
}

fn title_bar_bottom(rect: ratatui::layout::Rect) -> u16 {
    rect.y.saturating_add(rect.height)
}

/// Compute the overlay layout for a viewport without rendering it.
pub fn compute_overlay_layout(
    viewport: ratatui::layout::Rect,
    model: &OverlayModel,
) -> OverlayLayout {
    layer::layout(viewport, model)
}

/// Render an overlay frame and return its computed layout.
pub fn overlay_frame(
    frame: &mut ratatui::Frame,
    viewport: ratatui::layout::Rect,
    model: &OverlayModel,
    style: &OverlayStyle,
) -> OverlayLayout {
    layer::frame(frame, viewport, model, style)
}

/// Render an overlay frame into a raw buffer and return its computed layout.
pub fn render_overlay_frame(
    buf: &mut ratatui::buffer::Buffer,
    viewport: ratatui::layout::Rect,
    model: &OverlayModel,
    style: &OverlayStyle,
) -> OverlayLayout {
    layer::render_frame(buf, viewport, model, style)
}

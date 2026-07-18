const DEFAULT_SPLIT_RATIO: f64 = 0.5;
const DIVIDER_THICKNESS: u16 = 1;
const DEFAULT_MIN_PANE_SIZE: u16 = 0;
const DEFAULT_DIVIDER_CHAR: char = '│';
const HORIZONTAL_DIVIDER_CHAR: char = '│';
const VERTICAL_DIVIDER_CHAR: char = '─';

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SplitOrientation {
    #[default]
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DividerPos {
    Ratio(f64),
    FromStart(u16),
    FromEnd(u16),
}

impl Default for DividerPos {
    fn default() -> Self {
        Self::Ratio(DEFAULT_SPLIT_RATIO)
    }
}

/// Pure split pane state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SplitPaneModel {
    pub divider: DividerPos,
    pub orientation: SplitOrientation,
    pub min_first: u16,
    pub min_second: u16,
}

impl Default for SplitPaneModel {
    fn default() -> Self {
        Self {
            divider: DividerPos::default(),
            orientation: SplitOrientation::Horizontal,
            min_first: DEFAULT_MIN_PANE_SIZE,
            min_second: DEFAULT_MIN_PANE_SIZE,
        }
    }
}

impl SplitPaneModel {
    /// Create a split pane with a divider position.
    pub fn new(divider: DividerPos) -> Self {
        Self {
            divider,
            ..Self::default()
        }
    }

    /// Set divider position.
    pub fn with_divider(mut self, divider: DividerPos) -> Self {
        self.divider = divider;
        self
    }

    /// Set orientation.
    pub fn with_orientation(mut self, orientation: SplitOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Set minimum pane sizes.
    pub fn with_min_sizes(mut self, min_first: u16, min_second: u16) -> Self {
        self.min_first = min_first;
        self.min_second = min_second;
        self
    }

    /// Move the divider by `delta` cells within `total_len`, returning the new offset.
    pub fn move_divider(&mut self, delta: i16, total_len: u16) -> u16 {
        let current = resolve_divider_offset(total_len, self);
        let moved = if delta.is_negative() {
            current.saturating_sub(delta.unsigned_abs())
        } else {
            current.saturating_add(delta as u16)
        };
        let bounds = ClampBounds::new(pane_space(total_len), self.min_first, self.min_second);
        let clamped = clamp_divider_offset(moved, bounds);
        self.divider = DividerPos::FromStart(clamped);
        clamped
    }
}

/// Visual style for split pane dividers.
#[derive(Debug, Clone, Copy)]
pub struct SplitPaneStyle {
    pub divider_char: char,
    pub divider: ratatui::style::Style,
}

impl Default for SplitPaneStyle {
    fn default() -> Self {
        Self {
            divider_char: DEFAULT_DIVIDER_CHAR,
            divider: ratatui::style::Style::new().fg(ratatui::style::Color::Gray),
        }
    }
}

impl SplitPaneStyle {
    /// Create a style with the conventional divider character for an orientation.
    pub fn default_for_orientation(orientation: SplitOrientation) -> Self {
        Self {
            divider_char: match orientation {
                SplitOrientation::Horizontal => HORIZONTAL_DIVIDER_CHAR,
                SplitOrientation::Vertical => VERTICAL_DIVIDER_CHAR,
            },
            ..Self::default()
        }
    }

    /// Set divider character.
    pub fn with_divider_char(mut self, divider_char: char) -> Self {
        self.divider_char = divider_char;
        self
    }

    /// Set divider style.
    pub fn with_divider(mut self, divider: ratatui::style::Style) -> Self {
        self.divider = divider;
        self
    }
}

/// Computed split pane regions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SplitRegions {
    pub first: ratatui::layout::Rect,
    pub divider: ratatui::layout::Rect,
    pub second: ratatui::layout::Rect,
}

/// Compute split pane regions without rendering.
pub fn compute_split_regions(area: ratatui::layout::Rect, model: &SplitPaneModel) -> SplitRegions {
    if area.width == 0 || area.height == 0 {
        return SplitRegions::default();
    }

    let spans = split_lengths(area, model);
    match model.orientation {
        SplitOrientation::Horizontal => horizontal_regions(area, spans),
        SplitOrientation::Vertical => vertical_regions(area, spans),
    }
}

/// Render a split pane divider and return child regions.
pub fn render_split_pane(
    frame: &mut ratatui::Frame,
    area: ratatui::layout::Rect,
    model: &SplitPaneModel,
    style: &SplitPaneStyle,
) -> SplitRegions {
    let regions = compute_split_regions(area, model);
    if regions.divider.width == 0 || regions.divider.height == 0 {
        return regions;
    }

    paint_divider(frame.buffer_mut(), regions.divider, style);
    regions
}

#[derive(Debug, Clone, Copy)]
struct SplitLengths {
    first: u16,
    second: u16,
}

#[derive(Debug, Clone, Copy)]
struct ClampBounds {
    content_len: u16,
    min_first: u16,
    min_second: u16,
}

impl ClampBounds {
    fn new(content_len: u16, min_first: u16, min_second: u16) -> Self {
        Self {
            content_len,
            min_first,
            min_second,
        }
    }
}

fn split_lengths(area: ratatui::layout::Rect, model: &SplitPaneModel) -> SplitLengths {
    let total_len = primary_len(area, model.orientation);
    let content_len = pane_space(total_len);
    let first = resolve_divider_offset(total_len, model);
    SplitLengths {
        first,
        second: content_len.saturating_sub(first),
    }
}

fn horizontal_regions(area: ratatui::layout::Rect, lengths: SplitLengths) -> SplitRegions {
    let divider_x = area.x.saturating_add(lengths.first);
    SplitRegions {
        first: ratatui::layout::Rect::new(area.x, area.y, lengths.first, area.height),
        divider: ratatui::layout::Rect::new(
            divider_x,
            area.y,
            DIVIDER_THICKNESS.min(area.width),
            area.height,
        ),
        second: ratatui::layout::Rect::new(
            divider_x.saturating_add(DIVIDER_THICKNESS),
            area.y,
            lengths.second,
            area.height,
        ),
    }
}

fn vertical_regions(area: ratatui::layout::Rect, lengths: SplitLengths) -> SplitRegions {
    let divider_y = area.y.saturating_add(lengths.first);
    SplitRegions {
        first: ratatui::layout::Rect::new(area.x, area.y, area.width, lengths.first),
        divider: ratatui::layout::Rect::new(
            area.x,
            divider_y,
            area.width,
            DIVIDER_THICKNESS.min(area.height),
        ),
        second: ratatui::layout::Rect::new(
            area.x,
            divider_y.saturating_add(DIVIDER_THICKNESS),
            area.width,
            lengths.second,
        ),
    }
}

fn resolve_divider_offset(total_len: u16, model: &SplitPaneModel) -> u16 {
    let content_len = pane_space(total_len);
    let raw = match model.divider {
        DividerPos::Ratio(ratio) => {
            let ratio = ratio.clamp(0.0, 1.0);
            (f64::from(content_len) * ratio).round() as u16
        }
        DividerPos::FromStart(offset) => offset.min(content_len),
        DividerPos::FromEnd(offset) => content_len.saturating_sub(offset.min(content_len)),
    };
    let bounds = ClampBounds::new(content_len, model.min_first, model.min_second);
    clamp_divider_offset(raw, bounds)
}

fn clamp_divider_offset(raw: u16, bounds: ClampBounds) -> u16 {
    if bounds.min_first.saturating_add(bounds.min_second) > bounds.content_len {
        return bounds.min_first.min(bounds.content_len);
    }

    raw.clamp(
        bounds.min_first,
        bounds.content_len.saturating_sub(bounds.min_second),
    )
}

fn pane_space(total_len: u16) -> u16 {
    total_len.saturating_sub(DIVIDER_THICKNESS)
}

fn primary_len(area: ratatui::layout::Rect, orientation: SplitOrientation) -> u16 {
    match orientation {
        SplitOrientation::Horizontal => area.width,
        SplitOrientation::Vertical => area.height,
    }
}

fn paint_divider(
    buffer: &mut ratatui::buffer::Buffer,
    divider: ratatui::layout::Rect,
    style: &SplitPaneStyle,
) {
    for y in divider.y..rect_bottom(divider) {
        for x in divider.x..rect_right(divider) {
            buffer[(x, y)]
                .set_char(style.divider_char)
                .set_style(style.divider);
        }
    }
}

fn rect_right(rect: ratatui::layout::Rect) -> u16 {
    rect.x.saturating_add(rect.width)
}

fn rect_bottom(rect: ratatui::layout::Rect) -> u16 {
    rect.y.saturating_add(rect.height)
}

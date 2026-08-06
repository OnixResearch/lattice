const OVERLAY_PERCENT_MAX: u16 = 100;
const CENTER_DIVISOR: u16 = 2;
const EMPTY_EXTENT: u16 = 0;
const EMPTY_EXTENT_U32: u32 = 0;

#[derive(Debug, Clone, Copy)]
struct PlacementExtent {
    width: u16,
    height: u16,
}

#[derive(Debug, Clone, Copy)]
struct PlacementOrigin {
    x: u16,
    y: u16,
}

#[derive(Debug, Clone, Copy)]
struct PercentCells {
    available_cells: u16,
    percent: u16,
}

#[derive(Debug, Clone, Copy)]
struct CenterSpan {
    available_cells: u16,
    extent_cells: u16,
}

#[derive(Debug, Clone, Copy)]
struct AxisBounds {
    min_cell: i32,
    max_cell: i32,
}

fn resolve_size(size: super::OverlaySize, available_cells: u16) -> u16 {
    match size {
        super::OverlaySize::Fixed(size_cells) => size_cells.min(available_cells),
        super::OverlaySize::Percent(percent) => resolve_percent_cells(PercentCells {
            available_cells,
            percent,
        }),
    }
}

pub(super) fn compute_outer_rect(
    viewport: ratatui::layout::Rect,
    model: &super::OverlayModel,
) -> ratatui::layout::Rect {
    if viewport.width == 0 || viewport.height == 0 {
        return ratatui::layout::Rect::new(viewport.x, viewport.y, EMPTY_EXTENT, EMPTY_EXTENT);
    }

    let extent = PlacementExtent {
        width: resolve_size(model.width, viewport.width),
        height: resolve_size(model.height, viewport.height),
    };
    let origin = anchored_origin(viewport, extent, model);
    ratatui::layout::Rect::new(origin.x, origin.y, extent.width, extent.height)
}

fn resolve_percent_cells(request: PercentCells) -> u16 {
    let bounded_percent = request.percent.min(OVERLAY_PERCENT_MAX);
    let divisor = u32::from(OVERLAY_PERCENT_MAX);
    assert!(divisor != EMPTY_EXTENT_U32);
    let scaled_cells = u32::from(request.available_cells)
        .saturating_mul(u32::from(bounded_percent))
        .checked_div(divisor)
        .unwrap_or(EMPTY_EXTENT_U32);
    u16::try_from(scaled_cells).unwrap_or(request.available_cells)
}

fn anchored_origin(
    viewport: ratatui::layout::Rect,
    extent: PlacementExtent,
    model: &super::OverlayModel,
) -> PlacementOrigin {
    PlacementOrigin {
        x: clamp_axis(anchor_x(viewport, extent, model.anchor), model.offset_x, horizontal_bounds(viewport, extent)),
        y: clamp_axis(anchor_y(viewport, extent, model.anchor), model.offset_y, vertical_bounds(viewport, extent)),
    }
}

fn horizontal_bounds(viewport: ratatui::layout::Rect, extent: PlacementExtent) -> AxisBounds {
    AxisBounds {
        min_cell: i32::from(viewport.x),
        max_cell: i32::from(rect_right(viewport).saturating_sub(extent.width)),
    }
}

fn vertical_bounds(viewport: ratatui::layout::Rect, extent: PlacementExtent) -> AxisBounds {
    AxisBounds {
        min_cell: i32::from(viewport.y),
        max_cell: i32::from(rect_bottom(viewport).saturating_sub(extent.height)),
    }
}

fn anchor_x(viewport: ratatui::layout::Rect, extent: PlacementExtent, anchor: super::OverlayAnchor) -> i32 {
    match anchor {
        super::OverlayAnchor::Center | super::OverlayAnchor::Top | super::OverlayAnchor::Bottom => {
            i32::from(viewport.x.saturating_add(center_offset(CenterSpan {
                available_cells: viewport.width,
                extent_cells: extent.width,
            })))
        }
        super::OverlayAnchor::TopRight | super::OverlayAnchor::Right | super::OverlayAnchor::BottomRight => {
            i32::from(rect_right(viewport).saturating_sub(extent.width))
        }
        super::OverlayAnchor::BottomLeft | super::OverlayAnchor::Left | super::OverlayAnchor::TopLeft => {
            i32::from(viewport.x)
        }
    }
}

fn anchor_y(viewport: ratatui::layout::Rect, extent: PlacementExtent, anchor: super::OverlayAnchor) -> i32 {
    match anchor {
        super::OverlayAnchor::Center | super::OverlayAnchor::Left | super::OverlayAnchor::Right => {
            i32::from(viewport.y.saturating_add(center_offset(CenterSpan {
                available_cells: viewport.height,
                extent_cells: extent.height,
            })))
        }
        super::OverlayAnchor::Bottom | super::OverlayAnchor::BottomLeft | super::OverlayAnchor::BottomRight => {
            i32::from(rect_bottom(viewport).saturating_sub(extent.height))
        }
        super::OverlayAnchor::Top | super::OverlayAnchor::TopLeft | super::OverlayAnchor::TopRight => {
            i32::from(viewport.y)
        }
    }
}

fn center_offset(span: CenterSpan) -> u16 {
    span.available_cells
        .saturating_sub(span.extent_cells)
        .checked_div(CENTER_DIVISOR)
        .unwrap_or(EMPTY_EXTENT)
}

fn clamp_axis(base_cell: i32, offset_cells: i16, bounds: AxisBounds) -> u16 {
    let bounded = base_cell
        .saturating_add(i32::from(offset_cells))
        .clamp(bounds.min_cell, bounds.max_cell.max(bounds.min_cell));
    u16::try_from(bounded).unwrap_or(EMPTY_EXTENT)
}

fn rect_right(rect: ratatui::layout::Rect) -> u16 {
    rect.x.saturating_add(rect.width)
}

fn rect_bottom(rect: ratatui::layout::Rect) -> u16 {
    rect.y.saturating_add(rect.height)
}

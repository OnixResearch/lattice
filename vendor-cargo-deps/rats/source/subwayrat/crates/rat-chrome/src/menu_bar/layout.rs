//! Pure menu dropdown geometry helpers.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct DropdownRectRequest {
    pub viewport: ratatui::layout::Rect,
    pub anchor: ratatui::layout::Rect,
    pub requested_width: u16,
    pub requested_height: u16,
}

pub(super) fn dropdown_rect(request: DropdownRectRequest) -> ratatui::layout::Rect {
    debug_assert!(request.requested_width <= request.viewport.width || request.viewport.width == 0);
    debug_assert!(request.requested_height <= request.viewport.height || request.viewport.height == 0);
    if request.viewport.width == 0 || request.viewport.height == 0 {
        return ratatui::layout::Rect::new(request.viewport.x, request.viewport.y, 0, 0);
    }

    let width = request.requested_width.min(request.viewport.width);
    let height = request.requested_height.min(request.viewport.height);
    let max_x = rect_right(request.viewport).saturating_sub(width);
    let x = request.anchor.x.clamp(request.viewport.x, max_x.max(request.viewport.x));
    let below_y = request.anchor.y.saturating_add(request.anchor.height);
    let max_y = rect_bottom(request.viewport).saturating_sub(height);
    let y = below_y.clamp(request.viewport.y, max_y.max(request.viewport.y));

    ratatui::layout::Rect::new(x, y, width, height)
}

fn rect_right(rect: ratatui::layout::Rect) -> u16 {
    rect.x.saturating_add(rect.width)
}

fn rect_bottom(rect: ratatui::layout::Rect) -> u16 {
    rect.y.saturating_add(rect.height)
}

#[cfg(test)]
mod tests {
    use super::*;

    const VIEWPORT_WIDTH: u16 = 30;
    const VIEWPORT_HEIGHT: u16 = 8;
    const ANCHOR_X_NEAR_RIGHT_EDGE: u16 = 28;
    const ANCHOR_WIDTH: u16 = 4;
    const ANCHOR_HEIGHT: u16 = 1;
    const REQUESTED_WIDTH: u16 = 12;
    const REQUESTED_HEIGHT: u16 = 6;

    #[test]
    fn clamps_dropdown_to_right_edge() {
        let viewport = ratatui::layout::Rect::new(0, 0, VIEWPORT_WIDTH, VIEWPORT_HEIGHT);
        let anchor = ratatui::layout::Rect::new(ANCHOR_X_NEAR_RIGHT_EDGE, 0, ANCHOR_WIDTH, ANCHOR_HEIGHT);

        let rect = dropdown_rect(DropdownRectRequest {
            viewport,
            anchor,
            requested_width: REQUESTED_WIDTH,
            requested_height: REQUESTED_HEIGHT,
        });

        assert_eq!(rect.x.saturating_add(rect.width), VIEWPORT_WIDTH);
        assert_eq!(rect.y, ANCHOR_HEIGHT);
    }

    #[test]
    fn rejects_empty_viewport_without_underflow() {
        let viewport = ratatui::layout::Rect::new(5, 7, 0, VIEWPORT_HEIGHT);
        let anchor = ratatui::layout::Rect::new(ANCHOR_X_NEAR_RIGHT_EDGE, 0, ANCHOR_WIDTH, ANCHOR_HEIGHT);

        let rect = dropdown_rect(DropdownRectRequest {
            viewport,
            anchor,
            requested_width: REQUESTED_WIDTH,
            requested_height: REQUESTED_HEIGHT,
        });

        assert_eq!(rect, ratatui::layout::Rect::new(5, 7, 0, 0));
    }
}

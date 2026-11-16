use super::*;

#[test]
fn distant_extreme_canvas_coordinates_are_not_snap_targets() {
    let cursor = CanvasPosition {
        x: i32::MIN,
        y: i32::MIN,
    };
    let marker = CanvasPosition {
        x: i32::MAX,
        y: i32::MAX,
    };
    assert_eq!(nearest_port_snap_target(None, cursor, marker, marker), None);
    assert_eq!(
        nearest_port_snap_target(None, marker, marker, marker),
        Some((0, PortSnapTarget { marker, hit: marker }))
    );
}

#[test]
fn port_positions_use_wide_intermediates_before_canvas_admission() {
    let origin = CanvasPosition { x: i32::MAX - 2, y: 0 };
    let target =
        port_target(origin, 3, 0, true).expect("final port coordinates fit despite the intermediate right edge");
    assert_eq!(target.marker, CanvasPosition {
        x: i32::MAX - 1,
        y: PORT_MARKER_ROW_FROM_NODE_TOP
    });
    assert_eq!(target.hit, CanvasPosition {
        x: i32::MAX,
        y: PORT_HIT_ROW_FROM_NODE_TOP
    });
    assert_eq!(port_target(CanvasPosition { x: i32::MAX, y: 0 }, 3, 0, true), None);
    assert_eq!(port_target(CanvasPosition { x: 0, y: i32::MAX }, 3, 0, false), None);
    assert_eq!(port_target(CanvasPosition { x: 0, y: 0 }, 3, usize::MAX, false), None);
}

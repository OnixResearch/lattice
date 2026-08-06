use super::*;

const EMPTY_COUNT: u32 = 0;
const SINGLE_COUNT: u32 = 1;
const NORMAL_COUNT: u32 = 10;
const SHRUNK_COUNT: u32 = 3;
const VIEWPORT_SMALL: u32 = 4;
const VIEWPORT_LARGE: u32 = 20;
const START_INDEX: u32 = 0;
const SECOND_INDEX: u32 = 1;
const THIRD_INDEX: u32 = 2;
const FIFTH_INDEX: u32 = 4;
const SIXTH_INDEX: u32 = 5;
const LAST_NORMAL_INDEX: u32 = 9;
const OUT_OF_BOUNDS_INDEX: u32 = 99;

#[test]
fn visible_range_handles_empty_single_normal_and_oversized() {
    let empty = ViewportState::new(EMPTY_COUNT, VIEWPORT_SMALL).unwrap();
    assert_eq!(empty.visible_range(), VisibleRange::new(0, 0));
    let single = ViewportState::new(SINGLE_COUNT, VIEWPORT_SMALL).unwrap();
    assert_eq!(single.visible_range(), VisibleRange::new(0, 1));
    let normal = ViewportState::new(NORMAL_COUNT, VIEWPORT_SMALL).unwrap();
    assert_eq!(normal.visible_range(), VisibleRange::new(0, VIEWPORT_SMALL));
    let too_large = ViewportState::new(NORMAL_COUNT, VIEWPORT_LARGE).unwrap();
    assert_eq!(too_large.visible_range(), VisibleRange::new(0, NORMAL_COUNT));
}

#[test]
fn navigation_clamps_and_keeps_cursor_visible() {
    let state = ViewportState::new(NORMAL_COUNT, VIEWPORT_SMALL).unwrap().move_cursor(Movement::Previous);
    assert_eq!(state.cursor(), START_INDEX);
    assert_eq!(state.scroll_offset(), START_INDEX);
    let state = state.move_cursor(Movement::PageDown);
    assert_eq!(state.cursor(), FIFTH_INDEX);
    assert_eq!(state.visible_range(), VisibleRange::new(SECOND_INDEX, SIXTH_INDEX));
    let state = state.move_cursor(Movement::Last).move_cursor(Movement::Next);
    assert_eq!(state.cursor(), LAST_NORMAL_INDEX);
    assert!(state.visible_range().contains(LAST_NORMAL_INDEX));
}

#[test]
fn selection_extends_and_clamps_after_shrink() {
    let state = ViewportState::new(NORMAL_COUNT, VIEWPORT_SMALL)
        .unwrap()
        .move_cursor(Movement::Delta(THIRD_INDEX.try_into().unwrap()))
        .begin_selection()
        .extend_selection(Movement::PageDown);
    assert_eq!(state.selection().unwrap().normalized(), VisibleRange::new(THIRD_INDEX, 7));
    let shrunk = state.clamp_to_item_count(SHRUNK_COUNT);
    assert_eq!(shrunk.cursor(), THIRD_INDEX);
    assert_eq!(shrunk.selection().unwrap().normalized(), VisibleRange::new(THIRD_INDEX, SHRUNK_COUNT));
}

#[test]
fn malformed_ranges_and_geometry_are_rejected() {
    assert_eq!(ViewportState::new(NORMAL_COUNT, 0), Err(ViewportError::ZeroViewportExtent));
    assert_eq!(
        ViewportState::from_parts(ViewportParts {
            item_count: NORMAL_COUNT,
            viewport_extent: VIEWPORT_SMALL,
            cursor: OUT_OF_BOUNDS_INDEX,
            scroll_offset: START_INDEX,
            anchor: None,
            selection: None,
        }),
        Err(ViewportError::CursorOutOfBounds {
            index: OUT_OF_BOUNDS_INDEX,
        })
    );
    assert_eq!(
        SelectionRange::new(START_INDEX, OUT_OF_BOUNDS_INDEX, NORMAL_COUNT),
        Err(ViewportError::SelectionOutOfBounds {
            index: OUT_OF_BOUNDS_INDEX,
        })
    );
}

#[test]
fn disabled_navigation_fails_without_mutating_state() {
    let state = ViewportState::new(NORMAL_COUNT, VIEWPORT_SMALL).unwrap();
    assert_eq!(state.apply_navigation(NavigationRequest::Disabled), Err(ViewportError::DisabledMovement));
}

#[test]
fn tui_and_web_adapter_projection_share_logic() {
    let selection = SelectionRange::new(SECOND_INDEX, THIRD_INDEX, NORMAL_COUNT).unwrap();
    let tui = project_viewport(ViewportAdapterInput {
        renderer: RendererKind::Tui,
        widget: WidgetViewportKind::List,
        item_count: NORMAL_COUNT,
        viewport_extent: VIEWPORT_SMALL,
        cursor: FIFTH_INDEX,
        scroll_offset: START_INDEX,
        anchor: Some(SECOND_INDEX),
        selection: Some(selection),
    })
    .unwrap();
    let web = project_viewport(ViewportAdapterInput {
        renderer: RendererKind::Web,
        widget: WidgetViewportKind::List,
        item_count: NORMAL_COUNT,
        viewport_extent: VIEWPORT_SMALL,
        cursor: FIFTH_INDEX,
        scroll_offset: START_INDEX,
        anchor: Some(SECOND_INDEX),
        selection: Some(selection),
    })
    .unwrap();
    assert_eq!(tui.cursor, web.cursor);
    assert_eq!(tui.scroll_offset, web.scroll_offset);
    assert_eq!(tui.visible, web.visible);
    assert_eq!(tui.selection, web.selection);
    assert_eq!(tui.visible, VisibleRange::new(SECOND_INDEX, SIXTH_INDEX));
}

#[test]
fn uniform_viewport_compatibility_receipt_is_stable() {
    let empty = ViewportState::new(EMPTY_COUNT, VIEWPORT_SMALL).unwrap();
    let single = ViewportState::new(SINGLE_COUNT, VIEWPORT_SMALL).unwrap();
    let normal = ViewportState::new(NORMAL_COUNT, VIEWPORT_SMALL).unwrap();
    let paged = normal.clone().move_cursor(Movement::PageDown);
    let canonical = format!(
        "empty={}:{}\nsingle={}:{}\nnormal={}:{}\npaged={}:{}:cursor={}\n",
        empty.visible_range().start,
        empty.visible_range().end,
        single.visible_range().start,
        single.visible_range().end,
        normal.visible_range().start,
        normal.visible_range().end,
        paged.visible_range().start,
        paged.visible_range().end,
        paged.cursor(),
    );
    assert_eq!(canonical.lines().count(), 4);
    assert_eq!(
        crate::interaction_replay::blake3_hex(canonical.as_bytes()),
        "10af8f9a08e0a4edbb0e02594f60fd1974ff488091cace4a6b06439ff854a28f"
    );
}

#[test]
fn zero_sized_adapter_input_is_rejected() {
    let result = project_viewport(ViewportAdapterInput {
        renderer: RendererKind::Tui,
        widget: WidgetViewportKind::CommandPalette,
        item_count: NORMAL_COUNT,
        viewport_extent: 0,
        cursor: START_INDEX,
        scroll_offset: START_INDEX,
        anchor: None,
        selection: None,
    });
    assert_eq!(result, Err(ViewportError::ZeroViewportExtent));
}

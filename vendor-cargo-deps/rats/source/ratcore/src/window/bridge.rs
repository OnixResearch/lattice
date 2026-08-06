use super::*;

/// External renderer/widget state used to build an adapter projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewportAdapterInput {
    /// Renderer consuming the projection.
    pub renderer: RendererKind,
    /// Widget family consuming the projection.
    pub widget: WidgetViewportKind,
    /// Number of logical items.
    pub item_count: u32,
    /// Logical viewport extent supplied by renderer-owned measurement.
    pub viewport_extent: u32,
    /// Cursor index supplied by widget state.
    pub cursor: u32,
    /// Scroll offset supplied by widget state.
    pub scroll_offset: u32,
    /// Optional selection anchor supplied by widget state.
    pub anchor: Option<u32>,
    /// Optional selection supplied by widget state.
    pub selection: Option<SelectionRange>,
}

impl ViewportAdapterInput {
    /// Builds validated pure viewport state from adapter input.
    ///
    /// # Errors
    ///
    /// Returns a [`ViewportError`] when the adapter input contains invalid
    /// geometry, cursor, scroll, anchor, or selection data.
    pub fn into_state(self) -> Result<ViewportState, ViewportError> {
        ViewportState::from_parts(ViewportParts {
            item_count: self.item_count,
            viewport_extent: self.viewport_extent,
            cursor: self.cursor,
            scroll_offset: self.scroll_offset,
            anchor: self.anchor,
            selection: self.selection,
        })
    }
}

/// Renderer-neutral projection consumed by widget adapters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewportAdapterProjection {
    /// Renderer consuming the projection.
    pub renderer: RendererKind,
    /// Widget family consuming the projection.
    pub widget: WidgetViewportKind,
    /// Cursor index after validation/navigation.
    pub cursor: u32,
    /// Scroll offset after validation/navigation.
    pub scroll_offset: u32,
    /// Visible item range after validation/navigation.
    pub visible: VisibleRange,
    /// Optional normalized selection range.
    pub selection: Option<VisibleRange>,
}

/// Builds a renderer-neutral viewport projection from external widget state.
///
/// # Errors
///
/// Returns a [`ViewportError`] when the adapter input contains invalid geometry,
/// cursor, scroll, anchor, or selection data.
pub fn project_viewport(input: ViewportAdapterInput) -> Result<ViewportAdapterProjection, ViewportError> {
    let renderer = input.renderer;
    let widget = input.widget;
    let state = input.into_state()?;
    Ok(ViewportAdapterProjection {
        renderer,
        widget,
        cursor: state.cursor(),
        scroll_offset: state.scroll_offset(),
        visible: state.visible_range(),
        selection: state.selection().map(SelectionRange::normalized),
    })
}

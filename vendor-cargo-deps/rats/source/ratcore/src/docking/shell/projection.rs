/// Shell model supplied by an application.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WindowPanelShellModel {
    /// Shell records to validate and project.
    pub panels: Vec<PanelShell>,
    /// Focused panel ID according to application state.
    pub focused_panel: Option<ShellPanelId>,
}

impl WindowPanelShellModel {
    /// Creates a shell model.
    #[must_use]
    pub fn new(panels: Vec<PanelShell>, focused_panel: Option<ShellPanelId>) -> Self {
        Self {
            panels,
            focused_panel,
        }
    }
}

/// Shell diagnostic code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellDiagnosticCode {
    /// Duplicate panel identifier.
    DuplicatePanelId,
    /// Focus references an absent panel.
    StaleFocus,
    /// A required title is empty.
    EmptyTitle,
    /// Hidden panel is marked active.
    HiddenActivePanel,
    /// Control cannot emit an action or typed intent.
    InvalidControlAction,
    /// Geometry was clamped for a tiny frame.
    TinyFrame,
    /// Header controls overflowed the available width.
    ControlOverflow,
}

/// Shell diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellDiagnostic {
    /// Machine-readable diagnostic code.
    pub code: ShellDiagnosticCode,
    /// Panel associated with the diagnostic.
    pub panel_id: Option<ShellPanelId>,
    /// Control associated with the diagnostic.
    pub control_id: Option<String>,
    /// Human-readable message.
    pub message: String,
}

impl ShellDiagnostic {
    /// Creates a diagnostic.
    #[must_use]
    pub fn new(
        code: ShellDiagnosticCode,
        panel_id: Option<ShellPanelId>,
        control_id: Option<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            panel_id,
            control_id,
            message: message.into(),
        }
    }
}

/// Safe renderable shell panel projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PanelShellProjection {
    /// Stable panel identifier.
    pub panel_id: ShellPanelId,
    /// Safe title after validation fallback.
    pub title: String,
    /// Whether the panel is active after validation fallback.
    pub active: bool,
    /// Z-order.
    pub z_order: i32,
    /// Whether the panel is visible.
    pub visible: bool,
    /// Controls to render.
    pub controls: Vec<ShellControl>,
    /// Frame geometry.
    pub geometry: ShellGeometry,
    /// Caller-owned content slot identifier.
    pub client_slot_id: String,
}

/// Validated shell projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowPanelShellProjection {
    /// Safe renderable panels.
    pub panels: Vec<PanelShellProjection>,
    /// Diagnostics recorded during validation.
    pub diagnostics: Vec<ShellDiagnostic>,
}

/// Shell geometry options.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShellFrameOptions {
    /// Border thickness on each edge.
    pub border_thickness: u16,
    /// Header height.
    pub header_height: u16,
    /// Width reserved for each control.
    pub control_width: u16,
    /// Gap between controls.
    pub control_gap: u16,
    /// Resize affordance thickness.
    pub resize_affordance: u16,
}

impl Default for ShellFrameOptions {
    fn default() -> Self {
        Self {
            border_thickness: DEFAULT_BORDER_THICKNESS,
            header_height: DEFAULT_HEADER_HEIGHT,
            control_width: DEFAULT_CONTROL_WIDTH,
            control_gap: DEFAULT_CONTROL_GAP,
            resize_affordance: DEFAULT_RESIZE_AFFORDANCE,
        }
    }
}

/// Geometry assigned to a shell control.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellControlRegion {
    /// Control identifier.
    pub control_id: String,
    /// Control rectangle.
    pub rect: ShellRect,
    /// Whether the region is visible after clamping.
    pub visible: bool,
}

/// Resize affordance geometry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellResizeRegions {
    /// Top resize strip.
    pub top: ShellRect,
    /// Bottom resize strip.
    pub bottom: ShellRect,
    /// Left resize strip.
    pub left: ShellRect,
    /// Right resize strip.
    pub right: ShellRect,
}

/// Computed shell geometry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellGeometry {
    /// Full frame.
    pub frame: ShellRect,
    /// Header area.
    pub header: ShellRect,
    /// Body area under the header.
    pub body: ShellRect,
    /// Caller-owned client content area.
    pub client: ShellRect,
    /// Control regions.
    pub controls: Vec<ShellControlRegion>,
    /// Resize affordance regions.
    pub resize_regions: ShellResizeRegions,
    /// Geometry diagnostics.
    pub diagnostics: Vec<ShellDiagnostic>,
}

/// Renderer family consuming shell projections.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellRenderer {
    /// Meteorite/Dioxus renderer.
    Meteorite,
    /// Subwayrat/ratatui renderer.
    Subwayrat,
}

/// Renderer-specific shell projection shell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellRendererProjection {
    /// Renderer that will consume the projection.
    pub renderer: ShellRenderer,
    /// Shared semantic projection.
    pub projection: WindowPanelShellProjection,
}

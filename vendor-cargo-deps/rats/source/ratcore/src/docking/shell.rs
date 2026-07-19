//! Renderer-neutral window and panel shell primitives.
//!
//! Docking owns logical panel placement. This module owns pure frame metadata,
//! validation, and geometry for desktop-style panel shells while renderers own
//! drawing, pointer binding, DOM nodes, and terminal buffers.

use std::collections::BTreeSet;

const DEFAULT_BORDER_THICKNESS: u16 = 1;
const DEFAULT_HEADER_HEIGHT: u16 = 1;
const DEFAULT_CONTROL_WIDTH: u16 = 3;
const DEFAULT_CONTROL_GAP: u16 = 1;
const DEFAULT_RESIZE_AFFORDANCE: u16 = 1;
const UNTITLED_PANEL: &str = "(untitled)";
const ZERO_DIMENSION: u16 = 0;

/// Stable shell panel identifier.
pub type ShellPanelId = String;

/// Stable action identifier used by shell controls.
pub type ShellActionId = String;

/// Rectangle in renderer-neutral shell coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShellRect {
    /// Left coordinate.
    pub x: u16,
    /// Top coordinate.
    pub y: u16,
    /// Width.
    pub width: u16,
    /// Height.
    pub height: u16,
}

impl ShellRect {
    /// Creates a rectangle.
    #[must_use]
    pub const fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Returns true when either dimension is zero.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.width == ZERO_DIMENSION || self.height == ZERO_DIMENSION
    }
}

/// Declarative window control kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShellControlKind {
    /// Close control.
    Close,
    /// Minimize control.
    Minimize,
    /// Maximize or restore control.
    Maximize,
    /// Pin control.
    Pin,
    /// Float control.
    Float,
    /// Dock control.
    Dock,
    /// Renderer/application specific control.
    Custom(String),
}

/// Declarative shell control.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellControl {
    /// Stable control identifier.
    pub id: String,
    /// Control kind.
    pub kind: ShellControlKind,
    /// Action or typed intent target.
    pub action_id: Option<ShellActionId>,
    /// Whether the control can be activated.
    pub enabled: bool,
    /// Optional diagnostic metadata.
    pub diagnostic: Option<String>,
}

impl ShellControl {
    /// Creates an enabled control with an action target.
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        kind: ShellControlKind,
        action_id: impl Into<ShellActionId>,
    ) -> Self {
        Self {
            id: id.into(),
            kind,
            action_id: Some(action_id.into()),
            enabled: true,
            diagnostic: None,
        }
    }

    /// Creates a control without an action target for validation tests or diagnostics.
    #[must_use]
    pub fn without_action(id: impl Into<String>, kind: ShellControlKind) -> Self {
        Self {
            id: id.into(),
            kind,
            action_id: None,
            enabled: true,
            diagnostic: None,
        }
    }

    /// Sets enablement.
    #[must_use]
    pub const fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Sets diagnostic metadata.
    #[must_use]
    pub fn with_diagnostic(mut self, diagnostic: impl Into<String>) -> Self {
        self.diagnostic = Some(diagnostic.into());
        self
    }
}

/// Renderer-neutral shell record for a panel or floating window.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PanelShell {
    /// Stable panel identifier.
    pub panel_id: ShellPanelId,
    /// Display title.
    pub title: String,
    /// Whether this panel is active.
    pub active: bool,
    /// Z-order supplied by the owning application.
    pub z_order: i32,
    /// Whether the shell should be visible.
    pub visible: bool,
    /// Window controls.
    pub controls: Vec<ShellControl>,
    /// Frame bounds supplied by layout or caller measurement.
    pub frame: ShellRect,
    /// Caller-owned content slot identifier.
    pub client_slot_id: String,
}

impl PanelShell {
    /// Creates a visible inactive panel shell.
    #[must_use]
    pub fn new(
        panel_id: impl Into<ShellPanelId>,
        title: impl Into<String>,
        frame: ShellRect,
    ) -> Self {
        let panel_id = panel_id.into();
        Self {
            client_slot_id: panel_id.clone(),
            panel_id,
            title: title.into(),
            active: false,
            z_order: 0,
            visible: true,
            controls: Vec::new(),
            frame,
        }
    }

    /// Sets active state.
    #[must_use]
    pub const fn with_active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    /// Sets z-order.
    #[must_use]
    pub const fn with_z_order(mut self, z_order: i32) -> Self {
        self.z_order = z_order;
        self
    }

    /// Sets visibility.
    #[must_use]
    pub const fn with_visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    /// Sets controls.
    #[must_use]
    pub fn with_controls(mut self, controls: Vec<ShellControl>) -> Self {
        self.controls = controls;
        self
    }

    /// Sets caller-owned client slot identifier.
    #[must_use]
    pub fn with_client_slot(mut self, client_slot_id: impl Into<String>) -> Self {
        self.client_slot_id = client_slot_id.into();
        self
    }
}

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

/// Validates a shell model and computes safe renderable geometry.
#[must_use]
pub fn validate_window_panel_shell_model(
    model: &WindowPanelShellModel,
) -> WindowPanelShellProjection {
    let mut diagnostics = Vec::new();
    let duplicate_ids = duplicate_panel_ids(&model.panels);
    for panel_id in &duplicate_ids {
        diagnostics.push(ShellDiagnostic::new(
            ShellDiagnosticCode::DuplicatePanelId,
            Some(panel_id.clone()),
            None,
            "duplicate panel id",
        ));
    }
    if let Some(focused_panel) = &model.focused_panel
        && !model
            .panels
            .iter()
            .any(|panel| panel.panel_id == *focused_panel)
    {
        diagnostics.push(ShellDiagnostic::new(
            ShellDiagnosticCode::StaleFocus,
            Some(focused_panel.clone()),
            None,
            "focused panel is absent",
        ));
    }

    let mut panels = Vec::with_capacity(model.panels.len());
    for panel in &model.panels {
        let mut panel_diagnostics = Vec::new();
        let title = safe_title(panel, &mut panel_diagnostics);
        let active = safe_active(panel, &mut panel_diagnostics);
        let controls = safe_controls(panel, &mut panel_diagnostics);
        let mut geometry = compute_window_panel_shell_geometry(
            panel.frame,
            &controls,
            ShellFrameOptions::default(),
            Some(panel.panel_id.clone()),
        );
        diagnostics.append(&mut panel_diagnostics);
        diagnostics.extend(geometry.diagnostics.clone());
        geometry.diagnostics.clear();
        panels.push(PanelShellProjection {
            panel_id: panel.panel_id.clone(),
            title,
            active,
            z_order: panel.z_order,
            visible: panel.visible,
            controls,
            geometry,
            client_slot_id: panel.client_slot_id.clone(),
        });
    }

    WindowPanelShellProjection {
        panels,
        diagnostics,
    }
}

/// Computes deterministic frame geometry for a shell.
#[must_use]
pub fn compute_window_panel_shell_geometry(
    frame: ShellRect,
    controls: &[ShellControl],
    options: ShellFrameOptions,
    panel_id: Option<ShellPanelId>,
) -> ShellGeometry {
    let header_height = options.header_height.min(frame.height);
    let header = ShellRect::new(frame.x, frame.y, frame.width, header_height);
    let body_y = frame.y.saturating_add(header_height);
    let body_height = frame.height.saturating_sub(header_height);
    let body = ShellRect::new(frame.x, body_y, frame.width, body_height);
    let edge_total = options.border_thickness.saturating_mul(2);
    let client_x = frame.x.saturating_add(options.border_thickness);
    let client_y = body.y.saturating_add(options.border_thickness);
    let client_width = body.width.saturating_sub(edge_total);
    let client_height = body.height.saturating_sub(options.border_thickness);
    let client = ShellRect::new(client_x, client_y, client_width, client_height);
    let mut diagnostics = Vec::new();
    if frame.is_empty() || client.is_empty() {
        diagnostics.push(ShellDiagnostic::new(
            ShellDiagnosticCode::TinyFrame,
            panel_id.clone(),
            None,
            "frame is too small for preferred shell geometry",
        ));
    }
    let control_regions =
        compute_control_regions(frame, controls, options, &panel_id, &mut diagnostics);
    let resize_regions = compute_resize_regions(frame, options.resize_affordance);
    ShellGeometry {
        frame,
        header,
        body,
        client,
        controls: control_regions,
        resize_regions,
        diagnostics,
    }
}

/// Wraps the shared shell projection for a renderer.
#[must_use]
pub fn project_window_panel_shell_renderer(
    renderer: ShellRenderer,
    model: &WindowPanelShellModel,
) -> ShellRendererProjection {
    ShellRendererProjection {
        renderer,
        projection: validate_window_panel_shell_model(model),
    }
}

fn duplicate_panel_ids(panels: &[PanelShell]) -> BTreeSet<ShellPanelId> {
    let mut seen = BTreeSet::new();
    let mut duplicates = BTreeSet::new();
    for panel in panels {
        if !seen.insert(panel.panel_id.clone()) {
            duplicates.insert(panel.panel_id.clone());
        }
    }
    duplicates
}

fn safe_title(panel: &PanelShell, diagnostics: &mut Vec<ShellDiagnostic>) -> String {
    if panel.title.trim().is_empty() {
        diagnostics.push(ShellDiagnostic::new(
            ShellDiagnosticCode::EmptyTitle,
            Some(panel.panel_id.clone()),
            None,
            "panel title is empty",
        ));
        return UNTITLED_PANEL.to_string();
    }
    panel.title.clone()
}

fn safe_active(panel: &PanelShell, diagnostics: &mut Vec<ShellDiagnostic>) -> bool {
    if panel.active && !panel.visible {
        diagnostics.push(ShellDiagnostic::new(
            ShellDiagnosticCode::HiddenActivePanel,
            Some(panel.panel_id.clone()),
            None,
            "hidden panel cannot be active",
        ));
        return false;
    }
    panel.active
}

fn safe_controls(panel: &PanelShell, diagnostics: &mut Vec<ShellDiagnostic>) -> Vec<ShellControl> {
    panel
        .controls
        .iter()
        .map(|control| {
            if control.action_id.is_none() {
                diagnostics.push(ShellDiagnostic::new(
                    ShellDiagnosticCode::InvalidControlAction,
                    Some(panel.panel_id.clone()),
                    Some(control.id.clone()),
                    "control has no action target",
                ));
                return control.clone().with_enabled(false);
            }
            control.clone()
        })
        .collect()
}

fn compute_control_regions(
    frame: ShellRect,
    controls: &[ShellControl],
    options: ShellFrameOptions,
    panel_id: &Option<ShellPanelId>,
    diagnostics: &mut Vec<ShellDiagnostic>,
) -> Vec<ShellControlRegion> {
    let mut regions = Vec::with_capacity(controls.len());
    let mut right_edge = frame.x.saturating_add(frame.width);
    for control in controls {
        let step = options.control_width.saturating_add(options.control_gap);
        right_edge = right_edge.saturating_sub(step);
        let rect = ShellRect::new(
            right_edge,
            frame.y,
            options.control_width.min(frame.width),
            options.header_height.min(frame.height),
        );
        let visible = rect.x >= frame.x && !rect.is_empty();
        if !visible {
            diagnostics.push(ShellDiagnostic::new(
                ShellDiagnosticCode::ControlOverflow,
                panel_id.clone(),
                Some(control.id.clone()),
                "control does not fit in header",
            ));
        }
        regions.push(ShellControlRegion {
            control_id: control.id.clone(),
            rect,
            visible,
        });
    }
    regions
}

fn compute_resize_regions(frame: ShellRect, thickness: u16) -> ShellResizeRegions {
    let right_x = frame
        .x
        .saturating_add(frame.width.saturating_sub(thickness));
    let bottom_y = frame
        .y
        .saturating_add(frame.height.saturating_sub(thickness));
    ShellResizeRegions {
        top: ShellRect::new(frame.x, frame.y, frame.width, thickness.min(frame.height)),
        bottom: ShellRect::new(frame.x, bottom_y, frame.width, thickness.min(frame.height)),
        left: ShellRect::new(frame.x, frame.y, thickness.min(frame.width), frame.height),
        right: ShellRect::new(right_x, frame.y, thickness.min(frame.width), frame.height),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FRAME_X: u16 = 2;
    const FRAME_Y: u16 = 3;
    const FRAME_WIDTH: u16 = 40;
    const FRAME_HEIGHT: u16 = 12;
    const TINY_WIDTH: u16 = 1;
    const TINY_HEIGHT: u16 = 1;

    fn frame() -> ShellRect {
        ShellRect::new(FRAME_X, FRAME_Y, FRAME_WIDTH, FRAME_HEIGHT)
    }

    fn close_control() -> ShellControl {
        ShellControl::new("close", ShellControlKind::Close, "panel.close")
    }

    #[test]
    fn validates_positive_shell_model_and_geometry() {
        let panel = PanelShell::new("editor", "Editor", frame())
            .with_active(true)
            .with_controls(vec![close_control()]);
        let model = WindowPanelShellModel::new(vec![panel], Some("editor".to_string()));

        let projection = validate_window_panel_shell_model(&model);

        assert!(projection.diagnostics.is_empty());
        assert_eq!(projection.panels[0].title, "Editor");
        assert_eq!(
            projection.panels[0].geometry.header.height,
            DEFAULT_HEADER_HEIGHT
        );
        assert!(!projection.panels[0].geometry.client.is_empty());
    }

    #[test]
    fn invalid_shell_state_reports_diagnostics_and_safe_fallbacks() {
        let invalid = PanelShell::new("editor", "", frame())
            .with_active(true)
            .with_visible(false)
            .with_controls(vec![ShellControl::without_action(
                "broken",
                ShellControlKind::Custom("broken".to_string()),
            )]);
        let duplicate = PanelShell::new("editor", "Duplicate", frame());
        let model =
            WindowPanelShellModel::new(vec![invalid, duplicate], Some("missing".to_string()));

        let projection = validate_window_panel_shell_model(&model);

        assert!(
            projection
                .diagnostics
                .iter()
                .any(|diagnostic| { diagnostic.code == ShellDiagnosticCode::DuplicatePanelId })
        );
        assert!(
            projection
                .diagnostics
                .iter()
                .any(|diagnostic| { diagnostic.code == ShellDiagnosticCode::StaleFocus })
        );
        assert!(
            projection
                .diagnostics
                .iter()
                .any(|diagnostic| { diagnostic.code == ShellDiagnosticCode::EmptyTitle })
        );
        assert!(
            projection
                .diagnostics
                .iter()
                .any(|diagnostic| { diagnostic.code == ShellDiagnosticCode::HiddenActivePanel })
        );
        assert!(
            projection
                .diagnostics
                .iter()
                .any(|diagnostic| { diagnostic.code == ShellDiagnosticCode::InvalidControlAction })
        );
        assert_eq!(projection.panels[0].title, UNTITLED_PANEL);
        assert!(!projection.panels[0].active);
        assert!(!projection.panels[0].controls[0].enabled);
    }

    #[test]
    fn tiny_and_control_overflow_geometry_fail_closed() {
        let tiny = ShellRect::new(FRAME_X, FRAME_Y, TINY_WIDTH, TINY_HEIGHT);
        let controls = vec![close_control(), close_control(), close_control()];

        let geometry = compute_window_panel_shell_geometry(
            tiny,
            &controls,
            ShellFrameOptions::default(),
            Some("tiny".to_string()),
        );

        assert!(geometry.client.is_empty());
        assert!(
            geometry
                .diagnostics
                .iter()
                .any(|diagnostic| { diagnostic.code == ShellDiagnosticCode::TinyFrame })
        );
        assert!(
            geometry
                .diagnostics
                .iter()
                .any(|diagnostic| { diagnostic.code == ShellDiagnosticCode::ControlOverflow })
        );
    }

    #[test]
    fn renderer_projections_share_shell_semantics() {
        let panel =
            PanelShell::new("editor", "Editor", frame()).with_controls(vec![close_control()]);
        let model = WindowPanelShellModel::new(vec![panel], None);

        let web = project_window_panel_shell_renderer(ShellRenderer::Meteorite, &model);
        let tui = project_window_panel_shell_renderer(ShellRenderer::Subwayrat, &model);

        assert_eq!(web.projection, tui.projection);
    }
}
